//use std::thread::spawn;
use std::sync::{Arc,Mutex,Condvar};
use std::sync::mpsc::{Sender,Receiver};
use std::os::unix::io::{RawFd,AsRawFd};

use libc::consts::os::bsd44::SOCK_STREAM;
use openssl::x509::X509StoreContext;
use openssl::crypto::pkey::PKey;
use openssl::ssl::{SslContext, SslMethod};
use openssl::ssl::error::SslError;
use openssl::ssl;
use openssl::crypto::hash::Type::SHA512;
use openssl::x509::{X509,X509Generator,KeyUsage,ExtKeyUsage};
use pseudotcp::PseudoTcpStream;

use ssl::SslChannel;
use utils::socket::ChannelToSocket;
use intercom::ConnectError;
use ice::IceAgent;
use utils::duplex_channel;

const CIPHERS:&'static str = concat!(
	"ECDHE-ECDSA-AES128-GCM-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
	"ECDHE-ECDSA-AES128-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
	"ECDHE-ECDSA-AES128-SHA,",   // won't work with DTLSv1 (but probably with v1.2)
	"ECDH-ECDSA-AES128-SHA");    // <- this one is probably used

pub struct Connection {
	agent:             IceAgent,
	socket_type:       i32,
	controlling_mode:  bool,
	local_private_key: PKey,
	remote_public_key: Arc<PKey>,
	local_credentials: Arc<(Mutex<Option<String>>, Condvar)>,
	tcp_stream:        Option<PseudoTcpStream>,
}

impl Connection {
	pub fn new(socket_type:       i32,
	           local_private_key: PKey,
	           remote_public_key: PKey,
	           controlling_mode:  bool)
		-> Result<Connection, ConnectError>
	{
		let err = "IceAgent::new() failed";
		let agent = try!(IceAgent::new(controlling_mode)
		                 .map_err(|_| ConnectError::Internal(err)));

		let mut conn = Connection {
			agent:             agent,
			socket_type:       socket_type,
			controlling_mode:  controlling_mode,
			local_private_key: local_private_key,
			remote_public_key: Arc::new(remote_public_key),
			local_credentials: Arc::new((Mutex::new(None),Condvar::new())),
			tcp_stream:        None,
		};

		// TODO: async
		{
			let credentials = conn.agent.get_local_credentials().unwrap();
			let credentials = String::from_utf8(credentials).unwrap();

			let &(ref lock, ref cvar) = &*conn.local_credentials;
			let mut var = lock.lock().unwrap();
			*var = Some(credentials);
			cvar.notify_all();
		}

		Ok(conn)
	}

	fn generate_cert(private_key: &PKey) -> Result<X509,SslError> {
		let usage = [ExtKeyUsage::ClientAuth, ExtKeyUsage::ServerAuth];

		let gen = X509Generator::new()
			.set_valid_period(1) // days
			.set_sign_hash(SHA512)
			.set_usage(&[KeyUsage::KeyAgreement])
			.set_ext_usage(&usage);

		let cert = gen.sign(&private_key);
		cert
	}

	pub fn get_local_credentials(&self) -> String {
		let &(ref lock, ref cvar) = &*self.local_credentials;
		let mut credentials = lock.lock().unwrap();
		while credentials.is_none() {
			credentials = cvar.wait(credentials).unwrap();
		}

		credentials.clone().unwrap()
	}

	pub fn establish_connection(&mut self, remote_credentials: Vec<u8>)
		-> Result<RawFd, ConnectError>
	{
		let (ciphertext_ch, ice_ch) = duplex_channel();
		
		let res = self.agent.stream_to_channel(&remote_credentials, ice_ch);
		try!(res.map_err(|_| ConnectError::IceConnectFailed));

		let plaintext_ch = try!(self.encrypt_connection(ciphertext_ch)
		                            .map_err(|e| ConnectError::SslError(e)));

		let proto = 0;
		let ch = if self.socket_type != SOCK_STREAM {
			plaintext_ch
		} else {
			let (plain_tx, plain_rx) = plaintext_ch;
			let ((stream_tx,stream_rx), socket_ch) = duplex_channel();

			let stream = PseudoTcpStream::new_from(plain_tx, plain_rx,
			                                       stream_tx, stream_rx);
			stream.set_mtu(1400);
			//stream.set_no_delay(true);

			if self.controlling_mode {
				let res = stream.connect();
				let err = "Could not establish reliable connection";
				try!(res.map_err(|_| ConnectError::Internal(err)));
			}

			self.tcp_stream = Some(stream);

			socket_ch
		};

		let sock = try!(ChannelToSocket::new_from(self.socket_type, proto, ch)
		                                .map_err(|e| ConnectError::IoError(e)));
		let fd = sock.as_raw_fd();

		debug!("SSL fd={}", fd);
		Ok(fd)
	}

	fn encrypt_connection(&self,
	                      ciphertext_ch: (Sender<Vec<u8>>, Receiver <Vec<u8>>))
		-> Result<(Sender<Vec<u8>>, Receiver <Vec<u8>>), SslError>
	{
		let is_server         = self.controlling_mode;
		let remote_public_key = self.remote_public_key.clone();

		let flags = ssl::SSL_VERIFY_PEER |
		            ssl::SSL_VERIFY_FAIL_IF_NO_PEER_CERT;

		let mut ctx = try!(SslContext::new(SslMethod::Dtlsv1));
		ctx.set_verify_with_data(flags,
		                         Self::verify_cert,
		                         remote_public_key);

		let cert = try!(Self::generate_cert(&self.local_private_key));

		try!(ctx.set_certificate(&cert));
		try!(ctx.set_private_key(&self.local_private_key));
		try!(ctx.check_private_key());
		try!(ctx.set_cipher_list(CIPHERS));

		let (my_plain_ch,your_plain_ch) = duplex_channel();

		let ssl = try!(SslChannel::new(&ctx,
		                               is_server,
		                               ciphertext_ch,
		                               my_plain_ch));
		drop(ssl);

		Ok(your_plain_ch)
	}

	#[allow(unused_variables)]
	fn verify_cert(preverify_ok: bool,
	               x509_ctx: &X509StoreContext,
		           expected_key: &Arc<PKey>)
		-> bool
	{
		info!("ssl x509 callback");

		match x509_ctx.get_current_cert() {
			None => return false,
			Some(cert) => {
				let actual_key = cert.public_key();

				if expected_key.public_eq(&actual_key) {
					return true;
				} else {
					warn!("Expected different public key!");
					return false;
				}
			}
		}
	}
}

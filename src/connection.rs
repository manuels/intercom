//use std::thread::spawn;
use std::sync::{Arc,Mutex,Condvar};
use std::sync::mpsc::{channel,Sender,Receiver};
use std::os::unix::io::{RawFd,AsRawFd};
use std::thread::spawn;

use libc::consts::os::bsd44::{SOCK_DGRAM, SOCK_STREAM};
use openssl::x509::X509StoreContext;
use openssl::crypto::pkey::PKey;
use openssl::ssl::{SslContext, SslMethod};
use openssl::ssl::error::SslError;
use openssl::ssl;
use openssl::crypto::hash::Type::SHA256;
use openssl::x509::{X509,X509Generator,KeyUsage,ExtKeyUsage};
use pseudotcp::PseudoTcpStream;

use ssl::SslChannel;
use utils::socket::ChannelToSocket;
use intercom::ConnectError;
use ice::IceAgent;

const CIPHERS:&'static str = concat!(
	"ECDHE-ECDSA-AES128-GCM-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
	"ECDHE-ECDSA-AES128-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
	"ECDHE-ECDSA-AES128-SHA,",   // won't work with DTLSv1 (but probably with v1.2)
	"ECDH-ECDSA-AES128-SHA");    // <- this one is probably used

pub struct Connection {
	local_credentials: Arc<(Mutex<Option<Vec<u8>>>, Condvar)>,
	agent:             IceAgent,
	local_private_key: PKey,
	remote_public_key: Arc<PKey>,
	controlling_mode:  bool,
	socket_type:       i32,
}

impl Connection {
	pub fn new(socket_type: i32, local_private_key: PKey, remote_public_key: PKey,
	           controlling_mode: bool)
		-> Result<Connection, ConnectError>
	{
		let agent = try!(IceAgent::new(controlling_mode)
		                 .map_err(|_| ConnectError::Internal("IceAgent::new() failed")));

		let mut conn = Connection {
			local_credentials: Arc::new((Mutex::new(None),Condvar::new())),
			agent:             agent,
			local_private_key: local_private_key,
			remote_public_key: Arc::new(remote_public_key),
			controlling_mode:  controlling_mode,
			socket_type:       socket_type,
		};

		// TODO: async
		{
			let credentials = conn.agent.get_local_credentials().unwrap();

			let &(ref lock, ref cvar) = &*conn.local_credentials;
			let mut var = lock.lock().unwrap();
			*var = Some(credentials);
			cvar.notify_all();
		}

		Ok(conn)
	}

	fn generate_cert(private_key: &PKey) -> Result<X509,SslError> {
		let gen = X509Generator::new()
			.set_valid_period(365*2)
			//.set_CN("test_me")
			.set_sign_hash(SHA256)
			.set_usage(&[KeyUsage::KeyAgreement])
			.set_ext_usage(&[ExtKeyUsage::ClientAuth, ExtKeyUsage::ServerAuth]);

		let cert = gen.sign(&private_key);
		cert
	}

	pub fn get_local_credentials(&self) -> Vec<u8> {
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
		let (cipher_tx, ice_rx) = channel();
		let (ice_tx, cipher_rx) = channel();
		
		if self.agent.stream_to_channel(&remote_credentials, ice_tx, ice_rx).is_err() {
			info!("stream_to_channel failed");
			return Err(ConnectError::IceConnectFailed);
		}

		let ciphertext_ch = (cipher_tx, cipher_rx);
		let plaintext_ch = match self.encrypt_connection(ciphertext_ch) {
			Ok(ch) => ch,
			Err(ssl_err) => return Err(ConnectError::SslError(ssl_err)),
		};

		let proto = 0;
		let fd = match self.socket_type {
			SOCK_DGRAM => {
				let sock = ChannelToSocket::new_from(SOCK_DGRAM, proto, plaintext_ch).unwrap();
				sock.as_raw_fd()
			},
			SOCK_STREAM => {
				let (stream_tx, socket_rx) = channel();
				let (socket_tx, stream_rx) = channel();
				let socket_ch = (socket_tx, socket_rx);

				let (plaintext_tx, plaintext_rx) = plaintext_ch;
				let stream = PseudoTcpStream::new_from(plaintext_tx, plaintext_rx,
					stream_tx, stream_rx);

				if self.controlling_mode {
					stream.connect();
				}

				spawn(move || {
					// TODO: keep stream better alive!
					loop {};
					drop(stream);
				});

				let sock = ChannelToSocket::new_from(SOCK_STREAM, proto, socket_ch).unwrap();
				sock.as_raw_fd()
			},
			_ => unimplemented!(),
		};
		debug!("SSL fd={}", fd);
		Ok(fd)
	}

	fn encrypt_connection(&self, ciphertext_ch: (Sender<Vec<u8>>, Receiver <Vec<u8>>))
		-> Result<(Sender<Vec<u8>>, Receiver <Vec<u8>>), SslError>
	{
		let flags = ssl::SSL_VERIFY_PEER | ssl::SSL_VERIFY_FAIL_IF_NO_PEER_CERT;
		let mut ctx = try!(SslContext::new(SslMethod::Dtlsv1));
		ctx.set_verify_with_data(flags, Self::verify_cert, self.remote_public_key.clone());

		let cert = try!(Self::generate_cert(&self.local_private_key));
		try!(ctx.set_certificate(&cert));
		try!(ctx.set_private_key(&self.local_private_key));
		try!(ctx.check_private_key());
		try!(ctx.set_cipher_list(CIPHERS));

		let (my_plain_tx, your_plain_rx) = channel();
		let (your_plain_tx, my_plain_rx) = channel();
		let my_plain_ch =  (my_plain_tx, my_plain_rx);
		let your_plain_ch =  (your_plain_tx, your_plain_rx);

		let is_server = self.controlling_mode;
		let ssl = try!(SslChannel::new(&ctx, is_server,
			ciphertext_ch, my_plain_ch));
		drop(ssl);

		Ok(your_plain_ch)
	}

	#[allow(unused_variables)]
	fn verify_cert(preverify_ok: bool, x509_ctx: &X509StoreContext,
		           expected_key: &Arc<PKey>) -> bool
	{
		info!("ssl x509 callback");

		match x509_ctx.get_current_cert() {
			None => false,
			Some(cert) => {
				let actual_key = cert.public_key();
				
				if actual_key.public_eq(expected_key) {
					true
				} else {
					warn!("Expected different public key!");
					false
				}
			}
		}
	}
}

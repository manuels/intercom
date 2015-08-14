use std::sync::Arc;
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
use pseudotcp::PseudoTcpSocket;

use ssl::SslChannel;
use utils::socket::ChannelToSocket;
use intercom::ConnectError;
pub use nice::ControllingMode;
use ice::IceConnection;

use utils::duplex_channel;

macro_rules! try_msg {
	($desc:expr, $expr:expr) => (match $expr {
		Result::Ok(val)  => val,
		Result::Err(err) => {
			let error = ConnectError {
				description: format!($desc),
				cause: Some(Box::new(err))
			};
			return Err(error);
		},
	});
	($desc:expr, $expr:expr, $val:expr) => (match $expr {
		Result::Ok(val)  => val,
		Result::Err(_) => {
			let error = ConnectError {
				description: format!($desc),
				cause: $val
			};
			return Err(error);
		},
	});
}

const CIPHERS:[&'static str; 12] = [
	"ECDHE-ECDSA-AES256-GCM-SHA386",// these won't work with DTLSv1 (but probably with v1.2)
	"ECDHE-ECDSA-AES256-GCM-SHA256",
	"ECDHE-ECDSA-AES128-GCM-SHA386",
	"ECDHE-ECDSA-AES128-GCM-SHA256",
	"ECDHE-ECDSA-AES256-SHA386",
	"ECDHE-ECDSA-AES256-SHA256",
	"ECDHE-ECDSA-AES128-SHA386",
	"ECDHE-ECDSA-AES128-SHA256",
	"ECDHE-ECDSA-AES256-SHA",
	"ECDHE-ECDSA-AES128-SHA",
	"ECDH-ECDSA-AES256-SHA",
	"ECDH-ECDSA-AES128-SHA"];    // <- this one is probably used

pub struct Connection {
	ice:               Option<IceConnection>,
	socket_type:       i32,
	controlling_mode:  ControllingMode,
	local_private_key: PKey,
	remote_public_key: Arc<PKey>,
}

impl Connection {
	pub fn new(socket_type:       i32,
	           local_private_key: PKey,
	           remote_public_key: PKey,
	           controlling_mode:  ControllingMode)
		-> Result<Connection, ConnectError>
	{
		let conn = Connection {
			ice:               Some(IceConnection::new(controlling_mode)),
			socket_type:       socket_type,
			controlling_mode:  controlling_mode,
			local_private_key: local_private_key,
			remote_public_key: Arc::new(remote_public_key),
		};
		
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
		self.ice.as_ref().unwrap().get_local_credentials()
	}

	pub fn establish_connection(&mut self, remote_credentials: Vec<u8>)
		-> Result<RawFd, ConnectError>
	{
		let cred = String::from_utf8(remote_credentials).unwrap();
		let ciphertext_ch = try_msg!("ICE connect failed.",
		                             self.ice.as_mut().unwrap().to_channel(cred),
		                             None);

		let plaintext_ch = try_msg!("SSL connect failed.",
		                            self.encrypt_connection(ciphertext_ch));

		let ch = if self.socket_type != SOCK_STREAM {
			plaintext_ch
		} else {
			let tcp = if self.controlling_mode == ControllingMode::Client {
				try_msg!("PseudoTcpSocket::connect() failed",
				          PseudoTcpSocket::connect(plaintext_ch))
			}
			else {
				PseudoTcpSocket::listen(plaintext_ch)
			};

			tcp.notify_mtu(1400);
			//stream.set_no_delay(true);

			tcp.to_channel()
		};

		let proto = 0;
		let sock = try_msg!("ChannelToSocket::new_from() failed",
		                    ChannelToSocket::new_from(self.socket_type, proto, ch));

		let fd = sock.as_raw_fd();
		debug!("SSL fd={}", fd);
		
		Ok(fd)
	}

	fn encrypt_connection(&self,
	                      ciphertext_ch: (Sender<Vec<u8>>, Receiver <Vec<u8>>))
		-> Result<(Sender<Vec<u8>>, Receiver <Vec<u8>>), SslError>
	{
		let remote_public_key           = self.remote_public_key.clone();
		let (my_plain_ch,your_plain_ch) = duplex_channel();

		let ciphers = CIPHERS.connect(",");
		let flags   = ssl::SSL_VERIFY_PEER | ssl::SSL_VERIFY_FAIL_IF_NO_PEER_CERT;

		let cert    = try!(Self::generate_cert(&self.local_private_key));
		let mut ctx = try!(SslContext::new(SslMethod::Dtlsv1));

		ctx.set_verify_with_data(flags, Self::verify_cert, remote_public_key);

		try!(ctx.set_certificate(&cert));
		try!(ctx.set_private_key(&self.local_private_key));
		try!(ctx.check_private_key());
		try!(ctx.set_cipher_list(&ciphers[..]));

		let ssl = try!(SslChannel::new(&ctx,
		                               self.controlling_mode,
		                               ciphertext_ch,
		                               my_plain_ch));
		drop(ssl);

		Ok(your_plain_ch)
	}

	#[allow(unused_variables)]
	fn verify_cert(preverify_ok: bool,
	               x509_ctx:     &X509StoreContext,
		           expected_key: &Arc<PKey>)
		-> bool
	{
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

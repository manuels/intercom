extern crate time;

use libc::{c_void};

use std::os::unix::io::RawFd;
use time::Duration;
use time::PreciseTime;
use std::thread::sleep_ms;
use std::io::Cursor;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender,Receiver};
use std::thread;

use fake_dht::FakeDHT;
use ice::IceAgent;
use ecdh::public_key::PublicKey;
use ecdh::private_key::PrivateKey;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use openssl::crypto::hash;
use openssl::crypto::pkey::{PKey,Parts};
use openssl::crypto;
use openssl::crypto::hmac;
use openssl::x509::{X509, X509StoreContext};
use openssl::ssl::{SslContext, SslMethod};
use openssl::ssl;
use openssl::ssl::error::SslError;

use ::DHT as DHT_pull_in_scope;
use ::ConnectError;
use ::DBusResponder;

use libc::consts::os::bsd44::SOCK_DGRAM;
use libc::consts::os::bsd44::AF_UNIX;
use ssl::SslChannel;
use utils::socket::ChannelToSocket;

type DHT = FakeDHT;
type SharedKey = Vec<u8>;

const HMAC_HASH: hash::Type = hash::Type::SHA512;
const CRYPTO: crypto::symm::Type = crypto::symm::Type::AES_256_CBC;

pub struct DBusRequest<R:DBusResponder> {
	pub invocation:        R,
	pub remote_public_key: Vec<u8>,
	port:                  u32,
	timeout:               Duration,
}

impl<R:DBusResponder> DBusRequest<R>
{
	pub fn new(invocation:        R,
	           remote_public_key: Vec<u8>,
	           port:              u32,
	           timeout:           u32)
		-> DBusRequest<R>
	{
		DBusRequest {
			invocation:        invocation,
			remote_public_key: remote_public_key,
			port:              port,
			timeout:           Duration::seconds(timeout as i64),
		}
	}

	pub fn handle(&self,
	              local_private_key: &PrivateKey,
	              remote_public_key: &PublicKey,
	              shared_key:        &SharedKey,
	              my_hash:           &Vec<u8>,
	              your_hash:         &Vec<u8>,
	              cert:              &X509,
	              dht:               &mut DHT)
		-> Result<RawFd,ConnectError>
	{
		// TODO: async get and set credentials

		let controlling_mode = if my_hash > your_hash {true} else {false};
		let mut agent = try!(IceAgent::new(controlling_mode).map_err(|_|ConnectError::FOO));

		let mut fd = Err(ConnectError::RemoteCredentialsNotFound);
		let start = PreciseTime::now();

		while fd.is_err() && start.to(PreciseTime::now()) < self.timeout {
			fd = self.establish_connection(&local_private_key,
			                               &remote_public_key,
			                               &shared_key,
			                               &my_hash,
			                               &your_hash,
			                               controlling_mode,
			                               &cert,
			                               &mut agent,
			                               dht);
			info!("{}\tloop: fd={:?}", controlling_mode, fd.is_ok());
			if fd.is_err() {
				sleep_ms(500);
			}
		}

		fd
	}

	fn establish_connection(&self,
	                        local_private_key: &PrivateKey,
	                        remote_public_key: &PublicKey,
	                        shared_key:        &SharedKey,
	                        my_hash:           &Vec<u8>,
	                        your_hash:         &Vec<u8>,
	                        is_server:         bool,
	                        cert:              &X509,
	                        agent:             &mut IceAgent,
	                        dht:               &mut DHT)
		-> Result<RawFd,ConnectError>
	{
		let ttl = Duration::minutes(5);

		let publish_local_credentials = |dht: &mut DHT, c| dht.put(&my_hash, &c, ttl).map_err(|_| unimplemented!());
		let lookup_remote_credentials = |dht: &mut DHT| dht.get(&your_hash).map_err(|_| ConnectError::RemoteCredentialsNotFound);
		let p2p_connect = |agent: &mut IceAgent, c| {
			let (my_tx, your_rx) = channel();
			let (your_tx, my_rx) = channel();

			agent.stream_to_channel(&c, my_tx, my_rx)
				.map(|_| (your_tx, your_rx))
				.map_err(|_| ConnectError::IceConnectFailed)
		};

		let prepend_time = |c:Vec<_>| {
			let now = time::now_utc().to_timespec();

			let mut t = vec![];
			t.write_i64::<LittleEndian>(now.sec).unwrap();

			Ok(t+&c[..])
		};

		let select_most_recent = |mut v:Vec<Vec<u8>>| {
			v.sort_by(|x,y| {
				let mut x = Cursor::new(x.clone());
				let mut y = Cursor::new(y.clone());
				let x = x.read_i64::<LittleEndian>().unwrap();
				let y = y.read_i64::<LittleEndian>().unwrap();
				x.cmp(&y).reverse()
			});
			let credentials = v.get(0).unwrap().split_at(64/8);
			Ok(credentials.1.to_vec())
		};

		Ok(agent.get_local_credentials())
			.and_then(prepend_time)
			.and_then(|c| DBusRequest::<R>::encrypt_then_mac(shared_key, &c))
			.and_then(|c| publish_local_credentials(dht, c))
			.and_then(|_| lookup_remote_credentials(dht))
			.and_then(|l| DBusRequest::<R>::decrypt(shared_key, &l))
			.and_then(select_most_recent)
			.and_then(|c| {debug!("DBusRequest: remote creds='{}'", ::std::str::from_utf8(&c).unwrap()); Ok(c)})
			.and_then(|c| p2p_connect(agent, c))
			.and_then(|ch| self.ssl_connect(ch, &local_private_key, is_server, &remote_public_key, &cert))
	}

	/// bloat up 512-bit shared ECDH key to 768 bits (key, IC, hash = 3*256 bits)
	fn split_secret_key(shared_key: &Vec<u8>) -> (Vec<u8>, Vec<u8>, Vec<u8>)
	{
		assert_eq!(shared_key.len(), 512/8);
		let (key, seed) = shared_key[..].split_at(256/8);

		assert_eq!(seed.len(), 256/8);
		let typ = hash::Type::SHA512;
		let md  = hash::hash(typ, seed);
		let (iv, hash) = md[..].split_at(256/8);

		assert_eq!(key.len(),  256/8);
		assert_eq!(iv.len(),   256/8);
		assert_eq!(hash.len(), 256/8);

		(key.to_vec(), iv.to_vec(), hash.to_vec())
	}

	fn encrypt_then_mac(shared_key: &Vec<u8>,
	                    plaintext:  &Vec<u8>)
		-> Result<Vec<u8>,ConnectError>
	{
		let (key, iv, hash) = DBusRequest::<R>::split_secret_key(shared_key);

		assert_eq!(key.len(),  256/8);
		assert_eq!(iv.len(),   256/8);
		assert_eq!(hash.len(), 256/8);

		let ciphertext = crypto::symm::encrypt(CRYPTO, &key[..], iv.to_vec(), plaintext);
		let mac = hmac::hmac(HMAC_HASH, &hash[..], &ciphertext[..]);

		Ok(mac + &ciphertext[..])
	}

	fn decrypt(shared_key:  &Vec<u8>,
	           ciphertexts: &Vec<Vec<u8>>)
		-> Result<Vec<Vec<u8>>, ConnectError>
	{
		debug!("ciphertexts: len={:?}", ciphertexts.len());

		let (key, iv, hash) = DBusRequest::<R>::split_secret_key(shared_key);

		let res: Vec<_> = ciphertexts.iter().filter_map(|ctxt| {
			let (actual_hmac, ctxt) = ctxt.split_at(HMAC_HASH.md_len());
			let expected_hmac = hmac::hmac(HMAC_HASH, &hash[..], &ctxt[..]);

			let plaintext = crypto::symm::decrypt(CRYPTO, &key[..], iv.to_vec(), &ctxt[..]);

			assert_eq!(actual_hmac.len(), expected_hmac.len());
			if crypto::memcmp::eq(&actual_hmac, &expected_hmac) {
				Some(plaintext)
			} else {
				None
			}
		}).collect();

		if res.len() > 0 {
			Ok(res)
		} else {
			Err(ConnectError::RemoteCredentialsNotFound)
		}
	}

	fn ssl_connect(&self,
	               ciphertext_ch: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
	               private_key: &PrivateKey,
	               is_server: bool,
	               remote_public_key: &PublicKey,
	               cert: &X509)
		-> Result<RawFd,ConnectError>
	{
		info!("{}\tssl_connect()", is_server);
		let ptr = remote_public_key.to_evp_pkey().unwrap() as *mut c_void;
		let expected_key = PKey::from_handle(ptr, Parts::Public);

		fn callback(_preverify_ok: bool, x509_ctx: &X509StoreContext, expected_key: &PKey) -> bool{
			info!("ssl x509 callback");

			match x509_ctx.get_current_cert() {
				None => false,
				Some(cert) => {
					let actual_key = cert.public_key();
					
					if actual_key == *expected_key {
						true
					} else {
						warn!("Expected different public key!");
						false
					}
				}
			}
		};

		let mut buf = Cursor::new(vec![0u8; 4*1024]);
		private_key.to_pem(&mut buf).unwrap();
		buf.set_position(0);
		let pkey = try!(PKey::private_key_from_pem(&mut buf).map_err(|_| ConnectError::FOO));

		fn log_error(e: SslError) -> ConnectError {
			warn!("{:?}", e);
			ConnectError::SslError(e)
		};

		let cipher = concat!(
			"ECDHE-ECDSA-AES128-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
			"ECDHE-ECDSA-AES128-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
			"ECDHE-ECDSA-AES128-SHA,",   // won't work with DTLSv1 (but probably with v1.2)
			"ECDH-ECDSA-AES128-SHA");    // <- this one is probably used

		let flags = ssl::SSL_VERIFY_PEER | ssl::SSL_VERIFY_FAIL_IF_NO_PEER_CERT;
		let mut ctx = SslContext::new(SslMethod::Dtlsv1).unwrap();
		ctx.set_verify_with_data(flags, callback, expected_key);

		try!(ctx.set_certificate(cert).map_err(log_error));
		try!(ctx.set_private_key(&pkey).map_err(log_error));
		try!(ctx.check_private_key().map_err(log_error));
		try!(ctx.set_cipher_list(cipher).map_err(log_error));

		let (plaintext_fd, plaintext_ch) = ChannelToSocket::new(AF_UNIX, SOCK_DGRAM, 0).unwrap();
		try!(SslChannel::new(&ctx, is_server, ciphertext_ch, plaintext_ch).map_err(log_error));

		info!("{} ssl_connect done", is_server);
		Ok(plaintext_fd)
	}
}


#[cfg(test)]
mod tests {
	use std::os::unix::io::RawFd;
	use std::thread;
	use std::sync::{Arc, Barrier};

	use ecdh::public_key::PublicKey;
	use ecdh::private_key::PrivateKey;

	use dbus_request::DBusRequest;
	use fake_dht::FakeDHT;
	use ::DBusResponder;

	use libc::funcs::bsd43::{send,recv};
	use libc::types::common::c95::c_void;

	extern crate env_logger;

	struct TestResponder {
		index: i32
	}

	impl TestResponder {
		pub fn new(index: i32) -> TestResponder {
			TestResponder {
				index: index
			}
		}
	}

	impl DBusResponder for TestResponder {
		fn respond_ok(&self, _: RawFd) -> Result<(),()> {
			info!("!!! respond_ok {} !!!", self.index);
			Ok(())
		}

		fn respond_error(&self, err: ::ConnectError) -> Result<(),()> {
			error!("!!! respond_error {} {:?} !!!", self.index, err);
			Err(())
		}
	}

	#[test]
	fn test_handle() {
		env_logger::init().unwrap();
		unsafe { ::bindings_glib::g_type_init() };

		let mut dht1 = FakeDHT::new();
		let mut dht2 = dht1.clone();

		let resp1 = TestResponder::new(1);
		let resp2 = TestResponder::new(2);

		let timeout = 15;
		let port = 1;

		let alice_public_key1 = vec![48u8, 51, 48, 49, 48, 66, 67, 54, 53, 56, 51, 52, 65, 56, 54, 50, 65, 65, 57, 65, 51, 51, 69, 51, 65, 51, 48, 69, 52, 70, 57, 50, 49, 51, 57, 67, 50, 56, 49, 70, 68, 49, 48, 52, 54, 49, 54, 50, 51, 56, 66, 70, 67, 48, 49, 54, 68, 65, 66, 53, 69, 49, 48, 57, 68, 54, 69, 70, 48, 55, 55, 50, 55, 70, 69, 69, 51, 50, 48, 70, 69, 67, 54, 65, 53, 52, 69, 57, 49, 66, 53, 67, 49, 52, 54, 52, 49, 53, 54, 51, 48, 50, 50, 65, 57, 69, 50, 51, 53, 53, 66, 48, 65, 70, 65, 49, 54, 50, 54, 52, 66, 51, 68, 70, 65, 69, 50, 49, 55, 55, 66, 55, 70, 53];
		let alice_public_key2 = alice_public_key1.clone();
		let alice_private_key = vec![54u8, 69, 51, 50, 69, 54, 48, 50, 54, 69, 56, 66, 54, 69, 52, 48, 54, 53, 51, 57, 57, 54, 65, 69, 70, 70, 57, 65, 69, 49, 68, 55, 53, 49, 51, 66, 69, 52, 55, 55, 56, 56, 65, 68, 53, 67, 49, 51, 51, 51, 53, 65, 48, 52, 67, 54, 54, 65, 51, 57, 57, 68, 53, 51, 65, 53, 70, 65, 50, 55, 50, 66, 54, 55, 55, 68, 66, 54, 55, 48, 69, 66, 65, 50, 66, 66, 52, 70, 49, 67, 56, 49, 57, 52, 49, 57, 68, 50, 55, 67, 55, 66, 67, 53, 68, 51, 52, 56, 51, 56, 49, 49, 54, 56, 55, 49, 68, 56, 49, 48, 55, 50, 56, 66, 50, 49, 65, 55, 67, 66];
		let bob_private_key   = alice_private_key.clone();
		let bob_public_key   = alice_public_key1.clone();

		let alice_shared_key  = vec![0u8; 512/8];
		let bob_shared_key    = vec![0u8; 512/8];

		let alice_hash1 = vec![1];
		let alice_hash2 = vec![1];
		let bob_hash1   = vec![2];
		let bob_hash2   = vec![2];

		let barrier1 = Arc::new(Barrier::new(2));
		let barrier2 = barrier1.clone();

		let thread = thread::scoped(move || {
			let req1 = DBusRequest::new(resp1, bob_public_key.clone(), port, timeout);

			let key = PrivateKey::from_vec(&alice_private_key).unwrap();
			let cert = ::generate_cert(&key);
			let result = req1.handle(&key,
				&PublicKey::from_vec(&bob_public_key).unwrap(),
				&alice_shared_key,
				&alice_hash1,
				&bob_hash1,
				&cert.unwrap(),
				&mut dht1);

			if result.is_err() {
				error!("{:?}", result);
				assert!(result.is_ok());
			}

			let fd = result.unwrap_or(-1);
			debug!("fd1={:?}", fd);
			unsafe {
				send(fd, vec![0u8; 1].as_ptr() as *const c_void, 100, 0);
			}
			let mut len;
			loop {
				len = unsafe {
					let mut buf = [0; 8*1024];
					debug!("1: test recv()...");
					recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as u64, 0)
				};
				debug!("1: test recv() len == {}", len);
				if len > 0 {
					break
				}
			}
			debug!("1: test recv() done");
			assert_eq!(len, 100);
			req1.invocation.respond(Ok(fd)).unwrap();
			barrier1.wait();
			drop(req1);
		});

		let req2 = DBusRequest::new(resp2, alice_public_key2.clone(), port, timeout);

		let key = PrivateKey::from_vec(&bob_private_key).unwrap();
		let cert = ::generate_cert(&key);
		let result = req2.handle(&key,
			&PublicKey::from_vec(&alice_public_key2).unwrap(),
			&bob_shared_key,
			&bob_hash2,
			&alice_hash2,
			&cert.unwrap(),
			&mut dht2);
		assert!(result.is_ok());

		let fd = result.unwrap_or(-1);
		debug!("fd2={:?}", fd);
		unsafe {
			send(fd, vec![0u8; 1].as_ptr() as *const c_void, 100, 0);
		}
		let mut len;
		loop {
			len = unsafe {
				let mut buf = [0; 8*1024];
				debug!("2: test recv()...");
				recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as u64, 0)
			};
			debug!("2: test recv() len == {}", len);
			if len > 0 {
				break
			}
		}
		debug!("2: test recv() done");
		req2.invocation.respond(Ok(fd)).unwrap();
		barrier2.wait();
		
		drop(req2);
		drop(thread);
	}
}

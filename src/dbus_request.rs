extern crate time;

use std::os::unix::io::RawFd;
use time::Duration;
use time::PreciseTime;
use time::Timespec;
use std::thread::sleep_ms;
use std::io::Cursor;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender,Receiver};
use std::thread;

//use ::dgram_unix_socket::DgramUnixSocket;
use fake_dht::FakeDHT;
use ice::IceAgent;
use ecdh::public_key::PublicKey;
use ecdh::private_key::PrivateKey;

use openssl::crypto::hash;
use openssl::crypto::pkey::PKey;
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

trait ToVec {
	fn to_vec(&self) -> Vec<u8>;
	fn from_vec(vec: &Vec<u8>) -> Timespec;
}

impl ToVec for Timespec {
	fn to_vec(&self) -> Vec<u8> {
		vec![
		 ((self.sec  >> 54) & 0xff) as u8,
		 ((self.sec  >> 48) & 0xff) as u8,
		 ((self.sec  >> 40) & 0xff) as u8,
		 ((self.sec  >> 32) & 0xff) as u8,
		 ((self.sec  >> 24) & 0xff) as u8,
		 ((self.sec  >> 16) & 0xff) as u8,
		 ((self.sec  >>  8) & 0xff) as u8,
		 ((self.sec  >>  0) & 0xff) as u8,
		 ((self.nsec >> 24) & 0xff) as u8,
		 ((self.nsec >> 16) & 0xff) as u8,
		 ((self.nsec >>  8) & 0xff) as u8,
		 ((self.nsec >>  0) & 0xff) as u8,
		]
	}

	fn from_vec(vec: &Vec<u8>) -> Timespec {
		assert!(vec.len() == 12);

		let sec = 0 |
			(vec[0] as i64) << 54 | (vec[1] as i64) << 48 |
			(vec[2] as i64) << 40 | (vec[3] as i64) << 32 |
			(vec[4] as i64) << 24 | (vec[5] as i64) << 16 |
			(vec[6] as i64) <<  8 | (vec[7] as i64) <<  0;
		let nsec = 0 |
			(vec[8] as i32) << 24 | (vec[9] as i32) << 16 |
			(vec[10] as i32) <<  8 | (vec[11] as i32) <<  0;

		Timespec {
			sec:  sec,
			nsec: nsec
		}
	}
}

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
	              local_public_key:  &PublicKey,
	              remote_public_key: &PublicKey,
	              shared_key:        &SharedKey,
	              my_hash:           &Vec<u8>,
	              your_hash:         &Vec<u8>,
	              cert:              &X509,
	              dht:               &mut DHT)
		-> Result<RawFd,ConnectError>
	{
		// TODO: async get and set credentials

		let controlling_mode = (my_hash > your_hash);
		let mut agent = try!(IceAgent::new(controlling_mode).map_err(|_|ConnectError::FOO));

		let mut fd = Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND);
		let start = PreciseTime::now();

		while fd.is_err() && start.to(PreciseTime::now()) < self.timeout {
			fd = self.establish_connection(&local_private_key,
			                               &local_public_key,
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

		thread::spawn(move || {
			loop {
				sleep_ms(1000);
			}
			agent;
		});

		fd
	}

	fn establish_connection(&self,
	                        local_private_key: &PrivateKey,
	                        local_public_key:  &PublicKey,
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
		let (txA, rxA) = channel();
		let (txB, rxB) = channel();
		let ttl = Duration::minutes(5);

		let publish_local_credentials = |dht: &mut DHT, c| dht.put(&my_hash, &c, ttl);
		let lookup_remote_credentials = |dht: &mut DHT| dht.get(&your_hash);
		let p2p_connect = |agent: &mut IceAgent, c| {
			let res = agent.stream_to_channel(&c, txA, rxB).map(|_| (txB,rxA));

			res
		};
		/*let prepend_time = |mut c| {
			let mut v = time::get_time().to_vec();
			v.append(&mut c);
			Ok(v)
		};*/

		Ok(agent.get_local_credentials())
			//.and_then(|c| prepend_time(c))
			.and_then(|c| DBusRequest::<R>::encrypt(shared_key, &c))
			.and_then(|c| publish_local_credentials(dht, c).map_err(|_| unimplemented!()))
			.and_then(|_| lookup_remote_credentials(dht).map_err(|_| ConnectError::REMOTE_CREDENTIALS_NOT_FOUND))
			.and_then(|l| DBusRequest::<R>::decrypt(shared_key, &l))
			//.and_then(select_most_recent)
			.and_then(|c| {debug!("DBusRequest: remote creds='{}'", ::std::str::from_utf8(&c).unwrap()); Ok(c)})
			.and_then(|c| p2p_connect(agent, c).map_err(|_| ConnectError::REMOTE_CREDENTIALS_NOT_FOUND))
			.and_then(|ch| self.ssl_connect(ch, &local_private_key, is_server, &your_hash, &cert))
	}

	fn split_secret_key(shared_key: &Vec<u8>) -> (Vec<u8>, Vec<u8>, Vec<u8>)
	{
		assert_eq!(shared_key.len(), 512/8);

		let (key, seed) = shared_key[..].split_at(512/8/2);

		let typ = hash::Type::SHA512;
		let md  = hash::hash(typ, seed);
		let (iv, hash) = md[..].split_at(512/8/2);

		(key.to_vec(), iv.to_vec(), hash.to_vec())
	}

	fn encrypt(shared_key: &Vec<u8>,
	           plaintext:  &Vec<u8>)
		-> Result<Vec<u8>,ConnectError>
	{
		let (key, iv, hash) = DBusRequest::<R>::split_secret_key(shared_key);

		assert_eq!(key.len(),  256/8);
		assert_eq!(iv.len(),   256/8);
		assert_eq!(hash.len(), 256/8);

		info!("keylen={}", key.len());
		let ciphertext = crypto::symm::encrypt(CRYPTO, &key[..], iv.to_vec(), plaintext);

		let res = hmac::hmac(HMAC_HASH, &hash[..], &ciphertext[..]) + &ciphertext[..];

		Ok(res)
	}

	fn decrypt(shared_key:  &Vec<u8>,
	           ciphertexts: &Vec<Vec<u8>>)
		-> Result<Vec<u8>,ConnectError>
	{
		debug!("ciphertext: {:?}", ciphertexts.get(0).map(|v| &v[..]));
		let ctxt = try!(ciphertexts.get(0).ok_or(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND));

		let (key, iv, hash) = DBusRequest::<R>::split_secret_key(shared_key);

		let (actual_hmac,ctxt) = ctxt.split_at(HMAC_HASH.md_len());
		let expected_hmac = hmac::hmac(HMAC_HASH, &hash[..], &ctxt[..]);

		info!("de keylen={}", key.len());
		let plaintext = crypto::symm::decrypt(CRYPTO, &key[..], iv.to_vec(), &ctxt[..]);

		assert_eq!(actual_hmac.len(), expected_hmac.len());
		if crypto::memcmp::eq(&actual_hmac, &expected_hmac) {
			Ok(plaintext)
		} else {
			Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
		}
	}

	fn ssl_connect(&self, ciphertext_ch: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
			private_key: &PrivateKey, is_server: bool, your_hash: &Vec<u8>,
			cert: &X509) -> Result<RawFd,ConnectError>
	{
		info!("{}\tssl_connect()", is_server);
		fn callback(_preverify_ok: bool, x509_ctx: &X509StoreContext, expected_hash: &Vec<u8>) -> bool{
			info!("ssl x509 callback");

			/*if (x509_ctx.get_error().is_some()) {
				return false;
			}*/

			match x509_ctx.get_current_cert() {
				None => false,
				Some(cert) => {
					// cert.get_public_key() -> *evp_pkey_st
					match cert.fingerprint(hash::Type::SHA256) {
						Some(actual_hash) => {
							info!("cert fingerprints: {:?}\t{:?}", actual_hash, expected_hash);
							//crypto::memcmp::eq(&actual_hash[..], &expected_hash[..]);
							return true;
						},
						None => false,
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
			ConnectError::FOO
		};

		let cipher = concat!("ECDHE-ECDSA-AES128-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
			"ECDHE-ECDSA-AES128-SHA256,",// won't work with DTLSv1 (but probably with v1.2)
			"ECDHE-ECDSA-AES128-SHA,",   // won't work with DTLSv1 (but probably with v1.2)
			"ECDH-ECDSA-AES128-SHA");    // <- this one is probably used

		let mut ctx = SslContext::new(SslMethod::Dtlsv1).unwrap();
		try!(ctx.set_certificate(cert).map_err(log_error));
		try!(ctx.set_private_key(&pkey).map_err(log_error));
		try!(ctx.check_private_key().map_err(log_error));
		try!(ctx.set_cipher_list(cipher).map_err(log_error));

		let flags = ssl::SSL_VERIFY_PEER | ssl::SSL_VERIFY_FAIL_IF_NO_PEER_CERT;
		ctx.set_verify_with_data(flags, callback, your_hash.clone());

		let (plaintext_fd, plaintext_ch) = ChannelToSocket::new(AF_UNIX, SOCK_DGRAM, 0, false).unwrap();
		SslChannel::new(&ctx, is_server, ciphertext_ch, plaintext_ch).unwrap();

		info!("{} ssl_connect done", is_server);
		Ok(plaintext_fd)
	}
}


#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use time::Duration;
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
			error!("!!! respond_error {} {} !!!", self.index, err);
			Err(())
		}
	}

	impl ::DHT for HashMap<Vec<u8>,Vec<u8>> {
		fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()> {
			Ok(vec![self.get(key).unwrap().clone()])
		}

		fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, _: Duration)
			->  Result<(),()>
		{
			self.insert(key.clone(), value.clone());
			Ok(())
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
		let bob_public_key1   = alice_public_key1.clone();
		let bob_public_key2   = bob_public_key1.clone();

		let alice_shared_key  = vec![0u8; 512/8];
		let bob_shared_key    = vec![0u8; 512/8];

		let alice_hash1 = vec![1];
		let alice_hash2 = vec![1];
		let bob_hash1   = vec![2];
		let bob_hash2   = vec![2];

		let barrier1 = Arc::new(Barrier::new(2));
		let barrier2 = barrier1.clone();

		let thread = thread::scoped(move || {
			let req1 = DBusRequest::new(resp1, bob_public_key1.clone(), port, timeout);

			let key = PrivateKey::from_vec(&alice_private_key).unwrap();
			let cert = ::generate_cert(&key);
			let result = req1.handle(&key,
				&PublicKey::from_vec(&alice_public_key1).unwrap(),
				&PublicKey::from_vec(&bob_public_key1).unwrap(),
				&alice_shared_key,
				&alice_hash1,
				&bob_hash1,
				&cert.unwrap(),
				&mut dht1);
			//assert!(result.is_ok());
			if result.is_err() {
				error!("{:?}", result);
				assert!(result.is_ok());
			}

			let fd = result.unwrap_or(-1);
			info!("fd1={:?}", fd);
			for i in 0..1 {
				unsafe {
					send(fd, vec![0u8; 1].as_ptr() as *const c_void, 100, 0);
				}
			}
			let mut len = 0;
			loop {
				len = unsafe {
					let mut buf = [0; 8*1024];
					info!("1: test recv()...");
					recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as u64, 0)
				};
				if len > 0 {
					break
				}
			}
			info!("1: test recv() done");
			assert_eq!(len, 100);
			req1.invocation.respond(Ok(fd)).unwrap();
			barrier1.wait();
			drop(req1);
		});

		let req2 = DBusRequest::new(resp2, alice_public_key2.clone(), port, timeout);

		let key = PrivateKey::from_vec(&bob_private_key).unwrap();
		let cert = ::generate_cert(&key);
		let result = req2.handle(&key,
			&PublicKey::from_vec(&bob_public_key2).unwrap(),
			&PublicKey::from_vec(&alice_public_key2).unwrap(),
			&bob_shared_key,
			&bob_hash2,
			&alice_hash2,
			&cert.unwrap(),
			&mut dht2);
		assert!(result.is_ok());

		let fd = result.unwrap_or(-1);
		info!("fd2={:?}", fd);
		for i in 0..1 {
			unsafe {
				send(fd, vec![0u8; 1].as_ptr() as *const c_void, 100, 0);
			}
		}
		let mut len = 0;
		loop {
			len = unsafe {
				let mut buf = [0; 8*1024];
				info!("2: test recv()...");
				recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as u64, 0)
			};
			if len > 0 {
				break
			}
		}
		info!("2: test recv() done");
		assert_eq!(len, 1);
		req2.invocation.respond(Ok(fd)).unwrap();
		barrier2.wait();
		
		drop(req2);
		drop(thread);
	}
}

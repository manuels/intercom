extern crate time;

use std::time::duration::Duration;
use std::os::unix::Fd;
use time::SteadyTime;
use std::old_io::MemWriter;
use std::old_io::timer::sleep;
use from_pointer::cstr;

use utils::spawn_thread;
use fake_dht::FakeDHT;
use ice::IceAgent;
use ecdh::ecdh::ECDH;
use ecdh::public_key::PublicKey;
use ecdh::private_key::PrivateKey;
use glib::dbus_method_invocation::GDBusMethodInvocation as GInvocation;

use openssl::crypto::hash;
use openssl::crypto;
use openssl::crypto::hmac;

use ::DHT as DHT_pull_in_scope;
use ::ConnectError;
use ::DBusResponder;

type DHT = FakeDHT;

type SharedKey = Vec<u8>;

pub struct DBusRequest<R:DBusResponder> {
	pub invocation:    R,
	pub remote_public_key: Vec<u8>,
	port:              u32,
	timeout:           Duration,
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
	              dht:               &mut DHT)
		-> Result<Fd,ConnectError>
	{
		// TODO: async get and set credentials

		let controlling_mode = (local_public_key.to_vec() > remote_public_key.to_vec());
		let mut agent = try!(IceAgent::new(controlling_mode).map_err(|_|ConnectError::FOO));

		let mut fd = Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND);
		let end = SteadyTime::now() + self.timeout;

		while fd.is_err() && SteadyTime::now() < end {
			fd = self.establish_connection(&local_private_key,
			                               &local_public_key,
			                               &remote_public_key,
			                               &shared_key,
			                               &my_hash,
			                               &your_hash,
			                               &mut agent,
			                               dht);
			sleep(Duration::seconds(1));
		}

		fd
	}

	fn establish_connection(&self,
	                        local_private_key: &PrivateKey,
	                        local_public_key:  &PublicKey,
	                        remote_public_key: &PublicKey,
	                        shared_key:        &SharedKey,
	                        my_hash:           &Vec<u8>,
	                        your_hash:         &Vec<u8>,
	                        agent:             &mut IceAgent,
	                        dht:               &mut DHT)
		-> Result<Fd,ConnectError>
	{
		Ok(agent.get_local_credentials())
			.and_then(|c| self.encrypt(shared_key, &c))
			.and_then(|c| self.publish_local_credentials(dht, &local_public_key, &c))
			.and_then(|_| self.lookup_remote_credentials(dht, &remote_public_key))
			.and_then(|l| self.decrypt(shared_key, &l))
			//.and_then(select_most_recent)
			.and_then(|c| self.p2p_connect(agent, &c))
			//.and_then(|c| self.ssl_connect(c))
	}

	fn publish_local_credentials(&self,
	                             dht:              &mut DHT,
	                             local_public_key: &PublicKey,
	                             credentials:      &Vec<u8>)
		-> Result<(),ConnectError>
	{
		dht.put(&local_public_key.to_vec().map_in_place(|x| x as u8),
		        &credentials, Duration::minutes(5))
			.map_err(|_| unimplemented!())
	}
 
	fn lookup_remote_credentials(&self,
	                             dht:               &mut DHT,
	                             remote_public_key: &PublicKey)
		-> Result<Vec<Vec<u8>>,ConnectError>
	{
		dht.get(&remote_public_key.to_vec().map_in_place(|x| x as u8))
			.map_err(|_| ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
	}

	fn split_secret_key<'a>(&'a self, shared_key: &'a Vec<u8>)
		-> (Vec<u8>, Vec<u8>, Vec<u8>)
	{
		let (key, seed) = shared_key.as_slice().split_at(16);

		let typ = hash::Type::SHA512;
		let md  = hash::hash(typ, seed);
		let (iv, hash) = md.as_slice().split_at(16);

		(key.to_vec(), iv.to_vec(), hash.to_vec())
	}

	fn encrypt<'b>(&self, 
	               shared_key: &Vec<u8>,
	               plaintext:  &Vec<u8>)
		-> Result<Vec<u8>,ConnectError>
	{
		let typ = crypto::symm::Type::AES_128_CBC;
		let (key, iv, hash) = self.split_secret_key(shared_key);

		let mut ciphertext = crypto::symm::encrypt(typ, key.as_slice(), iv.to_vec(), plaintext);

		let typ = hash::Type::SHA512;
		let mut res = hmac::hmac(typ, hash.as_slice(), ciphertext.as_slice());
		res.append(&mut ciphertext);

		Ok(res)
	}

	fn decrypt<'b>(&self,
	               shared_key:  &Vec<u8>,
	               ciphertexts: &Vec<Vec<u8>>)
			-> Result<Vec<u8>,ConnectError>
	{
		warn!("DBusRequest::decrypt() is unimplemented!");
		// should be something like
		// |c| c.into_iter().filter_map(self.decrypt)
		// in the end

		debug!("ciphertext: {:?}", ciphertexts.get(0).map(|v| ::std::str::from_utf8(v.as_slice())));
		let ctxt = try!(ciphertexts.get(0).ok_or(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND));

		let (key, iv, hash) = self.split_secret_key(shared_key);

		let typ = hash::Type::SHA512;
		let (actual_hmac,ctxt) = ctxt.split_at(typ.md_len());
		let expected_hmac = hmac::hmac(typ, hash.as_slice(), ctxt.as_slice());

		let typ = crypto::symm::Type::AES_128_CBC;
		let plaintext = crypto::symm::decrypt(typ, key.as_slice(), iv.to_vec(), ctxt.as_slice());

		if actual_hmac == expected_hmac {
			Ok(plaintext)
		} else {
			Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
		}
	}

	fn p2p_connect(&self, agent: &mut IceAgent, credentials: &Vec<u8>)
		-> Result<Fd,ConnectError>
	{
		agent.stream_to_socket(credentials.clone())
			.map_err(|_|ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
	}

	fn ssl_connect(&self, fd: Fd) -> Result<Fd,ConnectError> {
		unimplemented!();
	}
}


#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::time::duration::Duration;
	use std::os::unix::Fd;
	use std::thread::Thread;

	use ecdh::public_key::PublicKey;
	use ecdh::private_key::PrivateKey;

	use dbus_request::DBusRequest;
	use fake_dht::FakeDHT;
	use ::DBusResponder;

	struct TestResponder;
	impl DBusResponder for TestResponder {
		fn respond_ok(&self, fd: Fd) -> Result<(),()> {
			Ok(())
		}

		fn respond_error(&self, err: ::ConnectError) -> Result<(),()> {
			Err(())
		}
	}

	impl ::DHT for HashMap<Vec<u8>,Vec<u8>> {
		fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()> {
			Ok(vec![self.get(key).unwrap().clone()])
		}

		fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration)
			->  Result<(),()>
		{
			self.insert(key.clone(), value.clone());
			Ok(())
		}
	}

	#[test]
	fn test_handle() {
		unsafe { ::bindings_glib::g_type_init() };

		let mut dht1 = FakeDHT::new();
		let mut dht2 = dht1.clone();

		let resp1 = TestResponder;
		let resp2 = TestResponder;

		let timeout = 99;
		let port = 1;

		let alice_public_key1 = vec![48i8, 51, 48, 49, 48, 66, 67, 54, 53, 56, 51, 52, 65, 56, 54, 50, 65, 65, 57, 65, 51, 51, 69, 51, 65, 51, 48, 69, 52, 70, 57, 50, 49, 51, 57, 67, 50, 56, 49, 70, 68, 49, 48, 52, 54, 49, 54, 50, 51, 56, 66, 70, 67, 48, 49, 54, 68, 65, 66, 53, 69, 49, 48, 57, 68, 54, 69, 70, 48, 55, 55, 50, 55, 70, 69, 69, 51, 50, 48, 70, 69, 67, 54, 65, 53, 52, 69, 57, 49, 66, 53, 67, 49, 52, 54, 52, 49, 53, 54, 51, 48, 50, 50, 65, 57, 69, 50, 51, 53, 53, 66, 48, 65, 70, 65, 49, 54, 50, 54, 52, 66, 51, 68, 70, 65, 69, 50, 49, 55, 55, 66, 55, 70, 53];
		let alice_public_key2 = alice_public_key1.clone();
		let alice_private_key = vec![54i8, 69, 51, 50, 69, 54, 48, 50, 54, 69, 56, 66, 54, 69, 52, 48, 54, 53, 51, 57, 57, 54, 65, 69, 70, 70, 57, 65, 69, 49, 68, 55, 53, 49, 51, 66, 69, 52, 55, 55, 56, 56, 65, 68, 53, 67, 49, 51, 51, 51, 53, 65, 48, 52, 67, 54, 54, 65, 51, 57, 57, 68, 53, 51, 65, 53, 70, 65, 50, 55, 50, 66, 54, 55, 55, 68, 66, 54, 55, 48, 69, 66, 65, 50, 66, 66, 52, 70, 49, 67, 56, 49, 57, 52, 49, 57, 68, 50, 55, 67, 55, 66, 67, 53, 68, 51, 52, 56, 51, 56, 49, 49, 54, 56, 55, 49, 68, 56, 49, 48, 55, 50, 56, 66, 50, 49, 65, 55, 67, 66];
		let bob_private_key = alice_private_key.clone();
		let bob_public_key1 = alice_public_key1.clone();
		let bob_public_key2 = bob_public_key1.clone();
/*
	pub fn handle(&self,
	              local_private_key: &PrivateKey,
	              local_public_key:  &PublicKey,
	              remote_public_key: &PublicKey,
	              shared_key:        &SharedKey,
	              my_hash:           &Vec<u8>,
	              your_hash:         &Vec<u8>,
	              dht:               &mut DHT)
	              */
		let alice_shared_key = vec![0u8; 512/8];
		let bob_shared_key = vec![0u8; 512/8];

		let alice_hash1 = vec![1];
		let alice_hash2 = vec![1];
		let bob_hash1 = vec![1];
		let bob_hash2 = vec![1];

		let thread = Thread::scoped(move || {
			let req1 = DBusRequest::new(resp1, vec![98], port, timeout);

			let result = req1.handle(&PrivateKey::from_vec(&alice_private_key).unwrap(),
				&PublicKey::from_vec(&alice_public_key1).unwrap(),
				&PublicKey::from_vec(&bob_public_key1).unwrap(),
				&alice_shared_key,
				&alice_hash1,
				&bob_hash1,
				&mut dht1);
			req1.invocation.respond(result).unwrap();
		});

		let req2 = DBusRequest::new(resp2, vec![97], port, timeout);

		let result = req2.handle(&PrivateKey::from_vec(&bob_private_key).unwrap(),
			&PublicKey::from_vec(&bob_public_key2).unwrap(),
			&PublicKey::from_vec(&alice_public_key2).unwrap(),
			&bob_shared_key,
			&bob_hash2,
			&alice_hash2,
			&mut dht2);
		req2.invocation.respond(result).unwrap();
		
		drop(thread);
	}
}

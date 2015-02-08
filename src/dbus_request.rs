extern crate time;

use std::time::duration::Duration;
use std::os::unix::Fd;
use time::SteadyTime;
use std::old_io::MemWriter;
use from_pointer::cstr;
use std::old_io::timer::sleep;

use ::DHT;
use ::ConnectError;
use ::DbusResponder;
use ice::IceAgent;
use utils::spawn_thread;

pub struct DbusRequest<R:DbusResponder> {
	pub invocation:    R,
	remote_public_key: Vec<u8>,
	port:              u32,
	timeout:           Duration,
}

impl<R:DbusResponder> DbusRequest<R> {
	pub fn new(invocation:        R,
	           remote_public_key: Vec<u8>,
	           port:              u32,
	           timeout:           u32)
		-> DbusRequest<R>
	{
		DbusRequest {
			invocation:        invocation,
			remote_public_key: remote_public_key,
			port:              port,
			timeout:           Duration::seconds(timeout as i64),
		}
	}

	pub fn handle<T: DHT>(&self, local_public_key: Vec<u8>, dht: &mut T)
		-> Result<Fd,ConnectError>
	{
		let controlling_mode = (local_public_key.as_slice() > self.remote_public_key.as_slice());
		let mut agent = try!(IceAgent::new(controlling_mode).map_err(|_|ConnectError::FOO));

		// TODO: async get and set credentials
		let mut fd = Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND);
		let begin = SteadyTime::now();

		while fd.is_err() && begin + self.timeout > SteadyTime::now() {
			info!("new connect attempt for {:?}", self.remote_public_key);
			fd = self.establish_connection(local_public_key.clone(), &mut agent, dht);
			debug!("establish_connection finished: {:?}", fd.is_ok());
			sleep(Duration::seconds(1));
		}

		/*
		spawn_thread("DBusRequest::loop", move || {
			// keep agent alive
			loop {};
			drop(agent);
		});*/

		fd
	}

	fn establish_connection<T:DHT>(&self, local_public_key: Vec<u8>,
				agent: &mut IceAgent, dht:&mut T)
		-> Result<Fd,ConnectError>
	{
		Ok(agent.get_local_credentials())
			//.and_then(|c| self.encrypt(c))
			.and_then(|c| self.publish_local_credentials(dht, local_public_key, c))
			.and_then(|_| self.lookup_remote_credentials(dht))
			.and_then(|l| self.decrypt(l))
			//.and_then(select_most_recent)
			.and_then(|c| self.p2p_connect(agent, c))
			//.and_then(|c| self.ssl_connect(c))
	}

	fn publish_local_credentials<T:DHT>(&self, dht: &mut T,
						local_public_key:Vec<u8>, credentials: Vec<u8>)
		-> Result<(),ConnectError>
	{
		dht.put(&local_public_key, &credentials,
				Duration::minutes(5))
			.map_err(|_| unimplemented!())
	}
 
	fn lookup_remote_credentials<T:DHT>(&self, dht: &mut T)
		-> Result<Vec<Vec<u8>>,ConnectError>
	{
		dht.get(&self.remote_public_key)
			.map_err(|_| ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
	}

	fn decrypt<'b>(&self, ciphertexts: Vec<Vec<u8>>)
			-> Result<Vec<u8>,ConnectError>
	{
		// should be something like
		// |c| c.into_iter().filter_map(self.decrypt).next())
		// in the end

		debug!("ciphertext: {:?}", ciphertexts.get(0).map(|v| ::std::str::from_utf8(v.as_slice())));
		ciphertexts.get(0).ok_or(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
			.map(|s| s.clone())
	}

	fn encrypt<'b>(&self, plaintext: Vec<u8>) -> Result<Vec<u8>,ConnectError> {
		unimplemented!();
	}

	fn p2p_connect(&self, agent: &mut IceAgent, credentials: Vec<u8>)
		-> Result<Fd,ConnectError>
	{
		agent.stream_to_socket(credentials)
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

	use dbus_request::DbusRequest;
	use fake_dht::FakeDHT;
	use ::DbusResponder;

	struct TestResponder;
	impl DbusResponder for TestResponder {
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

		let thread = Thread::scoped(move || {
			let req1 = DbusRequest::new(resp1, vec![98], port, timeout);

			let result = req1.handle("a".as_bytes().to_vec(), &mut dht1);
			req1.invocation.respond(result).unwrap();
		});

		let req2 = DbusRequest::new(resp2, vec![97], port, timeout);

		let result = req2.handle("b".as_bytes().to_vec(), &mut dht2);
		req2.invocation.respond(result).unwrap();
		
		drop(thread);
	}
}

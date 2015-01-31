extern crate time;

use std::time::duration::Duration;
use std::os::unix::Fd;
use time::SteadyTime;
use std::io::MemWriter;

use ::DHT;
use ::ConnectError;
use ::DbusResponder;
use ice::IceAgent;

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

	pub fn handle<T: DHT>(&self, local_public_key: &str, dht: &T)
		-> Result<Fd,ConnectError>
	{
		let controlling_mode = (local_public_key.as_bytes() > self.remote_public_key.as_slice());
		let agent = try!(IceAgent::new(controlling_mode).map_err(|_|ConnectError::FOO));

		// TODO: async get and set credentials
		let mut fd = Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND);
		let begin = SteadyTime::now();

		while fd.is_err() && begin + self.timeout > SteadyTime::now() {
			fd = self.establish_connection(&agent, dht);
		}
		fd
	}

	fn establish_connection<T:DHT>(&self, agent: &IceAgent, dht:&T)
		-> Result<Fd,ConnectError>
	{
		Ok(agent.get_local_credentials())
			//.and_then(|c| self.encrypt(c))
			.and_then(|c| self.publish_local_credentials(dht, c))
			.and_then(|_| self.lookup_remote_credentials(dht))
			.and_then(|l| self.decrypt(l))
			//.and_then(select_most_recent)
			.and_then(|c| self.p2p_connect(agent, c))
			//.and_then(|c| self.ssl_connect(c))
	}

	fn publish_local_credentials<T:DHT>(&self, dht: &T, credentials: Vec<u8>) -> Result<(),ConnectError> {
		dht.put(&self.remote_public_key, &credentials,
				Duration::minutes(5))
			.map_err(|_| unimplemented!())
	}
 
	fn lookup_remote_credentials<T:DHT>(&self, dht: &T)
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

		ciphertexts.get(0).ok_or(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND)
			.map(|s| s.clone())
	}

	fn encrypt<'b>(&self, plaintext: Vec<u8>) -> Result<Vec<u8>,ConnectError> {
		unimplemented!();
	}

	fn p2p_connect(&self, agent: &IceAgent, credentials: Vec<u8>) -> Result<Fd,ConnectError> {
		let count = try!(agent.set_remote_credentials(credentials));
		if count < 1 {
			return Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND);
		}

		//ch = try!(agent.stream_to_channel(ctx, stream))
		//channel2fd(ch)
		unimplemented!();
	}

	fn ssl_connect(&self, fd: Fd) -> Result<Fd,ConnectError> {
		unimplemented!();
	}
}

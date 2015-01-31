extern crate time;

use std::time::duration::Duration;
use std::os::unix::Fd;
use time::SteadyTime;
use std::io::MemWriter;

use glib::dbus_method_invocation::GDBusMethodInvocation;
use ::DHT;
use ::ConnectError;
use dht::LunaDHT;
use fake_dht::FakeDHT;
use ice::IceAgent;

pub struct DbusRequest {
	pub invocation:        GDBusMethodInvocation,
	remote_public_key: Vec<u8>,
	port:              u32,
	timeout:           Duration,
}

impl DbusRequest {
	pub fn new(invocation:        GDBusMethodInvocation,
	           remote_public_key: Vec<u8>,
	           port:              u32,
	           timeout:           u32)
		-> DbusRequest
	{
		DbusRequest {
			invocation:        invocation,
			remote_public_key: remote_public_key,
			port:              port,
			timeout:           Duration::seconds(timeout as i64),
		}
	}

	pub fn handle(&self, local_public_key: &str) -> Result<Fd,ConnectError>
	{
		let controlling_mode = (local_public_key.as_bytes() > self.remote_public_key.as_slice());
		let agent = try!(IceAgent::new(controlling_mode).map_err(|_|ConnectError::FOO));

		// TODO: async get and set credentials
		let mut fd = Err(ConnectError::REMOTE_CREDENTIALS_NOT_FOUND);
		let begin = SteadyTime::now();

		while fd.is_err() && begin + self.timeout > SteadyTime::now() {
			fd = self.establish_connection(&agent);
		}
		fd
	}

	fn establish_connection(&self, agent: &IceAgent) -> Result<Fd,ConnectError>
	{
		Ok(agent.get_local_credentials())
			//.and_then(|c| self.encrypt(c))
			.and_then(|c| self.publish_local_credentials(c))
			.and_then(|_| self.lookup_remote_credentials())
			.and_then(|l| self.decrypt(l))
			//.and_then(select_most_recent)
			.and_then(|c| self.p2p_connect(agent, c))
			//.and_then(|c| self.ssl_connect(c))
	}

	fn publish_local_credentials(&self, credentials: Vec<u8>) -> Result<(),ConnectError> {
		// TODO: append now_utc()
		let dht = FakeDHT::new();

		dht.put(&self.remote_public_key, &credentials,
				Duration::minutes(5))
			.map_err(|_| unimplemented!())
	}
 
	fn lookup_remote_credentials(&self) -> Result<Vec<Vec<u8>>,ConnectError> {
		let dht = FakeDHT::new();
		
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

extern crate time;

use std::time::duration::Duration;
use std::os::unix::Fd;
use time::SteadyTime;

use ice::IceAgent;

pub struct DbusRequest<'a> {
	remote_public_key: &'a str,
	port:              u16,
	timeout:           Duration,
}

impl<'a> DbusRequest<'a> {
	pub fn handle(&self, local_public_key: &str) -> Result<Fd,()>
	{
		let controlling_mode = (local_public_key > self.remote_public_key);
		let agent = try!(IceAgent::new(controlling_mode));

		// TODO: async get and set credentials
		let mut fd = Err(());
		let begin = SteadyTime::now();

		while fd.is_err() && begin + self.timeout > SteadyTime::now() {
			fd = self.establish_connection(&agent);
		}
		fd
	}

	fn establish_connection(&self, agent: &IceAgent) -> Result<Fd,()>
	{
		Ok(agent.get_local_credentials())
			//.and_then(|c| self.encrypt(c))
			.and_then(|c| self.publish_local_credentials(c))
			.and_then(|_| self.lookup_remote_credentials())
			//.and_then(|c| c.into_iter().filter_map(self.decrypt).next())
			//.and_then(select_most_recent)
			.and_then(|c| self.p2p_connect(c))
			//.and_then(|c| self.ssl_connect(c))
	}

	fn publish_local_credentials(&self, credentials: String) -> Result<(),()> {
		// TODO: append now_utc()
		unimplemented!();
	}
 
	fn lookup_remote_credentials(&self) -> Result<String,()> {
		unimplemented!();
	}

	fn decrypt<'b>(&self, ciphertext: String) -> Result<String,()> {
		unimplemented!();
	}

	fn encrypt<'b>(&self, plaintext: String) -> Result<String,()> {
		unimplemented!();
	}

	fn p2p_connect(&self, credentials: String) -> Result<Fd,()> {
		//try!(agent.parse_remote_sdp(credentials))
		//ch = try!(agent.stream_to_channel(ctx, stream))
		//channel2fd(ch)

		unimplemented!();
	}

	fn ssl_connect(&self, fd: Fd) -> Result<Fd,()> {
		unimplemented!();
	}
}

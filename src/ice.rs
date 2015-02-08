use std::os::unix::Fd;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Future;
use std::thread::Thread;
use libc;

use ::ConnectError;
use nice::agent::NiceAgent;
use nice::glib2::GMainLoop;
use nice::bindings_agent::GMainContext;
use utils::spawn_thread;

pub struct IceAgent {
	agent: NiceAgent,
	ctx:   *mut GMainContext,
	stream: u32,
	state_rx: Receiver<libc::c_uint>,
}

unsafe impl Send for IceAgent {}

impl IceAgent {
	pub fn new(controlling_mode: bool) -> Result<IceAgent,()>
	{
		let mainloop  = GMainLoop::new();
		let ctx       = mainloop.get_context() as *mut GMainContext;
		let mut agent = NiceAgent::new(ctx, controlling_mode);

		let (stream, state_rx) = try!(agent.add_stream(Some("ganymed")));

		spawn_thread("IceAgent::GMainLoop", move || {
			mainloop.run();
		});

		agent.gather_candidates(stream);

		Ok(IceAgent {
			agent:    agent,
			ctx:      ctx,
			stream:   stream,
			state_rx: state_rx
		})
	}

	pub fn get_local_credentials(&self) -> Vec<u8> {
		self.agent.generate_local_sdp().into_bytes()
	}

	pub fn stream_to_channel(&mut self, credentials: Vec<u8>)
			 -> Result<(Sender<Vec<u8>>,Receiver<Vec<u8>>), ()>
	{
		let cred = String::from_utf8(credentials).unwrap_or("".to_string());
		self.agent.stream_to_channel(self.ctx, self.stream, cred, &self.state_rx)
	}

	pub fn stream_to_socket(&mut self, credentials: Vec<u8>)
			-> Result<Fd,()>
	{
		let cred = String::from_utf8(credentials).unwrap_or("".to_string());
		self.agent.stream_to_socket(self.ctx, self.stream, cred, &self.state_rx)
	}
}

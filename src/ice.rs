#![allow(dead_code)]

use std::os::unix::io::RawFd;
use std::sync::mpsc::{Sender,Receiver};
use std::vec::Vec;
use std::sync::Future;
use std::thread::Thread;
use libc;

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
		let ctx       = *mainloop.get_context() as *mut GMainContext;
		let mut agent = try!(NiceAgent::new(ctx, controlling_mode));

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

	pub fn get_local_credentials(&mut self) -> Vec<u8> {
		self.agent.generate_local_sdp().into_bytes()
	}

	pub fn get_controlling_mode(&mut self) -> Result<bool,()> {
		self.agent.get_controlling_mode()
	}

	pub fn stream_to_channel(&mut self, credentials: &Vec<u8>,
		tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>)
			 -> Result<(), ()>
	{
		let cred = String::from_utf8(credentials.clone()).unwrap_or("".to_string());
		self.agent.stream_to_channel(self.ctx, self.stream, cred, &self.state_rx,
			tx, rx)
	}
}

#![allow(dead_code)]

use std::sync::mpsc::{Sender,Receiver};
use std::vec::Vec;
use std::thread;
use libc;

use nice::agent::NiceAgent;
use nice::glib2::GMainLoop;
use nice::bindings_agent::GMainContext;

pub struct IceAgent {
	agent:    NiceAgent,
	ctx:      *mut GMainContext,
	stream:   u32,
	state_rx: Receiver<libc::c_uint>,
}

unsafe impl Send for IceAgent {}
unsafe impl Sync for IceAgent {}

impl IceAgent {
	pub fn new(controlling_mode: bool) -> Result<IceAgent,()>
	{
		let mainloop  = GMainLoop::new();
		let ctx       = mainloop.get_context() as *mut GMainContext;
		let mut agent = try!(NiceAgent::new(ctx, controlling_mode));

		let (stream, state_rx) = try!(agent.add_stream(Some("intercom")));

		thread::Builder::new().name("IceAgent::GMainLoop".to_string()).spawn(move || {
			mainloop.run();
		}).unwrap();

		agent.gather_candidates(stream);

		Ok(IceAgent {
			agent:    agent,
			ctx:      ctx,
			stream:   stream,
			state_rx: state_rx
		})
	}

	pub fn get_local_credentials(&mut self) -> Result<Vec<u8>,()> {
		let cred = try!(self.agent.generate_local_sdp());
		Ok(cred.into_bytes())
	}

	pub fn get_controlling_mode(&mut self) -> Result<bool,()> {
		self.agent.get_controlling_mode()
	}

	pub fn stream_to_channel(&mut self,
		                     credentials: &Vec<u8>,
	                         tx:          Sender<Vec<u8>>,
	                         rx:          Receiver<Vec<u8>>)
		-> Result<(), ()>
	{
		debug!("before {:?}", credentials);
		let cred = String::from_utf8(credentials.clone()).unwrap_or("".to_string());
		debug!("after");
		self.agent.stream_to_channel(self.ctx, self.stream,
			cred, &self.state_rx, tx, rx)
	}
}

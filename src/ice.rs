use std::thread::Thread;

use nice::agent::NiceAgent;
use nice::glib2::GMainLoop;
use nice::bindings_agent::GMainContext;

pub struct IceAgent {
	agent: NiceAgent,
}

impl IceAgent {
	pub fn new(controlling_mode: bool) -> Result<IceAgent,()>
	{
		let mainloop  = GMainLoop::new();
		let ctx       = mainloop.get_context() as *mut GMainContext;
		let mut agent = NiceAgent::new(ctx, controlling_mode);

		let stream       = try!(agent.add_stream(Some("ganymed")));
		let mut gathered = try!(agent.gather_candidates(stream));

		Thread::spawn(move || {
			mainloop.run();
		});

		gathered.get(); // blocking!

		Ok(IceAgent { agent: agent })
	}

	pub fn get_local_credentials(&self) -> String {
		self.agent.generate_local_sdp()
	}
}

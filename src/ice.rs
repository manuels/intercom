use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender,Receiver};

use condition_variable::ConditionVariable;
use utils::duplex_channel;

use nice::{Agent, NiceComponentState};

pub use nice::ControllingMode;
use nice::NiceComponentState::NICE_COMPONENT_STATE_READY;
use nice::NiceComponentState::NICE_COMPONENT_STATE_FAILED;


pub struct IceConnection {
	tx: Sender<Vec<u8>>,
	rx: Option<Receiver<Vec<u8>>>,
	cred_tx: Sender<Option<String>>,
	cred_rx: Receiver<String>,
	state: Arc<ConditionVariable<NiceComponentState>>,
}

impl IceConnection {
	pub fn new(controlling_mode: ControllingMode) -> IceConnection {
		let (your_ch, (my_tx, my_rx)): ((Sender<Vec<u8>>, _),(_,_)) = duplex_channel();

		let (state_tx, state_rx)       = channel();
		let (your_cred_ch, my_cred_ch) = duplex_channel();

		thread::spawn(move || {
			let component_id = 1;
			let (cred_tx, cred_rx) = my_cred_ch;
			let recv_cb = move |buf:&[u8]| my_tx.send(buf.to_vec()).unwrap();

			let agent  = Agent::new(controlling_mode);
			let stream = agent.add_stream("video", 1, recv_cb).unwrap();
			let state  = stream.get_state();

			state_tx.send(state.clone()).unwrap();

			let credentials = agent.generate_local_sdp().unwrap();
			cred_tx.send(credentials).unwrap();

			while state.get().unwrap() != NICE_COMPONENT_STATE_READY {
				for cred in cred_rx.iter().take_while(Option::is_some) {
					let cred:String = cred.unwrap();
					agent.parse_remote_sdp(&cred[..]);
				}

				state.wait_for_in(&[NICE_COMPONENT_STATE_READY,
				                    NICE_COMPONENT_STATE_FAILED]).unwrap();
			}
			info!("won't accept any remote credentials anymore: state={:?}", state.get().unwrap());

			for buf in my_rx {
				let len = stream.send(component_id, &buf[..]).unwrap();
				assert_eq!(len, buf.len());
			};
			unreachable!();
		});

		let (your_tx, your_rx) = your_ch;
		IceConnection {
			cred_tx: your_cred_ch.0,
			cred_rx: your_cred_ch.1,
			tx:      your_tx,
			rx:      Some(your_rx),
			state:   state_rx.recv().unwrap(),
		}
	}

	pub fn to_channel(&mut self, cred: String) -> Result<(Sender<Vec<u8>>, Receiver<Vec<u8>>),()> {
		self.set_remote_credentials(cred);

		let state_list = [NICE_COMPONENT_STATE_READY, NICE_COMPONENT_STATE_FAILED];

		let state = self.get_state();
		state.wait_for_in(&state_list).unwrap();
		self.cred_tx.send(None).unwrap();

		let s = state.get();
		match s {
			Ok(NICE_COMPONENT_STATE_READY) => {
				Ok((self.tx.clone(), self.rx.take().unwrap()))
			},
			_ => Err(()),
		}
		
	}

	pub fn get_local_credentials(&self) -> String {
		self.cred_rx.recv().unwrap()
	}

	fn set_remote_credentials(&self, cred: String) {
		self.cred_tx.send(Some(cred));//.unwrap();
	}

	pub fn get_state(&self) -> Arc<ConditionVariable<NiceComponentState>> {
		self.state.clone()
	}
}

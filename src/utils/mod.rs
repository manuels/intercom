use std::sync::mpsc::{channel,Sender,Receiver};

pub mod socket;
pub mod is_readable;
pub mod retry;
pub mod convert_dbus_item;
//pub use retry::retry;

pub fn ignore<T,F>(res: Result<T,F>) {
	match res {
		Ok(_) => (),
		Err(_) => (),
	}
}

pub fn duplex_channel<T,U>()
	-> ((Sender<T>, Receiver<U>), (Sender<U>, Receiver<T>))
{
	let (a_tx, b_rx) = channel();
	let (b_tx, a_rx) = channel();
	let a_ch = (a_tx, a_rx);
	let b_ch = (b_tx, b_rx);

	(a_ch, b_ch)
}
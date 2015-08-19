use std::sync::mpsc::{channel,Sender,Receiver};
use std::fmt::Debug;

pub mod posix;
pub mod socket;
pub mod is_readable;
pub mod retry;
pub mod convert_dbus_item;

macro_rules! error_on {
	($test:expr, $error:expr) => {
		if $test {
			return Err($error);
		}
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

#[test]
fn test_on_error() {
	let fail = || {
		error_on!(true, ());
		Ok(())
	};
	assert_eq!(fail(), Err(()));

	let success = || {
		error_on!(false, ());
		Ok(())
	};
	assert_eq!(success(), Ok(()));
}

pub trait ResultExpect<R> {
	fn expect<'a>(self, msg: &'a str) -> R;
}

impl<R,E: Debug> ResultExpect<R> for Result<R,E> {
	fn expect<'a>(self, msg: &'a str) -> R {
		match self {
			Ok(v)  => v,
			Err(e) => panic!("{}: {:?}", msg, e),
		}
	}
}

pub mod socket;
pub mod is_readable;
pub mod channel_to_rw;
pub mod retry;
pub mod convert_dbus_item;
//pub use retry::retry;

pub fn ignore<T,F>(res: Result<T,F>) {
	match res {
		Ok(_) => (),
		Err(_) => (),
	}
}

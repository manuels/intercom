use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::sync::{Arc,Mutex,Condvar};

use std::thread;

pub struct IsReadable;

impl IsReadable {
	pub fn new(rx: Receiver<Vec<u8>>) -> (Receiver<Vec<u8>>, Arc<(Mutex<bool>, Condvar)>) {
		let (tx, new_rx) = channel();

		let my_readable = Arc::new((Mutex::new(false), Condvar::new()));

		let your_is_readable = my_readable.clone();
		thread::Builder::new().name("IsReadable".to_string()).spawn(move || {
			for buf in rx.iter() {
				let &(ref lock, ref cvar) = &*my_readable;
				let mut is_readable = lock.lock().unwrap();
				
				tx.send(buf);
				
				*is_readable = true;
				cvar.notify_one();
    		}
		});

		(new_rx, your_is_readable)
	}
}

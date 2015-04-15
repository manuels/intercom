#![allow(dead_code)]

use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::sync::{Arc,Mutex,Condvar};

use std::thread;

pub struct IsReadable {
	is_readable: Arc<(Mutex<bool>, Condvar)>
}

impl IsReadable {
	pub fn new(rx: Receiver<Vec<u8>>) -> (Receiver<Vec<u8>>, IsReadable) {
		let (tx, new_rx) = channel();

		let my_readable = Arc::new((Mutex::new(false), Condvar::new()));

		let your_is_readable = my_readable.clone();
		thread::Builder::new().name("IsReadable".to_string()).spawn(move || {
			for buf in rx.iter() {
				let &(ref lock, ref cvar) = &*my_readable;
				let mut is_readable = lock.lock().unwrap();
				
				tx.send(buf).unwrap();
				
				*is_readable = true;
				cvar.notify_one();
    		}
		}).unwrap();

		(new_rx, IsReadable { is_readable: your_is_readable })
	}

	pub fn when_readable<F>(&mut self, mut blk: F) where F: FnMut()
	{
		let &(ref lock, ref cvar) = &*self.is_readable;

		let mut readable = lock.lock().unwrap();
		while !*readable {
			readable = cvar.wait(readable).unwrap();
		}

		blk();

		*readable = false;
	}

	pub fn unpack(self) -> Arc<(Mutex<bool>, Condvar)> {
		self.is_readable
	}
}

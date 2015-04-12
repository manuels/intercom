use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::{channel,sync_channel};
use std::sync::{Arc,Mutex,Condvar};
use std::vec::Vec;
use std::io::Result;
use std::thread;

pub struct Pipe;

impl Pipe {
	pub fn new() -> (ChannelToReadWrite,ChannelToReadWrite) {
		Pipe::new_from_channels(channel(), channel())
	}

	pub fn new_from_channels(chA: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
	                         chB: (Sender<Vec<u8>>, Receiver<Vec<u8>>))
	 -> (ChannelToReadWrite,ChannelToReadWrite)
	{
		let (txA, rxA) = chA;
		let (txB, rxB) = chB;

		let rwA = ChannelToReadWrite::new(txA, rxA);
		let rwB = ChannelToReadWrite::new(txB, rxB);

		(rwA, rwB)
	}
}

pub struct ChannelToReadWrite {
	tx:          Sender<Vec<u8>>,
	rx:          Receiver<Vec<u8>>,
	is_readable: Arc<(Mutex<bool>, Condvar)>,
}

impl ChannelToReadWrite {
	pub fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> ChannelToReadWrite {
		let is_readable = Arc::new((Mutex::new(false), Condvar::new()));

		let (txx, rxx) = channel();

		let rw = ChannelToReadWrite {
			tx:          tx,
			rx:          rxx,
			is_readable: is_readable.clone()
		};

		thread::Builder::new().name("ChannelToReadWrite::new".to_string()).spawn(move || {
			for buf in rx.iter() {
				debug!("ChannelToReadWrite -> ch.read() done (len={}) -> IS READABLE 1/1", buf.len());

				txx.send(buf).unwrap();

				let &(ref lock, ref cvar) = &*is_readable;
				let mut readable = lock.lock().unwrap();
				*readable = true;
				cvar.notify_one();
			}
			panic!("fin");
		}).unwrap();

		rw
	}

	pub fn is_readable(&self) -> Arc<(Mutex<bool>, Condvar)> {
		self.is_readable.clone()
	}
}

impl Read for ChannelToReadWrite {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		debug!("ChannelToReadWrite read()... 1/2");

		let data = if true {
			self.rx.recv().unwrap()
		} else {
			let data = self.rx.try_recv();
			if data.is_err() {
				debug!("ChannelToReadWrite read() len=0");
				return Ok(0)
			}
			data.unwrap()
		};

		debug!("ChannelToReadWrite read() len={}/{} done 2/2", data.len(), buf.len());

		let len = if buf.len() < data.len() { buf.len() } else { data.len() };

		for i in 0..len {
			buf[i] = data[i];
		}

		Ok(data.len())
	}
}

impl Write for ChannelToReadWrite {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
		debug!("ChannelToReadWrite write() len={}", buf.len());
    	self.tx.send(buf.to_vec()).unwrap();
    	
    	Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
		debug!("ChannelToReadWrite write flush() (noop)");
    	Ok(())
    }
}

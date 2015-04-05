use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::sync::{Arc,Condvar};
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
	is_readable: Arc<Condvar>,
}

impl ChannelToReadWrite {
	pub fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> ChannelToReadWrite {
		let is_readable = Arc::new(Condvar::new());

		let (txx, rxx) = channel();
		let rw = ChannelToReadWrite {
			tx:          tx,
			rx:          rxx,
			is_readable: is_readable.clone()
		};

		thread::Builder::new().name("ChannelToReadWrite::new".to_string()).spawn(move || {
			for buf in rx.iter() {
				(*is_readable).notify_one();
				txx.send(buf);
			}
			panic!("fin");
		});

		rw
	}

	pub fn is_readable(&self) -> Arc<Condvar> {
		self.is_readable.clone()
	}
}

impl Read for ChannelToReadWrite {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let data = self.rx.recv().unwrap();

		let len = if buf.len() < data.len() { buf.len() } else { data.len() };

		for i in 0..len {
			buf[i] = data[i];
		}

		Ok(data.len())
	}
}

impl Write for ChannelToReadWrite {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
    	self.tx.send(buf.to_vec()).unwrap();
    	
    	Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
    	Ok(())
    }
}

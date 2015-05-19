use std::io::{Result, Error, ErrorKind};
use std::io::{Read, Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::vec::Vec;

use syscalls;

pub struct ChannelToReadWrite {
	tx: Sender<Vec<u8>>,
	rx: Receiver<Vec<u8>>,
}

impl ChannelToReadWrite {
	pub fn new()
		-> (ChannelToReadWrite, (Sender<Vec<u8>>,Receiver<Vec<u8>>))
	{
		let (tx_a, rx_a) = channel();
		let (tx_b, rx_b) = channel();

		let rw = ChannelToReadWrite::new_from(tx_a, rx_b);

		(rw, (tx_b, rx_a))
	}

	pub fn new_from(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>)
		-> (ChannelToReadWrite)
	{
		ChannelToReadWrite {
			tx: tx,
			rx: rx,
		}
	}
}

impl Write for ChannelToReadWrite {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let res = self.tx.send(buf.to_vec());

		if res.is_ok() {
			Ok(buf.len())
		} else {
			Err(Error::new(ErrorKind::Other, "send failed"))
		}
	}

    fn flush(&mut self) -> Result<()> {
    	Ok(())
    }
}

impl Read for ChannelToReadWrite {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		match self.rx.recv() {
			Err(_) => Err(Error::new(ErrorKind::Other, "recv failed")),
			Ok(data) => {
				let len = if buf.len() > data.len() {
					data.len()
				} else {
					buf.len()
				};

				for i in 0..len {
					buf[i] = data[i];
				}
				Ok(len)
			}
		}
	}
}

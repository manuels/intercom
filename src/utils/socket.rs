use libc::types::os::arch::c95::{c_int,size_t};
use libc::types::common::c95::c_void;
use libc::funcs::bsd43::{send,recv};
use std::io::{Error, Result, ErrorKind};
use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::vec::Vec;
use std::thread;
use std::os::unix::io::{AsRawFd,RawFd};

use libc::consts::os::bsd44::AF_UNIX;

use syscalls;

pub struct ChannelToSocket {
	fd: RawFd
}

impl ChannelToSocket {
	#[allow(dead_code)]
	pub fn new(typ: c_int,
	           protocol: c_int)
		-> Result<(ChannelToSocket, (Sender<Vec<u8>>, Receiver<Vec<u8>>))>
	{
		let (tx_a, rx_a) = channel();
		let (tx_b, rx_b) = channel();
		let fd = try!(ChannelToSocket::new_from(typ, protocol, (tx_a, rx_b)));

		Ok((fd, (tx_b, rx_a)))
	}

	pub fn new_from(typ: c_int,
	                protocol: c_int,
	                ch: (Sender<Vec<u8>>,Receiver<Vec<u8>>))
		-> Result<ChannelToSocket>
	{
		let domain = AF_UNIX;

		let (tx, rx) = ch;
		let (my_fd, your_fd) = try!(syscalls::socketpair(domain, typ, protocol));

		let fd_other = your_fd;
		let fd_read  = my_fd;
		let fd_write = my_fd;

		thread::Builder::new().name("ChannelToSocket::new_from recv".to_string()).spawn(move || {
			loop {
				let mut buf = vec![0u8; 8*1024];

				debug!("ChannelToSocket sock-to-tx recv... fd={}", fd_read);
				let len = unsafe {
					recv(fd_read, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t, 0)
				};
				debug!("ChannelToSocket sock-to-tx recv()'d fd={} len={}", fd_read, len);

				if len != -1 {
					buf.truncate(len as usize);
					
					if tx.send(buf).is_err() {
						error!("Could not forward data from fd={} to fd={}", fd_read, fd_other);
					}
				} else {
					panic!(Error::last_os_error());
				}
			}
		}).unwrap();

		thread::Builder::new().name("ChannelToSocket::new_from send".to_string()).spawn(move || {
			for buf in rx.iter() {
				debug!("ChannelToSocket rx-to-sock send()'ing fd={} len={}", fd_write, buf.len());
				let len = unsafe {
					send(fd_write, buf.as_ptr() as *const c_void, buf.len() as size_t, 0)
				};
				debug!("ChannelToSocket rx-to-sock sent() fd={} len={}", fd_write, len);

				if (len as usize) != buf.len() {
					if len < 0 {
						panic!(Error::last_os_error());
					} else {
						let msg = format!("rx-to-sock Could not send full buffer! fd={} (only {} instead of {}", fd_write, len, buf.len());
						panic!(Error::new(ErrorKind::Other, &msg[..]));
					}
				}
			}
			panic!("fin");
		}).unwrap();

		Ok(ChannelToSocket {fd: your_fd})
	}
}

impl AsRawFd for ChannelToSocket {
	fn as_raw_fd(&self) -> RawFd {
		self.fd
	}
}

impl Read for ChannelToSocket {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let fd = self.as_raw_fd();
		let len = unsafe {
			recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as u64, 0)
		};

		if len < 0 {
			let err = Error::last_os_error();
			const EAGAIN:i32 = 11;

			if err.raw_os_error().unwrap() == EAGAIN {
				Ok(0)
			} else {
				Err(err)
			}
		} else {
			Ok(len as usize)
		}
	}
}

impl Write for ChannelToSocket {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let fd = self.as_raw_fd();
		let len = unsafe {
			send(fd, buf.as_ptr() as *const c_void, buf.len() as u64, 0)
		};

		if len < 0 {
			Err(Error::last_os_error())
		} else {
			Ok(len as usize)
		}
	}

    fn flush(&mut self) -> Result<()> {
    	Ok(())
    }
}

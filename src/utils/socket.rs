use libc::types::os::arch::c95::{c_int,size_t};
use libc::types::common::c95::c_void;
use libc::funcs::bsd43::{send,recv};
use std::io::{Error, ErrorKind};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::vec::Vec;
use std::thread;
use std::os::unix::io::RawFd;

use syscalls;

pub struct ChannelToSocket;

impl ChannelToSocket {
	pub fn new(domain: c_int, typ: c_int, protocol: c_int)
		-> Result<(RawFd, (Sender<Vec<u8>>, Receiver<Vec<u8>>)), Error>
	{
		let (txA, rxA) = channel();
		let (txB, rxB) = channel();
		let fd = try!(ChannelToSocket::new_from(domain, typ, protocol, txA, rxB));

		Ok((fd, (txB, rxA)))
	}

	fn new_from(domain: c_int, typ: c_int, protocol: c_int,
	            tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>)
		-> Result<RawFd, Error>
	{
		let (my_fd, your_fd) = try!(syscalls::socketpair(domain, typ, protocol));

		let fd_read = my_fd;
		let fd_write = my_fd;

		thread::Builder::new().name("ChannelToSocket::new_from recv".to_string()).spawn(move || {
			loop {
				let mut buf = vec![0u8; 8*1024];

				let len = unsafe {
					recv(fd_read, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t, 0)
				};

				if len > 0 {
					buf.truncate(len as usize);
					tx.send(buf);
				} else {
					panic!(Error::last_os_error());
				}
			}
		});

		thread::Builder::new().name("ChannelToSocket::new_from send".to_string()).spawn(move || {
			for buf in rx.iter() {
				let len = unsafe {
					send(fd_write, buf.as_ptr() as *const c_void, buf.len() as size_t, 0)
				};

				if (len as usize) != buf.len() {
					if len < 0 {
						panic!(Error::last_os_error());
					} else {
						let msg = format!("Could not send full buffer! (only {} instead of {}", len, buf.len());
						panic!(Error::new(ErrorKind::Other, msg.as_slice()));
					}
				}
			}
			panic!("fin");
		});

		Ok(your_fd)
	}
}

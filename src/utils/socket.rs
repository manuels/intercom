use libc::types::os::arch::c95::{c_int,size_t};
use libc::types::common::c95::c_void;
use libc::funcs::bsd43::{send,recv};
use libc::funcs::posix88::unistd::close;
use std::io::{Error, Result, ErrorKind,};
use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::vec::Vec;
use std::thread;
use std::os::unix::io::{AsRawFd,RawFd};

use libc::consts::os::bsd44::AF_UNIX;
use libc::consts::os::bsd44::SOCK_STREAM;
use libc::consts::os::posix88::EAGAIN;

use utils::duplex_channel;
use syscalls;

pub struct ChannelToSocket {
	fd: RawFd
}

impl ChannelToSocket {
	#[allow(dead_code)]
	pub fn new(typ:      c_int,
	           protocol: c_int)
		-> Result<(ChannelToSocket, (Sender<Vec<u8>>, Receiver<Vec<u8>>))>
	{
		let (my_ch, your_ch) = duplex_channel();
		let fd = try!(ChannelToSocket::new_from(typ, protocol, my_ch));

		Ok((fd, your_ch))
	}

	pub fn new_from(typ:      c_int,
	                protocol: c_int,
	                ch:       (Sender<Vec<u8>>,Receiver<Vec<u8>>))
		-> Result<ChannelToSocket>
	{
		let domain = AF_UNIX;
		let (my_fd, your_fd) = try!(syscalls::socketpair(domain, typ, protocol));

		let (tx, rx) = ch;
		let fd_read  = my_fd;
		let fd_write = my_fd;

		Self::spawn_recv(tx, fd_read, typ).unwrap();
		Self::spawn_send(rx, fd_write).unwrap();

		Ok(ChannelToSocket {fd: your_fd})
	}

	fn spawn_recv(tx:      Sender<Vec<u8>>,
	              fd_read: RawFd,
	              typ:     c_int)
		-> Result<thread::JoinHandle<()>>
	{
		let name = "ChannelToSocket::new_from recv".to_string();

		thread::Builder::new().name(name).spawn(move || {
			loop {
				let mut buf = vec![0u8; 16*1024];

				debug!("ChannelToSocket sock-to-tx recv... fd={}", fd_read);
				let len = unsafe {
					let ptr = buf.as_mut_ptr() as *mut c_void;
					let len = buf.len() as size_t;

					recv(fd_read, ptr, len, 0)
				};
				debug!("ChannelToSocket sock-to-tx recv()'d fd={} len={}", fd_read, len);

				if typ == SOCK_STREAM && len == 0 {
					debug!("Socket closed by us.");
					unsafe { close(fd_read); }
					break;
				}

				if len != -1 {
					buf.truncate(len as usize);
					
					if let Err(e) = tx.send(buf) {
						unsafe { close(fd_read) };
						panic!("Could not forward data from fd={}: {:?}",
						       fd_read, e);
					}
				} else {
					unsafe { close(fd_read) };
					panic!(Error::last_os_error());
				}
			}
		})
	}

	fn spawn_send(rx:      Receiver<Vec<u8>>,
	              fd_write: RawFd)
		-> Result<thread::JoinHandle<()>>
	{
		let name = "ChannelToSocket::new_from send".to_string();
		
		thread::Builder::new().name(name).spawn(move || {
			for buf in rx.iter() {
				debug!("ChannelToSocket rx-to-sock send()'ing fd={} len={}", fd_write, buf.len());
				let len = unsafe {
					let ptr = buf.as_ptr() as *const c_void;
					let len = buf.len() as size_t;

					send(fd_write, ptr, len, 0)
				};
				debug!("ChannelToSocket rx-to-sock sent() fd={} len={}", fd_write, len);

				if (len as usize) != buf.len() {
					unsafe { close(fd_write) };

					if len < 0 {
						panic!(Error::last_os_error());
					} else {
						let msg = format!("rx-to-sock Could not send full buffer! fd={} (only {} instead of {}",
						                  fd_write, len, buf.len());
						panic!(Error::new(ErrorKind::Other, &msg[..]));
					}
				}
			}
			panic!("fin");
		})
	}
}

impl AsRawFd for ChannelToSocket {
	fn as_raw_fd(&self) -> RawFd {
		self.fd
	}
}

impl Read for ChannelToSocket {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		thread::sleep_ms(250); // TODO to prevent race conditions

		let len = unsafe {
			let fd  = self.as_raw_fd();
			let ptr = buf.as_mut_ptr() as *mut c_void;
			let len = buf.len() as u64;

			recv(fd, ptr, len, 0)
		};

		if len < 0 {
			let err = Error::last_os_error();

			if err.raw_os_error().unwrap() == EAGAIN {
				debug!("recv={}", EAGAIN);
				Ok(0)
			} else {
				debug!("recv={}", err);
				Err(err)
			}
		} else {
			debug!("recv={}", len);
			Ok(len as usize)
		}
	}
}

impl Write for ChannelToSocket {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let len = unsafe {
			let fd  = self.as_raw_fd();
			let ptr = buf.as_ptr() as *const c_void;
			let len = buf.len() as u64;

			send(fd, ptr, len, 0)
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

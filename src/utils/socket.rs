use utils::posix::Posix;

use libc::types::os::arch::c95::{c_int,size_t};
use libc::types::common::c95::c_void;
use std::io::{Error, Result, ErrorKind};
use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::vec::Vec;
use std::thread;
use std::os::unix::io::{AsRawFd,RawFd};

use libc::funcs::bsd43::{send,recv};
use libc::funcs::posix88::unistd::close;
use libc::consts::os::bsd44::AF_UNIX;
use libc::consts::os::bsd44::SOCK_STREAM;
use libc::consts::os::posix88::EAGAIN;

use utils::duplex_channel;
use syscalls;

struct Fd {
	fd: RawFd,
}

impl AsRawFd for Fd {
	fn as_raw_fd(&self) -> RawFd {
		self.fd
	}
}

impl Posix for Fd {}

pub struct ChannelToSocket {
	typ: c_int,
	fd:  RawFd,
}

impl ChannelToSocket {
	#[allow(dead_code)]
	pub fn new(typ:      c_int,
	           protocol: c_int)
		-> Result<(ChannelToSocket, (Sender<Vec<u8>>, Receiver<Vec<u8>>))>
	{
		let (my_ch, your_ch) = duplex_channel();
		let c2s = try!(ChannelToSocket::new_from(typ, protocol, my_ch));

		Ok((c2s, your_ch))
	}

	pub fn new_from(typ:      c_int,
	                protocol: c_int,
	                ch:       (Sender<Vec<u8>>,Receiver<Vec<u8>>))
		-> Result<ChannelToSocket>
	{
		let (tx, rx) = ch;

		let (my_fd, your_fd) = try!(syscalls::socketpair(AF_UNIX, typ, protocol));

		Self::spawn_recv(tx, Fd {fd: my_fd}, typ).unwrap();
		Self::spawn_send(rx, Fd {fd: my_fd}).unwrap();

		Ok(ChannelToSocket {
			fd:  your_fd,
			typ: typ,
		})
	}

	fn spawn_recv(tx: Sender<Vec<u8>>, fd: Fd, typ: c_int)
		-> Result<thread::JoinHandle<()>>
	{
		let name = String::from("ChannelToSocket::new_from recv");

		thread::Builder::new().name(name).spawn(move || {
			loop {
				let mut buf = vec![0u8; 16*1024];

				match (typ, fd.recv(&mut buf)) {
					(SOCK_STREAM, Ok(len)) if len == 0 => {
						debug!("Socket closed by us.");
						break;
					},
					(_, Ok(len)) => {
						buf.truncate(len);
						tx.send(buf);
						continue
					}
					(_, Err(e)) => {
						warn!("Could not forward data: {:?}", e);
						break;
					}
				}
			}
			info!("fin (fd was closed)");
		})
	}

	fn spawn_send(rx: Receiver<Vec<u8>>, fd: Fd)
		-> Result<thread::JoinHandle<()>>
	{
		let name = String::from("ChannelToSocket::new_from send");
		
		thread::Builder::new().name(name).spawn(move || {
			for buf in rx {
				match fd.send(&buf[..]) {
					Ok(len) if len == buf.len() => continue,
					Ok(len) => {
						warn!("rx-to-sock Could not send full buffer! (only {} instead of {})",
							len, buf.len());
						break;
					},
					Err(e) => {
						warn!("{:?}", e);
						break;
					}
				}
			}

			fd.close();
			info!("fin (rx was closed)");
		})
	}
}

impl AsRawFd for ChannelToSocket {
	fn as_raw_fd(&self) -> RawFd {
		self.fd
	}
}

impl Posix for ChannelToSocket {}

impl Read for ChannelToSocket {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let res = self.recv(buf);

		match (self.typ, res) {
			(SOCK_STREAM, Ok(len)) if len == 0 => {
				Err(Error::new(ErrorKind::NotConnected, "Connection closed"))
			},
			(_, Ok(len)) => Ok(len),
			(_, Err(ref e)) if e.raw_os_error().unwrap() == EAGAIN => Ok(0),
			(_, Err(e)) => Err(e),
		}
	}
}

impl Write for ChannelToSocket {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.send(buf)
	}

    fn flush(&mut self) -> Result<()> {
    	Ok(())
    }
}

mod tests {
	use super::ChannelToSocket;
	use libc::consts::os::bsd44::SOCK_STREAM;
	use std::io::{Read,Write};
	use std::thread;

	#[test]
	fn test_close_write_tx() {
		if let Ok((mut c2s, (tx, rx))) = ChannelToSocket::new(SOCK_STREAM, 0) {
			drop(tx);
			thread::sleep_ms(3000);

			let buf = vec![0u8];
			assert!(c2s.write(&buf[..]).is_err());
		} else {
			unreachable!();
		}
	}
	#[test]
	fn test_close_read_tx() {
		if let Ok((mut c2s, (tx, rx))) = ChannelToSocket::new(SOCK_STREAM, 0) {
			drop(tx);
			thread::sleep_ms(3000);

			let mut buf = vec![0u8];
			assert!(c2s.read(&mut buf[..]).is_err());
		} else {
			unreachable!();
		}
	}

	/*
	#[test]
	fn test_close_rx() {
		let mut buf = vec![0u8];
		
		if let Ok((mut c2s, (tx, rx))) = ChannelToSocket::new(SOCK_STREAM, 0) {
			drop(rx);

			// This will still work, because:
			// write() -> recv() -> try(tx.send()) -> fail -> close(fd)
			assert!(c2s.write(&buf[..]).is_ok());
			assert!(c2s.read(&mut buf[..]).is_err());
		} else {
			unreachable!();
		}
	}
	*/
}

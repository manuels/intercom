use utils::posix::{Posix, SHUT_RDWR};

use libc::types::os::arch::c95::c_int;
use std::io::{Error, Result, ErrorKind};
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

struct Fd {
	fd: RawFd,
}

impl Clone for Fd {
	fn clone(&self) -> Self {
		Fd {fd: self.dup().unwrap()}
	}
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

		let fd1 = Fd {fd: my_fd};
		let fd2 = fd1.clone();
		Self::spawn_recv(tx, fd1, typ).unwrap();
		Self::spawn_send(rx, fd2).unwrap();

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

				info!("recv()...");
				match (typ, fd.recv(&mut buf)) {
					(SOCK_STREAM, Ok(len)) if len == 0 => {
						info!("Socket closed by us.");
						break;
					},
					(_, Ok(len)) => {
						info!("recv()'ed {} bytes", len);
						buf.truncate(len);
						
						if tx.send(buf).is_ok() {
							continue
						} else {
							break
						}
					}
					(_, Err(e)) => {
						warn!("Could not forward data: {:?}", e);
						break;
					}
				}
			}

			let err = fd.shutdown(SHUT_RDWR);
			info!("fin (rx was closed): {:?}", err);
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

			let err = fd.shutdown(SHUT_RDWR);
			info!("fin (rx was closed): {:?}", err);
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
	use std::io::{Read,Write};
	use std::thread;
	use std::sync::{Arc,Barrier};

	use libc::consts::os::bsd44::SOCK_STREAM;
	use libc::consts::os::bsd44::AF_UNIX;
	
	use super::ChannelToSocket;
	use utils::posix::{Posix, SHUT_RDWR};

	#[test]
	fn test_close_write_tx() {
		if let Ok((mut c2s, (tx, rx))) = ChannelToSocket::new(SOCK_STREAM, 0) {
			drop(tx);
			thread::sleep_ms(3500);

			let buf = vec![0u8];
			assert!(c2s.write(&buf[..]).is_err());
			drop(rx);
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
			drop(rx);
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
			thread::sleep_ms(3500);

			// This will still work, because:
			// write() -> recv() -> try(tx.send()) -> fail -> close(fd)
			let res = c2s.write(&buf[..]);
			error!("{:?}", res);
			assert!(res.is_err());
			assert!(c2s.read(&mut buf[..]).is_err());
		} else {
			unreachable!();
		}
	}
	*/
	#[test]
	fn test_shutdown_recv() {
		let barrier1 = Arc::new(Barrier::new(2));
		let barrier2 = barrier1.clone();

		let (fda1, _) = ::syscalls::socketpair(AF_UNIX, SOCK_STREAM, 0).unwrap();

		let fda1 = super::Fd {fd: fda1};
		let fda2 = fda1.clone();

		thread::spawn(move || {
			fda1.shutdown(SHUT_RDWR).unwrap();
			barrier1.wait();
		});

		barrier2.wait();
		let mut buf = vec![0;10];
		match fda2.recv(&mut buf[..]) {
			Ok(len) if len == 0 => (),
			_ => unreachable!(),
		}
	}

	#[test]
	fn test_shutdown_send() {
		let barrier1 = Arc::new(Barrier::new(2));
		let barrier2 = barrier1.clone();

		let (fda1, _) = ::syscalls::socketpair(AF_UNIX, SOCK_STREAM, 0).unwrap();

		let fda1 = super::Fd {fd: fda1};
		let fda2 = fda1.clone();

		thread::spawn(move || {
			fda1.shutdown(SHUT_RDWR).unwrap();
			barrier1.wait();
		});

		barrier2.wait();
		let buf = vec![0;10];
		match fda2.send(&buf[..]) {
			Ok(len) => panic!("Ok({}) should be an Err()", len),
			Err(_) => (),
		}
	}
}

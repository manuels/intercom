#![feature(io,io_ext,libc)]

extern crate libc;

use std::marker::Send;
use std::io;
use std::io::{Read,Write};
use std::io::Cursor;
use std::io::{Result,Error,ErrorKind};
use std::io::copy;
use std::thread::{Builder,JoinHandle};
use std::thread::spawn;

use std::sync::{Arc,Mutex,Barrier};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender,Receiver,TryRecvError};
use std::vec::Vec;

use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use libc::funcs::posix88::unistd::close;
use libc::funcs::bsd43::{recv,send};
use libc::consts::os::posix88::EPIPE;

use libc::types::os::arch::c95::c_int;
use libc::types::common::c95::c_void;
use libc::types::os::arch::c95::size_t;

use libc::consts::os::bsd44::SOCK_STREAM;
use libc::consts::os::bsd44::AF_UNIX;

use syscalls;

use std::time::duration::Duration;
use std::old_io::timer::sleep;

/*
pub trait RedirectTo {
	//fn redirect_to<W>(mut self, mut writer: W) -> Result<JoinHandle>
	fn redirect_to<W>(&self, writer: &Arc<Mutex<W>>) -> Result<JoinHandle>
		where W: Write+Send+'static;
}

impl<R> RedirectTo for Arc<Mutex<R>> where R: Read+Send+'static {
	fn redirect_to<W>(&self, writer: &Arc<Mutex<W>>) -> Result<JoinHandle>
		where W: Write+Send+'static
	{
		let inlock = self.clone();
		let outlock = writer.clone();
		Builder::new().name("RedirectTo".to_string()).spawn(move || {
			let mut buf = [0u8; 8*1024];
			loop {
				let mut input = inlock.lock().unwrap();

				// if len == 0 on EOF we will have an infinite loop
				// so input is required to throw an error on EOF!
				let len = match input.read(&mut buf) {
					Ok(len) => len,
					Err(e) => {
						match e.kind() {
							ErrorKind::Interrupted => { break },
							_ => panic!(e),
						}
					}
				};

				let mut output = outlock.lock().unwrap();
				output.write(&buf[..len]).unwrap();
				drop(output);
			}
		})
	}
}

struct TestReader {
	c: u8
}

impl TestReader {
	fn new() -> TestReader {
		TestReader { c: 0 }
	}
}

impl Read for TestReader {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		if self.c > 100 {
			Err(Error::new(ErrorKind::Interrupted, "EOF"))
		} else {
			buf[0] = self.c as u8;
			self.c += 1;

			Ok(1)
		}
	}
}

#[test]
fn test_redirect_to() {
	let input = Arc::new(Mutex::new(TestReader::new()));
	let output = Arc::new(Mutex::new(io::sink()));

	let t = input.redirect_to(&output).unwrap();
	drop(input);
	t.join();
}
*/

pub struct FdIo {
	fd: RawFd
}

impl FdIo {
	pub fn from_fd(fd: RawFd) -> FdIo {
		FdIo {fd: fd}
	}
}

pub trait AsIo {
	fn as_io(self) -> FdIo;
}

impl<F> AsIo for F where F: AsRawFd {
	fn as_io(self) -> FdIo {
		FdIo {
			fd: self.as_raw_fd()
		}
	}
}

impl Drop for FdIo {
	fn drop(&mut self) {
		unsafe {
			close(self.fd);
		}
	}
}

impl Read for FdIo {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let flags = 0;

		let fd = self.fd;
		let len = unsafe {
			info!("recv {}", fd);
			recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t, flags)
		};
			info!("recv'd {} len={}", fd, len);

		if len < 0 {
			Err(Error::last_os_error())
		} else {
			Ok(len as usize)
		}
	}
}

impl Write for FdIo {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		let flags = 0;

		let fd = self.fd;
		let len = unsafe {
			send(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags)
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

/*
#[test]
fn test_as_io() {
	let (fd1,fd2) = ::syscalls::socketpair(AF_UNIX, SOCK_STREAM, 0).unwrap();
	let (fd3,fd4) = ::syscalls::socketpair(AF_UNIX, SOCK_STREAM, 0).unwrap();

	let mut io1 = FdIo::from_fd(fd1);
	let mut io2 = Arc::new(Mutex::new(FdIo::from_fd(fd2)));
	let mut io3 = Arc::new(Mutex::new(FdIo::from_fd(fd3)));
	let mut io4 = FdIo::from_fd(fd4);

	let t1 = io2.redirect_to(&io3).unwrap();
	let t2 = io3.redirect_to(&io2).unwrap();

	io1.write("foo".as_bytes());
	io4.write("bar".as_bytes());

	info!("a");
	let mut buf = [0u8; 3];
	info!("b");
	io1.read(&mut buf);
	info!("c");
	assert!(buf == "bar".as_bytes());
	info!("cc");
	io4.read(&mut buf);
	assert!(buf == "foo".as_bytes());
	info!("d");

	//sleep(Duration::seconds(1));

	drop(io1);
	drop(io4);

	t1.join();
	t2.join();
}
*/
trait IsReadable {
	fn is_readable(&self) -> bool;
}
/*
struct ToChannel;

impl ToChannel
{
	pub fn to_channel<T>(rw_mutex: &Arc<Mutex<T>>, maybe_readable: &Arc<Barrier>)
		-> (Sender<Vec<u8>>, Receiver<Vec<u8>>)
		where T: IsReadable+Read+Write+Send+'static
	{
		let (my_tx, your_rx) = channel();
		let (your_tx, my_rx): (_,Receiver<Vec<u8>>) = channel();

		let reader = rw_mutex.clone();
		let writer = rw_mutex.clone();
		let maybe_readable = maybe_readable.clone();

		spawn(move || {
			loop {
				maybe_readable.wait();

				let mut rw = reader.lock().unwrap();
				while rw.is_readable() {
					let mut buf = vec![0u8; 8*1024];
					let len = {
						rw.read(buf.as_mut_slice()).unwrap()
					};
					buf.truncate(len);
					my_tx.send(buf);
				}
			}
		});

		spawn(move || {
			loop {
				my_rx.recv().map(|buf| {
					let mut rw = writer.lock().unwrap();
					rw.write(buf.as_slice());
				});
			}
		});

		(your_tx, your_rx)
	}
}


pub struct ChannelToReadWrite {
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
}

impl ChannelToReadWrite {
    pub fn new(tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> 
        ChannelToReadWrite
    {
        ChannelToReadWrite {
            tx: tx,
            rx: rx,
        }
    }
}

impl Read for ChannelToReadWrite {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.rx.try_recv() {
            Ok(v) => {
            	let len = match buf.len() > v.len() {
            		true => v.len(),
            		false => buf.len(),
            	};
            	for i in 0..len {
            		buf[i] = v[i]
            	}
            	Ok(len)
            },
            Err(TryRecvError::Empty) => Ok(0),
            Err(TryRecvError::Disconnected) => {
                Err(Error::new(ErrorKind::ConnectionAborted, "Disconnected"))
            },
        }
    }
}

impl Write for ChannelToReadWrite {
    fn flush(&mut self) -> Result<()> {
    	Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.tx.send(buf.to_vec()) {
            Ok(()) => Ok(buf.len()),
            Err(_) => {
                Err(Error::new(ErrorKind::ConnectionAborted, "Disconnected?"))
            }
        }
    }
}

#[test]
fn test_read_channel_to_read_write() {
	let (tx, rrx) = channel();
	let (ttx, rx) = channel();

	let mut rw = ChannelToReadWrite::new(ttx, rrx);

	tx.send(vec![1,2,3]);
	let mut buf = vec![0u8; 1024];
	let len = rw.read(buf.as_mut_slice()).unwrap();
	assert!(len == 3);

	tx.send(vec![4,5]);
	let mut buf = vec![0u8; 1024];
	let len = rw.read(buf.as_mut_slice()).unwrap();
	assert!(len == 2);
}

fn socketpair(domain: c_int, typ: c_int, protocol: c_int)
	-> (RawFd, (Sender<Vec<u8>>, Receiver<Vec<u8>>))
{
	let (my_tx, your_rx) = channel();
	let (your_tx, my_rx) = channel();

	let (my_fd, your_fd) = syscalls::socketpair(domain, typ, protocol);

	spawn(|| {
		loop {
			let flags = 0;
			let buf = vec![0u8; 8*1024];
			let len = unsafe {
				recv(my_fd, buf.as_mut_ptr() as *mut c_void,
					buf.len() as size_t, flags)
			};

			match len {
				 0 => {},
				-1 => break,
				 _ => {
					buf.truncate(len);
					my_tx.send(buf);
				 }
			}
		}
	});

	spawn(|| {
		loop {
			let flags = 0;
			let buf = my_rx.recv();
			let len = unsafe {
				send(my_fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags);
			};

			if len != buf.len() {
				panic!("send() failed {}!={}!", len, buf.len());
			}
		}
	});

	(your_fd, (your_tx, your_rx))
}
*/
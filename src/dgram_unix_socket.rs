use std::os::unix::prelude::RawFd;
use libc::funcs::bsd43::send;
use libc::funcs::bsd43::recv;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use libc::types::common::c95::c_void;
use std::os;

pub struct DgramUnixSocket {
	fd: RawFd
}

impl DgramUnixSocket {
	pub fn new(fd: RawFd) -> DgramUnixSocket {
		DgramUnixSocket {
			fd: fd
		}
	}
}

/*
impl AsRawFd for DgramUnixSocket {
	fn as_raw_fd(&self) -> Fd {
		self.fd
	}
}
*/

impl Read for DgramUnixSocket {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize,Error> {
		let flags = 0;
		let ptr = buf.as_mut_ptr() as *mut c_void;

		let len = unsafe {
			recv(self.fd, ptr, buf.len() as u64, flags)
		};

		match len {
			/*
			-1 => {
				match errno() {
					EAGAIN => Err(Error::new(ErrorKind::Interrupted, "EAGAIN", None)),
					_ => Err(Error::new(ErrorKind::Other,
							"recv() returned -1",
							Some(os::error_string(os::errno() as i32)))),
				}
			},*/
			0 => Err(Error::new(ErrorKind::Other,
				"connection is closed")),
			_ => Ok(len as usize),
		}
	}
}

impl Write for DgramUnixSocket {
	fn write(&mut self, buf: &[u8]) -> Result<usize,Error> {
		let flags = 0;
		let ptr = buf.as_ptr() as *const c_void;

		let res = unsafe {
			send(self.fd, ptr, buf.len() as u64, flags)
		};
		if res == (buf.len() as i64) {
			Ok(res as usize)
		} else {
			Err(Error::last_os_error())
		}
	}

	fn flush(&mut self) -> Result<(),Error> {
		Ok(())
	}
}


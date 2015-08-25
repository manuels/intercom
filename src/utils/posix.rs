use std::io::{Error,Result};
use std::os::unix::io::AsRawFd;

use libc::funcs::posix88::unistd::dup;
pub use libc::consts::os::bsd44::SHUT_RDWR;
use libc::funcs::bsd43::{send,recv,shutdown};
use libc::funcs::posix88::unistd::close;
use libc::types::os::arch::c95::c_int;
use libc::types::common::c95::c_void;

pub trait Posix: AsRawFd {
	fn send(&self, buf: &[u8]) -> Result<usize> {
		let res = unsafe {
			let ptr = buf.as_ptr() as *const c_void;
			let len = buf.len() as u64;

			send(self.as_raw_fd(), ptr, len, 0)
		};
		if res > -1 {
			Ok(res as usize)
		} else {
			Err(Error::last_os_error())
		}
	}

	fn recv(&self, buf: &mut[u8]) -> Result<usize> {
		let res = unsafe {
			let ptr = buf.as_mut_ptr() as *mut c_void;
			let len = buf.len() as u64;

			recv(self.as_raw_fd(), ptr, len, 0)
		};
		if res > -1 {
			Ok(res as usize)
		} else {
			Err(Error::last_os_error())
		}
	}

	fn close(&self) -> Result<()> {
		let res = unsafe {
			close(self.as_raw_fd())
		};
		if res == 0 {
			Ok(())
		} else {
			Err(Error::last_os_error())
		}
	}

	fn shutdown(&self, flags: c_int) -> Result<()> {
		let res = unsafe {
			shutdown(self.as_raw_fd(), flags)
		};
		if res == 0 {
			Ok(())
		} else {
			Err(Error::last_os_error())
		}
	}

	fn dup(&self) -> Result<c_int> {
		let res = unsafe {
			dup(self.as_raw_fd())
		};
		if res > -1 {
			Ok(res)
		} else {
			Err(Error::last_os_error())
		}
	}
}

use libc::types::os::arch::c95::c_int;
use std::os::unix::io::RawFd;
use std::io::{Result,Error};

const O_NONBLOCK: c_int = 00004000;
const F_GETFL: c_int = 3;
const F_SETFL: c_int = 4;

mod syscall {
	use libc::types::os::arch::c95::c_int;

	extern "C" {
		pub fn socketpair(domain: c_int, typ: c_int, protocol: c_int, sv: *mut c_int) -> c_int;

		pub fn fcntl(fd: c_int, cmd: c_int, flags: c_int) -> c_int;
	}
}

pub fn socketpair(domain: c_int, typ: c_int, protocol: c_int)
	-> Result<(RawFd, RawFd)>
{
	let mut sv = [-1 as RawFd; 2];

	let res = unsafe {
		syscall::socketpair(domain, typ, protocol, sv.as_mut_ptr())
	};

	match res {
		0 => Ok((sv[0], sv[1])),
		_ => Err(Error::last_os_error())
	}
}

pub fn set_blocking(fd: c_int, blocking: bool) -> Result<()> {
	let flags = unsafe { syscall::fcntl(fd, F_GETFL, 0) };
	if flags < 0 {
		return Err(Error::last_os_error());
	}

	let flags = if blocking { flags & !O_NONBLOCK } else { flags|O_NONBLOCK };
	let res = unsafe { syscall::fcntl(fd, F_SETFL, flags) };
	if res != 0 {
		return Err(Error::last_os_error());
	}

	Ok(())
}

mod tests {
	use libc::types::os::arch::c95::size_t;
	use libc::funcs::bsd43::send;
	use libc::funcs::posix88::unistd::close;
	use libc::consts::os::bsd44::AF_UNIX;
	use libc::consts::os::bsd44::{SOCK_STREAM, SOCK_DGRAM};
	use std::thread;
	use std::sync::{Barrier, Arc};

	#[test]
	fn test_socketpair_close() {
		if let Ok((s1, s2)) = super::socketpair(AF_UNIX, SOCK_DGRAM, 0) {
			let buf = vec![0u8];
			let res = unsafe {
				close(s1);
				send(s2, buf.as_ptr() as *const _, buf.len() as size_t, 0)
			};
			assert_eq!(res, -1);
		} else {
			unreachable!();
		}
	}

	#[test]
	fn test_socketpair_close_spawn() {
		if let Ok((s1, s2)) = super::socketpair(AF_UNIX, SOCK_STREAM, 0) {
			let barrier1 = Arc::new(Barrier::new(2));
			let barrier2 = barrier1.clone();

			let s = s1;
			thread::spawn(move || {
				unsafe { close(s) };
				barrier2.wait();
			});

			barrier1.wait();

			let buf = vec![0u8];
			let res = unsafe {
				send(s2, buf.as_ptr() as *const _, buf.len() as size_t, 0)
			};
			assert_eq!(res, -1);
		} else {
			unreachable!();
		}
	}
}

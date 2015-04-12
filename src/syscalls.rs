use libc::types::os::arch::c95::c_int;
use std::os::unix::io::RawFd;
use std::io::Error;

mod syscall {
	use libc::types::os::arch::c95::c_int;

	extern "C" {
		pub fn socketpair(domain: c_int, typ: c_int, protocol: c_int, sv: *mut c_int) -> c_int;

		pub fn fcntl(fd: c_int, cmd: c_int, flags: c_int) -> c_int;
	}
}

pub fn socketpair(domain: c_int, typ: c_int, protocol: c_int)
	-> Result<(RawFd, RawFd), Error>
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

pub unsafe fn fcntl(fd: c_int, cmd: c_int, flags: c_int) -> c_int {
	syscall::fcntl(fd, cmd, flags)
}

extern crate libc;

use std::os::unix::io::RawFd;
use std::ptr;

use glib::g_variant::GVariant;
use bindings_glib::gint;

pub struct GDBusMethodInvocation {
	ptr: *mut i32
}

extern "C" {
	fn g_dbus_method_invocation_return_dbus_error(ptr: *mut i32,
		error_name: *const u8,
		error_msg:  *const u8);

	fn g_unix_fd_list_new_from_array(ptr: *const i32, size: gint)
		-> *mut i32;

	fn g_unix_fd_list_new() -> *mut i32;

	fn g_unix_fd_list_append(list: *mut i32, fd: libc::c_int, err: *mut *mut i32)
		-> libc::c_int;

	fn g_dbus_method_invocation_return_value(invoc: *mut i32,
		result: *mut i32);

	fn g_dbus_method_invocation_return_value_with_unix_fd_list(
		invoc: *mut i32, result: *mut i32, fds: *mut i32);
}

impl GDBusMethodInvocation {
	pub fn new(ptr: *mut i32) -> GDBusMethodInvocation {
		assert!(!ptr.is_null());

		GDBusMethodInvocation {ptr: ptr}
	}

	pub fn return_dbus_error<'a>(&self, error_name: &'a str, error_msg: &'a str)
	{
		unsafe {
			g_dbus_method_invocation_return_dbus_error(self.ptr as *mut i32,
				error_name.as_ptr(),
				error_msg.as_ptr());
		}
	}

//	pub fn return_result(&self, tuple: &GVariant, fds: Vec<RawFd>) {
	pub fn return_result(&self, tuple: &GVariant, fds: Vec<RawFd>) {
		unsafe {
			if fds.len() > 0 {
				let fd_list = if true {
					for f in fds.iter() {
						assert!(*f >= 0);
						debug!("fd={}", *f);
					}

					let fd_list = g_unix_fd_list_new_from_array(
						fds.as_ptr(),
						fds.len() as gint);
					fd_list
				} else {
					let fd_list = g_unix_fd_list_new();

					for f in fds.iter() {
						assert!(*f >= 0);
						debug!("fd={}", *f);

						let mut err = ptr::null_mut();
						let idx = g_unix_fd_list_append(fd_list, *f, &mut err);
						assert!(err.is_null());
						assert!(idx >= 0);
					}
					fd_list
				};
				assert!(!fd_list.is_null());
				
				assert!(!tuple.as_ptr().is_null());
				assert!(tuple.is_tuple());
				assert!(tuple.len() >= fds.len());

				info!("return_result called (fds_len={}) ;)", fds.len());
				g_dbus_method_invocation_return_value_with_unix_fd_list(self.ptr,
					tuple.as_ptr(), fd_list);
			}
			else {
				g_dbus_method_invocation_return_value(self.ptr, tuple.as_ptr());
			}
		}
	}
}

unsafe impl Send for GDBusMethodInvocation {}

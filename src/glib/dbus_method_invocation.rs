use std::os::unix::io::RawFd;

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

	pub fn return_result(&self, tuple: &GVariant, fds: Vec<RawFd>) {
		unsafe {
			if fds.len() > 0 {
				let fd_list = g_unix_fd_list_new_from_array(
					fds.as_slice().as_ptr(),
					fds.len() as gint);
				assert!(!fd_list.is_null());
				
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

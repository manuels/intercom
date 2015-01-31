use libc::types::os::arch::c95::c_long;

pub struct GDBusMethodInvocation {
	ptr: c_long
}

extern "C" {
	fn g_dbus_method_invocation_return_dbus_error(ptr: *mut i32,
		error_name: *const u8,
		error_msg:  *const u8);
}

impl GDBusMethodInvocation {
	pub fn new(ptr: *mut i32) -> GDBusMethodInvocation {
		assert!(!ptr.is_null());

		GDBusMethodInvocation {ptr: ptr as c_long}
	}

	pub fn return_dbus_error<'a>(&self, error_name: &'a str, error_msg: &'a str)
	{
		unsafe {
			g_dbus_method_invocation_return_dbus_error(self.ptr as *mut i32,
				error_name.as_ptr(),
				error_msg.as_ptr());
		}
	}
}

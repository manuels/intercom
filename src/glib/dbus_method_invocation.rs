pub struct GDBusMethodInvocation {
	ptr: *mut i32
}

extern "C" {
	fn g_dbus_method_invocation_return_dbus_error(ptr: *mut i32,
		error_name: *const u8,
		error_msg:  *const u8);
}

impl GDBusMethodInvocation {
	pub fn new(ptr: *mut i32) -> GDBusMethodInvocation {
		assert!(!ptr.is_null());

		GDBusMethodInvocation {ptr: ptr}
	}

	pub fn return_dbus_error<'a>(&self, error_name: &'a str, error_msg: &'a str)
	{
		unsafe {
			g_dbus_method_invocation_return_dbus_error(self.ptr,
				error_name.as_ptr(), error_msg.as_ptr());
		}
	}
}

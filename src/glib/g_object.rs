use std::mem;

use bindings_glib::g_signal_connect_data;

pub struct GObject {
	ptr: *mut i32,
}

impl GObject {
	pub fn from_ptr(ptr: *mut i32) -> GObject {
		assert!(!ptr.is_null());

		GObject {ptr: ptr}
	}

	pub fn connect_signal<U>(&self,
		signal_name: &str,
		callback: extern fn(),
		user_data: Box<U>)
	{
		let res = unsafe {
			g_signal_connect_data(self.ptr,
				signal_name.as_ptr(),
				Some(callback),
				mem::transmute(user_data),
				None,
				1)
		};
		assert!(res > 0);
	}
}

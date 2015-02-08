#![allow(unstable)]

use bindings_glib::{gsize,guchar, gint32};

use libc::types::os::arch::c95::c_int;
use std::os::unix::Fd;
use std::mem;

extern "C" {
	fn g_variant_new_fixed_array(typ: *const u8,
		elements: *const u8,
		n_elements: gsize,
		element_size: gsize
	) -> *mut c_int;

	fn g_variant_get_fixed_array(value: *const c_int,
		n_element: *mut gsize,
		typ: gsize,
	) -> *mut u8;

	fn g_variant_n_children(value: *const c_int) -> gsize;

	fn g_variant_get_child_value(value: *const c_int, index: gsize)
		-> *mut c_int;

	fn g_variant_new_handle(fd: gint32) -> *mut c_int;

	fn g_variant_new_tuple(children: *const *mut i32, n_children: gsize) -> *mut c_int;

	fn g_variant_ref(value: *mut c_int);
	fn g_variant_unref(value: *mut c_int);
}

pub struct GVariant {
	ptr: *mut c_int,
}

impl Drop for GVariant {
	fn drop(&mut self) {
		unsafe {
			g_variant_unref(self.ptr)
		}
	}
}

impl GVariant {
	pub fn from_ptr(ptr: *mut c_int) -> GVariant {
		assert!(mem::size_of::<gsize>() == mem::size_of::<usize>());

		assert!(!ptr.is_null());

		unsafe {
			g_variant_ref(ptr);
		}

		GVariant{ptr:ptr}
	}

	pub fn from_vec(vec: &Vec<u8>) -> GVariant {
		assert!(mem::size_of::<gsize>() == mem::size_of::<usize>());

		let ptr = unsafe {
			let typ = "y".as_ptr();
			g_variant_new_fixed_array(typ, vec.as_slice().as_ptr(),
				vec.len() as gsize, mem::size_of::<u8>() as gsize)
		};

		GVariant::from_ptr(ptr)
	}

	pub fn new_tuple(children: Vec<GVariant>) -> GVariant {
		let mut pointers = vec![];
		for c in children.iter() {
			unsafe {
				pointers.push(c.as_ptr());
			}
		}

		let ptr = unsafe { g_variant_new_tuple(pointers.as_ptr(), children.len() as gsize) };
		GVariant::from_ptr(ptr)
	}

	pub fn from_fd(fd: Fd) -> GVariant {
		let ptr = unsafe { g_variant_new_handle(fd as gint32) };

		GVariant::from_ptr(ptr)
	}

	pub fn to_vec(&self) -> Vec<u8> {
		// TODO assure type is ay!!

		assert!(mem::size_of::<gsize>() == mem::size_of::<usize>());

		let el_size = mem::size_of::<guchar>() as gsize;
		let mut len = 0 as gsize;

		let ptr = unsafe {
			g_variant_get_fixed_array(self.ptr, &mut len, el_size)
		};
		assert!(!ptr.is_null());

		unsafe {
			Vec::from_raw_buf(ptr, len as usize)
		}
	}

	pub fn to_vec_vec(&self) -> Vec<Vec<u8>> {
		unsafe {
			let len = g_variant_n_children(self.ptr);

			let mut vec = Vec::with_capacity(len as usize);
			for i in range(0, len) {
				let ptr = g_variant_get_child_value(self.ptr, i);
				assert!(!ptr.is_null());

				let child = GVariant{ ptr: ptr};
				vec.push(child.to_vec());
			}

			vec
		}
	}

	pub unsafe fn as_ptr(&self) -> *mut c_int {
		assert!(!self.ptr.is_null());
		self.ptr
	}
}

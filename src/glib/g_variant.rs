#![allow(dead_code)]

use bindings_glib::{gsize,guchar, gint32};

use libc::types::os::arch::c95::c_int;
use std::os::unix::io::RawFd;
use std::mem;
use std::slice;
use std::ffi::{CStr,CString};
use std::str;

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

	fn g_variant_new(typ: *const i8, fd: *mut c_int) -> *mut c_int;

	fn g_variant_new_tuple(children: *const *mut i32, n_children: gsize) -> *mut c_int;

	fn g_variant_ref(value: *mut c_int);
	fn g_variant_unref(value: *mut c_int);

	fn g_variant_get_type(ptr: *mut c_int) -> *const c_int;

	fn g_variant_type_is_array(t: *const c_int) -> c_int;
	fn g_variant_type_is_tuple(t: *const c_int) -> c_int;

	fn g_variant_type_dup_string(t: *const c_int) -> *const c_int;
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
			g_variant_new_fixed_array(typ, vec.as_ptr(),
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

		let ptr = unsafe {
			g_variant_new_tuple(pointers.as_ptr(), children.len() as gsize)
		};
		assert!(!ptr.is_null());

		GVariant::from_ptr(ptr)
	}

	pub fn from_fd(fd: RawFd) -> GVariant {
		assert!(fd >= 0);

		let ptr = unsafe { g_variant_new_handle(fd as gint32) };
		assert!(!ptr.is_null());

		GVariant::from_ptr(ptr)
	}

	pub fn new_fd_tuple(fd: RawFd) -> GVariant {
		assert!(fd >= 0);

		let fd = GVariant::from_fd(fd);
		let typ = CString::new("(@h)").unwrap();
		let ptr = unsafe { g_variant_new(typ.as_ptr(), fd.as_ptr()) };
		assert!(!ptr.is_null());

		GVariant::from_ptr(ptr)
	}

	pub fn is_array(&self) -> bool {
		unsafe {
			let t = g_variant_get_type(self.ptr);
			g_variant_type_is_array(t) != 0
		}
	}

	pub fn is_tuple(&self) -> bool {
		unsafe {
			let t = g_variant_get_type(self.ptr);
			g_variant_type_is_tuple(t) != 0
		}
	}

	pub fn to_vec(&self) -> Vec<u8> {
		assert!(mem::size_of::<gsize>() == mem::size_of::<usize>());

		let el_size = mem::size_of::<guchar>() as gsize;
		let mut len = 0 as gsize;

		assert!(self.is_array());

		let ptr = unsafe {
			g_variant_get_fixed_array(self.ptr, &mut len, el_size)
		};
		assert!(!ptr.is_null());

		unsafe {
			slice::from_raw_parts(ptr, len as usize).to_vec()
		}
	}

	pub fn len(&self) -> usize {
		unsafe {
			g_variant_n_children(self.ptr) as usize
		}
	}

	pub fn type_string(&self) -> &str {
		let slice = unsafe {
			let t = g_variant_get_type(self.ptr);
			let ptr = g_variant_type_dup_string(t);
			CStr::from_ptr(ptr as *mut i8)
		};

		str::from_utf8(slice.to_bytes()).unwrap()
	}

	pub fn to_vec_vec(&self) -> Vec<Vec<u8>> {
		unsafe {
			let mut vec = Vec::with_capacity(self.len() as usize);
			for i in 0..self.len() {
				let ptr = g_variant_get_child_value(self.ptr, i as gsize);
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

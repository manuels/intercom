#![allow(unstable)]

use std::ffi;
use std::str;
use std::ffi::CString;

pub trait FromUtf8Pointer {
	unsafe fn from_utf8_pointer(ptr: &*const i8) -> Result<Self, str::Utf8Error>;
}

impl FromUtf8Pointer for String {
	unsafe fn from_utf8_pointer(ptr: &*const i8) -> Result<Self, str::Utf8Error> {
		assert!(!ptr.is_null());

		let array = ffi::c_str_to_bytes(ptr);
		let utf8 = str::from_utf8(array);

		utf8.map(|s| {s.to_string()})
	}
}

pub fn cstr(string: &str) -> CString {
	CString::from_slice(string.as_bytes())
}

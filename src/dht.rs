#![allow(dead_code)]

use std::time::duration::Duration;

use ::DHT;
use bindings_glib::GBusType::*;
use bindings_lunadht;
use bindings_lunadht::{
	luna_dht_proxy_new_for_bus_sync, luna_dht_call_get_sync,
	luna_dht_call_put_sync};
use glib::g_variant::GVariant;

const APP_ID:i32 = 8877;

struct LunaDHT {
	proxy: *mut bindings_lunadht::_LunaDHT,
}

// TODO implement Drop

impl LunaDHT {
	pub fn new() -> LunaDHT {
		let name = "org.manuel.ganymed";
		let object_path = "/";
		let mut err = 0 as *mut i32;

		let bus_type = G_BUS_TYPE_SESSION | G_BUS_TYPE_STARTER;

		let proxy = unsafe {
			luna_dht_proxy_new_for_bus_sync(bus_type.bits(), 0, 
				name.as_ptr() as *const i32,
				object_path.as_ptr() as *const i32,
				0 as *mut i32,
				&mut err)
		};
		assert!(!err.is_null());

		LunaDHT {
			proxy: proxy
		}
	}
}

impl DHT for LunaDHT {
	fn get(&self, key: Vec<u8>) -> Result<Vec<Vec<u8>>,()> {
		let mut out = 0 as *mut i32;
		let mut err = 0 as *mut i32;

		let gkey = GVariant::from_vec(key);

		let res = unsafe {
			luna_dht_call_get_sync(self.proxy,
				APP_ID, gkey.as_ptr(), &mut out, 0 as *mut i32, &mut err)
		};
		if res == 0 {
			return Err(());
		}

		let results = GVariant::from_ptr(out);
		Ok(results.to_vec_vec())
	}

	fn put(&self, key: Vec<u8>, value: Vec<u8>, ttl: Duration) -> Result<(),()>
	{
		let mut err = 0 as *mut i32;
		let ttl_sec = ttl.num_seconds() as i32;

		let gkey = GVariant::from_vec(key);
		let gvalue = GVariant::from_vec(value);

		let res = unsafe {
			luna_dht_call_put_sync(self.proxy, APP_ID, gkey.as_ptr(),
				gvalue.as_ptr(), ttl_sec, 0 as *mut i32, &mut err)
		};
		if res == 0 {
			return Err(());
		}

		Ok(())
	}
}

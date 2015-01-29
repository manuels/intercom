#![feature(link_args)]
#![allow(unstable)]

#[macro_use] extern crate rustc_bitflags;
#[macro_use] extern crate log;
extern crate time;
extern crate libc;
extern crate nice;

use std::os::unix::Fd;
use std::sync::Future;
use std::time::duration::Duration;

use dbus_service::DbusService;

mod dht;
mod dbus_service;
mod dbus_request;
mod ice;
mod bindings_lunadht;
mod bindings_glib;
mod bindings_ganymed;
mod glib;
mod from_pointer;

trait DbusResponder {
	fn send(&self, fd: Fd) -> Result<(),()>;
	fn send_error<T>(&self, err:T) -> Result<(),()>;
}

trait DHT {
	fn get(&self, key: Vec<u8>) -> Result<Vec<Vec<u8>>,()>;
	fn put(&self, key: Vec<u8>, value: Vec<u8>, ttl: Duration) ->  Result<(),()>;
}

fn main() {
	let mut dbus_service = DbusService::new("org.manuel.ganymed");
	let local_public_key = "foobar";
	//TODO:
	// - publish public key

	for (request, responder) in dbus_service {
		Future::spawn(move || {
			request.handle(local_public_key)
				.and_then(|fd| responder.send(fd))
				.or_else(|err| responder.send_error(err))
		});
	}
}

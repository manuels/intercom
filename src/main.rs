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
mod fake_dht;

enum ConnectError {
	REMOTE_CREDENTIALS_NOT_FOUND,
	FOO,
}

trait DbusResponder {
	fn respond(&self, fd: Fd) -> Result<(),()>;
	fn respond_error(&self, err: ConnectError) -> Result<(),()>;
}

trait DHT {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()>;
	fn put(&self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration) ->  Result<(),()>;
}

fn main() {
	let mut dbus_service = DbusService::new("org.manuel.ganymed");
	let local_public_key = "(local_public_key)";
	//TODO:
	// - publish public key

	for request in dbus_service {
		Future::spawn(move || {
			match request.handle(local_public_key) {
				Ok(fd) =>   request.respond(fd),
				Err(err) => request.respond_error(err)
			}
		});
	}
}

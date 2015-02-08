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
use std::sync::{Arc, Mutex};
use std::env;

use dbus_service::DbusService;
use fake_dht::FakeDHT;

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
mod utils;

enum ConnectError {
	REMOTE_CREDENTIALS_NOT_FOUND,
	FOO,
}

trait DbusResponder {
	fn respond(&self, result: Result<Fd,ConnectError>) -> Result<(),()> {
		match result {
			Ok(fd) => self.respond_ok(fd),
			Err(e) => self.respond_error(e),
		}
	}
	fn respond_ok(&self, fd: Fd) -> Result<(),()>;
	fn respond_error(&self, err: ConnectError) -> Result<(),()>;
}

trait DHT {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()>;
	fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration) ->  Result<(),()>;
}

fn main() {
	let mut args = env::args();
	args.next();
	let dbus_service = DbusService::new(args.next().unwrap().into_string().unwrap().as_slice());
	let local_public_key = args.next().unwrap().into_string().unwrap().as_slice().as_bytes().to_vec();
	//TODO:
	// - publish public key

	for request in dbus_service {
		let local_key = local_public_key.clone();
		Future::spawn(move || {
			let mut dht = FakeDHT::new();
			let result = request.handle(local_key, &mut dht);
			request.invocation.respond(result).unwrap();
		});
	}
}

#![feature(link_args)]
#![allow(unstable)]

#[macro_use] extern crate rustc_bitflags;
#[macro_use] extern crate log;
extern crate time;
extern crate libc;
extern crate nice;
extern crate ecdh;
extern crate openssl;

use std::os::unix::Fd;
use std::sync::Future;
use std::time::duration::Duration;
use std::sync::{Arc, Mutex};
use std::env;
use std::borrow::Borrow;

use dbus_service::DBusService;
use fake_dht::FakeDHT;
use ecdh::public_key::PublicKey;
use ecdh::ecdh::ECDH;
use ecdh::private_key::PrivateKey;

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

trait DBusResponder {
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
	let dbus_path = args.next().unwrap();
	let local_private_key = args.next().unwrap().into_bytes().map_in_place(|x| x as i8);

	let dbus_service = DBusService::new(dbus_path.borrow());

	for request in dbus_service {
		let my_private_key = local_private_key.clone();

		Future::spawn(move || {
			let my_private_key = PrivateKey::from_vec(&my_private_key).unwrap();

			let my_public_key   = my_private_key.get_public_key();
			let your_public_key = PublicKey::from_vec(&request.remote_public_key.clone().map_in_place(|x| x as i8)).unwrap();

			let mut my_hash   = vec![];
			let mut your_hash = vec![];
			my_hash.push_all(my_public_key.to_vec().map_in_place(|x| x as u8).as_slice());
			my_hash.push_all(your_public_key.to_vec().map_in_place(|x| x as u8).as_slice());
			your_hash.push_all(your_public_key.to_vec().map_in_place(|x| x as u8).as_slice());
			your_hash.push_all(my_public_key.to_vec().map_in_place(|x| x as u8).as_slice());

			let shared_key = ECDH::compute_key(&my_private_key, &your_public_key).unwrap();

			let mut dht = FakeDHT::new();
			let result = request.handle(&my_private_key,
			                            &my_public_key,
			                            &your_public_key,
			                            &shared_key.to_vec(),
			                            &my_hash,
			                            &your_hash,
			                            &mut dht);
			request.invocation.respond(result).unwrap();
		});
	}
}

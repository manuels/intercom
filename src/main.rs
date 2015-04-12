#[macro_use] extern crate bitflags;
#[macro_use] extern crate log;
extern crate time;
extern crate libc;
extern crate nice;
extern crate ecdh;
extern crate openssl;
extern crate byteorder;

use std::os::unix::io::RawFd;
use std::thread;
use time::Duration;
use std::env;
use std::borrow::Borrow;
use std::fmt::{Display,Formatter,Error};
use std::io::Cursor;

use std::fs::File;

use dbus_service::DBusService;
use fake_dht::FakeDHT;
use ecdh::public_key::PublicKey;
use ecdh::ecdh::ECDH;
use ecdh::private_key::PrivateKey;

use openssl::crypto::pkey::PKey;
use openssl::crypto::hash::Type::SHA256;
use openssl::x509::{X509,X509Generator,KeyUsage,ExtKeyUsage};

mod dht;
//mod dgram_unix_socket;
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
mod syscalls;
mod ssl;

#[derive(Debug)]
pub enum ConnectError {
	REMOTE_CREDENTIALS_NOT_FOUND,
	FOO,
}

impl Display for ConnectError {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
		match self {
			REMOTE_CREDENTIALS_NOT_FOUND => fmt.write_str("REMOTE_CREDENTIALS_NOT_FOUND"),
			//_ => fmt.write_str("undef"),
		}
	}
}

pub trait DBusResponder {
	fn respond(&self, result: Result<RawFd,ConnectError>) -> Result<(),()> {
		match result {
			Ok(fd) => self.respond_ok(fd),
			Err(e) => self.respond_error(e),
		}
	}
	fn respond_ok(&self, fd: RawFd) -> Result<(),()>;
	fn respond_error(&self, err: ConnectError) -> Result<(),()>;
}

trait DHT {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()>;
	fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration) ->  Result<(),()>;
}

fn generate_cert(private_key: &PrivateKey) -> Result<X509,()> {
	let mut buf = Cursor::new(vec![0u8; 4*1024]);

	private_key.to_pem(&mut buf).unwrap();
	buf.set_position(0);
	let pkey = try!(PKey::private_key_from_pem(&mut buf).map_err(|_| ()));

	let gen = X509Generator::new()
		.set_valid_period(365*2)
		//.set_CN("test_me")
		.set_sign_hash(SHA256)
		.set_usage(&[KeyUsage::KeyAgreement])
		.set_ext_usage(&[ExtKeyUsage::ClientAuth, ExtKeyUsage::ServerAuth]);

	let cert = try!(gen.sign(&pkey).map_err(|_| ()));

	let mut file = File::create("/tmp/foo.txt").unwrap();
	cert.write_pem(&mut file).unwrap();

	Ok(cert)
}

fn main() {
	let mut args = env::args();
	args.next();
	let dbus_path = args.next().unwrap();
	let local_private_key = args.next().unwrap().into_bytes();

	let dbus_service = DBusService::new(dbus_path.borrow());

	for request in dbus_service {
		let my_private_key = local_private_key.clone();

		thread::spawn(move || {
			let my_private_key = PrivateKey::from_vec(&my_private_key).unwrap();

			let my_public_key   = my_private_key.get_public_key();
			let your_public_key = PublicKey::from_vec(&request.remote_public_key.clone()).unwrap();

			let my_hash   = my_public_key.to_vec() + &your_public_key.to_vec()[..];
			let your_hash = your_public_key.to_vec() + &my_public_key.to_vec()[..];

			let shared_key = ECDH::compute_key(&my_private_key, &your_public_key).unwrap();

			let cert = generate_cert(&my_private_key).unwrap();

			let mut dht = FakeDHT::new();
			let result = request.handle(&my_private_key,
			                            &my_public_key,
			                            &your_public_key,
			                            &shared_key.to_vec(),
			                            &my_hash,
			                            &your_hash,
			                            &cert,
			                            &mut dht);
			request.invocation.respond(result).unwrap();
		});
	}
}

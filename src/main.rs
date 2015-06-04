#[macro_use] extern crate bitflags;
#[macro_use] extern crate log;
extern crate time;
extern crate libc;
extern crate nice;
extern crate ecdh;
extern crate openssl;
extern crate nonblocking_socket;
extern crate byteorder;
extern crate env_logger;
extern crate pseudotcp;
extern crate rustc_serialize;

#[cfg(feature="dbus")]
extern crate dbus;

use std::os::unix::io::RawFd;
use std::thread;
use time::Duration;
use std::env;
use std::borrow::Borrow;
use std::io::{Read,Cursor};

use std::fs::File;

use ecdh::public_key::PublicKey;
use ecdh::ecdh::ECDH;
use ecdh::private_key::PrivateKey;

use openssl::crypto::pkey::PKey;
use openssl::crypto::hash::Type::SHA256;
use openssl::x509::{X509,X509Generator,KeyUsage,ExtKeyUsage};

mod dht;
#[cfg(feature="dbus")]
mod dbus_service;
mod dbus_request;
mod ice;
mod bindings_lunadht;
mod bindings_glib;
mod glib;
mod utils;
mod syscalls;
mod ssl;
mod intercom;
mod connection;
mod tests;

#[cfg(feature="dbus")]
use dbus_service::DBusService;
#[cfg(feature="dbus")]
use dbus::BusType;

use intercom::Intercom;

trait DHT {
       fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()>;
       fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration) ->  Result<(),()>;
}

fn generate_cert(private_key: &PKey) -> Result<X509,()> {
	let gen = X509Generator::new()
		.set_valid_period(365*2)
		//.set_CN("test_me")
		.set_sign_hash(SHA256)
		.set_usage(&[KeyUsage::KeyAgreement])
		.set_ext_usage(&[ExtKeyUsage::ClientAuth, ExtKeyUsage::ServerAuth]);

	let cert = try!(gen.sign(&private_key).map_err(|_| ()));

	let mut file = File::create("/tmp/foo.txt").unwrap();
	cert.write_pem(&mut file).unwrap();

	Ok(cert)
}


fn main() {
	env_logger::init().unwrap();

	start_intercom(env::args());
}

fn start_intercom<I:Iterator<Item=String>>(mut args: I) {
	args.next();
	let dbus_path = args.next().unwrap();
	let local_private_key = args.next().unwrap();

	let mut file = File::open(local_private_key).unwrap();
	let mut local_private_key = String::new();
	file.read_to_string(&mut local_private_key).unwrap();

	let intercom = Intercom::new(&local_private_key.into_bytes()).unwrap();
	DBusService::serve(intercom, &dbus_path[..], BusType::Session);
}

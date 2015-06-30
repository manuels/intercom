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
extern crate docopt;

#[cfg(feature="dbus")]
extern crate dbus;

#[cfg(not(test))]
use std::env;
use std::io::Read;

use docopt::Docopt;
use std::fs::File;

#[cfg(feature="dbus")]
mod dbus_service;
mod ice;
mod utils;
mod syscalls;
mod ssl;
mod intercom;
mod connection;
mod shared_secret;
#[cfg(test)]
mod tests;

#[cfg(feature="dbus")]
use dbus_service::DBusService;
#[cfg(feature="dbus")]
use dbus::BusType;

use intercom::Intercom;

static USAGE: &'static str = "
Usage: intercom [options]

Options:
    --private-key <file>   Use private key from a file
                           [default: ~/.config/intercom/private_key].
    --dbus <service>       DBus service name [default: org.manuel.intercom].
";

#[derive(RustcDecodable,Debug)]
struct Args {
  flag_private_key: Option<String>,
  flag_dbus:        Option<String>,
}

#[cfg(not(test))]
fn main() {
	env_logger::init().unwrap();

	start_intercom(env::args());
}

fn start_intercom<I:Iterator<Item=String>>(args: I) {
	let args: Args = Docopt::new(USAGE)
	                  .and_then(|d| d.argv(args).decode())
	                  .unwrap_or_else(|e| e.exit());

	let dbus_service      = args.flag_dbus.unwrap_or("org.manuel.intercom".to_string());
	let private_key_fname = args.flag_private_key.unwrap_or("~/.config/intercom/private_key".to_string());

	let mut file = File::open(private_key_fname).unwrap();
	let mut local_private_key = String::new();
	file.read_to_string(&mut local_private_key).unwrap();

	let intercom = Intercom::new(&local_private_key.into_bytes()).unwrap();
	DBusService::serve(intercom, &dbus_service[..], BusType::Session).unwrap();
}

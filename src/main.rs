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
use std::os::unix::fs::PermissionsExt;

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

use std::path::PathBuf;

#[cfg(feature="dbus")]
use dbus_service::DBusService;
#[cfg(feature="dbus")]
use dbus::BusType;

use intercom::Intercom;
use utils::ResultExpect;

static USAGE: &'static str = "
Usage: intercom [options]

Options:
    --private-key <file>   Use private key from a file
                           (default: $HOME/.config/intercom/private_key).
    --dbus <service>       DBus service name [default: org.manuel.intercom].
    --help                 Print this help.
";

#[derive(RustcDecodable,Debug)]
struct Args {
  flag_private_key: Option<String>,
  flag_dbus:        String,
}

#[cfg(not(test))]
fn main() {
	env_logger::init().unwrap();

	start_intercom(env::args());
}

fn start_intercom<I:Iterator<Item=String>>(args: I) {
	let mut home = std::env::home_dir()
	                    .unwrap_or(PathBuf::from("/tmp/"));

	let args: Args = Docopt::new(USAGE)
	                  .and_then(|d| d.argv(args).decode())
	                  .unwrap_or_else(|e| e.exit());

	let private_key_fname = args.flag_private_key
		.unwrap_or_else(|| {
			home.push(".config/intercom/private_key".to_string());
			home.as_path().to_str().unwrap().to_string()
		});

	let mut file = File::open(private_key_fname.clone())
	                    .expect(&format!("Could not open private key file '{}'", private_key_fname)[..]);

	let metadata = file.metadata().expect(&format!("Error reading permissions for '{}'", private_key_fname)[..]);
	if metadata.permissions().mode() != 0o400 {
		error!("Your private key file '{}' has {:o} permissions! It should be 400",
			private_key_fname, metadata.permissions().mode());
		return
	}

	let mut local_private_key = vec![0; 1024];
	let len = file.read(&mut local_private_key[..])
	              .expect(&format!("Could not read private key file '{}'", private_key_fname)[..]);
	local_private_key.truncate(len);

	let intercom = Intercom::new(&local_private_key).unwrap();

	let dbus_service = args.flag_dbus;
	DBusService::serve(intercom, &dbus_service[..], BusType::Session)
		.expect("Error listening on DBus");
}

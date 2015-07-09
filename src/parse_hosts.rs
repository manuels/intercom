use std::fs::File;
use std::io::Result;
use std::io::{BufReader,BufRead};
use std::collections::HashMap;

use rustc_serialize::hex::FromHex;

use ecdh;

struct Host {
	public_key: ecdh::PublicKey,
}

fn parse_hosts_file(fname: String) -> Result<HashMap<String,Host>> {
	let mut hashmap = HashMap::new();
	let file        = try!(File::open(fname.clone()));

	for (i, line) in BufReader::new(file).lines().enumerate() {
		match line {
			Err(e) => return Err(e),
			Ok(line) => {
				let fields:Vec<&str> = line.split(" ").collect();

				if fields.len() != 2 {
					warn!("Failed reading line {} of hosts file '{}'", i, fname);
					continue;
				}

				let (hostname, pub_key) = (fields[0], fields[1]);

				let vec = pub_key.from_hex();
				if vec.is_err() {
					warn!("Failed reading public key for '{}' in line {} of hosts file '{}'",
					                                      hostname, i, fname);
					continue;
				}

				match ecdh::PublicKey::from_vec(&vec.unwrap()) {
					Err(()) => warn!("Failed reading public key for '{}' in line {} of hosts file '{}'",
					                                      hostname, i, fname),
					Ok(public_key) => {
						let host = Host {
							public_key: public_key,
						};

						hashmap.insert(hostname.to_string(), host);
					}
				}
			}
		}
	}

	Ok(hashmap)
}

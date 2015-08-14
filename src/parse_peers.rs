#![allow(dead_code)]

use std::fs::File;
use std::io::Result;
use std::io::{BufReader,BufRead};
use std::collections::HashMap;

use rustc_serialize::hex::FromHex;

use ecdh;

pub struct Peer {
	pub public_key: ecdh::PublicKey,
}

pub fn parse_peers_file(fname: String) -> Result<HashMap<String,Peer>> {
	let mut hashmap = HashMap::new();
	let file        = try!(File::open(fname.clone()));

	for (i, line) in BufReader::new(file).lines().enumerate() {
		match line {
			Err(e) => return Err(e),
			Ok(line) => {
				let fields:Vec<&str> = line.split(" ").collect();

				if fields.len() != 2 {
					warn!("Failed reading line {} of peers file '{}'", i, fname);
					continue;
				}

				let (peername, pub_key) = (fields[0], fields[1]);

				let vec = pub_key.from_hex();
				if vec.is_err() {
					warn!("Failed reading public key for '{}' in line {} of peers file '{}'",
					                                      peername, i, fname);
					continue;
				}

				match ecdh::PublicKey::from_vec(&vec.unwrap()) {
					Err(()) => warn!("Failed reading public key for '{}' in line {} of peers file '{}'",
					                                      peername, i, fname),
					Ok(public_key) => {
						let peer = Peer {
							public_key: public_key,
						};

						hashmap.insert(peername.to_string(), peer);
					}
				}
			}
		}
	}

	Ok(hashmap)
}

use std::time::duration::Duration;
use std::io::{File, Open, Read, Write, Append};

pub struct FakeDHT {
	path: Path,
}

impl FakeDHT {
	pub fn new() -> FakeDHT {
		FakeDHT {
			path: Path::new("/tmp/fake_dht.txt"),
		}
	}
}

impl ::DHT for FakeDHT {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()> {
		let mut file = File::open_mode(&self.path, Open, Read).unwrap();

		let contents = file.read_to_string().unwrap();

		let k = String::from_utf8(key.clone()).unwrap();

		let mut last_match = None;
		//for line in contents.as_slice().lines() {
		for entries in contents.as_slice().split('\0') {
			if entries.starts_with(k.as_slice()) {
				let res = entries.slice_from(key.len());
				last_match = Some(res.to_string().into_bytes());
			}
		}

		match last_match {
			None =>    Ok(vec![]),
			Some(m) => Ok(vec![m]),
		}
	}

	fn put(&self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration)
		-> Result<(),()>
	{
		let mut file = File::open_mode(&self.path, Append, Write).unwrap();

		file.write(key.as_slice());
		file.write(value.as_slice());
		file.write_str("\n");
		Ok(())
	}
}
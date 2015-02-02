use std::time::duration::Duration;
use std::sync::{Arc, Mutex};
use std::old_io::{File, Open, Read, Write, Append};

pub struct FakeDHT {
	path: Arc<Mutex<Path>>,
}

impl FakeDHT {
	pub fn new() -> FakeDHT {
		FakeDHT {
			path: Arc::new(Mutex::new(Path::new("/tmp/fake_dht.txt"))),
		}
	}

	pub fn clone(&self) -> FakeDHT {
		FakeDHT {
			path: self.path.clone(),
		}
	}
}

impl ::DHT for FakeDHT {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()> {
		let path = self.path.lock().unwrap();
		let mut file = File::open_mode(&*path, Open, Read).unwrap();
		debug!("pre");

		let contents = file.read_to_string().unwrap();

		let k = String::from_utf8(key.clone()).unwrap();

		let mut last_match = None;
		for entries in contents.as_slice().split('\0') {
			if entries.starts_with(k.as_slice()) {
				let res = entries.slice_from(key.len());
				last_match = Some(res.to_string().into_bytes());
			}
		}

		debug!("get(): {:?}=len({:?})",
			::std::str::from_utf8(key.as_slice()),
			last_match.clone().map(|x| x.len()));

		debug!("post");
		match last_match {
			None =>    Ok(vec![]),
			Some(m) => Ok(vec![m]),
		}
	}

	fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, ttl: Duration)
		-> Result<(),()>
	{
		let path = self.path.lock().unwrap();
		let mut file = File::open_mode(&*path, Append, Write).unwrap();
		debug!("pre");

		debug!("put(): {:?}={:?}",
			::std::str::from_utf8(key.as_slice()),
			value.len());

		file.write(key.as_slice());
		file.write(value.as_slice());
		file.write_str("\0");
		debug!("post");
		Ok(())
	}
}

#[aacfg(test)]
mod test {
	use std::time::duration::Duration;
	use ::fake_dht::FakeDHT;
	use ::DHT;

	fn test_fake_dht() {
		let mut dht = FakeDHT::new();
		
		let key = vec![3,2,1];
		let value = vec![4,3,2,1];

		dht.put(&key, &value, Duration::minutes(10));

		let val = dht.get(&key).unwrap().get(0).unwrap().clone();
		assert!(val == value);
	}
}

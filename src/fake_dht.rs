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

		let mut last_match = None;
		loop {
			let klen = file.read_le_u32();
			if klen.is_err() { break; }

			let klen = klen.unwrap() as usize;
			let ekey = file.read_exact(klen).unwrap();

			let vlen = file.read_le_u32().unwrap() as usize;
			let eval = file.read_exact(vlen).unwrap();

			if &ekey.as_slice() == key {
				last_match = Some(eval);
			}
		}

		debug!("get(): {:?}=len({:?})",
			::std::str::from_utf8(key.as_slice()),
			last_match.clone().map(|x| x.len()));

		match last_match {
			None =>    Ok(vec![]),
			Some(m) => Ok(vec![m]),
		}
	}

	fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, _: Duration)
		-> Result<(),()>
	{
		let path = self.path.lock().unwrap();
		let mut file = File::open_mode(&*path, Append, Write).unwrap();

		debug!("put(): {:?}=len({:?})",
			::std::str::from_utf8(key.as_slice()),
			value.len());

		file.write_le_u32(key.len() as u32).unwrap();
		file.write_all(key.as_slice()).unwrap();

		file.write_le_u32(value.len() as u32).unwrap();
		file.write_all(value.as_slice()).unwrap();

		Ok(())
	}
}

#[cfg(test)]
mod test {
	use std::time::duration::Duration;
	use ::fake_dht::FakeDHT;
	use ::DHT;

	#[test]
	fn test_fake_dht() {
		let mut dht = FakeDHT::new();
		
		let key = vec![3,2,1];
		let value = vec![9,8,7,6];

		dht.put(&key, &value, Duration::minutes(10)).unwrap();

		let val = dht.get(&key).unwrap().get(0).unwrap().clone();
		assert!(val == value);
	}
}

use time::Duration;
use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::io::{Read,Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::path::Path;

pub struct FakeDHT {
	m: Arc<Mutex<i32>>,
}

impl FakeDHT {
	pub fn new() -> FakeDHT {
		FakeDHT {
			m: Arc::new(Mutex::new(0)),
		}
	}

	pub fn clone(&self) -> FakeDHT {
		FakeDHT {
			m: self.m.clone(),
		}
	}
}

impl ::DHT for FakeDHT {
	fn get(&self, key: &Vec<u8>) -> Result<Vec<Vec<u8>>,()> {
		let lock = self.m.lock().unwrap();
		let mut file = OpenOptions::new().read(true).open("/tmp/fake_dht.txt").unwrap();

		let mut last_match = None;
		loop {
			let klen = file.read_u32::<LittleEndian>();
			if klen.is_err() { break; }

			let klen = klen.unwrap() as usize;
			let mut ekey = Vec::new();
			ekey.resize(klen, 0);
			file.read(ekey.as_mut_slice()).unwrap();

			let vlen = file.read_u32::<LittleEndian>();
			let vlen = vlen.unwrap() as usize;
			let mut eval = Vec::new();
			eval.resize(vlen, 0);
			file.read(eval.as_mut_slice()).unwrap();

			if &ekey == key {
				last_match = Some(eval);
			}
		}

		debug!("get(): {:?}=len({:?})",
			::std::str::from_utf8(key.as_slice()),
			last_match.clone().map(|x| x.len()));

		drop(lock);
		match last_match {
			None =>    Ok(vec![]),
			Some(m) => Ok(vec![m]),
		}
	}

	fn put(&mut self, key: &Vec<u8>, value: &Vec<u8>, _: Duration)
		-> Result<(),()>
	{
		let lock = self.m.lock().unwrap();
		let mut file = OpenOptions::new().write(true).append(true).open("/tmp/fake_dht.txt").unwrap();

		debug!("put(): {:?}=len({:?})",
			::std::str::from_utf8(key.as_slice()),
			value.len());

		file.write_u32::<LittleEndian>(key.len() as u32).unwrap();
		file.write_all(key.as_slice()).unwrap();

		file.write_u32::<LittleEndian>(value.len() as u32).unwrap();
		file.write_all(value.as_slice()).unwrap();

		drop(lock);
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use time::Duration;
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

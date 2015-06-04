use std::thread::sleep_ms;

use time::{Duration,SteadyTime};

pub fn retry<F,R,E>(timeout: Duration, retry_time: Duration, mut blk: F) -> Result<R,E>
	where F: FnMut() -> Result<R,E>
{
	let end = SteadyTime::now() + timeout;

	let mut begin = SteadyTime::now();
	let mut result = blk();

	while result.is_err() && SteadyTime::now() < end {
		let wait = retry_time - (SteadyTime::now() - begin);
		let wait_ms = wait.num_milliseconds();
		if wait_ms > 0 {
			sleep_ms(wait_ms as u32);
		}

		begin = SteadyTime::now();
		result = blk();
	}
	
	result
}

#[test]
fn test_retry_ok() {
	let retry_time = Duration::zero();

	let func = || -> Result<i32, &str> { Ok(0) };
	let res = retry(Duration::seconds(10), retry_time, func);
	assert_eq!(Ok(0), res);

	let func = || -> Result<i32, &str> { sleep_ms(100); Ok(0) };
	let res = retry(Duration::milliseconds(10), retry_time, func);
	assert_eq!(Ok(0), res);

	let mut counter = 0;
	let func = || -> Result<i32, &str> {
		sleep_ms(100);
		counter += 1;

		if counter > 1 {
			Ok(0)
		} else {
			Err("foo")
		}
	};
	let res = retry(Duration::milliseconds(10), retry_time, func);
	assert_eq!(Err("foo"), res);

	let mut counter = 0;
	let func = || -> Result<i32, &str> {
		sleep_ms(10);
		counter += 1;

		if counter > 1 {
			Ok(0)
		} else {
			Err("foo")
		}
	};
	let res = retry(Duration::milliseconds(100), retry_time, func);
	assert_eq!(Ok(0), res);
}

extern crate env_logger;

use std::thread::{spawn,sleep_ms};
#[allow(unused_imports)]

use libc::consts::os::bsd44::{SOCK_DGRAM, SOCK_STREAM};
use libc::funcs::bsd43::{send,recv};
use libc::{size_t,c_void};

use dbus::Connection as DbusConnection;
use dbus::{Message,MessageItem,BusType};

use ::start_intercom;

#[test]
fn test_intercom() {
	env_logger::init().unwrap();

	spawn(|| {
		let args = "intercom --private-key ./private-key1 --dbus org.manuel.TestIntercom1";
		let args:Vec<String> = args.split(" ")
			.map(|s| s.to_string())
			.collect();
		assert_eq!(args, vec!["intercom","--private-key","./private-key1","--dbus","org.manuel.TestIntercom1"]);
		start_intercom(args.into_iter());
	});
		let args = "intercom --private-key ./private-key2 --dbus org.manuel.TestIntercom2";
		let args:Vec<String> = args.split(" ")
			.map(|s| s.to_string())
			.collect();
		debug!("{:?}", args);
	spawn(|| {

		start_intercom(args.into_iter());
	});
	sleep_ms(1000);

	let public_key1 = "0200bacee5e8690cea0f64403802fe22817804760c9bdb937acbf13c009f770120b8b481147861d0a4edc4bc2e8bf1645e91ef570b4feea8b701d557e79f11658a0daf";
	let public_key2 = "030184408030d8307535e48a9d499b25cef86c3a68ae2dcef6366acc433c840d74907e94cb0a65390569905735c676abc0d90f8f974f2dac66edbca38e3fd153d4743c";

	let sock_type = SOCK_STREAM;//SOCK_DGRAM;
	let app_id1 = "test1";
	let app_id2 = "test2";

	spawn(move || {
		let conn = DbusConnection::get_private(BusType::Session).unwrap();
		let mut msg = Message::new_method_call("org.manuel.TestIntercom1", "/",
		                                       "org.manuel.Intercom", "ConnectToKey").unwrap();
		msg.append_items(&[MessageItem::Int32(sock_type),
		                   MessageItem::Str(public_key2.to_string()),
		                   MessageItem::Str(app_id2.to_string()),
		                   MessageItem::Str(app_id1.to_string()),
		                   MessageItem::UInt32(2*60)]);

		let reply = conn.send_with_reply_and_block(msg, 2*60*1000).unwrap();
		match reply.get_items().pop().unwrap() {
			MessageItem::UnixFd(fd) => { 
				let fd = fd.into_fd();

				let buf = "foo".as_bytes();

				let len = unsafe {
					send(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, 0)
				};
				assert_eq!(buf.len(), len as usize);
			},
			_ => assert!(false),
		}

	});

	let conn = DbusConnection::get_private(BusType::Session).unwrap();
	let mut msg = Message::new_method_call("org.manuel.TestIntercom2", "/",
	                                       "org.manuel.Intercom", "ConnectToKey").unwrap();
	msg.append_items(&[MessageItem::Int32(sock_type),
	                   MessageItem::Str(public_key1.to_string()),
	                   MessageItem::Str(app_id1.to_string()),
	                   MessageItem::Str(app_id2.to_string()),
	                   MessageItem::UInt32(2*60)]);

	let reply = conn.send_with_reply_and_block(msg, 2*60*1000).unwrap();
	match reply.get_items().pop().unwrap() {
		MessageItem::UnixFd(fd) => {
			let fd = fd.into_fd();

			let mut buf = vec![0; 128];
			let len = unsafe {
				recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t, 0)
			};
			buf.truncate(len as usize);

			debug!("Received {:?}", buf);
			assert_eq!(buf, "foo".as_bytes());
		},
		_ => assert!(false),
	}
}

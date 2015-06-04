extern crate env_logger;

use std::thread::{spawn,sleep_ms};
use libc::consts::os::bsd44::{SOCK_DGRAM, SOCK_STREAM};

use dbus::Connection as DbusConnection;
use dbus::{Message,MessageItem,BusType};

use ::start_intercom;

#[test]
fn test_intercom() {
	env_logger::init().unwrap();

	spawn(|| {
		let args = vec!["intercom", "org.manuel.TestIntercom1", "private-key1"];
		let args:Vec<String> = args.iter()
			.map(|s| s.to_string())
			.collect();
		start_intercom(args.into_iter());
	});
	spawn(|| {
		let args = vec!["intercom", "org.manuel.TestIntercom2", "private-key2"];
		let args:Vec<String> = args.iter()
			.map(|s| s.to_string())
			.collect();
		start_intercom(args.into_iter());
	});
	sleep_ms(1000);

	let public_key1 = "3033303134334138453637313741463145424130444341343538383834323034453232433037414339454645313143324435314333433337303543304146374245393439343244414231303041333843424134383441344531373141373434453431433831363045363841454545373638364342344231304445303844463434313632303939";
	let public_key2 = "3033303036333243363341373741303742434442384241434644423931313646303544334546364233394232393532414230434530393730343631383741424339333837394137423938333539443439394345304441323736454430454638423845303330333034444436433038413531434438384441344345374145433033393241424439";

	let sock_type = SOCK_DGRAM;
	spawn(move || {
		let conn = DbusConnection::get_private(BusType::Session).unwrap();
		let mut msg = Message::new_method_call("org.manuel.TestIntercom1", "/",
		                                       "org.manuel.Intercom", "Connect").unwrap();
		msg.append_items(&[MessageItem::Int32(sock_type), MessageItem::Str(public_key2.to_string()),
		                   MessageItem::UInt32(1), MessageItem::UInt32(2*60)]);
		let _ = conn.send_with_reply_and_block(msg, 2*60*1000).unwrap();
	});

	let conn = DbusConnection::get_private(BusType::Session).unwrap();
	let mut msg = Message::new_method_call("org.manuel.TestIntercom2", "/",
	                                       "org.manuel.Intercom", "Connect").unwrap();
	msg.append_items(&[MessageItem::Int32(sock_type), MessageItem::Str(public_key1.to_string()),
	                   MessageItem::UInt32(1), MessageItem::UInt32(2*60)]);
	let _ = conn.send_with_reply_and_block(msg, 2*60*1000).unwrap();
}

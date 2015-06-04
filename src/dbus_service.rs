use time::Duration;

use dbus::{BusType,NameFlag,Connection,ConnectionItem,OwnedFd,MessageItem,Error,Message};
use dbus::obj::{Method,ObjectPath,Argument,Interface};

use intercom::Intercom;

const INTERFACE:&'static str = "org.manuel.Intercom";

pub struct DBusService;

impl DBusService {
	pub fn serve<'a>(intercom: Intercom, dbus_name: &'a str, bus_type: BusType)
		-> Result<(), Error>
	{
		let conn = try!(Connection::get_private(bus_type));
		try!(conn.register_name(dbus_name, NameFlag::ReplaceExisting as u32));

		let mut o = ObjectPath::new(&conn, "/", true);
		o.insert_interface(INTERFACE, Interface::new(
			vec![Method::new("Connect",
				vec![Argument::new("socket_type", "i"),
				     Argument::new("remote_public_key", "s"),
				     Argument::new("app_id", "s"),
				     Argument::new("timeout_sec", "u"),
				],
				vec![Argument::new("fd", "h")],
				Box::new(move |msg| Self::connect(&intercom, msg))
			)],
			vec![], vec![]));
		try!(o.set_registered(true));

		for n in conn.iter(1000) {
			match n {
				ConnectionItem::MethodCall(mut m) => {
					o.handle_message(&mut m);
				},
				ConnectionItem::Signal(_) | ConnectionItem::Nothing => (),
			}
		}

		Ok(())
	}

	fn connect(intercom: &Intercom, msg: &mut Message)
		-> Result<Vec<MessageItem>, (&'static str, String)>
	{
		let args = msg.get_items();

		let arg0 = args.get(0);
		let arg1 = args.get(1);
		let arg2 = args.get(2);
		let arg3 = args.get(3);

		match (arg0, arg1, arg2, arg3) {
			(Some(&MessageItem::Int32(socket_type)),
			 Some(&MessageItem::Str(ref remote_public_key)),
			 Some(&MessageItem::Str(ref app_id)),
			 Some(&MessageItem::UInt32(timeout_sec))) =>
			{
				let timeout = Duration::seconds(timeout_sec as i64);
				let fd = try!(intercom.connect(socket_type, remote_public_key.clone(),
				                               app_id.clone(), timeout)
				              .map_err(|e| ("org.manuel.Intercom.InternalError", format!("{:?}", e))));

				let result = vec![MessageItem::UnixFd(OwnedFd::new(fd))];
				Ok(result)
			},
	 		_ => {
				let err = format!("{}.InternalError", INTERFACE);
				let err = "org.manuel.Intercom.InternalError";
				Err((&err[..], "Internal error while parsing the arguments.".to_string()))
			},
		}
	}
}

use libc::types::os::arch::c95::{c_int,c_long};
use std::thread::Thread;
use std::mem;

use from_pointer::FromUtf8Pointer;
use dbus_request::DbusRequest;
use nice::glib2::GMainLoop;
use bindings_glib::GBusType::*;
use bindings_glib::GDBusCapabilityFlags::*;
use bindings_glib::GBusNameOwnerFlags::*;
use bindings_glib::{
		TRUE,
		FALSE,
		guint,
		gboolean,
		g_type_init,
		g_bus_own_name,
		g_dbus_connection_get_capabilities,
		g_dbus_interface_skeleton_export,
		g_signal_connect_data};
use bindings_ganymed::{ganymed_skeleton_new};

use glib::dbus_method_invocation::GDBusMethodInvocation;
use glib::g_variant::GVariant;

use std::os::unix::Fd;
use std::time::duration::Duration;
use std::io::timer::sleep;
use std::sync::mpsc::{channel,Sender,Receiver};

pub struct DbusService {
	ptr: *mut i32,
	rx: Receiver<DbusRequest>,
}

struct DbusRespond;

extern fn on_name_acquired(conn: c_long, name: c_long, user_data: Box<Sender<(c_long, c_long)>>)
{
	(*user_data).send((conn, name));
}

extern fn connect_to_node(dbus_obj:  *mut i32,
			invocation_ptr:          *mut i32,
			fd_list:                 *mut i32,
			gvar_remote_public_key:  *mut i32,
			port:                    guint,
			timeout:                 guint,
			channel:                 *mut Sender<DbusRequest>)
	-> gboolean
{
	debug!("connect_to_node() invoked.");

	assert!(!channel.is_null());

	let invocation = GDBusMethodInvocation::new(invocation_ptr);
	invocation.return_dbus_error("org.manuel.Ganymed.not_implemented", "msg");

	let remote_public_key = GVariant::from_ptr(gvar_remote_public_key).to_vec();

	let req = DbusRequest::new(remote_public_key, port, timeout);

	unsafe {
		(*channel).send(req)
	};

	TRUE
}

impl DbusService {
	pub fn new(service_name: &str) -> DbusService
	{
		unsafe { g_type_init() };

		Thread::spawn(|| {
			GMainLoop::new().run();
		});

		let bus_type = G_BUS_TYPE_SESSION;
		let flags = G_BUS_NAME_OWNER_FLAGS_ALLOW_REPLACEMENT | 
			G_BUS_NAME_OWNER_FLAGS_REPLACE;

		let conn = DbusService::acquire_name(service_name, bus_type, flags);

		let (tx, rx) = channel();
		let myself = DbusService {
			ptr: conn as *mut i32,
			rx:  rx
		};
		assert!(myself.supports_unix_fd_passing());

		myself.export_object_path("/org/manuel/Ganymed", tx);

	    myself
	}

	fn acquire_name(service_name: &str,
				bus_type: GBusType,
				flags: GBusNameOwnerFlags)
		-> *mut i32
	{
		let (tx, rx): (Sender<(c_long, c_long)>,_) = channel();

		let ptr = Box::new(tx);
		let res = unsafe {
			g_bus_own_name(bus_type.bits(),
				service_name.as_ptr(),
				flags.bits(),
				None,
				mem::transmute(Some(on_name_acquired)),
				None,
				mem::transmute(ptr),
				None)
		};

		let (conn, name) = rx.recv().unwrap();
		debug!("DBus name {} acquired", service_name);

		conn as *mut i32
	}

	fn export_object_path(&self, obj_path: &str, tx: Sender<DbusRequest>)
	{
		let skeleton = unsafe { ganymed_skeleton_new() as *mut i32 };

		unsafe {
			let mut error = 0 as *mut i32;
			let res = g_dbus_interface_skeleton_export(skeleton,
				self.ptr,
				obj_path.as_ptr(),
				error);
			assert!(error.is_null());

			let res = g_signal_connect_data(skeleton,
				"handle_connect".as_ptr(),
				mem::transmute(connect_to_node),
				mem::transmute(Box::new(tx)),
				None,
				1);
			assert!(res > 0);
		}
	}

	fn supports_unix_fd_passing(&self) -> bool {
		let bits = unsafe { g_dbus_connection_get_capabilities(self.ptr) };
		let flags = GDBusCapabilityFlags::from_bits(bits).unwrap();
		
		flags.contains(G_DBUS_CAPABILITY_FLAGS_UNIX_FD_PASSING)
	}
}


impl<'a> Iterator for DbusService {
	type Item = (DbusRequest, DbusRespond);

	fn next(&mut self) -> Option<(DbusRequest, DbusRespond)> {
		self.rx.recv().ok().map(|req| (req, DbusRespond))
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}
}

impl ::DbusResponder for DbusRespond {
	fn send(&self, fd: Fd) -> Result<(),()> { unimplemented!() }
	fn send_error<T>(&self, err: T) -> Result<(),()> { unimplemented!() }
}

use libc::types::os::arch::c95::c_long;
use std::thread::Thread;
use std::mem;

use dbus_request::DbusRequest;
use nice::glib2::GMainLoop;
use bindings_glib::GBusType::*;
use bindings_glib::GDBusCapabilityFlags::*;
use bindings_glib::GBusNameOwnerFlags::*;
use bindings_glib::{
		TRUE,
		guint,
		gboolean,
		g_type_init,
		g_bus_own_name,
		g_dbus_connection_get_capabilities,
		g_dbus_interface_skeleton_export};
use bindings_ganymed::ganymed_skeleton_new;
use ::ConnectError;
use ::DbusResponder;
use utils::spawn_thread;

use glib::dbus_method_invocation::GDBusMethodInvocation as GInvocation;
use glib::g_variant::GVariant;
use glib::g_object::GObject;
use from_pointer::cstr;

use std::os::unix::Fd;
use std::sync::mpsc::{channel,Sender,Receiver};

pub struct DbusService<R:DbusResponder> {
	ptr: *mut i32,
	rx: Receiver<DbusRequest<R>>,
}

struct DbusRespond;

extern fn on_name_acquired(conn: c_long, name: c_long, user_data: Box<Sender<(c_long, c_long)>>)
{
	debug!("on_name_acquired");
	if (*user_data).send((conn, name)).is_err() {
		warn!("on_name_acquired(): send() failed!");
	}
}

extern fn connect_to_node(dbus_obj:  *mut i32,
			invocation_ptr:          *mut i32,
			fd_list:                 *mut i32,
			gvar_remote_public_key:  *mut i32,
			port:                    guint,
			timeout:                 guint,
			channel:                 *mut Sender<DbusRequest<GInvocation>>)
	-> gboolean
{
	debug!("connect_to_node() invoked.");

	assert!(!channel.is_null());

	let invocation = GInvocation::new(invocation_ptr);

	let remote_public_key = GVariant::from_ptr(gvar_remote_public_key).to_vec();

	let req = DbusRequest::new(invocation, remote_public_key, port, timeout);

	unsafe {
		if (*channel).send(req).is_err() {
			warn!("on_name_acquired(): send() failed!");
		}
	};

	TRUE
}

impl DbusService<GInvocation> {
	pub fn new(service_name: &str) -> DbusService<GInvocation>
	{
		unsafe { g_type_init() };

		spawn_thread("DbusService::GMainLoop", || {
			// a bug in this thread is probably in one of the callback funcs
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
				cstr(service_name).as_ptr(),
				flags.bits(),
				None,
				mem::transmute(Some(on_name_acquired)),
				None,
				mem::transmute(ptr),
				None)
		};
		assert!(res > 0);

		let (conn, name) = rx.recv().unwrap();
		debug!("DBus name {} acquired", service_name);

		conn as *mut i32
	}

	fn export_object_path(&self, obj_path: &str, tx: Sender<DbusRequest<GInvocation>>)
	{
		let ptr = unsafe { ganymed_skeleton_new() as *mut i32 };
		let obj = GObject::from_ptr(ptr);

		unsafe {
			let mut error = 0 as *mut i32;
			g_dbus_interface_skeleton_export(ptr,
				self.ptr,
				obj_path.as_ptr(),
				&mut error);
			assert!(error.is_null());

			obj.connect_signal("handle_connect",
				mem::transmute(connect_to_node),
				Box::new(tx));
		}
	}

	fn supports_unix_fd_passing(&self) -> bool {
		let bits = unsafe { g_dbus_connection_get_capabilities(self.ptr) };
		let flags = GDBusCapabilityFlags::from_bits(bits).unwrap();
		
		flags.contains(G_DBUS_CAPABILITY_FLAGS_UNIX_FD_PASSING)
	}
}


impl<'a,R:DbusResponder+Send> Iterator for DbusService<R> {
	type Item = DbusRequest<R>;

	fn next(&mut self) -> Option<DbusRequest<R>> {
		self.rx.recv().ok()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}
}

impl ::DbusResponder for GInvocation {
	fn respond_ok(&self, fd: Fd) -> Result<(),()> {
		let result = GVariant::new_tuple(vec![GVariant::from_fd(fd)]);
		self.return_result(&result, vec![fd]);

		Ok(())
	}

	fn respond_error(&self, err: ::ConnectError) -> Result<(),()> {
		let (name, msg) = match err {
			ConnectError::REMOTE_CREDENTIALS_NOT_FOUND => 
				("org.manuel.Ganymed.credentials_not_found", ""),
			ConnectError::FOO =>
				("org.manuel.Ganymed.not_implemented", ""),
		};
		self.return_dbus_error(name, msg);
		Ok(())
	}
}

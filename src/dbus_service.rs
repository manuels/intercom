use libc::types::os::arch::c95::c_long;
use std::thread;
use std::mem;

use dbus_request::DBusRequest;
use nice::glib2::GMainLoop;
use bindings_glib::GBusType::*;
use bindings_glib::GDBusCapabilityFlags::*;
use bindings_glib::GBusNameOwnerFlags::*;
use bindings_glib::TRUE;
use bindings_glib::FALSE;
use bindings_glib::guint;
use bindings_glib::gboolean;
use bindings_glib::g_type_init;
use bindings_glib::g_bus_own_name;
use bindings_glib::g_dbus_connection_get_capabilities;
use bindings_glib::g_dbus_interface_skeleton_export;
use bindings_ganymed::ganymed_skeleton_new;

use ::ConnectError;
use ::DBusResponder;

use glib::dbus_method_invocation::GDBusMethodInvocation as GInvocation;
use glib::g_variant::GVariant;
use glib::g_object::GObject;
use std::ffi::CString;

use std::os::unix::io::RawFd;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

pub struct DBusService<R:DBusResponder>
{
	ptr: *mut i32,
	rx: Receiver<DBusRequest<R>>,
}

extern fn on_name_acquired(conn: c_long,
                           name: c_long,
                           user_data: Box<Sender<(c_long, c_long)>>)
	-> gboolean
{
	debug!("on_name_acquired");

	let pair = (conn, name);
	let res = (*user_data).send(pair);

	if res.is_err() {
		panic!("on_name_acquired(): send() failed!");
	}

	FALSE
}

extern fn connect_to_node(_:         *mut i32,
			invocation_ptr:          *mut i32,
			fd_list:                 *mut i32,
			gvar_remote_public_key:  *mut i32,
			port:                    guint,
			timeout:                 guint,
			channel:                 *mut Sender<DBusRequest<GInvocation>>)
	-> gboolean
{
	debug!("connect_to_node() invoked.");

	assert!(!channel.is_null());
	assert!(!invocation_ptr.is_null());
	assert!(!gvar_remote_public_key.is_null());

	let invoc = GInvocation::new(invocation_ptr);
	let remote_public_key = GVariant::from_ptr(gvar_remote_public_key).to_vec();
	let req = DBusRequest::new(invoc, remote_public_key, port, timeout);

	let res = unsafe { (*channel).send(req) };
	if res.is_err() {
		panic!("connect_to_node(): send() failed!");
	}

	TRUE
}

impl DBusService<GInvocation> {
	pub fn new(service_name: &str) -> DBusService<GInvocation>
	{
		unsafe { g_type_init() };

		thread::Builder::new().name("DBusService::GMainLoop".to_string()).spawn(move || {
			// a bug in this thread is probably in one of the callback funcs
			GMainLoop::new().run();
		});

		let bus_type = G_BUS_TYPE_SESSION;
		let flags = G_BUS_NAME_OWNER_FLAGS_REPLACE | 
			G_BUS_NAME_OWNER_FLAGS_ALLOW_REPLACEMENT;

		let conn = DBusService::acquire_name(service_name, bus_type, flags);

		let (tx, rx) = channel();
		let myself = DBusService {
			ptr: conn as *mut i32,
			rx:  rx
		};
		assert!(myself.supports_unix_fd_passing());

		myself.export_object_path("/org/manuel/Ganymed", tx);

	    myself
	}

	fn acquire_name(service_name: &str,
	                bus_type:     GBusType,
	                flags:        GBusNameOwnerFlags)
		-> *mut i32
	{
		let (tx, rx): (Sender<(c_long, c_long)>,_) = channel();

		let ptr = Box::new(tx);
		let res = unsafe {
			g_bus_own_name(bus_type.bits(),
				CString::new(service_name).unwrap().as_ptr(),
				flags.bits(),
				None,
				mem::transmute(Some(on_name_acquired)),
				None,
				mem::transmute(ptr),
				None)
		};
		assert!(res > 0);

		let (conn, _) = rx.recv().unwrap();
		debug!("DBus name {} acquired", service_name);

		conn as *mut i32
	}

	fn export_object_path(&self,
	                      obj_path: &str,
	                      tx:       Sender<DBusRequest<GInvocation>>)
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


impl<R:'static+DBusResponder+Send> Iterator for DBusService<R> {
	type Item = DBusRequest<R>;

	fn next(&mut self) -> Option<DBusRequest<R>> {
		self.rx.recv().ok()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}
}

impl DBusResponder for GInvocation {
	fn respond_ok(&self, fd: RawFd) -> Result<(),()> {
		let result = GVariant::new_tuple(vec![GVariant::from_fd(fd)]);

		self.return_result(&result, vec![fd]);
		Ok(())
	}

	fn respond_error(&self, err: ::ConnectError) -> Result<(),()> {
		let (name, msg) = match err {
			ConnectError::RemoteCredentialsNotFound => 
				("org.manuel.Ganymed.credentials_not_found", ""),
			ConnectError::IceConnectFailed => 
				("org.manuel.Ganymed.nice_connect_failed", ""),
			ConnectError::FOO =>
				("org.manuel.Ganymed.not_implemented", ""),
		};

		self.return_dbus_error(name, msg);
		Ok(())
	}
}

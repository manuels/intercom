use libc::types::os::arch::c95::{c_int,c_long};
use std::thread::Thread;
use std::mem;

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

use std::os::unix::Fd;
use std::time::duration::Duration;
use std::io::timer::sleep;
use std::sync::mpsc::{channel,Sender,Receiver};

pub struct DbusService;

struct DbusRespond;

extern fn on_name_acquired(conn: c_long, name: c_long, user_data: Box<Sender<(c_long, c_long)>>) {
	(*user_data).send((conn, name));
}

extern fn connect_to_node(dbus_obj: *mut i32,
		invocation:                 *mut i32,
		fd_list:                    *mut i32,
		arg_remote_public_key:      *mut i32,
		port:                       guint,
		timeout:                    guint)
	-> gboolean
{
	//(*user_data).send((conn, name));
	warn!("jol");

	TRUE
}

impl DbusService {
	pub fn new(service_name: &str) -> DbusService
	{
		unsafe { g_type_init() };

		let bus_type = G_BUS_TYPE_SESSION;
		let flags = G_BUS_NAME_OWNER_FLAGS_ALLOW_REPLACEMENT | 
			G_BUS_NAME_OWNER_FLAGS_REPLACE;

		let (cb_tx, rx): (Sender<(c_long, c_long)>,_) = channel();
		
		let ptr = Box::new(cb_tx);
		let res = unsafe {
	        g_bus_own_name(bus_type.bits(), service_name.as_ptr(), flags.bits(),
				None, mem::transmute(Some(on_name_acquired)), None, mem::transmute(ptr), None)
	    };

	    Thread::spawn(|| {
	    	GMainLoop::new().run();
	    });

	    let (conn, name) = rx.recv().unwrap();

	    let myself = DbusService;
	    myself.on_name_acquired(conn as *mut i32, name as *mut u8, 0 as *mut i32);

	    myself
	}

	fn on_name_acquired(&self, conn: *mut i32, name: *mut u8,
			user_data: *mut i32)
	{
		debug!("on_name_acquired");

		let bits = unsafe { g_dbus_connection_get_capabilities(conn) };
		let flags = GDBusCapabilityFlags::from_bits(bits).unwrap();
		assert!(flags.contains(G_DBUS_CAPABILITY_FLAGS_UNIX_FD_PASSING));

		let path = "/org/manuel/Ganymed";
		let skeleton = unsafe { ganymed_skeleton_new() };

		unsafe {
			let mut err = 0 as *mut i32;
			let res = g_dbus_interface_skeleton_export(skeleton as *mut i32, conn,
				path.as_ptr(), err);
			assert!(res != 0);
			assert!(err == 0 as *mut i32);

			let res = g_signal_connect_data(skeleton as *mut i32, "handle_connect".as_ptr(),
				mem::transmute(connect_to_node), 0 as *mut i32, None, 1);
			assert!(res > 0);
			//g_signal_connect(skeleton, "handle_find_node", mem::transmute(find_node), NULL);
		}
	}
}


impl<'a> Iterator for DbusService {
	type Item = (DbusRequest<'a>, DbusRespond);

	fn next<'b>(&mut self) -> Option<(DbusRequest<'b>, DbusRespond)> {
		let time = Duration::minutes(1);
		sleep(time);
		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}
}

impl ::DbusResponder for DbusRespond {
	fn send(&self, fd: Fd) -> Result<(),()> { unimplemented!() }
	fn send_error<T>(&self, err: T) -> Result<(),()> { unimplemented!() }
}

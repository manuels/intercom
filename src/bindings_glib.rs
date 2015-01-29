use libc::types::os::arch::c95::{c_int, c_uint, c_ulong, c_long};

pub type gint = c_int;
pub type guint = c_uint;
pub type gboolean = gint;
pub type guchar = u8;

pub const FALSE: gboolean = 0;
pub const TRUE:  gboolean = !FALSE;

pub mod GBusType {
	use libc::types::os::arch::c95::{c_int, c_uint};

	bitflags! {
		flags GBusType: c_int {
			const G_BUS_TYPE_STARTER = -1,
			const G_BUS_TYPE_NONE    =  0,
			const G_BUS_TYPE_SYSTEM  =  1,
			const G_BUS_TYPE_SESSION =  2,
		}
	}
}

pub mod GBusNameOwnerFlags {
	use libc::types::os::arch::c95::{c_int, c_uint};

	bitflags! {
		flags GBusNameOwnerFlags: c_int {
			const G_BUS_NAME_OWNER_FLAGS_NONE              = 0,
			const G_BUS_NAME_OWNER_FLAGS_ALLOW_REPLACEMENT = 1,
			const G_BUS_NAME_OWNER_FLAGS_REPLACE           = 2,
		}
	}
}

pub mod GConnectFlags {
	use libc::types::os::arch::c95::c_int;

	bitflags! {
		flags GConnectFlags: c_int {
			const G_CONNECT_AFTER   = 1<<0,
			const G_CONNECT_SWAPPED = 1<<1,
		}
	}
}

type GBusNameAcquiredCallback = extern "C" fn(conn: *mut i32, name: *mut u8,
	user_data: *mut i32);

extern "C" {
	pub fn g_type_init();

	pub fn g_bus_own_name(bus_type: c_int,
				name: *const u8 ,
				flags: c_int,
				bus_acquired_handler: Option<fn()>,
				name_acquired_handler: Option<GBusNameAcquiredCallback>,
				name_lost_handler: Option<fn()>,
				user_data: *mut c_int,
				user_data_free_func: Option<fn()>) -> c_uint;

	pub fn g_dbus_connection_get_capabilities(conn: *mut i32) -> c_int;

	pub fn g_dbus_interface_skeleton_export(skeleton: *mut i32,
				conn: *mut i32,
				object_path: *const u8,
				error: *mut i32) -> c_int;

	pub fn g_signal_connect_data(instance: *mut i32,
				detailed_signal: *const u8,
				c_handler: Option<extern fn()>,
				data: *mut i32,
				destroy_data: Option<extern fn()>,
				connect_flags: i32) -> c_ulong;
}

pub mod GDBusCapabilityFlags {
	use libc::types::os::arch::c95::{c_int, c_uint};

	bitflags! {
		flags GDBusCapabilityFlags: c_int {
			const G_DBUS_CAPABILITY_FLAGS_NONE            = 0,
			const G_DBUS_CAPABILITY_FLAGS_UNIX_FD_PASSING = (1<<0),
		}
	}
}

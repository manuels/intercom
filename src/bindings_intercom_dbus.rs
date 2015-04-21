#![allow(dead_code)]

extern crate libc;

use std::mem;


/*
struct _Intercom
*/
#[repr(C)]
pub struct _Intercom;

/*
struct _IntercomIface
		(int) parent_iface
		(int (int *)) gboolean
*/
#[repr(C)]
pub struct _IntercomIface {
	pub parent_iface: libc::c_int,
	pub gboolean: extern fn (*mut libc::c_int) -> libc::c_int,
}

/*
struct _IntercomProxy
		(int) parent_instance
		(IntercomProxyPrivate *) priv [struct _IntercomProxyPrivate *]
*/
#[repr(C)]
pub struct _IntercomProxy {
	pub parent_instance: libc::c_int,
	pub priv_: *mut _IntercomProxyPrivate,
}

/*
struct _IntercomProxyClass
		(int) parent_class
*/
#[repr(C)]
pub struct _IntercomProxyClass {
	pub parent_class: libc::c_int,
}

/*
struct _IntercomProxyPrivate
*/
#[repr(C)]
pub struct _IntercomProxyPrivate;

/*
struct _IntercomSkeleton
		(int) parent_instance
		(IntercomSkeletonPrivate *) priv [struct _IntercomSkeletonPrivate *]
*/
#[repr(C)]
pub struct _IntercomSkeleton {
	pub parent_instance: libc::c_int,
	pub priv_: *mut _IntercomSkeletonPrivate,
}

/*
struct _IntercomSkeletonClass
		(int) parent_class
*/
#[repr(C)]
pub struct _IntercomSkeletonClass {
	pub parent_class: libc::c_int,
}

/*
struct _IntercomSkeletonPrivate
*/
#[repr(C)]
pub struct _IntercomSkeletonPrivate;

/*
struct Intercom
*/
#[repr(C)]
pub struct Intercom;

/*
int * intercom_interface_info()
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_interface_info() -> *mut libc::c_int;
}


/*
int intercom_override_properties()
	(int *) klass
	(int) property_id_begin
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_override_properties(klass: *mut libc::c_int, property_id_begin: libc::c_int) -> libc::c_int;
}


/*
void intercom_complete_connect()
	(Intercom *) object [struct _Intercom *]
	(int *) invocation
	(int *) fd_list
	(int *) fd
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_complete_connect(object: *mut _Intercom, invocation: *mut libc::c_int, fd_list: *mut libc::c_int, fd: *mut libc::c_int);
}


/*
void intercom_complete_add_node()
	(Intercom *) object [struct _Intercom *]
	(int *) invocation
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_complete_add_node(object: *mut _Intercom, invocation: *mut libc::c_int);
}


/*
void intercom_complete_find_node()
	(Intercom *) object [struct _Intercom *]
	(int *) invocation
	(int *) nodes
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_complete_find_node(object: *mut _Intercom, invocation: *mut libc::c_int, nodes: *mut libc::c_int);
}


/*
void intercom_call_connect()
	(Intercom *) proxy [struct _Intercom *]
	(int) arg_socket_type
	(int *) arg_remote_public_key
	(int) arg_port
	(int) arg_timeout
	(int *) fd_list
	(int *) cancellable
	(int) callback
	(int) user_data
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_connect(proxy: *mut _Intercom, arg_socket_type: libc::c_int, arg_remote_public_key: *mut libc::c_int, arg_port: libc::c_int, arg_timeout: libc::c_int, fd_list: *mut libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int intercom_call_connect_finish()
	(Intercom *) proxy [struct _Intercom *]
	(int **) out_fd
	(int **) out_fd_list
	(int *) res
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_connect_finish(proxy: *mut _Intercom, out_fd: *mut *mut libc::c_int, out_fd_list: *mut *mut libc::c_int, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int intercom_call_connect_sync()
	(Intercom *) proxy [struct _Intercom *]
	(int) arg_socket_type
	(int *) arg_remote_public_key
	(int) arg_port
	(int) arg_timeout
	(int *) fd_list
	(int **) out_fd
	(int **) out_fd_list
	(int *) cancellable
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_connect_sync(proxy: *mut _Intercom, arg_socket_type: libc::c_int, arg_remote_public_key: *mut libc::c_int, arg_port: libc::c_int, arg_timeout: libc::c_int, fd_list: *mut libc::c_int, out_fd: *mut *mut libc::c_int, out_fd_list: *mut *mut libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void intercom_call_add_node()
	(Intercom *) proxy [struct _Intercom *]
	(const int *const *) arg_node_name
	(const int *) arg_remote_public_key
	(int *) cancellable
	(int) callback
	(int) user_data
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_add_node(proxy: *mut _Intercom, arg_node_name: *const *const libc::c_int, arg_remote_public_key: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int intercom_call_add_node_finish()
	(Intercom *) proxy [struct _Intercom *]
	(int *) res
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_add_node_finish(proxy: *mut _Intercom, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int intercom_call_add_node_sync()
	(Intercom *) proxy [struct _Intercom *]
	(const int *const *) arg_node_name
	(const int *) arg_remote_public_key
	(int *) cancellable
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_add_node_sync(proxy: *mut _Intercom, arg_node_name: *const *const libc::c_int, arg_remote_public_key: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void intercom_call_find_node()
	(Intercom *) proxy [struct _Intercom *]
	(const int *const *) arg_keywords
	(int *) cancellable
	(int) callback
	(int) user_data
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_find_node(proxy: *mut _Intercom, arg_keywords: *const *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int intercom_call_find_node_finish()
	(Intercom *) proxy [struct _Intercom *]
	(int **) out_nodes
	(int *) res
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_find_node_finish(proxy: *mut _Intercom, out_nodes: *mut *mut libc::c_int, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int intercom_call_find_node_sync()
	(Intercom *) proxy [struct _Intercom *]
	(const int *const *) arg_keywords
	(int **) out_nodes
	(int *) cancellable
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_call_find_node_sync(proxy: *mut _Intercom, arg_keywords: *const *const libc::c_int, out_nodes: *mut *mut libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void intercom_proxy_new()
	(int *) connection
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int) callback
	(int) user_data
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_proxy_new(connection: *mut libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
Intercom * intercom_proxy_new_finish() [struct _Intercom *]
	(int *) res
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_proxy_new_finish(res: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Intercom;
}


/*
Intercom * intercom_proxy_new_sync() [struct _Intercom *]
	(int *) connection
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_proxy_new_sync(connection: *mut libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Intercom;
}


/*
void intercom_proxy_new_for_bus()
	(int) bus_type
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int) callback
	(int) user_data
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_proxy_new_for_bus(bus_type: libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
Intercom * intercom_proxy_new_for_bus_finish() [struct _Intercom *]
	(int *) res
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_proxy_new_for_bus_finish(res: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Intercom;
}


/*
Intercom * intercom_proxy_new_for_bus_sync() [struct _Intercom *]
	(int) bus_type
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int **) error
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_proxy_new_for_bus_sync(bus_type: libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Intercom;
}


/*
Intercom * intercom_skeleton_new() [struct _Intercom *]
*/
#[link(name="intercom-dbus")]
extern "C" {
	pub fn intercom_skeleton_new() -> *mut _Intercom;
}


/* __INTERCOM_DBUS_BINDINGS_H__ # */

/* TYPE_INTERCOM ( intercom_get_type ( ) ) # */

/* INTERCOM ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_INTERCOM , Intercom ) ) # */

/* IS_INTERCOM ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_INTERCOM ) ) # */

/* INTERCOM_GET_IFACE ( o ) ( G_TYPE_INSTANCE_GET_INTERFACE ( ( o ) , TYPE_INTERCOM , IntercomIface ) ) struct */

/* TYPE_INTERCOM_PROXY ( intercom_proxy_get_type ( ) ) # */

/* INTERCOM_PROXY ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_INTERCOM_PROXY , IntercomProxy ) ) # */

/* INTERCOM_PROXY_CLASS ( k ) ( G_TYPE_CHECK_CLASS_CAST ( ( k ) , TYPE_INTERCOM_PROXY , IntercomProxyClass ) ) # */

/* INTERCOM_PROXY_GET_CLASS ( o ) ( G_TYPE_INSTANCE_GET_CLASS ( ( o ) , TYPE_INTERCOM_PROXY , IntercomProxyClass ) ) # */

/* IS_INTERCOM_PROXY ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_INTERCOM_PROXY ) ) # */

/* IS_INTERCOM_PROXY_CLASS ( k ) ( G_TYPE_CHECK_CLASS_TYPE ( ( k ) , TYPE_INTERCOM_PROXY ) ) typedef */

/* TYPE_INTERCOM_SKELETON ( intercom_skeleton_get_type ( ) ) # */

/* INTERCOM_SKELETON ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_INTERCOM_SKELETON , IntercomSkeleton ) ) # */

/* INTERCOM_SKELETON_CLASS ( k ) ( G_TYPE_CHECK_CLASS_CAST ( ( k ) , TYPE_INTERCOM_SKELETON , IntercomSkeletonClass ) ) # */

/* INTERCOM_SKELETON_GET_CLASS ( o ) ( G_TYPE_INSTANCE_GET_CLASS ( ( o ) , TYPE_INTERCOM_SKELETON , IntercomSkeletonClass ) ) # */

/* IS_INTERCOM_SKELETON ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_INTERCOM_SKELETON ) ) # */

/* IS_INTERCOM_SKELETON_CLASS ( k ) ( G_TYPE_CHECK_CLASS_TYPE ( ( k ) , TYPE_INTERCOM_SKELETON ) ) typedef */


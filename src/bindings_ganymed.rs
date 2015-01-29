#![allow(dead_code)]
#![allow(unstable)]

extern crate libc;

/*
struct _Ganymed
*/
#[repr(C)]
pub struct _Ganymed;

/*
struct _GanymedIface
		(int) parent_iface
		(int (int *)) gboolean
*/
#[repr(C)]
pub struct _GanymedIface {
	parent_iface: libc::c_int,
	gboolean: extern fn (*mut libc::c_int) -> libc::c_int,
}

/*
struct _GanymedProxy
		(int) parent_instance
		(GanymedProxyPrivate *) priv [struct _GanymedProxyPrivate *]
*/
#[repr(C)]
pub struct _GanymedProxy {
	parent_instance: libc::c_int,
	priv_: *mut _GanymedProxyPrivate,
}

/*
struct _GanymedProxyClass
		(int) parent_class
*/
#[repr(C)]
pub struct _GanymedProxyClass {
	parent_class: libc::c_int,
}

/*
struct _GanymedProxyPrivate
*/
#[repr(C)]
pub struct _GanymedProxyPrivate;

/*
struct _GanymedSkeleton
		(int) parent_instance
		(GanymedSkeletonPrivate *) priv [struct _GanymedSkeletonPrivate *]
*/
#[repr(C)]
pub struct _GanymedSkeleton {
	parent_instance: libc::c_int,
	priv_: *mut _GanymedSkeletonPrivate,
}

/*
struct _GanymedSkeletonClass
		(int) parent_class
*/
#[repr(C)]
pub struct _GanymedSkeletonClass {
	parent_class: libc::c_int,
}

/*
struct _GanymedSkeletonPrivate
*/
#[repr(C)]
pub struct _GanymedSkeletonPrivate;

/*
struct Ganymed
*/
#[repr(C)]
pub struct Ganymed;

/*
int * ganymed_interface_info()
*/
extern "C" {
	pub fn ganymed_interface_info() -> *mut libc::c_int;
}


/*
int ganymed_override_properties()
	(int *) klass
	(int) property_id_begin
*/
extern "C" {
	pub fn ganymed_override_properties(klass: *mut libc::c_int, property_id_begin: libc::c_int) -> libc::c_int;
}


/*
void ganymed_complete_connect()
	(Ganymed *) object [struct _Ganymed *]
	(int *) invocation
	(int *) fd_list
	(int *) fd
*/
extern "C" {
	pub fn ganymed_complete_connect(object: *mut _Ganymed, invocation: *mut libc::c_int, fd_list: *mut libc::c_int, fd: *mut libc::c_int);
}


/*
void ganymed_complete_add_node()
	(Ganymed *) object [struct _Ganymed *]
	(int *) invocation
*/
extern "C" {
	pub fn ganymed_complete_add_node(object: *mut _Ganymed, invocation: *mut libc::c_int);
}


/*
void ganymed_complete_find_node()
	(Ganymed *) object [struct _Ganymed *]
	(int *) invocation
	(int *) nodes
*/
extern "C" {
	pub fn ganymed_complete_find_node(object: *mut _Ganymed, invocation: *mut libc::c_int, nodes: *mut libc::c_int);
}


/*
void ganymed_call_connect()
	(Ganymed *) proxy [struct _Ganymed *]
	(int *) arg_remote_public_key
	(int) arg_port
	(int) arg_timeout
	(int *) fd_list
	(int *) cancellable
	(int) callback
	(int) user_data
*/
extern "C" {
	pub fn ganymed_call_connect(proxy: *mut _Ganymed, arg_remote_public_key: *mut libc::c_int, arg_port: libc::c_int, arg_timeout: libc::c_int, fd_list: *mut libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int ganymed_call_connect_finish()
	(Ganymed *) proxy [struct _Ganymed *]
	(int **) out_fd
	(int **) out_fd_list
	(int *) res
	(int **) error
*/
extern "C" {
	pub fn ganymed_call_connect_finish(proxy: *mut _Ganymed, out_fd: *mut *mut libc::c_int, out_fd_list: *mut *mut libc::c_int, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int ganymed_call_connect_sync()
	(Ganymed *) proxy [struct _Ganymed *]
	(int *) arg_remote_public_key
	(int) arg_port
	(int) arg_timeout
	(int *) fd_list
	(int **) out_fd
	(int **) out_fd_list
	(int *) cancellable
	(int **) error
*/
extern "C" {
	pub fn ganymed_call_connect_sync(proxy: *mut _Ganymed, arg_remote_public_key: *mut libc::c_int, arg_port: libc::c_int, arg_timeout: libc::c_int, fd_list: *mut libc::c_int, out_fd: *mut *mut libc::c_int, out_fd_list: *mut *mut libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void ganymed_call_add_node()
	(Ganymed *) proxy [struct _Ganymed *]
	(const int *const *) arg_node_name
	(const int *) arg_remote_public_key
	(int *) cancellable
	(int) callback
	(int) user_data
*/
extern "C" {
	pub fn ganymed_call_add_node(proxy: *mut _Ganymed, arg_node_name: *const *const libc::c_int, arg_remote_public_key: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int ganymed_call_add_node_finish()
	(Ganymed *) proxy [struct _Ganymed *]
	(int *) res
	(int **) error
*/
extern "C" {
	pub fn ganymed_call_add_node_finish(proxy: *mut _Ganymed, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int ganymed_call_add_node_sync()
	(Ganymed *) proxy [struct _Ganymed *]
	(const int *const *) arg_node_name
	(const int *) arg_remote_public_key
	(int *) cancellable
	(int **) error
*/
extern "C" {
	pub fn ganymed_call_add_node_sync(proxy: *mut _Ganymed, arg_node_name: *const *const libc::c_int, arg_remote_public_key: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void ganymed_call_find_node()
	(Ganymed *) proxy [struct _Ganymed *]
	(const int *const *) arg_keywords
	(int *) cancellable
	(int) callback
	(int) user_data
*/
extern "C" {
	pub fn ganymed_call_find_node(proxy: *mut _Ganymed, arg_keywords: *const *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int ganymed_call_find_node_finish()
	(Ganymed *) proxy [struct _Ganymed *]
	(int **) out_nodes
	(int *) res
	(int **) error
*/
extern "C" {
	pub fn ganymed_call_find_node_finish(proxy: *mut _Ganymed, out_nodes: *mut *mut libc::c_int, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int ganymed_call_find_node_sync()
	(Ganymed *) proxy [struct _Ganymed *]
	(const int *const *) arg_keywords
	(int **) out_nodes
	(int *) cancellable
	(int **) error
*/
extern "C" {
	pub fn ganymed_call_find_node_sync(proxy: *mut _Ganymed, arg_keywords: *const *const libc::c_int, out_nodes: *mut *mut libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void ganymed_proxy_new()
	(int *) connection
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int) callback
	(int) user_data
*/
extern "C" {
	pub fn ganymed_proxy_new(connection: *mut libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
Ganymed * ganymed_proxy_new_finish() [struct _Ganymed *]
	(int *) res
	(int **) error
*/
extern "C" {
	pub fn ganymed_proxy_new_finish(res: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Ganymed;
}


/*
Ganymed * ganymed_proxy_new_sync() [struct _Ganymed *]
	(int *) connection
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int **) error
*/
extern "C" {
	pub fn ganymed_proxy_new_sync(connection: *mut libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Ganymed;
}


/*
void ganymed_proxy_new_for_bus()
	(int) bus_type
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int) callback
	(int) user_data
*/
extern "C" {
	pub fn ganymed_proxy_new_for_bus(bus_type: libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
Ganymed * ganymed_proxy_new_for_bus_finish() [struct _Ganymed *]
	(int *) res
	(int **) error
*/
extern "C" {
	pub fn ganymed_proxy_new_for_bus_finish(res: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Ganymed;
}


/*
Ganymed * ganymed_proxy_new_for_bus_sync() [struct _Ganymed *]
	(int) bus_type
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int **) error
*/
extern "C" {
	pub fn ganymed_proxy_new_for_bus_sync(bus_type: libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _Ganymed;
}


/*
Ganymed * ganymed_skeleton_new() [struct _Ganymed *]
*/
#[link_args="src/ganymed-bindings.o"]
extern "C" {
	pub fn ganymed_skeleton_new() -> *mut _Ganymed;
}


/* __GANYMED_BINDINGS_H__ # */

/* TYPE_GANYMED ( ganymed_get_type ( ) ) # */

/* GANYMED ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_GANYMED , Ganymed ) ) # */

/* IS_GANYMED ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_GANYMED ) ) # */

/* GANYMED_GET_IFACE ( o ) ( G_TYPE_INSTANCE_GET_INTERFACE ( ( o ) , TYPE_GANYMED , GanymedIface ) ) struct */

/* TYPE_GANYMED_PROXY ( ganymed_proxy_get_type ( ) ) # */

/* GANYMED_PROXY ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_GANYMED_PROXY , GanymedProxy ) ) # */

/* GANYMED_PROXY_CLASS ( k ) ( G_TYPE_CHECK_CLASS_CAST ( ( k ) , TYPE_GANYMED_PROXY , GanymedProxyClass ) ) # */

/* GANYMED_PROXY_GET_CLASS ( o ) ( G_TYPE_INSTANCE_GET_CLASS ( ( o ) , TYPE_GANYMED_PROXY , GanymedProxyClass ) ) # */

/* IS_GANYMED_PROXY ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_GANYMED_PROXY ) ) # */

/* IS_GANYMED_PROXY_CLASS ( k ) ( G_TYPE_CHECK_CLASS_TYPE ( ( k ) , TYPE_GANYMED_PROXY ) ) typedef */

/* TYPE_GANYMED_SKELETON ( ganymed_skeleton_get_type ( ) ) # */

/* GANYMED_SKELETON ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_GANYMED_SKELETON , GanymedSkeleton ) ) # */

/* GANYMED_SKELETON_CLASS ( k ) ( G_TYPE_CHECK_CLASS_CAST ( ( k ) , TYPE_GANYMED_SKELETON , GanymedSkeletonClass ) ) # */

/* GANYMED_SKELETON_GET_CLASS ( o ) ( G_TYPE_INSTANCE_GET_CLASS ( ( o ) , TYPE_GANYMED_SKELETON , GanymedSkeletonClass ) ) # */

/* IS_GANYMED_SKELETON ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_GANYMED_SKELETON ) ) # */

/* IS_GANYMED_SKELETON_CLASS ( k ) ( G_TYPE_CHECK_CLASS_TYPE ( ( k ) , TYPE_GANYMED_SKELETON ) ) typedef */


#![allow(dead_code)]
#![allow(unstable)]

extern crate libc;

/*
struct _LunaDHT
*/
#[repr(C)]
pub struct _LunaDHT;

/*
struct _LunaDHTIface
		(int) parent_iface
		(int (int *)) gboolean
*/
#[repr(C)]
pub struct _LunaDHTIface {
	parent_iface: libc::c_int,
	gboolean: extern fn (*mut libc::c_int) -> libc::c_int,
}

/*
struct _LunaDHTProxy
		(int) parent_instance
		(LunaDHTProxyPrivate *) priv [struct _LunaDHTProxyPrivate *]
*/
#[repr(C)]
pub struct _LunaDHTProxy {
	parent_instance: libc::c_int,
	priv_: *mut _LunaDHTProxyPrivate,
}

/*
struct _LunaDHTProxyClass
		(int) parent_class
*/
#[repr(C)]
pub struct _LunaDHTProxyClass {
	parent_class: libc::c_int,
}

/*
struct _LunaDHTProxyPrivate
*/
#[repr(C)]
pub struct _LunaDHTProxyPrivate;

/*
struct _LunaDHTSkeleton
		(int) parent_instance
		(LunaDHTSkeletonPrivate *) priv [struct _LunaDHTSkeletonPrivate *]
*/
#[repr(C)]
pub struct _LunaDHTSkeleton {
	parent_instance: libc::c_int,
	priv_: *mut _LunaDHTSkeletonPrivate,
}

/*
struct _LunaDHTSkeletonClass
		(int) parent_class
*/
#[repr(C)]
pub struct _LunaDHTSkeletonClass {
	parent_class: libc::c_int,
}

/*
struct _LunaDHTSkeletonPrivate
*/
#[repr(C)]
pub struct _LunaDHTSkeletonPrivate;

/*
struct LunaDHT
*/
#[repr(C)]
pub struct LunaDHT;

/*
int * luna_dht_interface_info()
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_interface_info() -> *mut libc::c_int;
}


/*
int luna_dht_override_properties()
	(int *) klass
	(int) property_id_begin
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_override_properties(klass: *mut libc::c_int, property_id_begin: libc::c_int) -> libc::c_int;
}


/*
void luna_dht_complete_join()
	(LunaDHT *) object [struct _LunaDHT *]
	(int *) invocation
*/
////#[link(name="foo")]
extern "C" {
	pub fn luna_dht_complete_join(object: *mut _LunaDHT, invocation: *mut libc::c_int);
}


/*
void luna_dht_complete_get()
	(LunaDHT *) object [struct _LunaDHT *]
	(int *) invocation
	(int *) results
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_complete_get(object: *mut _LunaDHT, invocation: *mut libc::c_int, results: *mut libc::c_int);
}


/*
void luna_dht_complete_put()
	(LunaDHT *) object [struct _LunaDHT *]
	(int *) invocation
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_complete_put(object: *mut _LunaDHT, invocation: *mut libc::c_int);
}


/*
void luna_dht_call_join()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(const int *) arg_host
	(int) arg_port
	(int *) cancellable
	(int) callback
	(int) user_data
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_join(proxy: *mut _LunaDHT, arg_host: *const libc::c_int, arg_port: libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int luna_dht_call_join_finish()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int *) res
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_join_finish(proxy: *mut _LunaDHT, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int luna_dht_call_join_sync()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(const int *) arg_host
	(int) arg_port
	(int *) cancellable
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_join_sync(proxy: *mut _LunaDHT, arg_host: *const libc::c_int, arg_port: libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void luna_dht_call_get()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int) arg_app_id
	(int *) arg_key
	(int *) cancellable
	(int) callback
	(int) user_data
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_get(proxy: *mut _LunaDHT, arg_app_id: libc::c_int, arg_key: *mut libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int luna_dht_call_get_finish()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int **) out_results
	(int *) res
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_get_finish(proxy: *mut _LunaDHT, out_results: *mut *mut libc::c_int, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int luna_dht_call_get_sync()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int) arg_app_id
	(int *) arg_key
	(int **) out_results
	(int *) cancellable
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_get_sync(proxy: *mut _LunaDHT, arg_app_id: libc::c_int, arg_key: *mut libc::c_int, out_results: *mut *mut libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
void luna_dht_call_put()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int) arg_app_id
	(int *) arg_key
	(int *) arg_value
	(int) arg_ttl
	(int *) cancellable
	(int) callback
	(int) user_data
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_put(proxy: *mut _LunaDHT, arg_app_id: libc::c_int, arg_key: *mut libc::c_int, arg_value: *mut libc::c_int, arg_ttl: libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
int luna_dht_call_put_finish()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int *) res
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_put_finish(proxy: *mut _LunaDHT, res: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int luna_dht_call_put_sync()
	(LunaDHT *) proxy [struct _LunaDHT *]
	(int) arg_app_id
	(int *) arg_key
	(int *) arg_value
	(int) arg_ttl
	(int *) cancellable
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_call_put_sync(proxy: *mut _LunaDHT, arg_app_id: libc::c_int, arg_key: *mut libc::c_int, arg_value: *mut libc::c_int, arg_ttl: libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> libc::c_int;
}


/*
int luna_dht_get_joined()
	(LunaDHT *) object [struct _LunaDHT *]
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_get_joined(object: *mut _LunaDHT) -> libc::c_int;
}


/*
void luna_dht_set_joined()
	(LunaDHT *) object [struct _LunaDHT *]
	(int) value
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_set_joined(object: *mut _LunaDHT, value: libc::c_int);
}


/*
void luna_dht_proxy_new()
	(int *) connection
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int) callback
	(int) user_data
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_proxy_new(connection: *mut libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
LunaDHT * luna_dht_proxy_new_finish() [struct _LunaDHT *]
	(int *) res
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_proxy_new_finish(res: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _LunaDHT;
}


/*
LunaDHT * luna_dht_proxy_new_sync() [struct _LunaDHT *]
	(int *) connection
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_proxy_new_sync(connection: *mut libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _LunaDHT;
}


/*
void luna_dht_proxy_new_for_bus()
	(int) bus_type
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int) callback
	(int) user_data
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_proxy_new_for_bus(bus_type: libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, callback: libc::c_int, user_data: libc::c_int);
}


/*
LunaDHT * luna_dht_proxy_new_for_bus_finish() [struct _LunaDHT *]
	(int *) res
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_proxy_new_for_bus_finish(res: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _LunaDHT;
}


/*
LunaDHT * luna_dht_proxy_new_for_bus_sync() [struct _LunaDHT *]
	(int) bus_type
	(int) flags
	(const int *) name
	(const int *) object_path
	(int *) cancellable
	(int **) error
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_proxy_new_for_bus_sync(bus_type: libc::c_int, flags: libc::c_int, name: *const libc::c_int, object_path: *const libc::c_int, cancellable: *mut libc::c_int, error: *mut *mut libc::c_int) -> *mut _LunaDHT;
}


/*
LunaDHT * luna_dht_skeleton_new() [struct _LunaDHT *]
*/
//#[link(name="foo")]
extern "C" {
	pub fn luna_dht_skeleton_new() -> *mut _LunaDHT;
}


/* __NETWORK_BINDINGS_H__ # */

/* TYPE_LUNA_DHT ( luna_dht_get_type ( ) ) # */

/* LUNA_DHT ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_LUNA_DHT , LunaDHT ) ) # */

/* IS_LUNA_DHT ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_LUNA_DHT ) ) # */

/* LUNA_DHT_GET_IFACE ( o ) ( G_TYPE_INSTANCE_GET_INTERFACE ( ( o ) , TYPE_LUNA_DHT , LunaDHTIface ) ) struct */

/* TYPE_LUNA_DHT_PROXY ( luna_dht_proxy_get_type ( ) ) # */

/* LUNA_DHT_PROXY ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_LUNA_DHT_PROXY , LunaDHTProxy ) ) # */

/* LUNA_DHT_PROXY_CLASS ( k ) ( G_TYPE_CHECK_CLASS_CAST ( ( k ) , TYPE_LUNA_DHT_PROXY , LunaDHTProxyClass ) ) # */

/* LUNA_DHT_PROXY_GET_CLASS ( o ) ( G_TYPE_INSTANCE_GET_CLASS ( ( o ) , TYPE_LUNA_DHT_PROXY , LunaDHTProxyClass ) ) # */

/* IS_LUNA_DHT_PROXY ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_LUNA_DHT_PROXY ) ) # */

/* IS_LUNA_DHT_PROXY_CLASS ( k ) ( G_TYPE_CHECK_CLASS_TYPE ( ( k ) , TYPE_LUNA_DHT_PROXY ) ) typedef */

/* TYPE_LUNA_DHT_SKELETON ( luna_dht_skeleton_get_type ( ) ) # */

/* LUNA_DHT_SKELETON ( o ) ( G_TYPE_CHECK_INSTANCE_CAST ( ( o ) , TYPE_LUNA_DHT_SKELETON , LunaDHTSkeleton ) ) # */

/* LUNA_DHT_SKELETON_CLASS ( k ) ( G_TYPE_CHECK_CLASS_CAST ( ( k ) , TYPE_LUNA_DHT_SKELETON , LunaDHTSkeletonClass ) ) # */

/* LUNA_DHT_SKELETON_GET_CLASS ( o ) ( G_TYPE_INSTANCE_GET_CLASS ( ( o ) , TYPE_LUNA_DHT_SKELETON , LunaDHTSkeletonClass ) ) # */

/* IS_LUNA_DHT_SKELETON ( o ) ( G_TYPE_CHECK_INSTANCE_TYPE ( ( o ) , TYPE_LUNA_DHT_SKELETON ) ) # */

/* IS_LUNA_DHT_SKELETON_CLASS ( k ) ( G_TYPE_CHECK_CLASS_TYPE ( ( k ) , TYPE_LUNA_DHT_SKELETON ) ) typedef */


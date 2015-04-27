/*
 * Generated by gdbus-codegen 2.32.4. DO NOT EDIT.
 *
 * The license of this code is the same as for the source it was derived from.
 */

#ifndef __INTERCOM_DBUS_BINDINGS_H__
#define __INTERCOM_DBUS_BINDINGS_H__

#include <gio/gio.h>

G_BEGIN_DECLS


/* ------------------------------------------------------------------------ */
/* Declarations for org.manuel.Intercom */

#define TYPE_INTERCOM (intercom_get_type ())
#define INTERCOM(o) (G_TYPE_CHECK_INSTANCE_CAST ((o), TYPE_INTERCOM, Intercom))
#define IS_INTERCOM(o) (G_TYPE_CHECK_INSTANCE_TYPE ((o), TYPE_INTERCOM))
#define INTERCOM_GET_IFACE(o) (G_TYPE_INSTANCE_GET_INTERFACE ((o), TYPE_INTERCOM, IntercomIface))

struct _Intercom;
typedef struct _Intercom Intercom;
typedef struct _IntercomIface IntercomIface;

struct _IntercomIface
{
  GTypeInterface parent_iface;

  gboolean (*handle_add_node) (
    Intercom *object,
    GDBusMethodInvocation *invocation,
    const gchar *const *arg_node_name,
    const gchar *arg_remote_public_key);

  gboolean (*handle_connect) (
    Intercom *object,
    GDBusMethodInvocation *invocation,
    GUnixFDList *fd_list,
    gint arg_socket_type,
    GVariant *arg_remote_public_key,
    guint arg_port,
    guint arg_timeout);

  gboolean (*handle_find_node) (
    Intercom *object,
    GDBusMethodInvocation *invocation,
    const gchar *const *arg_keywords);

};

GType intercom_get_type (void) G_GNUC_CONST;

GDBusInterfaceInfo *intercom_interface_info (void);
guint intercom_override_properties (GObjectClass *klass, guint property_id_begin);


/* D-Bus method call completion functions: */
void intercom_complete_connect (
    Intercom *object,
    GDBusMethodInvocation *invocation,
    GUnixFDList *fd_list,
    GVariant *fd);

void intercom_complete_add_node (
    Intercom *object,
    GDBusMethodInvocation *invocation);

void intercom_complete_find_node (
    Intercom *object,
    GDBusMethodInvocation *invocation,
    GVariant *nodes);



/* D-Bus method calls: */
void intercom_call_connect (
    Intercom *proxy,
    gint arg_socket_type,
    GVariant *arg_remote_public_key,
    guint arg_port,
    guint arg_timeout,
    GUnixFDList *fd_list,
    GCancellable *cancellable,
    GAsyncReadyCallback callback,
    gpointer user_data);

gboolean intercom_call_connect_finish (
    Intercom *proxy,
    GVariant **out_fd,
    GUnixFDList **out_fd_list,
    GAsyncResult *res,
    GError **error);

gboolean intercom_call_connect_sync (
    Intercom *proxy,
    gint arg_socket_type,
    GVariant *arg_remote_public_key,
    guint arg_port,
    guint arg_timeout,
    GUnixFDList  *fd_list,
    GVariant **out_fd,
    GUnixFDList **out_fd_list,
    GCancellable *cancellable,
    GError **error);

void intercom_call_add_node (
    Intercom *proxy,
    const gchar *const *arg_node_name,
    const gchar *arg_remote_public_key,
    GCancellable *cancellable,
    GAsyncReadyCallback callback,
    gpointer user_data);

gboolean intercom_call_add_node_finish (
    Intercom *proxy,
    GAsyncResult *res,
    GError **error);

gboolean intercom_call_add_node_sync (
    Intercom *proxy,
    const gchar *const *arg_node_name,
    const gchar *arg_remote_public_key,
    GCancellable *cancellable,
    GError **error);

void intercom_call_find_node (
    Intercom *proxy,
    const gchar *const *arg_keywords,
    GCancellable *cancellable,
    GAsyncReadyCallback callback,
    gpointer user_data);

gboolean intercom_call_find_node_finish (
    Intercom *proxy,
    GVariant **out_nodes,
    GAsyncResult *res,
    GError **error);

gboolean intercom_call_find_node_sync (
    Intercom *proxy,
    const gchar *const *arg_keywords,
    GVariant **out_nodes,
    GCancellable *cancellable,
    GError **error);



/* ---- */

#define TYPE_INTERCOM_PROXY (intercom_proxy_get_type ())
#define INTERCOM_PROXY(o) (G_TYPE_CHECK_INSTANCE_CAST ((o), TYPE_INTERCOM_PROXY, IntercomProxy))
#define INTERCOM_PROXY_CLASS(k) (G_TYPE_CHECK_CLASS_CAST ((k), TYPE_INTERCOM_PROXY, IntercomProxyClass))
#define INTERCOM_PROXY_GET_CLASS(o) (G_TYPE_INSTANCE_GET_CLASS ((o), TYPE_INTERCOM_PROXY, IntercomProxyClass))
#define IS_INTERCOM_PROXY(o) (G_TYPE_CHECK_INSTANCE_TYPE ((o), TYPE_INTERCOM_PROXY))
#define IS_INTERCOM_PROXY_CLASS(k) (G_TYPE_CHECK_CLASS_TYPE ((k), TYPE_INTERCOM_PROXY))

typedef struct _IntercomProxy IntercomProxy;
typedef struct _IntercomProxyClass IntercomProxyClass;
typedef struct _IntercomProxyPrivate IntercomProxyPrivate;

struct _IntercomProxy
{
  /*< private >*/
  GDBusProxy parent_instance;
  IntercomProxyPrivate *priv;
};

struct _IntercomProxyClass
{
  GDBusProxyClass parent_class;
};

GType intercom_proxy_get_type (void) G_GNUC_CONST;

void intercom_proxy_new (
    GDBusConnection     *connection,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GAsyncReadyCallback  callback,
    gpointer             user_data);
Intercom *intercom_proxy_new_finish (
    GAsyncResult        *res,
    GError             **error);
Intercom *intercom_proxy_new_sync (
    GDBusConnection     *connection,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GError             **error);

void intercom_proxy_new_for_bus (
    GBusType             bus_type,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GAsyncReadyCallback  callback,
    gpointer             user_data);
Intercom *intercom_proxy_new_for_bus_finish (
    GAsyncResult        *res,
    GError             **error);
Intercom *intercom_proxy_new_for_bus_sync (
    GBusType             bus_type,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GError             **error);


/* ---- */

#define TYPE_INTERCOM_SKELETON (intercom_skeleton_get_type ())
#define INTERCOM_SKELETON(o) (G_TYPE_CHECK_INSTANCE_CAST ((o), TYPE_INTERCOM_SKELETON, IntercomSkeleton))
#define INTERCOM_SKELETON_CLASS(k) (G_TYPE_CHECK_CLASS_CAST ((k), TYPE_INTERCOM_SKELETON, IntercomSkeletonClass))
#define INTERCOM_SKELETON_GET_CLASS(o) (G_TYPE_INSTANCE_GET_CLASS ((o), TYPE_INTERCOM_SKELETON, IntercomSkeletonClass))
#define IS_INTERCOM_SKELETON(o) (G_TYPE_CHECK_INSTANCE_TYPE ((o), TYPE_INTERCOM_SKELETON))
#define IS_INTERCOM_SKELETON_CLASS(k) (G_TYPE_CHECK_CLASS_TYPE ((k), TYPE_INTERCOM_SKELETON))

typedef struct _IntercomSkeleton IntercomSkeleton;
typedef struct _IntercomSkeletonClass IntercomSkeletonClass;
typedef struct _IntercomSkeletonPrivate IntercomSkeletonPrivate;

struct _IntercomSkeleton
{
  /*< private >*/
  GDBusInterfaceSkeleton parent_instance;
  IntercomSkeletonPrivate *priv;
};

struct _IntercomSkeletonClass
{
  GDBusInterfaceSkeletonClass parent_class;
};

GType intercom_skeleton_get_type (void) G_GNUC_CONST;

Intercom *intercom_skeleton_new (void);


G_END_DECLS

#endif /* __INTERCOM_DBUS_BINDINGS_H__ */
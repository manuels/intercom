/*
 * Generated by gdbus-codegen 2.32.4. DO NOT EDIT.
 *
 * The license of this code is the same as for the source it was derived from.
 */

#ifndef __GANYMED_BINDINGS_H__
#define __GANYMED_BINDINGS_H__

#include <gio/gio.h>

G_BEGIN_DECLS


/* ------------------------------------------------------------------------ */
/* Declarations for org.manuel.Ganymed */

#define TYPE_GANYMED (ganymed_get_type ())
#define GANYMED(o) (G_TYPE_CHECK_INSTANCE_CAST ((o), TYPE_GANYMED, Ganymed))
#define IS_GANYMED(o) (G_TYPE_CHECK_INSTANCE_TYPE ((o), TYPE_GANYMED))
#define GANYMED_GET_IFACE(o) (G_TYPE_INSTANCE_GET_INTERFACE ((o), TYPE_GANYMED, GanymedIface))

struct _Ganymed;
typedef struct _Ganymed Ganymed;
typedef struct _GanymedIface GanymedIface;

struct _GanymedIface
{
  GTypeInterface parent_iface;

  gboolean (*handle_add_node) (
    Ganymed *object,
    GDBusMethodInvocation *invocation,
    const gchar *const *arg_node_name,
    const gchar *arg_remote_public_key);

  gboolean (*handle_connect) (
    Ganymed *object,
    GDBusMethodInvocation *invocation,
    GUnixFDList *fd_list,
    GVariant *arg_remote_public_key,
    guint arg_port,
    guint arg_timeout);

  gboolean (*handle_find_node) (
    Ganymed *object,
    GDBusMethodInvocation *invocation,
    const gchar *const *arg_keywords);

};

GType ganymed_get_type (void) G_GNUC_CONST;

GDBusInterfaceInfo *ganymed_interface_info (void);
guint ganymed_override_properties (GObjectClass *klass, guint property_id_begin);


/* D-Bus method call completion functions: */
void ganymed_complete_connect (
    Ganymed *object,
    GDBusMethodInvocation *invocation,
    GUnixFDList *fd_list,
    GVariant *fd);

void ganymed_complete_add_node (
    Ganymed *object,
    GDBusMethodInvocation *invocation);

void ganymed_complete_find_node (
    Ganymed *object,
    GDBusMethodInvocation *invocation,
    GVariant *nodes);



/* D-Bus method calls: */
void ganymed_call_connect (
    Ganymed *proxy,
    GVariant *arg_remote_public_key,
    guint arg_port,
    guint arg_timeout,
    GUnixFDList *fd_list,
    GCancellable *cancellable,
    GAsyncReadyCallback callback,
    gpointer user_data);

gboolean ganymed_call_connect_finish (
    Ganymed *proxy,
    GVariant **out_fd,
    GUnixFDList **out_fd_list,
    GAsyncResult *res,
    GError **error);

gboolean ganymed_call_connect_sync (
    Ganymed *proxy,
    GVariant *arg_remote_public_key,
    guint arg_port,
    guint arg_timeout,
    GUnixFDList  *fd_list,
    GVariant **out_fd,
    GUnixFDList **out_fd_list,
    GCancellable *cancellable,
    GError **error);

void ganymed_call_add_node (
    Ganymed *proxy,
    const gchar *const *arg_node_name,
    const gchar *arg_remote_public_key,
    GCancellable *cancellable,
    GAsyncReadyCallback callback,
    gpointer user_data);

gboolean ganymed_call_add_node_finish (
    Ganymed *proxy,
    GAsyncResult *res,
    GError **error);

gboolean ganymed_call_add_node_sync (
    Ganymed *proxy,
    const gchar *const *arg_node_name,
    const gchar *arg_remote_public_key,
    GCancellable *cancellable,
    GError **error);

void ganymed_call_find_node (
    Ganymed *proxy,
    const gchar *const *arg_keywords,
    GCancellable *cancellable,
    GAsyncReadyCallback callback,
    gpointer user_data);

gboolean ganymed_call_find_node_finish (
    Ganymed *proxy,
    GVariant **out_nodes,
    GAsyncResult *res,
    GError **error);

gboolean ganymed_call_find_node_sync (
    Ganymed *proxy,
    const gchar *const *arg_keywords,
    GVariant **out_nodes,
    GCancellable *cancellable,
    GError **error);



/* ---- */

#define TYPE_GANYMED_PROXY (ganymed_proxy_get_type ())
#define GANYMED_PROXY(o) (G_TYPE_CHECK_INSTANCE_CAST ((o), TYPE_GANYMED_PROXY, GanymedProxy))
#define GANYMED_PROXY_CLASS(k) (G_TYPE_CHECK_CLASS_CAST ((k), TYPE_GANYMED_PROXY, GanymedProxyClass))
#define GANYMED_PROXY_GET_CLASS(o) (G_TYPE_INSTANCE_GET_CLASS ((o), TYPE_GANYMED_PROXY, GanymedProxyClass))
#define IS_GANYMED_PROXY(o) (G_TYPE_CHECK_INSTANCE_TYPE ((o), TYPE_GANYMED_PROXY))
#define IS_GANYMED_PROXY_CLASS(k) (G_TYPE_CHECK_CLASS_TYPE ((k), TYPE_GANYMED_PROXY))

typedef struct _GanymedProxy GanymedProxy;
typedef struct _GanymedProxyClass GanymedProxyClass;
typedef struct _GanymedProxyPrivate GanymedProxyPrivate;

struct _GanymedProxy
{
  /*< private >*/
  GDBusProxy parent_instance;
  GanymedProxyPrivate *priv;
};

struct _GanymedProxyClass
{
  GDBusProxyClass parent_class;
};

GType ganymed_proxy_get_type (void) G_GNUC_CONST;

void ganymed_proxy_new (
    GDBusConnection     *connection,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GAsyncReadyCallback  callback,
    gpointer             user_data);
Ganymed *ganymed_proxy_new_finish (
    GAsyncResult        *res,
    GError             **error);
Ganymed *ganymed_proxy_new_sync (
    GDBusConnection     *connection,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GError             **error);

void ganymed_proxy_new_for_bus (
    GBusType             bus_type,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GAsyncReadyCallback  callback,
    gpointer             user_data);
Ganymed *ganymed_proxy_new_for_bus_finish (
    GAsyncResult        *res,
    GError             **error);
Ganymed *ganymed_proxy_new_for_bus_sync (
    GBusType             bus_type,
    GDBusProxyFlags      flags,
    const gchar         *name,
    const gchar         *object_path,
    GCancellable        *cancellable,
    GError             **error);


/* ---- */

#define TYPE_GANYMED_SKELETON (ganymed_skeleton_get_type ())
#define GANYMED_SKELETON(o) (G_TYPE_CHECK_INSTANCE_CAST ((o), TYPE_GANYMED_SKELETON, GanymedSkeleton))
#define GANYMED_SKELETON_CLASS(k) (G_TYPE_CHECK_CLASS_CAST ((k), TYPE_GANYMED_SKELETON, GanymedSkeletonClass))
#define GANYMED_SKELETON_GET_CLASS(o) (G_TYPE_INSTANCE_GET_CLASS ((o), TYPE_GANYMED_SKELETON, GanymedSkeletonClass))
#define IS_GANYMED_SKELETON(o) (G_TYPE_CHECK_INSTANCE_TYPE ((o), TYPE_GANYMED_SKELETON))
#define IS_GANYMED_SKELETON_CLASS(k) (G_TYPE_CHECK_CLASS_TYPE ((k), TYPE_GANYMED_SKELETON))

typedef struct _GanymedSkeleton GanymedSkeleton;
typedef struct _GanymedSkeletonClass GanymedSkeletonClass;
typedef struct _GanymedSkeletonPrivate GanymedSkeletonPrivate;

struct _GanymedSkeleton
{
  /*< private >*/
  GDBusInterfaceSkeleton parent_instance;
  GanymedSkeletonPrivate *priv;
};

struct _GanymedSkeletonClass
{
  GDBusInterfaceSkeletonClass parent_class;
};

GType ganymed_skeleton_get_type (void) G_GNUC_CONST;

Ganymed *ganymed_skeleton_new (void);


G_END_DECLS

#endif /* __GANYMED_BINDINGS_H__ */

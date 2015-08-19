#!/usr/bin/env python

from contextlib import contextmanager
import dbus
import socket

AF_UNIX     = socket.AF_UNIX
SOCK_STREAM = socket.SOCK_STREAM
SOCK_DGRAM  = socket.SOCK_DGRAM

SessionBus  = dbus.SessionBus
SystemBus   = dbus.SystemBus

@contextmanager
def connect(local_app_id,
            remote_app_id,
            peername=None,
            public_key=None,
            sock_type=SOCK_STREAM,
            timeout_sec=5*60,
            dbus_path='org.manuel.Intercom',
            bus=SessionBus):
	sock_type   = dbus.Int32(sock_type)
	app_id      = dbus.String("tcptunnel")
	timeout_sec = dbus.UInt32(timeout_sec)

	intercom = bus().get_object(dbus_path, "/", introspect=False)

	if public_key is None:
		peername  = dbus.String(peername)
		result = intercom.ConnectToPeer(sock_type,
			peername,
			local_app_id,
			remote_app_id,
			timeout_sec,
			dbus_interface='org.manuel.Intercom',
			timeout=timeout_sec)
	else:
		public_key  = dbus.String(public_key)
		result = intercom.ConnectToKey(sock_type,
			public_key,
			local_app_id,
			remote_app_id,
			timeout_sec,
			dbus_interface='org.manuel.Intercom',
			timeout=timeout_sec)

	fd     = result.take()
	sock   = socket.fromfd(fd, socket.AF_UNIX, sock_type, 0)

	yield sock

	sock.close()

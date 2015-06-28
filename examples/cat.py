#!/usr/bin/env python

# !!! DOES NOT WORK YET !!!

import sys
import dbus
import socket
import select
import os

dbus_path = sys.argv[1]
pub_key   = sys.argv[2]

bus = dbus.SessionBus()
intercom = bus.get_object(dbus_path, "/", introspect=False)

domain      = dbus.Int32(socket.SOCK_DGRAM)
public_key  = dbus.String(pub_key)
app_id      = dbus.String("cat")
timeout_sec = dbus.UInt32(60)

result = intercom.Connect(domain, public_key, app_id, timeout_sec, dbus_interface='org.manuel.Intercom')
fd     = result.take()
sock   = socket.fromfd(fd, socket.AF_UNIX, socket.SOCK_DGRAM, 0)

x = []
while len(x) == 0:
	r = [sys.stdin, sock]
	w = []
	x = []

	r, w, x = select.select(r, w, x)

	if sys.stdin in r:
		buf = sys.stdin.read(65535)
		sock.send(buf)

	if sock in r:
		buf = sock.recv(65535)
		sys.stdout.write(buf)

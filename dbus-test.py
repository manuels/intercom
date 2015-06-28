#!/usr/bin/env python

import sys
import dbus
import socket

dbus_path = sys.argv[1]
pub_key = sys.argv[2]

bus = dbus.SessionBus()
intercom = bus.get_object(dbus_path, "/", introspect=False)

sock_type   = socket.SOCK_STREAM
domain      = dbus.Int32(sock_type)
public_key  = dbus.String(pub_key)
port        = dbus.String("test")
timeout_sec = dbus.UInt32(2*60)

result = intercom.Connect(domain, public_key, port, timeout_sec, dbus_interface='org.manuel.Intercom', timeout=2*60)

fd   = result.take()
sock = socket.fromfd(fd, socket.AF_UNIX, sock_type, 0)

for i in range(3):
	sent = pub_key
	sock.send(sent)

	recved = sock.recv(1024)
	assert(sent != recved)
	print "{} received {}!".format(sent, recved)

print "ALL DONE!"

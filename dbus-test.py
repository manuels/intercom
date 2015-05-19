#!/usr/bin/env python

import sys
import dbus
import socket

dbus_path = sys.argv[1]
pub_key = sys.argv[2]

bus = dbus.SessionBus()
intercom = bus.get_object(dbus_path, "/org/manuel/Intercom", introspect=False)

domain      = dbus.Int32(socket.SOCK_DGRAM)
public_key  = dbus.ByteArray(pub_key)
port        = dbus.UInt32(0)
timeout_sec = dbus.UInt32(10)

result = intercom.connect(domain, public_key, port, timeout_sec, dbus_interface='org.manuel.Intercom')

fd   = result.take()
sock = socket.fromfd(fd, socket.AF_UNIX, socket.SOCK_DGRAM, 0)

for i in range(3):
	sent = pub_key
	sock.send(sent)

	recved = sock.recv(1024)
	assert(sent != recved)
	print "{} received {}!".format(sent, recved)

print "ALL DONE!"

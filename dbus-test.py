#!/usr/bin/env python

import socket

import gobject
gobject.threads_init()

from dbus.glib import DBusGMainLoop
DBusGMainLoop (set_as_default=True)
loop = gobject.MainLoop()

import sys

from dbus import glib
glib.init_threads()

import dbus
bus = dbus.SessionBus()

dbus_path = sys.argv[1]
pub_key = map(int, sys.argv[2].split(','))

introspect = False
if introspect:
	args = (2, pub_key, 99, 10)
	remote_object = bus.get_object(dbus_path, "/org/manuel/Intercom", introspect=True)
else:
	args = (dbus.Int32(2), dbus.ByteArray("".join(map(chr, pub_key))), dbus.UInt32(99), dbus.UInt32(10))
	remote_object = bus.get_object(dbus_path, "/org/manuel/Intercom", introspect=False)

def cb(fd):
	sock = socket.fromfd(fd.take(), socket.AF_UNIX, socket.SOCK_DGRAM, 0)

	for i in range(3):
		sent = "".join(map(chr, pub_key))
		sock.send(sent)

		recved = sock.recv(1024)
		assert(sent != recved)
		print "{} received {}!".format(sent, recved)
	print "ALL DONE!"
	loop.quit()

func = remote_object.get_dbus_method('connect', dbus_interface='org.manuel.Intercom')
func.call_async(*args, reply_handler=cb)

loop.run()

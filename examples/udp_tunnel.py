#!/usr/bin/env python

import sys
import dbus
import socket
import select
import os

proto = socket.SOCK_DGRAM

dbus_path = sys.argv[1] # 'org.manuel.Intercom'
pub_key   = sys.argv[2]

mode   = sys.argv[3]
if mode not in ['publish', 'connect']:
	sys.exit(1)
if mode == 'publish':
	other_mode = 'connect'
else:
	other_mode = 'publish'

host   = sys.argv[4]
port   = sys.argv[5]

def intercom(pub_key):
	bus = dbus.SessionBus()
	intercom = bus.get_object(dbus_path, "/", introspect=False)

	domain      = dbus.Int32(proto)
	public_key  = dbus.String(pub_key)
	app_id      = dbus.String("udptunnel")
	timeout_sec = dbus.UInt32(5*60)

	result = intercom.ConnectToKey(domain, public_key, app_id+mode, app_id+other_mode, timeout_sec, dbus_interface='org.manuel.Intercom', timeout=5*60)
	fd     = result.take()
	sock   = socket.fromfd(fd, socket.AF_UNIX, proto, 0)
	return sock
if mode == 'connect':
	print 'Listening on {}:{}'.format(host, port)
	s = socket.socket(socket.AF_INET, proto)
	s.bind((host, int(port)))

	local_sock = s
	print 'UDP connection open.'

	print 'Waiting for Intercom.'
	remote_sock      = intercom(pub_key)
	print 'Intercom established.'

elif mode == 'publish':
	print 'Waiting for Intercom.'
	remote_sock      = intercom(pub_key)
	print 'Intercom established.'

	local_sock = socket.socket(socket.AF_INET, proto)
	local_sock.connect((host, int(port)))	
	print 'UDP connection open.'


x = []
while len(x) == 0:
	r = [local_sock, remote_sock]
	w = []
	x = []

	r, w, x = select.select(r, w, x)

	for sock_in, sock_out in [(local_sock, remote_sock),
	                          (remote_sock, local_sock)]:
		if sock_in in r:
			buf = sock_in.recv(65535)
			sock_out.send(buf)

local_sock.close()
remote_sock.close()

print 'Connection closed.'

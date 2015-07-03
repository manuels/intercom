#!/usr/bin/env python

import sys
import dbus
import socket
import select
import os

dbus_path = sys.argv[1] # 'org.manuel.Intercom'
pub_key   = sys.argv[2]

mode   = sys.argv[3]
if mode not in ['publish', 'connect']:
	sys.exit(1)

host   = sys.argv[4]
port   = sys.argv[5]

def intercom(pub_key):
	bus = dbus.SessionBus()
	intercom = bus.get_object(dbus_path, "/", introspect=False)

	domain      = dbus.Int32(socket.SOCK_STREAM)
	public_key  = dbus.String(pub_key)
	app_id      = dbus.String("tcptunnel")
	timeout_sec = dbus.UInt32(5*60)

	result = intercom.Connect(domain, public_key, app_id, timeout_sec, dbus_interface='org.manuel.Intercom', timeout=5*60)
	fd     = result.take()
	sock   = socket.fromfd(fd, socket.AF_UNIX, socket.SOCK_STREAM, 0)
	return sock

if mode == 'connect':
	print 'Listening on {}:{}'.format(host, port)
	s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
	s.bind((host, int(port)))
	s.listen(1)

	print 'Waiting for TCP connection.'
	local_sock, addr = s.accept()
	print 'TCP connection established.'

	print 'Waiting for Intercom.'
	remote_sock      = intercom(pub_key)
	print 'Intercom established.'

elif mode == 'publish':
	print 'Waiting for Intercom.'
	remote_sock      = intercom(pub_key)
	print 'Intercom established.'

	print 'Waiting for TCP connection.'
	local_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
	local_sock.connect((host, int(port)))	
	print 'TCP connection established.'


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

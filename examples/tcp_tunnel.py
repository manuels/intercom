#!/usr/bin/env python

import sys
import socket
import select
import intercom

peername  = sys.argv[1]
mode   = sys.argv[2]
if mode not in ['publish', 'connect']:
	print "invalid mode: '{}'".format(mode)
	sys.exit(1)
if mode == 'publish':
	other_mode = 'connect'
else:
	other_mode = 'publish'

host, port = sys.argv[3].split(':')
dbus_path  = sys.argv[4] if len(sys.argv) > 4 else 'org.manuel.Intercom'

app_id        = 'tcptunnel'
local_app_id  = app_id+mode
remote_app_id = app_id+other_mode

print 'Waiting for Intercom {}.'.format(dbus_path)
with intercom.connect(local_app_id, remote_app_id,
		peername=peername, dbus_path=dbus_path) as remote_sock:
	print 'Intercom established.'

	proto = socket.SOCK_STREAM
	if mode == 'connect':
		print 'Listening on {}:{}'.format(host, port)
		s = socket.socket(socket.AF_INET, proto)
		s.bind((host, int(port)))
		s.listen(1)

		print 'Waiting for TCP connection.'
		local_sock, addr = s.accept()
		print 'TCP connection established.'
	elif mode == 'publish':
		print 'Waiting for TCP connection.'
		local_sock = socket.socket(socket.AF_INET, proto)
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

print 'Connection closed.'

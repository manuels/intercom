#!/usr/bin/env python

import sys
import select
import intercom

peername  = sys.argv[1]
dbus_path = sys.argv[2] if len(sys.argv) > 2 else 'org.manuel.Intercom'
app_id    = 'cat'

with intercom.connect(app_id, app_id,
		peername=peername, dbus_path=dbus_path) as sock:
	print 'SOCKET OPEN to {}'.format(peername)

	i = 0
	x = []
	while len(x) == 0 and i < 5:
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

		i += 1
		print i

	print 'CLOSING SOCKET'

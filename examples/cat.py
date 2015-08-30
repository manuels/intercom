#!/usr/bin/env python
"""intercom-cat.

Usage:
  cat.py [options] peer PEERNAME
  cat.py [options] key REMOTE_PUB_KEY
  cat.py (-h | --help)
  cat.py --version

Options:
  -h --help                  Show this screen.
  --version                  Show version.
  --dbus=DBUS_PATH           DBus path to intercom [default: org.manuel.Intercom].
  --app_id=APP_ID            Alias for setting local and remote app_id [default: cat].
  -l --local_app_id=APP_ID   Intercom's local_app_id.
  -r --remote_app_id=APP_ID  Intercom's remote_app_id.

"""
from docopt import docopt

import sys
import select
import intercom

def cat(peername, local_app_id, remote_app_id, dbus_path):
	with intercom.connect(local_app_id, remote_app_id,
			peername=peername, dbus_path=dbus_path) as sock:
		sys.stdout.write('SOCKET OPENED to {}\n'.format(peername))
	
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

		sys.stdout.write('CLOSING SOCKET\n')

if __name__ == '__main__':
	args = docopt(__doc__, version='intercom-cat 0.1')

	local_app_id  = args['--local_app_id'] or args['--app_id']
	remote_app_id = args['--remote_app_id'] or args['--app_id']

	cat(args['PEERNAME'], local_app_id, remote_app_id, args['--dbus'])

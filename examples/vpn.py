#!/usr/bin/env python

# requires:
# $ pip install python-pytun --user

import sys
import dbus
import socket
import pytun
import select
import os

user_id   = int(sys.argv[1])
dbus_path = sys.argv[2]
pub_key   = sys.argv[3]
local_ip_addr = sys.argv[4]
remote_ip_addr = sys.argv[5]

os.seteuid(user_id)
bus = dbus.SessionBus()
intercom = bus.get_object(dbus_path, "/org/manuel/Intercom", introspect=False)

domain      = dbus.Int32(socket.SOCK_DGRAM)
public_key  = dbus.ByteArray(pub_key)
app_id      = dbus.ByteArray("pyvpn")
timeout_sec = dbus.UInt32(60)

result = intercom.connect(domain, public_key, app_id, timeout_sec, dbus_interface='org.manuel.Intercom')
fd     = result.take()
sock   = socket.fromfd(fd, socket.AF_UNIX, socket.SOCK_DGRAM, 0)

os.seteuid(0)

tuntap = pytun.TunTapDevice(flags=pytun.IFF_TUN|pytun.IFF_NO_PI)
tuntap.addr    = local_ip_addr
tuntap.dstaddr = remote_ip_addr
tuntap.netmask = '255.255.255.0'
tuntap.mtu = 1500
tuntap.up()

x = []
while len(x) == 0:
	r = [tuntap, sock]
	w = []
	x = []

	r, w, x = select.select(r, w, x)

	if tuntap in r:
		buf = tuntap.read(65535)
		sock.send(buf)

	if sock in r:
		buf = sock.recv(65535)
		tuntap.write(buf)

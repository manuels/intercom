killall -9 'main'
rm -f /tmp/fake_dht.txt
touch /tmp/fake_dht.txt

PRIVATE_KEY1=01E2B7ADCD04C3B00163EEEE67E565F3883EEE435546ED96CF1845EC616B494FBEECD8849FCC78B6F2EA5F231419DDA52CAE2383D23D2A8F20BB39A7F1D08F334B2B
PRIVATE_KEY2=014920C7E99770FBE262881546DD0EF7268781AF47D8E336C1BC0A78B77E42DF0673640B0BA86BB7BF10C57F31A554DD250C0B411E7B28967AC88AD0C42C4963861A

PUBLIC_KEY1=0300AB25A03B2785165930E0C1E71ECE30E74E5F05954826A84FCE7A2AEFA1F61F852F20259931B0E1F684779BE034088632FD99518EC3EB79E7F52BF5C4D611CB295E
PUBLIC_KEY2=0301289B57E79AADDBE9557ADEF770F568E381DD006D540FDBBE9CF5D68D6ACDE6F95A613AB7D6CD4C203A84F134164007D817555446E55DA757B6CC152D0395330145

RUST_LOG=debug ./target/debug/main org.manuel.test_intercom1 ${PRIVATE_KEY1} 2>&1 &
RUST_LOG=debug ./target/debug/main org.manuel.test_intercom2 ${PRIVATE_KEY2} 2>&1 &
sleep 1

if [ 1 -eq 1 ]; then
	./dbus-test.py org.manuel.test_intercom1 ${PUBLIC_KEY2} &
	./dbus-test.py org.manuel.test_intercom2 ${PUBLIC_KEY1} &
else
	sudo -i echo

	sudo ip link delete tun0
	sudo ip link delete tun1
	sleep 1 

	sudo -E ./examples/vpn.py `id -u` org.manuel.test_intercom1 ${PUBLIC_KEY2} 10.99.0.2 10.99.0.1 &
	sleep 1 
	sudo -E ./examples/vpn.py `id -u` org.manuel.test_intercom2 ${PUBLIC_KEY1} 10.99.0.1 10.99.0.2 &

	sleep 1

	sudo route add -host 10.99.0.2 tun0
	sudo route add -host 10.99.0.1 tun1

	echo 1 | sudo tee /proc/sys/net/ipv4/conf/tun0/proxy_arp 1>/dev/null
	echo 1 | sudo tee /proc/sys/net/ipv4/conf/tun1/proxy_arp 1>/dev/null
	echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward 1>/dev/null

	sudo arp -Ds 10.99.0.2 tun0 pub
	sudo arp -Ds 10.99.0.1 tun1 pub

	wait

	killall -9  'main'
	sudo killall -9  'vpn.py'

	sudo ip link delete tun0
	sudo ip link delete tun1
fi
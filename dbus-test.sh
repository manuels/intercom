killall -9 'main'
rm -f /tmp/fake_dht.txt
touch /tmp/fake_dht.txt

RUST_LOG=debug ./target/debug/main org.manuel.test_intercom1 01E2B7ADCD04C3B00163EEEE67E565F3883EEE435546ED96CF1845EC616B494FBEECD8849FCC78B6F2EA5F231419DDA52CAE2383D23D2A8F20BB39A7F1D08F334B2B 2>&1 &
RUST_LOG=debug ./target/debug/main org.manuel.test_intercom2 014920C7E99770FBE262881546DD0EF7268781AF47D8E336C1BC0A78B77E42DF0673640B0BA86BB7BF10C57F31A554DD250C0B411E7B28967AC88AD0C42C4963861A 2>&1 &

sleep 3

if [ 0 -eq 1 ]; then
	dbus-send --print-reply --session --type=method_call \
		--dest=org.manuel.test_intercom1 /org/manuel/Intercom \
		org.manuel.Intercom.connect int32:2 array:byte:48,51,48,49,50,56,57,66,53,55,69,55,57,65,65,68,68,66,69,57,53,53,55,65,68,69,70,55,55,48,70,53,54,56,69,51,56,49,68,68,48,48,54,68,53,52,48,70,68,66,66,69,57,67,70,53,68,54,56,68,54,65,67,68,69,54,70,57,53,65,54,49,51,65,66,55,68,54,67,68,52,67,50,48,51,65,56,52,70,49,51,52,49,54,52,48,48,55,68,56,49,55,53,53,53,52,52,54,69,53,53,68,65,55,53,55,66,54,67,67,49,53,50,68,48,51,57,53,51,51,48,49,52,53 uint32:1 uint32:15 &

	dbus-send --print-reply --session --type=method_call \
		--dest=org.manuel.test_intercom2 /org/manuel/Intercom \
		org.manuel.Intercom.connect int32:2 array:byte:48,51,48,48,65,66,50,53,65,48,51,66,50,55,56,53,49,54,53,57,51,48,69,48,67,49,69,55,49,69,67,69,51,48,69,55,52,69,53,70,48,53,57,53,52,56,50,54,65,56,52,70,67,69,55,65,50,65,69,70,65,49,70,54,49,70,56,53,50,70,50,48,50,53,57,57,51,49,66,48,69,49,70,54,56,52,55,55,57,66,69,48,51,52,48,56,56,54,51,50,70,68,57,57,53,49,56,69,67,51,69,66,55,57,69,55,70,53,50,66,70,53,67,52,68,54,49,49,67,66,50,57,53,69 uint32:1 uint32:15 &
else
	./dbus-test.py org.manuel.test_intercom1 48,51,48,49,50,56,57,66,53,55,69,55,57,65,65,68,68,66,69,57,53,53,55,65,68,69,70,55,55,48,70,53,54,56,69,51,56,49,68,68,48,48,54,68,53,52,48,70,68,66,66,69,57,67,70,53,68,54,56,68,54,65,67,68,69,54,70,57,53,65,54,49,51,65,66,55,68,54,67,68,52,67,50,48,51,65,56,52,70,49,51,52,49,54,52,48,48,55,68,56,49,55,53,53,53,52,52,54,69,53,53,68,65,55,53,55,66,54,67,67,49,53,50,68,48,51,57,53,51,51,48,49,52,53 &
	./dbus-test.py org.manuel.test_intercom2 48,51,48,48,65,66,50,53,65,48,51,66,50,55,56,53,49,54,53,57,51,48,69,48,67,49,69,55,49,69,67,69,51,48,69,55,52,69,53,70,48,53,57,53,52,56,50,54,65,56,52,70,67,69,55,65,50,65,69,70,65,49,70,54,49,70,56,53,50,70,50,48,50,53,57,57,51,49,66,48,69,49,70,54,56,52,55,55,57,66,69,48,51,52,48,56,56,54,51,50,70,68,57,57,53,49,56,69,67,51,69,66,55,57,69,55,70,53,50,66,70,53,67,52,68,54,49,49,67,66,50,57,53,69 &
fi

wait

killall 'main'

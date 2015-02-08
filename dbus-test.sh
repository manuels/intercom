killall '<main>'

RUST_LOG=debug ./target/main org.manuel.test_ganymed1 1 &
RUST_LOG=debug ./target/main org.manuel.test_ganymed2 2 &

sleep 3

dbus-send --print-reply --session --type=method_call \
	--dest=org.manuel.test_ganymed1 /org/manuel/Ganymed \
	org.manuel.Ganymed.connect array:byte:50 uint32:1 uint32:15 &

dbus-send --print-reply --session --type=method_call \
	--dest=org.manuel.test_ganymed2 /org/manuel/Ganymed \
	org.manuel.Ganymed.connect array:byte:49 uint32:1 uint32:15 &

wait

killall '<main>'

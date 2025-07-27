#!/bin/bash
# run from the root project directory

# dbus backend test
printf "\nTesting out dbus backend.\n"
cargo build --example print_events --no-default-features --features mpris_dbus,mpris_base64_data_url
timeout 5s ./target/debug/examples/print_events &
sleep 0.5
./tests/playerctl_script.sh > target/dbus_output.txt &
wait

# zbus backend test
printf "\nTesting out zbus backend.\n"
cargo build --example print_events --no-default-features --features mpris_zbus,mpris_base64_data_url
timeout 5s ./target/debug/examples/print_events &
sleep 0.5
./tests/playerctl_script.sh > target/zbus_output.txt &
wait

# check for equality
printf "\Printing diff between outputs.\n"
diff target/dbus_output.txt target/zbus_output.txt

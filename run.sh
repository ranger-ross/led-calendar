DEVICE="/dev/hidraw11"

if [ ! -e "$DEVICE" ]; then
    echo "Error: Device $DEVICE does not exist"
    exit 1
fi


if [ ! -w "$DEVICE" ]; then
    sudo chown "$USER" "$DEVICE"
fi

cargo run

#!/bin/bash

set -e

APP_NAME="kbk-attendance"
APP_DIR="$(pwd)"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"

# Find cargo path
CARGO_PATH=$(which cargo)
if [ -z "$CARGO_PATH" ]; then
    # Common cargo installation path
    CARGO_PATH="$HOME/.cargo/bin/cargo"
fi

echo "Using cargo at: $CARGO_PATH"
echo "Creating systemd service..."
sudo tee ${SERVICE_FILE} > /dev/null << EOF
[Unit]
Description=KBK Attendance Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=${APP_DIR}
Environment="PATH=$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
ExecStart=${CARGO_PATH} run --release
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

echo "Reloading systemd daemon..."
sudo systemctl daemon-reload

echo "Enabling service..."
sudo systemctl enable ${APP_NAME}.service

echo "Starting service..."
sudo systemctl start ${APP_NAME}.service

echo "Service status:"
sudo systemctl status ${APP_NAME}.service --no-pager

echo ""
echo "Deployment complete!"
echo "Commands:"
echo "  sudo systemctl status ${APP_NAME}"
echo "  sudo journalctl -u ${APP_NAME} -f"
echo "  sudo systemctl restart ${APP_NAME}"
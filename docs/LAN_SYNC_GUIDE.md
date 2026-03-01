# LAN Sync Guide

This document explains how to use Pastry LAN sync.

## Overview

LAN sync lets devices in the same local network access and sync clipboard data through a browser.

- Default address format: `http://<your-lan-ip>:8080`
- Default port: `8080`

## Enable LAN Sync

1. Open Pastry settings.
2. Turn on **LAN sync**.
3. Confirm the app shows a QR code and an access URL.

If no LAN address is detected, check your Wi-Fi / network connection and try again.

## Connect from Another Device

1. Ensure your phone/tablet/computer is on the same LAN as the Pastry host.
2. Scan the QR code in Pastry, or open the shown URL manually.
3. Use the web page to send or receive clipboard content.

## Web Page Features

- **Receive tab**: shows the latest synced content, supports one-click copy.
- **Send tab**: push text, links, code snippets, and images to Pastry.
- **Real-time update**: updates are pushed live through WebSocket.

## Health Check

Open `http://<your-lan-ip>:8080/health`.

- If the service is available, it returns `ok`.

## Troubleshooting

- Cannot open page: verify same LAN, then check firewall/router rules for port `8080`.
- QR scan works but page fails: try opening the URL manually.
- Sync not updating: refresh the web page and ensure LAN sync is still enabled in Pastry.

## Security Notes

- The LAN sync service listens on `0.0.0.0:8080`.
- Any device that can reach this address in your LAN may access the sync page.
- Use trusted networks only, and disable LAN sync when not needed.

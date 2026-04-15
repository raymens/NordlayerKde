# NordLayer KDE Tray (Rust)

Small KDE-friendly system tray app that wraps NordLayer CLI commands.

## Features

- **Colored shield icon** in Plasma system tray:
  - 🟢 Green when connected
  - 🔴 Red when disconnected
  - 🟡 Amber when not logged in
  - 🟠 Orange when connecting/reconnecting
  - ⚫ Grey on error
- **Status display** in menu: shows connection state + currently connected gateway (if any)
- **Private Gateways** submenu: lists your private gateways with click-to-connect
- **Shared Gateways** submenu: lists public shared gateways with click-to-connect
- **Checkmark (✓)** next to the currently connected gateway
- **Quick actions**: Disconnect, Login, Refresh Status, Refresh Gateways, Quit
- **Desktop notifications** for all command results and errors

## Prerequisites

- Linux desktop session with DBus (KDE Plasma recommended)
- `nordlayer` CLI installed and available in `PATH`

Optional:
- `NORDLAYER_BIN=/path/to/nordlayer` if binary is not named `nordlayer`

## Build & Run

```bash
cd /home/raymens/RustroverProjects/NordlayerKde
cargo run
```

The tray icon will appear in your Plasma system tray (usually bottom-right).

## Test

```bash
cargo test
```

## How it works

- Queries `nordlayer status` (plain-text) to detect connection state + active gateway
- Queries `nordlayer gateways -f` (template) to list private and shared gateways by ID and name
- Separates gateways into two submenus for easy navigation
- Shows a checkmark next to whichever gateway you're connected to
- All menu items refresh status after running actions, so you immediately see the result


# Loomer - Wayland Screen Zoom Tool

A Rust-based screen zoom utility for Wayland compositors that allows you to zoom into specific regions of your screen.

## Features

- **Region Selection**: Click and drag to select any area of the screen to zoom
- **Smooth Zooming**: Mouse wheel support for precise zoom control
- **Panning**: Middle mouse button to navigate when zoomed in
- **Multi-monitor Support**: Choose which monitor to capture with `--monitor` flag
- **Transparent Window**: Undecorated, transparent overlay

## Installation

```bash
git clone https://github.com/Harshit-Dhanwalkar/loomer
cd loomer
cargo build --release
```

## Usage

```bash
# Basic usage (captures primary monitor)
./target/release/loomer

# Capture specific monitor
./target/release/loomer --monitor "HDMI-1"

# With cargo
cargo run -- --monitor "eDP-1"
```

## Controls

- Left Click + Drag: Select region to zoom into
- Mouse Wheel: Zoom in/out (when not in region mode)
- Middle Mouse Button: Pan when zoomed in
- Space: Reset zoom to full screen
- Q or Right Click: Exit application

# LICENSE

[MIT](LICENSE)

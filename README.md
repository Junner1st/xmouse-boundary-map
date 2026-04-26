# xmouse-boundary-map

X11 monitor boundary mapper for GNOME Shell/X11. It maps pointer crossings between horizontally adjacent monitors with different resolutions.

## Build

Install a requirement
```bash
cargo install cargo-deb
```

build method
```bash
cargo deb
```

## Check Monitor Names

```bash
cargo run -- --list-monitors
```

Example:

```text
DP-1: 3840x2160+0+0
HDMI-1: 1920x1080+3840+0
```

Adjacent monitors are auto-mapped by relative height. XInput2 raw motion is used so high-to-low resolution edges work even when X11 blocks the pointer at the larger monitor edge.

## Run

```bash
cargo run --
```

Installed packages can run it as a systemd user service:

```bash
systemctl --user daemon-reload
systemctl --user enable --now xmouse-boundary-map.service
systemctl --user status xmouse-boundary-map.service
```

The package installs a user unit, not a system unit, because the daemon needs
the logged-in X11 session. Use `systemctl --user ...`; plain
`systemctl status xmouse-boundary-map.service` will not find it.

If the service cannot see X11, import the session environment and restart it:

```bash
systemctl --user import-environment DISPLAY XAUTHORITY XDG_SESSION_TYPE
systemctl --user restart xmouse-boundary-map.service
```

Use `RUST_LOG=debug` for warp logs:

```bash
RUST_LOG=debug cargo run --
```

Mapping is disabled while dragging by default. Use `--map-drag` to allow drag-time warps.

## Config

Use `example-config.toml` for explicit monitor edges:

```bash
cargo run -- --config example-config.toml
```

Set `from` and `to` to the monitor names printed by `--list-monitors`.

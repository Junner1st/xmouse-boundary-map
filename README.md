# xmouse-boundary-map

X11 monitor boundary mouse mapper for GNOME Shell/X11. It reads active monitors from XRandR, listens to XInput2 raw motion, and uses `XWarpPointer` when the cursor crosses a configured left/right monitor edge.

The default behavior auto-maps horizontally adjacent monitors by relative resolution height:

```text
y_to = (y_from - from.y) / from.height * to.height + to.y
```

## Build

```bash
cargo build --release
```

## Check Monitor Names

```bash
cargo run -- --list-monitors
```

On this machine the current XRandR layout was detected as:

```text
HDMI-0: 1920x1080+0+360
DP-0: 2560x1440+1920+0
```

Because those monitors are horizontally adjacent, running without a config will auto-create both mappings.

The XInput2 raw motion path also handles the large-to-small monitor case where X11 blocks the pointer at the large monitor edge before it can enter the smaller monitor's vertical range. When the cursor is stuck on the edge and raw `dx` still points outward, the daemon maps the current height ratio and warps into the smaller monitor.

## Run

```bash
cargo run --
```

Use `RUST_LOG=debug` for warp logs:

```bash
RUST_LOG=debug cargo run --
```

By default, mapping is disabled while any mouse button is held to avoid GNOME/Mutter drag jumps. Use `--map-drag` to allow drag-time warps.

If XInput2 raw motion cannot be enabled, the daemon falls back to pointer polling. That fallback can handle ordinary crossings, but cannot fix blocked large-to-small edge movement.

## Config

Use `example-config.toml` if you want explicit monitor edges:

```bash
cargo run -- --config example-config.toml
```

For the detected layout above, an explicit config would use `HDMI-0` and `DP-0` instead of the sample `DP-1`/`HDMI-1` names.

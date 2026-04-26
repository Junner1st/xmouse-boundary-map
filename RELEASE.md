# Release

## Build a Release Binary

```bash
cargo build --release
```

The binary is written to:

```text
target/release/xmouse-boundary-map
```

## Build a Debian Package

Install `cargo-deb` once:

```bash
cargo install cargo-deb
```

Build the package:

```bash
cargo deb
```

The `.deb` file is written under:

```text
target/debian/
```

Install it locally:

```bash
sudo apt install ./target/debian/xmouse-boundary-map_0.1.0_amd64.deb
```

## Run After Installing

Start it manually:

```bash
xmouse-boundary-map
```

Or enable the packaged user service:

```bash
systemctl --user enable --now xmouse-boundary-map.service
```

On some GNOME/X11 setups, user services need the display environment imported first:

```bash
systemctl --user import-environment DISPLAY XAUTHORITY XDG_SESSION_TYPE
systemctl --user restart xmouse-boundary-map.service
```

## Release Checklist

1. Update `version` in `Cargo.toml`.
2. Run `cargo test`.
3. Run `cargo build --release`.
4. Run `cargo deb`.
5. Test install the generated `.deb`.

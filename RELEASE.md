# Release

## Build Debian Package

```bash
cargo install cargo-deb
cargo deb
```

Output:

```text
target/debian/xmouse-boundary-map_<version>-1_amd64.deb
```

## Install Locally

```bash
sudo apt install ./target/debian/xmouse-boundary-map_<version>-1_amd64.deb
```

## Run

```bash
xmouse-boundary-map
```

Or enable the user service:

```bash
systemctl --user enable --now xmouse-boundary-map.service
```

If the service cannot see X11:

```bash
systemctl --user import-environment DISPLAY XAUTHORITY XDG_SESSION_TYPE
systemctl --user restart xmouse-boundary-map.service
```

## Release Checklist

1. Update `version` in `Cargo.toml`.
2. Run `cargo test`.
3. Run `cargo deb`.
4. Test install the generated `.deb`.

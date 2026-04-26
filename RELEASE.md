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

When removing the package, the Debian maintainer scripts stop and disable the
active systemd user service before the unit file is removed:

```bash
sudo apt remove xmouse-boundary-map
```

## Run

```bash
xmouse-boundary-map
```

Or enable the systemd user service:

```bash
systemctl --user daemon-reload
systemctl --user enable --now xmouse-boundary-map.service
```

This is a user service installed at `/usr/lib/systemd/user/xmouse-boundary-map.service`,
so inspect it with `systemctl --user`, not system-wide `systemctl`:

```bash
systemctl --user status xmouse-boundary-map.service
journalctl --user -u xmouse-boundary-map.service -f
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
5. Test `sudo apt remove xmouse-boundary-map` stops the user service.

#!/bin/sh
# Build a local .deb from the current source tree.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
export PATH="${HOME}/.cargo/bin:${PATH}"
unset CARGO_TARGET_DIR

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
arch=$(dpkg --print-architecture)
if [ -z "$version" ] || [ -z "$arch" ]; then
    echo "strokelet: cannot read the package version or architecture" >&2
    exit 1
fi

if ! cargo build --release --locked --offline; then
    cargo build --release --locked
fi
bin="$root/target/release/strokelet"

shlib=""
if command -v dpkg-shlibdeps >/dev/null 2>&1; then
    shlib=$(dpkg-shlibdeps -O "$bin" 2>/dev/null | sed -n 's/^shlibs:Depends=//p' || true)
fi
if [ -z "$shlib" ]; then
    shlib="libc6, libevdev2, libudev1"
fi
depends="$shlib, gjs, gir1.2-adw-1, gir1.2-gtk-4.0, gnome-shell"

stage=$(mktemp -d)
cleanup() {
    rm -rf "$stage"
}
trap cleanup EXIT

mkdir -p \
    "$stage/DEBIAN" \
    "$stage/usr/bin" \
    "$stage/usr/lib/strokelet" \
    "$stage/usr/lib/systemd/user" \
    "$stage/usr/lib/udev/rules.d" \
    "$stage/usr/share/applications" \
    "$stage/usr/share/doc/strokelet" \
    "$stage/usr/share/gnome-shell/extensions/strokelet@local" \
    "$stage/usr/share/strokelet/settings" \
    "$stage/etc/xdg/autostart"

install -m 0755 "$bin" "$stage/usr/bin/strokelet"
install -m 0755 packaging/strokelet-mkruntime "$stage/usr/lib/strokelet/strokelet-mkruntime"
install -m 0755 packaging/enable-extension.sh "$stage/usr/lib/strokelet/enable-extension.sh"
install -m 0644 packaging/strokelet.service "$stage/usr/lib/systemd/user/strokelet.service"
install -m 0644 packaging/70-strokelet-uinput.rules "$stage/usr/lib/udev/rules.d/70-strokelet-uinput.rules"
install -m 0644 packaging/strokelet-settings.desktop "$stage/usr/share/applications/org.strokelet.Settings.desktop"
install -m 0644 packaging/strokelet-enable.desktop "$stage/etc/xdg/autostart/strokelet-enable.desktop"
install -m 0644 packaging/copyright "$stage/usr/share/doc/strokelet/copyright"
install -m 0644 settings/app.js "$stage/usr/share/strokelet/settings/app.js"
cp -a extension/strokelet@local/. "$stage/usr/share/gnome-shell/extensions/strokelet@local/"
rm -f "$stage/usr/share/gnome-shell/extensions/strokelet@local/STROKELET_OWNED"
chmod 0644 "$stage/usr/share/gnome-shell/extensions/strokelet@local/"*
find "$stage" -type d -exec chmod 0755 {} +

install -m 0755 packaging/postinst "$stage/DEBIAN/postinst"
install -m 0755 packaging/prerm "$stage/DEBIAN/prerm"
install -m 0755 packaging/postrm "$stage/DEBIAN/postrm"

cat > "$stage/DEBIAN/control" <<EOF
Package: strokelet
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Maintainer: xwpwx <xwpwx@users.noreply.github.com>
Depends: $depends
Homepage: https://github.com/xwpwx/strokelet
Description: mouse stroke shortcuts for GNOME
 Strokelet turns a mouse stroke or a two-button chord into a shortcut
 on GNOME Shell 45 through 50 Wayland sessions, and draws the stroke
 without taking focus.
EOF

mkdir -p "$root/dist"
out="$root/dist/strokelet_${version}_${arch}.deb"
rm -f "$out"
dpkg-deb --root-owner-group --build "$stage" "$out"
echo "strokelet: wrote $out"

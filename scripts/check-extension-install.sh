#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
home=$(mktemp -d)
export HOME="$home"
export XDG_DATA_HOME="$home/data"

"$root/scripts/install-extension.sh" >/dev/null
test -f "$XDG_DATA_HOME/gnome-shell/extensions/strokelet@local/metadata.json"
test "$(cat "$XDG_DATA_HOME/gnome-shell/extensions/strokelet@local/STROKELET_OWNED")" = "strokelet@local"

foreign="$home/foreign"
mkdir -p "$foreign/gnome-shell/extensions/strokelet@local"
printf '%s\n' '{"uuid":"strokelet@local"}' > "$foreign/gnome-shell/extensions/strokelet@local/metadata.json"
XDG_DATA_HOME="$foreign" "$root/scripts/install-extension.sh" >/dev/null && exit 1 || true
test -f "$foreign/gnome-shell/extensions/strokelet@local/metadata.json"
test ! -f "$foreign/gnome-shell/extensions/strokelet@local/STROKELET_OWNED"

XDG_DATA_HOME="$foreign" "$root/scripts/remove-extension.sh" >/dev/null && exit 1 || true
test -d "$foreign/gnome-shell/extensions/strokelet@local"

"$root/scripts/remove-extension.sh" >/dev/null
test ! -d "$XDG_DATA_HOME/gnome-shell/extensions/strokelet@local"

if XDG_SESSION_TYPE=x11 "$root/scripts/run-demo.sh" --device /dev/input/event0 --uid 1 --session 1 >/dev/null 2>&1; then
    echo "strokelet: run-demo accepted a non-Wayland session" >&2
    exit 1
fi

rm -rf "$home" "$foreign"
echo "strokelet: extension install checks ok"

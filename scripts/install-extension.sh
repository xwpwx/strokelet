#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
src="$root/extension/strokelet@local"
dest="${XDG_DATA_HOME:-$HOME/.local/share}/gnome-shell/extensions/strokelet@local"
marker="STROKELET_OWNED"

if [ ! -f "$src/metadata.json" ] || [ ! -f "$src/$marker" ]; then
    echo "strokelet: extension sources are incomplete" >&2
    exit 1
fi

if [ -e "$dest" ]; then
    if [ ! -f "$dest/$marker" ] || [ "$(cat "$dest/$marker")" != "strokelet@local" ]; then
        echo "strokelet: $dest exists and is not this project's extension; refusing to replace it" >&2
        exit 1
    fi
    rm -rf "$dest"
fi

mkdir -p "$(dirname "$dest")"
cp -a "$src" "$dest"
mkdir -p "$dest/settings"
cp -a "$root/settings/app.js" "$dest/settings/app.js"
echo "strokelet: installed to $dest"
echo "strokelet: first install may need a logout and login before GNOME Shell loads it. This script does not log you out."
echo "strokelet: enable it with: gnome-extensions enable strokelet@local"

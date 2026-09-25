#!/bin/sh
set -eu

dest="${XDG_DATA_HOME:-$HOME/.local/share}/gnome-shell/extensions/strokelet@local"
marker="STROKELET_OWNED"

if [ ! -d "$dest" ]; then
    echo "strokelet: $dest is not installed"
    exit 0
fi

if [ ! -f "$dest/$marker" ] || [ "$(cat "$dest/$marker")" != "strokelet@local" ]; then
    echo "strokelet: $dest is not this project's extension; refusing to remove it" >&2
    exit 1
fi

rm -rf "$dest"
echo "strokelet: removed $dest"
echo "strokelet: user data outside this extension directory was left in place"

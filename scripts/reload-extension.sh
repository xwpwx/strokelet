#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
"$root/scripts/install-extension.sh"

call() {
    gdbus call --session \
        --dest org.gnome.Shell \
        --object-path /org/gnome/Shell \
        --method "org.gnome.Shell.Extensions.$1" strokelet@local
}

call DisableExtension >/dev/null
call EnableExtension >/dev/null
echo "strokelet: reloaded strokelet@local"
echo "strokelet: edits to extension.js itself still need one logout. Other extension files reload with this script."

#!/bin/sh
# Enable the Shell extension and the user service. Safe to run more than once.
set -eu
gnome-extensions enable strokelet@local >/dev/null 2>&1 || true
systemctl --user enable --now strokelet.service >/dev/null 2>&1 || true

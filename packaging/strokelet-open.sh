#!/bin/sh
# Bring the extension and the background service back, then open settings.
gnome-extensions enable strokelet@local >/dev/null 2>&1 || true
systemctl --user start strokelet.service >/dev/null 2>&1 || true
exec strokelet settings

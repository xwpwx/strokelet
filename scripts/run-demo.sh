#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
device=""
uid=""
session=""
timeout=120

while [ $# -gt 0 ]; do
    case "$1" in
        --device) device=$2; shift 2 ;;
        --uid) uid=$2; shift 2 ;;
        --session) session=$2; shift 2 ;;
        --timeout-seconds) timeout=$2; shift 2 ;;
        *) echo "strokelet: unknown argument $1" >&2; exit 1 ;;
    esac
done

if [ -z "$device" ] || [ -z "$uid" ] || [ -z "$session" ]; then
    echo "usage: run-demo.sh --device PATH --uid UID --session SESSION [--timeout-seconds 120]" >&2
    exit 1
fi

if [ "${XDG_SESSION_TYPE:-}" != "wayland" ]; then
    echo "strokelet: this demo targets a Wayland session" >&2
    exit 1
fi

runtime="/run/strokelet/$uid"
if [ ! -d "$runtime" ]; then
    echo "strokelet: create the runtime directory once, as root, then rerun:" >&2
    echo "  sudo install -d -o root -g \"$(id -gn)\" -m 0770 $runtime" >&2
    exit 1
fi

exec "$root/target/debug/strokelet" run \
    --device "$device" \
    --uid "$uid" \
    --session "$session" \
    --timeout-seconds "$timeout" \
    --inject-copy

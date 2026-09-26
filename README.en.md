# Strokelet

[中文](README.md)

On a GNOME Shell 45–50 Wayland desktop, a stroke or two mouse buttons sends a shortcut and draws a trail that does not take focus. A normal click still works.

This is a small tool made for fun. The author was used to mouse gestures, and after moving to Ubuntu 26.04 — where the desktop session is Wayland only — built it with vibe coding.

## Install

Build the package from this repo, then install it:

```sh
scripts/build-deb.sh
sudo apt install ./dist/strokelet_0.1.2_amd64.deb
```

Log out once and back in. After that, Strokelet takes the mouse at each login. Open Settings from the mark in the top bar, or launch Strokelet from the app list, to change rules.

If exactly one usable relative mouse is present, it is selected automatically. With more than one, open Strokelet, choose the mouse under “Use this one”, and save. You can also write the path in `~/.config/strokelet/device`.

## What it is for

It is for keeping a hand on the mouse instead of reaching back to the keyboard for a few shortcuts. The action happens in the current window. Focus stays where it is.

- Select some text, hold the right button, stroke straight up, and release. That sends Ctrl+C once. A plain right click still opens the menu. This is the default when there is no config file.
- Bind a stroke, a “hold one mouse button, then press another”, or a “hold one button and roll the wheel one notch” to a shortcut. Examples: open a terminal, paste, close a window. Up to 16 rules.
- Show a name you chose, such as “Copy”, after the shortcut is sent. By default it sits centered near the bottom of the screen. Position and line width are under Display in Settings. An empty name shows nothing.
- A terminal’s copy shortcut can be the chord that terminal actually uses. In Ptyxis that is Ctrl+Shift+C.

What gets sent is the recorded shortcut. The current app handles it as usual: Ctrl+C copies when text is selected, and otherwise it is just that key chord.

## Where it runs

Two things have to be true: the session is Wayland, and the GNOME Shell major version is 45 through 50.

The extension declares `"shell-version": ["45", "46", "47", "48", "49", "50"]`. Those releases use the module imports introduced in Shell 45, and the trail is Cairo on `St.DrawingArea`. The official porting notes for 46 through 50 do not change the extension entry point. Minor versions of the same major match, so 50.0, 50.1, and 50.2 are all Shell 50. This machine has been used on Ubuntu 26.04.1 with Shell 50.1. Shell 45–49 follow that API, but have not been logged into here.

| System | Desktop | Wayland | This program |
| --- | --- | --- | --- |
| Ubuntu 22.04 | GNOME Shell 42 | A Wayland session is available | No. The module style predates Shell 45 |
| Ubuntu 24.04 | GNOME Shell 46 | Wayland is the default; the login screen can still offer Xorg | Declared. Shell 46 has not been logged into here |
| Ubuntu 24.10 | GNOME Shell 47 | Wayland is available | Declared. Shell 47 has not been logged into here |
| Ubuntu 25.04 | GNOME Shell 48 | Wayland is available | Declared. Shell 48 has not been logged into here |
| Ubuntu 25.10 | GNOME Shell 49 | The desktop session is Wayland only | Declared. Shell 49 has not been logged into here |
| Ubuntu 26.04 and 26.04.x | GNOME Shell 50.x | The desktop session is Wayland only | Yes. 50.1 has been run here |
| The next development series (Shell 51 in current archives) | GNOME Shell 51 | Wayland | Outside 45–50. Check the API before declaring it |

Mouse detection uses Linux evdev and logind, not a vendor version number. Other distributions come down to the same two facts: a Wayland GNOME session, and a Shell major from 45 to 50. The versions below are from package archives as of 2026-09. A rolling release that moves to Shell 51 leaves this range. None of these systems have been run here.

| Distribution | Current GNOME Shell | This program |
| --- | --- | --- |
| Fedora 43 | 49.x | Declared. Not logged into here |
| Fedora 44 | 50.x | Declared. The Workstation GNOME session is Wayland |
| Fedora 45, Rawhide | 51 | Outside 45–50 |
| Debian 12 | 43 | Older than Shell 45 |
| Debian 13 | 48.x | Declared. Not logged into here |
| Debian testing / unstable (2026-09) | 50.x | Declared |
| Debian experimental | 51 | Outside 45–50 |
| Arch Linux extra | 50.x | Declared. The GNOME session is Wayland |
| Arch gnome-unstable | 51 | Outside 45–50 |
| openSUSE Tumbleweed | 50.x | Declared |
| openSUSE Leap 16.0 | 48.x | Declared. Not logged into here |
| openSUSE Leap 15.6 | 45.x | Declared. Not logged into here |

KDE Plasma, Cinnamon, Xfce, and COSMIC do not load this GNOME extension. Linux Mint’s default desktop is Cinnamon, not this Shell.

The launcher exits unless `XDG_SESSION_TYPE` is `wayland`. From Shell 50 on, GNOME itself no longer offers an Xorg desktop session.

Actions target native Wayland windows, such as GNOME Text Editor. XWayland windows are only a fallback. The mouse must be a relative-motion device. Pass it with `--device`. Only one mouse is taken at a time. The demo is a foreground process. A shortcut is sent only while that process is running, the extension is connected, the session is unlocked, and the panel is not paused.

## Development checks

```sh
make init
make setup
make test
make check
```

Extension script checks:

```sh
gjs -m scripts/check-gjs-socket.js
scripts/check-extension-install.sh
```

## Starting it from two terminals

A hands-on check can follow this order:

1. Confirm the session is Wayland.
2. As root, create `/run/strokelet/<uid>` once. Owner root, group the user’s primary group, mode 0770.
3. Run `scripts/install-extension.sh`, then `gnome-extensions enable strokelet@local`. The first install may need a logout and login. The script does not log you out.
4. In one terminal, run `cargo build && ./target/debug/strokelet list-devices`. Note an accepted relative mouse path, and the session id from `loginctl list-sessions`.
5. Keep the GNOME session in the other terminal. Run:

```sh
scripts/run-demo.sh --device /dev/input/by-id/your-mouse --uid "$(id -u)" --session SESSION
```

It stays in the foreground for 120 seconds by default. `--timeout-seconds` extends that. That script turns shortcut injection on. A direct `strokelet run` leaves it off unless you pass `--inject-copy`. `--passthrough-only` only forwards input.

To edit rules, open another terminal and run `./target/debug/strokelet settings`. Click Add, or click a row. A rule is one of: draw a stroke, a mouse-button chord (the first button held is the start; left, right, middle, or a side button, then a second button), or hold a button and roll the wheel one notch. Then click Record next to the shortcut. The on-screen name can be something like “Copy”; leave it empty to show no text. Click Done, return to the list, and click Save. The stroke trigger can be left, right, middle, or a side button. While the demo is running, hold the stroke trigger and draw; you do not need to pause first. If the demo is not running, recording a stroke grabs the mouse briefly. Save writes `~/.config/strokelet/gestures.json`. A running demo reloads that file. With no config file, the default is still a straight stroke up on the right button, sending Ctrl+C. An older rule that only stored a direction is still treated as a straight line. An older button chord with no start button still means “hold whatever the stroke trigger was in that file”.

Pause from the mark in the top bar. Stop by waiting for the timeout, or press Ctrl+C in the terminal that is running it. Uninstall with `scripts/remove-extension.sh`. It only removes an extension directory marked as this project.

## Debugging

You do not need to log out for every change.

- After Rust changes, `cargo build` again and run `scripts/run-demo.sh`. Ctrl+C releases the mouse and removes the socket.
- After edits to `overlay.js`, `indicator.js`, or `transport.js`, run `scripts/reload-extension.sh`. It copies the extension into the user directory and has the current Shell turn it off and on.
- A logout is needed the first time you install, or when `extension.js` itself changes. GNOME 50 cannot hot-reload the extension entry file on Wayland.

Default recognition: movement within 12 counts is still a click, a stroke must be at least 80 counts, the longest hold is 2500 ms, and there are at most 16 rules. All of these can be changed under Recognition in Settings. The trail appears after about 12 screen pixels of movement.

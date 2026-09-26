import Cairo from 'gi://cairo';
import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import St from 'gi://St';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

export const StrokeletIndicator = GObject.registerClass({
    GTypeName: `StrokeletIndicator${Date.now()}`,
}, class extends PanelMenu.Button {
    _init(overlay) {
        super._init(0.0, 'Strokelet');
        this._overlay = overlay;
        this.paused = false;
        this._connected = false;
        this._mark = new St.DrawingArea({
            width: 16,
            height: 16,
            y_align: Clutter.ActorAlign.CENTER,
        });
        this._mark.connect('repaint', area => this._paintMark(area));
        this._mark.connect('style-changed', () => this._mark.queue_repaint());
        this.add_child(this._mark);
        this._status = new PopupMenu.PopupMenuItem('未连接', {reactive: false});
        this.menu.addMenuItem(this._status);
        this.menu.addAction('设置', () => this._openSettings());
        this._pauseItem = this.menu.addAction('暂停', () => this._togglePause());
        this.menu.addAction('退出', () => this._quit());
    }

    _togglePause() {
        this.paused = !this.paused;
        this._pauseItem.label.text = this.paused ? '启动' : '暂停';
        this._mark.queue_repaint();
        this._overlay.endActive();
    }

    setConnected(connected) {
        this._connected = connected;
        this._status.label.text = connected ? '已连接' : '未连接';
        this._mark.queue_repaint();
    }

    _quit() {
        this.menu.close();
        spawn(['systemctl', '--user', 'stop', 'strokelet.service']);
        spawn(['pkill', '-f', 'settings/app.js']);
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
            spawn(['gnome-extensions', 'disable', 'strokelet@local']);
            return GLib.SOURCE_REMOVE;
        });
    }

    _paintMark(area) {
        const cr = area.get_context();
        const [red, green, blue, alpha] = foreground(area);
        const faded = !this._connected || this.paused;
        cr.setSourceRGBA(red, green, blue, alpha * (faded ? 0.4 : 1));
        cr.setLineWidth(1.7);
        cr.setLineCap(Cairo.LineCap.ROUND);
        cr.moveTo(6.4, 10.4);
        cr.curveTo(6.0, 6.6, 8.8, 4.4, 12.2, 2.7);
        cr.stroke();
        cr.arc(6.1, 12.3, 2.05, 0, Math.PI * 2);
        cr.fill();
        cr.$dispose();
    }

    _openSettings() {
        this.menu.close();
        try {
            const command = strokeletCommand();
            const bundled = bundledSettings();
            if (bundled) {
                const launcher = Gio.SubprocessLauncher.new(Gio.SubprocessFlags.NONE);
                launcher.setenv('STROKELET_BIN', command, true);
                launcher.spawnv(['gjs', '-m', bundled]);
                return;
            }
            Gio.Subprocess.new([command, 'settings'], Gio.SubprocessFlags.NONE);
        } catch (error) {
            log(`strokelet: cannot open settings: ${error}`);
        }
    }
});

function spawn(argv) {
    try {
        Gio.Subprocess.new(argv, Gio.SubprocessFlags.NONE);
    } catch (error) {
        log(`strokelet: ${argv.join(' ')} failed: ${error}`);
    }
}

function strokeletCommand() {
    const local = GLib.build_filenamev([GLib.get_home_dir(), '.local', 'bin', 'strokelet']);
    if (Gio.File.new_for_path(local).query_exists(null))
        return local;
    if (Gio.File.new_for_path('/usr/bin/strokelet').query_exists(null))
        return '/usr/bin/strokelet';
    return 'strokelet';
}

function bundledSettings() {
    const extension = Extension.lookupByUUID('strokelet@local');
    if (!extension?.path)
        return null;
    const path = GLib.build_filenamev([extension.path, 'settings', 'app.js']);
    return Gio.File.new_for_path(path).query_exists(null) ? path : null;
}

function foreground(widget) {
    const color = widget.get_theme_node().get_foreground_color();
    if (typeof color.get_red === 'function')
        return [color.get_red(), color.get_green(), color.get_blue(), color.get_alpha()];
    const scale = Math.max(color.red, color.green, color.blue, color.alpha ?? 0) > 1 ? 255 : 1;
    return [
        color.red / scale,
        color.green / scale,
        color.blue / scale,
        (color.alpha ?? scale) / scale,
    ];
}

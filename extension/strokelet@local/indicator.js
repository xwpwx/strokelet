import Cairo from 'gi://cairo';
import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import St from 'gi://St';

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
        this._pauseItem = new PopupMenu.PopupSwitchMenuItem('暂停', false);
        this._pauseItem.connect('toggled', item => {
            this.paused = item.state;
            this._mark.queue_repaint();
            this._overlay.endActive();
        });
        this.menu.addMenuItem(this._pauseItem);
        this.menu.addAction('轨迹自检（5 秒）', () => this._selfTestAfterClose());
    }

    setConnected(connected) {
        this._connected = connected;
        this._status.label.text = connected ? '已连接' : '未连接';
        this._mark.queue_repaint();
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

    _selfTestAfterClose() {
        const id = this.menu.connect('open-state-changed', (_menu, open) => {
            if (open)
                return;
            this.menu.disconnect(id);
            GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
                this._overlay.selfTest();
                return GLib.SOURCE_REMOVE;
            });
        });
        this.menu.close();
    }
});

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

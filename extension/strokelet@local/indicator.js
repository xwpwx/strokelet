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
        this.add_child(new St.Label({text: '划'}));
        this._status = new PopupMenu.PopupMenuItem('未连接', {reactive: false});
        this.menu.addMenuItem(this._status);
        this._pauseItem = new PopupMenu.PopupSwitchMenuItem('暂停', false);
        this._pauseItem.connect('toggled', item => {
            this.paused = item.state;
            this._overlay.endActive();
        });
        this.menu.addMenuItem(this._pauseItem);
        this.menu.addAction('轨迹自检（5 秒）', () => this._selfTestAfterClose());
    }

    setConnected(connected) {
        this._status.label.text = connected ? '已连接' : '未连接';
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

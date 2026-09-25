import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export default class StrokeletExtension extends Extension {
    enable() {
        this._closed = false;
        this._generation = (this._generation ?? 0) + 1;
        const generation = this._generation;
        this._load(generation).catch(error => log(`strokelet: enable failed: ${error}`));
    }

    async _load(generation) {
        const stamp = Date.now();
        const [{GestureOverlay}, {StrokeletIndicator}, {DemoTransport}] = await Promise.all([
            import(`./overlay.js?stamp=${stamp}`),
            import(`./indicator.js?stamp=${stamp}`),
            import(`./transport.js?stamp=${stamp}`),
        ]);
        if (this._closed || this._generation !== generation)
            return;
        this._overlay = new GestureOverlay();
        this._indicator = new StrokeletIndicator(this._overlay);
        Main.panel.addToStatusArea('strokelet', this._indicator);
        this._transport = new DemoTransport(this._socketPath(), message => {
            this._onMessage(message);
        }, () => {
            this._overlay.endActive();
            this._indicator.setConnected(false);
        });
        this._transport.connect();
    }

    disable() {
        this._closed = true;
        if (this._stateSource) {
            GLib.source_remove(this._stateSource);
            this._stateSource = 0;
        }
        this._transport?.disconnect();
        this._transport = null;
        this._overlay?.destroy();
        this._overlay = null;
        this._indicator?.destroy();
        this._indicator = null;
    }

    _socketPath() {
        const uid = new Gio.Credentials().get_unix_user();
        return `/run/strokelet/${uid}/strokelet.sock`;
    }

    _onMessage(message) {
        if (message.type === 'hello') {
            this._indicator.setConnected(true);
            this._transport.send({type: 'ready', version: 1});
            this._lastPaused = this._indicator.paused;
            this._transport.send({type: 'pause', paused: this._lastPaused});
            this._sendState();
            this._stateSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
                this._sendState();
                return GLib.SOURCE_CONTINUE;
            });
            return;
        }
        if (message.type === 'ping') {
            this._transport.send({type: 'pong'});
            return;
        }
        if (message.type === 'begin') {
            this._overlay.begin(message.id);
            return;
        }
        if (message.type === 'end' || message.type === 'cancel') {
            this._overlay.end(message.id);
        }
    }

    _sendState() {
        if (this._lastPaused !== this._indicator.paused) {
            this._lastPaused = this._indicator.paused;
            this._transport.send({type: 'pause', paused: this._lastPaused});
        }
        const locked = Main.sessionMode.isLocked;
        this._transport.send({
            type: 'state',
            active: !locked && Main.sessionMode.currentMode !== 'unlock-dialog',
            locked,
            modifiers: this._heldModifiers(),
        });
    }

    _heldModifiers() {
        const [,, mods] = global.get_pointer();
        const names = [];
        if (mods & Clutter.ModifierType.CONTROL_MASK)
            names.push('Control');
        if (mods & Clutter.ModifierType.SHIFT_MASK)
            names.push('Shift');
        if (mods & Clutter.ModifierType.MOD1_MASK)
            names.push('Alt');
        if (mods & Clutter.ModifierType.SUPER_MASK)
            names.push('Super');
        return names;
    }
}

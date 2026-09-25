import GLib from 'gi://GLib';
import St from 'gi://St';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const SAMPLE_MS = 16;
const MAX_POINTS = 512;
const SHOW_AFTER_PX = 12;
const LINE_WIDTH = 3;

export class GestureOverlay {
    constructor() {
        const stage = global.stage;
        this._area = new St.DrawingArea({
            reactive: false,
            can_focus: false,
            width: stage.width,
            height: stage.height,
        });
        this._area.connect('repaint', area => this._repaint(area));
        Main.uiGroup.add_child(this._area);
        Main.uiGroup.set_child_above_sibling(this._area, null);
        this._stageId = stage.connect('notify::width', () => this._resize());
        this._stageHeightId = stage.connect('notify::height', () => this._resize());
        this._points = [];
        this._activeId = null;
        this._visible = false;
        this._sampleId = 0;
        this._selfTest = false;
    }

    begin(id) {
        if (this._activeId === id)
            return;
        this._stopSampling();
        this._points = [];
        this._visible = false;
        this._activeId = id;
        this._selfTest = false;
        this._startSampling();
    }

    end(id) {
        if (this._activeId !== id)
            return;
        this.endActive();
    }

    endActive() {
        this._stopSampling();
        this._activeId = null;
        this._selfTest = false;
        this._points = [];
        this._visible = false;
        this._area.queue_repaint();
    }

    selfTest() {
        this.begin('self-test');
        this._selfTest = true;
        this._selfTestId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 5000, () => {
            this._selfTestId = 0;
            if (this._selfTest)
                this.end('self-test');
            return GLib.SOURCE_REMOVE;
        });
    }

    destroy() {
        this._stopSampling();
        if (this._selfTestId)
            GLib.source_remove(this._selfTestId);
        const stage = global.stage;
        if (this._stageId)
            stage.disconnect(this._stageId);
        if (this._stageHeightId)
            stage.disconnect(this._stageHeightId);
        this._area.destroy();
        this._area = null;
    }

    _resize() {
        if (!this._area)
            return;
        this._area.set_size(global.stage.width, global.stage.height);
    }

    _startSampling() {
        this._sampleId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, SAMPLE_MS, () => {
            this._sample();
            return GLib.SOURCE_CONTINUE;
        });
    }

    _stopSampling() {
        if (this._sampleId) {
            GLib.source_remove(this._sampleId);
            this._sampleId = 0;
        }
    }

    _sample() {
        const [stageX, stageY] = global.get_pointer();
        const [ok, x, y] = this._area.transform_stage_point(stageX, stageY);
        if (!ok)
            return;
        this._points.push([x, y]);
        if (this._points.length > MAX_POINTS)
            this._points.shift();
        if (!this._visible && this._points.length > 1) {
            const [x0, y0] = this._points[0];
            const dx = x - x0;
            const dy = y - y0;
            if (Math.hypot(dx, dy) >= SHOW_AFTER_PX)
                this._visible = true;
        }
        if (this._visible)
            this._area.queue_repaint();
    }

    _repaint(area) {
        if (!this._visible || this._points.length < 2)
            return;
        const cr = area.get_context();
        cr.setSourceRGBA(0.2, 0.6, 1.0, 0.9);
        cr.setLineWidth(LINE_WIDTH);
        const [x0, y0] = this._points[0];
        cr.moveTo(x0, y0);
        for (let i = 1; i < this._points.length; i++)
            cr.lineTo(this._points[i][0], this._points[i][1]);
        cr.stroke();
        cr.$dispose();
    }
}

import Cairo from 'gi://cairo';
import GLib from 'gi://GLib';
import St from 'gi://St';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import {preferredSize, readPointer} from './shell.js';

const SAMPLE_MS = 16;
const MAX_POINTS = 512;
const SHOW_AFTER_PX = 12;
const DEFAULT_LINE_WIDTH = 6;
const NAME_BOTTOM_GAP = 104;

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
        this._nameId = 0;
        this._name = new St.Label({
            reactive: false,
            can_focus: false,
            visible: false,
            style: 'font-size: 28px; font-weight: bold; color: white; background-color: rgba(0, 0, 0, 0.72); padding: 8px 14px; border-radius: 10px;',
        });
        Main.uiGroup.add_child(this._name);
        Main.uiGroup.set_child_above_sibling(this._name, this._area);
    }

    finish(message) {
        if (this._activeId !== message.id)
            return;
        this.end(message.id);
        if (message.type === 'end' && typeof message.name === 'string' && message.name.length > 0)
            this.showName(message.name);
    }

    showName(text) {
        if (!this._name || !text)
            return;
        this._hideName();
        this._name.text = text;
        this._name.show();
        const {x: pointerX, y: pointerY} = readPointer();
        const fallbackWidth = [...text].length * 32 + 28;
        const fallbackHeight = 52;
        const {width, height} = preferredSize(this._name, fallbackWidth, fallbackHeight);
        const place = readAppearance().namePlace === 'pointer'
            ? placeNearPointer(pointerX, pointerY, width, height)
            : placeBottomCenter(pointerX, pointerY, width, height);
        this._name.set_position(place.x, place.y);
        this._nameId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1200, () => {
            this._nameId = 0;
            this._name?.hide();
            return GLib.SOURCE_REMOVE;
        });
    }

    begin(id) {
        if (this._activeId === id)
            return;
        this._stopSampling();
        this._points = [];
        this._visible = false;
        this._activeId = id;
        this._hideName();
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
        this._points = [];
        this._visible = false;
        this._hideName();
        this._area.queue_repaint();
    }

    _hideName() {
        if (this._nameId) {
            GLib.source_remove(this._nameId);
            this._nameId = 0;
        }
        this._name?.hide();
    }

    destroy() {
        this._stopSampling();
        this._hideName();
        this._name?.destroy();
        this._name = null;
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
        const {x: stageX, y: stageY} = readPointer();
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
        cr.setLineWidth(readAppearance().lineWidth);
        cr.setLineCap(Cairo.LineCap.ROUND);
        cr.setLineJoin(Cairo.LineJoin.ROUND);
        traceSmooth(cr, this._points);
        cr.stroke();
        cr.$dispose();
    }
}

function traceSmooth(cr, points) {
    const count = points.length;
    const at = index => points[Math.max(0, Math.min(count - 1, index))];
    cr.moveTo(points[0][0], points[0][1]);
    for (let i = 0; i < count - 1; i++) {
        const p0 = at(i - 1);
        const p1 = at(i);
        const p2 = at(i + 1);
        const p3 = at(i + 2);
        cr.curveTo(
            p1[0] + (p2[0] - p0[0]) / 6,
            p1[1] + (p2[1] - p0[1]) / 6,
            p2[0] - (p3[0] - p1[0]) / 6,
            p2[1] - (p3[1] - p1[1]) / 6,
            p2[0],
            p2[1],
        );
    }
}

function placeBottomCenter(pointerX, pointerY, width, height) {
    const monitor = monitorAt(pointerX, pointerY);
    const x = monitor.x + Math.round((monitor.width - width) / 2);
    const y = monitor.y + monitor.height - NAME_BOTTOM_GAP - height;
    return clampOnStage(x, y, width, height);
}

function placeNearPointer(pointerX, pointerY, width, height) {
    const margin = 12;
    return clampOnStage(pointerX + 18, pointerY - height - 16, width, height, margin);
}

function clampOnStage(x, y, width, height, margin = 12) {
    return {
        x: Math.round(Math.max(margin, Math.min(x, global.stage.width - width - margin))),
        y: Math.round(Math.max(margin, Math.min(y, global.stage.height - height - margin))),
    };
}

function monitorAt(x, y) {
    const layout = Main.layoutManager;
    return layout.findMonitorForPoint?.(x, y)
        || layout.primaryMonitor
        || {x: 0, y: 0, width: global.stage.width, height: global.stage.height};
}

function readAppearance() {
    const path = GLib.build_filenamev([
        GLib.get_user_config_dir(),
        'strokelet',
        'appearance.json',
    ]);
    try {
        const [ok, bytes] = GLib.file_get_contents(path);
        if (!ok)
            return {lineWidth: DEFAULT_LINE_WIDTH, namePlace: 'bottom'};
        const data = JSON.parse(new TextDecoder().decode(bytes));
        const width = Number(data.lineWidth);
        return {
            lineWidth: Number.isFinite(width)
                ? Math.min(20, Math.max(2, Math.round(width)))
                : DEFAULT_LINE_WIDTH,
            namePlace: data.namePlace === 'pointer' ? 'pointer' : 'bottom',
        };
    } catch {
        return {lineWidth: DEFAULT_LINE_WIDTH, namePlace: 'bottom'};
    }
}

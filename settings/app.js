import Adw from 'gi://Adw';
import Gdk from 'gi://Gdk?version=4.0';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk?version=4.0';

import {readLanguage, translate, translateDaemon, writeLanguage} from './strings.js';

let t = key => key;
let languageCode = 'zh';

const TRIGGERS = ['left', 'right', 'middle', 'forward', 'back'];
const NAME_PLACES = ['bottom', 'pointer'];
const STYLE = `
  .stroke-preview {
    background-color: alpha(@window_fg_color, 0.06);
    border-radius: 12px;
  }
  .kind-tag {
    padding: 2px 8px;
    border-radius: 999px;
    font-weight: 700;
    color: @accent_color;
    background-color: alpha(@accent_color, 0.16);
  }
`;

const app = new Adw.Application({application_id: 'org.strokelet.Settings'});

app.connect('activate', () => {
    installCss();
    openSettingsWindow(readLanguage());
});

function openSettingsWindow(language) {
    languageCode = language;
    t = (key, vars) => translate(language, key, vars);
    const config = readConfig();
    const window = new Adw.ApplicationWindow({
        application: app,
        title: 'Strokelet',
        default_width: 680,
        default_height: 560,
    });
    const toastOverlay = new Adw.ToastOverlay();
    const toolbar = new Adw.ToolbarView();
    const header = new Adw.HeaderBar();
    header.title_widget = new Adw.WindowTitle({
        title: 'Strokelet',
        subtitle: t('window.subtitle'),
    });
    const save = new Gtk.Button({label: t('action.save'), css_classes: ['suggested-action']});
    header.pack_end(save);
    toolbar.add_top_bar(header);
    const dirtyBanner = new Adw.Banner({
        title: t('dirty.title'),
        button_label: t('action.save'),
        revealed: false,
    });
    toolbar.add_top_bar(dirtyBanner);

    const page = new Adw.PreferencesPage();
    const languageGroup = new Adw.PreferencesGroup();
    const languageRow = new Adw.ComboRow({
        title: t('language.label'),
        model: modelOf(['中文', 'English']),
    });
    languageRow.selected = language === 'en' ? 1 : 0;
    languageGroup.add(languageRow);
    page.add(languageGroup);
    const mice = listMice();
    const savedMouse = readDevice();
    const mouseGroup = new Adw.PreferencesGroup({
        title: t('mouse.group'),
        description: mice.length > 1 ? t('mouse.many') : t('mouse.one'),
    });
    const mouseRow = new Adw.ComboRow({
        title: t('mouse.use'),
        model: modelOf(mice.length === 0 ? [t('mouse.none')] : mice.map(mouse => mouse.name)),
        sensitive: mice.length > 0,
    });
    const savedIndex = mice.findIndex(mouse => mouse.path === savedMouse);
    mouseRow.selected = savedIndex >= 0 ? savedIndex : 0;
    mouseGroup.add(mouseRow);
    page.add(mouseGroup);
    const selectedMouse = () => mice[mouseRow.selected]?.path || '';
    const triggerGroup = new Adw.PreferencesGroup({
        title: t('trigger.group'),
        description: t('trigger.description'),
    });
    const trigger = new Adw.ComboRow({
        title: t('trigger.label'),
        model: modelOf(TRIGGERS.map(name => buttonPhrase(name))),
    });
    trigger.selected = Math.max(0, TRIGGERS.indexOf(config.trigger));
    triggerGroup.add(trigger);
    page.add(triggerGroup);

    const appearance = readAppearance();
    const displayGroup = new Adw.PreferencesGroup({
        title: t('display.group'),
        description: t('display.description'),
    });
    const namePlaceRow = new Adw.ComboRow({
        title: t('display.namePlace'),
        model: modelOf([t('display.bottom'), t('display.pointer')]),
    });
    namePlaceRow.selected = Math.max(0, NAME_PLACES.indexOf(appearance.namePlace));
    const widthRow = new Adw.SpinRow({
        title: t('display.width'),
        subtitle: t('display.pixels'),
        adjustment: new Gtk.Adjustment({
            lower: 2,
            upper: 20,
            step_increment: 1,
            page_increment: 2,
            value: appearance.lineWidth,
        }),
        digits: 0,
    });
    displayGroup.add(namePlaceRow);
    displayGroup.add(widthRow);
    page.add(displayGroup);
    const selectedNamePlace = () => NAME_PLACES[namePlaceRow.selected] || 'bottom';
    const selectedLineWidth = () => Math.round(widthRow.get_value());

    const recognition = recognitionOf(config);
    const recognizeGroup = new Adw.PreferencesGroup({
        title: t('recognize.group'),
        description: t('recognize.description'),
    });
    const clickRow = spinRow(t('recognize.click'), t('recognize.clickHint'), recognition.clickSlop, 1, 400, 1);
    const lengthRow = spinRow(t('recognize.length'), t('recognize.lengthHint'), recognition.minLength, 20, 5000, 10);
    const durationRow = spinRow(t('recognize.duration'), t('recognize.durationHint'), recognition.maxDurationMs, 200, 60000, 100);
    const maxRulesRow = spinRow(t('recognize.maxRules'), t('recognize.rulesUnit'), recognition.maxRules, 1, 64, 1);
    recognizeGroup.add(clickRow);
    recognizeGroup.add(lengthRow);
    recognizeGroup.add(durationRow);
    recognizeGroup.add(maxRulesRow);
    page.add(recognizeGroup);
    const selectedRecognition = () => ({
        clickSlop: Math.round(clickRow.get_value()),
        minLength: Math.round(lengthRow.get_value()),
        maxDurationMs: Math.round(durationRow.get_value()),
        maxRules: Math.round(maxRulesRow.get_value()),
    });

    const rules = config.rules.map(rule => cloneRule(rule));
    const rulesGroup = new Adw.PreferencesGroup({title: t('rules.group')});
    const add = new Gtk.Button({label: t('action.add'), css_classes: ['flat'], tooltip_text: t('rules.addTooltip')});
    rulesGroup.set_header_suffix(add);
    const ruleRows = [];
    const triggerName = () => TRIGGERS[trigger.selected] || 'right';
    const snapshot = () => JSON.stringify({
        version: 1,
        trigger: triggerName(),
        rules,
        mouse: selectedMouse(),
        namePlace: selectedNamePlace(),
        lineWidth: selectedLineWidth(),
        recognition: selectedRecognition(),
    });
    let baseline = '';
    const updateDirty = () => {
        dirtyBanner.revealed = baseline !== '' && snapshot() !== baseline;
    };
    const refresh = () => {
        ruleRows.splice(0).forEach(row => rulesGroup.remove(row));
        rulesGroup.description = rules.length === 0
            ? t('rules.emptyHint')
            : t('rules.count', {count: rules.length, max: selectedRecognition().maxRules});
        if (rules.length === 0) {
            const empty = new Adw.ActionRow({
                title: t('rules.emptyTitle'),
                subtitle: t('rules.emptySubtitle'),
            });
            rulesGroup.add(empty);
            ruleRows.push(empty);
            return;
        }
        rules.forEach((rule, index) => {
            const row = new Adw.ActionRow({
                title: rowTitle(rule),
                subtitle: rowSubtitle(rule, triggerName()),
                activatable: true,
                subtitle_lines: 2,
            });
            const tag = new Gtk.Label({
                label: kindTag(rule),
                css_classes: ['caption', 'kind-tag'],
            });
            row.add_prefix(tag);
            const remove = new Gtk.Button({
                icon_name: 'user-trash-symbolic',
                valign: Gtk.Align.CENTER,
                css_classes: ['flat'],
                tooltip_text: t('rules.deleteTooltip'),
            });
            remove.connect('clicked', () => confirmDelete(window, rowTitle(rule), () => {
                rules.splice(index, 1);
                refresh();
            }));
            row.add_suffix(remove);
            row.add_suffix(new Gtk.Image({
                icon_name: 'go-next-symbolic',
                css_classes: ['dim-label'],
            }));
            row.connect('activated', () => editRule(window, rule, triggerName(), () => refresh()));
            rulesGroup.add(row);
            ruleRows.push(row);
        });
        updateDirty();
    };
    refresh();
    baseline = snapshot();
    page.add(rulesGroup);
    toolbar.set_content(page);
    toastOverlay.child = toolbar;
    window.content = toastOverlay;

    let shownTrigger = triggerName();
    add.connect('clicked', () => {
        if (rules.length >= selectedRecognition().maxRules) {
            toastOverlay.add_toast(new Adw.Toast({title: t('rules.max', {max: selectedRecognition().maxRules})}));
            return;
        }
        const rule = {};
        editRule(window, rule, triggerName(), () => {
            rules.push(rule);
            refresh();
        }, true);
    });
    mouseRow.connect('notify::selected', () => updateDirty());
    namePlaceRow.connect('notify::selected', () => updateDirty());
    widthRow.connect('notify::value', () => updateDirty());
    for (const row of [clickRow, lengthRow, durationRow, maxRulesRow])
        row.connect('notify::value', () => updateDirty());
    trigger.connect('notify::selected', () => {
        if (triggerName() === shownTrigger)
            return;
        shownTrigger = triggerName();
        refresh();
    });
    const writeConfig = () => {
        const text = `${JSON.stringify({
            version: 1,
            trigger: triggerName(),
            recognition: selectedRecognition(),
            rules,
        }, null, 2)}\n`;
        const problem = checkGestures(text);
        if (problem) {
            const dialog = new Adw.AlertDialog({
                heading: t('save.failed'),
                body: translateDaemon(language, problem),
            });
            dialog.add_response('ok', t('action.ok'));
            dialog.present(window);
            return false;
        }
        GLib.file_set_contents(configFile(), text);
        const mouse = selectedMouse();
        if (mouse)
            writeDevice(mouse);
        writeAppearance(selectedNamePlace(), selectedLineWidth());
        baseline = snapshot();
        updateDirty();
        const restarted = mouse ? restartService() : false;
        const reloaded = restarted ? false : sendReload();
        toastOverlay.add_toast(new Adw.Toast({
            title: restarted
                ? t('save.mouse')
                : reloaded
                    ? t('save.reloaded')
                    : t('save.later'),
        }));
        return true;
    };
    dirtyBanner.connect('button-clicked', () => writeConfig());
    save.connect('clicked', () => writeConfig());
    window.connect('close-request', () => {
        if (snapshot() === baseline)
            return false;
        const dialog = new Adw.AlertDialog({
            heading: t('dirty.closeTitle'),
            body: t('dirty.closeBody'),
        });
        dialog.add_response('cancel', t('action.keepEditing'));
        dialog.add_response('discard', t('action.discard'));
        dialog.set_response_appearance('discard', Adw.ResponseAppearance.DESTRUCTIVE);
        dialog.set_default_response('cancel');
        dialog.set_close_response('cancel');
        dialog.connect('response', (_dialog, response) => {
            if (response === 'discard') {
                baseline = snapshot();
                window.close();
            }
        });
        dialog.present(window);
        return true;
    });
    let languageReady = false;
    languageRow.connect('notify::selected', () => {
        if (!languageReady)
            return;
        const next = languageRow.selected === 1 ? 'en' : 'zh';
        if (next === language)
            return;
        const revert = () => {
            languageReady = false;
            languageRow.selected = language === 'en' ? 1 : 0;
            languageReady = true;
        };
        const go = () => {
            writeLanguage(next);
            openSettingsWindow(next);
            window.destroy();
        };
        if (snapshot() === baseline) {
            go();
            return;
        }
        const dialog = new Adw.AlertDialog({
            heading: t('language.switchTitle'),
            body: t('language.switchBody'),
        });
        dialog.add_response('cancel', t('action.cancel'));
        dialog.add_response('discard', t('language.discardSwitch'));
        dialog.add_response('save', t('language.saveSwitch'));
        dialog.set_response_appearance('discard', Adw.ResponseAppearance.DESTRUCTIVE);
        dialog.set_response_appearance('save', Adw.ResponseAppearance.SUGGESTED);
        dialog.set_default_response('cancel');
        dialog.set_close_response('cancel');
        dialog.connect('response', (_dialog, response) => {
            if (response === 'save' && writeConfig())
                go();
            else if (response === 'discard')
                go();
            else
                revert();
        });
        dialog.present(window);
    });
    languageReady = true;
    window.present();
}

app.run([]);

function installCss() {
    const css = new Gtk.CssProvider();
    css.load_from_string(STYLE);
    Gtk.StyleContext.add_provider_for_display(
        Gdk.Display.get_default(),
        css,
        Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

function confirmDelete(parent, title, done) {
    const dialog = new Adw.AlertDialog({
        heading: t('rules.deleteTitle', {name: title}),
        body: t('rules.deleteBody'),
    });
    dialog.add_response('cancel', t('action.cancel'));
    dialog.add_response('delete', t('action.delete'));
    dialog.set_response_appearance('delete', Adw.ResponseAppearance.DESTRUCTIVE);
    dialog.set_default_response('cancel');
    dialog.set_close_response('cancel');
    dialog.connect('response', (_dialog, response) => {
        if (response === 'delete')
            done();
    });
    dialog.present(parent);
}

function listMice() {
    const bin = GLib.getenv('STROKELET_BIN') || 'strokelet';
    try {
        const proc = Gio.Subprocess.new(
            [bin, 'list-devices'],
            Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_SILENCE,
        );
        const [, stdout] = proc.communicate_utf8(null, null);
        return (stdout || '').split('\n').flatMap(line => {
            if (!line.startsWith('accepted\t'))
                return [];
            const parts = line.split('\t');
            const path = parts[1] || '';
            const name = (parts[2] || '').replace(/^name=/, '') || path;
            return path ? [{path, name}] : [];
        });
    } catch {
        return [];
    }
}

function deviceFile() {
    return GLib.build_filenamev([GLib.get_user_config_dir(), 'strokelet', 'device']);
}

function readDevice() {
    try {
        const [ok, bytes] = GLib.file_get_contents(deviceFile());
        if (!ok)
            return '';
        return new TextDecoder().decode(bytes).split('\n')[0].trim();
    } catch {
        return '';
    }
}

function appearanceFile() {
    return GLib.build_filenamev([GLib.get_user_config_dir(), 'strokelet', 'appearance.json']);
}

function readAppearance() {
    try {
        const [ok, bytes] = GLib.file_get_contents(appearanceFile());
        if (!ok)
            return {namePlace: 'bottom', lineWidth: 6};
        const data = JSON.parse(new TextDecoder().decode(bytes));
        const width = Number(data.lineWidth);
        return {
            namePlace: data.namePlace === 'pointer' ? 'pointer' : 'bottom',
            lineWidth: Number.isFinite(width) ? Math.min(20, Math.max(2, Math.round(width))) : 6,
        };
    } catch {
        return {namePlace: 'bottom', lineWidth: 6};
    }
}

function writeAppearance(namePlace, lineWidth) {
    const dir = GLib.build_filenamev([GLib.get_user_config_dir(), 'strokelet']);
    GLib.mkdir_with_parents(dir, 0o755);
    let data = {};
    try {
        const [ok, bytes] = GLib.file_get_contents(appearanceFile());
        if (ok)
            data = JSON.parse(new TextDecoder().decode(bytes));
    } catch {
        data = {};
    }
    data.namePlace = namePlace;
    data.lineWidth = lineWidth;
    GLib.file_set_contents(appearanceFile(), `${JSON.stringify(data, null, 2)}\n`);
}

function writeDevice(path) {
    const dir = GLib.build_filenamev([GLib.get_user_config_dir(), 'strokelet']);
    GLib.mkdir_with_parents(dir, 0o755);
    GLib.file_set_contents(deviceFile(), `${path}\n`);
}

function restartService() {
    try {
        const proc = Gio.Subprocess.new(
            ['systemctl', '--user', 'restart', 'strokelet.service'],
            Gio.SubprocessFlags.STDOUT_SILENCE | Gio.SubprocessFlags.STDERR_SILENCE,
        );
        return proc.wait_check(null);
    } catch {
        return false;
    }
}

function configFile() {
    const dir = GLib.build_filenamev([GLib.get_user_config_dir(), 'strokelet']);
    GLib.mkdir_with_parents(dir, 0o755);
    return GLib.build_filenamev([dir, 'gestures.json']);
}

function readConfig() {
    try {
        const [ok, bytes] = GLib.file_get_contents(configFile());
        if (!ok)
            return defaultConfig();
        const config = JSON.parse(new TextDecoder().decode(bytes));
        if (!Array.isArray(config.rules))
            return defaultConfig();
        config.rules.forEach(rule => {
            if (hasButton(rule) && !rule.hold)
                rule.hold = config.trigger || 'right';
        });
        return config;
    } catch {
        return defaultConfig();
    }
}

function defaultConfig() {
    return {
        version: 1,
        trigger: 'right',
        rules: [{direction: 'up', modifiers: ['ctrl'], key: 'c'}],
    };
}

function recognitionOf(config) {
    const raw = config.recognition || {};
    return {
        clickSlop: clampNumber(raw.clickSlop, 1, 400, 12),
        minLength: clampNumber(raw.minLength, 20, 5000, 80),
        maxDurationMs: clampNumber(raw.maxDurationMs, 200, 60000, 2500),
        maxRules: clampNumber(raw.maxRules, 1, 64, 16),
    };
}

function clampNumber(value, lower, upper, fallback) {
    const number = Math.round(Number(value));
    if (!Number.isFinite(number))
        return fallback;
    return Math.min(upper, Math.max(lower, number));
}

function spinRow(title, subtitle, value, lower, upper, step) {
    return new Adw.SpinRow({
        title,
        subtitle,
        adjustment: new Gtk.Adjustment({
            lower,
            upper,
            step_increment: step,
            page_increment: step * 2,
            value,
        }),
        digits: 0,
    });
}

function cloneRule(rule) {
    const copy = {...rule};
    if (Array.isArray(rule.points))
        copy.points = rule.points.map(point => [...point]);
    if (Array.isArray(rule.modifiers))
        copy.modifiers = [...rule.modifiers];
    if (Array.isArray(rule.modifierCodes))
        copy.modifierCodes = [...rule.modifierCodes];
    return copy;
}

function commitRule(target, draft) {
    for (const key of Object.keys(target))
        delete target[key];
    Object.assign(target, cloneRule(draft));
}

function sendReload() {
    try {
        const uid = new Gio.Credentials().get_unix_user();
        const client = Gio.SocketClient.new();
        const connection = client.connect(Gio.UnixSocketAddress.new(`/run/strokelet/${uid}/strokelet.sock`), null);
        connection.get_output_stream().write_bytes(new GLib.Bytes('{"type":"reload"}\n'), null);
        connection.close(null);
        return true;
    } catch {
        return false;
    }
}

function shapeOf(rule, triggerName) {
    if (hasWheel(rule))
        return t('shape.wheel', {hold: buttonPhrase(rule.hold || 'right'), direction: wheelPhrase(rule.wheel)});
    if (hasButton(rule))
        return t('shape.chord', {hold: buttonPhrase(rule.hold || 'right'), press: buttonPhrase(rule.button)});
    const trigger = buttonPhrase(triggerName || 'right');
    if (Array.isArray(rule.points) && rule.points.length >= 2)
        return t('shape.drawn', {trigger});
    if (rule.direction)
        return t('shape.direction', {trigger, direction: directionPhrase(rule.direction)});
    return t('shape.unset');
}

function wheelPhrase(name) {
    return phrase(`direction.${name}`, name);
}

function kindTag(rule) {
    if (hasWheel(rule))
        return t('kind.wheel');
    return hasButton(rule) ? t('kind.chord') : t('kind.stroke');
}

function directionPhrase(name) {
    return phrase(`direction.${name}`, name);
}

function buttonPhrase(name) {
    return phrase(`button.${name}`, name);
}

function phrase(key, fallback) {
    const text = t(key);
    return text === key ? fallback : text;
}

function rowTitle(rule) {
    if (typeof rule.name === 'string' && rule.name.length > 0)
        return rule.name;
    const shortcut = labelOf(rule);
    return shortcut === t('rule.noShortcut') ? t('rule.untitled') : shortcut;
}

function rowSubtitle(rule, triggerName) {
    const action = shapeOf(rule, triggerName);
    if (typeof rule.name === 'string' && rule.name.length > 0)
        return `${action} · ${labelOf(rule)}`;
    return action;
}

function hasButton(rule) {
    return typeof rule.button === 'string' && rule.button.length > 0;
}

function hasWheel(rule) {
    return typeof rule.wheel === 'string' && rule.wheel.length > 0;
}

function hasStroke(rule) {
    return (Array.isArray(rule.points) && rule.points.length >= 2) || Boolean(rule.direction);
}

function checkGestures(text) {
    const bin = GLib.getenv('STROKELET_BIN');
    if (!bin)
        return t('editor.noCheck');
    try {
        const proc = Gio.Subprocess.new(
            [bin, 'check-gestures'],
            Gio.SubprocessFlags.STDIN_PIPE | Gio.SubprocessFlags.STDERR_PIPE,
        );
        const [, , stderr] = proc.communicate_utf8(text, null);
        if (proc.get_successful())
            return null;
        return stderr || t('editor.invalid');
    } catch (error) {
        return `${error}`;
    }
}

function labelOf(rule) {
    if (rule.label)
        return rule.label;
    const names = [...(rule.modifiers ?? []), rule.key].filter(Boolean).map(prettyChordPart);
    return names.length > 0 ? names.join('+') : t('rule.noShortcut');
}

function prettyChordPart(name) {
    const known = {
        ctrl: 'Ctrl',
        control: 'Ctrl',
        alt: 'Alt',
        shift: 'Shift',
        super: 'Super',
        meta: 'Super',
    };
    const lower = String(name).toLowerCase();
    if (known[lower])
        return known[lower];
    if (lower.length === 1)
        return lower.toUpperCase();
    return name;
}

function hasChord(rule) {
    return Number.isInteger(rule.keyCode) || (typeof rule.key === 'string' && rule.key.length > 0);
}

function watchProcess(argv, onLine, onClose) {
    let proc = null;
    let stdout = null;
    const stop = () => {
        stdout = null;
        if (proc === null)
            return;
        try {
            proc.force_exit();
        } catch {
        }
        proc = null;
    };
    try {
        proc = Gio.Subprocess.new(argv, Gio.SubprocessFlags.STDOUT_PIPE);
    } catch (error) {
        onClose(`${error}`);
        return () => {};
    }
    stdout = new Gio.DataInputStream({
        base_stream: proc.get_stdout_pipe(),
        close_base_stream: true,
    });
    const pump = () => {
        const stream = stdout;
        if (stream === null)
            return;
        stream.read_line_async(GLib.PRIORITY_DEFAULT, null, (source, result) => {
            if (source !== stdout)
                return;
            try {
                const [line] = source.read_line_finish_utf8(result);
                if (line === null) {
                    if (proc !== null)
                        onClose(t('editor.noResult'));
                    proc = null;
                    return;
                }
                if (onLine(line) && proc !== null)
                    pump();
            } catch {
                if (proc !== null)
                    onClose(t('editor.interrupted'));
                proc = null;
            }
        });
    };
    pump();
    return stop;
}

function previewPoints(rule) {
    if (Array.isArray(rule.points) && rule.points.length >= 2)
        return rule.points;
    return {
        up: [[0, 0], [0, -80]],
        down: [[0, 0], [0, 80]],
        left: [[0, 0], [-80, 0]],
        right: [[0, 0], [80, 0]],
    }[rule.direction] || null;
}

function strokePreview(rule) {
    const area = new Gtk.DrawingArea({
        content_height: 140,
        hexpand: true,
        css_classes: ['stroke-preview'],
    });
    area.set_draw_func((_widget, cr) => {
        const width = area.get_width();
        const height = area.get_height();
        const color = area.get_color();
        const pts = previewPoints(rule);
        if (!pts) {
            const layout = area.create_pango_layout(t('editor.previewPlaceholder'));
            const [textWidth, textHeight] = layout.get_pixel_size();
            cr.setSourceRGBA(color.red, color.green, color.blue, 0.45);
            cr.moveTo((width - textWidth) / 2, (height - textHeight) / 2);
            cr.showLayout(layout);
            return;
        }
        let minX = Infinity;
        let minY = Infinity;
        let maxX = -Infinity;
        let maxY = -Infinity;
        pts.forEach(([x, y]) => {
            minX = Math.min(minX, x);
            minY = Math.min(minY, y);
            maxX = Math.max(maxX, x);
            maxY = Math.max(maxY, y);
        });
        const span = Math.max(maxX - minX, maxY - minY, 1);
        const scale = Math.min(width, height) * 0.72 / span;
        const ox = width / 2 - ((minX + maxX) / 2) * scale;
        const oy = height / 2 - ((minY + maxY) / 2) * scale;
        const at = ([x, y]) => [ox + x * scale, oy + y * scale];
        cr.setSourceRGBA(color.red, color.green, color.blue, 0.9);
        cr.setLineWidth(3);
        cr.setLineCap(1);
        pts.forEach((point, index) => {
            const [px, py] = at(point);
            if (index === 0)
                cr.moveTo(px, py);
            else
                cr.lineTo(px, py);
        });
        cr.stroke();
        const [sx, sy] = at(pts[0]);
        const [ex, ey] = at(pts[pts.length - 1]);
        cr.arc(sx, sy, 3, 0, Math.PI * 2);
        cr.fill();
        cr.arc(ex, ey, 5, 0, Math.PI * 2);
        cr.fill();
    });
    return area;
}

function modelOf(items) {
    const list = new Gtk.StringList();
    items.forEach(item => list.append(item));
    return list;
}

function editRule(parent, rule, triggerName, done, isNew = false) {
    const draft = cloneRule(rule);
    const dialog = new Adw.Dialog({
        title: isNew ? t('editor.new') : t('editor.edit'),
        content_width: 480,
    });
    const toolbar = new Adw.ToolbarView();
    const header = new Adw.HeaderBar();
    const apply = new Gtk.Button({label: t('action.done'), css_classes: ['suggested-action']});
    header.pack_end(apply);
    toolbar.add_top_bar(header);
    const banner = new Adw.Banner({revealed: false});
    toolbar.add_top_bar(banner);

    const page = new Adw.PreferencesPage();
    const actionGroup = new Adw.PreferencesGroup({
        title: t('editor.how'),
        description: t('editor.howDescription'),
    });
    const strokeRadio = new Gtk.CheckButton({
        active: !hasButton(draft) && !hasWheel(draft),
        valign: Gtk.Align.CENTER,
    });
    const chordRadio = new Gtk.CheckButton({
        group: strokeRadio,
        active: hasButton(draft),
        valign: Gtk.Align.CENTER,
    });
    const wheelRadio = new Gtk.CheckButton({
        group: strokeRadio,
        active: hasWheel(draft),
        valign: Gtk.Align.CENTER,
    });
    const strokeRow = new Adw.ActionRow({title: t('editor.draw'), activatable_widget: strokeRadio});
    const drawButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    strokeRow.add_prefix(strokeRadio);
    strokeRow.add_suffix(drawButton);
    const chordRow = new Adw.ActionRow({title: t('editor.chord'), activatable_widget: chordRadio});
    const chordButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    chordRow.add_prefix(chordRadio);
    chordRow.add_suffix(chordButton);
    const wheelRow = new Adw.ActionRow({title: t('editor.wheel'), activatable_widget: wheelRadio});
    const wheelButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    wheelRow.add_prefix(wheelRadio);
    wheelRow.add_suffix(wheelButton);
    actionGroup.add(strokeRow);
    actionGroup.add(chordRow);
    actionGroup.add(wheelRow);
    page.add(actionGroup);

    const previewGroup = new Adw.PreferencesGroup({
        title: t('editor.preview'),
        description: t('editor.previewReady'),
    });
    const preview = strokePreview(draft);
    const previewBox = new Gtk.Box({
        margin_top: 6,
        margin_bottom: 12,
        margin_start: 12,
        margin_end: 12,
    });
    previewBox.append(preview);
    previewGroup.add(previewBox);
    page.add(previewGroup);

    const resultGroup = new Adw.PreferencesGroup({
        title: t('editor.then'),
        description: t('editor.thenDescription'),
    });
    const shortcutRow = new Adw.ActionRow({title: t('editor.shortcut')});
    const shortcutButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    shortcutRow.add_suffix(shortcutButton);
    shortcutRow.activatable_widget = shortcutButton;
    const nameRow = new Adw.EntryRow({
        title: t('editor.screenName'),
        text: typeof draft.name === 'string' ? draft.name : '',
        max_length: 16,
        show_apply_button: false,
    });
    resultGroup.add(shortcutRow);
    page.add(resultGroup);
    const nameGroup = new Adw.PreferencesGroup({
        title: t('editor.nameGroup'),
        description: t('editor.nameDescription'),
    });
    nameGroup.add(nameRow);
    page.add(nameGroup);
    toolbar.set_content(page);
    dialog.set_child(toolbar);

    const bin = GLib.getenv('STROKELET_BIN');
    let session = 0;
    let stopProcess = () => {};
    const stopRecording = () => {
        session += 1;
        stopProcess();
        stopProcess = () => {};
    };
    const recordButtons = [drawButton, chordButton, wheelButton, shortcutButton];
    const setRecording = busy => {
        recordButtons.forEach(button => {
            button.sensitive = !busy;
        });
        apply.sensitive = !busy;
        strokeRadio.sensitive = !busy;
        chordRadio.sensitive = !busy;
        wheelRadio.sensitive = !busy;
    };
    const showStatus = (message, cancellable) => {
        banner.title = translateDaemon(languageCode, message || '');
        banner.button_label = cancellable ? t('action.cancel') : '';
        banner.revealed = Boolean(message);
    };
    const refreshEditor = () => {
        const trigger = buttonPhrase(triggerName);
        strokeRow.subtitle = hasStroke(draft)
            ? shapeOf(draft, triggerName)
            : t('editor.drawEmpty', {trigger});
        chordRow.subtitle = hasButton(draft)
            ? shapeOf(draft, triggerName)
            : t('editor.chordEmpty');
        wheelRow.subtitle = hasWheel(draft)
            ? shapeOf(draft, triggerName)
            : t('editor.wheelEmpty');
        drawButton.label = hasStroke(draft) ? t('action.rerecord') : t('action.record');
        chordButton.label = hasButton(draft) ? t('action.rerecord') : t('action.record');
        wheelButton.label = hasWheel(draft) ? t('action.rerecord') : t('action.record');
        shortcutRow.subtitle = hasChord(draft) ? labelOf(draft) : t('editor.shortcutEmpty');
        shortcutButton.label = hasChord(draft) ? t('action.rerecord') : t('action.record');
        previewGroup.visible = strokeRadio.active;
        previewGroup.description = previewPoints(draft)
            ? t('editor.previewReady')
            : t('editor.previewEmpty');
        preview.queue_draw();
    };
    refreshEditor();
    strokeRadio.connect('toggled', () => refreshEditor());
    chordRadio.connect('toggled', () => refreshEditor());
    wheelRadio.connect('toggled', () => refreshEditor());

    const keys = new Gtk.EventControllerKey();
    page.add_controller(keys);
    keys.connect('key-pressed', (_controller, keyval) => {
        if (keyval === 0xff1b) {
            stopRecording();
            setRecording(false);
            showStatus(t('editor.cancelled'), false);
            refreshEditor();
            return true;
        }
        return false;
    });
    banner.connect('button-clicked', () => {
        stopRecording();
        setRecording(false);
        showStatus(t('editor.cancelled'), false);
        refreshEditor();
    });
    const begin = (argv, onLine) => {
        stopRecording();
        const current = session;
        if (!bin) {
            showStatus(t('editor.noBinary'), false);
            refreshEditor();
            return;
        }
        setRecording(true);
        stopProcess = watchProcess([bin, ...argv], line => {
            if (current !== session)
                return false;
            return onLine(line);
        }, message => {
            if (current !== session)
                return;
            setRecording(false);
            showStatus(message, false);
            refreshEditor();
        });
    };
    const readMessage = (line, onMessage) => {
        let msg;
        try {
            msg = JSON.parse(line);
        } catch {
            return true;
        }
        return onMessage(msg);
    };
    drawButton.connect('clicked', () => {
        strokeRadio.active = true;
        showStatus(t('editor.drawPrompt', {trigger: buttonPhrase(triggerName)}), true);
        begin(['capture-stroke', triggerName], line => readMessage(line, msg => {
            if (msg.type === 'stroke') {
                delete draft.direction;
                delete draft.button;
                delete draft.hold;
                delete draft.wheel;
                draft.points = msg.points;
                setRecording(false);
                showStatus(t('editor.drawSaved'), false);
                refreshEditor();
                return false;
            }
            if (msg.type === 'status') {
                showStatus(msg.message || banner.title, true);
                return true;
            }
            setRecording(false);
            showStatus(translateDaemon(languageCode, msg.message) || t('editor.notSaved'), false);
            refreshEditor();
            return false;
        }));
    });
    chordButton.connect('clicked', () => {
        chordRadio.active = true;
        showStatus(t('editor.chordPrompt'), true);
        begin(['capture-button', triggerName], line => readMessage(line, msg => {
            if (msg.type === 'button') {
                delete draft.direction;
                delete draft.points;
                delete draft.wheel;
                draft.hold = msg.hold;
                draft.button = msg.button;
                setRecording(false);
                showStatus(t('editor.chordSaved'), false);
                refreshEditor();
                return false;
            }
            if (msg.type === 'status') {
                showStatus(msg.message || banner.title, true);
                return true;
            }
            setRecording(false);
            showStatus(translateDaemon(languageCode, msg.message) || t('editor.notSaved'), false);
            refreshEditor();
            return false;
        }));
    });
    wheelButton.connect('clicked', () => {
        wheelRadio.active = true;
        showStatus(t('editor.wheelPrompt'), true);
        begin(['capture-wheel'], line => readMessage(line, msg => {
            if (msg.type === 'wheel') {
                delete draft.direction;
                delete draft.points;
                delete draft.button;
                draft.hold = msg.hold;
                draft.wheel = msg.wheel;
                setRecording(false);
                showStatus(t('editor.wheelSaved'), false);
                refreshEditor();
                return false;
            }
            if (msg.type === 'status') {
                showStatus(msg.message || banner.title, true);
                return true;
            }
            setRecording(false);
            showStatus(translateDaemon(languageCode, msg.message) || t('editor.notSaved'), false);
            refreshEditor();
            return false;
        }));
    });
    shortcutButton.connect('clicked', () => {
        showStatus(t('editor.shortcutPrompt'), true);
        begin(['capture-chord'], line => readMessage(line, msg => {
            if (msg.type === 'live') {
                shortcutButton.label = msg.label || t('editor.listeningPlain');
                showStatus(msg.label ? t('editor.listening', {label: msg.label}) : t('editor.listeningPlain'), true);
                return true;
            }
            if (msg.type === 'chord') {
                delete draft.modifiers;
                delete draft.key;
                draft.modifierCodes = msg.modifierCodes;
                draft.keyCode = msg.keyCode;
                draft.label = msg.label;
                setRecording(false);
                showStatus(t('editor.shortcutSaved'), false);
                refreshEditor();
                return false;
            }
            setRecording(false);
            showStatus(translateDaemon(languageCode, msg.message) || t('editor.notSaved'), false);
            refreshEditor();
            return false;
        }));
    });
    dialog.connect('closed', () => stopRecording());

    apply.connect('clicked', () => {
        if (strokeRadio.active) {
            if (!hasStroke(draft)) {
                showStatus(t('editor.needStroke'), false);
                return;
            }
            delete draft.button;
            delete draft.hold;
            delete draft.wheel;
        } else if (chordRadio.active) {
            if (!hasButton(draft)) {
                showStatus(t('editor.needChord'), false);
                return;
            }
            delete draft.direction;
            delete draft.points;
            delete draft.wheel;
        } else if (!hasWheel(draft)) {
            showStatus(t('editor.needWheel'), false);
            return;
        } else {
            delete draft.direction;
            delete draft.points;
            delete draft.button;
        }
        if (!hasChord(draft)) {
            showStatus(t('editor.needShortcut'), false);
            return;
        }
        const screenName = nameRow.text.trim();
        if ([...screenName].length > 16) {
            showStatus(t('editor.nameTooLong'), false);
            return;
        }
        if (screenName)
            draft.name = screenName;
        else
            delete draft.name;
        commitRule(rule, draft);
        done();
        dialog.close();
    });
    dialog.present(parent);
}

import Adw from 'gi://Adw';
import Gdk from 'gi://Gdk?version=4.0';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk?version=4.0';

const TRIGGERS = ['left', 'right', 'middle', 'forward', 'back'];
const TRIGGER_LABELS = ['左键', '右键', '中键', '侧键前进', '侧键后退'];
const MAX_RULES = 16;
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
        subtitle: '给鼠标动作配快捷键',
    });
    const save = new Gtk.Button({label: '保存', css_classes: ['suggested-action']});
    header.pack_end(save);
    toolbar.add_top_bar(header);
    const dirtyBanner = new Adw.Banner({
        title: '有修改还没保存',
        button_label: '保存',
        revealed: false,
    });
    toolbar.add_top_bar(dirtyBanner);

    const page = new Adw.PreferencesPage();
    const mice = listMice();
    const savedMouse = readDevice();
    const mouseGroup = new Adw.PreferencesGroup({
        title: '鼠标',
        description: mice.length > 1
            ? '这台电脑有多只鼠标。保存后使用下面这一只。'
            : '保存后使用这只鼠标。',
    });
    const mouseRow = new Adw.ComboRow({
        title: '使用这只',
        model: modelOf(mice.length === 0 ? ['没有找到鼠标'] : mice.map(mouse => mouse.name)),
        sensitive: mice.length > 0,
    });
    const savedIndex = mice.findIndex(mouse => mouse.path === savedMouse);
    mouseRow.selected = savedIndex >= 0 ? savedIndex : 0;
    mouseGroup.add(mouseRow);
    page.add(mouseGroup);
    const selectedMouse = () => mice[mouseRow.selected]?.path || '';
    const triggerGroup = new Adw.PreferencesGroup({
        title: '轨迹',
        description: '画轨迹时按住这个键。鼠标组合不看它，以你先按下的那个键为准。',
    });
    const trigger = new Adw.ComboRow({title: '触发键', model: modelOf(TRIGGER_LABELS)});
    trigger.selected = Math.max(0, TRIGGERS.indexOf(config.trigger));
    triggerGroup.add(trigger);
    page.add(triggerGroup);

    const rules = config.rules.map(rule => cloneRule(rule));
    const rulesGroup = new Adw.PreferencesGroup({title: '规则'});
    const add = new Gtk.Button({label: '添加', css_classes: ['flat'], tooltip_text: '添加一条规则'});
    rulesGroup.set_header_suffix(add);
    const ruleRows = [];
    const triggerName = () => TRIGGERS[trigger.selected] || 'right';
    const snapshot = () => JSON.stringify({
        version: 1,
        trigger: triggerName(),
        rules,
        mouse: selectedMouse(),
    });
    let baseline = '';
    const updateDirty = () => {
        dirtyBanner.revealed = baseline !== '' && snapshot() !== baseline;
    };
    const refresh = () => {
        ruleRows.splice(0).forEach(row => rulesGroup.remove(row));
        rulesGroup.description = rules.length === 0
            ? '点这一栏右边的「添加」。一条规则是一个鼠标动作，加上要按下的快捷键。'
            : `点一条可以修改。已有 ${rules.length} 条，最多 ${MAX_RULES} 条。`;
        if (rules.length === 0) {
            const empty = new Adw.ActionRow({
                title: '还没有规则',
                subtitle: '添加后，选画轨迹或按两个鼠标键，再录快捷键。',
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
                label: hasButton(rule) ? '组合' : '轨迹',
                css_classes: ['caption', 'kind-tag'],
            });
            row.add_prefix(tag);
            const remove = new Gtk.Button({
                icon_name: 'user-trash-symbolic',
                valign: Gtk.Align.CENTER,
                css_classes: ['flat'],
                tooltip_text: '删除这条规则',
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
        if (rules.length >= MAX_RULES) {
            toastOverlay.add_toast(new Adw.Toast({title: `最多 ${MAX_RULES} 条规则`}));
            return;
        }
        const rule = {};
        editRule(window, rule, triggerName(), () => {
            rules.push(rule);
            refresh();
        }, true);
    });
    mouseRow.connect('notify::selected', () => updateDirty());
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
            rules,
        }, null, 2)}\n`;
        const problem = checkGestures(text);
        if (problem) {
            const dialog = new Adw.AlertDialog({heading: '没有保存', body: problem});
            dialog.add_response('ok', '好');
            dialog.present(window);
            return;
        }
        GLib.file_set_contents(configFile(), text);
        const mouse = selectedMouse();
        if (mouse)
            writeDevice(mouse);
        baseline = snapshot();
        updateDirty();
        const restarted = mouse ? restartService() : false;
        const reloaded = restarted ? false : sendReload();
        toastOverlay.add_toast(new Adw.Toast({
            title: restarted
                ? '已保存，已换上这只鼠标'
                : reloaded
                    ? '已保存，正在运行的演示已换上新规则'
                    : '已保存。下次启动时会用这份规则',
        }));
    };
    dirtyBanner.connect('button-clicked', () => writeConfig());
    save.connect('clicked', () => writeConfig());
    window.connect('close-request', () => {
        if (snapshot() === baseline)
            return false;
        const dialog = new Adw.AlertDialog({
            heading: '还有没保存的修改',
            body: '关掉窗口会丢掉这些修改。',
        });
        dialog.add_response('cancel', '继续编辑');
        dialog.add_response('discard', '丢掉');
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
    window.present();
});

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
        heading: `删除「${title}」？`,
        body: '这条规则会从列表里去掉。要点保存才会写进文件。',
    });
    dialog.add_response('cancel', '取消');
    dialog.add_response('delete', '删除');
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
    if (hasButton(rule))
        return `按住${buttonPhrase(rule.hold || 'right')}，再按${buttonPhrase(rule.button)}`;
    const trigger = buttonPhrase(triggerName || 'right');
    if (Array.isArray(rule.points) && rule.points.length >= 2)
        return `按住${trigger}画的轨迹`;
    if (rule.direction)
        return `按住${trigger}${directionPhrase(rule.direction)}`;
    return '还没设置动作';
}

function directionPhrase(name) {
    return {up: '向上', down: '向下', left: '向左', right: '向右'}[name] || name;
}

function buttonPhrase(name) {
    return {left: '左键', right: '右键', middle: '中键', forward: '侧键前进', back: '侧键后退'}[name] || name;
}

function rowTitle(rule) {
    if (typeof rule.name === 'string' && rule.name.length > 0)
        return rule.name;
    const shortcut = labelOf(rule);
    return shortcut === '未设置' ? '未完成' : shortcut;
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

function hasStroke(rule) {
    return (Array.isArray(rule.points) && rule.points.length >= 2) || Boolean(rule.direction);
}

function checkGestures(text) {
    const bin = GLib.getenv('STROKELET_BIN');
    if (!bin)
        return '找不到 strokelet，无法检查轨迹';
    try {
        const proc = Gio.Subprocess.new(
            [bin, 'check-gestures'],
            Gio.SubprocessFlags.STDIN_PIPE | Gio.SubprocessFlags.STDERR_PIPE,
        );
        const [, , stderr] = proc.communicate_utf8(text, null);
        if (proc.get_successful())
            return null;
        return stderr || '规则无效';
    } catch (error) {
        return `${error}`;
    }
}

function labelOf(rule) {
    if (rule.label)
        return rule.label;
    const names = [...(rule.modifiers ?? []), rule.key].filter(Boolean).map(prettyChordPart);
    return names.length > 0 ? names.join('+') : '未设置';
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
                        onClose('录制没有返回结果');
                    proc = null;
                    return;
                }
                if (onLine(line) && proc !== null)
                    pump();
            } catch {
                if (proc !== null)
                    onClose('录制中断');
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
            const layout = area.create_pango_layout('画出后显示在这里');
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
        title: isNew ? '新规则' : '编辑规则',
        content_width: 480,
    });
    const toolbar = new Adw.ToolbarView();
    const header = new Adw.HeaderBar();
    const apply = new Gtk.Button({label: '完成', css_classes: ['suggested-action']});
    header.pack_end(apply);
    toolbar.add_top_bar(header);
    const banner = new Adw.Banner({revealed: false});
    toolbar.add_top_bar(banner);

    const page = new Adw.PreferencesPage();
    const actionGroup = new Adw.PreferencesGroup({
        title: '怎么触发',
        description: '只选一种。画轨迹用窗口里的触发键；鼠标组合以你先按下的键为准。',
    });
    const strokeRadio = new Gtk.CheckButton({active: !hasButton(draft), valign: Gtk.Align.CENTER});
    const chordRadio = new Gtk.CheckButton({
        group: strokeRadio,
        active: hasButton(draft),
        valign: Gtk.Align.CENTER,
    });
    const strokeRow = new Adw.ActionRow({title: '画出轨迹', activatable_widget: strokeRadio});
    const drawButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    strokeRow.add_prefix(strokeRadio);
    strokeRow.add_suffix(drawButton);
    const chordRow = new Adw.ActionRow({title: '鼠标组合', activatable_widget: chordRadio});
    const chordButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    chordRow.add_prefix(chordRadio);
    chordRow.add_suffix(chordButton);
    actionGroup.add(strokeRow);
    actionGroup.add(chordRow);
    page.add(actionGroup);

    const previewGroup = new Adw.PreferencesGroup({
        title: '轨迹预览',
        description: '小点是起点，大点是松开的地方。',
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
        title: '然后做什么',
        description: '录快捷键时键盘会暂时独占，系统不会先把这组键吃掉。',
    });
    const shortcutRow = new Adw.ActionRow({title: '快捷键'});
    const shortcutButton = new Gtk.Button({valign: Gtk.Align.CENTER});
    shortcutRow.add_suffix(shortcutButton);
    shortcutRow.activatable_widget = shortcutButton;
    const nameRow = new Adw.EntryRow({
        title: '屏幕上显示',
        text: typeof draft.name === 'string' ? draft.name : '',
        max_length: 16,
        show_apply_button: false,
    });
    resultGroup.add(shortcutRow);
    page.add(resultGroup);
    const nameGroup = new Adw.PreferencesGroup({
        title: '屏幕提示',
        description: '留空则不显示。识别成功后，这个名字出现在指针旁边大约一秒。',
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
    const recordButtons = [drawButton, chordButton, shortcutButton];
    const setRecording = busy => {
        recordButtons.forEach(button => {
            button.sensitive = !busy;
        });
        apply.sensitive = !busy;
        strokeRadio.sensitive = !busy;
        chordRadio.sensitive = !busy;
    };
    const showStatus = (message, cancellable) => {
        banner.title = message || '';
        banner.button_label = cancellable ? '取消' : '';
        banner.revealed = Boolean(message);
    };
    const refreshEditor = () => {
        const trigger = buttonPhrase(triggerName);
        strokeRow.subtitle = hasStroke(draft)
            ? shapeOf(draft, triggerName)
            : `按住${trigger}，在屏幕上画一笔后松开`;
        chordRow.subtitle = hasButton(draft)
            ? shapeOf(draft, triggerName)
            : '先按住一个键，再按另一个';
        drawButton.label = hasStroke(draft) ? '重录' : '录制';
        chordButton.label = hasButton(draft) ? '重录' : '录制';
        shortcutRow.subtitle = hasChord(draft) ? labelOf(draft) : '还没录';
        shortcutButton.label = hasChord(draft) ? '重录' : '录制';
        previewGroup.visible = strokeRadio.active;
        previewGroup.description = previewPoints(draft)
            ? '小点是起点，大点是松开的地方。'
            : '画出之后，轨迹会显示在这里。';
        preview.queue_draw();
    };
    refreshEditor();
    strokeRadio.connect('toggled', () => refreshEditor());

    const keys = new Gtk.EventControllerKey();
    page.add_controller(keys);
    keys.connect('key-pressed', (_controller, keyval) => {
        if (keyval === 0xff1b) {
            stopRecording();
            setRecording(false);
            showStatus('已取消', false);
            refreshEditor();
            return true;
        }
        return false;
    });
    banner.connect('button-clicked', () => {
        stopRecording();
        setRecording(false);
        showStatus('已取消', false);
        refreshEditor();
    });
    const begin = (argv, onLine) => {
        stopRecording();
        const current = session;
        if (!bin) {
            showStatus('找不到 strokelet', false);
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
        showStatus(`按住${buttonPhrase(triggerName)}画出轨迹，然后松开。`, true);
        begin(['capture-stroke', triggerName], line => readMessage(line, msg => {
            if (msg.type === 'stroke') {
                delete draft.direction;
                delete draft.button;
                delete draft.hold;
                draft.points = msg.points;
                setRecording(false);
                showStatus('轨迹已记下。点完成后才会写进这条规则。', false);
                refreshEditor();
                return false;
            }
            if (msg.type === 'status') {
                showStatus(msg.message || banner.title, true);
                return true;
            }
            setRecording(false);
            showStatus(msg.message || '这次没有保存', false);
            refreshEditor();
            return false;
        }));
    });
    chordButton.connect('clicked', () => {
        chordRadio.active = true;
        showStatus('先按住起始键，再按另一个。左键、右键、中键或侧键都可以。', true);
        begin(['capture-button', triggerName], line => readMessage(line, msg => {
            if (msg.type === 'button') {
                delete draft.direction;
                delete draft.points;
                draft.hold = msg.hold;
                draft.button = msg.button;
                setRecording(false);
                showStatus('鼠标组合已记下。点完成后才会写进这条规则。', false);
                refreshEditor();
                return false;
            }
            if (msg.type === 'status') {
                showStatus(msg.message || banner.title, true);
                return true;
            }
            setRecording(false);
            showStatus(msg.message || '这次没有保存', false);
            refreshEditor();
            return false;
        }));
    });
    shortcutButton.connect('clicked', () => {
        showStatus('键盘已暂时独占。按下快捷键，松开主键后记下。', true);
        begin(['capture-chord'], line => readMessage(line, msg => {
            if (msg.type === 'live') {
                shortcutButton.label = msg.label || '正在听…';
                showStatus(msg.label ? `正在听：${msg.label}` : '正在听…', true);
                return true;
            }
            if (msg.type === 'chord') {
                delete draft.modifiers;
                delete draft.key;
                draft.modifierCodes = msg.modifierCodes;
                draft.keyCode = msg.keyCode;
                draft.label = msg.label;
                setRecording(false);
                showStatus('快捷键已记下。点完成后才会写进这条规则。', false);
                refreshEditor();
                return false;
            }
            setRecording(false);
            showStatus(msg.message || '这次没有保存', false);
            refreshEditor();
            return false;
        }));
    });
    dialog.connect('closed', () => stopRecording());

    apply.connect('clicked', () => {
        if (strokeRadio.active) {
            if (!hasStroke(draft)) {
                showStatus('先画出轨迹，或改选鼠标组合。', false);
                return;
            }
            delete draft.button;
            delete draft.hold;
        } else if (!hasButton(draft)) {
            showStatus('先按下两个鼠标键，或改选画出轨迹。', false);
            return;
        } else {
            delete draft.direction;
            delete draft.points;
        }
        if (!hasChord(draft)) {
            showStatus('再录一组快捷键。', false);
            return;
        }
        const screenName = nameRow.text.trim();
        if ([...screenName].length > 16) {
            showStatus('屏幕名称最多 16 个字。', false);
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

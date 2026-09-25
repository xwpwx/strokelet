import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk?version=4.0';

const TRIGGERS = ['right', 'middle', 'forward', 'back'];
const app = new Adw.Application({application_id: 'org.strokelet.Settings'});

app.connect('activate', () => {
    const config = readConfig();
    const window = new Adw.ApplicationWindow({
        application: app,
        title: 'Strokelet',
        default_width: 560,
        default_height: 480,
    });
    const toolbar = new Adw.ToolbarView();
    const header = new Adw.HeaderBar();
    const save = new Gtk.Button({label: '保存', css_classes: ['suggested-action']});
    header.pack_end(save);
    toolbar.add_top_bar(header);

    const page = new Adw.PreferencesPage();
    const group = new Adw.PreferencesGroup({
        title: '鼠标动作',
        description: '按住触发键画出轨迹，再录一条快捷键。太像的轨迹不能同时留下。',
    });
    const trigger = new Adw.ComboRow({title: '触发键', model: modelOf(TRIGGERS)});
    trigger.selected = Math.max(0, TRIGGERS.indexOf(config.trigger));
    group.add(trigger);

    const list = new Gtk.ListBox({selection_mode: Gtk.SelectionMode.NONE, css_classes: ['boxed-list']});
    const rules = config.rules.map(rule => ({...rule}));
    const refresh = () => {
        list.remove_all();
        rules.forEach((rule, index) => {
            const row = new Adw.ActionRow({title: `${shapeOf(rule)} → ${titleOf(rule)}`});
            const edit = new Gtk.Button({label: '编辑'});
            edit.connect('clicked', () => editRule(window, rule, TRIGGERS[trigger.selected], () => refresh()));
            const remove = new Gtk.Button({label: '删除'});
            remove.connect('clicked', () => {
                rules.splice(index, 1);
                refresh();
            });
            row.add_suffix(edit);
            row.add_suffix(remove);
            list.append(row);
        });
    };
    refresh();
    group.add(list);
    const add = new Gtk.Button({label: '添加规则'});
    add.connect('clicked', () => {
        const rule = {};
        editRule(window, rule, TRIGGERS[trigger.selected], () => {
            rules.push(rule);
            refresh();
        });
    });
    group.add(add);
    page.add(group);
    toolbar.set_content(page);
    window.set_content(toolbar);
    save.connect('clicked', () => {
        const text = `${JSON.stringify({
            version: 1,
            trigger: TRIGGERS[trigger.selected],
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
        const reloaded = sendReload();
        const dialog = new Adw.AlertDialog({
            heading: '已保存',
            body: reloaded ? '正在运行的演示已重新读取规则。' : '演示进程没在监听。下次启动 run 时会使用这份规则。',
        });
        dialog.add_response('ok', '好');
        dialog.present(window);
    });
    window.present();
});

app.run([]);

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
        return JSON.parse(new TextDecoder().decode(bytes));
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

function writeConfig(config) {
    const text = `${JSON.stringify(config, null, 2)}\n`;
    GLib.file_set_contents(configFile(), text);
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

function shapeOf(rule) {
    if (Array.isArray(rule.points) && rule.points.length >= 2)
        return '轨迹';
    return rule.direction || '未画';
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

function titleOf(rule) {
    if (typeof rule.name === 'string' && rule.name.length > 0)
        return rule.name;
    return labelOf(rule);
}

function labelOf(rule) {
    if (rule.label)
        return rule.label;
    const names = [...(rule.modifiers ?? []), rule.key].filter(Boolean);
    return names.length > 0 ? names.join('+') : '未设置';
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

function strokePreview(rule) {
    const area = new Gtk.DrawingArea({content_width: 180, content_height: 120, hexpand: true});
    area.set_draw_func((_widget, cr, width, height) => {
        const pts = rule.points;
        if (!Array.isArray(pts) || pts.length < 2)
            return;
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
        const scale = Math.min(width, height) * 0.8 / span;
        const ox = width / 2 - ((minX + maxX) / 2) * scale;
        const oy = height / 2 - ((minY + maxY) / 2) * scale;
        cr.setSourceRGBA(0.2, 0.45, 0.85, 1);
        cr.setLineWidth(2);
        pts.forEach(([x, y], index) => {
            const px = ox + x * scale;
            const py = oy + y * scale;
            if (index === 0)
                cr.moveTo(px, py);
            else
                cr.lineTo(px, py);
        });
        cr.stroke();
    });
    return area;
}

function modelOf(items) {
    const list = new Gtk.StringList();
    items.forEach(item => list.append(item));
    return list;
}

function editRule(parent, rule, triggerName, done) {
    const dialog = new Adw.Dialog({title: '编辑规则', content_width: 420});
    const toolbar = new Adw.ToolbarView();
    const header = new Adw.HeaderBar();
    const apply = new Gtk.Button({label: '完成', css_classes: ['suggested-action']});
    header.pack_end(apply);
    toolbar.add_top_bar(header);
    const box = new Gtk.Box({
        orientation: Gtk.Orientation.VERTICAL,
        spacing: 12,
        margin_top: 12,
        margin_bottom: 12,
        margin_start: 12,
        margin_end: 12,
    });
    const draw = new Gtk.Button({label: hasStroke(rule) ? '重新画出轨迹' : '画出轨迹'});
    const preview = strokePreview(rule);
    const capture = new Gtk.Button({label: hasChord(rule) ? labelOf(rule) : '按下快捷键'});
    const nameEntry = new Gtk.Entry({
        placeholder_text: '例如 复制。留空则不显示',
        text: typeof rule.name === 'string' ? rule.name : '',
        max_length: 16,
    });
    const status = new Gtk.Label({
        label: '按住触发键，在屏幕上画一笔再松开。演示开着时直接画，它会把这一笔传回来。Esc 取消。',
        wrap: true,
        xalign: 0,
    });
    box.append(draw);
    box.append(preview);
    box.append(capture);
    box.append(new Gtk.Label({label: '屏幕上显示', xalign: 0}));
    box.append(nameEntry);
    box.append(status);
    toolbar.set_content(box);
    dialog.set_child(toolbar);

    const bin = GLib.getenv('STROKELET_BIN');
    let stopRecording = () => {};
    const keys = new Gtk.EventControllerKey();
    box.add_controller(keys);
    keys.connect('key-pressed', (_controller, keyval) => {
        if (keyval === 0xff1b) {
            stopRecording();
            status.label = '已取消';
            return true;
        }
        return false;
    });
    const begin = (argv, onLine) => {
        stopRecording();
        if (!bin) {
            status.label = '找不到 strokelet';
            return;
        }
        stopRecording = watchProcess([bin, ...argv], onLine, message => {
            status.label = message;
        });
    };
    draw.connect('clicked', () => {
        draw.label = '正在录制轨迹…';
        status.label = '按住触发键画出轨迹，然后松开。';
        begin(['capture-stroke', triggerName], line => {
            let msg;
            try {
                msg = JSON.parse(line);
            } catch {
                return true;
            }
            if (msg.type === 'stroke') {
                delete rule.direction;
                rule.points = msg.points;
                draw.label = '重新画出轨迹';
                preview.queue_draw();
                status.label = '轨迹已记下。点完成后才会写进这条规则。';
                return false;
            }
            if (msg.type === 'status') {
                status.label = msg.message || status.label;
                return true;
            }
            draw.label = hasStroke(rule) ? '重新画出轨迹' : '画出轨迹';
            status.label = msg.message || '这次没有保存';
            return false;
        });
    });
    capture.connect('clicked', () => {
        capture.label = '正在录制…';
        status.label = '键盘已暂时独占。松开主键后记下，Esc 取消。';
        begin(['capture-chord'], line => {
            let msg;
            try {
                msg = JSON.parse(line);
            } catch {
                return true;
            }
            if (msg.type === 'live') {
                capture.label = msg.label || '正在录制…';
                return true;
            }
            if (msg.type === 'chord') {
                delete rule.modifiers;
                delete rule.key;
                rule.modifierCodes = msg.modifierCodes;
                rule.keyCode = msg.keyCode;
                rule.label = msg.label;
                capture.label = msg.label;
                status.label = '快捷键已记下。点完成后才会写进这条规则。';
                return false;
            }
            capture.label = hasChord(rule) ? labelOf(rule) : '按下快捷键';
            status.label = msg.message || '这次没有保存';
            return false;
        });
    });
    dialog.connect('closed', () => stopRecording());

    apply.connect('clicked', () => {
        if (!hasStroke(rule)) {
            status.label = '先画出轨迹';
            return;
        }
        if (!hasChord(rule)) {
            status.label = '先按下一组快捷键';
            return;
        }
        const screenName = nameEntry.text.trim();
        if ([...screenName].length > 16) {
            status.label = '屏幕名称最多 16 个字';
            return;
        }
        if (screenName)
            rule.name = screenName;
        else
            delete rule.name;
        done();
        dialog.close();
    });
    dialog.present(parent);
}

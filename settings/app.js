import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk?version=4.0';

const TRIGGERS = ['right', 'middle', 'forward', 'back'];
const DIRECTIONS = ['up', 'down', 'left', 'right'];
const KEYS = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'f1', 'f2', 'f3', 'f4', 'f5', 'f6', 'f7', 'f8', 'f9', 'f10', 'f11', 'f12',
    'arrow-up', 'arrow-down', 'arrow-left', 'arrow-right',
    'enter', 'escape', 'tab', 'backspace', 'delete',
];
const MODIFIERS = ['ctrl', 'shift', 'alt', 'super'];

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
        description: '同一个触发键的每个方向只能有一条快捷键',
    });
    const trigger = new Adw.ComboRow({title: '触发键', model: modelOf(TRIGGERS)});
    trigger.selected = Math.max(0, TRIGGERS.indexOf(config.trigger));
    group.add(trigger);

    const list = new Gtk.ListBox({selection_mode: Gtk.SelectionMode.NONE, css_classes: ['boxed-list']});
    const rules = config.rules.map(rule => ({...rule, modifiers: [...rule.modifiers]}));
    const refresh = () => {
        list.remove_all();
        rules.forEach((rule, index) => {
            const row = new Adw.ActionRow({title: `${rule.direction} → ${labelOf(rule)}`});
            const edit = new Gtk.Button({label: '编辑'});
            edit.connect('clicked', () => editRule(window, rules, rule, () => refresh()));
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
        const rule = {direction: 'down', modifiers: ['ctrl'], key: 'v'};
        editRule(window, rules, rule, () => {
            const existing = rules.findIndex(item => item.direction === rule.direction);
            if (existing >= 0)
                rules[existing] = rule;
            else
                rules.push(rule);
            refresh();
        });
    });
    group.add(add);
    page.add(group);
    toolbar.set_content(page);
    window.set_content(toolbar);
    save.connect('clicked', () => {
        const next = {
            version: 1,
            trigger: TRIGGERS[trigger.selected],
            rules,
        };
        writeConfig(next);
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

function labelOf(rule) {
    return [...rule.modifiers, rule.key].join('+');
}

function modelOf(items) {
    const list = new Gtk.StringList();
    items.forEach(item => list.append(item));
    return list;
}

function editRule(parent, rules, rule, done) {
    const dialog = new Adw.Dialog({title: '编辑规则', content_width: 420});
    const toolbar = new Adw.ToolbarView();
    const header = new Adw.HeaderBar();
    const apply = new Gtk.Button({label: '完成', css_classes: ['suggested-action']});
    header.pack_end(apply);
    toolbar.add_top_bar(header);
    const box = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: 12, margin_top: 12, margin_bottom: 12, margin_start: 12, margin_end: 12});
    const direction = new Adw.ComboRow({title: '方向', model: modelOf(DIRECTIONS)});
    direction.selected = Math.max(0, DIRECTIONS.indexOf(rule.direction));
    const key = new Adw.ComboRow({title: '主键', model: modelOf(KEYS)});
    key.selected = Math.max(0, KEYS.indexOf(rule.key));
    box.append(direction);
    box.append(key);
    const switches = {};
    MODIFIERS.forEach(name => {
        const row = new Adw.SwitchRow({title: name, active: rule.modifiers.includes(name)});
        switches[name] = row;
        box.append(row);
    });
    toolbar.set_content(box);
    dialog.set_child(toolbar);
    apply.connect('clicked', () => {
        const nextDirection = DIRECTIONS[direction.selected];
        if (rules.some(item => item !== rule && item.direction === nextDirection)) {
            const alert = new Adw.AlertDialog({
                heading: '这个方向已经有规则',
                body: '先删掉旧规则，或者改成另一个方向。',
            });
            alert.add_response('ok', '好');
            alert.present(parent);
            return;
        }
        rule.direction = nextDirection;
        rule.key = KEYS[key.selected];
        rule.modifiers = MODIFIERS.filter(name => switches[name].active);
        done();
        dialog.close();
    });
    dialog.present(parent);
}

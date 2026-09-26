import * as Config from 'resource:///org/gnome/shell/misc/config.js';

// 扩展从 Shell 45 起改成模块导入。45 到 50 的移植说明里，
// 本扩展用到的入口、DrawingArea 和 get_pointer 没有不兼容改动。
export const SHELL_MAJOR_MIN = 45;
export const SHELL_MAJOR_MAX = 50;

export function shellVersion() {
    return Config.PACKAGE_VERSION;
}

export function shellMajor() {
    const [major] = String(Config.PACKAGE_VERSION).split('.');
    return Number(major);
}

export function shellSupported() {
    const major = shellMajor();
    return major >= SHELL_MAJOR_MIN && major <= SHELL_MAJOR_MAX;
}

// 45–50 都是 [x, y, modifiers]。缺第三项时当成没有修饰键。
export function readPointer() {
    const reading = global.get_pointer();
    return {
        x: Number(reading?.[0] ?? 0),
        y: Number(reading?.[1] ?? 0),
        mods: Number(reading?.[2] ?? 0),
    };
}

// 有的版本返回 [最小, 自然]，有的版本返回一个数。量不到就用估算。
export function preferredSize(widget, fallbackWidth, fallbackHeight) {
    try {
        const widthResult = widget.get_preferred_width(-1);
        const minWidth = Array.isArray(widthResult) ? widthResult[0] : widthResult;
        const heightResult = widget.get_preferred_height(minWidth > 0 ? minWidth : -1);
        const minHeight = Array.isArray(heightResult) ? heightResult[0] : heightResult;
        return {
            width: minWidth > 0 ? minWidth : fallbackWidth,
            height: minHeight > 0 ? minHeight : fallbackHeight,
        };
    } catch {
        return {width: fallbackWidth, height: fallbackHeight};
    }
}

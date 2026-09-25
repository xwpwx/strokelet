import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

const path = GLib.getenv('STROKELET_TEST_SOCK') ?? '/tmp/strokelet-gjs-socket.sock';
try {
    Gio.File.new_for_path(path).delete(null);
} catch {
    /* absent */
}

const service = new Gio.SocketService();
service.add_address(
    Gio.UnixSocketAddress.new(path),
    Gio.SocketType.STREAM,
    Gio.SocketProtocol.DEFAULT,
    null,
);

const loop = GLib.MainLoop.new(null, false);
service.connect('incoming', (_service, connection) => {
    const input = Gio.DataInputStream.new(connection.get_input_stream());
    const [line] = input.read_line_utf8(null);
    const message = JSON.parse(line);
    if (message.type !== 'ready' || message.version !== 1)
        throw new Error(`unexpected ${line}`);
    connection.get_output_stream().write_bytes(
        new GLib.Bytes('{"type":"hello","version":1}\n'),
        null,
    );
    connection.close(null);
    loop.quit();
});

const client = Gio.SocketClient.new();
const stream = client.connect(Gio.UnixSocketAddress.new(path), null);
stream.get_output_stream().write_bytes(new GLib.Bytes('{"type":"ready","version":1}\n'), null);
GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
    const input = Gio.DataInputStream.new(stream.get_input_stream());
    const [line] = input.read_line_utf8(null);
    const message = JSON.parse(line);
    if (message.type !== 'hello' || message.version !== 1)
        throw new Error(`unexpected ${line}`);
    stream.close(null);
    return GLib.SOURCE_REMOVE;
});
loop.run();
service.stop();
Gio.File.new_for_path(path).delete(null);
print('strokelet: gjs unix socket round-trip ok\n');

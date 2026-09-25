import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

export class DemoTransport {
    constructor(path, onMessage, onDisconnect) {
        this._path = path;
        this._onMessage = onMessage;
        this._onDisconnect = onDisconnect;
        this._connection = null;
        this._input = null;
        this._output = null;
        this._source = 0;
        this._buf = '';
    }

    connect() {
        try {
            const client = Gio.SocketClient.new();
            const address = Gio.UnixSocketAddress.new(this._path);
            this._connection = client.connect(address, null);
            this._output = this._connection.get_output_stream();
            this._input = Gio.DataInputStream.new(this._connection.get_input_stream());
            this._source = this._input.base_stream.create_source(null);
            this._source.set_callback(() => this._read());
            this._source.attach(null);
        } catch (error) {
            log(`strokelet: socket connect failed: ${error}`);
            this._fail();
        }
    }

    send(message) {
        if (!this._output)
            return;
        try {
            this._output.write_bytes(new GLib.Bytes(`${JSON.stringify(message)}\n`), null);
        } catch (error) {
            log(`strokelet: socket write failed: ${error}`);
            this._fail();
        }
    }

    disconnect() {
        this._close();
    }

    _read() {
        try {
            const [bytes] = this._input.read_line_utf8(null);
            if (bytes === null) {
                this._fail();
                return GLib.SOURCE_REMOVE;
            }
            const message = JSON.parse(bytes);
            this._onMessage(message);
            return GLib.SOURCE_CONTINUE;
        } catch (error) {
            log(`strokelet: socket read failed: ${error}`);
            this._fail();
            return GLib.SOURCE_REMOVE;
        }
    }

    _fail() {
        this._close();
        this._onDisconnect();
    }

    _close() {
        if (this._source) {
            this._source.destroy();
            this._source = 0;
        }
        this._connection?.close(null);
        this._connection = null;
        this._input = null;
        this._output = null;
    }
}

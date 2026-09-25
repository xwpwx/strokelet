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
        this._retryId = 0;
        this._stopped = false;
        this._loggedRetry = false;
    }

    connect() {
        this._stopped = false;
        this._attempt();
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
        this._stopped = true;
        if (this._retryId) {
            GLib.source_remove(this._retryId);
            this._retryId = 0;
        }
        this._close();
    }

    _attempt() {
        if (this._stopped || this._connection)
            return;
        try {
            const client = Gio.SocketClient.new();
            const address = Gio.UnixSocketAddress.new(this._path);
            this._connection = client.connect(address, null);
            this._output = this._connection.get_output_stream();
            this._input = Gio.DataInputStream.new(this._connection.get_input_stream());
            this._source = this._input.base_stream.create_source(null);
            this._source.set_callback(() => this._read());
            this._source.attach(null);
            this._loggedRetry = false;
        } catch (error) {
            if (!this._loggedRetry) {
                log(`strokelet: waiting for socket ${this._path}: ${error}`);
                this._loggedRetry = true;
            }
            this._schedule();
        }
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
        const wasConnected = this._connection !== null;
        this._close();
        if (wasConnected)
            this._onDisconnect();
        this._schedule();
    }

    _schedule() {
        if (this._stopped || this._retryId)
            return;
        this._retryId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
            this._retryId = 0;
            this._attempt();
            return GLib.SOURCE_REMOVE;
        });
    }

    _close() {
        if (this._source) {
            this._source.destroy();
            this._source = 0;
        }
        try {
            this._connection?.close(null);
        } catch {
            /* already closed */
        }
        this._connection = null;
        this._input = null;
        this._output = null;
    }
}

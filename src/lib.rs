mod capture;
mod config;
mod copy;
mod copy_gate;
mod gesture;
mod input;
mod protocol;
mod runtime;
mod server;
mod session;
mod stroke;

pub use capture::{CaptureUpdate, ChordCapture};
pub use config::{
    ConfigError, GestureConfig, MouseButton, TriggerButton, config_path, device_path, parse_config,
    write_config,
};
pub use copy::{
    Chord, CopyOutput, EmitError, KeySink, OutputEvent, chord_device_codes, evdev_from_gtk_keycode,
    key_names,
};
pub use copy_gate::{CopyGate, DesktopState, Injection, Modifiers, Observation, SessionState};
pub use gesture::{Decision, Direction, Gesture, Limits, WheelDirection};
pub use input::{
    DeviceProfile, DeviceReject, FrameProcessor, ListedMouse, PickError, classify_device,
    event_tuple, grab_allowed, key_event, pick_mouse, rel_event, syn_dropped, syn_report,
    virtual_mouse_codes,
};
pub use protocol::{
    CancelReason, ClientUpdate, HEALTH_TIMEOUT_MS, LineCodec, Link, MAX_LINE_BYTES, Outcome,
    PING_INTERVAL_MS, PROTOCOL_VERSION, PollAction, ProtocolError, STATE_TTL_MS, ServerLine,
};
pub use runtime::InjectionLedger;
pub use server::{
    PathError, PathFacts, RUNTIME_DIR_MODE, SOCKET_MODE, current_uid, facts_from_paths,
    peer_is_target, peer_uid, validate_runtime_path,
};
pub use session::{SessionFacts, is_active_unlocked, session_state};
pub use stroke::{ABSOLUTE_MAX_RULES, MAX_RULES, best_match, conflicts, straight_points};

pub fn status_message() -> &'static str {
    "strokelet: demo"
}

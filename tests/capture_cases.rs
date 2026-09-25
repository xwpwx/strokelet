use evdev::KeyCode;
use strokelet::{CaptureUpdate, ChordCapture};

#[test]
fn ctrl_alt_t_is_recorded_from_presses() {
    let mut capture = ChordCapture::new();
    assert!(matches!(
        capture.down(KeyCode::KEY_LEFTCTRL.0),
        Some(CaptureUpdate::Live(label)) if label == "Ctrl"
    ));
    assert!(matches!(
        capture.down(KeyCode::KEY_LEFTALT.0),
        Some(CaptureUpdate::Live(label)) if label == "Ctrl+Alt"
    ));
    capture.down(KeyCode::KEY_T.0);
    let finished = capture.up(KeyCode::KEY_T.0);
    match finished {
        Some(CaptureUpdate::Finished(chord)) => {
            assert_eq!(
                chord.modifier_codes(),
                &[KeyCode::KEY_LEFTCTRL.0, KeyCode::KEY_LEFTALT.0]
            );
            assert_eq!(chord.key_code(), KeyCode::KEY_T.0);
            assert_eq!(chord.key_name(), "Ctrl+Alt+T");
        }
        other => panic!("expected a chord, got {other:?}"),
    }
}

#[test]
fn modifier_released_before_the_main_key_stays_in_the_chord() {
    let mut capture = ChordCapture::new();
    capture.down(KeyCode::KEY_LEFTCTRL.0);
    capture.down(KeyCode::KEY_C.0);
    capture.up(KeyCode::KEY_LEFTCTRL.0);
    match capture.up(KeyCode::KEY_C.0) {
        Some(CaptureUpdate::Finished(chord)) => {
            assert_eq!(chord.modifier_codes(), &[KeyCode::KEY_LEFTCTRL.0]);
            assert_eq!(chord.key_code(), KeyCode::KEY_C.0);
        }
        other => panic!("expected a chord, got {other:?}"),
    }
}

#[test]
fn escape_cancels_and_a_second_key_is_rejected() {
    let mut escape = ChordCapture::new();
    assert!(matches!(
        escape.down(KeyCode::KEY_ESC.0),
        Some(CaptureUpdate::Cancelled(message)) if message == "已取消"
    ));

    let mut two = ChordCapture::new();
    two.down(KeyCode::KEY_A.0);
    assert!(matches!(
        two.down(KeyCode::KEY_B.0),
        Some(CaptureUpdate::Cancelled(_))
    ));
}

#[test]
fn key_repeat_does_not_count_as_a_second_press() {
    let mut capture = ChordCapture::new();
    capture.down(KeyCode::KEY_LEFTCTRL.0);
    assert!(capture.down(KeyCode::KEY_LEFTCTRL.0).is_none());
    capture.down(KeyCode::KEY_T.0);
    assert!(capture.down(KeyCode::KEY_T.0).is_none());
}

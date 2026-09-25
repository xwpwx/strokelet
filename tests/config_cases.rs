use evdev::KeyCode;

use strokelet::{
    Chord, CopyOutput, Direction, EmitError, GestureConfig, KeySink, OutputEvent, TriggerButton,
    chord_device_codes, evdev_from_gtk_keycode, key_names, parse_config, straight_points,
    write_config,
};

#[test]
fn default_rule_is_right_button_up_ctrl_c() {
    let config = GestureConfig::builtin_default();
    assert_eq!(config.trigger, TriggerButton::Right);
    let chord = config
        .chord_for_points(&straight_points(Direction::Up))
        .unwrap();
    assert_eq!(chord.key_name(), "c");
    assert_eq!(chord.modifier_codes(), &[evdev::KeyCode::KEY_LEFTCTRL.0]);
}

#[test]
fn shift_is_pressed_after_ctrl() {
    let chord = Chord::parse(&["shift".into(), "ctrl".into()], "v").unwrap();
    assert_eq!(chord.key_name(), "v");
    assert!(chord.modifier_codes()[0] < chord.modifier_codes()[1]);
}

#[test]
fn unknown_key_and_duplicate_direction_are_rejected() {
    assert!(Chord::parse(&[], "volume").is_err());
    let text = r#"{
        "version": 1,
        "trigger": "right",
        "rules": [
            {"direction": "up", "modifiers": ["ctrl"], "key": "c"},
            {"direction": "up", "modifiers": ["ctrl"], "key": "v"}
        ]
    }"#;
    assert!(parse_config(text).is_err());
}

#[test]
fn screen_name_is_optional_and_trimmed() {
    let named = parse_config(
        r#"{
        "version": 1,
        "trigger": "right",
        "rules": [{"direction": "up", "modifiers": ["ctrl"], "key": "c", "name": " 复制 "}]
    }"#,
    )
    .unwrap();
    let rule = named
        .rule_for_points(&straight_points(Direction::Up))
        .unwrap();
    assert_eq!(rule.screen_name, "复制");

    let blank = parse_config(
        r#"{
        "version": 1,
        "trigger": "right",
        "rules": [{"direction": "up", "modifiers": ["ctrl"], "key": "c", "name": "  "}]
    }"#,
    )
    .unwrap();
    assert_eq!(
        blank
            .rule_for_points(&straight_points(Direction::Up))
            .unwrap()
            .screen_name,
        ""
    );
    assert!(parse_config(
        r#"{
        "version": 1,
        "trigger": "right",
        "rules": [{"direction": "up", "modifiers": ["ctrl"], "key": "c", "name": "复制\n"}]
    }"#
    )
    .is_err());
    assert!(parse_config(
        r#"{
        "version": 1,
        "trigger": "right",
        "rules": [{"direction": "up", "modifiers": ["ctrl"], "key": "c", "name": "一二三四五六七八九十一二三四五六七"}]
    }"#
    )
    .is_err());
}

#[test]
fn saved_file_loads_back() {
    let path = std::env::temp_dir().join(format!("strokelet-config-{}.json", std::process::id()));
    let config = GestureConfig::builtin_default();
    write_config(&path, &config).unwrap();
    let loaded = GestureConfig::load(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(loaded, config);
}

#[test]
fn every_allowlisted_key_parses() {
    for name in key_names() {
        assert!(Chord::parse(&[], name).is_ok(), "{name}");
    }
}

#[test]
fn gtk_keycode_is_eight_above_evdev() {
    assert_eq!(evdev_from_gtk_keycode(54), Some(KeyCode::KEY_C.0));
    assert_eq!(evdev_from_gtk_keycode(37), Some(KeyCode::KEY_LEFTCTRL.0));
    assert_eq!(evdev_from_gtk_keycode(50), Some(KeyCode::KEY_LEFTSHIFT.0));
}

#[test]
fn recorded_space_and_right_ctrl_stay_distinct() {
    let chord = Chord::from_codes(
        &[KeyCode::KEY_RIGHTCTRL.0, KeyCode::KEY_LEFTSHIFT.0],
        KeyCode::KEY_SPACE.0,
        "Shift+Right Ctrl+space",
    )
    .unwrap();
    assert_eq!(
        chord.modifier_codes(),
        &[KeyCode::KEY_LEFTSHIFT.0, KeyCode::KEY_RIGHTCTRL.0]
    );
    assert_ne!(chord.modifier_codes()[1], KeyCode::KEY_LEFTCTRL.0);
    let mut output = CopyOutput::new(VecSink { events: Vec::new() });
    output.send_chord(&chord).unwrap();
    assert_eq!(
        output.sink().events[0],
        OutputEvent::Key {
            code: KeyCode::KEY_LEFTSHIFT.0,
            down: true
        }
    );
    assert_eq!(
        output.sink().events[2],
        OutputEvent::Key {
            code: KeyCode::KEY_RIGHTCTRL.0,
            down: true
        }
    );
    assert_eq!(
        output.sink().events[4],
        OutputEvent::Key {
            code: KeyCode::KEY_SPACE.0,
            down: true
        }
    );
    assert!(Chord::from_codes(&[], KeyCode::KEY_COMMA.0, "comma").is_ok());
}

#[test]
fn machine_mouse_and_empty_shortcuts_are_rejected() {
    assert!(Chord::from_codes(&[], KeyCode::KEY_SYSRQ.0, "sysrq").is_err());
    assert!(Chord::from_codes(&[], KeyCode::KEY_POWER.0, "power").is_err());
    assert!(Chord::from_codes(&[], KeyCode::KEY_SLEEP.0, "sleep").is_err());
    assert!(Chord::from_codes(&[], KeyCode::KEY_WAKEUP.0, "wake").is_err());
    assert!(Chord::from_codes(&[], KeyCode::BTN_LEFT.0, "click").is_err());
    assert!(Chord::from_codes(&[], KeyCode::KEY_C.0, "").is_err());
    let five = [
        KeyCode::KEY_LEFTCTRL.0,
        KeyCode::KEY_RIGHTCTRL.0,
        KeyCode::KEY_LEFTSHIFT.0,
        KeyCode::KEY_RIGHTSHIFT.0,
        KeyCode::KEY_LEFTALT.0,
    ];
    assert!(Chord::from_codes(&five, KeyCode::KEY_C.0, "too many").is_err());
}

#[test]
fn legacy_names_and_recorded_codes_both_load() {
    let legacy = parse_config(
        r#"{
        "version": 1,
        "trigger": "right",
        "rules": [{"direction": "up", "modifiers": ["ctrl"], "key": "c"}]
    }"#,
    )
    .unwrap();
    let chord = legacy
        .chord_for_points(&straight_points(Direction::Up))
        .unwrap();
    assert_eq!(chord.modifier_codes(), &[KeyCode::KEY_LEFTCTRL.0]);
    assert_eq!(chord.key_code(), KeyCode::KEY_C.0);

    let recorded = parse_config(
        r#"{
        "version": 1,
        "trigger": "right",
        "rules": [{"direction": "down", "modifierCodes": [97], "keyCode": 57, "label": "Right Ctrl+space"}]
    }"#,
    )
    .unwrap();
    let space = recorded
        .chord_for_points(&straight_points(Direction::Down))
        .unwrap();
    assert_eq!(space.modifier_codes(), &[KeyCode::KEY_RIGHTCTRL.0]);
    assert_eq!(space.key_code(), KeyCode::KEY_SPACE.0);
}

#[test]
fn virtual_keyboard_omits_machine_and_mouse_keys() {
    let codes = chord_device_codes();
    assert!(codes.contains(&KeyCode::KEY_COMMA.0));
    assert!(codes.contains(&KeyCode::KEY_SPACE.0));
    assert!(codes.contains(&KeyCode::KEY_RIGHTCTRL.0));
    for banned in [
        KeyCode::KEY_SYSRQ,
        KeyCode::KEY_POWER,
        KeyCode::KEY_SLEEP,
        KeyCode::KEY_WAKEUP,
        KeyCode::KEY_POWER2,
        KeyCode::KEY_SUSPEND,
        KeyCode::BTN_LEFT,
    ] {
        assert!(!codes.contains(&banned.0), "{banned:?}");
    }
}

struct VecSink {
    events: Vec<OutputEvent>,
}

impl KeySink for VecSink {
    fn emit(&mut self, event: OutputEvent) -> Result<(), EmitError> {
        self.events.push(event);
        Ok(())
    }
}

#[test]
fn broken_json_is_not_an_empty_table() {
    assert!(parse_config("{").is_err());
    assert!(parse_config(r#"{"version": 9, "trigger": "right", "rules": []}"#).is_err());
}

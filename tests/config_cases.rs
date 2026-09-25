use strokelet::{
    Chord, Direction, GestureConfig, TriggerButton, key_names, parse_config, write_config,
};

#[test]
fn default_rule_is_right_button_up_ctrl_c() {
    let config = GestureConfig::builtin_default();
    assert_eq!(config.trigger, TriggerButton::Right);
    let chord = config.chord_for(Direction::Up).unwrap();
    assert_eq!(chord.key_name(), "c");
    assert_eq!(chord.modifiers().len(), 1);
}

#[test]
fn shift_is_pressed_after_ctrl() {
    let chord = Chord::parse(&["shift".into(), "ctrl".into()], "v").unwrap();
    assert_eq!(chord.key_name(), "v");
    assert!(chord.modifiers()[0].code() < chord.modifiers()[1].code());
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
fn broken_json_is_not_an_empty_table() {
    assert!(parse_config("{").is_err());
    assert!(parse_config(r#"{"version": 9, "trigger": "right", "rules": []}"#).is_err());
}

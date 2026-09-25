use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{Chord, Direction};

pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerButton {
    Right,
    Middle,
    Forward,
    Back,
}

impl TriggerButton {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "right" => Some(TriggerButton::Right),
            "middle" => Some(TriggerButton::Middle),
            "forward" => Some(TriggerButton::Forward),
            "back" => Some(TriggerButton::Back),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TriggerButton::Right => "right",
            TriggerButton::Middle => "middle",
            TriggerButton::Forward => "forward",
            TriggerButton::Back => "back",
        }
    }

    pub fn evdev_code(self) -> u16 {
        match self {
            TriggerButton::Right => evdev::KeyCode::BTN_RIGHT.0,
            TriggerButton::Middle => evdev::KeyCode::BTN_MIDDLE.0,
            TriggerButton::Forward => evdev::KeyCode::BTN_FORWARD.0,
            TriggerButton::Back => evdev::KeyCode::BTN_BACK.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GestureConfig {
    pub trigger: TriggerButton,
    pub rules: Vec<(Direction, Chord)>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Invalid(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(message) | ConfigError::Invalid(message) => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl GestureConfig {
    pub fn builtin_default() -> Self {
        Self {
            trigger: TriggerButton::Right,
            rules: vec![(
                Direction::Up,
                Chord::parse(&["ctrl".to_string()], "c").expect("default chord is valid"),
            )],
        }
    }

    pub fn chord_for(&self, direction: Direction) -> Option<&Chord> {
        self.rules
            .iter()
            .find(|(rule_direction, _)| *rule_direction == direction)
            .map(|(_, chord)| chord)
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::builtin_default());
        }
        let text = fs::read_to_string(path).map_err(|err| ConfigError::Io(err.to_string()))?;
        parse_config(&text).map_err(ConfigError::Invalid)
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        });
    base.join("strokelet").join("gestures.json")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDto {
    version: u32,
    trigger: String,
    rules: Vec<RuleDto>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleDto {
    direction: String,
    modifiers: Vec<String>,
    key: String,
}

pub fn parse_config(text: &str) -> Result<GestureConfig, String> {
    let file: FileDto = serde_json::from_str(text).map_err(|err| err.to_string())?;
    if file.version != CONFIG_VERSION {
        return Err(format!("unsupported config version {}", file.version));
    }
    let trigger = TriggerButton::parse(&file.trigger)
        .ok_or_else(|| format!("unknown trigger {}", file.trigger))?;
    let mut rules = Vec::new();
    for rule in file.rules {
        let direction = parse_direction(&rule.direction)?;
        if rules.iter().any(|(existing, _)| *existing == direction) {
            return Err(format!("duplicate direction {}", rule.direction));
        }
        if rule.key.is_empty() {
            return Err("shortcut key is empty".into());
        }
        let chord = Chord::parse(&rule.modifiers, &rule.key)?;
        rules.push((direction, chord));
    }
    Ok(GestureConfig { trigger, rules })
}

fn parse_direction(name: &str) -> Result<Direction, String> {
    match name {
        "up" => Ok(Direction::Up),
        "down" => Ok(Direction::Down),
        "left" => Ok(Direction::Left),
        "right" => Ok(Direction::Right),
        _ => Err(format!("unknown direction {name}")),
    }
}

pub fn write_config(path: &Path, config: &GestureConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ConfigError::Io(err.to_string()))?;
    }
    let mut rules = Vec::new();
    for (direction, chord) in &config.rules {
        rules.push(serde_json::json!({
            "direction": direction_name(*direction),
            "modifiers": chord.modifiers().iter().map(|modifier| match modifier {
                crate::Modifier::Ctrl => "ctrl",
                crate::Modifier::Shift => "shift",
                crate::Modifier::Alt => "alt",
                crate::Modifier::Super => "super",
            }).collect::<Vec<_>>(),
            "key": chord.key_name(),
        }));
    }
    let body = serde_json::json!({
        "version": CONFIG_VERSION,
        "trigger": config.trigger.name(),
        "rules": rules,
    });
    let text =
        serde_json::to_string_pretty(&body).map_err(|err| ConfigError::Invalid(err.to_string()))?;
    fs::write(path, text + "\n").map_err(|err| ConfigError::Io(err.to_string()))
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Up => "up",
        Direction::Down => "down",
        Direction::Left => "left",
        Direction::Right => "right",
    }
}

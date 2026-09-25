use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::stroke::{MAX_RULES, best_match, conflicts, straight_points};
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeRule {
    pub points: Vec<(f64, f64)>,
    pub chord: Chord,
    /// 识别并执行后显示在屏幕上的名字。空字符串表示不显示。
    pub screen_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GestureConfig {
    pub trigger: TriggerButton,
    pub rules: Vec<StrokeRule>,
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
            rules: vec![StrokeRule {
                points: straight_points(Direction::Up),
                chord: Chord::parse(&["ctrl".to_string()], "c").expect("default chord is valid"),
                screen_name: String::new(),
            }],
        }
    }

    pub fn chord_for_points(&self, points: &[(f64, f64)]) -> Option<&Chord> {
        self.rule_for_points(points).map(|rule| &rule.chord)
    }

    pub fn rule_for_points(&self, points: &[(f64, f64)]) -> Option<&StrokeRule> {
        let templates: Vec<_> = self.rules.iter().map(|rule| rule.points.clone()).collect();
        best_match(&templates, points).map(|index| &self.rules[index])
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
    #[serde(default)]
    direction: String,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    modifiers: Vec<String>,
    #[serde(default)]
    key: String,
    #[serde(default, rename = "modifierCodes")]
    modifier_codes: Vec<u16>,
    #[serde(default, rename = "keyCode")]
    key_code: Option<u16>,
    #[serde(default)]
    label: String,
    #[serde(default)]
    name: String,
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
        if rules.len() >= MAX_RULES {
            return Err(format!("at most {MAX_RULES} strokes"));
        }
        let points = points_from_rule(&rule)?;
        if rules
            .iter()
            .any(|existing: &StrokeRule| conflicts(&existing.points, &points))
        {
            return Err("这条轨迹和已有的太像".into());
        }
        let chord = chord_from_rule(&rule)?;
        let screen_name = screen_name_from(&rule.name)?;
        rules.push(StrokeRule {
            points,
            chord,
            screen_name,
        });
    }
    Ok(GestureConfig { trigger, rules })
}

fn points_from_rule(rule: &RuleDto) -> Result<Vec<(f64, f64)>, String> {
    let has_points = !rule.points.is_empty();
    let has_direction = !rule.direction.is_empty();
    if has_points && has_direction {
        return Err("stroke uses both a direction and points".into());
    }
    if has_direction {
        let direction = parse_direction(&rule.direction)?;
        return Ok(straight_points(direction));
    }
    if rule.points.len() < 2 {
        return Err("stroke needs at least two points".into());
    }
    if rule.points.len() > 2048 {
        return Err("stroke has too many points".into());
    }
    let mut points = Vec::with_capacity(rule.points.len());
    for point in &rule.points {
        if !point[0].is_finite()
            || !point[1].is_finite()
            || point[0].abs() > 100_000.0
            || point[1].abs() > 100_000.0
        {
            return Err("stroke point is out of range".into());
        }
        points.push((point[0], point[1]));
    }
    Ok(points)
}

fn chord_from_rule(rule: &RuleDto) -> Result<Chord, String> {
    let has_names = !rule.modifiers.is_empty() || !rule.key.is_empty();
    let has_codes = rule.key_code.is_some() || !rule.modifier_codes.is_empty();
    if has_names && has_codes {
        return Err("shortcut uses both names and key codes".into());
    }
    if let Some(key_code) = rule.key_code {
        let label = if rule.label.is_empty() {
            key_code.to_string()
        } else {
            rule.label.clone()
        };
        return Chord::from_codes(&rule.modifier_codes, key_code, &label);
    }
    if rule.key.is_empty() {
        return Err("shortcut key is empty".into());
    }
    Chord::parse(&rule.modifiers, &rule.key)
}

fn screen_name_from(raw: &str) -> Result<String, String> {
    if raw.chars().any(char::is_control) {
        return Err("屏幕名称不能换行".into());
    }
    let name = raw.trim();
    if name.chars().count() > 16 {
        return Err("屏幕名称最多 16 个字".into());
    }
    Ok(name.to_string())
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
    for rule in &config.rules {
        let mut value = serde_json::json!({
            "points": rule.points.iter().map(|(x, y)| [x, y]).collect::<Vec<_>>(),
            "modifierCodes": rule.chord.modifier_codes(),
            "keyCode": rule.chord.key_code(),
            "label": rule.chord.key_name(),
        });
        if !rule.screen_name.is_empty() {
            value["name"] = serde_json::Value::String(rule.screen_name.clone());
        }
        rules.push(value);
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

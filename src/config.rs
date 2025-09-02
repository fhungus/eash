use crate::{
    element::{BasicElement, ElementType},
    error::EASHError,
    misc_types::{Alignment, Color, HexColor, Spring, VisualState, Width},
};
use serde::Deserialize;
use std::{fs, str::FromStr};

#[derive(Deserialize)]
pub struct Config {
    pub chain_elements: Vec<ConfigElement>,
    pub spring: SpringConfig,
}

// clone trait of shame...
#[derive(Deserialize, Clone)]
pub struct SpringConfig {
    pub spacing: u16,
    pub constant: f32,
    pub dampening: f32,
}

// feels superfluous...
impl From<SpringConfig> for Spring {
    fn from(sc: SpringConfig) -> Spring {
        Spring {
            spacing: sc.spacing,
            constant: sc.constant,
            dampening: sc.dampening
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConfigElement {
    BasicElement {
        content: String,
        visual_state: ConfigVisualState,
    },
    Prompt,
}

#[derive(Deserialize, Clone)]
pub struct ConfigVisualState {
    pub align: String,
    pub width: String,
    pub padding: u32,
    pub bg_color: ConfigColor,
    pub color: ConfigColor,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConfigColor {
    Transparent,
    Solid { r: u8, g: u8, b: u8 },
    Gradient { from: HexColor, to: HexColor },
}

impl From<ConfigColor> for Color {
    fn from(value: ConfigColor) -> Self {
        match value {
            ConfigColor::Transparent => Color::Transparent,
            ConfigColor::Solid { r, g, b } => Color::Solid(HexColor { r, g, b }),
            ConfigColor::Gradient { from, to } => Color::Gradient(from, to),
        }
    }
}

pub fn file_to_config(filepath: String) -> Result<Config, EASHError> {
    let contents = fs::read_to_string(filepath)?;
    let config: Config = toml::from_str(&contents)
        .map_err(|e| EASHError::ConfigSyntaxError(e.message().to_string()))?;
    Ok(config)
}

impl FromStr for Alignment {
    type Err = EASHError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(Alignment::Left),
            "center" => Ok(Alignment::Center),
            "right" => Ok(Alignment::Right),
            _ => Err(Self::Err::ConfigInvalidType {
                expected: "Alignment",
                got: s.to_string(),
            }),
        }
    }
}

fn bracket_match(s: &str) -> Result<Option<(String, String)>, EASHError> {
    let s = s.trim();
    let Some(open_paren_pos) = s.find('(') else {
        return Ok(None);
    };

    let Some(close_paren_pos_rel) = s[open_paren_pos..].find(')') else {
        return Err(EASHError::ConfigMalformedBracket(s.to_string()));
    };
    let close_paren_pos = open_paren_pos + close_paren_pos_rel;

    // Check for another '(' between the first one and the closing one.
    if s[open_paren_pos + 1..close_paren_pos].contains('(') {
        return Err(EASHError::ConfigMalformedBracket(s.to_string()));
    }

    let outside = s[..open_paren_pos].trim().to_string();
    let inside = s[open_paren_pos + 1..close_paren_pos].trim().to_string();

    Ok(Some((outside, inside)))
}

impl FromStr for Width {
    type Err = EASHError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (variant, value_string) =
            bracket_match(s)?.ok_or_else(|| EASHError::ConfigInvalidType {
                expected: "width",
                got: s.to_string(),
            })?;
        let value = value_string
            .parse::<u32>()
            .map_err(|_| EASHError::ConfigInvalidType {
                expected: "integer",
                got: value_string,
            })?;

        match variant.as_str() {
            "Minimum" => Ok(Self::Minimum(value)),
            "Units" => Ok(Self::Units(value)),
            _ => Err(EASHError::ConfigInvalidType {
                expected: "width type",
                got: variant,
            }),
        }
    }
}

impl TryFrom<ConfigVisualState> for VisualState {
    type Error = EASHError;

    fn try_from(value: ConfigVisualState) -> Result<Self, Self::Error> {
        Ok(VisualState {
            align: Alignment::from_str(&value.align)?,
            width: Width::from_str(&value.width)?,
            padding: value.padding,
            bg_color: value.bg_color.into(),
            color: value.color.into(),
        })
    }
}

pub fn find_config() -> Result<Option<String>, EASHError> {
    let config_dirs = vec![
        "./eash.toml",
        "./eash/eash.toml",
        "!/eash/eash.toml", // ! is a standin for the xdg_config_home env variable if it exists
        "!/eash.toml",
    ];

    let xdg_config_result = std::env::var("XDG_CONFIG_HOME");
    let ambatuborrow = &xdg_config_result;

    for i in config_dirs.iter() {
        let mut path: String;
        if i.starts_with("!") {
            if ambatuborrow.is_err() {
                continue;
            }
            let xdg_config_dir = ambatuborrow.clone().unwrap();

            path = xdg_config_dir.to_string();
            path.push_str(&i[1..]);
        } else {
            path = i.to_string();
        }

        if std::fs::exists(&path)? {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

    pub fn get_elements_from_config(config: &Config) -> Result<Vec<ElementType>, EASHError> {
        let mut elements: Vec<ElementType> = Vec::new();
        for i in config.chain_elements.iter() {
            match i {
                ConfigElement::BasicElement {
                    content,
                    visual_state,
                } => {
                    // MOST MEMORY DUPLICATING CODE AWARD: me 🎖️
                    let vs: VisualState = visual_state.clone().try_into()?;
                    elements.push(ElementType::BasicElement(BasicElement {
                        content: content.clone(),
                        visual_state: vs,
                    }));
                }
                ConfigElement::Prompt => {
                    return Err(EASHError::ConfigPromptUsed);
                }
            }
        }

        return Ok(elements);
    }

use std::time::Instant;

use crate::error::EASHError;
use crossterm::style::Color as ctColor;
use serde::Deserialize;

pub enum Direction {
    Left,
    Right,
}

#[derive(Deserialize, Clone)]
pub struct HexColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub enum Color {
    Transparent,
    Solid(HexColor),
    Gradient(HexColor, HexColor),
}

impl Color {
    pub fn is_gradient(&self) -> bool {
        matches!(self, Self::Gradient(_, _))
    }

    pub fn to_color_for_char(&self, distance: f32) -> ctColor {
        match self {
            Self::Transparent => self.to_flat_color().unwrap(),
            Self::Solid(_) => self.to_flat_color().unwrap(), // should be safe enough 👍
            Self::Gradient(f, t) => ctColor::Rgb {
                r: (f.r as f32 + ((t.r as i16 - f.r as i16) as f32 * distance)) as u8,
                g: (f.g as f32 + ((t.g as i16 - f.g as i16) as f32 * distance)) as u8,
                b: (f.b as f32 + ((t.b as i16 - f.b as i16) as f32 * distance)) as u8,
            },
        }
    }

    // i really only seperated these two functions because i was worried that having the gradient one just
    // shrug off probably bad behaviour could make debugging harder and is therefore probably a bad habit
    // for me to have.
    pub fn to_flat_color(&self) -> Result<ctColor, EASHError> {
        match self {
            Self::Gradient(_, _) => Err(EASHError::ColorNotFlat),
            Self::Solid(c) => Ok(ctColor::Rgb {
                r: c.r,
                g: c.g,
                b: c.b,
            }),
            Self::Transparent => Ok(ctColor::Reset),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Spring {
    pub spacing: u16,
    pub constant: f32,
    pub dampening: f32,
}

pub enum Alignment {
    Left,
    Center,
    Right,
}

pub enum Width {
    Units(u32),
    Minimum(u32),
}

#[derive(Debug)]
pub enum Glyph {
    Single(char),
    Animated {
        characters: String,
        speed: f32, // in seconds
    },
}

impl Glyph {
    pub fn get_current_glyph(&self, then: &Instant) -> char {
        match self {
            Glyph::Single(c) => *c,
            Glyph::Animated { characters, speed } => {
                // probably could do this with less computation tbh
                let time_since_then = then.elapsed().as_millis();
                let speed_ms: u128;
                unsafe {
                    // is there a better way to do this? please?
                    speed_ms = (speed * 1000.0).floor().to_int_unchecked();
                };
                
                println!("{}", time_since_then.rem_euclid(speed_ms * characters.len() as u128));
                let index = (time_since_then % (speed_ms * characters.len() as u128)
                    / speed_ms / speed_ms) as usize;
                characters.chars().nth(index).unwrap_or('!')
            }
        }
    }
}

pub enum TriggerType {
    EveryFrame,
    PromptUpdate,
    Timed(f32), // seconds
}

pub struct VisualState {
    pub align: Alignment,
    pub width: Width,
    pub padding: u32,
    pub bg_color: Color,
    pub color: Color,
}

impl Default for VisualState {
    fn default() -> Self {
        Self {
            align: Alignment::Left,
            color: Color::Solid(HexColor {
                r: 255,
                g: 255,
                b: 255,
            }),
            padding: 1,
            bg_color: Color::Transparent,
            width: Width::Minimum(0),
        }
    }
}

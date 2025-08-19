use crossterm::style::Color as ctColor;

pub enum Direction {
    Left,
    Right,
}

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
    pub fn to_color_for_char(&self, distance: f32) -> ctColor {
        match self {
            Self::Transparent => ctColor::Reset,
            Self::Solid(c) => ctColor::Rgb {
                r: c.r,
                g: c.g,
                b: c.b,
            },
            Self::Gradient(f, t) => ctColor::Rgb {
                r: (f.r as f32 + ((t.r as i16 - f.r as i16) as f32 * distance)) as u8,
                g: (f.g as f32 + ((t.g as i16 - f.g as i16) as f32 * distance)) as u8,
                b: (f.b as f32 + ((t.b as i16 - f.b as i16) as f32 * distance)) as u8,
            },
        }
    }
}

pub enum Alignment {
    Left,
    Right,
}

pub enum Width {
    Units(u32),
    Minimum(u32),
}

// temporary until i figure out what a glyph should be
pub type Glyph = char;

pub fn process_print_width_as_unit(
    print_content: &str,
    u: u32,
    padding: u32,
    align: &Alignment,
) -> String {
    let next_print_content;
    // same thing again lol
    if (u as usize) > print_content.len() + (padding as usize * 2) {
        let spacing = " ".repeat((print_content.len() + (padding as usize * 2)) - u as usize);
        match align {
            Alignment::Left => next_print_content = format!("{}{}", print_content, spacing),
            Alignment::Right => next_print_content = format!("{}{}", spacing, print_content),
        }
    } else if (u as usize) < print_content.len() + (padding as usize * 2) {
        // if its too long, cut it down!!!
        // currently really easy to make crash and just kinda bad
        next_print_content = format!(
            "{}{}",
            (&print_content[0..(u as usize - 1) - (padding as usize * 2) - 3]).to_string(),
            "..."
        );
    } else {
        next_print_content = print_content.to_string()
    }

    return next_print_content;
}

pub fn process_print_width_as_min(
    print_content: &str,
    m: u32,
    padding: u32,
    align: &Alignment,
) -> String {
    // if its smaller add some spacing
    if (m as usize) > print_content.len() + (padding as usize * 2) {
        let spacing = " ".repeat(m as usize - (print_content.len() + (padding as usize * 2)));
        match align {
            Alignment::Left => return format!("{}{}", print_content, spacing),
            Alignment::Right => return format!("{}{}", spacing, print_content),
        }
    }
    return print_content.to_string();
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
        return Self {
            align: Alignment::Left,
            color: Color::Solid(HexColor {
                r: 255,
                g: 255,
                b: 255,
            }),
            padding: 1,
            bg_color: Color::Transparent,
            width: Width::Minimum(0),
        };
    }
}

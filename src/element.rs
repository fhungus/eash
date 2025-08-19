use crate::{
    error::EASHError,
    misc_types::{
        Direction, Glyph, VisualState, Width, process_print_width_as_min,
        process_print_width_as_unit,
    },
    prompt::Prompt,
};
use crossterm::{cursor::MoveToColumn, queue, style::ResetColor};
use std::io::Write;

pub enum TriggerType {
    EveryFrame,
    PromptUpdate,
    Timed(f32), // seconds
}

pub struct BasicElement {
    visual_state: VisualState,
    content: String,
}

pub struct ElementWithGlyph {
    visual_state: VisualState,
    element: Glyph,
    glyph: Glyph,
    direction: Direction,
}

pub enum ElementType {
    BasicElement(Glyph),
    ElementWithGlyph(ElementWithGlyph),
    Prompt(Prompt),
}

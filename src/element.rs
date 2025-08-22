use crate::{
    misc_types::{Direction, Glyph, VisualState},
    prompt::Prompt,
};
use std::sync::{Arc, Mutex};

pub struct BasicElement {
    pub visual_state: VisualState,
    pub content: String,
}

pub struct ElementWithGlyph {
    pub visual_state: VisualState,
    pub element: Glyph,
    pub glyph: Glyph,
    pub direction: Direction,
}

pub enum ElementType {
    BasicElement(BasicElement),
    // ElementWithGlyph(ElementWithGlyph),
    Prompt(Arc<Mutex<Prompt>>),
}

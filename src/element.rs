use crate::{
    error::EASHError,
    misc_types::{VisualState, Width, process_print_width_as_min, process_print_width_as_unit},
};
use crossterm::{cursor::MoveToColumn, queue, style::ResetColor};
use std::io::Write;

pub struct Element<W: Write + Send> {
    pub id: String,
    pub render: Box<dyn Fn(&mut W, u16) -> Result<(), EASHError> + Send>,
    pub get_width: Box<dyn Fn() -> u16 + Send>,
}

// good enough ig
#[macro_use]
macro_rules! new_element {
    ($id:literal, $r:item, $gw: item) => {
        Element {
            id: $id,
            render: Box::new($r),
            get_width: Box::new($gw),
        }
    };
}

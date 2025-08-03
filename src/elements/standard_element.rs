use crate::elements::Element;
use crate::misc_types::{
    VisualState, Width, process_print_width_as_min, process_print_width_as_unit,
};

use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{PrintStyledContent, ResetColor, Stylize},
};
use std::io::{Stdout, Write};

pub struct StandardElement {
    pub state: VisualState,
    pub content: String,
}

impl Element for StandardElement {
    fn get_width(&self) -> u32 {
        return match self.state.width {
            Width::Units(u) => u,
            Width::Minimum(m) => {
                (self.content.len() as u32 + (self.state.padding * 2)).clamp(m, u32::MAX)
            }
        };
    }

    fn render(&self, start_position: i32, w: &mut Stdout) {
        let terminal_position = if start_position >= 0 {
            start_position
        } else {
            0
        };
        _ = queue!(w, ResetColor, MoveToColumn(terminal_position as u16));

        let vs = &self.state;
        let mut print_content = self.content.clone();

        // width checking shennanigans
        print_content = match vs.width {
            Width::Minimum(m) => {
                process_print_width_as_min(&print_content, m, vs.padding, &vs.align)
            }
            Width::Units(u) => {
                process_print_width_as_unit(&print_content, u, vs.padding, &vs.align)
            }
        };

        // padding
        let padding = " ".repeat(vs.padding as usize);
        print_content = format!("{}{}{}", padding, print_content, padding);

        // We shouldn't NEED to do this if both color and background_color are solid, which they usually are.
        // [TODO] just print everything at once if both colors are solid
        let mut i = 0;
        for char in print_content.chars() {
            i = i + 1;
            if i + start_position < 0 {
                continue;
            }

            let distance = i as f32 / print_content.len() as f32;
            let styled_character = char
                .to_string()
                .with(vs.color.to_color_for_char(distance))
                .on(vs.bg_color.to_color_for_char(distance));

            _ = queue!(w, PrintStyledContent(styled_character));
        }

        w.flush()
            .expect("george orwell 1984 has FORBID me from flushing...");
    }
}

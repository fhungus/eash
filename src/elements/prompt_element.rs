use crate::elements::element_trait::Element;
use crate::prompt::Prompt;
use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::{PrintStyledContent, Stylize},
};
use std::io::{Stdout, Write};
use std::sync::{Arc, Mutex};

pub struct PromptElement {
    pub prompt: Arc<Mutex<Prompt>>,
}

impl Element for PromptElement {
    fn get_width(&self) -> u32 {
        let lock = self
            .prompt
            .lock()
            .expect("horrific tragedy occured while trying to unlock prompt mutex");
        return lock.prompt.len() as u32;
    }

    fn render(&self, start_position: i32, w: &mut Stdout) {
        let lock = self
            .prompt
            .lock()
            .expect("horrific tragedy occured while trying to unlock prompt mutex");

        for (i, character) in lock.prompt.chars().enumerate() {
            if start_position + (i as i32) < (0 as i32) {
                continue;
            }

            let mut formatted = character.to_string().stylize();
            // TODO: better, gradient supporting selection highlighting
            if lock.position_is_in_selection(i as u16) {
                formatted = formatted.on_white().black();
            }

            queue!(
                w,
                MoveToColumn(start_position as u16 + i as u16),
                PrintStyledContent(formatted)
            )
            .expect("Cannot print character");
        }

        // this won't work if we have any elements rendering after the prompt, this is a placeholder
        if start_position >= 0 {
            queue!(
                w,
                MoveToColumn(start_position as u16 + lock.cursor_position)
            )
            .expect("Cannot print MoveToColumn");
        }

        w.flush()
            .expect("george orwell 1984 has FORBID me from flushing...");
    }
}

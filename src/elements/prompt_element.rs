use crate::elements::element_trait::Element;
use crate::prompt::Prompt;
use crossterm::{
    queue,
    style::{PrintStyledContent, Stylize},
};
use std::io::{Stdout, Write};
use std::sync::{Arc, Mutex};

pub struct PromptElement {
    prompt: Arc<Mutex<Prompt>>,
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

            queue!(w, PrintStyledContent(formatted)).expect("Cannot print character");
        }

        w.flush()
            .expect("george orwell 1984 has FORBID me from flushing...");
    }
}

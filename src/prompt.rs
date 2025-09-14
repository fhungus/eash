use crossterm::style::Stylize;

use crate::misc_types::Direction;

pub struct Prompt {
    pub prompt: String,
    pub cursor_position: u16,
    pub selection_start: Option<u16>, // if None, then there is no selection
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            cursor_position: 0,
            selection_start: None,
        }
    }
}

impl Prompt {
    pub fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor_position);
    }

    // check if a selection even exists, and if so whether the position in the prompt is within aformentioned selection
    pub fn position_is_in_selection(&self, position: u16) -> bool {
        match self.selection_start {
            Some(start) => {
                if (self.cursor_position as i16) - (start as i16) >= 0 {
                    self.cursor_position >= position && start <= position
                } else {
                    start >= position && self.cursor_position <= position
                }
            }
            None => false,
        }
    }

    pub fn find_skippable_in_direction(&self, direction: Direction) -> u16 {
        let (l, r) = self.prompt.split_at_checked(self.cursor_position as usize).unwrap();
        match direction {
            Direction::Left => {
                let mut iter = l.chars().rev().enumerate();
                _ = iter.next();
                for (i, c) in iter {
                    // check if THIS character is "skippable", if it is, set cursor_pos and return here
                    match c {
                        '/' | ' ' | '.' | ',' | '\'' | '"' => { return self.cursor_position - (i as u16) },
                        _ => {}
                    }
                }
            }
            Direction::Right => {
                let mut iter = r.chars().enumerate();
                _ = iter.next();
                for (i, c) in iter {
                    // same thing.. duplication... ):
                    match c {
                        '/' | ' ' | '.' | ',' | '\'' | '"' => { return i as u16 + self.cursor_position },
                        _ => {}
                    }
                }
            }
        };

        // failed to find anything so return the start/end of the prompt
        return match direction {
            Direction::Left => 0,
            Direction::Right => self.prompt.len() as u16
        }
    }

    // move cursor forwards or backwards a "word" (actually a bit more than that)
    pub fn jump_in_direction(&mut self, direction: Direction) {
        let jump_to = self.find_skippable_in_direction(direction);
        self.cursor_position = jump_to;
    }

    // them funny skipping motions
    pub fn ctrl_backspace(&mut self) -> bool {
        let cut_position = self.find_skippable_in_direction(Direction::Left);
        if cut_position == self.cursor_position {
            return self.delete_character();
        };

        let left_side = &self.prompt[0..cut_position as usize];
        let right_side = &self.prompt[self.cursor_position as usize..];

        self.prompt = format!("{}{}", left_side, right_side);
        self.cursor_position = cut_position;

        return false;
    }

    // move the cursor in the direction, space times
    pub fn move_cursor(&mut self, space: u32, direction: Direction) {
        // would it be cursed if we could cast direction into a i16?
        let neg = match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };

        let new_position = self.cursor_position as i16 + (space as i16 * neg);
        if new_position <= 0 {
            self.cursor_position = 0;
            return;
        } else if new_position >= (self.prompt.len() as i16) {
            self.cursor_position = self.prompt.len() as u16;
            return;
        }

        self.cursor_position = new_position as u16
    }

    // handle left & right
    // returns whether to "bump" or not
    pub fn horiziontal_arrow(&mut self, direction: Direction, shift: bool, ctrl: bool) -> bool {
        if shift && self.selection_start.is_none() {
            self.start_selection();
        }

        let prev = self.cursor_position;
        if ctrl {
            self.jump_in_direction(direction);
        } else {
            self.move_cursor(1, direction);
        }

        return prev == self.cursor_position;
    }

    // insert a character at the cursors current position
    pub fn insert_character(&mut self, character: char) {
        // if we're at or past the end of the string just append
        if self.cursor_position >= self.prompt.len() as u16 {
            self.prompt.push(character);
            self.cursor_position = self.prompt.len() as u16;
        } else {
            self.prompt
                .insert((self.cursor_position) as usize, character);
            self.cursor_position += 1;
        }
    }

    // delete character at cursor position
    // returns whether to "bump" or not
    pub fn delete_character(&mut self) -> bool {
        if self.prompt.len() == 0 || self.cursor_position <= 0 {
            return true;
        }

        if self.cursor_position >= self.prompt.len() as u16 {
            self.prompt.pop();
        } else {
            self.prompt.remove((self.cursor_position - 1) as usize);
        }

        if self.cursor_position != 0 {
            self.cursor_position -= 1;
        }

        return false;
    }

    // delete whatever is selected at the current moment (will panic if there is no selection)
    // returns whether to "bump" or not
    pub fn delete_selection(&mut self) -> bool {
        let selection = self.selection_start.expect(
            "i should've checked whether a selection EXISTED before calling this function.",
        );

        // might need this more often
        let (smaller, bigger) = if selection > self.cursor_position {
            (self.cursor_position as usize, selection as usize)
        } else {
            (selection as usize, self.cursor_position as usize)
        };

        let mut bump = false;

        let first = if smaller > 0 {
            &self.prompt[0..smaller - 1]
        } else {
            bump = true;
            ""
        };

        let second = if bigger < self.prompt.len() {
            &self.prompt[bigger + 1..]
        } else {
            ""
        };

        self.selection_start = None;
        self.cursor_position = (smaller - 1) as u16;
        self.prompt = format!("{}{}", first, second);

        return bump;
    }

    // handles all backspace logic
    // returns whether to "bump" or not
    pub fn backspace(&mut self) -> bool {
        if self.selection_start.is_some() {
            return self.delete_selection();
        }

        if self.cursor_position <= 0 {
            return true;
        }

        self.delete_character();

        return false;
    }
}

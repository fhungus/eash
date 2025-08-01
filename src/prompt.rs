use crate::misc_types::Direction;

pub struct Prompt {
    pub cursor_position: u16,
    pub prompt: String,
    pub selection_start: Option<u16>, // if None, then there is no selection
}

impl Prompt {
    pub fn start_selection(&mut self) {
        self.selection_start = Some(self.cursor_position);
    }

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
        let increment = match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };

        let mut i = self.cursor_position as i16;
        loop {
            i = i + increment;
            if i <= 1 {
                return 1;
            }

            // check if THIS character is "skippable", if it is, set cursor_pos and return here
            // TODO: make this part support utf16, mostly just in case i need it in the future
            // TODO: (also) do this just generally better
            let bytes = self.prompt.as_bytes()[i as usize];
            if ' ' as u8 == bytes || '/' as u8 == bytes || '=' as u8 == bytes {
                return i as u16;
            }
        }
    }

    pub fn jump_in_direction(&mut self, direction: Direction) {
        let jump_to = self.find_skippable_in_direction(direction);
        self.cursor_position = jump_to;
    }

    pub fn ctrl_backspace(&mut self) {
        let cut_position = self.find_skippable_in_direction(Direction::Left);
        if cut_position == self.cursor_position {
            return;
        };

        let left_side = &self.prompt[0..cut_position as usize];
        let right_side = &self.prompt[self.cursor_position as usize..];

        self.prompt = format!("{}{}", left_side, right_side);
        self.cursor_position = cut_position;
    }

    pub fn move_cursor(&mut self, space: u32, direction: Direction) {
        // would it be cursed if we could cast direction into a i16?
        let neg = match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };

        let new_position = (self.cursor_position as i16 + (space as i16 * neg)) as u16;
        if new_position <= 1 {
            self.cursor_position = 1;
            return;
        } else if new_position >= self.prompt.len() as u16 {
            self.cursor_position = self.prompt.len() as u16;
        }
    }
}

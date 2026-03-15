use std::time::Instant;

use super::App;

impl App {
    /// Called when a digit key is pressed in normal mode.
    /// Two digits trigger an immediate jump; a single digit waits for the 1 s tick timeout.
    pub fn push_jump_digit(&mut self, digit: char) {
        self.jump_buf.push(digit);
        self.jump_at = Some(Instant::now());
        if self.jump_buf.len() == 2 {
            self.commit_jump();
        }
    }

    /// Execute the jump to the position stored in `jump_buf`, then clear it.
    pub fn commit_jump(&mut self) {
        if let Ok(n) = self.jump_buf.parse::<usize>() {
            if n > 0 && n <= self.sorted_indices.len() {
                self.selected = n - 1;
                self.list_state.select(Some(self.selected));
                self.ensure_data();
            }
        }
        self.jump_buf.clear();
        self.jump_at = None;
    }

    /// Cancel any in-progress jump without navigating.
    pub fn cancel_jump(&mut self) {
        self.jump_buf.clear();
        self.jump_at = None;
    }
}

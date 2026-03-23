use crate::time::Instant;

use super::App;

impl App {
    /// Called when a digit key is pressed in normal mode.
    pub fn push_jump_digit(&mut self, digit: char) {
        self.nav.jump_buf.push(digit);
        self.nav.jump_at = Some(Instant::now());
        if self.nav.jump_buf.len() == 2 || self.nav.sorted_indices.len() < 10 {
            self.commit_jump();
        }
    }

    /// Execute the jump to the position stored in `jump_buf`, then clear it.
    pub fn commit_jump(&mut self) {
        if let Ok(n) = self.nav.jump_buf.parse::<usize>() {
            if n > 0 && n <= self.nav.sorted_indices.len() {
                self.nav.selected = n - 1;
                self.update_nav_offset();
                self.nav.last_nav_at = Some(Instant::now());
            }
        }
        self.nav.jump_buf.clear();
        self.nav.jump_at = None;
    }

    /// Cancel any in-progress jump without navigating.
    pub fn cancel_jump(&mut self) {
        self.nav.jump_buf.clear();
        self.nav.jump_at = None;
    }
}

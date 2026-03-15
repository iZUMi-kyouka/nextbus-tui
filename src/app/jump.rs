use std::time::Instant;

use super::App;

impl App {
    /// Called when a digit key is pressed in normal mode.
    /// Two digits trigger an immediate jump; a single digit waits for the 1 s tick timeout.
    pub fn push_jump_digit(&mut self, digit: char) {
        self.jump_buf.push(digit);
        self.jump_at = Some(Instant::now());
        // Commit immediately when 2 digits are entered, or when the list is
        // short enough that a single digit uniquely identifies any stop.
        if self.jump_buf.len() == 2 || self.sorted_indices.len() < 10 {
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

#[cfg(test)]
mod tests {
    use super::App;
    use std::sync::mpsc;
    use std::time::Instant;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        let mut app = App::new(tx);
        app.favourites.clear();
        app.rebuild_list();
        app
    }

    #[test]
    fn single_digit_waits_in_large_list() {
        let mut app = make_app();
        // 33 stops → must wait for second digit or timeout
        app.push_jump_digit('3');
        assert_eq!(app.jump_buf, "3");
        assert!(app.jump_at.is_some());
    }

    #[test]
    fn single_digit_commits_immediately_in_small_list() {
        let mut app = make_app();
        app.search_query = "COM".to_string();
        app.rebuild_list();
        let count = app.sorted_indices.len();
        assert!(count < 10, "COM filter gives {} results, expected < 10", count);
        app.push_jump_digit('1');
        assert!(app.jump_buf.is_empty(), "should commit immediately in small list");
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn two_digits_commit_immediately() {
        let mut app = make_app();
        app.push_jump_digit('1');
        assert!(!app.jump_buf.is_empty());
        app.push_jump_digit('0');
        assert!(app.jump_buf.is_empty());
        assert_eq!(app.selected, 9); // position 10 → index 9
    }

    #[test]
    fn commit_jump_out_of_range_keeps_selection() {
        let mut app = make_app();
        let original = app.selected;
        app.jump_buf = "99".to_string();
        app.commit_jump();
        assert_eq!(app.selected, original);
        assert!(app.jump_buf.is_empty());
    }

    #[test]
    fn commit_jump_zero_is_no_op() {
        let mut app = make_app();
        let original = app.selected;
        app.jump_buf = "0".to_string();
        app.commit_jump();
        assert_eq!(app.selected, original);
        assert!(app.jump_buf.is_empty());
    }

    #[test]
    fn cancel_jump_clears_buf_and_timer() {
        let mut app = make_app();
        app.jump_buf = "5".to_string();
        app.jump_at = Some(Instant::now());
        app.cancel_jump();
        assert!(app.jump_buf.is_empty());
        assert!(app.jump_at.is_none());
    }
}

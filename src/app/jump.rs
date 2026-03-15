use std::time::Instant;

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
        if let Ok(n) = self.nav.jump_buf.parse::<usize>()
            && n > 0
            && n <= self.nav.sorted_indices.len()
        {
            self.nav.selected = n - 1;
            self.nav.list_state.select(Some(self.nav.selected));
            self.ensure_data();
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

#[cfg(test)]
mod tests {
    use super::App;
    use std::sync::mpsc;
    use std::time::Instant;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        App::new_test(tx)
    }

    #[test]
    fn single_digit_waits_in_large_list() {
        let mut app = make_app();
        app.push_jump_digit('3');
        assert_eq!(app.nav.jump_buf, "3");
        assert!(app.nav.jump_at.is_some());
    }

    #[test]
    fn single_digit_commits_immediately_in_small_list() {
        let mut app = make_app();
        app.nav.search_query = "COM".to_string();
        app.rebuild_list();
        let count = app.nav.sorted_indices.len();
        assert!(
            count < 10,
            "COM filter gives {} results, expected < 10",
            count
        );
        app.push_jump_digit('1');
        assert!(
            app.nav.jump_buf.is_empty(),
            "should commit immediately in small list"
        );
        assert_eq!(app.nav.selected, 0);
    }

    #[test]
    fn two_digits_commit_immediately() {
        let mut app = make_app();
        app.push_jump_digit('1');
        assert!(!app.nav.jump_buf.is_empty());
        app.push_jump_digit('0');
        assert!(app.nav.jump_buf.is_empty());
        assert_eq!(app.nav.selected, 9);
    }

    #[test]
    fn commit_jump_out_of_range_keeps_selection() {
        let mut app = make_app();
        let original = app.nav.selected;
        app.nav.jump_buf = "99".to_string();
        app.commit_jump();
        assert_eq!(app.nav.selected, original);
        assert!(app.nav.jump_buf.is_empty());
    }

    #[test]
    fn commit_jump_zero_is_no_op() {
        let mut app = make_app();
        let original = app.nav.selected;
        app.nav.jump_buf = "0".to_string();
        app.commit_jump();
        assert_eq!(app.nav.selected, original);
        assert!(app.nav.jump_buf.is_empty());
    }

    #[test]
    fn cancel_jump_clears_buf_and_timer() {
        let mut app = make_app();
        app.nav.jump_buf = "5".to_string();
        app.nav.jump_at = Some(Instant::now());
        app.cancel_jump();
        assert!(app.nav.jump_buf.is_empty());
        assert!(app.nav.jump_at.is_none());
    }
}

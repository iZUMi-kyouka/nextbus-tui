/// Column where the stop-list panel ends (exclusive).
/// Matches the percentage split used by `ui::render_panels`.
/// Used by `app::mouse` for hit-testing; keep both in sync.
pub fn list_x_end(term_w: u16) -> u16 {
    if term_w < 70 {
        term_w / 2 // 50 % — very narrow
    } else if term_w < 100 {
        term_w * 40 / 100 // 40 % — narrow
    } else {
        term_w * 33 / 100 // 33 % — normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_x_end_very_narrow() {
        assert_eq!(list_x_end(60), 30);
    }

    #[test]
    fn list_x_end_narrow() {
        assert_eq!(list_x_end(80), 32);
    }

    #[test]
    fn list_x_end_normal() {
        assert_eq!(list_x_end(120), 39);
    }

    #[test]
    fn list_x_end_boundary_70() {
        // exactly 70 → narrow branch (40 %)
        assert_eq!(list_x_end(70), 28);
    }

    #[test]
    fn list_x_end_boundary_100() {
        // exactly 100 → normal branch (33 %)
        assert_eq!(list_x_end(100), 33);
    }
}

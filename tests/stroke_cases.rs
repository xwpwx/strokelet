use strokelet::{Direction, best_match, conflicts, straight_points};

#[test]
fn translated_and_scaled_up_stroke_matches_the_builtin_line() {
    let templates = vec![straight_points(Direction::Up)];
    let moved: Vec<_> = [(4.0, 10.0), (4.0, -150.0)].into_iter().collect();
    assert_eq!(best_match(&templates, &moved), Some(0));
}

#[test]
fn down_stroke_does_not_match_up() {
    let templates = vec![straight_points(Direction::Up)];
    assert_eq!(
        best_match(&templates, &straight_points(Direction::Down)),
        None
    );
}

#[test]
fn diagonal_does_not_match_up() {
    let templates = vec![straight_points(Direction::Up)];
    assert_eq!(best_match(&templates, &[(0.0, 0.0), (100.0, -100.0)]), None);
}

#[test]
fn hook_does_not_match_a_straight_line() {
    let templates = vec![straight_points(Direction::Up)];
    let hook = vec![(0.0, 0.0), (0.0, -80.0), (80.0, -80.0)];
    assert_eq!(best_match(&templates, &hook), None);
}

#[test]
fn a_new_stroke_too_close_to_an_existing_one_is_rejected() {
    let existing = straight_points(Direction::Up);
    let almost = vec![(0.0, 0.0), (2.0, -80.0)];
    assert!(conflicts(&existing, &almost));
    assert!(!conflicts(&existing, &straight_points(Direction::Left)));
}

#[test]
fn closest_template_wins_when_the_margin_is_clear() {
    let templates = vec![
        straight_points(Direction::Up),
        straight_points(Direction::Right),
    ];
    assert_eq!(
        best_match(&templates, &straight_points(Direction::Right)),
        Some(1)
    );
}

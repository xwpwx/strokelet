use strokelet::{Decision, Direction, Gesture, Limits};

fn gesture() -> Gesture {
    Gesture::new(Limits::default())
}

fn decide(motions: &[(f64, f64)], elapsed_ms: u64) -> Decision {
    let mut gesture = gesture();
    gesture.press();
    for &(dx, dy) in motions {
        gesture.motion(dx, dy);
    }
    gesture.release(elapsed_ms)
}

#[test]
fn upward_stroke_copies_once() {
    let mut gesture = gesture();
    gesture.press();
    for _ in 0..10 {
        gesture.motion(0.0, -10.0);
    }

    assert_eq!(gesture.release(500), Decision::Stroke(Direction::Up));
    assert_eq!(gesture.release(500), Decision::None);
}

#[test]
fn release_without_press_is_none() {
    assert_eq!(gesture().release(0), Decision::None);
}

#[test]
fn stationary_right_click() {
    let mut gesture = gesture();
    gesture.press();
    assert!(!gesture.is_drawing());
    assert_eq!(gesture.release(40), Decision::RightClick);
}

#[test]
fn small_jitter_is_right_click() {
    assert_eq!(decide(&[(3.0, 4.0)], 80), Decision::RightClick);
}

#[test]
fn exact_slop_is_right_click() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -12.0);
    assert!(!gesture.is_drawing());
    assert_eq!(gesture.release(80), Decision::RightClick);
}

#[test]
fn just_past_start_is_not_a_click() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -13.0);
    assert!(gesture.is_drawing());
    assert_eq!(gesture.release(80), Decision::Cancel);
}

#[test]
fn upward_79_cancels() {
    assert_eq!(decide(&[(0.0, -79.0)], 100), Decision::Cancel);
}

#[test]
fn upward_80_copies() {
    assert_eq!(
        decide(&[(0.0, -80.0)], 100),
        Decision::Stroke(Direction::Up)
    );
}

#[test]
fn duration_2500_copies() {
    assert_eq!(
        decide(&[(0.0, -80.0)], 2500),
        Decision::Stroke(Direction::Up)
    );
}

#[test]
fn duration_2501_cancels() {
    assert_eq!(decide(&[(0.0, -80.0)], 2501), Decision::Cancel);
}

#[test]
fn drawing_timeout_ends_drawing_before_release() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -80.0);
    gesture.advance(2500);
    assert!(gesture.is_drawing());
    gesture.advance(2501);
    assert!(!gesture.is_drawing());
    gesture.motion(0.0, -40.0);
    assert_eq!(gesture.release(2600), Decision::Cancel);
}

#[test]
fn late_drawing_after_deadline_cancels() {
    let mut gesture = gesture();
    gesture.press();
    gesture.advance(2501);
    assert!(!gesture.is_drawing());
    gesture.motion(0.0, -80.0);
    assert!(!gesture.is_drawing());
    assert_eq!(gesture.release(2600), Decision::Cancel);
}

#[test]
fn stale_timestamp_does_not_revive_drawing() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -80.0);
    gesture.advance(2501);
    gesture.advance(100);
    assert!(!gesture.is_drawing());
    assert_eq!(gesture.release(100), Decision::Cancel);
}

#[test]
fn pending_hold_past_deadline_stays_right_click() {
    let mut gesture = gesture();
    gesture.press();
    gesture.advance(4000);
    assert!(!gesture.is_drawing());
    assert_eq!(gesture.release(4000), Decision::RightClick);
}

#[test]
fn lateral_floor_limits_short_upward_stroke() {
    let limits = Limits {
        start_counts: 12.0,
        click_slop_counts: 12.0,
        min_up_counts: 40.0,
        max_duration_ms: 2500,
    };
    let mut inside = Gesture::new(limits);
    inside.press();
    inside.motion(12.0, -40.0);
    assert_eq!(inside.release(100), Decision::Stroke(Direction::Up));

    let mut outside = Gesture::new(limits);
    outside.press();
    outside.motion(13.0, -40.0);
    assert_eq!(outside.release(100), Decision::Cancel);
}

#[test]
fn long_press_inside_slop_stays_right_click() {
    assert_eq!(decide(&[(0.0, -12.0)], 4000), Decision::RightClick);
}

#[test]
fn lateral_30_copies() {
    assert_eq!(
        decide(&[(30.0, -100.0)], 200),
        Decision::Stroke(Direction::Up)
    );
    assert_eq!(
        decide(&[(-30.0, -100.0)], 200),
        Decision::Stroke(Direction::Up)
    );
}

#[test]
fn lateral_31_cancels() {
    assert_eq!(decide(&[(31.0, -100.0)], 200), Decision::Cancel);
    assert_eq!(decide(&[(-31.0, -100.0)], 200), Decision::Cancel);
}

#[test]
fn straightness_within_limit_copies() {
    assert_eq!(
        decide(&[(0.0, -100.0), (8.0, 0.0), (-8.0, 0.0)], 200),
        Decision::Stroke(Direction::Up)
    );
}

#[test]
fn straightness_past_limit_cancels() {
    assert_eq!(
        decide(&[(0.0, -100.0), (9.0, 0.0), (-9.0, 0.0)], 200),
        Decision::Cancel
    );
}

#[test]
fn round_trip_is_not_a_click() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, 100.0);
    assert!(gesture.is_drawing());
    gesture.motion(0.0, -100.0);
    assert!(gesture.is_drawing());
    assert_eq!(gesture.release(200), Decision::Cancel);
}

#[test]
fn downward_stroke_matches() {
    assert_eq!(
        decide(&[(0.0, 120.0)], 200),
        Decision::Stroke(Direction::Down)
    );
}

#[test]
fn right_and_left_strokes_match() {
    assert_eq!(
        decide(&[(120.0, 0.0)], 200),
        Decision::Stroke(Direction::Right)
    );
    assert_eq!(
        decide(&[(-120.0, 0.0)], 200),
        Decision::Stroke(Direction::Left)
    );
}

#[test]
fn diagonal_stroke_cancels() {
    assert_eq!(decide(&[(100.0, -100.0)], 200), Decision::Cancel);
}

#[test]
fn retrace_uses_path_length_not_endpoint() {
    assert_eq!(decide(&[(0.0, -100.0), (0.0, 20.0)], 200), Decision::Cancel);
}

#[test]
fn split_axis_frames_are_less_straight_than_one_frame() {
    assert_eq!(
        decide(&[(20.0, -100.0)], 200),
        Decision::Stroke(Direction::Up)
    );
    assert_eq!(decide(&[(20.0, 0.0), (0.0, -100.0)], 200), Decision::Cancel);
}

#[test]
fn cancel_waits_for_release_and_blocks_copy() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -100.0);
    assert!(gesture.is_drawing());
    gesture.cancel();
    assert!(!gesture.is_drawing());
    gesture.motion(0.0, -100.0);
    assert_eq!(gesture.release(100), Decision::Cancel);
    assert_eq!(gesture.release(100), Decision::None);
}

#[test]
fn cancel_before_drawing_is_not_a_click() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(3.0, 4.0);
    gesture.cancel();
    assert_eq!(gesture.release(40), Decision::Cancel);
}

#[test]
fn movement_between_slop_and_start_cancels() {
    let mut gesture = Gesture::new(Limits {
        start_counts: 30.0,
        click_slop_counts: 12.0,
        min_up_counts: 80.0,
        max_duration_ms: 2500,
    });
    gesture.press();
    gesture.motion(0.0, -20.0);
    assert!(!gesture.is_drawing());
    assert_eq!(gesture.release(100), Decision::Cancel);
}

#[test]
fn second_press_does_not_reset_an_open_gesture() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -80.0);
    gesture.press();
    assert_eq!(gesture.release(100), Decision::Stroke(Direction::Up));
}

#[test]
fn next_press_starts_a_fresh_gesture() {
    let mut gesture = gesture();
    gesture.press();
    gesture.motion(0.0, -80.0);
    assert_eq!(gesture.release(100), Decision::Stroke(Direction::Up));
    gesture.press();
    assert_eq!(gesture.release(40), Decision::RightClick);
}

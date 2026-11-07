//! Tests of the shared render support:
//! the clock step each draw pass takes against the animation times in the markup.
//!
//! What:
//! the markup files are read as text and every animation time is compared with `PASS_STEP`.
//! Why:
//! a pass moves the test clock by `PASS_STEP` so that an animation started in one pass has ended in the next;
//! a longer animation written later would be drawn mid-way again,
//! on a fast build,
//! and the pixel tests would depend on build speed.

/// What:
/// `use crate::ui_render_support::PASS_STEP;` imports the clock step of a draw pass from the render support.
/// Why:
/// the test compares it with the animation times of the markup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PASS_STEP } from "./ui_render_support";
/// ```
use crate::ui_render_support::PASS_STEP;

/// What:
/// `times_after` gives every number of milliseconds that follows `marker` and ends in `ms` in `text`.
/// Why:
/// the markup writes animation times as `duration: 150ms;`
/// and the shared enter time as `motion-enter: 120ms;`.
fn times_after(text: &str, marker: &str) -> Vec<u64> {
    let mut times = Vec::new();
    for part in text.split(marker).skip(1) {
        let digits: String = part.chars().take_while(|character| return character.is_ascii_digit()).collect();
        if digits.is_empty() || !part[digits.len()..].starts_with("ms") {
            continue;
        }
        times.push(digits.parse::<u64>().expect("a run of ASCII digits is a number"));
    }
    return times;
}

#[test]
fn the_pass_step_covers_the_longest_animation_time_of_the_markup() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let mut longest = 0;
    let mut files_read = 0;
    for entry in std::fs::read_dir(&folder).expect("the ui folder is there") {
        let path = entry.expect("a folder entry").path();
        if path.extension().and_then(|extension| return extension.to_str()) != Some("slint") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a markup file is text");
        files_read += 1;
        for time in times_after(&text, "duration: ").into_iter().chain(times_after(&text, "motion-enter: ")) {
            longest = longest.max(time);
        }
    }
    assert!(files_read > 0, "the markup files were found in {folder:?}");
    assert!(longest > 0, "the markup has animation times to compare");
    assert!(
        u128::from(longest) <= PASS_STEP.as_millis(),
        "an animation of {longest} ms is longer than the {} ms a draw pass moves the clock",
        PASS_STEP.as_millis()
    );
}

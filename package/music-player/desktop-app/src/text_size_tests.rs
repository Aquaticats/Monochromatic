//! Tests for the pure Text size rules in `text_size`.

use super::*;
use serde_json::{Map, Value};

/// What: `all_at` gives sizes with every element set to `px`.
fn all_at(px: u16) -> TextSizes {
    return TextElement::ALL.iter().fold(TextSizes::default(), |sizes, element| return sizes.with(*element, px));
}

/// The defaults are the sizes the app drew before the setting existed.
#[test]
fn the_defaults_are_the_sizes_the_app_always_drew() {
    let sizes = TextSizes::default();
    assert_eq!(sizes.get(TextElement::PaneTitle), 20);
    assert_eq!(sizes.get(TextElement::GroupHeading), 20);
    assert_eq!(sizes.get(TextElement::FolderName), 14);
    assert_eq!(sizes.get(TextElement::TrackTitle), 14);
    assert!(sizes.is_default());
}

/// The limits are 10 and 32, and every default lies between them.
#[test]
fn the_limits_hold_every_default() {
    assert_eq!((MIN_TEXT_SIZE_PX, MAX_TEXT_SIZE_PX), (10, 32));
    for element in TextElement::ALL {
        assert!((MIN_TEXT_SIZE_PX..=MAX_TEXT_SIZE_PX).contains(&element.default_px()), "{element:?}");
    }
}

/// The window numbers the steppers 0 to 3 in the order of `ALL`; any other number is no element.
#[test]
fn a_stepper_index_names_its_element() {
    assert_eq!(TextElement::from_index(0), Some(TextElement::PaneTitle));
    assert_eq!(TextElement::from_index(1), Some(TextElement::GroupHeading));
    assert_eq!(TextElement::from_index(2), Some(TextElement::FolderName));
    assert_eq!(TextElement::from_index(3), Some(TextElement::TrackTitle));
    assert_eq!(TextElement::from_index(4), None);
    assert_eq!(TextElement::from_index(-1), None);
}

/// The session keys are four distinct names that end in `_px`, one per element.
#[test]
fn every_element_has_its_own_session_key() {
    let mut keys: Vec<&str> = TextElement::ALL.iter().map(|element| return element.session_key()).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), 4);
    assert!(keys.iter().all(|key| return key.starts_with("text_size_") && key.ends_with("_px")), "{keys:?}");
}

/// One step moves one pixel and touches only its element.
#[test]
fn a_step_moves_one_pixel_of_one_element() {
    let up = TextSizes::default().stepped(TextElement::FolderName, 1);
    assert_eq!(up.get(TextElement::FolderName), 15);
    assert_eq!(up.get(TextElement::PaneTitle), 20);
    assert_eq!(up.get(TextElement::TrackTitle), 14);
    let down = up.stepped(TextElement::FolderName, -1);
    assert_eq!(down, TextSizes::default());
}

/// A step at an end changes nothing, and a large step stops at the end.
#[test]
fn a_step_stops_at_the_limits() {
    let top = all_at(MAX_TEXT_SIZE_PX);
    assert_eq!(top.stepped(TextElement::TrackTitle, 1), top);
    let bottom = all_at(MIN_TEXT_SIZE_PX);
    assert_eq!(bottom.stepped(TextElement::TrackTitle, -1), bottom);
    assert_eq!(TextSizes::default().stepped(TextElement::PaneTitle, 1000).get(TextElement::PaneTitle), 32);
    assert_eq!(TextSizes::default().stepped(TextElement::PaneTitle, -1000).get(TextElement::PaneTitle), 10);
    assert_eq!(TextSizes::default().stepped(TextElement::PaneTitle, i32::MAX).get(TextElement::PaneTitle), 32);
    assert_eq!(TextSizes::default().stepped(TextElement::PaneTitle, i32::MIN).get(TextElement::PaneTitle), 10);
}

/// `with` holds a size inside the limits, so no caller builds one outside.
#[test]
fn with_holds_a_size_inside_the_limits() {
    assert_eq!(TextSizes::default().with(TextElement::GroupHeading, 0).get(TextElement::GroupHeading), 10);
    assert_eq!(TextSizes::default().with(TextElement::GroupHeading, 9).get(TextElement::GroupHeading), 10);
    assert_eq!(TextSizes::default().with(TextElement::GroupHeading, 33).get(TextElement::GroupHeading), 32);
    assert_eq!(TextSizes::default().with(TextElement::GroupHeading, u16::MAX).get(TextElement::GroupHeading), 32);
    assert_eq!(TextSizes::default().with(TextElement::GroupHeading, 21).get(TextElement::GroupHeading), 21);
}

/// The buttons report their ends: minus is off at 10 and plus is off at 32, and both are on between.
#[test]
fn the_buttons_know_their_ends() {
    let bottom = all_at(MIN_TEXT_SIZE_PX);
    let top = all_at(MAX_TEXT_SIZE_PX);
    let middle = TextSizes::default();
    for element in TextElement::ALL {
        assert!(!bottom.can_decrease(element) && bottom.can_increase(element), "{element:?} at 10");
        assert!(top.can_decrease(element) && !top.can_increase(element), "{element:?} at 32");
        assert!(middle.can_decrease(element) && middle.can_increase(element), "{element:?} at its default");
    }
}

/// Changing a size makes the sizes differ from the defaults, and putting it back makes them the defaults again.
#[test]
fn is_default_follows_the_sizes() {
    let changed = TextSizes::default().stepped(TextElement::TrackTitle, 1);
    assert!(!changed.is_default());
    assert!(changed.stepped(TextElement::TrackTitle, -1).is_default());
}

/// A whole number inside the limits is the size; the ends count.
#[test]
fn a_stored_size_inside_the_limits_is_kept() {
    for px in [10_u16, 11, 20, 31, 32] {
        assert_eq!(size_or_default(TextElement::FolderName, Some(&Value::from(px))), px);
    }
}

/// A missing field is the default and says nothing, because a file written before the setting has none.
#[test]
fn a_missing_stored_size_is_the_default() {
    assert_eq!(size_or_default(TextElement::PaneTitle, None), 20);
    assert_eq!(size_or_default(TextElement::TrackTitle, None), 14);
}

/// Anything that is not a whole number from 10 to 32 is the default of that element.
#[test]
fn a_corrupt_stored_size_is_the_default_of_its_element() {
    let mut object = Map::new();
    object.insert("px".to_string(), Value::from(14));
    let corrupt = [
        Value::from(9),
        Value::from(33),
        Value::from(0),
        Value::from(-12),
        Value::from(70000),
        Value::from(14.5),
        Value::from(14.0),
        Value::from("14"),
        Value::from("abc"),
        Value::Null,
        Value::from(true),
        Value::Array(vec![Value::from(14)]),
        Value::Object(object),
    ];
    for value in &corrupt {
        assert_eq!(size_or_default(TextElement::PaneTitle, Some(value)), 20, "{value}");
        assert_eq!(size_or_default(TextElement::FolderName, Some(value)), 14, "{value}");
    }
}

/// The sizes serialize under the four session keys and nothing else.
#[test]
fn the_sizes_serialize_under_their_session_keys() {
    let value = serde_json::to_value(TextSizes::default()).expect("the sizes serialize");
    let object = value.as_object().expect("the sizes are a JSON object");
    assert_eq!(object.len(), 4);
    for element in TextElement::ALL {
        assert_eq!(object.get(element.session_key()), Some(&Value::from(element.default_px())), "{element:?}");
    }
}

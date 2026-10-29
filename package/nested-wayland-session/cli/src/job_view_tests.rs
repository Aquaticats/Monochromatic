//! Property decoding follows Plasma's `QVariant` conversions.

/// Brings the decoders into scope.
use super::*;

/// Builds an owned variant.
fn owned(value: Value<'_>) -> OwnedValue {
    return OwnedValue::try_from(value).expect("a plain value converts to an owned value");
}

/// A percent of any integer width moves the bar;
///  other types do not.
#[test]
fn percent_accepts_integer_widths_only() {
    for (value, expected) in [
        (Value::U32(42), Some(42)),
        (Value::I32(42), Some(42)),
        (Value::U8(7), Some(7)),
        (Value::I64(9), Some(9)),
        (Value::I32(-1), None),
        (Value::F64(0.5), None),
        (Value::from("50"), None),
        (Value::Bool(true), None),
    ] {
        let properties = HashMap::from([("percent".to_string(), owned(value.try_clone().expect("cloning a value without file descriptors succeeds")))]);
        assert_eq!(percent_property(&properties, "percent"), expected, "{value:?}");
    }
    assert_eq!(percent_property(&HashMap::new(), "percent"), None);
}

/// Only the four displayed properties are decoded;
///  `descriptionField` and unknown keys are not.
#[test]
fn only_displayed_properties_are_decoded() {
    let properties = HashMap::from([
        ("title".to_string(), owned(Value::from("Analysing"))),
        ("infoMessage".to_string(), owned(Value::from("track 1"))),
        ("percent".to_string(), owned(Value::U32(3))),
        ("suspended".to_string(), owned(Value::Bool(true))),
        ("descriptionField".to_string(), owned(Value::from("ignored"))),
    ]);
    assert_eq!(
        job_changes(&properties),
        JobChanges {
            title: Some("Analysing".to_string()),
            info_message: Some("track 1".to_string()),
            percent: Some(3),
            suspended: Some(true),
        },
    );
    assert_eq!(job_changes(&HashMap::new()), JobChanges::default());
}

/// A title of the wrong type is ignored rather than coerced.
#[test]
fn wrong_typed_strings_and_flags_are_ignored() {
    let properties = HashMap::from([
        ("title".to_string(), owned(Value::U32(1))),
        ("suspended".to_string(), owned(Value::from("yes"))),
    ]);
    assert_eq!(job_changes(&properties), JobChanges::default());
}

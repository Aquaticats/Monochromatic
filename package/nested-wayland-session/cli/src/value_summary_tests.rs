//! Summaries keep the type a client sent and never print a payload.

/// Brings the summary functions into scope.
use super::*;

/// Builds an owned variant for a test map.
fn owned(value: Value<'_>) -> OwnedValue {
    return OwnedValue::try_from(value).expect("a plain value converts to an owned value");
}

/// Plain values print with their signature,
///  sorted by key.
#[test]
fn plain_values_are_sorted_with_signatures() {
    let map: HashMap<String, OwnedValue> = HashMap::from([
        ("urgency".to_string(), owned(Value::U8(2))),
        ("value".to_string(), owned(Value::I32(50))),
        ("transient".to_string(), owned(Value::Bool(true))),
        ("category".to_string(), owned(Value::from("transfer"))),
    ]);
    assert_eq!(
        format_summaries(&summarize_map(&map)),
        "category:s:transfer,transient:b:true,urgency:y:2,value:i:50",
    );
}

/// Byte arrays show their length,
///  not their contents,
///  and hostile text stays encoded.
#[test]
fn containers_show_counts_and_text_is_encoded() {
    let pixels = Value::from(vec![1_u8, 2, 3, 4]);
    let map: HashMap<String, OwnedValue> = HashMap::from([
        ("image-data".to_string(), owned(pixels)),
        ("a,b:c".to_string(), owned(Value::from("x y;z"))),
    ]);
    assert_eq!(
        format_summaries(&summarize_map(&map)),
        "a%2Cb%3Ac:s:x%20y%3Bz,image-data:ay:array-4",
    );
}

/// An empty map is an empty string,
///  and a variant inside a variant is unwrapped.
#[test]
fn empty_maps_and_nested_variants() {
    assert_eq!(format_summaries(&summarize_map(&HashMap::new())), "");
    assert_eq!(describe_value(&Value::Value(Box::new(Value::U32(7)))), "7");
}

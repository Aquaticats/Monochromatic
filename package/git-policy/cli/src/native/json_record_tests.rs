//! Controls for JSON record reading and writing against `JSON.parse` and `JSON.stringify`.

use super::*;

/// Only an object is a record;
///  every other document is refused.
#[test]
fn only_objects_are_records() {
    assert!(parse_object("{\"a\":1}").is_some());
    assert!(parse_object("{}").is_some());
    assert!(parse_object("[1]").is_none());
    assert!(parse_object("1").is_none());
    assert!(parse_object("\"x\"").is_none());
    assert!(parse_object("null").is_none());
    assert!(parse_object("{\"a\":").is_none());
    assert!(parse_object("").is_none());
    // JSON.parse refuses comments and trailing commas, and so does the record reader.
    assert!(parse_object("{\"a\":1,}").is_none());
    assert!(parse_object("{/* c */\"a\":1}").is_none());
}

/// The last duplicate key wins,
///  as in `JSON.parse`.
#[test]
fn the_last_duplicate_key_wins() {
    let map: Map<String, Value> = parse_object("{\"a\":\"first\",\"a\":\"last\"}").expect("object");
    assert_eq!(string_field(&map, "a"), Some("last"));
}

/// Text fields are read only from strings.
#[test]
fn text_fields_are_strings_only() {
    let map: Map<String, Value> =
        parse_object("{\"s\":\"x\",\"n\":1,\"b\":true,\"z\":null}").expect("object");
    assert_eq!(string_field(&map, "s"), Some("x"));
    assert_eq!(string_field(&map, "n"), None);
    assert_eq!(string_field(&map, "b"), None);
    assert_eq!(string_field(&map, "z"), None);
    assert_eq!(string_field(&map, "missing"), None);
}

/// Safe integers follow `Number.isSafeInteger` over doubles.
#[test]
fn safe_integers_follow_javascript() {
    let map: Map<String, Value> = parse_object(
        "{\"a\":1,\"b\":1.0,\"c\":1e0,\"d\":1.5,\"e\":9007199254740991,\"f\":9007199254740992,\
         \"g\":-9007199254740991,\"h\":-9007199254740992,\"i\":\"1\",\"j\":18446744073709551615,\
         \"k\":1e20,\"l\":-0,\"m\":4503599627370495.0,\"n\":9007199254740992.0,\"o\":-3.0}",
    )
    .expect("object");
    assert_eq!(safe_integer_field(&map, "a"), Some(1));
    assert_eq!(safe_integer_field(&map, "b"), Some(1));
    assert_eq!(safe_integer_field(&map, "c"), Some(1));
    assert_eq!(safe_integer_field(&map, "d"), None);
    assert_eq!(safe_integer_field(&map, "e"), Some(MAX_SAFE_INTEGER));
    assert_eq!(safe_integer_field(&map, "f"), None);
    assert_eq!(safe_integer_field(&map, "g"), Some(-MAX_SAFE_INTEGER));
    assert_eq!(safe_integer_field(&map, "h"), None);
    assert_eq!(safe_integer_field(&map, "i"), None);
    assert_eq!(safe_integer_field(&map, "j"), None);
    assert_eq!(safe_integer_field(&map, "k"), None);
    assert_eq!(safe_integer_field(&map, "l"), Some(0));
    // A fractional spelling is read exactly below 2^52; `serde_json` without its
    // `float_roundtrip` feature may misround one closer to 2^53, a spelling neither wrapper writes.
    assert_eq!(safe_integer_field(&map, "m"), Some(4_503_599_627_370_495));
    assert_eq!(safe_integer_field(&map, "n"), None);
    assert_eq!(safe_integer_field(&map, "o"), Some(-3));
    assert_eq!(safe_integer_field(&map, "missing"), None);
}

/// Equality to an expected number accepts every spelling of it.
#[test]
fn number_equality_compares_values() {
    let map: Map<String, Value> =
        parse_object("{\"a\":2,\"b\":2.0,\"c\":\"2\",\"d\":3}").expect("object");
    assert!(number_equals(&map, "a", 2));
    assert!(number_equals(&map, "b", 2));
    assert!(!number_equals(&map, "c", 2));
    assert!(!number_equals(&map, "d", 2));
    assert!(!number_equals(&map, "missing", 2));
}

/// The fatal decoder accepts valid UTF-8,
///  drops one leading mark,
///  and refuses invalid bytes.
#[test]
fn the_fatal_decoder_matches_text_decoder() {
    assert_eq!(decode_fatal(b"{}"), Some("{}"));
    assert_eq!(decode_fatal(b"\xef\xbb\xbf{}"), Some("{}"));
    assert_eq!(
        decode_fatal(b"\xef\xbb\xbf\xef\xbb\xbf{}"),
        Some("\u{feff}{}")
    );
    assert_eq!(decode_fatal(b"{\xff}"), None);
}

/// The writer reproduces `JSON.stringify` for every field kind,
///  in insertion order.
#[test]
fn the_writer_matches_json_stringify() {
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", 2)
        .string("text", "a\"b\\c\n\t\u{1}\u{7f}é/")
        .boolean("yes", true)
        .boolean("no", false)
        .integer("negative", -5)
        .raw(
            "list",
            string_array(&[String::from("x"), String::from("y\"")]).as_str(),
        )
        .raw("empty", string_array(&[]).as_str());
    // Computed with Node 24:
    // JSON.stringify({schemaVersion:2,text:'a"b\\c\n\t\u0001\u007fé/',yes:true,no:false,negative:-5,list:['x','y"'],empty:[]})
    assert_eq!(
        writer.finish(),
        "{\"schemaVersion\":2,\"text\":\"a\\\"b\\\\c\\n\\t\\u0001\u{7f}é/\",\"yes\":true,\
         \"no\":false,\"negative\":-5,\"list\":[\"x\",\"y\\\"\"],\"empty\":[]}"
    );
    assert_eq!(ObjectWriter::new().finish(), "{}");
}

/// Quoting escapes exactly what `JSON.stringify` escapes.
#[test]
fn quoting_matches_json_stringify() {
    assert_eq!(quote(""), "\"\"");
    assert_eq!(quote("\u{8}\u{c}\r"), "\"\\b\\f\\r\"");
    assert_eq!(quote("\u{1f}"), "\"\\u001f\"");
    assert_eq!(quote("\u{2028}"), "\"\u{2028}\"");
}

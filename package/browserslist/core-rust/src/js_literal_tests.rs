//! What:
//!  Tests for the packed-JavaScript literal parser: accepted shapes and adversarial rejections.
//! Why:
//!  The parser is the trust boundary for downloaded data, so every rejection names a byte offset the tests pin.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseModuleExports', () => { /* accepts the subset, rejects the rest */ });
//! ```

use crate::error::BrowserslistError;
use crate::js_literal::{JsValue, object_get, parse_module_exports};

/// Parse `module.exports=<body>;\n` with a fixed file name.
fn parse(body: &str) -> Result<JsValue, BrowserslistError> {
    return parse_module_exports("agents.js", &format!("module.exports={body};\n"));
}

/// Assert the body fails and the message names the file and the expected offset (counted from the body start).
fn assert_rejected(body: &str, offset_in_body: usize) {
    let error = parse(body).expect_err("body must be rejected");
    let expected_offset = "module.exports=".len() + offset_in_body;
    assert_eq!(
        error.to_string().contains(&format!("byte offset {expected_offset}")),
        true,
        "{body:?} gave {error}"
    );
    assert_eq!(error.to_string().starts_with("agents.js: "), true, "{error}");
}

/// The subset's value kinds all parse, with keys in source order.
#[test]
fn accepts_the_subset() {
    let value = parse(r#"{A:{"0":0.074518,B:"ms",C:["","I","oB"],D:null,E:"Opera Mini",F:0},Z:[]}"#)
        .expect("subset literal parses");
    let browser = object_get(&value, "A").expect("A present");
    assert_eq!(object_get(browser, "B"), Some(&JsValue::Text("ms".to_string())));
    assert_eq!(object_get(browser, "0"), Some(&JsValue::Number("0.074518".to_string())));
    assert_eq!(object_get(browser, "D"), Some(&JsValue::Null));
    assert_eq!(object_get(browser, "E"), Some(&JsValue::Text("Opera Mini".to_string())));
    assert_eq!(
        object_get(browser, "C"),
        Some(&JsValue::Array(vec![
            JsValue::Text(String::new()),
            JsValue::Text("I".to_string()),
            JsValue::Text("oB".to_string()),
        ]))
    );
    assert_eq!(object_get(&value, "Z"), Some(&JsValue::Array(Vec::new())));
}

/// `object_get` answers `None` for a missing key and for a non-object.
#[test]
fn object_get_absent_cases() {
    let value = parse("{A:1}").expect("parses");
    assert_eq!(object_get(&value, "B"), None);
    assert_eq!(object_get(&JsValue::Null, "A"), None);
}

/// The empty object and array are valid.
#[test]
fn accepts_empty_containers() {
    assert_eq!(parse("{}").expect("empty object"), JsValue::Object(Vec::new()));
    assert_eq!(parse("[]").expect("empty array"), JsValue::Array(Vec::new()));
}

/// Envelope violations name offset zero or the offending byte.
#[test]
fn rejects_bad_envelopes() {
    let missing_prefix = parse_module_exports("browsers.js", "exports={}\n").expect_err("prefix");
    assert_eq!(missing_prefix.to_string().contains("byte offset 0"), true);
    for (source, offset) in [
        ("module.exports= {};\n", 15),
        ("module.exports={}\n", 17),
        ("module.exports={};", 18),
        ("module.exports={};\r\n", 18),
        ("module.exports={};\n\n", 19),
        ("module.exports={};\nx", 19),
        ("module.exports=", 15),
    ] {
        let error = parse_module_exports("agents.js", source).expect_err("envelope");
        assert_eq!(error.to_string().contains(&format!("byte offset {offset}")), true, "{source:?} gave {error}");
    }
}

/// Trailing commas, holes and duplicate keys are outside the subset.
#[test]
fn rejects_structural_deviations() {
    assert_rejected("[1,]", 3);
    assert_rejected("[,1]", 1);
    assert_rejected("{A:1,}", 5);
    assert_rejected("{A:1,A:2}", 5);
    assert_rejected("{A 1}", 2);
    assert_rejected("{A:1", 4);
    assert_rejected("[1 2]", 2);
    assert_rejected("{1:2}", 1);
}

/// Strings with escapes, controls, non-ASCII bytes, or no end are rejected at the offending byte.
#[test]
fn rejects_unsafe_strings() {
    assert_rejected(r#""a\nb""#, 2);
    assert_rejected("\"a\u{7f}b\"", 2);
    assert_rejected("\"a\nb\"", 2);
    assert_rejected("\"caf\u{e9}\"", 4);
    assert_rejected("\"abc", 5);
}

/// Number forms other than non-negative integers and plain decimals are rejected.
#[test]
fn rejects_number_deviations() {
    assert_rejected("01", 1);
    assert_rejected("1.", 2);
    assert_rejected("1.5.2", 3);
    assert_rejected("1e5", 1);
    assert_rejected("-1", 0);
    assert_rejected(".5", 0);
    assert_rejected("1x", 1);
}

/// Keywords other than `null`, and identifiers that merely start with it, are rejected.
#[test]
fn rejects_other_keywords() {
    assert_rejected("true", 0);
    assert_rejected("undefined", 0);
    assert_rejected("nul", 0);
    assert_rejected("nullx", 4);
    assert_rejected("[1,\u{feff}2]", 3);
}

/// Nesting at the cap parses; one level deeper is rejected before deeper state is built.
#[test]
fn bounds_nesting_depth() {
    let at_cap = format!("{}{}", "[".repeat(8), "]".repeat(8));
    assert_eq!(parse(&at_cap).is_ok(), true);
    let over_cap = format!("{}{}", "[".repeat(9), "]".repeat(9));
    assert_rejected(&over_cap, 8);
}

/// A hostile deeply nested input finishes without exhausting the stack.
#[test]
fn survives_hostile_depth() {
    let hostile = "[".repeat(200_000);
    assert_eq!(parse(&hostile).is_err(), true);
}

/// Control: a body the parser must reject is not accepted by `parse` (the helper can fail).
#[test]
fn positive_control_helper_can_fail() {
    assert_eq!(parse("{A:1,}").is_err(), true);
    assert_eq!(parse("{A:1}").is_ok(), true);
}

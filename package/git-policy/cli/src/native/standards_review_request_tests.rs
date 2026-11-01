//! Controls for the request body and the answer reading. Bodies are checked by parsing them back, never by copied text.

use super::*;

/// Every character class a JSON string must escape, plus ones it must not.
const AWKWARD: &str = "quote\" back\\ nl\n cr\r tab\t bell\u{7} del\u{7f} sep\u{2028} astral\u{1f600} done";

/// The body parses back to the input, the model and the question, and keeps the script's key order.
#[test]
fn body_round_trips_and_keeps_key_order() {
    let body: String = request_body(AWKWARD);
    let parsed: Value = serde_json::from_str(body.as_str()).expect("a JSON body");
    assert_eq!(parsed["input"], AWKWARD);
    assert_eq!(parsed["model"], MODEL);
    assert_eq!(parsed["questions"][0]["name"], QUESTION_NAME);
    assert_eq!(parsed["questions"][0]["choices"].as_array().map(Vec::len), Some(CHOICES.len()));
    for (index, (value, description)) in CHOICES.iter().enumerate() {
        assert_eq!(parsed["questions"][0]["choices"][index]["value"], *value);
        assert_eq!(parsed["questions"][0]["choices"][index]["description"], *description);
    }
    let positions: Vec<usize> = ["\"model\"", "\"input\"", "\"questions\"", "\"type\"", "\"name\"", "\"instructions\"", "\"choices\""]
        .iter()
        .map(|key| return body.find(key).expect("key present"))
        .collect();
    assert!(positions.windows(2).all(|pair| return pair[0] < pair[1]), "{positions:?}");
    assert!(body.starts_with(&format!("{{\"model\":\"{MODEL}\",\"input\":\"")), "no whitespace between tokens");
}

/// JavaScript does not escape DEL or U+2028 in `JSON.stringify`, and neither does the body.
#[test]
fn only_required_characters_are_escaped() {
    let body: String = request_body(AWKWARD);
    assert!(body.contains('\u{7f}'));
    assert!(body.contains('\u{2028}'));
    assert!(!body.contains('\n') && !body.contains('\t'));
    assert!(body.contains("\\u0007"));
}

/// A success answer reads into the top choice and every probability, in the answer's order.
#[test]
fn an_answer_reads_into_verdict_and_scores() {
    let raw: &str = r#"{"answers":[{"type":"choice","choice":"no","probabilities":[{"value":"yes","probability":0.43},{"value":"partial","probability":0.02},{"value":"no","probability":0.55}]}]}"#;
    let judged: Judged = read_answer(200, raw).expect("judged");
    assert_eq!(judged.top, "no");
    assert_eq!(judged.outcome, "cost_verdict: no (yes 0.43 partial 0.02 no 0.55)");
}

/// An answer without a choice or probabilities is unjudged, and a missing field prints as JavaScript prints it.
#[test]
fn a_bare_answer_is_unjudged() {
    let judged: Judged = read_answer(200, r#"{"answers":[{"type":"choice"}]}"#).expect("judged");
    assert_eq!(judged.top, "");
    assert_eq!(judged.outcome, "cost_verdict:  ()");
    let odd: Judged = read_answer(200, r#"{"answers":[{"choice":"yes","probabilities":[{}]}]}"#).expect("judged");
    assert_eq!(odd.outcome, "cost_verdict: yes (undefined undefined)");
}

/// Each failure branch names its cause, and a quoted body is cut by UTF-16 units.
#[test]
fn failures_name_their_cause_and_cut_the_body() {
    let long: String = "\u{1f600}".repeat(FAILURE_EXCERPT_UNITS);
    let error: String = read_answer(500, long.as_str()).expect_err("http failure");
    assert!(error.starts_with("HTTP 500 "));
    assert_eq!(error.trim_start_matches("HTTP 500 ").encode_utf16().count(), FAILURE_EXCERPT_UNITS);
    assert!(read_answer(404, "").expect_err("status").starts_with("HTTP 404"));
    assert!(read_answer(200, "not json").expect_err("json").starts_with("the response is not JSON"));
    for refused in [r#"{"answers":[]}"#, r#"{}"#, r#"{"answers":[{"type":"refusal"}]}"#, r#"{"answers":null}"#] {
        let message: String = read_answer(200, refused).expect_err("refusal");
        assert!(message.starts_with("Luna refused the question or returned no answer: "), "{refused}");
        assert!(message.ends_with(refused));
    }
    assert!(read_answer(200, r#"{"answers":[{"choice":"no","probabilities":{}}]}"#).is_err());
}

/// The status range that counts as success is 200 up to, not including, 300.
#[test]
fn success_statuses_are_the_2xx_range() {
    let raw: &str = r#"{"answers":[{"choice":"no"}]}"#;
    for status in [199_u16, 300, 301, 400, 599] {
        assert!(read_answer(status, raw).is_err(), "{status}");
    }
    for status in [200_u16, 204, 299] {
        assert!(read_answer(status, raw).is_ok(), "{status}");
    }
}

//! Controls for argument templates: the closed field set, escapes, rejections and rendering.

use super::*;

/// Values with one environment variable.
fn values<'a>(environment: &'a [(OsString, OsString)], top_level: &'a OsStr) -> Values<'a> {
    return Values { head: "0123456789abcdef0123", top_level, environment };
}

/// Render a template source with the given environment.
fn rendered(source: &str, environment: &[(OsString, OsString)]) -> Result<OsString, String> {
    let template: Template = parse_template(source)?;
    return render(&template, &values(environment, OsStr::new("/work/tree")));
}

/// Literal text and fields keep their order, and every field of the closed set renders its value.
#[test]
fn fields_render_in_the_order_written() {
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("HOME"), OsString::from("/home/x"))];
    assert_eq!(
        rendered("u-{shortHead}:{head}:{topLevel}:{env:HOME}", &environment),
        Ok(OsString::from("u-0123456789ab:0123456789abcdef0123:/work/tree:/home/x"))
    );
    assert_eq!(rendered("plain", &[]), Ok(OsString::from("plain")));
    assert_eq!(rendered("", &[]), Ok(OsString::new()));
}

/// The short head is as long as its constant says.
#[test]
fn the_short_head_has_the_documented_length() {
    let text: OsString = rendered("{shortHead}", &[]).expect("rendered");
    assert_eq!(text.len(), SHORT_HEAD_LENGTH);
}

/// Doubled braces are literal braces, and they never start a field.
#[test]
fn doubled_braces_are_literal() {
    assert_eq!(rendered("{{head}}", &[]), Ok(OsString::from("{head}")));
    assert_eq!(rendered("a{{b}}c", &[]), Ok(OsString::from("a{b}c")));
}

/// An unknown field, a lone brace and an unclosed field are errors that name what was written.
#[test]
fn unknown_fields_and_lone_braces_are_refused_with_the_field_list() {
    let unknown: String = parse_template("{commit}").expect_err("refused");
    assert!(unknown.contains("{commit}") && unknown.contains(FIELD_LIST), "{unknown}");
    assert!(parse_template("a}").expect_err("refused").contains("lone `}`"));
    assert!(parse_template("{head").expect_err("refused").contains("no closing brace"));
    assert!(parse_template("{}").expect_err("refused").contains("not a field"));
}

/// A variable name must be a plain identifier, so no field can name a path or an expression.
#[test]
fn environment_names_must_be_plain_identifiers() {
    for bad in ["{env:}", "{env:1A}", "{env:A-B}", "{env:A B}", "{env:$(x)}"] {
        assert!(parse_template(bad).is_err(), "{bad}");
    }
    assert!(parse_template("{env:_A1}").is_ok());
}

/// An unset variable stops the render with its name, and the last of repeated entries wins.
#[test]
fn an_unset_variable_is_an_error_and_the_last_entry_wins() {
    assert_eq!(rendered("{env:NOPE}", &[]), Err(String::from("the environment variable NOPE is not set")));
    let repeated: Vec<(OsString, OsString)> = vec![
        (OsString::from("V"), OsString::from("first")),
        (OsString::from("V"), OsString::from("last")),
    ];
    assert_eq!(rendered("{env:V}", &repeated), Ok(OsString::from("last")));
}

/// Text is never interpreted: shell syntax stays literal.
#[test]
fn shell_syntax_is_literal_text() {
    assert_eq!(rendered("$(git rev-parse HEAD) `x` $HOME; | &", &[]), Ok(OsString::from("$(git rev-parse HEAD) `x` $HOME; | &")));
}

//! What:
//!  The argument templates of a declared post-commit action: literal text with named fields from a closed set.
//! Why:
//!  An action is an argument array run directly, never a shell string, so the values a shell command computed
//!  (`$(git rev-parse --short HEAD)`, `$PATH`) become named fields the wrapper fills in itself.
//!  The set is closed and validated when the configuration loads, so a typo is a configuration error that names the
//!  field, not an empty argument at run time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! render(parse('--unit=review-{shortHead}'), { head, topLevel, environment }); // '--unit=review-0123456789ab'
//! ```

/// Owned operating-system text: a top-level path or an environment value need not be UTF-8.
use std::ffi::{OsStr, OsString};

/// How many characters of the landed commit `{shortHead}` is: long enough that two landings never share a unit name.
pub const SHORT_HEAD_LENGTH: usize = 12;

/// The fields a template accepts, spelled for the configuration error.
pub const FIELD_LIST: &str = "{head}, {shortHead}, {topLevel}, {env:NAME}";

/// What:
///  One named field.
/// Why:
///   The closed set is the whole vocabulary of an action; everything else is literal text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Field = 'head' | 'shortHead' | 'topLevel' | { env: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Field {
    /// The full object name of the landed commit.
    Head,
    /// The first `SHORT_HEAD_LENGTH` characters of it.
    ShortHead,
    /// The absolute top level of the worktree the commit landed in.
    TopLevel,
    /// The value of one environment variable of the wrapper.
    Environment(String),
}

/// What:
///  One piece of a template.
/// Why:
///   Literal text and fields alternate in the order written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Piece = { text: string } | { field: Field };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Piece {
    /// Text copied as it is.
    Text(String),
    /// A field filled in at run time.
    Field(Field),
}

/// What:
///  A parsed argument.
/// Why:
///   Parsing once at configuration load keeps the run free of syntax errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Template = Piece[];
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template(pub Vec<Piece>);

/// What:
///  The values a template renders from.
/// Why:
///   The wrapper has them already, so no Git start and no shell is needed to compute them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Values = { head: string; topLevel: string; environment: Record<string, string> };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Values<'a> {
    /// The landed commit's object name.
    pub head: &'a str,
    /// The worktree's top level.
    pub top_level: &'a OsStr,
    /// The wrapper's environment.
    pub environment: &'a [(OsString, OsString)],
}

/// What:
///  Read the field named between braces.
/// Why:
///   `env:NAME` takes a variable name of letters, digits and underscores that does not start with a digit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseField('env:HOME'); // { env: 'HOME' }
/// ```
fn parse_field(name: &str) -> Result<Field, String> {
    match name {
        "head" => return Ok(Field::Head),
        "shortHead" => return Ok(Field::ShortHead),
        "topLevel" => return Ok(Field::TopLevel),
        _ => {}
    }
    if let Some(variable) = name.strip_prefix("env:") {
        let mut characters = variable.chars();
        let starts_well: bool = characters.next().is_some_and(|first| { return first == '_' || first.is_ascii_alphabetic(); });
        if starts_well && characters.all(|rest| { return rest == '_' || rest.is_ascii_alphanumeric(); }) {
            return Ok(Field::Environment(String::from(variable)));
        }
        return Err(format!(
            "{{{name}}} names no environment variable: a name has letters, digits and underscores and does not start \
             with a digit"
        ));
    }
    return Err(format!("{{{name}}} is not a field; the fields are {FIELD_LIST}"));
}

/// What:
///  Parse one argument template in a single pass.
/// Why:
///   `{{` and `}}` write a literal brace; a lone brace or an unknown field is an error naming what was written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseTemplate('a{{b}}-{head}'); // [{ text: 'a{b}-' }, { field: head }]
/// ```
pub fn parse_template(source: &str) -> Result<Template, String> {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut text: String = String::new();
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '{' if characters.peek() == Some(&'{') => {
                characters.next();
                text.push('{');
            }
            '}' if characters.peek() == Some(&'}') => {
                characters.next();
                text.push('}');
            }
            '{' => {
                let mut name: String = String::new();
                let mut closed: bool = false;
                for inner in characters.by_ref() {
                    if inner == '}' {
                        closed = true;
                        break;
                    }
                    name.push(inner);
                }
                if !closed {
                    return Err(format!("`{{{name}` has no closing brace; write `{{{{` for a literal brace"));
                }
                if !text.is_empty() {
                    pieces.push(Piece::Text(std::mem::take(&mut text)));
                }
                pieces.push(Piece::Field(parse_field(name.as_str())?));
            }
            '}' => return Err(String::from("a lone `}` has no opening brace; write `}}` for a literal brace")),
            other => text.push(other),
        }
    }
    if !text.is_empty() {
        pieces.push(Piece::Text(text));
    }
    return Ok(Template(pieces));
}

/// What:
///  Fill a template in.
/// Why:
///   A field with no value (an unset variable) is an error naming it, so the action is skipped with a reason
///  instead of running with an empty argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// render(template, values); // string
/// ```
pub fn render(template: &Template, values: &Values<'_>) -> Result<OsString, String> {
    let mut rendered: OsString = OsString::new();
    for piece in &template.0 {
        match piece {
            Piece::Text(text) => rendered.push(text),
            Piece::Field(Field::Head) => rendered.push(values.head),
            Piece::Field(Field::ShortHead) => {
                rendered.push(values.head.chars().take(SHORT_HEAD_LENGTH).collect::<String>());
            }
            Piece::Field(Field::TopLevel) => rendered.push(values.top_level),
            Piece::Field(Field::Environment(name)) => {
                let found = values.environment.iter().rev().find(|(candidate, _)| { return candidate == name.as_str(); });
                match found {
                    Some((_, value)) => rendered.push(value),
                    None => return Err(format!("the environment variable {name} is not set")),
                }
            }
        }
    }
    return Ok(rendered);
}

/// Template syntax and rendering are controlled in the test module.
#[cfg(test)]
#[path = "post_commit_template_tests.rs"]
mod tests;

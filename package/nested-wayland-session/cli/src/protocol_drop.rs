//! The `drop-file` command line:
//!  several paths,
//!  quoting for names with spaces,
//!  and an optional drop point.

/// Standard library items.
use std::path::PathBuf;

/// Sibling modules of this crate.
use super::{parse_f64, Command};

/// Most files one drop carries.
///
/// What:
///      `pub const MAX_FILES: usize = 256;`.
/// Why:
///       Bounds the payload a typo or a glob can produce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_FILES = 256;
/// ```
pub const MAX_FILES: usize = 256;

/// One token of the line and whether it was written in double quotes.
///
/// What:
///      `struct Token { text: String, quoted: bool }`.
/// Why:
///       Only an unquoted `at` is the position keyword,
///  so a file named `at` stays reachable as `"at"`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Token = { text: string; quoted: boolean };
/// ```
struct Token {
    /// The token with quotes and escapes resolved.
    text: String,
    /// Whether the token was written in double quotes.
    quoted: bool,
}

/// Split the line into tokens.
///
/// What:
///      `fn tokenize(line: &str) -> Result<Vec<Token>, String>`.
///  Whitespace separates tokens.
///  A token that starts with `"` runs to the next unescaped `"`;
///  inside it `\"` is a quote and `\\` a backslash.
///  Anything else is kept verbatim,
///  so an unquoted path with a backslash or a quote inside is unchanged.
/// Why:
///       Names with spaces need a delimiter,
///  and paths that never used one must keep working.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tokenize(line: string): Token[] { ... }
/// ```
fn tokenize(line: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut characters = line.chars().peekable();
    while let Some(next) = characters.peek().copied() {
        if next.is_whitespace() {
            characters.next();
            continue;
        }
        let mut text = String::new();
        let quoted = next == '"';
        if quoted {
            characters.next();
            let mut closed = false;
            while let Some(character) = characters.next() {
                match character {
                    '\\' if matches!(characters.peek(), Some('"' | '\\')) => {
                        text.extend(characters.next());
                    }
                    '"' => {
                        closed = true;
                        break;
                    }
                    other => text.push(other),
                }
            }
            if !closed {
                return Err("drop-file has a quoted path with no closing quote".to_string())
            }
            if characters.peek().is_some_and(|after| return !after.is_whitespace()) {
                return Err("drop-file needs whitespace after a closing quote".to_string())
            }
        } else {
            while let Some(character) = characters.peek().copied() {
                if character.is_whitespace() {
                    break;
                }
                text.push(character);
                characters.next();
            }
        }
        tokens.push(Token { text, quoted });
    }
    return Ok(tokens)
}

/// Parse the part of a `drop-file` line after the verb.
///
/// What:
///      `pub(super) fn parse(rest: &str) -> Result<Command, String>`.
///  Forms:
///  `PATH...`,
///  `PATH... at X Y`,
///  and the earlier `PATH X Y`
///  (exactly one unquoted path followed by two numbers).
///  Quote a path that contains whitespace:
///  `"/music/My Album/a b.mp3"`.
/// Why:
///       The earlier grammar split on whitespace,
///  so a path with a space became several wrong arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDrop(rest: string): Command { ... }
/// ```
pub(super) fn parse(rest: &str) -> Result<Command, String> {
    let tokens = tokenize(rest)?;
    let count = tokens.len();
    let mut point = None;
    let mut path_count = count;
    let keyword = count >= 3 && !tokens[count - 3].quoted && tokens[count - 3].text == "at";
    if keyword {
        let x = parse_f64(Some(tokens[count - 2].text.as_str()), "drop-file x")?;
        let y = parse_f64(Some(tokens[count - 1].text.as_str()), "drop-file y")?;
        point = Some((x, y));
        path_count = count - 3;
    } else if count == 3 && !tokens[1].quoted && !tokens[2].quoted && tokens[1].text.parse::<f64>().is_ok() {
        let x = parse_f64(Some(tokens[1].text.as_str()), "drop-file x")?;
        let y = parse_f64(Some(tokens[2].text.as_str()), "drop-file y")?;
        point = Some((x, y));
        path_count = 1;
    } else if count == 2 && !tokens[1].quoted && tokens[1].text.parse::<f64>().is_ok() {
        return Err("drop-file needs both x and y or neither".to_string())
    }
    if point.is_some_and(|(x, y)| return !x.is_finite() || !y.is_finite()) {
        return Err("drop-file coordinates must be finite".to_string())
    }
    if path_count == 0 {
        return Err("drop-file requires at least one path".to_string())
    }
    if path_count > MAX_FILES {
        return Err(format!("drop-file takes at most {MAX_FILES} paths"))
    }
    let paths = tokens.iter().take(path_count).map(|token| return PathBuf::from(&token.text)).collect();
    return Ok(Command::DropFile { paths, x: point.map(|(x, _)| return x), y: point.map(|(_, y)| return y) })
}

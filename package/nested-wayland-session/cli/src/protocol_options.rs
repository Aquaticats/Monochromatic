//! The `name=value` options that follow the positional arguments of the pointer commands.

/// Sibling modules of this crate.
use crate::key_chord::parse_chord;

/// Error text that is built only on failure.
use crate::lazy_error_text::result_or_text;

/// Positional arguments and `name=value` options split apart.
///
/// What:
///      `pub(super) struct Options<'a> { pub positional: Vec<&'a str>, pairs: Vec<(&'a str, &'a str)> }`.
/// Why:
///       `click 10 20 right mods=ctrl count=2` has a positional button and two options;
///  both orders and every omission must parse the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Options = { positional: string[]; pairs: Map<string, string> };
/// ```
pub(super) struct Options<'a> {
    /// The command verb,
    ///  named in every error.
    verb: &'a str,
    /// Tokens without an equals sign,
    ///  in order.
    pub positional: Vec<&'a str>,
    /// The `name=value` tokens,
    ///  in order.
    pairs: Vec<(&'a str, &'a str)>,
}

/// Split the rest of a command line into positional arguments and options.
///
/// What:
///      `pub(super) fn split<'a>(tokens: &mut SplitWhitespace<'a>, verb: &str, allowed: &[&str]) -> Result<Options<'a>, String>`.
///  Refuses an option name outside `allowed` and a repeated option,
///  naming the verb and the valid names.
/// Why:
///       A misspelled option must fail the command instead of being ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function split(tokens: string[], verb: string, allowed: string[]): Options { ... }
/// ```
pub(super) fn split<'a>(
    tokens: &mut std::str::SplitWhitespace<'a>,
    verb: &'a str,
    allowed: &[&str],
) -> Result<Options<'a>, String> {
    let mut positional = Vec::new();
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    for token in tokens {
        let Some((name, value)) = token.split_once('=') else {
            positional.push(token);
            continue;
        };
        if !allowed.contains(&name) {
            return Err(format!("{verb} does not take {name}=; its options are {}", allowed.join(", ")))
        }
        for (existing, _) in &pairs {
            if *existing == name {
                return Err(format!("{verb} option {name}= was given twice"))
            }
        }
        pairs.push((name, value));
    }
    return Ok(Options { verb, positional, pairs })
}

/// Typed readers for the options a command accepted.
impl Options<'_> {
    /// Read a whole-number option inside an inclusive range,
    ///  or its default when absent.
    ///
    /// What:
    ///      `pub(super) fn number(&self, name: &str, default: u64, range: (u64, u64)) -> Result<u64, String>`.
    /// Why:
    ///       Counts and delays need exact bounds so a typo cannot hold the control socket for hours.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// number(name: string, fallback: number, range: [number, number]): number { ... }
    /// ```
    pub(super) fn number(&self, name: &str, default: u64, range: (u64, u64)) -> Result<u64, String> {
        let mut found = None;
        for (key, value) in &self.pairs {
            if *key == name {
                found = Some(value);
                break;
            }
        }
        let Some(value) = found else {
            return Ok(default)
        };
        let verb = self.verb;
        let parsed = result_or_text(value.parse::<u64>(), format_args!("{verb} {name}= is not a whole number: {value}"))?;
        if parsed < range.0 || parsed > range.1 {
            return Err(format!("{verb} {name}= must be from {} to {}, got: {parsed}", range.0, range.1))
        }
        return Ok(parsed)
    }

    /// Read the `mods=` option as a validated modifier chord,
    ///  empty when absent.
    ///
    /// What:
    ///      `pub(super) fn modifiers(&self) -> Result<String, String>`.
    ///  Every name must resolve to a key.
    /// Why:
    ///       A bad modifier name fails the command before any key is pressed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// modifiers(): string { ... }
    /// ```
    pub(super) fn modifiers(&self) -> Result<String, String> {
        let mut found = None;
        for (key, value) in &self.pairs {
            if *key == "mods" {
                found = Some(value);
                break;
            }
        }
        let Some(value) = found else {
            return Ok(String::new())
        };
        if let Err(message) = parse_chord(value) {
            return Err(format!("{} mods= {message}", self.verb))
        }
        return Ok((*value).to_string())
    }
}

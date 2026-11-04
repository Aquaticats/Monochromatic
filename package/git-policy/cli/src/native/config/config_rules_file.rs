//! What:
//!  The `rulesFile` option of `security/forbidden-strings`:
//!  which files in the
//!       repository hold the scanner's private rules,
//!  each written relative to the
//!       repository's top level.
//!  This module checks one path;
//!  `config_policies.rs` reads the string and array forms and calls it for each path.
//! Why:
//!  The user decided on 2026-10-05 that configuration names the rules file first,
//!  then
//!      `FORBIDDEN_STRINGS_RULES`,
//!  then the default file.
//!  A configured name is checked
//!      when the configuration is read:
//!  it must stay inside the repository by its words
//!      alone,
//!  so a configuration cannot point the scanner at a file elsewhere on the
//!      machine.
//!  A name that is missing on disk is not a configuration error;
//!  loading
//!      the rules reports it when the policy runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const rulesFile = parseRulesFile(option); // throws RulesFileRefusal
//! ```

/// What:
///  Why a `rulesFile` value was refused.
///  `#[derive(...)]` generates copying,
///  debug
///       printing and `==`.
/// Why:
///   The configuration error names the key and explains this one reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RulesFileRefusal = 'empty' | 'absolute' | 'drive' | 'backslash' | 'nul' | 'empty-component' | 'dot-component';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RulesFileRefusal {
    /// The value is empty,
    ///  so it names no file.
    Empty,
    /// The value starts with `/`,
    ///  so it does not start at the repository's top level.
    Absolute,
    /// The value starts with a drive letter and a colon,
    ///  which Windows reads as another volume.
    Drive,
    /// The value holds a backslash,
    ///  which Windows reads as a separator.
    Backslash,
    /// The value holds a NUL character,
    ///  which no file name can hold.
    Nul,
    /// The value has an empty component:
    ///  two slashes in a row or a final slash.
    EmptyComponent,
    /// The value has a `.` or `..` component;
    ///  `..` could leave the repository.
    DotComponent,
}

/// What:
///  The explanation of one refusal,
///  completing the sentence "the value ...".
/// Why:
///   One place owns the words,
///  so every refusal reads the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalReason(refusal: RulesFileRefusal): string;
/// ```
pub fn rules_file_refusal_reason(refusal: RulesFileRefusal) -> &'static str {
    // `match` names every refusal, so a new one cannot be added without its words.
    match refusal {
        RulesFileRefusal::Empty => return "is empty",
        RulesFileRefusal::Absolute => return "starts with /",
        RulesFileRefusal::Drive => return "starts with a drive letter",
        RulesFileRefusal::Backslash => return "contains a backslash; use / between directories",
        RulesFileRefusal::Nul => return "contains a NUL character",
        RulesFileRefusal::EmptyComponent => {
            return "has an empty component (two slashes in a row or a trailing slash)";
        }
        RulesFileRefusal::DotComponent => return "has a . or .. component",
    }
}

/// What:
///  Whether one character is `/`,
///  the separator of the option's components.
/// Why:
///   A named predicate for `split`,
///  because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSlash = (character: string) => character === '/';
/// ```
fn is_slash(character: char) -> bool {
    return character == '/';
}

/// What:
///  Whether `text` starts with an ASCII letter and a colon,
///  as `C:` does.
/// Why:
///   Joining such a value to the repository's top level on Windows would name
///       another volume.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startsWithDrive = (text: string) => /^[A-Za-z]:/.test(text);
/// ```
fn starts_with_drive(text: &str) -> bool {
    let bytes: &[u8] = text.as_bytes();
    // `bytes.len() > 1` keeps the two indexed reads inside the text.
    return bytes.len() > 1 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
}

/// What:
///  Check one `rulesFile` value.
///  `Result<(), RulesFileRefusal>` is "accepted" or the
///       first reason it is not.
/// Why:
///   A value that is relative,
///  uses `/` between non-empty components,
///  and has no `.`
///       or `..` component names a file inside the repository on every platform,
///  whatever
///       the file system later holds.
///  The checks run in a fixed order so the first
///       reason reported is the same on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRulesFile(value: string): void; // throws RulesFileRefusal
/// ```
pub fn check_rules_file(value: &str) -> Result<(), RulesFileRefusal> {
    if value.is_empty() {
        return Err(RulesFileRefusal::Empty);
    }
    if value.starts_with('/') {
        return Err(RulesFileRefusal::Absolute);
    }
    if starts_with_drive(value) {
        return Err(RulesFileRefusal::Drive);
    }
    if value.contains('\\') {
        return Err(RulesFileRefusal::Backslash);
    }
    if value.contains('\0') {
        return Err(RulesFileRefusal::Nul);
    }
    for component in value.split(is_slash) {
        if component.is_empty() {
            return Err(RulesFileRefusal::EmptyComponent);
        }
        if component == "." || component == ".." {
            return Err(RulesFileRefusal::DotComponent);
        }
    }
    // `Ok(())` is success with no value.
    return Ok(());
}

/// Option-value controls stay out of the release executable.
#[cfg(test)]
#[path = "config_rules_file_tests.rs"]
mod tests;

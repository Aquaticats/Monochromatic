//! Reading what the toolchain lookup prints:
//!  the pinned `bin` JSON on standard output,
//!  and its messages on standard error.
//!
//! Everything here is a pure function of text,
//!  so it is tested with captured and hostile output and no toolchain.

/// The answer is one JSON object.
use serde_json::Value;
/// What:
///  `PathBuf` is an owned filesystem path.
/// Why:
///  Directories are checked against the file system and returned as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::{Path, PathBuf};

/// Longest message kept from the lookup,
///  in characters.
///  Chosen,
///  not measured:
///  the longest message seen was an uncaught error of about 400 characters
///  with its stack.
pub const MESSAGE_LIMIT: usize = 1000;

/// What:
///  The tool directories in the output of `mpm-shadow bin --json`:
///  the `pathDirs` of the pinned answer,
///  kept only when absolute and an existing directory,
///  first occurrence only.
///  `Result<Vec<PathBuf>, String>` is the list or the reason the output is not that answer.
/// Why:
///  A relative entry would depend on the working directory,
///  and an entry that is not a directory cannot hold a program.
///  The pinned shape (`exposureDir`,
///  `pathDirs`,
///  `bins`) is checked in full:
///  the exposure directory leads `pathDirs` and every linked binary of `bins`
///  sits in it,
///  so the directories alone answer where the tools are,
///  and a change of the published interface must fail here
///  instead of half-reading an answer of a new shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function binPaths(json: string): string[] // throws when json is not the pinned answer
/// ```
pub fn bin_paths(json: &str) -> Result<Vec<PathBuf>, String> {
    let parsed: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => {
            return Err(format!("the bin-paths answer is not JSON: {error}"));
        }
    };
    let Some(answer) = parsed.as_object() else {
        return Err("the bin-paths answer is not a JSON object".to_string());
    };
    let Some(exposure) = answer.get("exposureDir").and_then(Value::as_str) else {
        return Err("the bin-paths answer has no exposureDir string".to_string());
    };
    if exposure.is_empty() || Path::new(exposure).is_absolute() {
        return Err(
            "the bin-paths answer names an exposure directory that is not relative to the worktree"
                .to_string(),
        );
    }
    let Some(dirs) = answer.get("pathDirs").and_then(Value::as_array) else {
        return Err("the bin-paths answer has no pathDirs array".to_string());
    };
    let Some(bins) = answer.get("bins").and_then(Value::as_object) else {
        return Err("the bin-paths answer has no bins object".to_string());
    };
    for (name, path) in bins.iter() {
        if path.as_str().is_none() {
            return Err(format!("the bin-paths answer names {name} without a path"));
        }
    }
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in dirs {
        let Some(line) = entry.as_str() else {
            return Err("an entry of the bin-paths answer's pathDirs is not a path".to_string());
        };
        let path = PathBuf::from(line);
        if !path.is_absolute() || !path.is_dir() {
            tracing::debug!(
                entry = line,
                "the bin-paths answer printed a tool directory that is not used"
            );
            continue;
        }
        if !found.contains(&path) {
            found.push(path);
        }
    }
    return Ok(found);
}

/// What:
///  Whether a line is the run's own bookkeeping rather than a message about the project:
///  the version footer node prints after an uncaught error.
/// Why:
///  The version footer says nothing about the cause
///  and appears on every failed run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNoise = (line: string) => /^Node\.js v\d/.test(line);
/// ```
fn is_noise(line: &str) -> bool {
    return line.starts_with("Node.js v");
}

/// What:
///  The lookup's standard error as one message:
///  escape sequences and control characters removed,
///  bookkeeping lines dropped,
///  the rest verbatim,
///  cut at `MESSAGE_LIMIT` characters.
/// Why:
///  The text is shown to the reader as the lookup's exact words,
///  and it may contain project-supplied names,
///  so nothing that moves the cursor or recolors the screen is passed on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function message(stderr: string): string
/// ```
pub fn message(stderr: &str) -> String {
    let mut plain = String::new();
    let mut characters = stderr.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            // A control sequence is `ESC [`, parameter bytes, then one final byte from `@` to `~`;
            // an operating-system command is `ESC ]` up to a bell or `ESC \`.
            match characters.next() {
                Some('[') => {
                    for next in characters.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    for next in characters.by_ref() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' {
                            characters.next_if_eq(&'\\');
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        if character == '\n' || character == '\t' || !character.is_control() {
            plain.push(character);
        }
    }
    let mut kept: Vec<&str> = Vec::new();
    for raw in plain.lines() {
        let line = raw.trim_end();
        if !line.is_empty() && !is_noise(line) {
            kept.push(line);
        }
    }
    let joined = kept.join("\n");
    if joined.chars().count() <= MESSAGE_LIMIT {
        return joined;
    }
    let cut: String = joined.chars().take(MESSAGE_LIMIT).collect();
    return format!("{cut}...");
}

/// The pinned answer,
///  messages,
///  and the noise removal on captured and hostile text.
#[cfg(test)]
#[path = "parse_tests.rs"]
mod tests;

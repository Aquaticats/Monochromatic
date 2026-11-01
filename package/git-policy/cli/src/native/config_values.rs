//! What:
//!  Typed readers over parsed JSONC values,
//!  each naming the key it rejects.
//! Why:
//!  Every schema site reports the same actionable shape:
//!  which key,
//!  what was found,
//!      what is accepted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // readBoolean(value, 'hooks.concurrentCommits') throws ConfigError naming that key.
//! ```

/// What:
///  `use super::...` imports from a sibling module of this crate.
/// Why:
///   Every reader fails with the configuration error type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ConfigError } from './config-error.ts';
/// ```
use super::config_error::ConfigError;
/// What:
///  Import the repository's JSONC value model and its UTF-16 decoder.
/// Why:
///   The schema reads the parser's exact values;
///  it never re-parses text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type JsoncEntry, type JsoncValue, unitsToString } from 'jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncEntry, JsoncKind, JsoncValue, units_to_string};

/// What:
///  `u64` is an unsigned 64-bit integer.
///  Siblings:
///  `u32`,
///  `usize`,
///  `i64`,
///  `f64`.
///       The value is JavaScript's `Number.MAX_SAFE_INTEGER` (2^53 - 1).
/// Why:
///   The incumbent accepted only safe integers,
///  and journals written by either
///       implementation must agree;
///  `u64` holds that bound exactly,
///  `u32` cannot,
///  and
///       `f64` would reintroduce rounding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;
/// ```
pub(crate) const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// What:
///  Join a parent key path and a member key with a dot;
///  the top level has no parent.
/// Why:
///   Diagnostics name nested keys the way the documentation writes them
///       (`indexLock.unprovenOwnerTimeoutMs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function memberPath(parent: string, key: string): string;
/// ```
pub(crate) fn member_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        // `String::from` copies the borrowed key into an owned string for the caller.
        return String::from(key);
    }
    // What: `format!` builds an owned `String` from a template; `{parent}` interpolates.
    // Why:  The joined path outlives both borrowed inputs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `${parent}.${key}`;
    // ```
    return format!("{parent}.{key}");
}

/// What:
///  Describe a value's JSON type in words for a diagnostic.
/// Why:
///   "found a string" tells the author what to change without reading the schema.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindName(value: JsoncValue): string;
/// ```
pub(crate) fn kind_name(value: &JsoncValue) -> &'static str {
    // What: `match &value.kind { ... }` picks the arm whose pattern fits the payload variant.
    //       `{ .. }` ignores the variant's fields; `=>` separates pattern from result.
    // Why:  The compiler rejects this function if a JSON kind is ever added and not named.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (value.kind) { case 'null': return 'null'; ... }
    // ```
    match &value.kind {
        JsoncKind::Null => return "null",
        JsoncKind::Boolean { .. } => return "a boolean",
        JsoncKind::Number { .. } => return "a number",
        JsoncKind::Text { .. } => return "a string",
        JsoncKind::Array { .. } => return "an array",
        JsoncKind::Record { .. } => return "an object",
    }
}

/// What:
///  Build the failure for a value of the wrong JSON type at a named key.
/// Why:
///   Null gets its own wording:
///  it is valid JSON but never a cli-git setting,
///  and
///       the fix (remove the key) differs from a plain type mismatch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrongKind(value: JsoncValue, path: string, expected: string): ConfigError;
/// ```
pub(crate) fn wrong_kind(value: &JsoncValue, path: &str, expected: &str) -> ConfigError {
    // The top level has no key name, so it is described instead.
    let subject: String = if path.is_empty() {
        String::from("The configuration document")
    } else {
        format!("Configuration key {path}")
    };
    // What: `if let PATTERN = value` runs the block only when the payload is that variant.
    // Why:  Only null needs the "remove the key" remedy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value.kind === 'null') return new ConfigError(`${subject} must not be null; ...`);
    // ```
    if let JsoncKind::Null = value.kind {
        return ConfigError::new(
            format!(
                "{subject} must not be null; remove it to use the default or give it {expected}."
            )
            .as_str(),
        );
    }
    return ConfigError::new(
        format!("{subject} must be {expected}, found {}.", kind_name(value)).as_str(),
    );
}

/// What:
///  Decode every member key of one object and reject repeats.
///       `&[JsoncEntry]` borrows the parser's member list;
///  `Vec<String>` is an owned,
///       growable list (sibling `&[String]` would borrow one that does not exist yet).
///       `Result<T, E>` is "value or error":
///  `Ok(keys)` or `Err(error)`.
/// Why:
///   JSON allows a key twice,
///  and `"hooks"` equals `"hooks"` after decoding;
///       two definitions of one setting have no unambiguous meaning.
///  The returned keys
///       are parallel to `entries`,
///  so callers index both by position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function memberKeys(entries: JsoncEntry[], path: string): string[];
/// ```
pub(crate) fn member_keys(entries: &[JsoncEntry], path: &str) -> Result<Vec<String>, ConfigError> {
    // What: `Vec::<String>::with_capacity(n)` makes an empty list with room for `n` items;
    //       `mut` allows pushing to it.
    // Why:  One slot per member avoids regrowing the list.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const keys: string[] = [];
    // ```
    let mut keys: Vec<String> = Vec::<String>::with_capacity(entries.len());
    for entry in entries {
        // What: `match` on the decoder's `Result` separates text from a decode failure.
        //       `&entry.key.units` lends the decoded UTF-16 units without copying them.
        // Why:  A lone surrogate is not text; it must fail here, not become "?" later.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let key: string; try { key = decode(units); } catch { throw new ConfigError(...); }
        // ```
        let key: String = match units_to_string(&entry.key.units) {
            Ok(decoded) => decoded,
            Err(_) => {
                // What: `Err(...)` is the failure variant of `Result`; `return` leaves early.
                // Why:  One undecodable key invalidates the whole configuration.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // throw new ConfigError('... is not valid Unicode text ...');
                // ```
                return Err(ConfigError::new(
                    format!(
                        "A key under {} is not valid Unicode text (unpaired UTF-16 surrogate): {}.",
                        container_name(path),
                        String::from_utf16_lossy(&entry.key.units)
                    )
                    .as_str(),
                ));
            }
        };
        // `.contains(&key)` borrows the new key to compare it with the earlier ones.
        if keys.contains(&key) {
            return Err(ConfigError::new(
                format!(
                    "Configuration key {} is defined more than once; keep exactly one definition.",
                    member_path(path, key.as_str())
                )
                .as_str(),
            ));
        }
        keys.push(key);
    }
    // `Ok(keys)` is the success variant carrying the decoded keys.
    return Ok(keys);
}

/// What:
///  Name an object for a diagnostic about one of its keys.
/// Why:
///   The top-level object has no key path of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function containerName(path: string): string;
/// ```
fn container_name(path: &str) -> String {
    if path.is_empty() {
        return String::from("the configuration document");
    }
    return String::from(path);
}

/// What:
///  Require a JSON boolean at a named key.
/// Why:
///   `"true"` and `1` are not switches;
///  coercing them would hide typing mistakes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function boolean(value: JsoncValue, path: string): boolean;
/// ```
pub(crate) fn boolean(value: &JsoncValue, path: &str) -> Result<bool, ConfigError> {
    // What: The pattern `{ value: flag }` copies the variant's `value` field into `flag`.
    // Why:  The payload is read only when the kind really is a boolean.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value.kind === 'boolean') return value.value;
    // ```
    if let JsoncKind::Boolean { value: flag } = value.kind {
        return Ok(flag);
    }
    return Err(wrong_kind(value, path, "true or false"));
}

/// What:
///  Require a JSON string at a named key and decode it to Rust text.
/// Why:
///   Severities,
///  rule names and patterns are text;
///  numbers and arrays are mistakes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function text(value: JsoncValue, path: string): string;
/// ```
pub(crate) fn text(value: &JsoncValue, path: &str) -> Result<String, ConfigError> {
    // What: `let Some(units) = ... else { return ... };` unwraps the present case or exits.
    // Why:  Absence of string units means the value is another JSON type.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const units = value.textUnits(); if (units === undefined) throw wrongKind(...);
    // ```
    let Some(units) = value.text_units() else {
        return Err(wrong_kind(value, path, "a string"));
    };
    match units_to_string(units) {
        Ok(decoded) => return Ok(decoded),
        Err(_) => {
            return Err(ConfigError::new(
                format!(
                    "Configuration key {path} is not valid Unicode text (unpaired UTF-16 surrogate): {}.",
                    String::from_utf16_lossy(units)
                )
                .as_str(),
            ));
        }
    }
}

/// What:
///  Require an array of strings at a named key.
/// Why:
///   Rule lists and exclusion patterns never coerce other types into text;
///  the
///       failing element's index is part of the reported key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function strings(value: JsoncValue, path: string): string[];
/// ```
pub(crate) fn strings(value: &JsoncValue, path: &str) -> Result<Vec<String>, ConfigError> {
    let Some(elements) = value.elements() else {
        return Err(wrong_kind(value, path, "an array of strings"));
    };
    let mut result: Vec<String> = Vec::<String>::with_capacity(elements.len());
    // `.iter().enumerate()` yields `(index, element)` pairs; `usize` is the index type.
    for (index, element) in elements.iter().enumerate() {
        let element_path: String = format!("{path}[{index}]");
        // What: A trailing `?` returns the `Err` to our caller, or unwraps the `Ok` value.
        // Why:  One malformed element invalidates the list; no partial list is returned.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // result.push(text(element, elementPath)); // a throw propagates
        // ```
        result.push(text(element, element_path.as_str())?);
    }
    return Ok(result);
}

/// What:
///  Require an exact integer between `minimum` and JavaScript's safe-integer bound.
/// Why:
///   Timeouts and counters are whole numbers;
///  `1.5`,
///  `-1`,
///  `"3"` and values past
///       2^53 - 1 are rejected instead of being rounded.
///  `1e3` and `2.0` are exact
///       integers and stay accepted,
///  as they were by the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function safeInteger(value: JsoncValue, path: string, minimum: number): number;
/// ```
pub(crate) fn safe_integer(
    value: &JsoncValue,
    path: &str,
    minimum: u64,
) -> Result<u64, ConfigError> {
    let expected: String = format!("a whole number from {minimum} to {MAX_SAFE_INTEGER}");
    // `identity` is the parser's exact value; `..` skips the original spelling.
    let JsoncKind::Number { identity, .. } = &value.kind else {
        return Err(wrong_kind(value, path, expected.as_str()));
    };
    let failure: ConfigError = ConfigError::new(
        format!(
            "Configuration key {path} must be {expected}, found {}.",
            value.number_token().unwrap_or("a number")
        )
        .as_str(),
    );
    if identity.is_negative() {
        return Err(failure);
    }
    // Every zero spelling (`0`, `-0`, `0e9`) is exactly zero.
    let exact: u64 = if identity.is_zero() {
        0
    } else {
        // What: `.parse::<u64>()` converts decimal text to `u64`; `::<u64>` names the target
        //       type explicitly. Both parses return `Result`.
        // Why:  The identity stores canonical digits and a decimal exponent as text, so no
        //       floating-point rounding occurs; a negative exponent means a fraction.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const coefficient = BigInt(digits); const exponent = Number(exponentText);
        // ```
        let (Ok(coefficient), Ok(exponent)) = (
            identity.digits().parse::<u64>(),
            identity.exponent().parse::<u32>(),
        ) else {
            return Err(failure);
        };
        // What: `checked_pow`/`checked_mul` return `None` on overflow instead of wrapping.
        // Why:  `1e400` must be rejected, not silently truncated to a small number.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const exact = coefficient * 10n ** BigInt(exponent); // BigInt never overflows
        // ```
        let Some(scale) = 10_u64.checked_pow(exponent) else {
            return Err(failure);
        };
        let Some(product) = coefficient.checked_mul(scale) else {
            return Err(failure);
        };
        product
    };
    if exact < minimum || exact > MAX_SAFE_INTEGER {
        return Err(failure);
    }
    return Ok(exact);
}

/// Boundary controls for every reader stay out of the release executable.
#[cfg(test)]
#[path = "config_values_tests.rs"]
mod tests;

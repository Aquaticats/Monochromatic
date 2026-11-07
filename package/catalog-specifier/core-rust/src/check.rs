//! What:
//! The checks:
//! from a manifest path and its bytes to a list of findings,
//! following the README contract step by step.
//! Why:
//! Every consumer must read a manifest the same way,
//! so the byte handling,
//! the duplicate detection and the section walk
//! live here once and nothing else in the repository re-implements them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // check.ts
//! export function checkManifest(path: string, bytes: Uint8Array): Finding[]
//! export function checkManifestPath(path: string): Finding | undefined
//! export function checkManifestBytes(path: string, bytes: Uint8Array): Finding[]
//! ```

/// What:
/// `use std::collections::HashSet;` imports a set type:
/// a collection of unique values with fast membership tests.
/// Why:
/// Duplicate detection needs "have I seen this name already" in constant time,
/// so a long dependency list stays linear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No import needed: `Set` is global in TS.
/// ```
use std::collections::HashSet;

/// What:
/// Imports the parser and the tree types from the repository's JSONC crate.
/// `parse_jsonc` turns text into a tree whose object members stay in a vector,
/// so duplicate keys stay visible.
/// `units_to_string` converts UTF-16 code units to a Rust `String` and fails on a lone surrogate.
/// Why:
/// A map-based JSON parser would silently keep only the last duplicate,
/// which the contract forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, unitsToString, type JsoncKey, type JsoncKind, type JsoncValue } from '@monochromatic-dev/jsonc-edit-core-ts';
/// ```
use monochromatic_jsonc_edit::{JsoncKey, JsoncKind, JsoncValue, parse_jsonc, units_to_string};

/// Imports the finding types from the sibling module.
use crate::finding::{Finding, FindingKind};
/// Imports the grammar test and the section kind from the sibling module.
use crate::grammar::{SectionKind, is_allowed_specifier};

/// What:
/// `const CHECKED_SECTIONS: [(&str, SectionKind); 4]` is a fixed-size array (length 4,
/// known at compile time) of pairs.
/// `(&str, SectionKind)` is a tuple:
/// two values side by side.
/// The sibling collection `Vec<T>` can grow,
/// and `&[T]` is a
/// borrowed view of either;
/// a fixed array is used because the four section names never change.
/// Why:
/// One table drives both "is this root member a checked section" and "which grammar applies".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHECKED_SECTIONS: readonly [string, SectionKind][] = [['dependencies', 'install'], /* ... */];
/// ```
const CHECKED_SECTIONS: [(&str, SectionKind); 4] = [
    ("dependencies", SectionKind::Install),
    ("devDependencies", SectionKind::Install),
    ("optionalDependencies", SectionKind::Install),
    ("peerDependencies", SectionKind::Peer),
];

/// The file names pnpm reads as manifests but that this crate cannot parse.
const UNSUPPORTED_FILE_NAMES: [&str; 3] = ["package.json5", "package.yaml", "package.yml"];

/// What:
/// `const BYTE_ORDER_MARK: [u8; 3]` is a fixed array of three `u8` (unsigned 8-bit integers,
/// 0 to 255;
/// the siblings are
/// `i8`,
/// `u16`,
/// `u32`).
/// `0xEF` is a number written in base 16.
/// Why:
/// `u8` because file contents are bytes.
/// These three bytes are the UTF-8 encoding of the optional marker at a file's start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BYTE_ORDER_MARK = new Uint8Array([0xef, 0xbb, 0xbf]);
/// ```
const BYTE_ORDER_MARK: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// What:
/// `fn bare_finding(kind: FindingKind, path: &str) -> Finding` builds a finding with every optional field empty.
/// `path: &str` borrows the caller's text;
/// the function copies it into the owned `String` the finding holds.
/// Why:
/// Callers then fill only the fields their kind uses,
/// so no call site repeats five `None`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bareFinding(kind: FindingKind, path: string): Finding { return { kind, path }; }
/// ```
fn bare_finding(kind: FindingKind, path: &str) -> Finding {
    // What: `path.to_string()` allocates a new owned `String` holding a copy of the borrowed text.
    // `None` is the empty case of `Option`, the Rust stand-in for `undefined`.
    // Why: The struct owns its path, so it can outlive the call; every optional field starts absent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { kind, path, section: undefined, name: undefined, specifier: undefined, detail: undefined };
    // ```
    return Finding {
        kind,
        path: path.to_string(),
        section: None,
        name: None,
        specifier: None,
        detail: None,
    };
}

/// What:
/// `fn key_text(key: &JsoncKey) -> String` turns an object key into text.
/// `&JsoncKey` borrows the key read-only.
/// `String::from_utf16_lossy` converts UTF-16 code units to a `String`,
/// replacing any lone surrogate with the
/// replacement character instead of failing.
/// Why:
/// A manifest key may hold an escaped unpaired surrogate that a Rust `String` cannot express.
/// Names are only shown and
/// compared,
/// so the lossy form is enough;
/// two different lone surrogates would compare equal,
/// which only over-reports a duplicate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyText(key: JsoncKey): string { return key.value; }
/// ```
fn key_text(key: &JsoncKey) -> String {
    return String::from_utf16_lossy(&key.units);
}

/// What:
/// `fn section_kind_of(name: &str) -> Option<SectionKind>` looks a root member name up in the table.
/// Returns `Some(kind)` when the name is a checked section and `None` otherwise.
/// Why:
/// The root walk ignores every member that is not one of the four sections.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sectionKindOf(name: string): SectionKind | undefined
/// ```
fn section_kind_of(name: &str) -> Option<SectionKind> {
    // What: `for (section_name, kind) in CHECKED_SECTIONS` loops over the array and unpacks each pair into two variables.
    // Why: The table is tiny, so a plain loop beats building a map.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [sectionName, kind] of CHECKED_SECTIONS) { if (sectionName === name) return kind; }
    // ```
    for (section_name, kind) in CHECKED_SECTIONS {
        if section_name == name {
            // What: `Some(kind)` wraps the value in the present case of `Option`.
            // Why: Tell the caller "yes, this is a checked section, and here is its grammar".
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return kind;
            // ```
            return Some(kind);
        }
    }
    // What: `None` is the empty case of `Option`. Why: The name is not a checked section.
    return None;
}

/// What:
/// `fn check_specifier(...)` checks one dependency value and appends at most one finding.
/// `findings: &mut Vec<Finding>` borrows the caller's growable list for writing (`&mut`),
/// so pushes are visible to the caller.
/// `Vec<T>` is the growable array;
/// the siblings are `&[T]` (a borrowed view) and `[T; N]` (fixed length).
/// Why:
/// `&mut Vec` and not a returned `Vec` so the section walk can keep one list in source order without merging.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkSpecifier(path: string, section: string, name: string, kind: SectionKind, value: JsoncValue, findings: Finding[]): void
/// ```
fn check_specifier(
    path: &str,
    section: &str,
    name: &str,
    kind: SectionKind,
    value: &JsoncValue,
    findings: &mut Vec<Finding>,
) {
    // What: `match &value.kind { JsoncKind::Text { units, .. } => ..., _ => ... }` picks a branch by the value's JSON type.
    // `&value.kind` borrows the field; `{ units, .. }` binds the `units` field and ignores the others; `_` matches every
    // other case (number, boolean, null, array, object).
    // Why: Only a JSON string can be a specifier; anything else is a `non-string-specifier` finding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value.kind.type !== 'string') { findings.push({ kind: 'non-string-specifier', /* ... */ }); return; }
    // ```
    match &value.kind {
        JsoncKind::Text { units, .. } => {
            // What: `units_to_string(units)` returns `Result<String, FromUtf16Error>`: `Ok(text)` or `Err(error)`.
            // `&units` is not needed because `units` is already a borrow here.
            // Why: A lone surrogate has no UTF-8 form, so such a specifier cannot be a catalog reference and is refused.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // let text: string | undefined; try { text = unitsToString(units); } catch { text = undefined; }
            // ```
            let decoded = units_to_string(units);
            // What: `match &decoded { Ok(text) => ..., Err(_) => false }` picks a branch by the result's case. `&decoded`
            // borrows it; `Ok(text)` binds the inner `String`; `Err(_)` matches a failure and ignores its detail.
            // `text.as_str()` borrows the `String` as a `&str`.
            // Why: Accept only a decodable string that the grammar allows; a lone surrogate is never allowed.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const allowed = text !== undefined && isAllowedSpecifier(kind, text);
            // ```
            let allowed: bool = match &decoded {
                Ok(text) => is_allowed_specifier(kind, text.as_str()),
                Err(_) => false,
            };
            if allowed {
                return;
            }
            // What: `String::from_utf16_lossy(units)` is the lossy conversion again, used only to show the bad specifier.
            // Why: The finding must print something even when the text held a lone surrogate.
            let shown: String = String::from_utf16_lossy(units);
            // What: `Finding { ..., ..bare_finding(...) }` is struct-update syntax: take every field from the base value
            // after `..` except the ones written before it. `Some(...)` wraps a value in the present case of `Option`.
            // Why: Set only the four fields this kind uses without repeating the other `None`s.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // findings.push({ ...bareFinding('disallowed-specifier', path), section, name, specifier: shown });
            // ```
            findings.push(Finding {
                section: Some(section.to_string()),
                name: Some(name.to_string()),
                specifier: Some(shown),
                ..bare_finding(FindingKind::DisallowedSpecifier, path)
            });
        }
        _ => {
            findings.push(Finding {
                section: Some(section.to_string()),
                name: Some(name.to_string()),
                ..bare_finding(FindingKind::NonStringSpecifier, path)
            });
        }
    }
}

/// What:
/// `fn check_section(...)` checks one checked section's value:
/// it must be an object,
/// and each member is walked in order.
/// Why:
/// The section-level rules (malformed value,
/// duplicate names) and the per-entry rule meet here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkSection(path: string, section: string, kind: SectionKind, value: JsoncValue, findings: Finding[]): void
/// ```
fn check_section(path: &str, section: &str, kind: SectionKind, value: &JsoncValue, findings: &mut Vec<Finding>) {
    // What: `value.entries()` returns `Some(members)` for an object and `None` for any other JSON type.
    // `let ... else { ... }` binds on `Some` and otherwise runs the block, which must leave the function.
    // Why: A section holding `null`, an array, a string, a number or a boolean is one `malformed-section` finding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const members = value.entries(); if (members === undefined) { findings.push(/* malformed */); return; }
    // ```
    let Some(members) = value.entries() else {
        findings.push(Finding {
            section: Some(section.to_string()),
            ..bare_finding(FindingKind::MalformedSection, path)
        });
        return;
    };
    // What: `HashSet::new()` makes an empty set; `let mut` allows the set to change. The type `HashSet<String>` is inferred.
    // Why: Track dependency names already seen in this section.
    let mut seen_names: HashSet<String> = HashSet::new();
    for member in members {
        let name: String = key_text(&member.key);
        // What: `seen_names.insert(...)` adds the name and returns `false` if it was already present; `!` negates.
        // `.clone()` makes an independent copy of the `String` so the set owns one and `name` keeps the other.
        // Why: Every occurrence after the first is a `duplicate-key` finding, and it is reported before that entry's own result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (seenNames.has(name)) findings.push(/* duplicate */); else seenNames.add(name);
        // ```
        if !seen_names.insert(name.clone()) {
            findings.push(Finding {
                section: Some(section.to_string()),
                name: Some(name.clone()),
                ..bare_finding(FindingKind::DuplicateKey, path)
            });
        }
        check_specifier(path, section, &name, kind, &member.value, findings);
    }
}

/// What:
/// `pub fn check_manifest_path(path: &str) -> Option<Finding>` tests only the file name.
/// Returns `Some(finding)` for `package.json5`,
/// `package.yaml` and `package.yml`,
/// and `None` for every other name.
/// Why:
/// pnpm reads those formats and this crate has no parser for them,
/// so the name alone is refused.
/// The comparison is case
/// sensitive on the last path segment,
/// so `my-package.yaml` and `package.yaml.bak` pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkManifestPath(path: string): Finding | undefined
/// ```
pub fn check_manifest_path(path: &str) -> Option<Finding> {
    // What: `path.rsplit('/')` splits from the right; `.next()` takes the first piece, which is the last segment, as an
    // `Option<&str>`; `.unwrap_or(path)` falls back to the whole path when there is no piece.
    // Why: Match the file name, never a directory name such as `package.yaml/package.json`.
    // Gotcha: `.unwrap_or(...)` on `Option` hides the `None` case; `rsplit` always yields one piece, so it cannot trigger.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fileName = path.split('/').at(-1) ?? path;
    // ```
    let file_name: &str = path.rsplit('/').next().unwrap_or(path);
    // What: `UNSUPPORTED_FILE_NAMES.contains(&file_name)` is true when the array holds an equal `&str`.
    // `&file_name` lends a reference to the variable because `contains` compares by reference.
    // Why: Exact, case-sensitive name match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (UNSUPPORTED_FILE_NAMES.includes(fileName)) return bareFinding('unsupported-manifest-format', path);
    // ```
    if UNSUPPORTED_FILE_NAMES.contains(&file_name) {
        return Some(bare_finding(FindingKind::UnsupportedManifestFormat, path));
    }
    return None;
}

/// What:
/// `pub fn check_manifest_bytes(path: &str, bytes: &[u8]) -> Vec<Finding>` checks a `package.json`'s contents.
/// `&[u8]` is a borrowed view of bytes (the siblings are `Vec<u8>`,
/// which owns them,
/// and `[u8; N]`,
/// fixed length).
/// Why:
/// `&[u8]` and not `&str` because the contract starts from raw file bytes:
/// invalid UTF-8 is itself a finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkManifestBytes(path: string, bytes: Uint8Array): Finding[]
/// ```
pub fn check_manifest_bytes(path: &str, bytes: &[u8]) -> Vec<Finding> {
    // What: `bytes.strip_prefix(&BYTE_ORDER_MARK[..])` returns `Some(rest)` when the bytes start with the marker.
    // `[..]` slices the whole array into a `&[u8]`. `match` picks the stripped bytes or the original.
    // Why: pnpm removes one leading marker, so the contract does too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const body = startsWithBom(bytes) ? bytes.subarray(3) : bytes;
    // ```
    let body: &[u8] = match bytes.strip_prefix(&BYTE_ORDER_MARK[..]) {
        Some(rest) => rest,
        None => bytes,
    };
    // What: `std::str::from_utf8(body)` checks the bytes are valid UTF-8 and returns `Ok(&str)` or `Err(error)`.
    // `return vec![...]` builds a one-element `Vec` with the `vec!` macro (a macro is code the compiler expands for us).
    // Why: Bytes that are not UTF-8 cannot be a manifest; report it as one `invalid-manifest` finding and stop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let text: string; try { text = new TextDecoder('utf-8', { fatal: true }).decode(body); } catch { return [invalid(path, 'not valid UTF-8')]; }
    // ```
    let Ok(text) = std::str::from_utf8(body) else {
        return vec![Finding {
            detail: Some("not valid UTF-8".to_string()),
            ..bare_finding(FindingKind::InvalidManifest, path)
        }];
    };
    // What: `parse_jsonc(text)` returns `Result<JsoncValue, JsoncParseError>`. `match` handles both cases.
    // `error.to_string()` builds the error's text. Why: Unparseable text, including empty input, is one `invalid-manifest`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let root: JsoncValue; try { root = parseJsonc(text); } catch (error) { return [invalid(path, String(error))]; }
    // ```
    let root: JsoncValue = match parse_jsonc(text) {
        Ok(value) => value,
        Err(error) => {
            return vec![Finding {
                detail: Some(error.to_string()),
                ..bare_finding(FindingKind::InvalidManifest, path)
            }];
        }
    };
    // What: `root.entries()` is `None` unless the root is an object; `let ... else` leaves the function in that case.
    // Why: A root that is an array or a scalar is one `invalid-manifest` finding.
    let Some(root_members) = root.entries() else {
        return vec![Finding {
            detail: Some("the root value is not an object".to_string()),
            ..bare_finding(FindingKind::InvalidManifest, path)
        }];
    };

    // What: `Vec::new()` makes an empty growable list. `let mut` lets it change. The type comes from the annotation.
    // Why: Collect findings in source order as the root members are walked.
    let mut findings: Vec<Finding> = Vec::new();
    let mut seen_sections: HashSet<String> = HashSet::new();
    for member in root_members {
        let member_name: String = key_text(&member.key);
        // What: `let Some(kind) = ... else { continue; }` binds on `Some`; otherwise `continue` skips to the next member.
        // Why: Members that are not checked sections (name, scripts, resolutions, ...) are ignored.
        let Some(kind) = section_kind_of(&member_name) else {
            continue;
        };
        if !seen_sections.insert(member_name.clone()) {
            findings.push(Finding {
                section: Some(member_name.clone()),
                ..bare_finding(FindingKind::DuplicateKey, path)
            });
        }
        check_section(path, &member_name, kind, &member.value, &mut findings);
    }
    return findings;
}

/// What:
/// `pub fn check_manifest(path: &str, bytes: &[u8]) -> Vec<Finding>` is the single entry most callers use:
/// refuse an unsupported file name without reading the bytes,
/// otherwise check the bytes.
/// Why:
/// The contract says unsupported names are never read,
/// so one function makes that order impossible to get wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkManifest(path: string, bytes: Uint8Array): Finding[] {
///   return checkManifestPath(path) ? [checkManifestPath(path)!] : checkManifestBytes(path, bytes);
/// }
/// ```
pub fn check_manifest(path: &str, bytes: &[u8]) -> Vec<Finding> {
    // What: `if let Some(finding) = check_manifest_path(path)` runs the block only when the name is unsupported.
    // Why: Return that one finding and skip reading the bytes.
    if let Some(finding) = check_manifest_path(path) {
        return vec![finding];
    }
    return check_manifest_bytes(path, bytes);
}

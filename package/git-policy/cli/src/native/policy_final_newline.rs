//! What:
//!  The built-in `final-newline` policy:
//!  a non-empty UTF-8 text file without NUL
//!       bytes must end with exactly one LF byte.
//! Why:
//!  The rule,
//!  its finding and its generated and fixture path families are the installed
//!      wrapper's (`final-newline-policy.ts`,
//!  `final-newline-normalize.ts`),
//!  so the native
//!      wrapper reports the same files with the same words.
//!  A file whose `text` Git attribute is unset (`-text`, which `binary` includes) keeps its exact bytes,
//!      which the installed wrapper could only do by path;
//!  the attribute travels with a package that moves,
//!      where a path list goes stale.
//!  The normalized bytes are
//!      also the full-content correction a fixing lifecycle applies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const replacement = normalizeFinalNewline(bytes); if (replacement.kind === 'changed') findings.push(...);
//! ```

/// Import the candidate's mode.
use super::candidate_object::CandidateMode;
/// Import the candidate's change kinds.
use super::candidate_record::CandidateChange;
/// Import the candidate and version types.
use super::candidate_version::{Candidate, CandidateVersion};
/// Import the event form of a pathname.
use super::event_path::EventPath;
/// Import the lifecycle's candidates and the correction a fix applies.
use super::policy_content::{ContentState, Correction, LifecycleContent};
/// Import the finding and outcome types of a check.
use super::policy_engine::{PolicyFinding, PolicyOutcome};
/// Import the lifecycle points;
///  only a direct fix corrects.
use super::policy_trigger::Trigger;
/// Import the facts interface that prepares candidates.
use super::repository_facts::RepositoryFacts;
/// `HashSet<T>` is a collection of distinct values with constant-time membership tests.
use std::collections::HashSet;
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The policy-local code of a final-newline finding.
pub const FINAL_NEWLINE_CODE: &str = "noncanonical-final-newline";

/// The message of a final-newline finding,
///  as the installed wrapper words it.
pub const FINAL_NEWLINE_MESSAGE: &str = "Non-empty text file must end with exactly one LF byte.";

/// What:
///  Path prefixes whose files keep their exact bytes.
///  `&[&[u8]]` is a borrowed list
///       of byte strings baked into the program.
/// Why:
///   A fuzz seed directory or a fixture source is an input whose trailing bytes are the test.
///  A directory that declares `-text` in its own `.gitattributes` needs no entry:
///       the policy reads the attribute.
///  Only directories that declare nothing stay listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PRESERVED_PREFIXES = ['package/forbidden-strings/core.fuzz/seed/', ...];
/// ```
const PRESERVED_PREFIXES: &[&[u8]] = &[
    b"package/forbidden-strings/core.fuzz/seed/",
    b"package/test-fixture/toml-edit/src/",
];

/// What:
///  Whether one byte is `/`,
///  the separator of Git pathnames.
/// Why:
///   A named predicate for `split`,
///  because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSlash = (byte: number) => byte === 0x2f;
/// ```
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Whether the policy preserves the bytes of the file at `path`.
/// Why:
///   The prefixes above,
///  any file below a `dist/final/node/` directory,
///  and any
///       file below a `bundle/node/` directory hold generated or fixture bytes.
///  A path
///       that merely ends in such a directory name holds no file below it.
///  The `text` attribute is a separate exemption, read from Git for the files that would change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isFinalNewlineExcluded(path: string): boolean;
/// ```
pub fn is_final_newline_excluded(path: &[u8]) -> bool {
    for prefix in PRESERVED_PREFIXES {
        if path.starts_with(prefix) {
            return true;
        }
    }
    let segments: Vec<&[u8]> = path.split(is_slash).collect();
    // `.windows(4)` visits each run of four neighbouring segments: three directory names
    // and at least one more segment below them.
    for run in segments.windows(4) {
        if run[0] == b"dist" && run[1] == b"final" && run[2] == b"node" {
            return true;
        }
    }
    for run in segments.windows(3) {
        if run[0] == b"bundle" && run[1] == b"node" {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether one byte is not LF.
/// Why:
///   A named predicate for `rposition`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isContentByte = (byte: number) => byte !== 0x0a;
/// ```
fn is_content_byte(byte: &u8) -> bool {
    return *byte != b'\n';
}

/// What:
///  The canonical bytes of a file,
///  or nothing when they are already canonical or
///       not text.
///  `Option<Vec<u8>>` is "replacement bytes or nothing".
/// Why:
///   Empty files,
///  files with a NUL byte and files that are not UTF-8 are left alone;
///       every other file ends with exactly one LF:
///  missing ones are added and extra
///       ones removed,
///  interior bytes untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalizeFinalNewline(bytes: Uint8Array): Uint8Array | undefined;
/// ```
pub fn normalized_final_newline(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.is_empty() || bytes.contains(&0) || std::str::from_utf8(bytes).is_err() {
        return None;
    }
    // What: `.iter().rposition(..)` is the index of the last byte that is not LF, if any.
    // Why:  The content ends just after it; a file of LF bytes alone has no content.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const contentEnd = bytes.findLastIndex(isContentByte) + 1;
    // ```
    let content_end: usize = match bytes.iter().rposition(is_content_byte) {
        Some(last) => last + 1,
        None => 0,
    };
    if content_end + 1 == bytes.len() {
        // Exactly one LF follows the content: the bytes are canonical.
        return None;
    }
    // `.to_vec()` copies the content; `.push` appends the one LF.
    let mut normalized: Vec<u8> = bytes[..content_end].to_vec();
    normalized.push(b'\n');
    return Some(normalized);
}

/// What:
///  Whether the policy reads a candidate at all.
/// Why:
///   Deletions have no content;
///  symbolic links and submodules are not files of text;
///       preserved paths keep their bytes.
///  The `text` attribute is read later, only for files whose bytes are not canonical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checked = candidate.change !== 'deleted' && (candidate.mode === 'regular' || candidate.mode === 'executable') && !isFinalNewlineExcluded(candidate.path);
/// ```
fn is_checked(candidate: &Candidate) -> bool {
    if candidate.change == CandidateChange::Deleted {
        return false;
    }
    if candidate.mode != CandidateMode::Regular && candidate.mode != CandidateMode::Executable {
        return false;
    }
    return !is_final_newline_excluded(candidate.path.as_slice());
}

/// What:
///  The finding for one candidate.
///  `EventPath::from_git_bytes` renders the pathname
///       as text,
///  replacing bytes that are not UTF-8,
///  and keeps such bytes exactly.
/// Why:
///   The finding names the file the person must change,
///  and says whether this
///       lifecycle proposed the correction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finding = { code: 'noncanonical-final-newline', message, path: candidate.path, patch };
/// ```
fn finding(candidate: &Candidate, fix_available: bool) -> PolicyFinding {
    return PolicyFinding {
        code: FINAL_NEWLINE_CODE,
        message: String::from(FINAL_NEWLINE_MESSAGE),
        path: Some(EventPath::from_git_bytes(candidate.path.as_slice())),
        location: None,
        fix_available,
    };
}

/// What:
///  Check every candidate of the lifecycle.
///  `<F: RepositoryFacts>` accepts any facts
///       provider;
///  `&mut ContentState` lends the shared candidates for reading and takes
///       the corrections.
/// Why:
///   One finding per file whose bytes are not canonical,
///  in candidate order;
///  no
///       candidates means no findings,
///  and unreadable content means the policy failed.
///       Only a direct fix applies corrections (the installed wrapper's
///       `canApplyPatches`),
///  so only there does each finding carry one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkFinalNewline(content, lifecycle, facts, trigger): Promise<PolicyOutcome>;
/// ```
pub fn check_final_newline<F: RepositoryFacts>(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut F,
    trigger: Trigger,
    can_apply_patches: bool,
) -> PolicyOutcome {
    let version: Rc<CandidateVersion> = match content.version(lifecycle, facts) {
        Ok(Some(found)) => found,
        // `Vec::new()` is the empty list: a lifecycle without candidates has nothing to report.
        Ok(None) => return PolicyOutcome::Findings(Vec::new()),
        Err(outcome) => return outcome,
    };
    // Every checked file whose bytes are not canonical, with the bytes a fix would write.
    let mut pending: Vec<(&Candidate, Rc<[u8]>, Vec<u8>)> = Vec::new();
    for candidate in version.candidates() {
        if !is_checked(candidate) {
            continue;
        }
        let bytes: Rc<[u8]> = match content.bytes(candidate) {
            Ok(read) => read,
            Err(outcome) => return outcome,
        };
        let Some(normalized) = normalized_final_newline(&bytes) else {
            continue;
        };
        pending.push((candidate, bytes, normalized));
    }
    // What: One lookup for every file that would change.
    // Why:  A file whose `text` attribute is unset keeps its exact bytes; a commit that needs no
    //       change starts no attribute query.
    let paths: Vec<&[u8]> = pending
        .iter()
        .map(|(candidate, _, _)| { return candidate.path.as_slice(); })
        .collect();
    let unset: HashSet<Vec<u8>> = match content.text_unset(paths.as_slice()) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    let mut findings: Vec<PolicyFinding> = Vec::new();
    for (candidate, bytes, normalized) in pending {
        if unset.contains(&candidate.path) {
            continue;
        }
        let corrects: bool = trigger == Trigger::DirectFix || can_apply_patches;
        if corrects {
            content.propose(Correction {
                path: candidate.path.clone(),
                mode: candidate.mode,
                before: bytes,
                // `Rc::from(vec)` moves the bytes into a shared handle.
                after: Rc::from(normalized),
            });
        }
        findings.push(finding(candidate, corrects));
    }
    return PolicyOutcome::Findings(findings);
}

/// Normalization,
///  exclusion and check controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_final_newline_tests.rs"]
mod tests;

/// Controls for the `text` attribute exemption stay out of the release executable.
#[cfg(test)]
#[path = "policy_final_newline_attribute_tests.rs"]
mod attribute_tests;

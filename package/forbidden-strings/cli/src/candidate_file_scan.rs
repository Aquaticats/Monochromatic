//! Scans one candidate file:
//! the part of a scan that runs once per file,
//! in parallel over every file of the run.
//!
//! The per-file function takes the run's shared,
//! read-only facts as one value
//! so the parallel call names a function and carries its state as data.

/// Imports the set type that holds the skipped rules files.
use std::collections::HashSet;
/// Imports the owned path type of the working directory and the repository root.
use std::path::PathBuf;

/// Imports the compiled rules every file is scanned with.
use crate::frx_load::LoadedRules;
/// Imports the module that scans one file's lines with the rules.
use crate::frx_scan;
/// Imports the module that scans a file's logical name with the rules.
use crate::path_scan;
/// Imports the two skip decisions and the binary-checked file read that the scan of one file starts with.
use crate::{is_config_file_at_cwd, is_walker_skipped, read_with_binary_check};

/// The facts of one run that every file's scan reads and none changes.
#[derive(Clone, Copy)]
pub(crate) struct CandidateFileScan<'run> {
    /// Whether the files came from the working-tree walk (`--all`).
    pub(crate) all: bool,
    /// The canonical working directory,
    /// when it can be resolved.
    pub(crate) cwd_canonical: &'run Option<PathBuf>,
    /// The nearest ancestor holding `.git`,
    /// which names files relative to the repository.
    pub(crate) repository_root: &'run Option<PathBuf>,
    /// The scanner's own generated rule sources,
    /// never scanned.
    pub(crate) self_match_set: &'run HashSet<PathBuf>,
    /// The rules files of this run,
    /// skipped when the walk finds them.
    pub(crate) skip_set: &'run HashSet<PathBuf>,
    /// The `--name-path` overrides,
    /// one per positional file or none.
    pub(crate) name_paths: &'run Vec<String>,
    /// The compiled rules every file is scanned with.
    pub(crate) loaded: &'run LoadedRules,
}

/// Scans one candidate file and returns its findings.
///
/// `map_with` hands every worker a copy of the shared facts,
/// and `(index, p)` is the file's position and path.
pub(crate) fn scan_candidate_file(context: &mut CandidateFileScan<'_>, (index, p): (usize, &String)) -> Vec<String> {
    let CandidateFileScan { all, cwd_canonical, repository_root, self_match_set, skip_set, name_paths, loaded } = *context;
    // Always skip the scanner's own ruleset files at cwd
    // (forbidden-strings.*.txt), regardless of --all vs explicit
    // args: they hold literal rule bodies that self-match.
    if is_config_file_at_cwd(p, cwd_canonical.as_deref()) {
        return Vec::new();
    }
    // Likewise skip the three generated rule-source files, path-
    // anchored by canonical path, in both modes. A same-named file
    // elsewhere has a different canonical path and is still scanned.
    if is_walker_skipped(p, self_match_set) {
        return Vec::new();
    }
    // What:     `if all && is_walker_skipped(p, skip_set) { return Vec::new(); }`.
    //           Only runs the rules-file skip check on walker
    //           output (--all mode). For explicit positional args
    //           (`forbidden-strings <path>...` without --all),
    //           the file is scanned -- the user asked -- except
    //           the scanner's own generated rule sources, skipped
    //           by the `self_match_set` check above.
    // Why:      Closes BUG 6: the previous basename-based skip
    //           hid real findings on
    //           `sub/forbidden-strings.local.txt` and friends
    //           passed as explicit args. The new check applies
    //           only when the walker discovered the file
    //           automatically.
    //
    //           Inside the conditional, `is_walker_skipped`
    //           canonicalizes the path and compares against
    //           the pre-built skip set. Path-anchored matching
    //           also closes BUG 11 (Windows backslash basename)
    //           by routing through `std::fs::canonicalize`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (all && isWalkerSkipped(p, skipSet)) return [];
    // ```
    if all && is_walker_skipped(p, skip_set) {
        return Vec::new();
    }
    // What:     `let content = fs::read(p).unwrap_or_default();`.
    //           `fs::read` returns `Result<Vec<u8>, io::Error>`
    //           (the file's raw bytes or an I/O error).
    //           `.unwrap_or_default()` extracts the `Ok` value or
    //           substitutes `Vec::<u8>::default()` (the empty
    //           vec) and SILENTLY DROPS the error. The implicit
    //           inferred type is `Vec<u8>`. Sibling pattern:
    //           `fs::read_to_string` returns `Result<String, _>`
    //           but requires UTF-8 -- we want raw bytes here
    //           because rules scan binary files too.
    // Why:      A file we can't read (permissions, vanished,
    //           etc.) becomes "empty content" and the scan
    //           pass produces zero hits for it. Crashing the
    //           whole walk on one unreadable file is worse.
    // Gotcha:   `.unwrap_or_default()` SILENTLY discards the
    //           `io::Error`. We accept that here because the
    //           per-file scan is best-effort.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let content: Uint8Array;
    // try { content = await readFile(p); }
    // catch (e) { return [`${p}: read error: ${e.message}`]; }
    // return scanContent(p, content, ruleset);
    // ```
    // What:     `match fs::read(p) { Ok(c) => ..., Err(e) => ... }`.
    //           Read error path now emits a synthetic "hit"
    //           string formatted as `{path}: read error: {err}`
    //           instead of silently substituting empty content.
    //           The synthetic entry makes the file appear in the
    //           output report AND keeps the exit code at 1 (hits
    //           non-empty -> ExitCode::from(1) downstream).
    // Why:      Closes BUG 4. Pre-fix, `fs::read(p).unwrap_or_default()`
    //           dropped every io::Error: permissions, missing
    //           file, broken symlink, /proc EACCES, etc. became
    //           "empty content", the scan emitted zero hits, and
    //           the run exited 0. A secret-scanning CI control
    //           must NOT silently pass on unreadable files; the
    //           operator needs to know they had no signal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { content = await readFile(p); }
    // catch (e) { return [`${p}: read error: ${e}`]; }
    // ```
    // Overrides name historical candidate paths while keeping temporary
    // content files isolated. Without an override, use the real file's
    // repository-relative name or all supplied external path segments.
    let logical: String = match name_paths.get(index) {
        Some(named) => named.clone(),
        None => path_scan::logical_path(p, repository_root.as_deref()),
    };
    let pathname = path_scan::scan_path(&logical, loaded);
    let mut findings = pathname.findings;
    let content_result = match read_with_binary_check(p) {
        Ok(c) => Some(c),
        Err(e) => {
            findings.push(format!("{}: read error: {}", pathname.display, e));
            None
        }
    };
    // What:     `frx_scan::scan_file(p, &content, &loaded)` splits the file
    //           into lines and runs each loaded set's `line_matches` under a
    //           fail-closed unwind boundary, returning `PATH:LINE rule=N`
    //           findings. `&content` and `&loaded` are read-only borrows; the
    //           returned `Vec<String>` is this function's tail expression.
    // Why:      Hand the just-read bytes to the engine line scan; the returned
    //           findings become this function's contribution to the parallel
    //           flat_map output.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return scanFile(p, content, loaded);
    // ```
    if let Some(content) = content_result {
        findings.extend(frx_scan::scan_file(&pathname.display, &content, loaded));
    }
    // An opaque operand index lets policy clients recover candidate
    // identity even when several paths redact to the same label.
    if !name_paths.is_empty() {
        let mut labelled: Vec<String> = Vec::with_capacity(findings.len());
        for hit in findings {
            labelled.push(format!("{} input={}", hit, index));
        }
        return labelled;
    }
    return findings
}

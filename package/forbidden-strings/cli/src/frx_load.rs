//! Loads the scan's rule sets onto the in-house forbidden-regex engine.
//!
//! Stage two of the engine swap (#384) routes rule loading through the stage-one frx
//! compiler instead of the resharp/`regex` pipeline.
//!  Two sources feed a scan:
//!
//! - the resolved runtime rules file (the `forbidden-strings.local.txt` precedence
//!   chain),
//!    loaded through the validated per-user runtime cache with text fallback;
//!    and
//! - the builtin baseline,
//!    embedded as a precompiled serialized `RegexSet` and
//!   rebuilt via `load_precompiled` (never recompiled),
//!    active only under
//!   `--builtin-rules`.
//!
//! A scan runs each set in order and attributes findings by rule id.
//!  The two sets
//! carry independent 0-based id spaces,
//!  so the builtin's ids are offset past the
//! runtime set's rule count:
//!  the runtime rules keep ids `0..user_len` and the
//! builtin takes `user_len..`,
//!  mirroring the old "user rules first,
//!  baseline
//! appended" ordering the `--builtin-rules` contract documents.

/// Imports the CLI-boundary error channel and its construction macro.
use anyhow::{anyhow, Result};

/// Imports the engine's compiled-ruleset type held by each loaded set.
use forbidden_regex::RegexSet;

/// Native rule-file paths must not be converted to lossy strings before filesystem reads.
use std::path::{Path, PathBuf};

/// Imports the list entry type and the snapshot that reads and joins its files.
use crate::{rules_list::RulesEntry, rules_snapshot::RulesSnapshot};

/// Imports runtime cache module,
///  hybrid matcher,
///  and precompiled builtin path.
use crate::{load_precompiled, runtime_cache, runtime_matcher::RuntimeRules};

/// Imports text compiler used only by in-memory fuzzing loader.
#[cfg(feature = "fuzzing")]
use crate::compile_rules;

/// Runtime hybrid or builtin engine matcher behind common batch interface.
pub(crate) enum ScanMatcher {
    /// Mutable runtime rules split into exact literals and restricted regexes.
    Runtime(
        /// Hybrid runtime matcher retaining original ids.
        RuntimeRules,
    ),
    /// Trusted embedded baseline or in-memory fuzz fixture.
    Regex(
        /// Existing engine matcher with local ids.
        RegexSet,
    ),
}

/// Matcher delegation preserving one scan interface.
impl ScanMatcher {
    /// Reports `(line index, local rule id)` pairs.
    pub(crate) fn line_matches(&self, buf: &[u8], starts: &[usize]) -> Vec<(usize, usize)> {
        match self {
            Self::Runtime(rules) => return rules.line_matches(buf, starts),
            Self::Regex(set) => return set.line_matches(buf, starts),
        }
    }
}

/// One compiled rule set plus the identity data applied to its findings.
///
/// A named rule renders as `rule=<name>`;
///  an unnamed rule falls back to `base` plus
/// its index,
///  giving each source a disjoint numeric range in the combined output.
pub(crate) struct ScanSet {
    /// Matcher whose local ids are attributed against `base` and `names`.
    pub(crate) matcher: ScanMatcher,
    /// Rule-id offset for unnamed rules:
    ///  the runtime set's rule count for the
    /// builtin,
    ///  else 0.
    pub(crate) base: usize,
    /// Per-rule section names parallel to the set's indices;
    ///  `None` falls back to
    /// the offset numeric id.
    pub(crate) names: Vec<Option<String>>,
}

/// The ordered rule sets a scan runs,
///  each contributing identified findings.
///
/// Built by [`load`];
///  consumed by the scan path,
///  which runs every set against each
/// file's lines.
///  Holding the sets in one value keeps the scan loop source-agnostic.
pub struct LoadedRules {
    /// Sets scanned in order:
    ///  the runtime set first (when present),
    ///  then the builtin.
    sets: Vec<ScanSet>,
    /// Redacted cache warnings emitted before findings.
    cache_warnings: Vec<runtime_cache::CacheWarning>,
    /// Optional rules files that were absent and skipped,
    ///  in list order.
    skipped: Vec<PathBuf>,
}

/// Reads a value out of a `LoadedRules` for the scan path.
impl LoadedRules {
    /// Returns the optional rules files that were absent and skipped,
    ///  so an embedding host without a log subscriber can report them.
    pub(crate) fn skipped_optional(&self) -> &[PathBuf] {
        return &self.skipped
    }

    /// Iterates each compiled set with its rule-id base offset and name table.
    ///
    /// The scan path runs `line_matches` on each set and renders every rule id
    /// through the set's names (falling back to `base` plus the id),
    ///  so runtime and
    /// builtin findings never collide.
    pub(crate) fn iter_sets(&self) -> impl Iterator<Item = &ScanSet> {
        return self.sets.iter()
    }

    /// Returns redacted runtime-cache warning records in load order.
    pub(crate) fn cache_warnings(&self) -> &[runtime_cache::CacheWarning] {
        return &self.cache_warnings
    }
}

/// Parses the builtin baseline's embedded name sidecar into a per-rule name table.
///
/// The build step writes one line per baseline rule,
///  in compiled order:
///  the rule's
/// section name,
///  or an empty line for an unnamed legacy rule.
///  An entry count that
/// disagrees with the compiled set is a build-invariant break,
///  surfaced fail-closed
/// rather than misattributing findings.
fn parse_builtin_names(text: &str, expected: usize) -> Result<Vec<Option<String>>> {
    let names: Vec<Option<String>> = text
        .lines()
        .map(|line| return (!line.is_empty()).then(|| return line.to_string()))
        .collect();
    if names.len() != expected {
        return Err(anyhow!(
            "builtin baseline: name sidecar holds {} entries for {} rules",
            names.len(),
            expected,
        ));
    }
    return Ok(names)
}

/// Loads the runtime rules file and,
///  under the flag,
///  the precompiled builtin baseline.
///
/// Reads the resolved `rules_path`,
///  loads or repairs its compiled runtime cache,
/// and (when `builtin_rules`) appends the embedded baseline rebuilt from `precompiled` with its name sidecar
/// `builtin_names`.
///  A missing implicit default file is tolerated only under
/// `--builtin-rules` (the baseline alone scans);
///  an explicitly named missing file,
///  or
/// any other read failure,
///  errors.
///  A runtime rule whose name collides with a builtin
/// name fails the load closed,
///  so a finding's `rule=<name>` is never ambiguous.
/// Every error is redacted:
///  an I/O error names only the path,
///  a compile error
/// carries only an opaque rule index plus the engine's static reason,
///  and a
/// collision carries only the section name,
///  never rule text.
#[cfg(any(test, feature = "fuzzing"))]
pub fn load(
    rules_path: impl AsRef<Path>,
    builtin_rules: bool,
    explicit: bool,
    precompiled: &[u8],
    builtin_names: &str,
) -> Result<LoadedRules> {
    let entries: [RulesEntry; 1] = [single_entry(rules_path.as_ref(), builtin_rules, explicit)];
    return load_listed(&entries, builtin_rules, !explicit, precompiled, builtin_names);
}

/// Builds the one-entry list the one-path contract means.
///
/// Only the implicit default file may be absent,
///  and only when the shipped baseline can scan alone;
///  a path named by a flag or the environment is required.
pub(crate) fn single_entry(path: &Path, builtin_rules: bool, explicit: bool) -> RulesEntry {
    return RulesEntry { path: path.to_path_buf(), optional: builtin_rules && !explicit };
}

/// Loads the files of a rules list and,
///  under the flag,
///  the precompiled builtin baseline.
///
/// Every entry is read in list order and the present files are joined into one rule set
/// (see [`crate::rules_list`] for the grammar and `RulesSnapshot` for the join).
///  A required entry that cannot be read errors naming its path;
///  an optional entry whose file is absent is skipped and logged at `info` level,
///  and [`LoadedRules::skipped_optional`] lists it.
///  When no file is present the baseline scans alone under `--builtin-rules`;
///  without it the load fails closed, because scanning with no rules would pass every file.
///  `quiet_absence` logs skipped optional entries at `debug` instead of `info`;
///  only the implicit default file passes it,
///  so a repository without a local rules file does not print a line on every scan.
pub(crate) fn load_listed(
    entries: &[RulesEntry],
    builtin_rules: bool,
    quiet_absence: bool,
    precompiled: &[u8],
    builtin_names: &str,
) -> Result<LoadedRules> {
    let mut sets: Vec<ScanSet> = Vec::new();
    let mut cache_warnings: Vec<runtime_cache::CacheWarning> = Vec::new();

    // Read the listed files. A required entry that cannot be read, and any unreadable present file,
    // surfaces here with its path.
    let snapshot = RulesSnapshot::read(entries)?;
    let skipped: Vec<PathBuf> = snapshot.skipped();
    for path in &skipped {
        if quiet_absence {
            tracing::debug!(path = %path.display(), "optional rules file is absent; skipped");
        } else {
            tracing::info!(path = %path.display(), "optional rules file is absent; skipped");
        }
    }

    // With no present file the baseline may scan alone; otherwise there is nothing to scan with.
    let user_rules = if snapshot.is_empty() {
        if !builtin_rules {
            snapshot.require_present()?;
        }
        None
    } else {
        let cache_load = runtime_cache::load_or_compile(&snapshot)?;
        cache_warnings = cache_load.warnings;
        Some(cache_load.compiled)
    };

    // The runtime set takes ids 0..user_len; the builtin baseline is offset past it.
    let mut next_base = 0;
    if let Some(compiled) = user_rules {
        next_base = compiled.len();
        sets.push(ScanSet {
            names: compiled.names().to_vec(),
            matcher: ScanMatcher::Runtime(compiled),
            base: 0,
        });
    }

    if builtin_rules {
        let set = load_precompiled(precompiled)
            .map_err(|reason| return anyhow!("builtin baseline: {}", reason))?;
        let names = parse_builtin_names(builtin_names, set.len())?;
        // Reject a runtime name that shadows a builtin name: both would render as
        // the same `rule=<name>` token, making findings ambiguous. Section names
        // are non-secret by design (they surface in findings), so naming the
        // collision here stays inside the redaction contract.
        let builtin_set: std::collections::HashSet<&str> =
            names.iter().flatten().map(|name| return name.as_str()).collect();
        for user_name in sets.iter().flat_map(|scan_set| return scan_set.names.iter().flatten()) {
            if builtin_set.contains(user_name.as_str()) {
                return Err(anyhow!(
                    "rules {}: rule name '{}' collides with a builtin baseline rule name",
                    snapshot.label(),
                    user_name,
                ));
            }
        }
        sets.push(ScanSet {
            matcher: ScanMatcher::Regex(set),
            base: next_base,
            names,
        });
    }

    return Ok(LoadedRules { sets, cache_warnings, skipped })
}

/// Builds a single-set `LoadedRules` from in-memory rule text,
///  for the fuzz targets.
///
/// Compiles `text` through the same strict loader the runtime path uses
/// (`compile_rules`:
///  escape literals,
///  drop `m`/`x`,
///  fail closed on any other flag,
/// names carried for tail-format sources),
///  then wraps the one resulting set at
/// rule-id base 0.
///  It skips the file read and the precompiled builtin baseline so a
/// target can drive the loader and scan path from an in-memory source.
///  The redacted
/// `LoadError` is returned verbatim so a target can assert the strict-loader
/// contract.
///  Gated on `fuzzing`;
///  the production loader is [`load`].
#[cfg(feature = "fuzzing")]
pub fn load_from_text(text: &str) -> std::result::Result<LoadedRules, crate::LoadError> {
    let compiled = compile_rules(text)?;
    return Ok(LoadedRules {
        sets: vec![ScanSet {
            matcher: ScanMatcher::Regex(compiled.set),
            base: 0,
            names: compiled.names,
        }],
        cache_warnings: Vec::new(),
        skipped: Vec::new(),
    })
}

/// Compile cache-free hybrid rules through the production matcher for fuzz/embedding controls.
#[cfg(any(test, feature = "fuzzing"))]
pub(crate) fn hybrid_from_text(text: &str) -> Result<LoadedRules> {
    // RuntimeRules owns both literal and regex subsets; errors retain their fixed redacted reasons.
    let compiled: RuntimeRules = RuntimeRules::compile(text).map_err(|error| return anyhow!(error))?;
    // Copy non-secret identities before moving the compiled matcher into the owning set.
    let names: Vec<Option<String>> = compiled.names().to_vec();
    return Ok(LoadedRules {
        sets: vec![ScanSet { matcher: ScanMatcher::Runtime(compiled), base: 0, names }],
        cache_warnings: Vec::new(),
        skipped: Vec::new(),
    });
}

/// Constructs a cache-free hybrid matcher for pathname unit tests.
///
/// These tests must never read or publish artifacts in a user's runtime cache.
#[cfg(test)]
pub(crate) fn test_rules(text: &str) -> LoadedRules {
    // Extract only test-fixture success; production callers receive the redacted Result instead.
    return hybrid_from_text(text).expect("compile pathname test rules");
}

/// Registers the loader precedence and offset tests (sidecar,
///  lint-exempt).
#[cfg(test)]
#[path = "frx_load_tests.rs"]
mod tests;

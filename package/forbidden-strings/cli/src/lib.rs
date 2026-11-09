//! lib support for the forbidden-strings scanner.

/// Registers the `cli` child module.
// What:     `mod cli;` declares a child module whose source lives in `cli.rs`.
//           `mod` is Rust's module system: it does NOT import names; it tells
//           the compiler "this file/module exists, compile it". Names referenced
//           via `crate::cli::xxx` afterward.
// Why:      Clap-backed argument parsing now lives beside the run loop but outside
//           it, replacing the previous handwritten argv scanner.
// Gotcha:   `mod foo;` without a body is NOT an import; it's a registration.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent. Closest: TypeScript automatically picks up files
// // in `include` paths; Rust requires explicit `mod` declarations.
// ```
mod cli;

/// Registers the `rules` child module.
// What:     `mod walk;` declares child modules whose source lives in sibling
//           `.rs` files. `mod` registers each file with the compiler; it does
//           not import names into local scope.
// Why:      We split the binary into focused files: CLI parsing, rules, scanning,
//           hit formatting, and the working-tree walker.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent. Closest: TypeScript automatically picks up files
// // in `include` paths; Rust requires explicit `mod` declarations.
// ```
mod rule;
/// Registers the `walk` child module.
mod walk;
/// Registers the working-tree discovery the walk's tracked-file union reads through.
mod git_working_tree;
/// Registers the per-file scan that runs in parallel over every candidate file.
mod candidate_file_scan;
/// Registers the rules compilation and file listing that run side by side at startup.
mod startup_work;
/// Registers the `frx_load` child module:
///  the forbidden-regex rule loader.
mod frx_load;
/// Registers the `frx_scan` child module:
///  the forbidden-regex line scan.
mod frx_scan;
/// Shared structured findings keep embedded policies independent of terminal formatting.
mod scan_finding;
/// Public findings contain only source locations and non-secret rule identities.
pub use scan_finding::ScanFinding;
/// Thread-isolated named-callback boundary for embedded rule loading.
mod load_request;
/// The comma-separated rules-list grammar shared by the flag,
/// the environment variable,
/// and embedding hosts.
mod rules_list;
/// Embedding hosts build entries directly or parse the same list value the command line accepts.
pub use rules_list::{parse_rules_list, RulesEntry, RulesListError};
/// Reads the files of a rules list once and joins them into the text the scanner compiles.
mod rules_snapshot;
/// In-process candidate snapshots reuse the standalone scan core and redaction rules.
mod scanner;
/// Embedded consumers identify candidates independently of redacted display labels.
pub use scanner::{CandidateScan, Scanner};
/// Fixed-token cache warnings remain structured for embedded event reporting.
pub use runtime_cache::CacheWarning;
/// Embedding hosts say whether a load that misses may write the shared compiled-rules cache,
///  and ask beforehand whether the cache already holds the text.
pub use runtime_cache::{compiled_rules_artifact_exists, CacheAccess};
/// Lowercase hexadecimal spelling shared by digest directory names and redacted labels.
mod bytes_as_lowercase_hex;
/// Native-byte path normalization and protocol-safe display encoding.
mod path_name_bytes;
/// Registers component-level pathname matching and masked display labels.
mod path_scan;
/// Registers runtime cache envelope,
///  path,
///  warning,
///  and publication implementation.
mod runtime_cache;
/// Registers hybrid exact-literal and restricted-regex runtime matcher.
mod runtime_matcher;

/// Registers the unit-test fixture root,
/// placed in the build directory instead of `/tmp`.
#[cfg(test)]
mod test_scratch;

/// Registers the `fuzz_api` child module.
// What:     `#[cfg(feature = "fuzzing")] pub mod fuzz_api;` registers
//           the curated re-export module ONLY when the `fuzzing`
//           Cargo feature is on. The bin target leaves the feature
//           off and never sees this module; fuzz targets (in
//           `fuzz/Cargo.toml`) enable the feature and use
//           `forbidden_strings::fuzz_api::*` to import internals.
// Why:      Keep the production public surface unchanged while
//           letting fuzz targets reach the internal helpers they
//           need.
//
// In TS you'd write (pseudocode):
// ```ts
// // No clean equivalent; conditional re-export at build time.
// ```
#[cfg(feature = "fuzzing")]
pub mod fuzz_api;

/// What:
///      `pub const BUILTIN_RULES: &str = include_str!("../data/builtin-rules.txt");`
///           declares an exported constant whose value is the entire
///           betterleaks-ported baseline ruleset,
///  pasted into the binary at
///           COMPILE time by the `include_str!` macro.
///  `&str` is a read-only
///           view of UTF-8 bytes baked into the binary;
///  the sibling `String`
///           would heap-allocate the same bytes again at runtime for no
///           benefit.
/// Why:
///       Ships the baseline inside the published binary so the opt-in
///           `--builtin-rules` flag works with no rules file on disk,
///  and lets
///           consumers (the forbidden-regex bench crate) import the same rules
///           instead of `include_str!`-ing a fragile relative path into this
///           repository.
/// Gotcha:
///    Editing `data/builtin-rules.txt` changes nothing until the crate
///           is rebuilt,
///  and the file itself is GENERATED by
///           `bin/port-betterleaks.ts`;
///  never edit it by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import BUILTIN_RULES from '../data/builtin-rules.txt' with { type: 'text' };
/// export { BUILTIN_RULES };
/// ```
pub const BUILTIN_RULES: &str = include_str!("../data/builtin-rules.txt");

/// The builtin baseline precompiled into a serialized `RegexSet`,
///  embedded at build
/// time.
///
/// What:
///      `include_bytes!(concat!(env!("OUT_DIR"), "/builtin-rules-precompiled.bin"))`
///           bakes the byte blob `build.rs` produced from `data/builtin-rules.txt`
///           into the binary.
///  `&[u8]` is a read-only view of those bytes;
///  the runtime
///           loader hands them to `load_precompiled`,
///  which the engine's validating
///           `from_bytes` decodes without recompiling.
/// Why:
///       Compiling the full baseline at startup is not viable (the migration
///           measured tens of seconds),
///  so the baseline is compiled once at build time
///           and only decoded at runtime.
///  Mutable runtime rules use their separate
///           content-addressed per-user cache with text fallback.
/// Gotcha:
///    The blob is regenerated whenever `data/builtin-rules.txt` or the
///           shared frx parser sources change (`build.rs` `rerun-if-changed`);
///  editing
///           the baseline file changes nothing until the crate is rebuilt.
pub const BUILTIN_PRECOMPILED: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/builtin-rules-precompiled.bin"));

/// The builtin baseline's rule-name sidecar,
///  embedded at build time.
///
/// One line per baseline rule,
///  in compiled order:
///  the rule's tail-format section
/// name (its betterleaks id),
///  or an empty line for an unnamed legacy rule.
///  The
/// loader pairs it with `BUILTIN_PRECOMPILED` so a baseline finding renders as
/// `rule=<name>` instead of a drifting numeric index;
///  a count mismatch against the
/// decoded set fails the load closed.
pub const BUILTIN_NAMES: &str =
    include_str!(concat!(env!("OUT_DIR"), "/builtin-rules-names.txt"));

/// Re-exports the forbidden-regex rule compiler's public surface.
// What:     `pub use rule::frx::{...}` lifts the engine's rule-compiler entry
//           points to the crate root so they are reachable crate-public API.
//           `frx_load::load` calls the runtime cache and `load_precompiled`;
//           `frx_scan::scan_file` runs the resulting sets against each file.
// Why:      These are the live load-path construction functions and the redacted
//           error they return. Exposing them at the crate root lets the loader,
//           the build script's sibling parser, and the fuzz surface share one API.
//
// In TS you'd write (pseudocode):
// ```ts
// export { compileFromText, loadPrecompiled, LoadError } from "./rule/frx";
// ```
pub use rule::frx::{compile_from_text, compile_rules, load_precompiled, CompiledRules, LoadError};

/// Imports dependencies used by this module.
// What:     `use anyhow::Result;` imports `anyhow`'s one-parameter application
//           result alias. Sibling typed results name their exact error type.
// Why:      The CLI boundary keeps one catastrophic-error channel while ordinary
//           scan findings continue to return numeric exit codes.
//
// In TS you'd write (pseudocode):
// ```ts
// type Result<T> = T; // failures throw Error objects
// ```
use anyhow::Result;

/// Imports dependencies used by this module.
// What:     `use clap::Parser;` brings the parser trait into this module. Rust
//           only lets trait methods such as `Cli::try_parse_from(...)` be called
//           when the trait is in scope. `::` is Rust's namespace separator.
// Why:      `run_cli_from_env` should let clap validate argv and render help,
//           version, and parse errors.
//
// In TS you'd write (pseudocode):
// ```ts
// import { parseArgs } from "some-cli-parser";
// ```
use clap::Parser;

/// Imports dependencies used by this module.
// What:     `use std::env;` imports the std `env` module so we can reference
//           `env::args` / `env::var`.
// Why:      Reading argv and environment variables.
//
// In TS you'd write (pseudocode):
// ```ts
// import { argv, env } from "node:process";
// ```
use std::env;

// What:     `std::fs::canonicalize` is referenced via the full path at
//           three sites (rules-path skip set, walker skip lookup); no
//           bare `use std::fs;` because the per-file `fs::read` slurp
//           moved into `read_with_binary_check` which uses
//           `std::fs::File` directly.
// Why:      Background on file-reading performance choices: `fs::read`
//           is empirically faster than `mmap`-based access on this
//           workload (many small files; per-file VMA setup cost
//           dominates the saved alloc) -- the E2 mmap experiment
//           regressed wall time by 35% on Mono and 43% on the Linux
//           kernel. See PERF.md "Mmap experiment (rejected)".
//           Thread-local scratch buffers were also tried 2026-05-03;
//           rayon's nested-parallelism work-stealing (scan_content
//           uses inner par_iter via prefix-matched and combined-
//           shard fan-out) re-entered the outer flat_map_iter on
//           the SAME thread while the buffer was borrowed,
//           triggering a `RefCell already borrowed` panic. The
//           per-file alloc cost is dwarfed by the unicode-mode
//           speedup; not worth the re-entrancy hazard.

/// Imports dependencies used by this module.
// What:     `use std::io::Write;` imports the `Write` TRAIT (interface-
//           like). Methods declared by a trait are only callable when
//           the trait is in scope, even when used via macros like
//           `writeln!`.
// Why:      We use `writeln!(handle, ...)` to emit hits.
//
// In TS you'd write (pseudocode):
// ```ts
// // Unnecessary in TS.
// ```
use std::io::Write;

/// Imports dependencies used by this module.
// What:     `use rayon::prelude::*;` brings rayon's parallel-iterator
//           extension methods into scope (`par_iter`, `flat_map_iter`,
//           etc.).
// Why:      The two-phase main loop uses `par_iter` for both the
//           parallel-read phase and the parallel-scan phase.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent.
// ```
use rayon::prelude::*;

/// Imports dependencies used by this module.
// What:     `use crate::cli::Cli;` imports the clap-backed parsed option struct
//           from the sibling module. `crate::` is the absolute root of this
//           crate.
// Why:      `run_cli_from_env` needs the generated parser and the typed fields it
//           returns.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Cli } from "./cli";
// ```
use crate::cli::Cli;

/// Imports the per-file scan and the shared facts it reads,
/// so the parallel scan below names a function and carries its state as data.
use crate::candidate_file_scan::{scan_candidate_file, CandidateFileScan};

/// Imports the side-by-side startup work,
/// so the rules compile while the working tree is listed.
use crate::startup_work::{run_startup_work, StartupRequest};
/// Imports the listing result that carries the tracked-file union's diagnostic.
use crate::walk::WorkingTreeListing;

/// Implements `build_skip_set`.
// What:     `fn build_skip_set(rules_path: &str) -> HashSet<PathBuf>`
//           returns the set of CANONICAL absolute paths to skip when
//           walking the tree in `--all` mode. Pre-fix this logic was a
//           basename check (`is_skipped_file`) that matched anywhere in
//           the tree, so an unrelated `sub/forbidden-strings.local.txt`
//           was silently dropped along with the actual rule file. Path-
//           anchored matching pins each skip to its specific filesystem
//           location.
// Why:      Closes BUG 6 (basename skip applies to arbitrary explicit
//           args) and BUG 11 (Windows path basename via rsplit('/')) in
//           one shape change. Path-anchoring removes both failure modes:
//           the basename collision cannot trigger because we compare
//           full canonical paths, and the Windows backslash separator
//           is handled inside `std::fs::canonicalize` / `PathBuf::eq`.
//
//           Skip set composition:
//             - The actual rules file (whatever the user passed via
//               `--rules` or `FORBIDDEN_STRINGS_RULES`; falls back to
//               the default `forbidden-strings.local.txt` in cwd).
//
//           The three generated rule-source files live in
//           `build_self_match_set`, which applies to positional
//           arguments as well.
//
//           The caller separately decides WHEN to apply the skip:
//           explicit positional args are NEVER skipped by this set (the
//           user asked for them); only walker output in --all mode is
//           filtered.
//
// In TS you'd write (pseudocode):
// ```ts
// function buildSkipSet(rulesPath: string): Set<string> {
//   const set = new Set<string>();
//   try { set.add(fs.realpathSync(rulesPath)); } catch {}
//   for (const k of CANONICAL_SELF_MATCH_PATHS) {
//     try { set.add(fs.realpathSync(k)); } catch {}
//   }
//   return set;
// }
// ```
fn build_skip_set(rules_entries: &[RulesEntry]) -> std::collections::HashSet<std::path::PathBuf> {
    // What:     `let mut set: HashSet<PathBuf> = HashSet::new();` -- the
    //           usual mutable-empty-collection pattern.
    // Why:      Accumulate canonical-form paths we want to skip.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const set = new Set<string>();
    // ```
    let mut set: std::collections::HashSet<std::path::PathBuf> =
        std::collections::HashSet::new();

    // What:     `if let Ok(p) = std::fs::canonicalize(rules_path) { set.insert(p); }`.
    //           `canonicalize` resolves symlinks AND makes the path
    //           absolute; identical files reached via different
    //           relative paths compare equal at the canonical level.
    //           A missing rules file would fail to canonicalize -- the
    //           loader will surface that error separately via
    //           `load_ruleset`, so we silently skip the insertion here.
    // Why:      Anchor the skip on the actual filesystem identity of
    //           the rules file rather than its basename.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { set.add(fs.realpathSync(rulesPath)); } catch {}
    // ```
    // Every listed file is a rules file, so every one that exists is skipped; an absent one cannot be walked.
    for entry in rules_entries {
        if let Ok(p) = std::fs::canonicalize(&entry.path) {
            set.insert(p);
        }
    }
    return set
}

/// The three generated rule sources,
///  named relative to the repository root.
/// Each is generated source containing literal copies of rule bodies,
///  so each matches its own rules.
const SELF_MATCH_SOURCES: [&str; 3] = [
    "package/forbidden-strings/cli/data/betterleaks-default-config.toml",
    "package/forbidden-strings/cli/data/builtin-rules.txt",
    "package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts",
];

/// Implements `build_self_match_set`.
// What:     `fn build_self_match_set(repository_root: Option<&Path>)
//           -> HashSet<PathBuf>` returns the CANONICAL absolute paths
//           of the three generated rule-source files, resolved against
//           the repository root. Each is generated source containing
//           literal copies of rule bodies, so each matches its own
//           rules. Pinned by their expected location so the matcher
//           does not fire on unrelated files of the same name
//           elsewhere in the tree.
// Why:      The set applies in BOTH modes, `--all` walker output and
//           explicit positional arguments, exactly as
//           `is_config_file_at_cwd` does for the ruleset files at cwd.
//           CI passes the changed files positionally, so a push that
//           renames or edits one of these files hands it to the scanner,
//           which reported its own rule bodies and failed every later
//           push (the validated-tree watermark only advances on a green
//           run). `build_skip_set` stays `--all`-only: the user's own
//           rules file, named positionally, is still scanned.
//           The anchor is the repository root, never the working
//           directory: the `--all` walk starts at the working directory,
//           so a run started inside `package/forbidden-strings/cli`
//           reached the same three files by a path below it while a
//           working-directory anchor resolved nothing, and the run
//           reported its own rule bodies (issue 1233). A standalone run
//           outside any repository keeps the working directory, the only
//           place the sources could sit then. A path that does not
//           resolve is dropped silently: the file is then absent from the
//           walked tree and from any positional argument too.
//
// In TS you'd write (pseudocode):
// ```ts
// function buildSelfMatchSet(repositoryRoot: string | undefined): Set<string> {
//   const set = new Set<string>();
//   const anchor = repositoryRoot ?? process.cwd();
//   for (const k of SELF_MATCH_SOURCES) {
//     try { set.add(fs.realpathSync(path.join(anchor, k))); } catch {}
//   }
//   return set;
// }
// ```
fn build_self_match_set(
    repository_root: Option<&std::path::Path>,
) -> std::collections::HashSet<std::path::PathBuf> {
    let mut set: std::collections::HashSet<std::path::PathBuf> =
        std::collections::HashSet::new();
    let anchor: std::path::PathBuf = match repository_root {
        Some(root) => root.to_path_buf(),
        None => match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => { return set },
        },
    };
    for source in SELF_MATCH_SOURCES {
        if let Ok(p) = std::fs::canonicalize(anchor.join(source)) {
            set.insert(p);
        }
    }
    return set
}

/// Implements `is_walker_skipped`.
// What:     `fn is_walker_skipped(path: &str, skip_set: &HashSet<PathBuf>) -> bool`
//           returns true when the path's canonical form matches a
//           skip-set entry. Applied with the rules-file set to walker
//           output in --all mode only, and with the generated rule-source
//           set (`build_self_match_set`) to walker output and explicit
//           positional args alike.
// Why:      Closes BUG 6: the previous `is_skipped_file` ran on every
//           queued path regardless of source, hiding real positive
//           findings on `sub/forbidden-strings.local.txt`-style explicit
//           args. The path-anchored form here is consulted only when
//           the caller knows the path came from the walker.
//
// In TS you'd write (pseudocode):
// ```ts
// function isWalkerSkipped(path: string, skipSet: Set<string>): boolean {
//   try {
//     const canonical = fs.realpathSync(path);
//     return skipSet.has(canonical);
//   } catch { return false; }
// }
// ```
pub(crate) fn is_walker_skipped(
    path: &str,
    skip_set: &std::collections::HashSet<std::path::PathBuf>,
) -> bool {
    // What:     Canonicalize per file and lookup in the skip set. A
    //           canonicalize failure (broken symlink, vanished file)
    //           returns false -- if we cannot resolve the path, we are
    //           definitely not skipping it. The downstream `fs::read`
    //           will surface any read error via the BUG 4 fix.
    // Why:      Per-file canonicalize is one stat syscall; with the
    //           ~2700-file walked corpus, that's a few ms total --
    //           well under the scan cost itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try {
    //   const canonical = fs.realpathSync(path);
    //   return skipSet.has(canonical);
    // } catch { return false; }
    // ```
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return skip_set.contains(&canonical);
    }
    return false
}

/// Whether a file name is one of the scanner's own rules files (`forbidden-strings.*.txt`).
fn is_rules_file_name(name: &str) -> bool {
    return name.starts_with("forbidden-strings.") && name.ends_with(".txt");
}

/// Implements `is_config_file_at_cwd`.
// What:     `fn is_config_file_at_cwd(path, cwd_canonical) -> bool` returns
//           true when `path` names a `forbidden-strings.*.txt` file sitting
//           DIRECTLY in the current working directory. The basename check is
//           cheap and runs first; only on a name match do we canonicalize to
//           confirm the file's parent IS the cwd, so a same-named file in a
//           subdirectory is not matched.
// Why:      These are the scanner's own ruleset files
//           (`forbidden-strings.local.txt`, `.local.example.txt`,
//           `.append.txt`, `.append.local.txt`). Scanning them re-derives the
//           rule bodies as self-matches, so they are always skipped, in BOTH
//           `--all` walker mode and explicit-positional-arg mode. Unlike the
//           `--all`-only `build_skip_set` guard, this one also fires on
//           explicit args, because CI passes changed files positionally
//           (`forbidden-strings --rules ... <changed>...`) and an edited
//           `forbidden-strings.append.txt` would otherwise self-match.
//
//           The cwd anchor keeps BUG 6 / BUG 11 closed: a file like
//           `sub/forbidden-strings.local.txt` (different parent) still scans,
//           because only a config file at the cwd root is skipped.
pub(crate) fn is_config_file_at_cwd(
    path: &str,
    cwd_canonical: Option<&std::path::Path>,
) -> bool {
    let name_matches = std::path::Path::new(path)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(is_rules_file_name);
    if !name_matches {
        return false;
    }
    let Some(cwd) = cwd_canonical else {
        return false;
    };
    let Ok(canonical) = std::fs::canonicalize(path) else {
        return false;
    };
    return canonical.parent() == Some(cwd)
}

/// Defines the `BIN_PROBE_SIZE` constant.
// What:     `BIN_PROBE_SIZE` is the byte length read up-front from every
//           file before deciding whether the file is binary. 8 KiB is
//           the same probe size the pre-BUG-5 `is_likely_binary`
//           heuristic used; it matches `git diff`'s "binary or text"
//           heuristic threshold.
// Why:      The probe length tunes a tradeoff: smaller probe lets a
//           binary file with a leading text header (PDF header,
//           machine-O header) sneak past as text; larger probe wastes
//           memory on small files. 8 KiB catches the common cases
//           (PNG, JPG, ELF, WASM, zip, ZSTD frames) and is the
//           established convention.
const BIN_PROBE_SIZE: usize = 8192;

/// Implements `read_with_binary_check`.
// What:     `read_with_binary_check(path)` reads a file under a binary
//           heuristic:
//             1. Always read the first `BIN_PROBE_SIZE` bytes.
//             2. If the file is smaller than that, return what we got.
//             3. If the probe contains a NUL byte and the file is
//                larger than the probe, return only the probe (the
//                rest is treated as binary tail and not scanned).
//             4. Otherwise (probe is NUL-free), read and return the
//                full file.
// Why:      Closes the BUG-5 regression without re-introducing the
//           soundness gap that BUG 5 fixed. BUG 5 removed a heuristic
//           that threw away the WHOLE file when the first 8 KiB
//           contained a NUL byte; that masked secrets sitting BEFORE
//           the NUL. This rule keeps that signal (the first 8 KiB is
//           always scanned), but caps the per-file work on large
//           binary blobs (firmware images, vmlinuz, font caches, lock
//           sidecars) at 8 KiB instead of full content. Acceptable
//           miss: a secret living AFTER a NUL byte in a file that is
//           ALSO larger than 8 KiB. Acceptable: those files are the
//           "binary blob with bytes that happen to spell a secret"
//           case, and the secret-leak risk is dominated by source
//           files and small lock files which still scan in full.
//
// In TS you'd write (pseudocode):
// ```ts
// function readWithBinaryCheck(path: string): Buffer {
//   const fd = fs.openSync(path, "r");
//   try {
//     const probe = Buffer.alloc(BIN_PROBE_SIZE);
//     const n = fs.readSync(fd, probe, 0, BIN_PROBE_SIZE, null);
//     if (n < BIN_PROBE_SIZE) return probe.subarray(0, n);
//     if (probe.indexOf(0) !== -1) return probe;
//     return Buffer.concat([probe, fs.readSync.readRestOf(fd)]);
//   } finally {
//     fs.closeSync(fd);
//   }
// }
// ```
pub(crate) fn read_with_binary_check(path: &str) -> Result<Vec<u8>, std::io::Error> {
/// Imports dependencies used by this module.
    use std::fs::File;
/// Imports dependencies used by this module.
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::with_capacity(BIN_PROBE_SIZE);
    (&mut file)
        .take(BIN_PROBE_SIZE as u64)
        .read_to_end(&mut buf)?;

    if buf.len() < BIN_PROBE_SIZE {
        return Ok(buf);
    }

    if memchr::memchr(0, &buf).is_some() {
        return Ok(buf);
    }

    file.read_to_end(&mut buf)?;
    return Ok(buf)
}

/// The rules list the `FORBIDDEN_STRINGS_RULES` variable carries,
/// when it is set to valid text.
fn rules_from_environment() -> Option<String> {
    return env::var("FORBIDDEN_STRINGS_RULES").ok();
}

/// Run the scanner from real process arguments and environment values.
// What:     `pub fn run_cli_from_env() -> Result<i32>` is the library
//           entry point. It asks clap to parse `env::args()`, applies the
//           `FORBIDDEN_STRINGS_RULES` fallback, loads the ruleset, runs the
//           parallel scan, prints hits to stderr, and returns the exit code the
//           OS should see. `Result<i32>` lets the binary thin wrapper
//           decide how to report a catastrophic failure (the `Err` arm) versus a
//           regular run (`Ok(0)` clean, `Ok(1)` violation, `Ok(2)` usage error
//           already printed). Sibling shape considered: returning `ExitCode`
//           directly, rejected because tests written against the lib want a plain
//           `i32` they can compare, and `ExitCode` has no `Eq`.
// Why:      Coordinate clap parsing, ruleset loading, parallel scan, and result
//           reporting from a unit-testable surface. The bin target's `main` stays
//           a tiny wrapper that turns the returned code into an `ExitCode` and
//           prints `Err` to stderr with a fixed prefix.
//
// In TS you'd write (pseudocode):
// ```ts
// async function runCliFromEnv(): Promise<number> {
//   const cli = parseArgs(process.argv);
//   // ...
//   return anyViolation ? 1 : 0;
// }
// process.exit(await runCliFromEnv());
// ```
pub fn run_cli_from_env() -> Result<i32> {
    // What:     `Cli::try_parse_from(env::args())` calls the clap-generated parser.
    //           `::` is Rust's namespace operator. `env::args()` includes the
    //           program name at index 0, which clap expects so it can render usage.
    //           `match` extracts the success payload (`Ok(parsed_cli)`) or handles
    //           clap's display/error wrapper (`Err(parse_error)`).
    // Why:      Replace the handwritten argv loop with clap while preserving this
    //           function's non-panicking `Result<i32>` boundary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let cli: Cli;
    // try { cli = parseArgs(process.argv); }
    // catch (error) { print(error); return error.exitCode; }
    // ```
    let cli = match Cli::try_parse_from(env::args()) {
        Ok(parsed_cli) => parsed_cli,
        Err(parse_error) => {
            // What:     `if let Err(print_error) = parse_error.print() { ... }`
            //           is a one-arm pattern match over clap's attempt to write
            //           help, version, or parse errors to the right stream.
            //           `print_error.into()` converts an IO error into the
            //           `anyhow::Error` this function's `Err` arm carries.
            // Why:      If clap cannot print, surface that catastrophic failure to
            //           `main` instead of silently returning a success code.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const printed = error.print();
            // if (!printed.ok) throw new Error(String(printed.error));
            // ```
            if let Err(print_error) = parse_error.print() {
                return Err(print_error.into());
            }

            // What:     `return Ok(parse_error.exit_code());` converts clap's
            //           display/error outcome into this function's normal exit-code
            //           channel. Help and version return 0; invalid syntax returns 2.
            // Why:      Keep `run_cli_from_env` testable without letting clap call
            //           `std::process::exit` inside the library.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return error.exitCode;
            // ```
            return Ok(parse_error.exit_code());
        }
    };

    // A subcommand is a disjoint operation, not a scan option. Dispatch before
    // reading scan-mode environment fallback or walking candidate files.
    if let Some(command) = &cli.command {
        let compile_entries: Vec<RulesEntry> = match parse_rules_list(command.rules_path()) {
            Ok(parsed) => parsed,
            Err(error) => {
                eprintln!("forbidden-strings: --rules: {}", error);
                return Ok(2);
            }
        };
        for skipped in runtime_cache::compile_rules_to_cache(&compile_entries)? {
            tracing::info!(path = %skipped.display(), "optional rules file is absent; skipped");
        }
        return Ok(0);
    }

    // What:     `cli.rules_path.or_else(|| env::var("...").ok()).unwrap_or_else(...)`
    //           applies the existing rules-path precedence. `Option::or_else`
    //           keeps clap's `Some(path)` when `--rules` was present; otherwise it
    //           runs the closure that reads `FORBIDDEN_STRINGS_RULES`. `.ok()`
    //           converts `Result<String, VarError>` into `Option<String>`.
    //           `.unwrap_or_else(...)` supplies the owned default `String` only
    //           when both sources are absent. Sibling type `&str` would borrow from
    //           either argv or env storage; owned `String` is simpler and matches
    //           the loader API.
    // Why:      Preserve documented precedence: `--rules`, then env var, then
    //           `./forbidden-strings.local.txt`.
    //
    // What:     `let explicit_rules_source = cli.rules_path.is_some() || ...;`
    //           records, BEFORE the `or_else` chain below consumes
    //           `cli.rules_path`, whether the rules path was named explicitly
    //           (`--rules` or the env var) rather than falling back to the cwd
    //           default. `.is_some()` asks an `Option` "do you hold a value?";
    //           `env::var(...).is_ok()` asks the same of the env lookup's
    //           `Result`.
    // Why:      `--builtin-rules` tolerates a MISSING implicit default file
    //           (baseline-only scan) but must still error on an explicitly
    //           named missing file; that distinction is decided here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const explicitRulesSource = cli.rulesPath !== undefined
    //   || process.env.FORBIDDEN_STRINGS_RULES !== undefined;
    // ```
    // The list grammar (`crate::rules_list`): `--rules` and the variable carry the same comma-separated list,
    // an entry ending in `?` may be absent, and `--rules` replaces the variable instead of adding to it.
    let rules_value: Option<String> = cli.rules_path.or_else(rules_from_environment);
    let explicit_rules_source: bool = rules_value.is_some();

    // What:     `let builtin_rules = cli.builtin_rules;` copies the parsed
    //           boolean flag, same tiny-copy shape as `all` below.
    // Why:      The rules-loading closure below branches on it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const builtinRules = cli.builtinRules;
    // ```
    let builtin_rules = cli.builtin_rules;

    // In TS you'd write (pseudocode):
    // ```ts
    // const rulesPath = cli.rulesPath ?? process.env.FORBIDDEN_STRINGS_RULES ??
    //   "forbidden-strings.local.txt";
    // ```
    let rules_entries: Vec<RulesEntry> = match rules_value {
        Some(value) => match parse_rules_list(&value) {
            Ok(parsed) => parsed,
            Err(error) => {
                eprintln!("forbidden-strings: invalid rules list in --rules or FORBIDDEN_STRINGS_RULES: {}", error);
                return Ok(2);
            }
        },
        None => vec![frx_load::single_entry(
            std::path::Path::new("forbidden-strings.local.txt"),
            builtin_rules,
            false,
        )],
    };

    // What:     `let all = cli.all;` copies the parsed boolean flag. `bool` is a
    //           tiny copy type, unlike owned `String` or `Vec<String>`.
    // Why:      The rest of the run loop already branches on a local named `all`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const all = cli.all;
    // ```
    let all = cli.all;

    // What:     `let mut files = cli.files;` moves the owned vector of parsed
    //           positional file paths out of the `Cli` struct and makes the local
    //           binding mutable. Sibling `Vec<&str>` would borrow, but clap gives us
    //           owned `String`s that can outlive parsing.
    // Why:      `--all` mode replaces this vector with walker output later; explicit
    //           positional mode keeps the clap-parsed values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let files = cli.files;
    // ```
    let mut files = cli.files;
    // Logical path overrides pair one-for-one with explicit content file operands.
    let name_paths = cli.name_paths;
    if !name_paths.is_empty() && (all || name_paths.len() != files.len()) {
        eprintln!("forbidden-strings: --name-path requires one value per positional file and cannot accompany --all");
        return Ok(2);
    }

    // Run `load_ruleset` and `list_files` concurrently when --all is
    // set: rules loading is CPU-bound (regex compile + AC build);
    // file walking is I/O-bound (directory traversal + gitignore parse).
    // They share no state, so overlapping them shaves whichever side
    // is shorter.
    // What:     `rayon::join(|| f1(), || f2())` runs two closures in
    //           parallel using the rayon threadpool. Returns a tuple
    //           of their return values once both finish. If only one
    //           closure has substantial work (e.g. when --all is off,
    //           we have no file walk to do), join still runs both --
    //           but the empty closure adds negligible cost.
    // Why:      Rules load is ~12ms for a 1k-rule ruleset; file walk
    //           is ~7ms on this repo. Sequential = 19ms; parallel = 12ms.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [rulesetResult, filesResult] = await Promise.all([
    //   loadRuleset(rulesPath),
    //   all ? listFiles(".") : Promise.resolve(null),
    // ]);
    // ```
    // What:     The first closure picks the loader by the `--builtin-rules`
    //           flag: `load_ruleset_with_builtin` appends the embedded
    //           baseline (and tolerates a missing implicit default file);
    //           `load_ruleset` is the unchanged flagless path.
    // Why:      Existing users see byte-identical behavior unless they opt in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ruleset = builtinRules
    //   ? loadRulesetWithBuiltin(rulesPath, explicitRulesSource)
    //   : loadRuleset(rulesPath);
    // ```
    // What:     `frx_load::load(rules_path, builtin_rules, explicit, BUILTIN_PRECOMPILED)`
    //           is the stage-two engine loader. It compiles the resolved runtime rules
    //           file from text and, under `--builtin-rules`, appends the embedded
    //           precompiled baseline; it returns a `LoadedRules` holding the ordered
    //           sets with their rule-id offsets. Errors are redacted (an I/O error
    //           names only the path; a compile error carries only an opaque index).
    // Why:      Replaces the resharp/`regex`/aho-corasick `load_ruleset` pipeline with
    //           the in-house engine while keeping this coordination point unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rulesResult = frxLoad(rulesPath, builtinRules, explicitRulesSource, BUILTIN_PRECOMPILED);
    // ```
    let startup_request = StartupRequest {
        rules_entries: &rules_entries,
        builtin_rules,
        quiet_absence: !explicit_rules_source,
        list_working_tree: all,
    };
    let (rules_result, listed_result): (Result<_>, Option<Result<WorkingTreeListing>>) = run_startup_work(&startup_request);

    // What:     `let loaded = match rules_result { Ok(r) => r, Err(e) => { ...; return ... } };`
    //           destructures the `Result<LoadedRules>`. `Ok(r)` binds the loaded sets;
    //           `Err(e)` prints the redacted error and early-returns exit 2.
    // Why:      Unwrap the `Result` while presenting a friendly error instead of a panic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let loaded: LoadedRules;
    // try { loaded = rulesResult; }
    // catch (e) { console.error(`forbidden-strings: ${e}`); process.exit(2); }
    // ```
    let loaded = match rules_result {
        Ok(r) => r,
        Err(e) => {
            // eprintln, not tracing: the user-facing CLI error contract is "forbidden-strings:
            // <msg>" on stderr, asserted by integration tests; a tracing target/level prefix
            // would break it.
            eprintln!("forbidden-strings: {}", e);
            return Ok(2);
        }
    };

    // Cache diagnostics are compact redacted JSON lines. Emit them before any
    // scan findings so cli-git's mixed-protocol parser sees deterministic order.
    for warning in loaded.cache_warnings() {
        eprintln!("{}", warning);
    }

    // What:     `if let Some(listed) = listed_result { match listed { ... } }`.
    //           One-arm pattern match: enter the block ONLY when
    //           `listed_result` is `Some`, binding the inner
    //           `Result<Vec<String>>` to `listed`. Inside, a
    //           regular `match` extracts `Ok` (replace `files` with the
    //           walker's output) or `Err` (print, exit 2).
    // Why:      `listed_result` is `Some(...)` only when `--all` was
    //           passed; otherwise `None` and we skip silently, leaving
    //           `files` set to whatever came from positional args.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (listedResult !== null) {
    //   try { files = listedResult; }
    //   catch (e) { console.error(`forbidden-strings: ${e}`); process.exit(2); }
    // }
    // ```
    if let Some(listed) = listed_result {
        match listed {
            Ok(listing) => {
                // What:     The tracked-file union's gap is printed before the
                //           scan starts, on the same stderr channel as every
                //           other CLI diagnostic, and its path goes through the
                //           pathname scan so a matching component is masked and
                //           protocol-sensitive bytes are escaped like every
                //           other path in a finding.
                // Why:      A scan that cannot cover force-added tracked files
                //           must say so; silence there reads as a clean scan.
                //           Printing the raw path would make this the one
                //           diagnostic that can quote a pathname the rules
                //           forbid.
                if let Some(gap) = listing.union_gap {
                    let shown = path_scan::scan_path(&gap.at.to_string_lossy(), &loaded).display;
                    eprintln!(
                        "forbidden-strings: the tracked-file union is unavailable: {}: {}; tracked files that .gitignore hides are not covered by this scan",
                        shown, gap.cause
                    );
                }
                files = listing.files;
            }
            Err(e) => {
                // eprintln, not tracing: same user-facing "forbidden-strings: <msg>" CLI error
                // contract on stderr asserted by integration tests.
                eprintln!("forbidden-strings: {}", e);
                return Ok(2);
            }
        }
    }

    // Fused read+scan: each rayon worker reads one file's bytes with
    // `read_with_binary_check` and immediately scans them. The two-phase
    // split that used to live here (Phase A reads, Phase B scans) traded
    // cache locality for a clean separation but produced no measurable
    // speedup -- after P1 the AC scan is so fast that file bytes go from
    // disk to AC to discard within tens of microseconds. Fusing keeps each
    // file's bytes hot in L1/L2 across the read->scan boundary instead of
    // risking eviction during the materialize-then-iterate round trip.
    // What:     `files.par_iter().enumerate().map_with(candidate_scan,
    //           scan_candidate_file).flatten_iter().collect::<Vec<String>>()`
    //           runs read+scan as one rayon work unit per file. Each
    //           worker's `Read` buffer lives only until that file's scan
    //           finishes; rayon work-steals across cores.
    // Why:      `read_with_binary_check` opens the file, reads a
    //           `BIN_PROBE_SIZE` prefix, and reads the whole file only when
    //           that prefix holds no NUL byte; a binary larger than the
    //           probe is scanned in the prefix alone. One plain read keeps
    //           the per-file cost to one open, one probe, and (for text)
    //           one `read_to_end`, with no per-file mapping setup. The
    //           mmap variant that once replaced it measured slower on this
    //           many-small-files workload and was removed; PERF.md records
    //           the measurement under "Mmap experiment (rejected)".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hits = (await Promise.all(
    //   files.map(async (p) => scanContent(p, await readWithBinaryCheck(p), ruleset))
    // )).flat();
    // ```
    // What:     Build the canonical-path skip set once at startup
    //           (rather than per-file). The set captures the actual
    //           rules file plus the canonical generated-source paths;
    //           empty when none of them resolve. Used only in --all
    //           mode to filter walker output.
    // Why:      Closes BUG 6: explicit positional args are never
    //           skipped; only walker output is filtered, and the filter
    //           is path-anchored (not basename-anchored), so
    //           `sub/forbidden-strings.local.txt` no longer collides
    //           with the actual rules file path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const skipSet = buildSkipSet(rulesPath);
    // ```
    let skip_set = if all { build_skip_set(&rules_entries) } else { std::collections::HashSet::new() };

    // What:     Canonical cwd, resolved once. `is_config_file_at_cwd`
    //           compares each candidate's canonical parent against this to
    //           skip the scanner's own `forbidden-strings.*.txt` ruleset
    //           files at the repo root, in both --all and explicit-arg modes.
    // Why:      Resolve symlinks once here rather than per file.
    let cwd_canonical = std::fs::canonicalize(".").ok();
    // A present `.git` at an ancestor identifies repository-relative name labels
    // and anchors the generated rule-source paths, wherever the run starts.
    let repository_root = path_scan::repository_root();
    // What:     The generated rule-source paths, built once for BOTH modes.
    // Why:      They hold literal copies of rule bodies and match their own
    //           rules, so a positional argument naming one (CI's changed-file
    //           list after a rename or edit) must be skipped the way the
    //           walker skips it. See `build_self_match_set`.
    let self_match_set = build_self_match_set(repository_root.as_deref());

    let candidate_scan = CandidateFileScan {
        all,
        cwd_canonical: &cwd_canonical,
        repository_root: &repository_root,
        self_match_set: &self_match_set,
        skip_set: &skip_set,
        name_paths: &name_paths,
        loaded: &loaded,
    };
    let hits: Vec<String> = files
        .par_iter()
        .enumerate()
        .map_with(candidate_scan, scan_candidate_file)
        .flatten_iter()
        .collect();

    // What:     `std::io::stderr().lock()` returns a `StderrLock`, an
    //           RAII handle holding the stderr mutex. Held writes
    //           don't interleave with other threads.
    // Why:      Print all hits in one batch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const h of hits) process.stderr.write(h + "\n");
    // ```
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    for h in &hits {
        let _ = writeln!(handle, "{}", h);
    }

    // What:     `if hits.is_empty() { Ok(0) } else { Ok(1) }`.
    //           This is an `if`-as-EXPRESSION (not statement) with no
    //           trailing `;`: its value becomes the function's return.
    //           `Ok(0)` and `Ok(1)` construct the success variant of
    //           `Result<i32>` with the OS-exit code inside; the
    //           bin wrapper converts to `ExitCode` for the actual exit.
    // Why:      No hits = clean exit; one or more hits = "violation"
    //           exit so CI marks the run as failed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return hits.length === 0 ? 0 : 1;
    // ```
    if hits.is_empty() {
        return Ok(0)
    } else {
        return Ok(1)
    }
}

//! Build step:
//!  precompile the embedded builtin baseline into a serialized `RegexSet`.
//!
//! The migration plan (open-questions resolution) measured that compiling the full
//! ported baseline at scanner startup is not viable:
//!  individual faithful rules take
//! seconds to determinize,
//!  tens of seconds in the worst case.
//!  So the baseline is
//! compiled once here,
//!  at build time,
//!  and serialized with the engine's `to_bytes`;
//! `lib.rs` embeds the blob with `include_bytes!` and the runtime loader rebuilds it
//! through the stage-one `load_precompiled` (the engine's validating `from_bytes`),
//! which only decodes,
//!  never recompiles.
//!  Mutable runtime rules use a separate
//! content-addressed per-user artifact and compile from text only on cache recovery.
//!
//! The two-form parse and literal escaper are shared verbatim with the runtime frx
//! compiler by `#[path]`-including the same source files:
//!  the build script cannot
//! `use` its own crate,
//!  so this is the only way to keep one parser.
//!  The engine is a
//! build-dependency so this script can call `RegexSet::new`.

// Scoped to this build-script compilation only. The shared `error.rs` this script
// `#[path]`-includes declares `LoadError::Compile` and `LoadError::Precompiled`, which
// the runtime `frx.rs` constructs but this build step does not (it only calls
// `parse_patterns`). Those variants are dead in the build-script crate yet live in the
// real crate, which keeps `-D dead_code`; suppressing here avoids editing the shared
// stage-one source to satisfy a build-script-only false positive.
#![allow(dead_code)]

/// Imports the engine's compiled-ruleset type built and serialized here.
use forbidden_regex::RegexSet;

/// Imports the environment and filesystem std pieces the build step reads and writes.
use std::{env, fs, path::Path};

/// Registers the redacted load-error type shared with the runtime frx compiler.
///
/// The parser returns it;
///  the build step never renders it with rule text,
///  only its
/// `Display`,
///  whose interpolations are an opaque index and a config flag letter.
#[path = "src/rule/frx/error.rs"]
mod error;

/// Registers the literal-to-verbose-dialect escaper shared with the runtime compiler.
#[path = "src/rule/frx/escape.rs"]
mod escape;

/// Registers the format autodetector,
///  legacy line parser,
///  and flag policy shared
/// with the runtime.
#[path = "src/rule/frx/format.rs"]
mod format;

/// Registers the tail-format sectioned parser shared with the runtime.
///
/// The `format` module this build script includes now routes through the section
/// parser,
///  so the section source must be included here too or the shared `#[path]`
/// parser would not compile.
///  The committed baseline is legacy-format,
///  so this parser
/// is never exercised at build time;
///  it only needs to compile.
#[path = "src/rule/frx/sections.rs"]
mod sections;

/// Compiles the ported baseline once and writes its serialized bytes into `OUT_DIR`,
/// beside a name sidecar carrying each rule's section name for finding identity.
///
/// Parses the committed,
///  generated `data/builtin-rules.txt` through the shared
/// parser,
///  compiles the whole set through the engine,
///  serializes it,
///  and writes the
/// blob for `include_bytes!` plus one sidecar line per rule (the rule's section
/// name,
///  or an empty line for an unnamed legacy rule) for `include_str!`.
///  A parse
/// or compile failure fails the build,
///  which is the correct fail-closed response to
/// a corrupt or un-ported baseline;
///  the error is surfaced only through its redacted
/// `Display`,
///  never rule text.
fn main() {
    // Rerun only when the baseline data or the shared parser sources change; the
    // build script's own edits are tracked by cargo automatically.
    println!("cargo:rerun-if-changed=data/builtin-rules.txt");
    println!("cargo:rerun-if-changed=src/rule/frx/format.rs");
    println!("cargo:rerun-if-changed=src/rule/frx/sections.rs");
    println!("cargo:rerun-if-changed=src/rule/frx/escape.rs");
    println!("cargo:rerun-if-changed=src/rule/frx/error.rs");

    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");

    let ported_path = Path::new(&manifest_dir).join("data/builtin-rules.txt");
    let ported = fs::read_to_string(&ported_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", ported_path.display()));

    // Parse the baseline into named, engine-ready rules, then compile the whole
    // set in one pass. `Display` on either error carries no rule text.
    let rules = format::parse_rules(&ported)
        .unwrap_or_else(|error| panic!("parse builtin baseline: {error}"));
    let patterns: Vec<String> =
        rules.iter().map(|rule| return rule.pattern.clone()).collect();
    let set = RegexSet::new(&patterns)
        .unwrap_or_else(|error| panic!("compile builtin baseline: {error}"));
    let bytes = set
        .to_bytes()
        .unwrap_or_else(|error| panic!("serialize builtin baseline: {error}"));

    let out_path = Path::new(&out_dir).join("builtin-rules-precompiled.bin");
    fs::write(&out_path, bytes)
        .unwrap_or_else(|error| panic!("write {}: {error}", out_path.display()));

    // One sidecar line per rule, in compiled order: the section name, or an empty
    // line for an unnamed legacy rule. The runtime loader rejects a count mismatch,
    // so the trailing newline per entry keeps `lines()` at exactly one per rule.
    let names_text: String = rules
        .iter()
        .map(|rule| return format!("{}\n", rule.name.as_deref().unwrap_or_default()))
        .collect();
    let names_path = Path::new(&out_dir).join("builtin-rules-names.txt");
    fs::write(&names_path, names_text)
        .unwrap_or_else(|error| panic!("write {}: {error}", names_path.display()));
}

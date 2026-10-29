//! Rule compiler targeting the in-house `forbidden-regex` engine.
//!
//! This module owns the two-form rule-file format,
//!  the flag policy,
//!  UTF-8 BOM
//! stripping,
//!  redacted load-error reporting,
//!  and both construction paths (from
//! rule text and from a serialized precompiled `RegexSet`).
//!  It is the live load
//! path:
//!  `crate::frx_load` builds the scan's rule sets through these entry points,
//! and `crate::frx_scan` runs them.
//!  The resharp/`regex`-crate pipeline it once sat
//! beside was deleted in the engine-swap teardown (#385).
//!
//! The engine is always in verbose,
//!  multiline mode,
//!  so this compiler never calls
//! `forbidden_regex::compile` (its single-pattern path logs the pattern via
//! `tracing`,
//!  a leak vector).
//!  It builds only through `RegexSet::new` and
//! `RegexSet::from_bytes`,
//!  neither of which logs,
//!  so a failing rule's bytes never
//! reach a subscriber.

/// Imports the engine's compiled-ruleset type and its compile-time error.
use forbidden_regex::{CompileError, RegexSet};

/// Registers the redacted load-error type.
mod error;
/// Registers the literal-to-verbose-dialect escaper.
mod escape;
/// Registers the format autodetector,
///  legacy line parser,
///  and flag policy.
mod format;
/// Registers the tail-format sectioned parser and header grammar.
mod sections;

/// Re-exports the redacted load-error type as this module's public failure.
pub use error::LoadError;

/// Re-exports the literal-to-verbose-dialect escaper for the scanner's fuzz targets.
///
/// The `fuzz_literal_roundtrip` target drives this escaper directly (not through the
/// two-form format layer,
///  whose `.trim()`,
///  comment,
///  and regex classification would
/// eat the adversarial cases it must exercise:
///  leading `#`,
///  embedded newlines,
///  a
/// leading `/`).
///  Gated on `fuzzing` so the production surface stays unchanged.
#[cfg(feature = "fuzzing")]
pub use escape::escape_literal;

/// One compiled rule set paired with the per-rule names that drive finding identity.
///
/// `names` is parallel to the set's rule indices:
///  a tail-format source names every
/// rule (its section name,
///  rendered in findings as `rule=<name>`),
///  a legacy source
/// names none (findings fall back to the offset numeric index).
///  Splitting the two
/// out of the parse keeps the engine set free of identity concerns.
pub struct CompiledRules {
    /// Compiled engine set the scan path runs.
    pub set: RegexSet,
    /// Per-rule section names parallel to the set's rule indices;
    ///  `None` for
    /// legacy line-based rules.
    pub names: Vec<Option<String>>,
}

/// Runtime rule kind retained across parser and hybrid matcher seam.
pub(crate) enum RuntimeRuleKind {
    /// Original exact literal bytes before dialect escaping.
    ExactLiteral(
        /// Exact source bytes consumed by direct literal matcher.
        Vec<u8>,
    ),
    /// Engine-ready restricted-regex dialect pattern.
    RestrictedRegex(
        /// Parsed dialect pattern consumed by in-house regex engine.
        String,
    ),
}

/// Parsed runtime rule with finding identity and pre-escape kind.
pub(crate) struct RuntimeRuleInput {
    /// Optional tail-format section identity.
    pub(crate) name: Option<String>,
    /// Exact-literal or restricted-regex matcher input.
    pub(crate) kind: RuntimeRuleKind,
}

/// Parses runtime source while preserving bare-literal distinction.
pub(crate) fn parse_runtime_rules(
    text: &str,
) -> Result<Vec<RuntimeRuleInput>, LoadError> {
    let rules = format::parse_rules(text)?;
    return Ok(rules
        .into_iter()
        .map(|rule| {
            let kind = if let Some(literal) = rule.literal {
                RuntimeRuleKind::ExactLiteral(literal)
            } else {
                RuntimeRuleKind::RestrictedRegex(rule.pattern)
            };
            return RuntimeRuleInput { name: rule.name, kind }
        })
        .collect())
}

/// Compiles a rule source into a combined `RegexSet` plus its per-rule names.
///
/// Parses the autodetected format (escaping literals,
///  applying the flag policy,
/// stripping a BOM),
///  then validates each rule through the engine to attribute a
/// redacted,
///  index-bearing error to the first offender,
///  and finally assembles the
/// combined set.
///  Validation and assembly both go through `RegexSet::new`,
///  so no
/// pattern is ever logged.
///  Errors carry only an opaque rule index and the
/// engine's static reason,
///  never rule text.
pub fn compile_rules(text: &str) -> Result<CompiledRules, LoadError> {
    let rules = format::parse_rules(text)?;
    let (names, patterns): (Vec<Option<String>>, Vec<String>) =
        rules.into_iter().map(|rule| return (rule.name, rule.pattern)).unzip();
    // Validate rule by rule first: `RegexSet::new` over the whole slice reports
    // only its first error without the offending index, so a per-rule pass
    // recovers the index for the redacted diagnostic. Each single-rule set is
    // discarded; the combined set is built once below.
    for (index, pattern) in patterns.iter().enumerate() {
        if let Err(reason) = RegexSet::new(std::slice::from_ref(pattern)) {
            return Err(LoadError::Compile { index, reason });
        }
    }
    // Every rule compiled individually, so assembly cannot fail on a rule; any
    // error here is a genuine engine invariant break, surfaced fail-closed with a
    // sentinel index and the engine's static reason (still no rule text).
    let set = RegexSet::new(&patterns).map_err(|reason| return LoadError::Compile {
        index: patterns.len(),
        reason,
    })?;
    return Ok(CompiledRules { set, names });
}

/// Compiles a rule source (autodetected format) into a combined `RegexSet`.
///
/// A projection over [`compile_rules`] for callers that only need the engine set
/// (the build-time baseline verifier,
///  the fuzz targets);
///  the runtime loader uses
/// [`compile_rules`] so findings can carry rule names.
pub fn compile_from_text(text: &str) -> Result<RegexSet, LoadError> {
    return compile_rules(text).map(|compiled| return compiled.set);
}

/// Loads a precompiled serialized `RegexSet` from bytes.
///
/// Wraps the engine's `from_bytes`,
///  which decodes and structurally validates the
/// blob before it can match.
///  The planning doc resolved that the builtin baseline
/// is embedded precompiled at build time (runtime compilation is too slow);
///  stage
/// two performs the embedding,
///  and this is the construction path it will use.
///  A
/// decode or validation failure becomes a redacted `Precompiled` error.
pub fn load_precompiled(bytes: &[u8]) -> Result<RegexSet, LoadError> {
    return RegexSet::from_bytes(bytes)
        .map_err(|reason: CompileError| return LoadError::Precompiled { reason });
}

/// Registers the compile,
///  redaction,
///  and precompiled round-trip tests
/// (sidecar,
///  lint-exempt).
#[cfg(test)]
mod compile_tests;

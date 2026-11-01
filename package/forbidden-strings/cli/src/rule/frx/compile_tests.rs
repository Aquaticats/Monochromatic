// Compile, redaction, and precompiled round-trip tests for the frx rule compiler.

use super::format::parse_patterns;
use super::{compile_from_text, compile_rules, load_precompiled, LoadError};
use forbidden_regex::{CompileError, RegexSet};
use rayon::prelude::*;

// The staged builtin baseline (crate data) and the repo-root append file.
const BUILTIN_PORTED: &str = include_str!("../../../data/builtin-rules.txt");
const APPEND_PORTED: &str = include_str!("../../../../../../forbidden-strings.append.txt");

// A distinctive, non-secret canary fed through every load-error path; it must
// never surface in any diagnostic or on the tracing/stderr path.
const SENTINEL: &str = "frx_redaction_canary_do_not_leak_Zq9";

// Builds one rule string per load-error path, each carrying the sentinel.
fn error_rules() -> [String; 4] {
    return [
        format!("/{SENTINEL}/i"),        // bad flag: policy rejects 'i'
        format!("/{SENTINEL}*/"),        // dialect rejection: '*' unsupported
        format!("/(?:{SENTINEL})?/"),    // empty-matchable: whole pattern nullable
        format!("/{SENTINEL}{{2000}}/"), // oversized repetition: count over the cap
    ]
}

#[test]
fn builtin_ported_all_compile() {
    // Exercise the module's parse+escape+flag pipeline over the whole ported
    // builtin, then compile each rule through the engine in parallel. Failures
    // are reported by opaque index only, never rule text.
    let patterns = parse_patterns(BUILTIN_PORTED).expect("parse builtin ported");
    assert!(!patterns.is_empty());
    let failures: Vec<usize> = patterns
        .par_iter()
        .enumerate()
        .filter_map(|(index, pattern)| {
            return RegexSet::new(std::slice::from_ref(pattern)).err().map(|_| return index)
        })
        .collect();
    assert!(failures.is_empty(), "builtin ported rules failed at indices {failures:?}");
}

#[test]
fn append_ported_compiles_end_to_end() {
    // The small append file runs through the full compile_from_text pipeline
    // (parse, escape, validate, assemble).
    let set = compile_from_text(APPEND_PORTED).expect("append ported compiles");
    assert!(!set.is_empty());
}

#[test]
fn each_error_path_is_exercised() {
    let bad_flag = compile_from_text(&format!("/{SENTINEL}/i")).expect_err("flag");
    assert!(matches!(bad_flag, LoadError::UnsupportedFlag { flag: 'i', .. }));

    let dialect = compile_from_text(&format!("/{SENTINEL}*/")).expect_err("dialect");
    assert!(matches!(dialect, LoadError::Compile { reason: CompileError::Syntax { .. }, .. }));

    let empty = compile_from_text(&format!("/(?:{SENTINEL})?/")).expect_err("empty");
    assert!(matches!(empty, LoadError::Compile { reason: CompileError::EmptyMatchable, .. }));

    let oversized = compile_from_text(&format!("/{SENTINEL}{{2000}}/")).expect_err("oversized");
    assert!(matches!(oversized, LoadError::Compile { reason: CompileError::Syntax { .. }, .. }));
}

#[test]
fn sentinel_never_leaks_through_a_diagnostic() {
    for rule in error_rules() {
        let err = compile_from_text(&rule).expect_err("must fail closed");
        assert!(!format!("{err}").contains(SENTINEL), "sentinel in Display of {err:?}");
        assert!(!format!("{err:?}").contains(SENTINEL), "sentinel in Debug");
    }
}

// A thread-safe sink capturing whatever a tracing subscriber would write out.
#[derive(Clone)]
struct BufSink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for BufSink {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("sink lock").extend_from_slice(data);
        return Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufSink {
    type Writer = BufSink;
    fn make_writer(&'a self) -> BufSink {
        return self.clone()
    }
}

#[test]
fn sentinel_never_reaches_a_tracing_subscriber() {
    // Even with a max-verbosity subscriber active, the compile path must emit
    // nothing carrying the sentinel: the module never calls the engine's
    // pattern-logging `compile`, only `RegexSet::new`.
    let buf = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(BufSink(buf.clone()))
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        for rule in error_rules() {
            let _ = compile_from_text(&rule);
        }
    });
    let captured = String::from_utf8_lossy(&buf.lock().expect("sink lock")).into_owned();
    assert!(!captured.contains(SENTINEL), "sentinel reached the tracing/stderr path");
}

#[test]
fn compile_rules_carries_names_per_format() {
    // A tail-format source names every rule with its section name; a legacy
    // source names none, so findings fall back to numeric ids.
    let tail = compile_rules("==> qqq-one <==\n/foo[0-9]/\n==> qqq-two <==\nabcdefgh\n")
        .expect("tail compiles");
    assert_eq!(tail.set.len(), 2);
    let tail_names: Vec<Option<&str>> =
        tail.names.iter().map(|name| return name.as_deref()).collect();
    assert_eq!(tail_names, vec![Some("qqq-one"), Some("qqq-two")]);

    let legacy = compile_rules("/foo[0-9]/\nabcdefgh\n").expect("legacy compiles");
    assert_eq!(legacy.set.len(), 2);
    assert_eq!(legacy.names, vec![None, None]);
}

#[test]
fn precompiled_round_trip_matches() {
    // A ruleset serialized via the engine's to_bytes reloads through the module's
    // from_bytes construction path and matches identically.
    let set = compile_from_text("/foo[0-9]/\nbar.baz").expect("compile");
    let bytes = set.to_bytes().expect("serialize");
    let reloaded = load_precompiled(&bytes).expect("load precompiled");
    assert_eq!(reloaded.len(), set.len());
    assert!(reloaded.is_match(b"foo7"));
    assert!(reloaded.is_match(b"bar.baz"));
    // The literal '.' was escaped, so a wildcard match is rejected.
    assert!(!reloaded.is_match(b"barxbaz"));
}

#[test]
fn precompiled_rejects_garbage_redacted() {
    let err = load_precompiled(b"not a valid serialized regexset").expect_err("garbage");
    assert!(matches!(err, LoadError::Precompiled { .. }));
    assert!(!format!("{err}").is_empty());
}

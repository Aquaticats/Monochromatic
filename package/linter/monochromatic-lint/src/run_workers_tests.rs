//! What:
//!  Controls for bounded workers,
//!  result ordering and panic containment.
//! Why:
//!  Output must not depend on scheduling,
//!  every plan must be processed exactly once at every
//! concurrency limit,
//!  and a panic must become that file's finding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(processPlans.name, () => { /* order, limits, exactly-once writes, containment */ });
//! ```

/// Import the worker entry points and fixture helpers.
use super::{
    WORKER_STACK_BYTES, contained_source, contained_with, lint_thread, panicked, process_plans,
};
use crate::run_file::{FileOutcome, SourceOutcome};
use crate::run_lfs::LfsRepos;
use crate::run_plan::FilePlan;
use crate::run_test_support::{ALL_RULES, CONFIG, plan, read, write};
use crate::run_write::WriteError;
use crate::rust_file_engine::RustFileEngine;
use crate::rust_workspace::WorkspacePreparation;
use crate::test_fs::Fixture;

/// Discard workspace progress;
///  no test here loads a workspace.
fn progress(_message: String) {}

/// An engine that must never be asked to load anything in these tests.
fn engine() -> RustFileEngine {
    return RustFileEngine::new(WorkspacePreparation::SourceOnly, progress);
}

/// Every concurrency limit processes each plan once and returns outcomes in plan order.
#[test]
fn every_limit_processes_each_plan_once_in_plan_order() {
    for concurrency in [1, 2, 3, 64] {
        let fixture: Fixture = Fixture::new();
        write(&fixture.path, CONFIG, ALL_RULES);
        let mut plans: Vec<FilePlan> = Vec::<FilePlan>::new();
        for index in 0..17 {
            let name: String = format!("doc/f{index:02}.md");
            // Each file has one fixable finding and a distinct number of remaining duplicate headings.
            let mut source: String = String::from("# Title.\n");
            for _ in 0..index {
                source.push_str("\n# Title\n");
            }
            write(&fixture.path, name.as_str(), source.as_str());
            plans.push(plan(&fixture.path, name.as_str()));
        }
        let lfs: LfsRepos = LfsRepos::new();
        let mut semantic: RustFileEngine = engine();
        let outcomes: Vec<FileOutcome> =
            process_plans(plans.as_slice(), true, &lfs, concurrency, &mut semantic);
        assert_eq!(outcomes.len(), 17, "limit {concurrency}");
        for (index, outcome) in outcomes.iter().enumerate() {
            let name: String = format!("doc/f{index:02}.md");
            assert!(outcome.written, "{name} at limit {concurrency}");
            // single-h1 and no-duplicate-heading each report once per extra heading.
            assert_eq!(
                outcome.findings.len(),
                index * 2,
                "{name} at limit {concurrency}"
            );
            for finding in &outcome.findings {
                assert_eq!(finding.filename, name);
            }
            assert!(
                read(&fixture.path, name.as_str()).starts_with("# Title\n"),
                "{name} at limit {concurrency}"
            );
        }
        assert_eq!(semantic.workspace_count(), 0);
    }
}

/// No plans is an ordinary empty result at any limit.
#[test]
fn no_plans_yield_no_outcomes() {
    let lfs: LfsRepos = LfsRepos::new();
    let mut semantic: RustFileEngine = engine();
    assert!(process_plans(&[], true, &lfs, 4, &mut semantic).is_empty());
    assert!(process_plans(&[], false, &lfs, 1, &mut semantic).is_empty());
}

/// Lint mode at a high limit writes nothing and still reports every file.
#[test]
fn lint_mode_reports_without_writing_at_any_limit() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let mut plans: Vec<FilePlan> = Vec::<FilePlan>::new();
    for index in 0..5 {
        let name: String = format!("f{index}.md");
        write(&fixture.path, name.as_str(), "# Title.\n");
        plans.push(plan(&fixture.path, name.as_str()));
    }
    let lfs: LfsRepos = LfsRepos::new();
    let mut semantic: RustFileEngine = engine();
    let outcomes: Vec<FileOutcome> = process_plans(plans.as_slice(), false, &lfs, 8, &mut semantic);
    assert_eq!(outcomes.len(), 5);
    for (index, outcome) in outcomes.iter().enumerate() {
        assert!(!outcome.written);
        assert_eq!(outcome.findings.len(), 1);
        assert_eq!(outcome.findings[0].filename, format!("f{index}.md"));
        assert_eq!(
            read(&fixture.path, format!("f{index}.md").as_str()),
            "# Title.\n"
        );
    }
}

/// A plan that needs the semantic engine is processed on the calling thread and keeps its slot.
#[test]
fn semantic_plans_keep_their_position_among_worker_plans() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        CONFIG,
        r#"[
          { "files": ["**/*.md"], "rules": { "markdown/no-trailing-punctuation": { "severity": "error" } } },
          { "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }
        ]"#,
    );
    let mut plans: Vec<FilePlan> = Vec::<FilePlan>::new();
    for name in ["a.md", "b.rs", "c.md", "d.md"] {
        // The Rust file has no Cargo manifest above it, so semantic checking fails explicitly.
        write(&fixture.path, name, "# Title.\n");
        plans.push(plan(&fixture.path, name));
    }
    assert!(plans[1].needs_semantic_engine());
    let lfs: LfsRepos = LfsRepos::new();
    let mut semantic: RustFileEngine = engine();
    let outcomes: Vec<FileOutcome> = process_plans(plans.as_slice(), false, &lfs, 4, &mut semantic);
    let mut codes: Vec<&str> = Vec::<&str>::new();
    for outcome in &outcomes {
        assert_eq!(outcome.findings.len(), 1);
        codes.push(outcome.findings[0].code.as_str());
    }
    assert_eq!(
        codes,
        [
            "markdown/no-trailing-punctuation",
            "core/processing-failure",
            "markdown/no-trailing-punctuation",
            "markdown/no-trailing-punctuation"
        ]
    );
    assert_eq!(outcomes[1].findings[0].filename, "b.rs");
    assert!(
        outcomes[1].findings[0].message.contains("Cargo.toml"),
        "{}",
        outcomes[1].findings[0].message
    );
}

/// A caught panic becomes one processing finding that names the payload and says the file was not verified.
#[test]
fn a_panic_becomes_a_processing_finding() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let planned: FilePlan = plan(&fixture.path, "a.md");
    let payload: Box<dyn std::any::Any + Send> = Box::new(String::from("parser exploded"));
    let outcome: FileOutcome = panicked(&planned, payload.as_ref());
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].code, "core/processing-failure");
    assert_eq!(outcome.findings[0].filename, "a.md");
    assert!(outcome.findings[0].processing_failure);
    assert_eq!(
        outcome.findings[0].message,
        "Processing panicked: parser exploded. This file was not verified and was not rewritten."
    );
    assert!(!outcome.written);
}

/// In-memory processing is contained the same way and returns ordinary outcomes unchanged.
#[test]
fn in_memory_processing_passes_ordinary_outcomes_through() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    let planned: FilePlan = plan(&fixture.path, "a.md");
    let lfs: LfsRepos = LfsRepos::new();
    let outcome: SourceOutcome = contained_source(&planned, "# Title.\n", true, &lfs, None);
    assert_eq!(outcome.fixed.as_deref(), Some("# Title\n"));
    assert!(outcome.findings.is_empty());
}

/// Worker threads get the main thread's usual 8 MiB stack,
///  so nesting that a default 2 MiB thread
/// stack cannot hold still parses there.
///  The Rust parser recurses once per parenthesis:
///  in the bounded
/// container the debug executable needed more than 3 MiB and at most 4 MiB of main-thread stack for
/// this depth (`ulimit -s` 3072 KiB overflowed,
///  4096 KiB passed).
///  A smaller worker stack,
///  or running
/// these files on this 2 MiB test thread instead of on workers,
///  overflows and aborts the test binary.
#[test]
fn workers_parse_nesting_deeper_than_a_default_thread_stack_holds() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    // Documented items and one nested expression, so a completed check has no finding at all.
    let nested: String = format!(
        "//! Nested expression.\n\n/// Entry point.\nfn main() {{\n    let x: u32 = {}1{};\n}}\n",
        "(".repeat(1200),
        ")".repeat(1200)
    );
    let mut plans: Vec<FilePlan> = Vec::<FilePlan>::new();
    // Two files at a limit of two start two workers; one file would stay on this thread.
    for name in ["a.rs", "b.rs"] {
        write(&fixture.path, name, nested.as_str());
        plans.push(plan(&fixture.path, name));
    }
    let lfs: LfsRepos = LfsRepos::new();
    let mut semantic: RustFileEngine = engine();
    let outcomes: Vec<FileOutcome> = process_plans(plans.as_slice(), false, &lfs, 2, &mut semantic);
    assert_eq!(outcomes.len(), 2);
    for outcome in &outcomes {
        assert_eq!(outcome.findings, Vec::new());
        assert!(!outcome.written);
    }
}

/// Stack bytes the probe must use below its thread's first frame:
///  7.5 MiB,
///  half a mebibyte under the lint
/// stack,
///  which leaves room for the thread's own start-up frames and its thread-local storage.
const PROBE_BYTES: usize = 7 * 1024 * 1024 + 512 * 1024;

/// Recurse in frames of at least 4 KiB until the frames below `top` span `target` bytes,
///  and return that span.
/// The span is measured from frame addresses,
///  so it does not depend on how large the compiler makes each frame.
fn stack_reach(top: usize, target: usize) -> usize {
    let frame: [u8; 4096] = std::hint::black_box([0_u8; 4096]);
    let span: usize = top.abs_diff((&raw const frame).addr());
    if span >= target {
        return span;
    }
    // Using the frame after the call keeps it alive across the recursion.
    return stack_reach(top, target).max(usize::from(std::hint::black_box(frame[0])));
}

/// Probe how much stack the current thread can use,
///  starting from this frame.
fn probe_stack() -> usize {
    let anchor: u8 = std::hint::black_box(0);
    return stack_reach((&raw const anchor).addr(), PROBE_BYTES);
}

/// A thread from `lint_thread` holds 7.5 MiB of frames,
///  so its stack is not below 8 MiB by more than its own
/// start-up needs,
///  and the constant is the documented 8 MiB.
///  The nesting control passes from about 4 MiB,
///  and every
/// stack smaller than the probe's span overflows here and aborts the test binary.
#[test]
fn lint_threads_hold_the_documented_eight_mebibyte_stack() {
    let reached: usize = lint_thread()
        .spawn(probe_stack)
        .expect("start a lint thread")
        .join()
        .expect("the probe returns");
    assert!(reached >= PROBE_BYTES, "{reached}");
    assert_eq!(WORKER_STACK_BYTES, 8 * 1024 * 1024);
}

/// A writer that panics,
///  standing in for a defect inside per-file processing.
fn panicking_writer(_path: &std::path::Path, _contents: &[u8]) -> Result<(), WriteError> {
    panic!("writer exploded");
}

/// A real panic inside the contained region is caught,
///  reported for that file,
///  and leaves the file unchanged.
#[test]
fn a_real_panic_is_contained_per_file() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, CONFIG, ALL_RULES);
    write(&fixture.path, "a.md", "# Title.\n");
    let planned: FilePlan = plan(&fixture.path, "a.md");
    let lfs: LfsRepos = LfsRepos::new();
    let outcome: FileOutcome = contained_with(&planned, true, &lfs, None, panicking_writer);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(
        outcome.findings[0].message,
        "Processing panicked: writer exploded. This file was not verified and was not rewritten."
    );
    assert!(outcome.findings[0].processing_failure);
    assert!(!outcome.written);
    assert_eq!(read(&fixture.path, "a.md"), "# Title.\n");
    // Lint mode never reaches the writer, so the same plan is processed normally.
    let lint: FileOutcome = contained_with(&planned, false, &lfs, None, panicking_writer);
    assert_eq!(lint.findings.len(), 1);
    assert_eq!(lint.findings[0].code, "markdown/no-trailing-punctuation");
}

//! What:
//!  THROWAWAY SPIKE controls for the closure `'static` classification.
//! Why:
//!  Each case carries the verdict the proposal expects; the harness reports what hir actually answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const c of cases) compare(expected, classify(c.source)); also flip the bound and expect the verdict to flip.
//! ```

/// Import the spike switch.
use crate::rust_closure_static_spike::force;
/// Import settings without any rule selected.
use crate::rust_rule_settings::RustRuleSettings;
/// Import the production fixture.
use crate::rust_semantic_test_support::SemanticFixture;

/// What the proposal expects for one closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Expect {
    /// Closure passes the criterion.
    Allow,
    /// Closure stays reported.
    Report,
}

/// One closure expectation: `needle` is the exact source text that starts the closure.
struct Want {
    /// Text at which the closure (or macro call for macro cases) starts.
    needle: &'static str,
    /// Proposal verdict.
    expect: Expect,
    /// True when hir is known to answer differently (recorded in the report).
    known_gap: bool,
}

/// One fixture source and its closures.
struct Case {
    /// Label.
    name: &'static str,
    /// Complete Rust source.
    source: &'static str,
    /// Closures of interest.
    wants: &'static [Want],
    /// Also run the twin with every `+ 'static` bound removed; each allow must flip to report.
    flip: bool,
}

/// One observed verdict.
struct Observed {
    /// Start offset of the closure, or of the macro call for in-macro closures.
    start: usize,
    /// ALLOW or REPORT.
    allow: bool,
    /// True for closures found by expanding a macro call.
    in_macro: bool,
    /// Evidence trail.
    trace: String,
}

/// Run the spike over one source and parse the findings.
fn observe(fixture: &mut SemanticFixture, source: &str) -> Vec<Observed> {
    force(true);
    let findings = fixture
        .session
        .check_rules(&fixture.source_path, source, "input.rs", RustRuleSettings::default())
        .expect("check spike snapshot");
    force(false);
    let mut seen: Vec<Observed> = Vec::new();
    for finding in findings {
        if finding.code != "spike/closure-static" {
            continue;
        }
        let message: String = finding.message;
        let in_macro: bool = message.starts_with("macro-closure@");
        if !message.starts_with("closure@") && !in_macro {
            seen.push(Observed { start: finding.labels[0].span.offset, allow: false, in_macro: true, trace: message });
            continue;
        }
        let (head, trace): (&str, &str) = message.split_once(" :: ").expect("trace separator");
        let (position, word): (&str, &str) = head.split_once(' ').expect("verdict word");
        let start: usize = position.split_once('@').expect("at").1.parse().expect("offset");
        seen.push(Observed { start, allow: word == "ALLOW", in_macro, trace: String::from(trace) });
    }
    return seen;
}

/// Verdict text.
fn word(allow: bool) -> &'static str {
    return if allow { "allow" } else { "report" };
}

/// Run one case and one twin; return table lines and the list of unexpected results.
fn run_case(fixture: &mut SemanticFixture, case: &Case, source: &str, twin: bool, table: &mut Vec<String>, bad: &mut Vec<String>) {
    let observed: Vec<Observed> = observe(fixture, source);
    for want in case.wants {
        let start: usize = source.find(want.needle).unwrap_or_else(|| panic!("needle {:?} missing in {}", want.needle, case.name));
        let hit = observed.iter().find(|item| item.start == start);
        let (got, trace): (String, String) = match hit {
            Some(item) => (String::from(word(item.allow)), format!("{}{}", if item.in_macro { "[in macro] " } else { "" }, item.trace)),
            None => (String::from("not-visited"), String::from("no closure verdict at this offset (not visible to the syntax walk)")),
        };
        let expected: Expect = if twin { Expect::Report } else { want.expect };
        let matches: bool = got == word(expected == Expect::Allow);
        let status: &str = match (matches, want.known_gap && !twin) {
            (true, false) => "OK",
            (false, true) => "GAP",
            (false, false) => "MISMATCH",
            (true, true) => "UNEXPECTED-PASS",
        };
        table.push(format!(
            "{}{} | needle {:?} | expected {} | got {} | {} | {}",
            case.name,
            if twin { " (twin: all `+ 'static` removed)" } else { "" },
            want.needle,
            word(expected == Expect::Allow),
            got,
            status,
            trace
        ));
        if status == "MISMATCH" || status == "UNEXPECTED-PASS" {
            bad.push(table.last().expect("pushed").clone());
        }
    }
}

/// Remove every `'static` bound so only the bound changes.
fn without_static(source: &str) -> String {
    return source.replace(" + 'static", "").replace("trait Task: 'static", "trait Task");
}

/// The case table.
fn cases() -> Vec<Case> {
    return vec![
        Case { name: "thread::spawn(move || ..) capturing", flip: false,
            source: "use std::thread;\nfn main() { let state = String::from(\"x\"); let h = thread::spawn(move || state.len()); let _ = h.join(); }\n",
            wants: &[Want { needle: "move || state.len()", expect: Expect::Allow, known_gap: false }] },
        Case { name: "thread::Builder::new().spawn(move || ..)", flip: false,
            source: "use std::thread;\nfn main() { let state = String::from(\"x\"); let _ = thread::Builder::new().spawn(move || state.len()); }\n",
            wants: &[Want { needle: "move || state.len()", expect: Expect::Allow, known_gap: false }] },
        Case { name: "slint-style on_ping(&self, impl FnMut(i32) -> () + 'static)", flip: true,
            source: "struct Win;\nimpl Win { pub fn on_ping(&self, mut f: impl FnMut(i32) -> () + 'static) { f(1); } }\nfn main() { let w = Win; let n = 5; w.on_ping(move |x| { let _ = x + n; }); }\n",
            wants: &[Want { needle: "move |x| { let _ = x + n; }", expect: Expect::Allow, known_gap: false }] },
        Case { name: "fn run<F: FnOnce() + 'static>(f: F)", flip: true,
            source: "fn run<F: FnOnce() + 'static>(f: F) { f() }\nfn main() { let s = String::new(); run(move || drop(s)); }\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "fn run<F>(f: F) where F: FnOnce() + 'static", flip: true,
            source: "fn run<F>(f: F) where F: FnOnce() + 'static { f() }\nfn main() { let s = String::new(); run(move || drop(s)); }\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "Box<dyn FnMut() + 'static> parameter via Box::new", flip: false,
            source: "fn run(mut f: Box<dyn FnMut() + 'static>) { f() }\nfn main() { let s = String::new(); run(Box::new(move || { let _ = &s; })); }\n",
            wants: &[Want { needle: "move || { let _ = &s; }", expect: Expect::Allow, known_gap: false }] },
        Case { name: "Box<dyn FnMut()> parameter (implicit 'static default)", flip: false,
            source: "fn run(mut f: Box<dyn FnMut()>) { f() }\nfn main() { let s = String::new(); run(Box::new(move || { let _ = &s; })); }\n",
            wants: &[Want { needle: "move || { let _ = &s; }", expect: Expect::Allow, known_gap: false }] },
        Case { name: "Box<dyn FnMut() + 'a> parameter (not static)", flip: false,
            source: "fn run<'a>(mut f: Box<dyn FnMut() + 'a>) { f() }\nfn main() { let s = String::new(); run(Box::new(|| { let _ = &s; })); }\n",
            wants: &[Want { needle: "|| { let _ = &s; }", expect: Expect::Report, known_gap: false }] },
        Case { name: "impl FnMut() + Send + 'static", flip: true,
            source: "fn run(mut f: impl FnMut() + Send + 'static) { f() }\nfn main() { let s = String::new(); run(move || { let _ = &s; }); }\n",
            wants: &[Want { needle: "move || { let _ = &s; }", expect: Expect::Allow, known_gap: false }] },
        Case { name: "trait method, concrete receiver", flip: true,
            source: "trait Spawner { fn go(&self, f: impl FnOnce() + 'static); }\nstruct S;\nimpl Spawner for S { fn go(&self, f: impl FnOnce() + 'static) { f() } }\nfn main() { let s = String::new(); S.go(move || drop(s)); }\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "trait method, receiver is a generic type parameter T: Spawner", flip: true,
            source: "trait Spawner { fn go(&self, f: impl FnOnce() + 'static); }\nfn via<T: Spawner>(t: &T) { let s = String::new(); t.go(move || drop(s)); }\nfn main() {}\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "trait method, UFCS Spawner::go(&s, ..)", flip: true,
            source: "trait Spawner { fn go(&self, f: impl FnOnce() + 'static); }\nstruct S;\nimpl Spawner for S { fn go(&self, f: impl FnOnce() + 'static) { f() } }\nfn main() { let s = String::new(); Spawner::go(&S, move || drop(s)); }\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "trait method, dyn receiver with Box<dyn FnOnce() + 'a> (not static)", flip: false,
            source: "trait Spawner { fn go<'a>(&self, f: Box<dyn FnOnce() + 'a>); }\nfn via(t: &dyn Spawner) { let s = String::new(); t.go(Box::new(move || drop(s))); }\nfn main() {}\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Report, known_gap: false }] },
        Case { name: "slint Timer::start style: closure is the third argument", flip: true,
            source: "struct Timer;\nimpl Timer { fn start(&self, mode: u8, interval: u32, mut callback: impl FnMut() + 'static) { let _ = (mode, interval); callback(); } }\nfn main() { let t = Timer; let n = 1; t.start(0, 10, move || { let _ = n + 1; }); }\n",
            wants: &[Want { needle: "move || { let _ = n + 1; }", expect: Expect::Allow, known_gap: false }] },
        Case { name: "closure wrapped in redundant parentheses as a spawn argument", flip: false,
            source: "fn main() { let s = String::new(); let _ = std::thread::spawn((move || s.len())); }\n",
            wants: &[Want { needle: "move || s.len()", expect: Expect::Allow, known_gap: false }] },
        Case { name: "trait method, dyn receiver with Box<dyn FnOnce() + 'static>", flip: false,
            source: "trait Spawner { fn go(&self, f: Box<dyn FnOnce() + 'static>); }\nfn via(t: &dyn Spawner) { let s = String::new(); t.go(Box::new(move || drop(s))); }\nfn main() {}\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "impl-level bound: impl<F: FnMut() + 'static> Holder<F> { fn set(.., f: F) }", flip: true,
            source: "struct Holder<F> { f: Option<F> }\nimpl<F: FnMut() + 'static> Holder<F> { fn set(&mut self, f: F) { self.f = Some(f); } }\nfn main() { let s = String::new(); let mut h = Holder { f: None }; h.set(move || { let _ = &s; }); }\n",
            wants: &[Want { needle: "move || { let _ = &s; }", expect: Expect::Allow, known_gap: false }] },
        Case { name: "supertrait bound: F: Task, trait Task: 'static", flip: true,
            source: "trait Task: 'static { fn run(self); }\nimpl<F: FnOnce() + 'static> Task for F { fn run(self) { self() } }\nfn submit<T: Task>(t: T) { t.run() }\nfn main() { let s = String::new(); submit(move || drop(s)); }\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: false }] },
        Case { name: "callee is a value of fn-trait type: G: Fn(Box<dyn FnOnce() + 'static>)", flip: false,
            source: "fn go<G: Fn(Box<dyn FnOnce() + 'static>)>(g: G) { let s = String::new(); g(Box::new(move || drop(s))); }\nfn main() {}\n",
            wants: &[Want { needle: "move || drop(s)", expect: Expect::Allow, known_gap: true }] },
        Case { name: "std::panic::catch_unwind(|| ..)", flip: false,
            source: "fn main() { let v = vec![1]; let _ = std::panic::catch_unwind(|| v.len()); }\n",
            wants: &[Want { needle: "|| v.len()", expect: Expect::Report, known_gap: false }] },
        Case { name: "v.iter().map(|x| x + offset)", flip: false,
            source: "fn main() { let v = vec![1, 2]; let offset = 2; let w: Vec<i32> = v.iter().map(|x| x + offset).collect(); drop(w); }\n",
            wants: &[Want { needle: "|x| x + offset", expect: Expect::Report, known_gap: false }] },
        Case { name: "let-bound closure then spawn(f)", flip: false,
            source: "use std::thread;\nfn main() { let state = String::new(); let f = move || state.len(); let _ = thread::spawn(f); }\n",
            wants: &[Want { needle: "move || state.len()", expect: Expect::Report, known_gap: false }] },
        Case { name: "capture-free closure passed to thread::spawn", flip: false,
            source: "use std::thread;\nfn main() { let _ = thread::spawn(move || 1 + 1); }\n",
            wants: &[Want { needle: "move || 1 + 1", expect: Expect::Report, known_gap: false }] },
        Case { name: "capture-free closure passed to 'static fn (twin of the captured case)", flip: false,
            source: "fn run<F: FnOnce() + 'static>(f: F) { f() }\nfn main() { run(|| ()); }\n",
            wants: &[Want { needle: "|| ()", expect: Expect::Report, known_gap: false }] },
        Case { name: "nested closure: outer spawn body and inner map callback", flip: false,
            source: "use std::thread;\nfn main() { let offset = 2; let _ = thread::spawn(move || { let v = vec![1, 2]; v.iter().map(|x| x + offset).sum::<i32>() }); }\n",
            wants: &[
                Want { needle: "move || { let v", expect: Expect::Allow, known_gap: false },
                Want { needle: "|x| x + offset", expect: Expect::Report, known_gap: false },
            ] },
        Case { name: "scoped threads: thread::scope(|s| s.spawn(|| ..)) (no 'static)", flip: false,
            source: "use std::thread;\nfn main() { let v = vec![1]; thread::scope(|s| { s.spawn(|| v.len()); }); }\n",
            wants: &[
                Want { needle: "|s| { s.spawn", expect: Expect::Report, known_gap: false },
                Want { needle: "|| v.len()", expect: Expect::Report, known_gap: false },
            ] },
        Case { name: "closure inside assert!(..) token tree", flip: false,
            source: "fn main() { let v = vec![1, 2]; let offset = 1; assert!(v.iter().any(|x| *x > offset)); }\n",
            wants: &[Want { needle: "assert!(", expect: Expect::Report, known_gap: false }] },
        Case { name: "closure inside user macro_rules! wrapper around thread::spawn", flip: false,
            source: "macro_rules! go { ($e:expr) => { std::thread::spawn($e) }; }\nfn main() { let state = String::new(); let _ = go!(move || state.len()); }\n",
            wants: &[Want { needle: "go!(move", expect: Expect::Allow, known_gap: false }] },
    ];
}

/// Run all cases and twins; write the table when `SPIKE_TABLE_OUT` names a file.
fn run_all(table_out: bool) -> (Vec<String>, Vec<String>) {
    let mut fixture: SemanticFixture = SemanticFixture::new();
    let mut table: Vec<String> = Vec::new();
    let mut bad: Vec<String> = Vec::new();
    for case in cases() {
        run_case(&mut fixture, &case, case.source, false, &mut table, &mut bad);
        if case.flip {
            let twin: String = without_static(case.source);
            assert_ne!(twin, case.source, "twin must differ: {}", case.name);
            let allow_wants: Vec<&Want> = case.wants.iter().filter(|want| want.expect == Expect::Allow).collect();
            for want in allow_wants {
                run_single(&mut fixture, case.name, &twin, want, &mut table, &mut bad);
            }
        }
    }
    if table_out {
        if let Some(path) = std::env::var_os("SPIKE_TABLE_OUT") {
            std::fs::write(path, table.join("\n") + "\n").expect("write table");
        }
    }
    return (table, bad);
}

/// Run one twin closure expectation, which must flip to report.
fn run_single(fixture: &mut SemanticFixture, name: &'static str, source: &str, want: &Want, table: &mut Vec<String>, bad: &mut Vec<String>) {
    let observed: Vec<Observed> = observe(fixture, source);
    let start: usize = source.find(want.needle).expect("needle in twin");
    let hit = observed.iter().find(|item| item.start == start);
    let (got, trace): (&str, String) = match hit {
        Some(item) => (word(item.allow), item.trace.clone()),
        None => ("not-visited", String::new()),
    };
    let status: &str = if got == "report" { "OK" } else { "MISMATCH" };
    table.push(format!("{name} (twin: all `+ 'static` removed) | needle {:?} | expected report | got {got} | {status} | {trace}", want.needle));
    if status == "MISMATCH" {
        bad.push(table.last().expect("pushed").clone());
    }
}

/// The full table: every proposal expectation must match, except recorded gaps, and every twin must flip.
#[test]
fn closure_static_cases_and_flip_controls() {
    let (table, bad) = run_all(true);
    for line in &table {
        eprintln!("SPIKE-CASE {line}");
    }
    assert!(bad.is_empty(), "unexpected results:\n{}", bad.join("\n"));
}

/// The harness must flag a deliberately wrong expectation: spawn with capture claimed to be reported.
#[test]
fn harness_flags_a_deliberately_wrong_expectation() {
    let mut fixture: SemanticFixture = SemanticFixture::new();
    let wrong: Case = Case {
        name: "WRONG on purpose: thread::spawn(move || ..) expected report",
        flip: false,
        source: "use std::thread;\nfn main() { let state = String::from(\"x\"); let h = thread::spawn(move || state.len()); let _ = h.join(); }\n",
        wants: &[Want { needle: "move || state.len()", expect: Expect::Report, known_gap: false }],
    };
    let mut table: Vec<String> = Vec::new();
    let mut bad: Vec<String> = Vec::new();
    run_case(&mut fixture, &wrong, wrong.source, false, &mut table, &mut bad);
    eprintln!("SPIKE-CONTROL {}", table.join("\n"));
    assert_eq!(bad.len(), 1, "the wrong expectation must be flagged as MISMATCH: {table:?}");
    assert!(bad[0].contains("MISMATCH"));
}

/// Same wrong expectation through the full-table assertion: it must panic.
#[test]
#[should_panic(expected = "unexpected results")]
fn wrong_expectation_fails_the_table_assertion() {
    let mut fixture: SemanticFixture = SemanticFixture::new();
    let mut table: Vec<String> = Vec::new();
    let mut bad: Vec<String> = Vec::new();
    let wrong: Case = Case {
        name: "WRONG on purpose",
        flip: false,
        source: "fn main() { let _ = std::thread::spawn(move || 1 + 1); }\n",
        wants: &[Want { needle: "move || 1 + 1", expect: Expect::Allow, known_gap: false }],
    };
    run_case(&mut fixture, &wrong, wrong.source, false, &mut table, &mut bad);
    assert!(bad.is_empty(), "unexpected results:\n{}", bad.join("\n"));
}

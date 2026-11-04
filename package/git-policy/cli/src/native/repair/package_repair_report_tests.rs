//! The report: nothing for no findings, one success line, a failure line that names its command, and `--quiet`.

use super::*;
use std::path::PathBuf;

/// A destination with a space and a quote in it, which the printed text must carry exactly.
fn destination() -> PathBuf {
    return PathBuf::from("/work/new tree");
}

/// A finding that worked.
fn worked(tool: Tool, millis: u128) -> Finding {
    return Finding { tool, outcome: Outcome::Repaired { detail: format!("{tool:?} detail"), millis } };
}

/// A finding that failed.
fn failed(tool: Tool) -> Finding {
    return Finding { tool, outcome: Outcome::Failed { reason: format!("{tool:?} reason"), run_instead: format!("run {tool:?} instead") } };
}

/// No finding prints nothing, quiet or not.
#[test]
fn no_findings_print_nothing() {
    assert_eq!(render(destination().as_path(), &[], false), "");
    assert_eq!(render(destination().as_path(), &[], true), "");
}

/// Successes share one line that names the destination, each tool's detail and the time it took.
#[test]
fn successes_share_one_line() {
    let text: String = render(destination().as_path(), &[worked(Tool::Pnpm, 2_100), worked(Tool::Cargo, 300)], false);
    assert_eq!(text.lines().count(), 1, "{text}");
    assert!(text.contains(destination().to_str().expect("utf8")), "{text}");
    assert!(text.contains("Pnpm detail") && text.contains("Cargo detail"), "{text}");
    assert!(text.contains("2.1 s"), "the time is shown in seconds: {text}");
    assert!(text.ends_with('\n'), "{text:?}");
}

/// `--quiet` silences a success.
#[test]
fn quiet_silences_success() {
    assert_eq!(render(destination().as_path(), &[worked(Tool::Pnpm, 5)], true), "");
}

/// A failure prints under `--quiet` too, with the reason and the command that finishes it, one line each.
#[test]
fn failures_print_even_when_quiet() {
    for quiet in [false, true] {
        let text: String = render(destination().as_path(), &[failed(Tool::Pnpm), failed(Tool::Cargo)], quiet);
        assert_eq!(text.lines().count(), 2, "{text}");
        for tool in [Tool::Pnpm, Tool::Cargo] {
            assert!(text.contains(format!("{tool:?} reason").as_str()) && text.contains(format!("run {tool:?} instead").as_str()), "{text}");
        }
    }
}

/// A mix prints the failures and, when not quiet, the success.
#[test]
fn a_mix_prints_both_kinds_unless_quiet() {
    let findings: [Finding; 2] = [worked(Tool::Cargo, 10), failed(Tool::Toolchain)];
    let loud: String = render(destination().as_path(), &findings, false);
    let quiet: String = render(destination().as_path(), &findings, true);
    assert_eq!(loud.lines().count(), 2, "{loud}");
    assert_eq!(quiet.lines().count(), 1, "{quiet}");
    assert!(quiet.contains("run Toolchain instead"), "{quiet}");
}

/// When some repair failed the success line says it covers part of the state, so it is not read as the whole.
#[test]
fn a_mix_says_the_success_is_partial() {
    let mixed: String = render(destination().as_path(), &[worked(Tool::Cargo, 10), failed(Tool::Toolchain)], false);
    let whole: String = render(destination().as_path(), &[worked(Tool::Cargo, 10)], false);
    let success_of = |text: &str| -> String { return text.lines().find(|line: &&str| { return line.contains("Cargo detail"); }).unwrap_or_default().to_owned(); };
    assert!(success_of(mixed.as_str()).contains("part of"), "{mixed}");
    assert!(!success_of(whole.as_str()).contains("part of"), "{whole}");
}

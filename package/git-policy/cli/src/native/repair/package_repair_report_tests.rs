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

/// A tool stopped at its ceiling is a finding that names the tool, what it was doing, how long it ran and the command that runs
/// it again; every tool has such a sentence.
#[test]
fn a_tool_stopped_at_its_ceiling_is_named_with_what_it_did_and_the_command_to_run() {
    let ceiling: std::time::Duration = crate::wait_ceilings::REPAIR_TOOL_CEILING;
    for (tool, name) in [(Tool::Pnpm, "pnpm"), (Tool::Cargo, "cargo"), (Tool::Toolchain, "mpm-shadow")] {
        let run: ToolRun = ToolRun { end: ToolEnd::CeilingReached(ceiling), millis: ceiling.as_millis(), stdout: Vec::new(), notes: Vec::new() };
        let finding: Finding = finding_from(tool, &run, "detail", "the command to run");
        let Outcome::Failed { reason, run_instead } = &finding.outcome else { panic!("{finding:?}") };
        assert!(reason.contains(name) && reason.contains(ceiling.as_secs().to_string().as_str()) && reason.contains("whole process group"), "{reason}");
        assert_eq!(run_instead, "the command to run");
        let line: String = render(destination().as_path(), &[finding], true);
        assert!(line.contains(name) && line.contains("run the command to run"), "{line}");
    }
}

/// The ceiling is past the quiet window, or the window would never end anything.
#[test]
fn the_shipped_ceiling_is_past_the_shipped_quiet_window() {
    let environment: crate::package_repair::RepairEnvironment = crate::package_repair::RepairEnvironment::from_process();
    assert!(environment.ceiling > environment.quiet_window);
    assert_eq!(environment.ceiling, crate::wait_ceilings::REPAIR_TOOL_CEILING);
}

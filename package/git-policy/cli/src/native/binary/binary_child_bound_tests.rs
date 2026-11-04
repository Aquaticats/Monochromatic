//! What:
//!  A Git step of a commit that runs past its bound is stopped, the commit says which step, its class and the bound,
//!  and the repository recovers.
//! Why:
//!  The bound used to be 30 minutes for every step, so a stalled step held a commit for as long as a stalled repack.
//!  Each control starts a real wrapped commit under `strace` with a delay on the return of every `execve`
//!  (the start of the wrapper's own Git children), and a bound shorter than that delay, so the first step the wrapper
//!  watches outlasts it; the invariants are those of the fault tables (`binary_fault_support.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await injectAndExpectRecovery({ call: 'execve', delayMs: boundMs + margin, env: { CLI_GIT_CHILD_WAIT_SECONDS: bound } });
//! ```

use super::commit::remove_settled;
use super::fault::{Effect, Injection, Ran, When, run_injected_with_environment, staged_repository, verify_recovery};
use super::strace::strace_available;
use super::support::{Fixture, fixture};
use git_policy_cli::child_class::{CHILD_WAIT_VARIABLE, ChildClass, class_of};
use std::ffi::OsString;
use std::path::PathBuf;

/// The bound the control sets, in seconds: the smallest the variable accepts.
const BOUND_SECONDS: u64 = 1;

/// How long the start of each Git child is delayed beyond the bound, in seconds.
const MARGIN_SECONDS: u64 = 1;

/// The commit with the bound set and the notice interval at its smallest, so a notice precedes the stop.
fn bounded_commit(fixture: &Fixture, repo: &std::path::Path, label: &str) -> Ran {
    let fault: Injection = Injection {
        syscall: "execve",
        effect: Effect::SlowReturn((BOUND_SECONDS + MARGIN_SECONDS) * 1_000_000),
        when: When::FromNth(1),
        follow_forks: true,
    };
    let bound: String = BOUND_SECONDS.to_string();
    // The notice interval equals the bound, so the grace, which never passes the interval, is at most the bound.
    return run_injected_with_environment(
        fixture,
        repo,
        label,
        &fault,
        &[(CHILD_WAIT_VARIABLE, bound.as_str()), ("CLI_GIT_WAIT_NOTICE_SECONDS", bound.as_str())],
    );
}

/// A watched step that outlasts the bound is stopped; the commit fails naming the bound, the class and the variable,
///  and the next command recovers.
#[test]
fn a_git_step_past_its_bound_fails_the_commit_and_names_the_class() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no start was delayed");
        return;
    }
    let fixture: Fixture = fixture("child-bound");
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let ran: Ran = bounded_commit(&fixture, repo.as_path(), "child-bound");
    let text: String = ran.text();
    assert!(ran.injected(), "no start was delayed, so nothing was tested: {text}");
    assert_ne!(ran.code(), 0, "a stopped step must fail the commit: {text}");
    assert!(text.contains(format!("showed no sign of work for {BOUND_SECONDS} seconds and was stopped").as_str()), "{text}");
    assert!(text.contains(CHILD_WAIT_VARIABLE), "{text}");
    // The sentence is "<command>, <class label> showed no sign of work for ...": the label must be the one the production
    // classification gives that command.
    let (before, found): (&str, ChildClass) = [ChildClass::Plumbing, ChildClass::Object]
        .iter()
        .filter_map(|class| {
            return text
                .find(format!(", {} showed no sign of work for", class.label()).as_str())
                .map(|at| { return (at, *class); });
        })
        .min_by_key(|(at, _)| { return *at; })
        .map(|(at, class)| { return (&text[..at], class); })
        .expect("the sentence names a class");
    // The command starts at the last "git " that follows a space, a colon or a quote (an earlier path may hold one).
    let start: usize = before
        .match_indices("git ")
        .map(|(at, _)| { return at; })
        .filter(|at| { return *at == 0 || before[..*at].ends_with(|previous: char| { return " :\"(".contains(previous); }); })
        .last()
        .expect("the stopped command is named");
    let command: &str = &before[start..];
    let words: Vec<OsString> = command.split_whitespace().skip(1).map(OsString::from).collect();
    assert_eq!(found, class_of(words.as_slice()), "the class of `{command}`: {text}");
    assert_eq!(verify_recovery(&fixture, repo.as_path(), "child-bound", text.as_str()), Ok(()));
    remove_settled(&fixture);
}

//! What:
//!  Controls for the executable's linter:
//!  the linked linter's contract on a real repository tree,
//!  and every failure of the guard around it,
//! planted through the fix function.
//! Why:
//!  The real linter proves what the policy relies on (a rewrite,
//! a kept finding,
//! an exclude,
//! odd names);
//!  the planted functions reach each failure quickly:
//!  a panic,
//! a stall with no sign of work,
//! a run that ends with a status the configuration never produces.
//!  The planted stall is the guard's required control:
//!  against a guard that runs the fix on the caller's thread it never returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = embeddedLinterWith(plantedFix, window).lint({ topLevel, options, path, source });
//! ```

/// The linter under test and its private parts.
use super::{
    ABANDONED_REASON, EmbeddedLinter, LINT_QUIET_WINDOW, embedded_linter, embedded_linter_with, guard_failure_reason,
};
use crate::config_schema::{MarkdownAutofixOptions, MarkdownRule, PolicyConfig};
use crate::markdown_linter_guard::GuardFailure;
use crate::markdown_linter_output::{EXPLANATION_LIMIT, LintRun, RemainingFinding};
use crate::policy_markdown::{LintRequest, MarkdownLinter};
use crate::test_support::{fixture, remove, wait_until};
use monochromatic_lint::run_embedded::{EmbeddedFailure, EmbeddedRequest, EmbeddedRun};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// The quiet window of the one control whose planted fix never returns,
/// so the abandonment shows quickly.
/// Why:
///   Only a stalled fix waits out its window.
/// Every control whose fix returns or panics at once uses the production
///  `LINT_QUIET_WINDOW`,
/// which only judges a hang:
/// under a loaded host a planted panic came back later than this window
///  and read as a timeout instead of the panic,
/// in the container gate of `692dcb9f48` (2026-11-03T00:08:43-05:00).
const SHORT_WINDOW: Duration = Duration::from_millis(100);

/// The default options:
///  the LFS rule and no exclusion.
fn default_options() -> MarkdownAutofixOptions {
    return PolicyConfig::defaults().markdown_autofix;
}

/// Lint `source` as `path` from `top_level` with `linter` and the default options.
fn lint(
    linter: &mut EmbeddedLinter,
    top_level: &Path,
    path: &[u8],
    source: &[u8],
) -> Result<LintRun, String> {
    let options: MarkdownAutofixOptions = default_options();
    return linter.lint(&LintRequest {
        top_level,
        options: &options,
        path,
        source,
    });
}

/// The bound of the executable's linter is the measured one the former child process had.
#[test]
fn the_executable_linter_uses_its_labelled_unmeasured_window() {
    assert_eq!(embedded_linter().window, LINT_QUIET_WINDOW);
}

/// How many times the planted stall started,
/// to show that no second one starts beside an abandoned one.
static STALL_STARTS: AtomicUsize = AtomicUsize::new(0);

/// A fix function that never returns,
/// as an infinite loop would not.
fn stalls(_request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    STALL_STARTS.fetch_add(1, Ordering::SeqCst);
    loop {
        std::thread::park();
    }
}

/// A fix function that panics,
/// as a rule with a defect would.
fn panics(_request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    panic!("planted panic in the linter");
}

/// The status a planted run ends with:
/// one the one-rule configuration never produces.
const NEVER_PRODUCED_STATUS: u8 = 1;

/// A fix function whose run ends with a status the configuration never produces.
fn ends_with_error_status(_request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    return Err(EmbeddedFailure {
        exit_status: NEVER_PRODUCED_STATUS,
        explanation: String::from("planted"),
    });
}

/// A fix function that hands back the request's source unchanged and the name it was asked about as its fix.
fn echoes(request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    return Ok(EmbeddedRun {
        fixed: String::from_utf8_lossy(request.source).into_owned(),
        findings: Vec::new(),
    });
}

/// A stalled fix is abandoned when it shows no sign of work,
/// with a reason that names the limit and the ways forward;
///  the commit goes on to report the candidate unchecked instead of hanging.
#[test]
fn a_stalled_linter_is_abandoned_when_it_shows_no_sign_of_work() {
    let root: PathBuf = fixture("linter-stall");
    let mut linter: EmbeddedLinter = embedded_linter_with(stalls, SHORT_WINDOW);
    let reason: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("abandoned");
    assert_eq!(reason, guard_failure_reason(&GuardFailure::Quiet(SHORT_WINDOW)));
    assert!(reason.contains(SHORT_WINDOW.as_millis().to_string().as_str()), "{reason}");
    assert!(reason.contains("markdown/autofix"), "{reason}");
    // The stalled thread is still running: the next candidate is refused without starting another.
    let starts: usize = STALL_STARTS.load(Ordering::SeqCst);
    let later: String = lint(&mut linter, root.as_path(), b"b.md", b"b\n").expect_err("refused");
    assert_eq!(later, ABANDONED_REASON);
    assert_eq!(STALL_STARTS.load(Ordering::SeqCst), starts, "no second thread started");
    remove(root.as_path());
}

/// A panic message far longer than the explanation bound is cut at that bound in the reason.
fn panics_with_a_long_message(_request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    panic!("{}", "p".repeat(EXPLANATION_LIMIT * 3));
}

/// The outer panic's text goes through the same bound as the linter's own explanations.
#[test]
fn a_long_panic_message_is_bounded() {
    let root: PathBuf = fixture("linter-long-panic");
    let mut linter: EmbeddedLinter = embedded_linter_with(panics_with_a_long_message, LINT_QUIET_WINDOW);
    let reason: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("reported");
    assert!(reason.contains(format!("{}...", "p".repeat(EXPLANATION_LIMIT)).as_str()), "{reason}");
    assert!(!reason.contains("p".repeat(EXPLANATION_LIMIT + 1).as_str()), "the payload is cut");
    remove(root.as_path());
}

/// A panic in the linter comes back as a reason and the next candidate still lints.
#[test]
fn a_panicking_linter_is_reported_and_the_wrapper_goes_on() {
    let root: PathBuf = fixture("linter-panic");
    let mut linter: EmbeddedLinter = embedded_linter_with(panics, LINT_QUIET_WINDOW);
    let reason: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("reported");
    assert!(reason.contains("planted panic in the linter"), "{reason}");
    assert!(reason.contains("internal error"), "{reason}");
    let mut healthy: EmbeddedLinter = embedded_linter_with(echoes, LINT_QUIET_WINDOW);
    assert_eq!(
        lint(&mut healthy, root.as_path(), b"a.md", b"a\n"),
        Ok(LintRun {
            fixed: b"a\n".to_vec(),
            remaining: Vec::new(),
        })
    );
    remove(root.as_path());
}

/// A run with a status the configuration never produces is refused with that status.
#[test]
fn an_unexpected_status_is_refused() {
    let root: PathBuf = fixture("linter-status");
    let mut linter: EmbeddedLinter = embedded_linter_with(ends_with_error_status, LINT_QUIET_WINDOW);
    let reason: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("refused");
    assert!(reason.contains(format!("status {NEVER_PRODUCED_STATUS}").as_str()), "{reason}");
    remove(root.as_path());
}

/// Each cause of a guard failure reads as a sentence that says what happened.
#[test]
fn guard_failures_read_as_sentences() {
    let thread: String = guard_failure_reason(&GuardFailure::Start(String::from("no threads")));
    assert!(thread.contains("no threads"), "{thread}");
    let vanished: String = guard_failure_reason(&GuardFailure::Vanished);
    assert!(vanished.contains("without a result"), "{vanished}");
}

/// The candidate name reaches the linter as the exact bytes Git gave,
/// including a dash,
/// a space and a byte that is not UTF-8.
#[test]
fn a_candidate_name_is_the_exact_bytes_the_linter_is_given() {
    let root: PathBuf = fixture("linter-name");
    // The planted function compares the bytes it is given with the bytes of the original name, so a lossy
    // conversion anywhere between Git's bytes and the linter fails here.
    fn checks_the_name(request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
        if request.name.as_os_str().as_bytes() != EXACT_NAME {
            return Err(EmbeddedFailure {
                exit_status: NEVER_PRODUCED_STATUS + 1,
                explanation: String::from("the name was converted"),
            });
        }
        return Ok(EmbeddedRun {
            fixed: String::from("ok"),
            findings: Vec::new(),
        });
    }
    let mut linter: EmbeddedLinter = embedded_linter_with(checks_the_name, LINT_QUIET_WINDOW);
    let run: LintRun = lint(&mut linter, root.as_path(), EXACT_NAME, b"a\n").expect("the name arrived unchanged");
    assert_eq!(run.fixed, b"ok".to_vec());
    remove(root.as_path());
}

/// A name with a dash,
/// a space and a byte that is not UTF-8.
const EXACT_NAME: &[u8] = b"-x dir/caf\xe9 =.md";

/// A repository directory with an LFS endpoint,
///  the PNG rule and one tracked image.
fn lfs_tree(root: &Path) -> PathBuf {
    let top: PathBuf = root.join("top");
    std::fs::create_dir_all(top.join("pkg/asset")).expect("tree");
    std::fs::write(
        top.join(".lfsconfig"),
        b"[lfs]\n\turl = https://lfs.example.com/org/repo\n",
    )
    .expect("lfsconfig");
    std::fs::write(
        top.join(".gitattributes"),
        b"*.png filter=lfs diff=lfs merge=lfs -text\n",
    )
    .expect("attributes");
    std::fs::write(top.join("pkg/asset/shot.png"), b"image bytes").expect("image");
    return top;
}

/// The object URL of `pkg/asset/shot.png`;
///  the digest is `sha256sum` of `image bytes`.
const SHOT_URL: &str = "https://lfs.example.com/org/repo/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png";

/// The linked linter rewrites a relative image link,
///  keeps an exact one,
///  reports a stale object URL as a remaining finding,
///  honors the exclude option,
///  and lints a name that is not UTF-8 and one that starts with `-`.
#[test]
fn the_linked_linter_meets_the_contract() {
    let root: PathBuf = fixture("linter-real");
    let top: PathBuf = lfs_tree(root.as_path());
    let mut linter: EmbeddedLinter = embedded_linter();
    let rewritten: String = format!("# T\n\n![shot]({SHOT_URL})\n");
    for name in [&b"pkg/README.md"[..], b"pkg/caf\xe9.md", b"pkg/-x.mdx"] {
        assert_eq!(
            lint(&mut linter, top.as_path(), name, b"# T\n\n![shot](asset/shot.png)\n"),
            Ok(LintRun {
                fixed: rewritten.clone().into_bytes(),
                remaining: Vec::new(),
            }),
            "{:?}",
            std::ffi::OsStr::from_bytes(name)
        );
    }
    assert_eq!(
        lint(&mut linter, top.as_path(), b"pkg/README.md", rewritten.as_bytes()),
        Ok(LintRun {
            fixed: rewritten.clone().into_bytes(),
            remaining: Vec::new(),
        })
    );
    let stale: String = format!(
        "![shot](https://lfs.example.com/org/repo/{}/pkg/asset/gone.png)\n",
        "b".repeat(64)
    );
    assert_eq!(
        lint(&mut linter, top.as_path(), b"pkg/README.md", stale.as_bytes()),
        Ok(LintRun {
            fixed: stale.clone().into_bytes(),
            remaining: vec![RemainingFinding {
                rule: MarkdownRule::LfsImageUrl,
                line: 1,
                column: 1,
                message: String::from(
                    "Object URL names pkg/asset/gone.png, which no longer exists in the repository."
                ),
            }],
        })
    );
    let mut options: MarkdownAutofixOptions = default_options();
    options.exclude = vec![String::from("pkg/")];
    let unchanged: &[u8] = b"![shot](asset/shot.png)\n";
    assert_eq!(
        linter.lint(&LintRequest {
            top_level: top.as_path(),
            options: &options,
            path: b"pkg/README.md",
            source: unchanged,
        }),
        Ok(LintRun {
            fixed: unchanged.to_vec(),
            remaining: Vec::new(),
        })
    );
    remove(root.as_path());
}

/// The quiet window of the control below:
/// longer than the sampling interval of the progress meter.
const WORKING_WINDOW: Duration = Duration::from_millis(1200);

/// How many times the working fix writes a file,
/// a pause after each:
/// a multiple of the window in all.
const WORKING_ROUNDS: u32 = 15;

/// The file the working fix writes after its last round.
/// Why:
///  A control that abandons the fix leaves it running on its own thread,
/// still writing into the fixture,
/// so the control removes the fixture only once this file says the writes are over;
/// a removal that raced a write failed with "Directory not empty".
const WORKING_FINISHED: &str = "linter-finished";

/// A fix that keeps working (it writes a file a round,
/// with a pause between) and then answers,
/// with no busy loop;
/// its last write is `WORKING_FINISHED`.
fn works_then_answers(request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    let scratch: PathBuf = request.top_level.join("linter-progress");
    for round in 0..WORKING_ROUNDS {
        std::fs::write(&scratch, round.to_string()).expect("the fixture writes its progress");
        std::thread::sleep(Duration::from_millis(200));
    }
    std::fs::write(request.top_level.join(WORKING_FINISHED), b"").expect("the fixture says its writes are over");
    return echoes(request);
}

/// A linter that keeps making progress is never abandoned however long it runs:
/// the window judges quiet time,
/// not wall time.
#[test]
fn a_linter_that_keeps_working_is_never_abandoned_however_long_it_runs() {
    let root: PathBuf = fixture("linter-working");
    let mut linter: EmbeddedLinter = embedded_linter_with(works_then_answers, WORKING_WINDOW);
    let started: std::time::Instant = std::time::Instant::now();
    let run: Result<LintRun, String> = lint(&mut linter, root.as_path(), b"a.md", b"a\n");
    assert!(run.is_ok(), "{run:?}");
    assert!(started.elapsed() > WORKING_WINDOW * 2, "the fixture ran only {:?}", started.elapsed());
    remove(root.as_path());
}

/// A fix that keeps working past the ceiling is abandoned there:
/// the reason names the tool,
/// the file,
/// the ceiling and the ways
/// forward,
/// and the next candidate is refused without a second thread,
/// as after a quiet one.
#[test]
fn a_linter_that_keeps_working_is_abandoned_at_its_ceiling_naming_the_file() {
    let root: PathBuf = fixture("linter-ceiling");
    let ceiling: Duration = Duration::from_secs(2);
    let mut linter: EmbeddedLinter = embedded_linter_with(works_then_answers, WORKING_WINDOW).with_ceiling(ceiling);
    let reason: String = lint(&mut linter, root.as_path(), b"big.md", b"a\n").expect_err("abandoned at the ceiling");
    assert!(reason.contains("monochromatic-lint") && reason.contains("big.md") && reason.contains(format!("{} seconds", ceiling.as_secs()).as_str()), "{reason}");
    assert!(reason.contains("markdown/autofix") && reason.contains("by hand"), "{reason}");
    let later: String = lint(&mut linter, root.as_path(), b"b.md", b"b\n").expect_err("refused");
    assert_eq!(later, ABANDONED_REASON);
    // The abandoned fix is still writing into the fixture on its own thread.
    let finished: PathBuf = root.join(WORKING_FINISHED);
    assert!(wait_until(&|| return finished.exists()), "the abandoned fix ended its writes");
    remove(root.as_path());
}

/// The shipped linter waits longer than its window,
/// or the window would never end anything.
#[test]
fn the_shipped_ceiling_is_past_the_shipped_quiet_window() {
    assert!(crate::wait_ceilings::LINT_CEILING > LINT_QUIET_WINDOW);
}

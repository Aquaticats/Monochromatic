//! What:
//!  Controls for the executable's linter:
//!  the linked linter's contract on a real repository tree,
//!  and every failure of the guard around it, planted through the fix function.
//! Why:
//!  The real linter proves what the policy relies on (a rewrite, a kept finding, an exclude, odd names);
//!  the planted functions reach each failure quickly:
//!  a panic, a stall past the deadline, a run that ends with a status the configuration never produces.
//!  The planted stall is the guard's required control:
//!  against a guard that runs the fix on the caller's thread it never returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = embeddedLinterWith(plantedFix, deadline).lint({ topLevel, options, path, source });
//! ```

/// The linter under test and its private parts.
use super::{EmbeddedLinter, LINT_DEADLINE, embedded_linter, embedded_linter_with, guard_failure_reason};
use crate::config_schema::{MarkdownAutofixOptions, MarkdownRule, PolicyConfig};
use crate::markdown_linter_guard::GuardFailure;
use crate::markdown_linter_output::{LintRun, RemainingFinding};
use crate::policy_markdown::{LintRequest, MarkdownLinter};
use crate::test_support::{fixture, remove};
use monochromatic_lint::run_embedded::{EmbeddedFailure, EmbeddedRequest, EmbeddedRun};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A deadline short enough for a test and long enough for a thread to start and answer.
const SHORT_DEADLINE: Duration = Duration::from_millis(100);

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
fn the_executable_linter_uses_the_measured_deadline() {
    assert_eq!(embedded_linter().deadline, LINT_DEADLINE);
    assert_eq!(LINT_DEADLINE, Duration::from_secs(60));
}

/// A fix function that never returns, as an infinite loop would not.
fn stalls(_request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    loop {
        std::thread::park();
    }
}

/// A fix function that panics, as a rule with a defect would.
fn panics(_request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
    panic!("planted panic in the linter");
}

/// The status a planted run ends with: one the one-rule configuration never produces.
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

/// A stalled fix is abandoned at the deadline with a reason that names the limit and the ways forward;
///  the commit goes on to report the candidate unchecked instead of hanging.
#[test]
fn a_stalled_linter_is_abandoned_at_the_deadline() {
    let root: PathBuf = fixture("linter-stall");
    let mut linter: EmbeddedLinter = embedded_linter_with(stalls, SHORT_DEADLINE);
    let reason: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("abandoned");
    assert_eq!(reason, guard_failure_reason(&GuardFailure::TimedOut(SHORT_DEADLINE)));
    assert!(reason.contains(SHORT_DEADLINE.as_millis().to_string().as_str()), "{reason}");
    assert!(reason.contains("markdown/autofix"), "{reason}");
    remove(root.as_path());
}

/// A panic in the linter comes back as a reason and the next candidate still lints.
#[test]
fn a_panicking_linter_is_reported_and_the_wrapper_goes_on() {
    let root: PathBuf = fixture("linter-panic");
    let mut linter: EmbeddedLinter = embedded_linter_with(panics, SHORT_DEADLINE);
    let reason: String = lint(&mut linter, root.as_path(), b"a.md", b"a\n").expect_err("reported");
    assert!(reason.contains("planted panic in the linter"), "{reason}");
    assert!(reason.contains("internal error"), "{reason}");
    let mut healthy: EmbeddedLinter = embedded_linter_with(echoes, SHORT_DEADLINE);
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
    let mut linter: EmbeddedLinter = embedded_linter_with(ends_with_error_status, SHORT_DEADLINE);
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

/// The candidate name reaches the linter as the exact bytes Git gave, including a dash, a space and a byte that is not UTF-8.
#[test]
fn a_candidate_name_is_the_exact_bytes_the_linter_is_given() {
    let root: PathBuf = fixture("linter-name");
    fn names(request: &EmbeddedRequest<'_>) -> Result<EmbeddedRun, EmbeddedFailure> {
        return Ok(EmbeddedRun {
            fixed: request.name.display().to_string(),
            findings: Vec::new(),
        });
    }
    let mut linter: EmbeddedLinter = embedded_linter_with(names, SHORT_DEADLINE);
    let name: &[u8] = b"-x dir/caf\xe9 =.md";
    let run: LintRun = lint(&mut linter, root.as_path(), name, b"a\n").expect("usable");
    assert_eq!(
        run.fixed,
        std::ffi::OsStr::from_bytes(name).to_string_lossy().into_owned().into_bytes()
    );
    remove(root.as_path());
}

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

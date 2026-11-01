//! What:
//!  Controls for the `markdown/autofix` policy over real candidates,
//!  with a scripted
//!       linter in place of the child process.
//! Why:
//!  Which candidates reach the linter,
//!  in which order,
//!  and what the policy reports and
//!      proposes for each answer are decided here,
//!  independently of the program it runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await checkMarkdown({ content, lifecycle, facts, state: { linter: scripted } });
//! ```

/// The policy under test.
use super::{
    LintRequest, MARKDOWN_AUTOFIX_CODE, MARKDOWN_VIOLATION_CODE, MarkdownLinter, MarkdownState,
    UNCHECKED_REMEDY, check_markdown, is_markdown_path, markdown_state,
};
use crate::candidate_object::CandidateMode;
use crate::candidate_prediction::CandidateRequest;
use crate::candidate_version::CandidateVersion;
use crate::config_schema::MarkdownRule;
use crate::diagnostics::EngineFailureCode;
use crate::event_path::EventPath;
use crate::markdown_linter_output::{LintRun, RemainingFinding};
use crate::policy_content::{ContentState, Correction, LifecycleContent};
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
use crate::policy_test_support::{ScriptedFacts, outside_worktree, scripted_facts};
use crate::policy_trigger::Trigger;
use crate::test_support::{executable, fixture, git, remove, repository};
use std::cell::RefCell;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// How the scripted linter answers a request.
type Answer = fn(&LintRequest<'_>) -> Result<LintRun, String>;

/// One request the scripted linter received.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Linted {
    /// The working directory it was given.
    top_level: PathBuf,
    /// The candidate's pathname.
    path: Vec<u8>,
    /// The candidate's bytes.
    source: Vec<u8>,
}

/// A linter that logs each request and answers by script.
struct ScriptedLinter {
    /// The answer to every request.
    answer: Answer,
    /// Every request,
    ///  shared with the test.
    log: Rc<RefCell<Vec<Linted>>>,
}

/// The scripted linter logs,
///  then answers.
impl MarkdownLinter for ScriptedLinter {
    fn lint(&mut self, request: &LintRequest<'_>) -> Result<LintRun, String> {
        self.log.borrow_mut().push(Linted {
            top_level: request.top_level.to_path_buf(),
            path: request.path.to_vec(),
            source: request.source.to_vec(),
        });
        return (self.answer)(request);
    }
}

/// The source with its one relative image link rewritten.
fn rewritten(source: &[u8]) -> Vec<u8> {
    let text: &str = std::str::from_utf8(source).expect("linted sources are text");
    return text
        .replace("(img.png)", "(https://lfs/img.png)")
        .into_bytes();
}

/// The finding the scripted linter leaves.
fn stale() -> RemainingFinding {
    return RemainingFinding {
        rule: MarkdownRule::LfsImageUrl,
        line: 2,
        column: 3,
        message: String::from("Object URL names gone.png, which no longer exists."),
    };
}

/// Answer:
///  rewrite the image link,
///  leave nothing.
fn rewrite(request: &LintRequest<'_>) -> Result<LintRun, String> {
    return Ok(LintRun {
        fixed: rewritten(request.source),
        remaining: Vec::new(),
    });
}

/// Answer:
///  rewrite the image link and leave one finding.
fn rewrite_and_report(request: &LintRequest<'_>) -> Result<LintRun, String> {
    return Ok(LintRun {
        fixed: rewritten(request.source),
        remaining: vec![stale()],
    });
}

/// Answer:
///  change nothing and leave one finding.
fn report(request: &LintRequest<'_>) -> Result<LintRun, String> {
    return Ok(LintRun {
        fixed: request.source.to_vec(),
        remaining: vec![stale()],
    });
}

/// Answer:
///  the run could not be used.
fn refuse(_request: &LintRequest<'_>) -> Result<LintRun, String> {
    return Err(String::from("monochromatic-lint exited with status 3"));
}

/// A state with the scripted linter,
///  the LFS rule and `exclude`,
///  and the shared log.
fn scripted_state(answer: Answer, exclude: &[&str]) -> (MarkdownState, Rc<RefCell<Vec<Linted>>>) {
    let log: Rc<RefCell<Vec<Linted>>> = Rc::new(RefCell::new(Vec::new()));
    let mut state: MarkdownState = markdown_state(Box::new(ScriptedLinter {
        answer,
        log: Rc::clone(&log),
    }));
    state.options.exclude = exclude.iter().map(ToString::to_string).collect();
    return (state, log);
}

/// The lifecycle content of `git add --all`.
fn add_all() -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Add(vec![OsString::from("--all")]));
}

/// Scripted facts whose candidates come from `repo`,
///  at the scripted top level `/r`.
fn facts_for(repo: &Path) -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.to_path_buf());
    return facts;
}

/// The Latin-1 name `café.md`.
const LATIN_NAME: &[u8] = b"caf\xe9.md";

/// A repository with one candidate of each kind the policy must tell apart.
fn mixed_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("e.md"), b"committed\n").expect("committed");
    git(repo.as_path(), &["add", "e.md"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::remove_file(repo.join("e.md")).expect("delete");
    std::fs::write(repo.join("a.md"), b"# A\n![x](img.png)\n").expect("a");
    std::fs::write(repo.join("b.mdx"), b"plain\n").expect("b");
    std::fs::write(repo.join("c.MD"), b"![x](img.png)\n").expect("upper case");
    std::fs::write(repo.join(".md"), b"![x](img.png)\n").expect("no extension");
    std::fs::write(repo.join("d.txt"), b"![x](img.png)\n").expect("text");
    std::os::unix::fs::symlink("a.md", repo.join("link.md")).expect("link");
    executable(repo.join("run.md").as_path(), b"![x](img.png)\n");
    std::fs::create_dir(repo.join("excluded")).expect("excluded");
    std::fs::write(repo.join("excluded/h.md"), b"![x](img.png)\n").expect("excluded file");
    std::fs::write(repo.join("i.md"), b"caf\xe9\n").expect("not UTF-8");
    std::fs::write(
        repo.join(OsString::from_vec(LATIN_NAME.to_vec())),
        b"![x](img.png)\n",
    )
    .expect("Latin-1 name");
    return repo;
}

/// The autofix finding about `path`,
///  offering a fix or not.
fn autofix(path: &[u8], fix_available: bool) -> PolicyFinding {
    let event: EventPath = EventPath::from_git_bytes(path);
    return PolicyFinding {
        code: MARKDOWN_AUTOFIX_CODE,
        message: format!(
            "monochromatic-lint --fix (markdown/lfs-image-url) rewrites {}.",
            event.text()
        ),
        path: Some(event),
        location: None,
        fix_available,
    };
}

/// The correction of `path` from `before` to its rewritten bytes.
fn correction(path: &[u8], mode: CandidateMode, before: &[u8]) -> Correction {
    return Correction {
        path: path.to_vec(),
        mode,
        before: Rc::from(before),
        after: Rc::from(rewritten(before)),
    };
}

/// Only regular and executable files named `.md` or `.mdx`,
///  not excluded,
///  not deleted and
/// holding UTF-8 reach the linter,
///  in candidate order,
///  from the top level;
///  each rewrite is
/// reported,
///  and corrected only by a direct fix.
#[test]
fn each_eligible_candidate_reaches_the_linter_once_in_order() {
    let root: PathBuf = fixture("markdown-eligible");
    let repo: PathBuf = mixed_repository(root.as_path());
    let (mut state, log) = scripted_state(rewrite, &["excluded/"]);
    let mut facts: ScriptedFacts = facts_for(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let expected: Vec<PolicyFinding> = vec![
        autofix(b"a.md", false),
        autofix(LATIN_NAME, false),
        autofix(b"run.md", false),
    ];
    for trigger in [Trigger::PreForward, Trigger::DirectCheck] {
        assert_eq!(
            check_markdown(&mut content, &add_all(), &mut facts, &mut state, trigger),
            PolicyOutcome::Findings(expected.clone()),
            "{trigger:?}"
        );
    }
    assert_eq!(content.take_proposals(), Vec::<Correction>::new());
    let mut linted: Vec<Vec<u8>> = Vec::new();
    for request in log.borrow().iter() {
        assert_eq!(request.top_level, PathBuf::from("/r"));
        linted.push(request.path.clone());
    }
    let once: Vec<Vec<u8>> = vec![
        b"a.md".to_vec(),
        b"b.mdx".to_vec(),
        LATIN_NAME.to_vec(),
        b"run.md".to_vec(),
    ];
    assert_eq!(linted, [once.clone(), once].concat());
    assert_eq!(log.borrow()[0].source, b"# A\n![x](img.png)\n");
    // A direct fix offers each correction and proposes it with the file's mode.
    let fixed: PolicyOutcome = check_markdown(
        &mut content,
        &add_all(),
        &mut facts,
        &mut state,
        Trigger::DirectFix,
    );
    assert_eq!(
        fixed,
        PolicyOutcome::Findings(vec![
            autofix(b"a.md", true),
            autofix(LATIN_NAME, true),
            autofix(b"run.md", true),
        ])
    );
    assert_eq!(
        content.take_proposals(),
        vec![
            correction(b"a.md", CandidateMode::Regular, b"# A\n![x](img.png)\n"),
            correction(LATIN_NAME, CandidateMode::Regular, b"![x](img.png)\n"),
            correction(b"run.md", CandidateMode::Executable, b"![x](img.png)\n"),
        ]
    );
    remove(root.as_path());
}

/// A repository with `a.md` holding one relative image link.
fn single_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.md"), b"# A\n![x](img.png)\n").expect("a");
    return repo;
}

/// The policy's outcome for `a.md` under `answer`,
///  at `trigger`.
fn single(answer: Answer, trigger: Trigger) -> PolicyOutcome {
    let root: PathBuf = fixture(format!("markdown-single-{trigger:?}").as_str());
    let repo: PathBuf = single_repository(root.as_path());
    let (mut state, _log) = scripted_state(answer, &[]);
    let outcome: PolicyOutcome = check_markdown(
        &mut ContentState::new(),
        &add_all(),
        &mut facts_for(repo.as_path()),
        &mut state,
        trigger,
    );
    remove(root.as_path());
    return outcome;
}

/// Findings the fixes leave follow the rewrite,
///  in the installed wrapper's words;
///  an
/// unchanged source reports only them.
#[test]
fn remaining_findings_are_violations_after_the_rewrite() {
    let violation: PolicyFinding = PolicyFinding {
        code: MARKDOWN_VIOLATION_CODE,
        message: String::from(
            "lfs-image-url at a.md:2:3: Object URL names gone.png, which no longer exists.",
        ),
        path: Some(EventPath::from_git_bytes(b"a.md")),
        location: None,
        fix_available: false,
    };
    assert_eq!(
        single(rewrite_and_report, Trigger::PreForward),
        PolicyOutcome::Findings(vec![autofix(b"a.md", false), violation.clone()])
    );
    assert_eq!(
        single(report, Trigger::DirectCheck),
        PolicyOutcome::Findings(vec![violation])
    );
}

/// A candidate the linter cannot check ends the policy,
///  naming that candidate,
///  and no
/// later candidate is linted.
#[test]
fn a_candidate_the_linter_cannot_check_fails_the_policy() {
    let root: PathBuf = fixture("markdown-refused");
    let repo: PathBuf = single_repository(root.as_path());
    std::fs::write(repo.join("b.md"), b"b\n").expect("b");
    let (mut state, log) = scripted_state(refuse, &[]);
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut facts_for(repo.as_path()),
            &mut state,
            Trigger::PreForward
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::PolicyIncomplete,
            message: format!("monochromatic-lint exited with status 3. {UNCHECKED_REMEDY}"),
            path: Some(EventPath::from_git_bytes(b"a.md")),
        }
    );
    assert_eq!(log.borrow().len(), 1);
    remove(root.as_path());
}

/// Exclude patterns that do not compile end the policy before any candidate is linted.
#[test]
fn exclude_patterns_that_do_not_compile_fail_the_policy() {
    let root: PathBuf = fixture("markdown-bad-exclude");
    let repo: PathBuf = single_repository(root.as_path());
    let (mut state, log) = scripted_state(rewrite, &["a{b"]);
    for _ in 0..2 {
        match check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut facts_for(repo.as_path()),
            &mut state,
            Trigger::PreForward,
        ) {
            PolicyOutcome::Failed {
                code: EngineFailureCode::PolicyIncomplete,
                message,
                path: None,
            } => {
                assert!(
                    message.starts_with(
                        "cli-git could not compile the markdown/autofix exclude pattern \"a{b\": "
                    ) && message.ends_with(UNCHECKED_REMEDY),
                    "{message}"
                );
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(log.borrow().len(), 0);
    remove(root.as_path());
}

/// No candidates,
///  candidates that cannot be prepared,
///  and no selected rule lint nothing.
#[test]
fn nothing_to_lint_lints_nothing() {
    let (mut state, log) = scripted_state(rewrite, &[]);
    let mut idle: ScriptedFacts = scripted_facts();
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &LifecycleContent::None,
            &mut idle,
            &mut state,
            Trigger::DirectFix
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(idle.asked, Vec::<String>::new());
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut scripted_facts(),
            &mut state,
            Trigger::PreForward
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("the scripted facts prepare no candidates"),
            path: None,
        }
    );
    let root: PathBuf = fixture("markdown-no-rule");
    let repo: PathBuf = single_repository(root.as_path());
    state.options.rules = Vec::new();
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut facts_for(repo.as_path()),
            &mut state,
            Trigger::DirectFix
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(log.borrow().len(), 0);
    remove(root.as_path());
}

/// The top level is asked only for a candidate the linter must see;
///  an answer without
/// one is content that could not be read.
#[test]
fn the_top_level_is_asked_only_when_a_candidate_needs_the_linter() {
    let root: PathBuf = fixture("markdown-top-level");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("only.txt"), b"text\n").expect("text");
    let (mut state, _log) = scripted_state(rewrite, &[]);
    let mut text_only: ScriptedFacts = facts_for(repo.as_path());
    text_only.location = Err(String::from("never asked"));
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut text_only,
            &mut state,
            Trigger::PreForward
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(text_only.asked, vec![String::from("candidates")]);
    std::fs::write(repo.join("a.md"), b"a\n").expect("a");
    let mut unlocated: ScriptedFacts = facts_for(repo.as_path());
    unlocated.location = Err(String::from("git could not be asked"));
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut unlocated,
            &mut state,
            Trigger::PreForward
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("git could not be asked"),
            path: None,
        }
    );
    let mut bare: ScriptedFacts = facts_for(repo.as_path());
    bare.location = Ok(outside_worktree());
    assert_eq!(
        check_markdown(
            &mut ContentState::new(),
            &add_all(),
            &mut bare,
            &mut state,
            Trigger::PreForward
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from(
                "cli-git could not find the repository's top level, where monochromatic-lint must run."
            ),
            path: None,
        }
    );
    remove(root.as_path());
}

/// A candidate whose bytes vanished is unreadable content,
///  never a clean file.
#[test]
fn unreadable_candidate_bytes_are_content_unavailable() {
    let root: PathBuf = fixture("markdown-unreadable");
    let repo: PathBuf = single_repository(root.as_path());
    let mut facts: ScriptedFacts = facts_for(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let version: Rc<CandidateVersion> = content
        .version(&add_all(), &mut facts)
        .expect("prepared")
        .expect("candidates");
    let object: String = version.candidates()[0]
        .object
        .clone()
        .expect("object")
        .as_str()
        .to_owned();
    std::fs::remove_file(
        repo.join(".git/objects")
            .join(&object[..2])
            .join(&object[2..]),
    )
    .expect("remove the loose object");
    let (mut state, log) = scripted_state(rewrite, &[]);
    match check_markdown(
        &mut content,
        &add_all(),
        &mut facts,
        &mut state,
        Trigger::PreForward,
    ) {
        PolicyOutcome::Failed { code, .. } => {
            assert_eq!(code, EngineFailureCode::ContentUnavailable)
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(log.borrow().len(), 0);
    remove(root.as_path());
}

/// The extension test is the linter's:
///  exact case,
///  and a name that is only an extension
/// has none.
#[test]
fn markdown_paths_are_the_linter_s() {
    for (path, expected) in [
        (&b"a.md"[..], true),
        (b"dir/b.mdx", true),
        (b"..md", true),
        (LATIN_NAME, true),
        (b"a.MD", false),
        (b"a.Mdx", false),
        (b".md", false),
        (b"dir/.mdx", false),
        (b"a.md.txt", false),
        (b"a.markdown", false),
        (b"md", false),
        (b"", false),
    ] {
        assert_eq!(is_markdown_path(path), expected, "{path:?}");
    }
}

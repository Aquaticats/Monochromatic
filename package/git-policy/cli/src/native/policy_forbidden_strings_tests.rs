//! What:
//!  Controls for the forbidden-strings policy over real candidates and a real rules
//!       file:
//!  where the rules come from,
//!  which candidates are scanned,
//!  how matches are
//!       redacted,
//!  and how a scan that cannot complete fails.
//! Why:
//!  A rules file read from the wrong place,
//!  a candidate skipped,
//!  a matched string in a
//!      message or a failed matcher read as clean would each let a secret through.
//!  Every
//!      control loads rules in its own process with a disposable home and cache.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await check(['--', 'a.txt'])).toEqual([{ code: 'forbidden-string', message: 'Forbidden string matched at line 2 (rule 0).', path: 'a.txt' }]);
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{
    ENGINE_ERROR_MESSAGE, FORBIDDEN_STRING_CODE, LINE_BREAK_MESSAGE, ScannerSettings, ScannerState,
    check_forbidden_strings, default_scanner_settings, incomplete_scan, match_finding,
    unloaded_scanner,
};
use crate::candidate_prediction::CandidateRequest;
use crate::config_schema::RulesFileEntry;
use crate::diagnostics::EngineFailureCode;
/// The event form of a pathname.
use crate::event_path::EventPath;
use crate::policy_content::{ContentState, LifecycleContent};
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
use crate::policy_test_support::{ScriptedFacts, main_worktree, scripted_facts};
use crate::scanner_test_support::{needle, run_isolated};
use crate::test_support::{git, repository};
use forbidden_strings::{CandidateScan, ScanFinding};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Scripted facts at the top level of `repo`,
///  preparing candidates there with real Git.
fn facts_in(repo: &Path) -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(main_worktree(
        repo.to_str().expect("fixture path is UTF-8"),
        "",
    ));
    facts.candidates_repository = Some(repo.to_path_buf());
    return facts;
}

/// The candidates of `git add` with these arguments.
fn add(values: &[&str]) -> LifecycleContent {
    let mut region: Vec<OsString> = Vec::new();
    for value in values {
        region.push(OsString::from(value));
    }
    return LifecycleContent::Requested(CandidateRequest::Add(region));
}

/// Settings with these options and variable.
fn settings(rules_file: Option<&str>, variable: Option<&OsStr>, builtin: bool) -> ScannerSettings {
    let mut chosen: ScannerSettings = default_scanner_settings();
    chosen.options.builtin_rules = builtin;
    chosen.options.rules_files = rules_file.map(|name| {
        return vec![RulesFileEntry { path: String::from(name), optional: false }];
    });
    chosen.rules_variable = variable.map(OsStr::to_os_string);
    return chosen;
}

/// The policy's outcome for one lifecycle in `repo`,
///  with a fresh content and scanner.
fn check(repo: &Path, lifecycle: &LifecycleContent, chosen: &ScannerSettings) -> PolicyOutcome {
    return check_forbidden_strings(
        &mut ContentState::new(),
        lifecycle,
        &mut facts_in(repo),
        chosen,
        &mut unloaded_scanner(),
    );
}

/// A content match at `line` about `path`.
fn content_match(path: &str, line: usize) -> PolicyFinding {
    return PolicyFinding {
        code: FORBIDDEN_STRING_CODE,
        message: format!("Forbidden string matched at line {line} (rule 0)."),
        path: Some(EventPath::display(path)),
        location: None,
        fix_available: false,
    };
}

/// A repository whose `a.txt` holds the needle on line 2 and whose `clean.txt` does not.
fn repository_with_needle(work: &Path) -> PathBuf {
    let repo: PathBuf = repository(work, "repo");
    std::fs::write(repo.join("a.txt"), format!("first\n{}\n", needle())).expect("needle");
    std::fs::write(repo.join("clean.txt"), b"clean\n").expect("clean");
    return repo;
}

/// The rules file is the configured one,
///  else the variable's,
///  else the default in the
/// repository's top level;
///  the rules file itself is never scanned.
#[test]
fn rules_come_from_the_configuration_the_variable_or_the_default_file() {
    run_isolated(
        "policy_forbidden_strings::tests::rules_come_from_the_configuration_the_variable_or_the_default_file",
        "forbidden-rules-sources",
        rules_sources,
    );
}

/// The body of `rules_come_from_the_configuration_the_variable_or_the_default_file`.
fn rules_sources(work: &Path) {
    let repo: PathBuf = repository_with_needle(work);
    let expected: PolicyOutcome = PolicyOutcome::Findings(vec![content_match("a.txt", 2)]);
    let both: LifecycleContent = add(&["a.txt", "clean.txt"]);
    // From the variable, relative to the top level and absolute.
    std::fs::write(repo.join("rules.txt"), format!("{}\n", needle())).expect("rules");
    assert_eq!(
        check(
            repo.as_path(),
            &both,
            &settings(None, Some(OsStr::new("rules.txt")), false)
        ),
        expected
    );
    let outside: PathBuf = work.join("outside-rules.txt");
    std::fs::write(&outside, format!("{}\n", needle())).expect("outside rules");
    assert_eq!(
        check(
            repo.as_path(),
            &both,
            &settings(None, Some(outside.as_os_str()), false)
        ),
        expected
    );
    // The configured file wins over a variable that names a missing file.
    assert_eq!(
        check(
            repo.as_path(),
            &both,
            &settings(Some("rules.txt"), Some(OsStr::new("missing.txt")), false)
        ),
        expected
    );
    // The default file in the top level, when neither is set.
    std::fs::rename(
        repo.join("rules.txt"),
        repo.join("forbidden-strings.local.txt"),
    )
    .expect("default rules");
    assert_eq!(
        check(repo.as_path(), &both, &settings(None, None, false)),
        expected
    );
    // The rules file holds the needle, but is never scanned whichever source named it.
    for (configured, variable) in [
        (None, None),
        (Some("forbidden-strings.local.txt"), None),
        (None, Some(OsStr::new("forbidden-strings.local.txt"))),
        (None, Some(OsStr::new("sub/../forbidden-strings.local.txt"))),
    ] {
        assert_eq!(
            check(
                repo.as_path(),
                &add(&["forbidden-strings.local.txt"]),
                &settings(configured, variable, false)
            ),
            PolicyOutcome::Findings(Vec::new()),
            "{configured:?} {variable:?}"
        );
    }
}

/// A rules file that is named and missing fails the policy as incomplete;
///  a missing
/// default file is tolerated only beside the built-in rules;
///  nothing to scan loads nothing.
#[test]
fn missing_rules_files_fail_only_when_something_is_scanned() {
    run_isolated(
        "policy_forbidden_strings::tests::missing_rules_files_fail_only_when_something_is_scanned",
        "forbidden-rules-missing",
        missing_rules,
    );
}

/// The body of `missing_rules_files_fail_only_when_something_is_scanned`.
fn missing_rules(work: &Path) {
    let repo: PathBuf = repository_with_needle(work);
    let clean: LifecycleContent = add(&["clean.txt"]);
    for chosen in [
        settings(Some("missing.txt"), None, true),
        settings(None, Some(OsStr::new("missing.txt")), true),
        settings(None, None, false),
    ] {
        match check(repo.as_path(), &clean, &chosen) {
            PolicyOutcome::Failed { code, message, .. } => {
                assert_eq!(code, EngineFailureCode::PolicyIncomplete, "{chosen:?}");
                assert!(
                    message.starts_with(
                        "cli-git could not load the forbidden-strings rules, so no file was scanned:"
                    ),
                    "{message}"
                );
            }
            other => panic!("expected a failure for {chosen:?}, got {other:?}"),
        }
    }
    // The built-in rules alone, without the default file: the clean file is clean.
    assert_eq!(
        check(repo.as_path(), &clean, &settings(None, None, true)),
        PolicyOutcome::Findings(Vec::new())
    );
    // Only a deletion: nothing is scanned, so the missing file is never loaded.
    std::fs::write(repo.join("gone.txt"), b"committed\n").expect("committed");
    git(repo.as_path(), &["add", "gone.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=gone"]);
    std::fs::remove_file(repo.join("gone.txt")).expect("delete");
    assert_eq!(
        check(
            repo.as_path(),
            &add(&["gone.txt"]),
            &settings(Some("missing.txt"), None, false)
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    // No candidates at all: nothing is prepared or loaded.
    let mut idle: ScriptedFacts = scripted_facts();
    assert_eq!(
        check_forbidden_strings(
            &mut ContentState::new(),
            &LifecycleContent::None,
            &mut idle,
            &settings(Some("missing.txt"), None, false),
            &mut unloaded_scanner(),
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(idle.asked, Vec::<String>::new());
}

/// Matches name the masked path and an opaque rule,
///  never the matched text;
///  the bytes are
/// what the command would stage;
///  the rules load once per invocation.
#[test]
fn matches_are_redacted_and_read_from_what_would_be_staged() {
    run_isolated(
        "policy_forbidden_strings::tests::matches_are_redacted_and_read_from_what_would_be_staged",
        "forbidden-redacted",
        redacted_matches,
    );
}

/// The body of `matches_are_redacted_and_read_from_what_would_be_staged`.
fn redacted_matches(work: &Path) {
    let repo: PathBuf = repository_with_needle(work);
    let rules: PathBuf = work.join("rules.txt");
    std::fs::write(&rules, format!("{}\n", needle())).expect("rules");
    let chosen: ScannerSettings = settings(None, Some(rules.as_os_str()), false);
    // A pathname component that matches is masked in the reported path.
    let named: String = format!("dir/{}.txt", needle());
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join(&named), b"clean content\n").expect("named file");
    let outcome: PolicyOutcome = check(repo.as_path(), &add(&["a.txt", named.as_str()]), &chosen);
    assert_eq!(
        outcome,
        PolicyOutcome::Findings(vec![
            content_match("a.txt", 2),
            PolicyFinding {
                code: FORBIDDEN_STRING_CODE,
                message: String::from("Forbidden string matched in pathname segment 2 (rule 0)."),
                path: Some(EventPath::from_git_bytes("dir/[REDACTED]".as_bytes())),
                location: None,
                fix_available: false,
            },
        ])
    );
    assert!(!format!("{outcome:?}").contains(needle().as_str()));
    // The staged copy holds the needle and the worktree copy is clean: `git add` would stage clean bytes.
    std::fs::write(repo.join("swap.txt"), format!("{}\n", needle())).expect("dirty");
    git(repo.as_path(), &["add", "swap.txt"]);
    std::fs::write(repo.join("swap.txt"), b"clean now\n").expect("clean");
    assert_eq!(
        check(repo.as_path(), &add(&["swap.txt"]), &chosen),
        PolicyOutcome::Findings(Vec::new())
    );
    // The reverse: staged clean, worktree dirty; the add would stage the needle.
    git(repo.as_path(), &["add", "swap.txt"]);
    std::fs::write(repo.join("swap.txt"), format!("x\n{}\n", needle())).expect("dirty");
    assert_eq!(
        check(repo.as_path(), &add(&["swap.txt"]), &chosen),
        PolicyOutcome::Findings(vec![content_match("swap.txt", 2)])
    );
    // A direct check reads the worktree copy too.
    assert_eq!(
        check(
            repo.as_path(),
            &LifecycleContent::Requested(CandidateRequest::Direct(vec![OsString::from(
                "swap.txt"
            )])),
            &chosen
        ),
        PolicyOutcome::Findings(vec![content_match("swap.txt", 2)])
    );
    // The scanner's own rule sources are not scanned.
    let source: &str = "package/forbidden-strings/cli/data/builtin-rules.txt";
    std::fs::create_dir_all(repo.join("package/forbidden-strings/cli/data")).expect("source dir");
    std::fs::write(repo.join(source), format!("{}\n", needle())).expect("rule source");
    assert_eq!(
        check(repo.as_path(), &add(&[source]), &chosen),
        PolicyOutcome::Findings(Vec::new())
    );
    // Loaded once: after the first scan the rules file is gone and a second scan still matches.
    let mut content: ContentState = ContentState::new();
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut scanner: ScannerState = unloaded_scanner();
    let lifecycle: LifecycleContent = add(&["a.txt"]);
    let first: PolicyOutcome =
        check_forbidden_strings(&mut content, &lifecycle, &mut facts, &chosen, &mut scanner);
    std::fs::remove_file(&rules).expect("remove rules");
    let second: PolicyOutcome =
        check_forbidden_strings(&mut content, &lifecycle, &mut facts, &chosen, &mut scanner);
    assert_eq!(first, second);
    assert_eq!(
        second,
        PolicyOutcome::Findings(vec![content_match("a.txt", 2)])
    );
    // A load failure is remembered too: a second check reports it without loading again.
    let mut failing: ScannerState = unloaded_scanner();
    let missing: ScannerSettings = settings(Some("missing.txt"), None, false);
    let mut failing_facts: ScriptedFacts = facts_in(repo.as_path());
    let mut failing_content: ContentState = ContentState::new();
    let one: PolicyOutcome = check_forbidden_strings(
        &mut failing_content,
        &lifecycle,
        &mut failing_facts,
        &missing,
        &mut failing,
    );
    std::fs::write(repo.join("missing.txt"), format!("{}\n", needle())).expect("now present");
    let two: PolicyOutcome = check_forbidden_strings(
        &mut failing_content,
        &lifecycle,
        &mut failing_facts,
        &missing,
        &mut failing,
    );
    assert_eq!(one, two);
    assert!(matches!(one, PolicyOutcome::Failed { .. }), "{one:?}");
}

/// A pathname with a line break cannot be inspected:
///  the policy fails as incomplete and
/// names no pathname.
#[test]
fn an_uninspectable_pathname_fails_the_policy() {
    run_isolated(
        "policy_forbidden_strings::tests::an_uninspectable_pathname_fails_the_policy",
        "forbidden-line-break",
        line_break,
    );
}

/// The body of `an_uninspectable_pathname_fails_the_policy`.
fn line_break(work: &Path) {
    let repo: PathBuf = repository_with_needle(work);
    let rules: PathBuf = work.join("rules.txt");
    std::fs::write(&rules, format!("{}\n", needle())).expect("rules");
    let name: &[u8] = b"line\nbreak.txt";
    std::fs::write(repo.join(OsStr::from_bytes(name)), b"clean\n").expect("odd name");
    let outcome: PolicyOutcome = check(
        repo.as_path(),
        &add(&["--all"]),
        &settings(None, Some(rules.as_os_str()), false),
    );
    assert_eq!(
        outcome,
        PolicyOutcome::Failed {
            code: EngineFailureCode::PolicyIncomplete,
            message: String::from(LINE_BREAK_MESSAGE),
            path: None,
        }
    );
}

/// Failure findings are found before any match is reported,
///  whatever their order;
///  matches
/// alone are reported in order.
#[test]
fn failure_findings_take_precedence_over_matches() {
    let matched: CandidateScan = CandidateScan {
        identity: 0,
        display_path: String::from("a.txt"),
        findings: vec![ScanFinding::Content {
            line: 3,
            rule: String::from("named-rule"),
        }],
        scanned_bytes: 10,
    };
    assert_eq!(incomplete_scan(&matched), None);
    for (finding, message) in [
        (ScanFinding::EngineError, ENGINE_ERROR_MESSAGE),
        (ScanFinding::PathnameLineBreak, LINE_BREAK_MESSAGE),
    ] {
        let failing: CandidateScan = CandidateScan {
            identity: 1,
            display_path: String::from("[REDACTED]"),
            findings: vec![
                ScanFinding::Name {
                    component: 1,
                    rule: String::from("0"),
                },
                finding.clone(),
            ],
            scanned_bytes: 0,
        };
        assert_eq!(
            incomplete_scan(&failing),
            Some(PolicyOutcome::Failed {
                code: EngineFailureCode::PolicyIncomplete,
                message: String::from(message),
                path: None,
            })
        );
        assert_eq!(match_finding(&finding, "x"), None);
    }
    assert_eq!(
        match_finding(&matched.findings[0], "a.txt"),
        Some(PolicyFinding {
            code: FORBIDDEN_STRING_CODE,
            message: String::from("Forbidden string matched at line 3 (rule named-rule)."),
            path: Some(EventPath::from_git_bytes("a.txt".as_bytes())),
            location: None,
            fix_available: false,
        })
    );
}

/// A repository location that cannot be read fails the policy as unavailable content.
#[test]
fn an_unknown_top_level_is_unavailable_content() {
    run_isolated(
        "policy_forbidden_strings::tests::an_unknown_top_level_is_unavailable_content",
        "forbidden-no-root",
        unknown_root,
    );
}

/// The body of `an_unknown_top_level_is_unavailable_content`.
fn unknown_root(work: &Path) {
    let repo: PathBuf = repository_with_needle(work);
    let mut unanswered: ScriptedFacts = facts_in(repo.as_path());
    unanswered.location = Err(String::from("no location answer"));
    assert_eq!(
        check_forbidden_strings(
            &mut ContentState::new(),
            &add(&["a.txt"]),
            &mut unanswered,
            &settings(None, None, true),
            &mut unloaded_scanner(),
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("no location answer"),
            path: None,
        }
    );
    let mut outside: ScriptedFacts = facts_in(repo.as_path());
    outside.location = Ok(crate::policy_test_support::outside_worktree());
    match check_forbidden_strings(
        &mut ContentState::new(),
        &add(&["a.txt"]),
        &mut outside,
        &settings(None, None, true),
        &mut unloaded_scanner(),
    ) {
        PolicyOutcome::Failed { code, .. } => {
            assert_eq!(code, EngineFailureCode::ContentUnavailable);
        }
        other => panic!("expected a failure, got {other:?}"),
    }
}

/// Content already in a parent commit's tree is not scanned again.
#[path = "policy_forbidden_strings_known_content_tests.rs"]
mod known_content;

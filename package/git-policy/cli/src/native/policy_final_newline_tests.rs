//! What:
//!  Controls for the final-newline rule,
//!  its preserved paths and its check over real
//!       candidates.
//! Why:
//!  The rule decides which bytes a fix writes;
//!  a wrong normalization would rewrite
//!      binary or fixture files,
//!  and a missed candidate kind would let a file through.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(normalizeFinalNewline(bytes('a'))).toEqual({ kind: 'changed', bytes: bytes('a\n') });
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{
    FINAL_NEWLINE_CODE, FINAL_NEWLINE_MESSAGE, check_final_newline, is_final_newline_excluded,
    normalized_final_newline,
};
use crate::candidate_object::CandidateMode;
use crate::candidate_prediction::CandidateRequest;
use crate::diagnostics::EngineFailureCode;
use crate::event_path::EventPath;
use crate::policy_content::Correction;
use crate::policy_content::{ContentState, LifecycleContent};
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::policy_trigger::Trigger;
use crate::test_support::{fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;

/// Missing and extra final LF bytes are corrected;
///  empty,
///  NUL-holding,
///  non-UTF-8 and
/// canonical bytes are left alone;
///  interior bytes are never touched.
#[test]
fn the_rule_requires_exactly_one_final_line_feed() {
    let unchanged: [&[u8]; 7] = [
        b"",
        b"a\n",
        b"\n",
        b"nul\0inside",
        b"\xff not utf-8",
        b"crlf\r\n",
        b"crlf twice\r\n\r\n",
    ];
    for bytes in unchanged {
        assert_eq!(normalized_final_newline(bytes), None, "{bytes:?}");
    }
    let changed: [(&[u8], &[u8]); 7] = [
        (b"a", b"a\n"),
        (b"a\n\n\n", b"a\n"),
        (b"\n\n", b"\n"),
        (b"a\nb\n\n", b"a\nb\n"),
        (b"carriage\n\r", b"carriage\n\r\n"),
        (
            "multibyte \u{e9}".as_bytes(),
            "multibyte \u{e9}\n".as_bytes(),
        ),
        (b"x", b"x\n"),
    ];
    for (bytes, expected) in changed {
        assert_eq!(
            normalized_final_newline(bytes),
            Some(expected.to_vec()),
            "{bytes:?}"
        );
    }
}

/// The five preserved path families,
///  each with its depth condition and a near miss.
#[test]
fn preserved_paths_keep_their_bytes() {
    for path in [
        "package/forbidden-strings/cli.fuzz/seed/case",
        "package/forbidden-strings/core.fuzz/seed/case",
        "package/test-fixture/toml-edit/src/case.toml",
        "dist/final/node/index.mjs",
        "pkg/dist/final/node/index.mjs",
        "pkg/dist/final/node/deeper/chunk.mjs",
        "bundle/node/index.mjs",
        "pkg/bundle/node/index.mjs",
    ] {
        assert!(is_final_newline_excluded(path.as_bytes()), "{path}");
    }
    for path in [
        "package/forbidden-strings/cli.fuzz/seedling",
        "x/package/forbidden-strings/cli.fuzz/seed/case",
        "package/test-fixture/toml-edit/srcs/case",
        "dist/final/node",
        "pkg/dist/final/node",
        "dist/final/nodes/index.mjs",
        "dist/finals/node/index.mjs",
        "dists/final/node/index.mjs",
        "bundle/node",
        "pkg/bundle/nodes/index.mjs",
        "bundles/node/index.mjs",
        "plain.txt",
    ] {
        assert!(!is_final_newline_excluded(path.as_bytes()), "{path}");
    }
}

/// The finding about the file at `path`,
///  offering a fix or not.
fn finding_with(path: &str, fix_available: bool) -> PolicyFinding {
    return PolicyFinding {
        code: FINAL_NEWLINE_CODE,
        message: String::from(FINAL_NEWLINE_MESSAGE),
        path: Some(EventPath::from_git_bytes(path.as_bytes())),
        location: None,
        fix_available,
    };
}

/// The finding about the file at `path` of a lifecycle that does not correct.
fn finding(path: &str) -> PolicyFinding {
    return finding_with(path, false);
}

/// The correction of the file at `path` from `before` to `after`.
fn correction(path: &str, mode: CandidateMode, before: &[u8], after: &[u8]) -> Correction {
    return Correction {
        path: path.as_bytes().to_vec(),
        mode,
        before: Rc::from(before),
        after: Rc::from(after),
    };
}

/// The check reports every regular or executable candidate whose bytes are not canonical,
/// in candidate order,
///  and nothing else.
#[test]
fn the_check_reports_each_noncanonical_text_file() {
    let root: PathBuf = fixture("final-newline-check");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("deleted.txt"), b"committed\n").expect("committed");
    git(repo.as_path(), &["add", "deleted.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::remove_file(repo.join("deleted.txt")).expect("delete");
    std::fs::write(repo.join("b-missing.txt"), b"missing").expect("missing");
    std::fs::write(repo.join("a-extra.txt"), b"extra\n\n").expect("extra");
    std::fs::write(repo.join("canonical.txt"), b"fine\n").expect("canonical");
    std::fs::write(repo.join("empty.txt"), b"").expect("empty");
    std::fs::write(repo.join("binary.bin"), b"\0binary").expect("binary");
    std::os::unix::fs::symlink("no-newline-target", repo.join("link")).expect("link");
    std::fs::create_dir_all(repo.join("pkg/dist/final/node")).expect("bundle directory");
    std::fs::write(repo.join("pkg/dist/final/node/index.mjs"), b"bundle").expect("bundle");
    let script: PathBuf = repo.join("script.sh");
    crate::test_support::executable(script.as_path(), b"#!/bin/sh");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.clone());
    let lifecycle: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Add(vec![OsString::from("--all")]));
    let mut content: ContentState = ContentState::new();
    assert_eq!(
        check_final_newline(&mut content, &lifecycle, &mut facts, Trigger::PreForward, false),
        PolicyOutcome::Findings(vec![
            finding("a-extra.txt"),
            finding("b-missing.txt"),
            finding("script.sh"),
        ])
    );
    // A check proposes nothing; only a direct fix offers and proposes each correction.
    assert_eq!(
        check_final_newline(&mut content, &lifecycle, &mut facts, Trigger::DirectCheck, false),
        PolicyOutcome::Findings(vec![
            finding("a-extra.txt"),
            finding("b-missing.txt"),
            finding("script.sh"),
        ])
    );
    assert_eq!(content.take_proposals(), Vec::<Correction>::new());
    assert_eq!(
        check_final_newline(&mut content, &lifecycle, &mut facts, Trigger::DirectFix, false),
        PolicyOutcome::Findings(vec![
            finding_with("a-extra.txt", true),
            finding_with("b-missing.txt", true),
            finding_with("script.sh", true),
        ])
    );
    assert_eq!(
        content.take_proposals(),
        vec![
            correction(
                "a-extra.txt",
                CandidateMode::Regular,
                b"extra\n\n",
                b"extra\n"
            ),
            correction(
                "b-missing.txt",
                CandidateMode::Regular,
                b"missing",
                b"missing\n"
            ),
            correction(
                "script.sh",
                CandidateMode::Executable,
                b"#!/bin/sh",
                b"#!/bin/sh\n"
            ),
        ]
    );
    // Without candidates there is nothing to report and nothing is prepared.
    let mut idle: ScriptedFacts = scripted_facts();
    assert_eq!(
        check_final_newline(
            &mut ContentState::new(),
            &LifecycleContent::None,
            &mut idle,
            Trigger::DirectFix,
            false
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(idle.asked, Vec::<String>::new());
    // Candidates that cannot be prepared fail the policy.
    assert_eq!(
        check_final_newline(
            &mut ContentState::new(),
            &lifecycle,
            &mut scripted_facts(),
            Trigger::DirectCheck,
            false
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("the scripted facts prepare no candidates"),
            path: None,
        }
    );
    // A candidate whose bytes vanished fails the policy instead of being skipped.
    let mut unreadable_facts: ScriptedFacts = scripted_facts();
    unreadable_facts.candidates_repository = Some(repo.clone());
    let only: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Add(vec![OsString::from("b-missing.txt")]));
    let mut unreadable: ContentState = ContentState::new();
    let listed: std::rc::Rc<crate::candidate_version::CandidateVersion> = unreadable
        .version(&only, &mut unreadable_facts)
        .expect("listed")
        .expect("candidates");
    let object: String = listed.candidates()[0]
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
    match check_final_newline(
        &mut unreadable,
        &only,
        &mut unreadable_facts,
        Trigger::PreForward,
        false,
    ) {
        PolicyOutcome::Failed { code, .. } => {
            assert_eq!(code, EngineFailureCode::ContentUnavailable);
        }
        other => panic!("expected a failure, got {other:?}"),
    }
    remove(root.as_path());
}

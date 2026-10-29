//! What:
//!  `git push` region facts in every spelling Git 2.56.0 accepts,
//!  with real-Git
//!       controls for the table and for the readings the incumbent got wrong.
//! Why:
//!  `isDryRun` gates the manual-push policy checks;
//!  reading a push option's value `-n`
//!      as a dry run would skip them for a real push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parsePushRegion(['-nf', 'origin']).isDryRun).toBe(true);
//! ```

/// The parser,
///  its table,
///  the oracles and the real-Git fixture helpers.
use super::{PUSH_TABLE, PushRegion, parse_push_region, stackable_option_tokens};
use crate::command_options::{OptionErrorKind, WrapperOccurrence};
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, git_status, os_arguments, output_text, remove,
    repository_with_tracked_file,
};
use std::path::{Path, PathBuf};
use std::process::Output;

/// Parse a region Git accepts,
///  with no wrapper flags.
fn region(values: &[&str]) -> PushRegion {
    return parse_push_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// An atomicity choice counts in either direction and in abbreviated form.
#[test]
fn detects_an_atomicity_choice() {
    assert!(!region(&["origin", "main"]).atomic_stated);
    assert!(!region(&[]).atomic_stated);
    for values in [
        vec!["--atomic", "origin", "main"],
        vec!["--no-atomic", "origin", "main"],
        vec!["origin", "main", "--atomic"],
        vec!["--at"],
        vec!["--no-at"],
        vec!["--atomic", "--no-atomic"],
    ] {
        assert!(region(values.as_slice()).atomic_stated, "{values:?}");
    }
    // In a value position or after `--` the token is not an option.
    for values in [
        vec!["-o", "--atomic", "origin"],
        vec!["--repo", "--atomic"],
        vec!["origin", "--", "--atomic"],
    ] {
        assert!(!region(values.as_slice()).atomic_stated, "{values:?}");
    }
}

/// The last dry-run choice wins,
///  in clusters and abbreviations too.
#[test]
fn reports_the_final_dry_run_state() {
    for values in [
        vec!["-n"],
        vec!["--dry-run", "origin"],
        vec!["--dry"],
        vec!["-nf"],
        vec!["-fn"],
        vec!["-vnq"],
        vec!["--no-dry-run", "-n"],
    ] {
        assert!(region(values.as_slice()).dry_run, "{values:?}");
    }
    for values in [
        vec!["origin", "main"],
        vec!["-n", "--no-dry-run"],
        vec!["--dry-run", "--no-dry"],
        // Divergence: `-n` here is the value of `-o`, `--repo` or `--exec`, or a refspec.
        vec!["-o", "-n", "origin"],
        vec!["--push-option", "--dry-run"],
        vec!["--repo", "-n"],
        vec!["--exec", "-n"],
        vec!["-on"],
        vec!["origin", "--", "-n"],
    ] {
        assert!(!region(values.as_slice()).dry_run, "{values:?}");
    }
}

/// Wrapper flags are reported by position;
///  a region Git refuses is reported as refused.
#[test]
fn reports_wrapper_flags_and_refusals() {
    let parsed: PushRegion = parse_push_region(
        os_arguments(&["--cli-git-keep-going", "-o", "--cli-git-keep-going"]).as_slice(),
        &[b"--cli-git-keep-going"],
    )
    .expect("valid region");
    assert_eq!(
        parsed.wrapper,
        vec![WrapperOccurrence { flag: 0, token: 0 }]
    );
    for (values, kind) in [
        (vec!["--unknown"], OptionErrorKind::UnknownOption),
        (vec!["-x"], OptionErrorKind::UnknownOption),
        (vec!["--atomic=1"], OptionErrorKind::UnexpectedValue),
        (vec!["--d"], OptionErrorKind::AmbiguousOption),
        (vec!["-o"], OptionErrorKind::MissingValue),
        (vec!["--no-ipv4"], OptionErrorKind::UnknownOption),
    ] {
        assert_eq!(
            parse_push_region(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused")
                .kind,
            kind,
            "{values:?}"
        );
    }
}

/// The copied table lists exactly the long options,
///  required values and negations the
/// binary reports.
#[test]
fn table_matches_git_push_completion_helper() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("push-table");
    assert_table_invariants(PUSH_TABLE);
    assert_eq!(
        render_completion(PUSH_TABLE),
        git_completion(root.as_path(), &["push"])
    );
    remove(directory.as_path());
}

/// Refs the bare remote holds.
fn remote_refs(remote: &Path) -> String {
    return output_text(&git(remote, &["for-each-ref", "--format=%(refname)"]));
}

/// Real Git accepts `--at`,
///  sends nothing for `-nf`,
///  and reads `-n` after `-o` as a push
/// option value,
///  so that push is real.
#[test]
fn abbreviation_cluster_and_value_readings_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("push-readings");
    let remote: PathBuf = directory.join("remote.git");
    git(
        directory.as_path(),
        &["init", "--quiet", "--bare", "remote.git"],
    );
    git(
        remote.as_path(),
        &["config", "receive.advertisePushOptions", "true"],
    );
    git(
        root.as_path(),
        &["push", "--quiet", "-nf", "../remote.git", "main"],
    );
    assert_eq!(remote_refs(remote.as_path()), "");
    assert!(region(&["--quiet", "-nf", "../remote.git", "main"]).dry_run);
    git(
        root.as_path(),
        &[
            "push",
            "--quiet",
            "-o",
            "-n",
            "../remote.git",
            "main:refs/heads/option",
        ],
    );
    assert_eq!(remote_refs(remote.as_path()), "refs/heads/option");
    assert!(!region(&["--quiet", "-o", "-n", "../remote.git", "main"]).dry_run);
    git(
        root.as_path(),
        &["push", "--quiet", "--at", "../remote.git", "main"],
    );
    assert!(remote_refs(remote.as_path()).contains("refs/heads/main"));
    assert!(region(&["--quiet", "--at", "../remote.git", "main"]).atomic_stated);
    let ambiguous: Output = git_status(root.as_path(), &["push", "--d", "../remote.git"]);
    assert_eq!(ambiguous.status.code(), Some(129));
    remove(directory.as_path());
}

/// The option tokens a stacked push repeats, or nothing, for a region written as `values`.
fn stackable(values: &[&str]) -> Option<Vec<String>> {
    return stackable_option_tokens(os_arguments(values).as_slice()).map(|tokens| {
        return tokens
            .iter()
            .map(|token| {
                return token.to_string_lossy().into_owned();
            })
            .collect();
    });
}

/// A plain update keeps its options and drops the remote and the refspecs the stacked push replaces.
#[test]
fn a_plain_push_keeps_its_options_and_drops_its_positionals() {
    assert_eq!(stackable(&[]), Some(vec![]));
    assert_eq!(stackable(&["origin", "HEAD:main"]), Some(vec![]));
    assert_eq!(stackable(&["--no-verify", "origin", "HEAD:main"]), Some(vec![String::from("--no-verify")]));
    assert_eq!(
        stackable(&["-o", "ci.skip", "origin", "main"]),
        Some(vec![String::from("-o"), String::from("ci.skip")])
    );
    assert_eq!(stackable(&["--", "origin", "main"]), Some(vec![String::from("--")]));
    assert_eq!(stackable(&["--no-force", "origin", "main"]), Some(vec![String::from("--no-force")]));
    assert_eq!(stackable(&["--force", "--no-force", "origin", "main"]).map(|tokens| tokens.len()), Some(2));
}

/// Every option that rewrites, removes or widens what is sent, or changes what the caller reads back, is never stacked.
#[test]
fn forms_that_rewrite_remove_widen_or_redirect_a_push_are_never_stacked() {
    for option in [
        "--force", "-f", "--force-with-lease", "--force-with-lease=main:abc", "--force-if-includes", "--mirror", "--delete",
        "-d", "--all", "--branches", "--tags", "--prune", "--follow-tags", "--repo=origin", "--porcelain", "-u",
        "--set-upstream", "-n", "--dry-run", "--del", "--mir", "--no-such-option",
    ] {
        assert_eq!(stackable(&[option, "origin", "main"]), None, "{option}");
    }
    assert_eq!(stackable(&["origin", "+main"]), None);
    assert_eq!(stackable(&["origin", "+HEAD:main"]), None);
    assert_eq!(stackable(&["--", "origin", "+main"]), None);
}

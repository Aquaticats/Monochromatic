//! Controls for the relay that masks the URLs of a push's two streams.

use super::*;
use crate::lfs_output_mask::CREDENTIALS_MASK;
use crate::test_support::{executable, fixture, remove};
use std::path::PathBuf;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// What:
///  The two streams of a push reach the caller with the secrets of their URLs masked,
/// each on its own stream,
/// and Git's status comes back.
/// Why:
///   Git names the destination it could not reach on its standard error and the one it pushed to in the summary on its
/// standard output,
/// and a destination can carry a token in a path segment or in its query (audit `alpha-12-lfs`, finding "A typed push
/// relays URL path and query secrets without the wrapper's mask").
#[test]
fn the_two_streams_of_a_push_reach_the_caller_with_their_url_secrets_masked() {
    let root: PathBuf = fixture("push-relay-masked");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(bin.as_path()).expect("bin directory");
    let canary: &str = "relaycanary";
    let told_out: PathBuf = root.join("out.txt");
    let told_err: PathBuf = root.join("err.txt");
    std::fs::write(told_out.as_path(), format!("To https://user:{canary}@host.invalid/{canary}/repo.git\n")).expect("the summary");
    std::fs::write(
        told_err.as_path(),
        format!("fatal: unable to access 'https://host.invalid/{canary}/lfs?token={canary}': Failed to connect\n"),
    )
    .expect("the failure");
    // The lines are read from files, so the fixture's own text is never quoted into shell syntax.
    let source: String = format!("#!/bin/sh\ncat '{out}'\ncat '{err}' >&2\nexit 3\n", out = told_out.display(), err = told_err.display());
    executable(bin.join("git").as_path(), source.as_bytes());
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let ended: ForwardedRun =
        run_masked_push(bin.join("git").as_path(), words(&["push", "origin", "main"]).as_slice(), &[], Some(0), &mut out, &mut err);
    let shown_out: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    let shown_err: String = String::from_utf8_lossy(err.as_slice()).into_owned();
    remove(root.as_path());
    assert_eq!(ended, ForwardedRun::Ran(3), "Git's own status comes back");
    assert!(!shown_out.contains(canary), "the summary's secret is masked: {shown_out}");
    assert!(!shown_err.contains(canary), "the failure's secret is masked: {shown_err}");
    assert_eq!(shown_out, format!("To https://{m}@host.invalid/{m}/{m}\n", m = CREDENTIALS_MASK));
    assert_eq!(
        shown_err,
        format!("fatal: unable to access 'https://host.invalid/{m}/{m}?{m}={m}': Failed to connect\n", m = CREDENTIALS_MASK),
    );
}

/// What:
///  The meter Git draws is asked for only when the caller's standard error is a terminal and the person asked for neither
/// the meter nor silence.
/// Why:
///   This face reads that stream through a pipe, and Git draws its meter only on a terminal,
/// so the meter native Git would have shown is lost unless Git is asked for it.
#[test]
fn the_progress_meter_is_asked_for_only_where_native_git_would_have_drawn_one() {
    let asked: Vec<OsString> = words(&["push", "origin", "main"]);
    assert_eq!(with_progress(asked.as_slice(), Some(0), false), asked, "a piped standard error asks nothing of Git");
    assert_eq!(with_progress(asked.as_slice(), Some(0), true), words(&["push", "--progress", "origin", "main"]), "a terminal gets the meter back");
    assert_eq!(
        with_progress(words(&["push", "--quiet", "origin", "main"]).as_slice(), Some(0), true),
        words(&["push", "--quiet", "origin", "main"]),
        "a quiet push keeps its quiet",
    );
    assert_eq!(
        with_progress(words(&["push", "-q", "origin"]).as_slice(), Some(0), true),
        words(&["push", "-q", "origin"]),
        "the short quiet option counts too",
    );
    assert_eq!(
        with_progress(words(&["push", "--progress", "origin"]).as_slice(), Some(0), true),
        words(&["push", "--progress", "origin"]),
        "the meter is asked for once",
    );
    assert_eq!(
        with_progress(words(&["push", "origin", "main"]).as_slice(), Some(0), true),
        words(&["push", "--progress", "origin", "main"]),
        "the option follows the subcommand, never the program",
    );
}

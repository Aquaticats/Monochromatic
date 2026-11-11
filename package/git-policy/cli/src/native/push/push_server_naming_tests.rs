//! Controls for the word a push command gives as its destination,
//! which is what a wait and a stop name the server from.

use super::*;

/// The password of the fixture's URL,
/// made up for this control.
const CANARY_PASS: &str = "CANARYPASS9137";

/// The label and the destination of a command's words are read by one rule,
/// which skips an option and the value of a `-c`.
#[test]
fn the_words_of_a_command_skip_an_option_and_the_value_of_a_c() {
    let mut command: Command = Command::new("git");
    command.args(["-c", "core.hooksPath=/dev/null", "--no-pager", "push", "origin", "main"]);
    let mut arguments = command.get_args();
    assert_eq!(first_word(&mut arguments).as_deref(), Some("push"));
    assert_eq!(first_word(&mut arguments).as_deref(), Some("origin"));
    assert_eq!(first_word(&mut arguments).as_deref(), Some("main"));
    assert_eq!(first_word(&mut arguments), None, "a word that ran out answers nothing");
}

/// A push that names a remote and one that names a URL both give it as their destination,
/// and a fetch with no destination gives none.
#[test]
fn the_destination_is_the_word_after_the_subcommand() {
    let mut named: Command = Command::new("git");
    named.args(["push", "origin", "refs/heads/main:refs/heads/main"]);
    assert_eq!(destination_of(&named).as_deref(), Some("origin"));
    let mut url: Command = Command::new("git");
    url.args(["push", "https://host.invalid/repo.git", "main"]);
    assert_eq!(destination_of(&url).as_deref(), Some("https://host.invalid/repo.git"));
    let mut upstream: Command = Command::new("git");
    upstream.args(["fetch", "--prune"]);
    assert_eq!(destination_of(&upstream), None);
}

/// What:
///  A destination written as a URL answers with its host and port,
/// with no userinfo in it.
/// Why the question runs on the process's own runtime:
///   `ServerNaming::ask` is awaited by a push that itself runs as a task of that runtime.
#[test]
fn a_url_destination_names_its_host_without_asking_git() {
    let environment: Vec<(OsString, OsString)> = Vec::new();
    let mut command: Command = Command::new("git");
    // Written from the parts, so the repository's credential scanner reads no URL userinfo here.
    let written: String = format!("https://lfs:{CANARY_PASS}@127.0.0.1:36437/lfs");
    command.args(["push", written.as_str(), "main"]);
    let mut naming: ServerNaming<'_> = ServerNaming::of(&command, environment.as_slice());
    assert_eq!(naming.server(), None, "nothing is known before the question is asked");
    assert!(crate::lazy_runtime::run_on_runtime(naming.ask()).is_ok(), "the question is asked");
    assert_eq!(naming.server(), Some("127.0.0.1:36437"));
}

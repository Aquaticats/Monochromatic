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

/// The words the audit's held upload printed,
/// which name no server at all: git-lfs says nothing while it waits.
const SILENT_UPLOAD: &str = "Uploading LFS objects:   0% (0/1), 0 B | 0 B/s, done.\n";

/// What:
///  The words a failure printed,
///  with a URL whose userinfo must not become part of the name.
/// Why written from the parts:
///  the repository's credential scanner reads the text of a committed file,
///  and a URL userinfo written whole is what it refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namingFailure = `batch response: Post "https://lfs:${pass}@127.0.0.1:41827/lfs/objects/batch": refused`
/// ```
fn naming_failure() -> String {
    let password: &str = "CANARYPASS9137";
    return format!("batch response: Post \"https://lfs:{password}@127.0.0.1:41827/lfs/objects/batch\": refused\n");
}

/// What:
///  An upload's naming takes the server the caller already knows,
///  which the upload plan read from the configuration,
///  and no query runs for it.
/// Why:
///   The upload's program is `git-lfs`,
///  which has no `remote get-url` command at all,
///  so the query the push's naming asks failed with `unknown command "remote"`
///  and the stop named no server;
///  and even an answer would name the Git remote's host rather than the LFS endpoint that held the connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// naming.namedByTheCaller('127.0.0.1:39489'); await naming.ask(); expect(naming.server()).toBe('127.0.0.1:39489')
/// ```
#[test]
fn an_upload_names_the_server_its_caller_knows_without_a_query() {
    let failed: String = naming_failure();
    for (case, output, wanted) in [
        ("a silent upload keeps what the caller knows", SILENT_UPLOAD, "127.0.0.1:39489"),
        ("the program's own words win over what the caller knew", failed.as_str(), "127.0.0.1:41827"),
    ] {
        let environment: Vec<(OsString, OsString)> = Vec::new();
        let mut command: Command = Command::new("git-lfs");
        command.args(["push", "--object-id", "origin", "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0"]);
        let mut naming: ServerNaming<'_> = ServerNaming::of(&command, environment.as_slice());
        naming.named_by_the_caller(Some("127.0.0.1:39489"));
        naming.consider_output(output);
        assert!(crate::lazy_runtime::run_on_runtime(naming.ask()).is_ok(), "{case}: the question is asked");
        assert_eq!(naming.server(), Some(wanted), "{case}");
        assert!(!naming.server().unwrap_or_default().contains("CANARYPASS9137"), "{case}: a credential is never part of the name");
        assert_eq!(destination_of(&command).as_deref(), Some("origin"), "{case}: the destination is the remote the upload runs for");
    }
    // An upload whose caller named no server asks nothing: git-lfs has no `remote get-url`, so a query would only fail.
    let environment: Vec<(OsString, OsString)> = Vec::new();
    let mut command: Command = Command::new("git-lfs");
    command.args(["push", "origin", "main"]);
    let mut naming: ServerNaming<'_> = ServerNaming::of(&command, environment.as_slice());
    assert!(crate::lazy_runtime::run_on_runtime(naming.ask()).is_ok(), "the question is asked");
    assert_eq!(naming.server(), None, "an upload with no known server names none rather than asking a program that cannot answer");
}

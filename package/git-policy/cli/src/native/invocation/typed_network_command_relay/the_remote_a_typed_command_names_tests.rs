//! Controls for the remote a typed command names:
//! which word is the remote,
//! which URL Git answers with,
//! which URLs ssh carries,
//! and the host and connect words a sentence may print.

use super::{
    RemoteRead,
    TypedNetworkCommand,
    remote_read_for,
    ssh_carries,
    typed_command,
};
use crate::typed_network_command_relay::a_target_the_sentence_may_name::{
    Target,
    target_of,
};
use crate::typed_network_command_relay::the_options_before_the_remote::remote_operand;

/// What:
///  The words of one command line, as the person typed them.
/// Why:
///  Every control below reads arguments and never a shell line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (...parts: string[]) => parts.map((part) => Buffer.from(part));
/// ```
fn words(parts: &[&str]) -> Vec<std::ffi::OsString> {
    return parts
        .iter()
        .map(|part| return std::ffi::OsString::from(*part))
        .collect();
}

/// A clone's own word is its remote, and a directory after it is not.
#[test]
fn a_clone_names_its_url_as_its_first_word() {
    let arguments: Vec<std::ffi::OsString> = words(&[
        "clone",
        "ssh://target/work/served.git",
        "/work/clones/probe",
    ]);
    // What: `Some((command, region))` is the pair of the command word and the words after it.
    // Why:  The control reads both, and a command outside the four answers nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = typedCommand(argv);
    // ```
    let (command, region, _prefix) =
        typed_command(arguments.as_slice()).expect("a clone is a typed network command");
    assert_eq!(command, TypedNetworkCommand::Clone);
    assert_eq!(
        remote_read_for(command, region),
        Some(RemoteRead::Operand(std::ffi::OsStr::new(
            "ssh://target/work/served.git"
        )))
    );
}

/// A fetch's remote is the first word that is not an option, and an option's value is not that word.
#[test]
fn a_fetch_names_its_remote_after_the_option_values() {
    let arguments: Vec<std::ffi::OsString> = words(&[
        "fetch",
        "--depth",
        "1",
        "--upload-pack",
        "/usr/bin/git-upload-pack",
        "origin",
        "main",
    ]);
    let (fetch, fetched_region, _fetch_prefix) =
        typed_command(arguments.as_slice()).expect("a fetch is a typed network command");
    assert_eq!(
        remote_operand(fetch, fetched_region),
        Some(&std::ffi::OsString::from("origin"))
    );
    let pull: Vec<std::ffi::OsString> = words(&["pull", "--no-rebase", "upstream", "topic"]);
    let (pulled, pulled_region, _pull_prefix) =
        typed_command(pull.as_slice()).expect("a pull is a typed network command");
    assert_eq!(
        remote_operand(pulled, pulled_region),
        Some(&std::ffi::OsString::from("upstream"))
    );
}

/// A word Git's table does not know leaves the remote unread, so the command keeps today's process replacement.
#[test]
fn an_unknown_option_leaves_the_remote_unread() {
    let arguments: Vec<std::ffi::OsString> = words(&["fetch", "--no-such-option-at-all", "origin"]);
    let (unknown, unknown_region, _unknown_prefix) =
        typed_command(arguments.as_slice()).expect("the command word is still read");
    assert_eq!(remote_operand(unknown, unknown_region), None);
}

/// A command outside the four is not a typed network command, and a global option comes before the command word.
#[test]
fn only_the_four_network_commands_are_read() {
    assert_eq!(
        typed_command(words(&["status", "--short"]).as_slice()),
        None
    );
    let with_globals: Vec<std::ffi::OsString> =
        words(&["-C", "/work/served", "--no-pager", "push", "origin", "main"]);
    let (pushing, pushed_region, prefix) =
        typed_command(with_globals.as_slice()).expect("the command follows the global options");
    assert_eq!(pushing, TypedNetworkCommand::Push);
    assert_eq!(
        remote_operand(pushing, pushed_region),
        Some(&std::ffi::OsString::from("origin"))
    );
    // The words before the command are Git's global options, which the read of the remote must carry with it.
    assert_eq!(
        prefix,
        words(&["-C", "/work/served", "--no-pager"]).as_slice()
    );
}

/// Every scheme ssh speaks is carried, and every other transport is not.
#[test]
fn ssh_carries_its_own_schemes_and_the_scp_form() {
    for url in [
        "ssh://target/work/served.git",
        "SSH://target/work/served.git",
        "git+ssh://target/served.git",
        "ssh+git://target/served.git",
        "target:work/served.git",
        "root@target:served.git",
        "target:22/served.git",
        "gh:served.git",
    ] {
        assert!(ssh_carries(url), "{url} is an ssh remote");
    }
    for url in [
        "https://example.invalid/served.git",
        "http://example.invalid/served.git",
        "git://example.invalid/served.git",
        "file:///work/served.git",
        "/work/served.git",
        "../served.git",
        "served.git",
        "",
    ] {
        assert!(!ssh_carries(url), "{url} is not an ssh remote");
    }
}

/// The host is the name the remote gives, and the connect words carry the port only when it is not ssh's own.
#[test]
fn the_host_is_the_name_the_remote_gives() {
    assert_eq!(
        target_of("ssh://target/work/served.git"),
        Some(Target {
            host: String::from("target"),
            connect: String::from("target")
        })
    );
    assert_eq!(
        target_of("ssh://root@127.0.0.1:2222/work/served.git"),
        Some(Target {
            host: String::from("127.0.0.1"),
            connect: String::from("-p 2222 root@127.0.0.1")
        })
    );
    assert_eq!(
        target_of("ssh://target:22/work/served.git"),
        Some(Target {
            host: String::from("target"),
            connect: String::from("target")
        })
    );
    assert_eq!(
        target_of("root@target:served.git"),
        Some(Target {
            host: String::from("target"),
            connect: String::from("root@target")
        })
    );
}

/// A host, user or port that is not a plain word is dropped rather than printed into a command.
#[test]
fn nothing_a_host_cannot_hold_reaches_the_sentence() {
    // A URL that names no host this wrapper may print gets no sentence at all, and no byte is dropped and printed as if the
    //  remote had named what is left.
    assert_eq!(target_of("ssh://$(id)/x"), None);
    assert_eq!(target_of("ssh://ho$(st)/x"), None);
    assert_eq!(target_of("ssh://us;er@target/x"), None);
    assert_eq!(target_of("ssh://:2222/x"), None);
    assert_eq!(
        target_of("ssh://target:notaport/x"),
        Some(Target {
            host: String::from("target"),
            connect: String::from("target")
        })
    );
}

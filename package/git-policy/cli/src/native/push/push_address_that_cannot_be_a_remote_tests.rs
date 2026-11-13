//! Controls of the line issue 6293 draws between "can never be a remote" and "not reachable now".
//! Why:
//!  The line cannot come from Git's words, so each case here names the words Git 2.56.0 answered when the address was
//!  pushed to from a fixture on 2026-11-13 (`keep/push-address-probe-output.txt` and
//!  `keep/push-address-shapes-output.txt` beside that run's report), and the reading this module makes of it.

use super::*;

/// Git's answer to every relative or absolute path that is not a repository now,
/// whatever the reason:
/// measured for `DISABLED`, `no-such-remote`, `/mnt/not-mounted/repo.git`, a directory that exists and holds no
/// repository, `my remote` and `a`+newline+`b`, all with Git 2.56.0.
const A_PATH_THAT_IS_NOT_A_REPOSITORY_NOW: &str = "fatal: 'X' does not appear to be a git repository";

/// What:
///  The four shapes no machine state can turn into a remote are read as final.
/// Why:
///  Each is Git's own answer to the shape itself, quoted beside the case.
#[test]
fn an_address_that_no_machine_state_can_make_a_remote_is_final() {
    // `git push '' main`: "fatal: bad repository ''".
    assert_eq!(cannot_be_a_remote(""), Some(CannotBeARemote::Empty));
    // This project's reserved word (`AGENTS.md` rule APQ): a repository of that name does receive a push
    // (`git push ../DISABLED main` exited 0 in the fixture), so the word is a decision of ours, not a syntax of Git.
    assert_eq!(cannot_be_a_remote(DISABLED_ADDRESS), Some(CannotBeARemote::TheReservedWord));
    // `git push -force main`: "error: did you mean `--force` (with two dashes)?", exit 129, and no transport start.
    assert_eq!(cannot_be_a_remote("-force"), Some(CannotBeARemote::AnOption));
    assert_eq!(cannot_be_a_remote("--force"), Some(CannotBeARemote::AnOption));
    // `git push ://example.com/repo main`: "fatal: protocol '' is not supported".
    assert_eq!(cannot_be_a_remote("://example.com/repo"), Some(CannotBeARemote::NoProtocolName));
    assert_eq!(cannot_be_a_remote("://"), Some(CannotBeARemote::NoProtocolName));
}

/// What:
///  Every address that might name a remote now or later keeps its retries, whatever Git answered.
/// Why:
///  A path that is not there now may be mounted, cloned or written; a word that is not exactly the reserved one is a
///  path; an unknown protocol may get its helper; a host that does not answer may answer.
///  Git answered the same `fatal: '…' does not appear to be a git repository` for the reserved word and for every one of
///  the paths, which is why the words alone decide nothing.
#[test]
fn an_address_that_might_be_a_remote_later_is_never_final() {
    for address in [
        // A directory of the reserved name, and the reserved word with a space: both are ordinary paths to Git.
        "./DISABLED",
        "disabled",
        "DISABLED ",
        // A path that is not there now, and one whose drive is not mounted:
        "no-such-remote",
        "/mnt/not-mounted/repo.git",
        "file:///mnt/not-mounted/repo.git",
        // A directory that exists and is no repository: it may become one.
        "/home/a/plain-dir",
        // Names with a space and with a newline, which a filesystem allows:
        "my remote",
        "a\nb",
        " ",
        // A protocol whose helper may be installed, and one Git does not know yet:
        "foo://example.com/repo",
        "1foo://example.com/repo",
        // A host that does not answer, with and without user information that Git masks in its own message:
        "https://127.0.0.1:1/repo.git",
        "https://user:secret@127.0.0.1:1/repo.git",
        // A remote-like address Git resolves through ssh:
        "user@127.0.0.1:/srv/repo.git",
    ] {
        assert_eq!(cannot_be_a_remote(address), None, "{address:?} is quoted in Git's words as {A_PATH_THAT_IS_NOT_A_REPOSITORY_NOW}");
    }
}

/// What:
///  One address that can never be a remote ends a push that names several, because Git stops at the first that fails.
/// Why:
///  Measured with Git 2.56.0: `remote.origin.pushurl` set to `DISABLED` and then to a working repository, a plain
///  `git push origin main` failed on `DISABLED` and the working address received nothing.
#[test]
fn one_address_that_cannot_be_a_remote_ends_a_push_that_names_several() {
    assert_eq!(first_that_cannot_be_remote(&[]), None, "an address this release could not read keeps its retries");
    let working: Vec<String> = vec![String::from("/srv/real.git"), String::from("https://host/repo.git")];
    assert_eq!(first_that_cannot_be_remote(working.as_slice()), None);
    let mixed: Vec<String> = vec![String::from("/srv/real.git"), String::from(DISABLED_ADDRESS)];
    assert_eq!(first_that_cannot_be_remote(mixed.as_slice()), Some((DISABLED_ADDRESS, CannotBeARemote::TheReservedWord)));
    let blank: Vec<String> = vec![String::new(), String::from("/srv/real.git")];
    assert_eq!(first_that_cannot_be_remote(blank.as_slice()), Some(("", CannotBeARemote::Empty)));
}

/// What:
///  The line a person reads says what was not done, why, and that no try can mend it,
///  and it is one line.
/// Why:
///  The notice of a failed push already names the branch and the commit above this line, and a person reads the first
///  line of a notice.
#[test]
fn the_line_says_the_push_was_not_made_and_why_in_one_line() {
    let reserved: String = words(DISABLED_ADDRESS, CannotBeARemote::TheReservedWord);
    assert!(!reserved.contains('\n'), "the line is one line: {reserved}");
    assert!(reserved.contains("did not push"), "{reserved}");
    assert!(reserved.contains(DISABLED_ADDRESS), "{reserved}");
    assert!(reserved.contains("stopped trying at once"), "{reserved}");
    assert!(reserved.contains("saved in this repository"), "{reserved}");
    assert!(reserved.contains("git remote set-url --push"), "a way forward is named: {reserved}");
    for (address, why) in [
        ("", CannotBeARemote::Empty),
        ("-force", CannotBeARemote::AnOption),
        ("://example.com/repo", CannotBeARemote::NoProtocolName),
    ] {
        let line: String = words(address, why);
        assert!(!line.contains('\n'), "{line}");
        assert!(line.contains("did not push") || line.contains("could not push"), "{line}");
        assert!(line.contains("stopped trying at once"), "{line}");
    }
}

/// What:
///  An address this program prints never shows the password a URL may carry.
/// Why:
///  Git masks user information in its own messages, and the line this program writes must not be the one that prints a
///  secret to a terminal, a log or a transcript.
#[test]
fn a_printed_address_never_shows_the_user_information_of_a_url() {
    let masked: String = words("-https://user:secret@host/repo.git", CannotBeARemote::AnOption);
    assert!(!masked.contains("secret"), "{masked}");
    assert!(!masked.contains("user"), "{masked}");
    assert!(masked.contains("-https://***@host/repo.git"), "{masked}");
    // An address without a URL's user information is printed as it is, and so is a path holding an `@`.
    assert!(words("-force", CannotBeARemote::AnOption).contains("`-force`"));
    assert!(words("://user@host/x", CannotBeARemote::NoProtocolName).contains("://***@host/x"));
}

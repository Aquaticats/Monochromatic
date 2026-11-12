//! Controls for the arguments real Git is given for a commit it builds,
//! and for reading the parents of what it made.

use super::*;

fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

fn strings(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The parents come from the header block only,
/// in order;
/// a line in the message that looks like a parent is not one.
#[test]
fn parents_come_from_the_header_block_in_order() {
    let merge: &[u8] = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\nparent 1111\nparent 2222\nauthor A <a@b> 1 +0000\n\nparent 3333\n";
    assert_eq!(parents_of(merge), vec![String::from("1111"), String::from("2222")]);
    let root: &[u8] = b"tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\nauthor A <a@b> 1 +0000\n\nparent 9999\n";
    assert_eq!(parents_of(root), Vec::<String>::new());
}

/// A commit of named paths is given neither the paths nor the separator nor `--only`,
/// and keeps its message options.
#[test]
fn named_paths_and_the_separator_are_left_out_for_git() {
    let (globals, commit) = private_invocation(
        &arguments(&["commit", "--only", "-m", "subject", "--", "a.txt", "dir/b.txt"]),
        &strings(&["a.txt", "dir/b.txt"]),
        TransactionMode::ExplicitPath,
        false,
        false,
    );
    assert!(globals.is_empty());
    assert_eq!(commit, strings(&["-m", "subject"]));
}

/// Global options that would send Git elsewhere are dropped,
/// the others are kept before the command.
#[test]
fn global_options_that_redirect_git_are_dropped() {
    let (globals, commit) = private_invocation(
        &arguments(&["-C", "/elsewhere", "-c", "core.editor=true", "commit", "-S", "-m", "signed"]),
        &[],
        TransactionMode::Index,
        false,
        false,
    );
    assert_eq!(globals, strings(&["-c", "core.editor=true"]));
    assert_eq!(commit, strings(&["-S", "-m", "signed"]));
}

/// A selection the private index already holds is not given to Git again,
/// with its pathspec file and its pathspecs.
#[test]
fn a_selection_the_private_index_holds_is_not_given_to_git_again() {
    let line: Vec<OsString> = arguments(&["commit", "-i", "-m", "all", "--pathspec-from-file=list", "--pathspec-file-nul", "--", "a.txt"]);
    assert_eq!(private_invocation(&line, &strings(&["a.txt"]), TransactionMode::Index, false, true).1, strings(&["-m", "all"]));
    let patch: Vec<OsString> = arguments(&["commit", "--patch", "-m", "p"]);
    assert_eq!(private_invocation(&patch, &[], TransactionMode::Index, false, true).1, strings(&["-m", "p"]));
    assert_eq!(private_invocation(&patch, &[], TransactionMode::Index, false, false).1, strings(&["--patch", "-m", "p"]));
}

/// `-a` is removed once it has staged into the private index,
/// whether alone or in a cluster,
/// and not otherwise.
#[test]
fn commit_all_is_removed_only_when_the_index_already_holds_it() {
    let line: Vec<OsString> = arguments(&["commit", "-am", "all"]);
    assert_eq!(private_invocation(&line, &[], TransactionMode::Index, true, false).1, strings(&["-m", "all"]));
    assert_eq!(private_invocation(&line, &[], TransactionMode::Index, false, false).1, strings(&["-am", "all"]));
}

/// What:
///  A message that holds a byte which is not UTF-8 reaches the real commit with that byte.
/// Why:
///   The person's message is bytes,
/// and a spelling with replacement characters records another message:
///  the commit would say something the person never typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(privateInvocation({ arguments: ['commit', '-m', Buffer.from('message-\xff'), '--', 'a.txt'], pathspecs: ['a.txt'] }).commitArgs).toEqual(['-m', Buffer.from('message-\xff')]);
/// ```
#[test]
fn a_message_that_is_not_text_reaches_git_with_its_own_bytes() {
    let message: OsString = crate::path_bytes::os_from_path_bytes(b"message-\xff");
    let line: Vec<OsString> = vec![OsString::from("commit"), OsString::from("-m"), message.clone(), OsString::from("--"), OsString::from("a.txt")];
    let (globals, commit): (Vec<OsString>, Vec<OsString>) = private_invocation(&line, &arguments(&["a.txt"]), TransactionMode::ExplicitPath, false, false);
    assert!(globals.is_empty());
    assert_eq!(commit, vec![OsString::from("-m"), message]);
}

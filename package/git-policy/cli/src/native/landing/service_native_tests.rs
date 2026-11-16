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

/// What:
///  The form is read from the words Git itself reads:
///  a value an option takes is never a form,
///  a unique prefix of a form option is that option,
///  and a token after `--` is a path.
/// Why:
///  The scanner this replaces looked at every argument alone, so
///  `--message --fixup=reword:HEAD` was read as a reword fixup and the tree check was switched off for a commit that carried
///  the index's tree (the ninth review's first finding, reproduced at the tip),
///  and `--amen` was read as an ordinary commit, so a commit Git makes was refused (measured, `keep/abbreviated-options-measured.md`).
///  The rows are generated from the options the installed Git lists,
///  so an option a later Git adds appears as a row nobody answered.
#[test]
fn the_form_follows_git_option_grammar() {
    let root: PathBuf = crate::test_support::fixture("form-grammar");
    // Git answers its own option list only inside a repository, which is also where a commit is read.
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let listed: String = crate::test_support::git_text(repo.as_path(), &["commit", "--git-completion-helper"]);
    let mut valued: Vec<String> = Vec::new();
    let mut longs: Vec<String> = Vec::new();
    for word in listed.split_whitespace() {
        match word.strip_suffix('=') {
            Some(name) => {
                valued.push(String::from(name));
                longs.push(String::from(name));
            }
            None if word.starts_with("--") => longs.push(String::from(word)),
            None => {}
        }
    }
    assert!(!valued.is_empty(), "the installed Git lists the options of git commit that take a value: {listed}");
    // Every option that takes a value, given a value spelled like a form: the form must not change.
    let spellings: [&str; 3] = ["--amend", "--fixup=reword:HEAD", "--fixup=amend:HEAD"];
    for option in &valued {
        for value in spellings {
            let line: Vec<OsString> = vec![OsString::from(option.as_str()), OsString::from(value)];
            assert_eq!(
                the_form_of_the_commit(&line),
                TheFormOfTheCommit::CarriesTheIndex,
                "the value {value} of {option} is a value, not a form"
            );
        }
    }
    // A short option's value, joined and separate.
    for line in [arguments(&["-m--amend"]), arguments(&["-m", "--fixup=reword:HEAD"])] {
        assert_eq!(the_form_of_the_commit(&line), TheFormOfTheCommit::CarriesTheIndex, "a short option's value is no form: {line:?}");
    }
    // Every prefix of --amend and --fixup that Git would take as the whole option.
    for (name, shortest) in [("--amend", 3_usize), ("--fixup", 4_usize)] {
        let mut shared: usize = name.len();
        for other in &longs {
            if other != name {
                let common: usize = other.chars().zip(name.chars()).take_while(|(left, right)| { return left == right; }).count();
                if common == name.len() && other.len() > name.len() {
                    // A longer name that this one prefixes makes every prefix ambiguous.
                    shared = 0;
                }
            }
        }
        assert!(shared > 0, "{name} has no longer option sharing its whole spelling");
        for length in shortest..=name.len() {
            let prefix: &str = &name[..length];
            let unique: usize = longs.iter().filter(|other| { return other.starts_with(prefix); }).count();
            if unique != 1 {
                continue;
            }
            let alone: Vec<OsString> = vec![OsString::from(prefix)];
            let expected: TheFormOfTheCommit = if name == "--amend" {
                TheFormOfTheCommit::AmendsTheLastCommit
            } else {
                let with_value: Vec<OsString> = vec![OsString::from(format!("{prefix}=reword:HEAD"))];
                assert_eq!(
                    the_form_of_the_commit(&with_value),
                    TheFormOfTheCommit::CarriesTheRewordedTree,
                    "{prefix}=reword:HEAD is a reword fixup"
                );
                continue;
            };
            assert_eq!(the_form_of_the_commit(&alone), expected, "{prefix} is {name} to Git");
        }
    }
    // After the end-of-options delimiter every token is a path.
    let after_delimiter: Vec<OsString> = vec![
        OsString::from("--"),
        OsString::from("--amend"),
        OsString::from("--fixup=reword:HEAD"),
    ];
    assert_eq!(the_form_of_the_commit(&after_delimiter), TheFormOfTheCommit::CarriesTheIndex, "a path is no form");
    crate::test_support::remove(root.as_path());
}

//! What:
//!  The line a moved branch rebuilds a commit with,
//!  and the options it drops because the captured bytes already hold them.
//! Why:
//!  These are pure readings of an argument list,
//!  so they are checked without starting Git;
//!  the controls that land such a commit run the built executable
//!  (`binary_moved_branch_commit_tests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(withoutSettledOptions({ region: ['--patch'], interactive: true, borrowed: undefined })).toEqual({ kind: 'line', tokens: [] });
//! ```

use super::{BorrowedAuthor, Settled, borrows_its_author, rewords_a_commit, rewords_another_commit, without_selection_letters, without_settled_options};
use crate::commit_args::normalise_commit_args;
use std::ffi::{OsStr, OsString};

/// The words of one command line as the walk reads them.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// What one region keeps,
/// each token with its own bytes,
/// with the settled options taken out.
/// Why:
///   The walk reads the region the way `reissued_arguments` hands it over,
///  with a value inside a short cluster split out and the paths after the separator untouched.
fn kept(items: &[&str], interactive: bool, borrowed: Option<&BorrowedAuthor>) -> Vec<OsString> {
    let region: Vec<OsString> = normalise_commit_args(words(items).as_slice());
    return match without_settled_options(region.as_slice(), interactive, borrowed) {
        Settled::Line(tokens) => tokens,
        Settled::NeedsTheAuthorship => panic!("the walk asked for an authorship it was given none of: {items:?}"),
        Settled::Cannot => panic!("the walk refused a line it should rebuild: {items:?}"),
    };
}

/// The authorship of a commit a `-C` reused,
/// as the rebuild reads it.
fn reused() -> BorrowedAuthor {
    return BorrowedAuthor { author: String::from("Reused Author <reused@example.com>"), date: String::from("1234567890 +0000") };
}

/// What:
///  A line whose selection the private index holds loses every way of asking for it again.
/// Why:
///  The person chose the hunks once;
///  a rebuild that kept `--patch` or `-p` would ask a question at a terminal the commit no longer has,
///  and Git's answer there is the end of the input and no selection at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(kept(['--patch'], true)).toEqual([]);
/// expect(kept(['-p'], true)).toEqual([]);
/// ```
#[test]
fn a_settled_selection_loses_every_option_that_would_ask_again() {
    for items in [
        vec!["--patch"],
        vec!["--interactive"],
        vec!["-p"],
        vec!["-i"],
        vec!["-qp"],
        vec!["-e", "--patch", "-S"],
    ] {
        let tokens: Vec<OsString> = kept(items.as_slice(), true, None);
        for token in &tokens {
            let text: String = token.to_string_lossy().into_owned();
            assert!(!text.contains('p') && !text.contains('i'), "{items:?} kept the selection option {text}");
        }
    }
    // The rest of a cluster survives, and a key that follows `-S` is never read as a flag.
    assert_eq!(kept(&["-qp"], true, None), vec![OsString::from("-q")]);
    assert_eq!(kept(&["-Skey-with-p"], true, None), vec![OsString::from("-Skey-with-p")]);
    assert_eq!(kept(&["--quiet", "--patch", "--no-verify"], true, None), words(&["--quiet", "--no-verify"]));
}

/// A selection that was not settled keeps the options the person typed.
#[test]
fn a_selection_that_was_not_settled_keeps_its_options() {
    assert_eq!(kept(&["--patch"], false, None), vec![OsString::from("--patch")]);
    assert_eq!(kept(&["-p"], false, None), vec![OsString::from("-p")]);
}

/// What:
///  A line that took its message and author from another commit drops that option,
///  and the author and date the person typed with it,
///  because the rebuild supplies both from the commit Git built.
/// Why:
///  `-C` copies the authorship of the commit it names;
///  a rebuilt commit that kept `-C` would read that commit twice,
///  and one that dropped it without `--author` and `--date` would lose the authorship the person kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(kept(['-C', 'HEAD', '-S'], false, Some(reused()))).toEqual(['-S']);
/// ```
#[test]
fn a_reused_authorship_leaves_its_option_and_the_authorship_the_person_typed() {
    let borrowed: BorrowedAuthor = reused();
    for items in [
        vec!["-C", "HEAD"],
        vec!["-CHEAD"],
        vec!["-c", "HEAD"],
        vec!["--reuse-message=HEAD"],
        vec!["--reedit-message", "HEAD"],
        vec!["--author=Me <me@example.com>", "-C", "HEAD"],
        vec!["-C", "HEAD", "--date=2020-01-01T00:00:00Z"],
    ] {
        let tokens: Vec<OsString> = kept(items.as_slice(), false, Some(&borrowed));
        assert!(tokens.is_empty(), "{items:?} kept {tokens:?}");
    }
    // An author the person typed beside a borrowed commit is in the commit Git built, which the rebuild reads it from.
    assert_eq!(kept(&["--author=Me <me@example.com>", "-C", "HEAD"], false, Some(&borrowed)), Vec::<OsString>::new());
}

/// A line that borrows an authorship nobody read yet asks for it.
#[test]
fn a_line_that_borrows_an_authorship_says_so_once() {
    let region: Vec<OsString> = words(&["-C", "HEAD"]);
    assert!(matches!(without_settled_options(region.as_slice(), false, None), Settled::NeedsTheAuthorship));
    assert!(borrows_its_author(OsStr::new("-C")));
    assert!(borrows_its_author(OsStr::new("--reuse-message=HEAD")));
    assert!(!borrows_its_author(OsStr::new("-m")));
}

/// What:
///  A fixup commit is made of its message,
///  so its option goes and the message lands,
///  except for the one fixup whose commit is the branch's own tree.
/// Why:
///  `--fixup=<commit>` and `--fixup=amend:<commit>` commit the index this preparation captured,
///  while `--fixup=reword:<commit>` commits the branch's own tree,
///  which a replay of the captured tree would get wrong:
///  such a line is landed from the fields of the commit Git built instead
///  (`rewords_a_commit`,
/// and the arm of `service_commit.rs` for a branch that moved).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(kept(['--fixup=HEAD'], false, None)).toEqual([]);
/// expect(withoutSettledOptions({ region: ['--fixup=reword:HEAD'], interactive: false })).toEqual({ kind: 'cannot' });
/// ```
#[test]
fn a_fixup_that_commits_the_index_goes_and_a_reword_does_not() {
    for items in [vec!["--fixup=HEAD"], vec!["--fixup", "HEAD"], vec!["--fixup=amend:HEAD"], vec!["--squash=HEAD"]] {
        assert!(kept(items.as_slice(), false, None).is_empty(), "{items:?}");
    }
    for items in [vec!["--fixup=reword:HEAD"], vec!["--squash=reword:HEAD"]] {
        assert!(rewords_another_commit(OsStr::new(items[0])), "{items:?}");
        let region: Vec<OsString> = words(items.as_slice());
        assert!(matches!(without_settled_options(region.as_slice(), false, None), Settled::Cannot), "{items:?}");
    }
}

/// What:
///  The line the commit loop reads to know that an empty commit is what the person asked for.
/// Why:
///   `--fixup=reword:<commit>` makes its commit from the branch,
///  so the preparation of the attempt that lands it on a branch that moved holds no change at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(rewordsACommit(['commit', '--fixup=reword:HEAD'], 0)).toBe(true);
/// expect(rewordsACommit(['commit', '-m', '--fixup=reword:HEAD'], 0)).toBe(false);
/// ```
#[test]
fn a_reword_fixup_is_the_line_that_asks_for_an_empty_commit() {
    // The subcommand's index is the caller's: a global option before `commit` moves it.
    let asks = |items: &[&str], subcommand: usize| -> bool {
        return rewords_a_commit(words(items).as_slice(), subcommand);
    };
    assert!(asks(&["commit", "--fixup=reword:HEAD"], 0));
    assert!(asks(&["commit", "--squash=reword:HEAD"], 0));
    assert!(asks(&["commit", "--quiet", "--fixup=reword:HEAD", "--no-enforce-only"], 0));
    assert!(asks(&["-c", "core.x=1", "commit", "--fixup=reword:HEAD"], 2));
    // A value that reads like the option is the person's text, and neither a plain fixup nor a path is one of these.
    assert!(!asks(&["commit", "-m", "--fixup=reword:HEAD"], 0));
    assert!(!asks(&["commit", "--fixup=HEAD"], 0));
    assert!(!asks(&["commit", "--fixup=amend:HEAD"], 0));
    assert!(!asks(&["commit", "-m", "x", "--", "--fixup=reword:HEAD"], 0));
}

/// What:
///  The value of an option is the person's text,
///  so a value that reads like one of the options a rebuild drops is kept.
/// Why:
///  `git commit -m --patch` commits the message `--patch`;
///  dropping the token would silently change what the person wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(kept(['-m', '--patch'], true, None)).toEqual(['-m', '--patch']);
/// ```
#[test]
fn a_message_that_reads_like_a_dropped_option_is_kept() {
    assert_eq!(kept(&["-m", "--patch"], true, None), words(&["-m", "--patch"]));
    assert_eq!(kept(&["--message=-p"], true, None), vec![OsString::from("--message=-p")]);
    assert_eq!(kept(&["-m", "-C"], false, None), words(&["-m", "-C"]));
}

/// Everything after the separator is a path,
/// and a path is never read as an option.
#[test]
fn every_path_after_the_separator_is_kept_as_it_is() {
    assert_eq!(
        kept(&["--patch", "--", "-p", "--fixup=reword:HEAD"], true, None),
        words(&["--", "-p", "--fixup=reword:HEAD"])
    );
}

/// A cluster keeps the letters a rebuild does not supply itself.
#[test]
fn a_cluster_keeps_every_letter_a_rebuild_does_not_supply() {
    assert_eq!(without_selection_letters(OsStr::new("-qpa")), Some(OsString::from("-qa")));
    assert_eq!(without_selection_letters(OsStr::new("-pi")), None);
    assert_eq!(without_selection_letters(OsStr::new("-Spq")), Some(OsString::from("-Spq")));
}

//! What:
//!  Which commands the linked-worktree policy guards,
//!  and every rejection text.
//! Why:
//!  A harmless form read as destructive blocks an inspection;
//!  a destructive form read
//!      as harmless lets `git reset --hard` run in the primary checkout.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(decideLinkedWorktree(['clean', '-n'])).toEqual({ kind: 'pass' });
//! ```

/// The decision,
///  its texts and the argument builder.
use super::{
    DirtyQuestion, GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE, LinkedWorktreeDecision,
    TargetTree, decide_linked_worktree, dirty_question, dirty_worktree_message, exemption_notice,
    ignored_file_message, main_worktree_message, outside_worktree_message, repository_stash_message,
    resolve_linked_worktree, target_tree,
};
use crate::command_test_support::{fixture, git, git_status, os_arguments, remove, repository};
use crate::effective_target::EffectiveTarget;
use crate::rule_worktree_dirty::DirtyScope;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The index of a form,
/// so the coverage control can tell a full list from one with a hole.
/// A `match` must name every variant,
/// so a new form stops compilation here until it has an index.
fn index_of(command: GuardedCommand) -> usize {
    return match command {
        GuardedCommand::Stash => 0,
        GuardedCommand::StashDrop => 1,
        GuardedCommand::Clean => 2,
        GuardedCommand::CleanIgnored => 3,
        GuardedCommand::CleanIgnoredOnly => 4,
        GuardedCommand::ResetHard => 5,
        GuardedCommand::ResetMixed => 6,
        GuardedCommand::RestoreStaged => 7,
        GuardedCommand::RestoreWorktree => 8,
        GuardedCommand::CheckoutPaths => 9,
        GuardedCommand::CheckoutMove => 10,
        GuardedCommand::SwitchForce => 11,
        GuardedCommand::SwitchMove => 12,
        GuardedCommand::Remove => 13,
        GuardedCommand::Move => 14,
        GuardedCommand::CheckoutIndex => 15,
        GuardedCommand::ReadTree => 16,
        GuardedCommand::AbortMerge => 17,
        GuardedCommand::AbortRebase => 18,
        GuardedCommand::AbortPick => 19,
        GuardedCommand::AbortAm => 20,
    };
}

/// `GuardedCommand::ALL` holds every form once,
/// so a control that walks it reaches every rejection text.
#[test]
fn every_guarded_form_is_listed_once() {
    let mut seen: Vec<usize> = GuardedCommand::ALL.iter().map(|command| { return index_of(*command); }).collect();
    seen.sort_unstable();
    seen.dedup();
    let forms: usize = index_of(GuardedCommand::AbortAm) + 1;
    let expected: Vec<usize> = (0..forms).collect();
    assert_eq!(seen, expected, "GuardedCommand::ALL must hold each of the {forms} forms once");
}

/// Decide one invocation written as text.
fn decide(values: &[&str]) -> LinkedWorktreeDecision {
    return decide_linked_worktree(os_arguments(values).as_slice());
}

/// Every stash form,
///  a deleting clean and a file-rewriting reset need the worktree kind.
#[test]
fn destructive_forms_need_the_worktree_kind() {
    assert_eq!(LINKED_WORKTREE_REQUIRED_CODE, "linked-worktree-required");
    for (values, command) in [
        (vec!["stash"], GuardedCommand::Stash),
        (vec!["stash", "drop"], GuardedCommand::StashDrop),
        (vec!["stash", "pop"], GuardedCommand::Stash),
        (vec!["stash", "clear"], GuardedCommand::StashDrop),
        (vec!["stash", "push", "-m", "note"], GuardedCommand::Stash),
        (vec!["stash", "--no-such-option"], GuardedCommand::Stash),
        (vec!["clean"], GuardedCommand::Clean),
        (vec!["clean", "-fd"], GuardedCommand::Clean),
        (vec!["clean", "-n", "--no-dry-run"], GuardedCommand::Clean),
        (vec!["clean", "-i"], GuardedCommand::Clean),
        (vec!["clean", "-fdx"], GuardedCommand::CleanIgnored),
        (vec!["clean", "-f", "-x"], GuardedCommand::CleanIgnored),
        (vec!["clean", "--force", "-d", "--quiet", "-x"], GuardedCommand::CleanIgnored),
        (vec!["clean", "-fdX"], GuardedCommand::CleanIgnoredOnly),
        (vec!["clean", "-X"], GuardedCommand::CleanIgnoredOnly),
        (vec!["reset"], GuardedCommand::ResetMixed),
        (vec!["reset", "--mixed"], GuardedCommand::ResetMixed),
        (vec!["reset", "file"], GuardedCommand::ResetMixed),
        (vec!["reset", "--", "--hard"], GuardedCommand::ResetMixed),
        (vec!["reset", "--hard"], GuardedCommand::ResetHard),
        (vec!["reset", "--merge", "HEAD~1"], GuardedCommand::ResetHard),
        (vec!["reset", "--keep"], GuardedCommand::ResetHard),
        (vec!["reset", "--soft", "--hard"], GuardedCommand::ResetHard),
        (
            vec!["-C", "dir", "--no-pager", "reset", "--hard"],
            GuardedCommand::ResetHard,
        ),
        (vec!["restore", "a.txt"], GuardedCommand::RestoreWorktree),
        (vec!["restore", "--staged", "--", "."], GuardedCommand::RestoreStaged),
        (vec!["restore", "-S", "a.txt"], GuardedCommand::RestoreStaged),
        (vec!["restore", "--staged", "--worktree", "a.txt"], GuardedCommand::RestoreWorktree),
        (vec!["checkout", "--", "a.txt"], GuardedCommand::CheckoutPaths),
        (vec!["checkout", "-f"], GuardedCommand::CheckoutPaths),
        (vec!["switch", "--discard-changes", "topic"], GuardedCommand::SwitchForce),
        (vec!["switch", "-f", "topic"], GuardedCommand::SwitchForce),
        (vec!["rm", "-rf", "dir"], GuardedCommand::Remove),
        (vec!["mv", "--force", "a", "b"], GuardedCommand::Move),
        (vec!["checkout-index", "-f", "-a"], GuardedCommand::CheckoutIndex),
        (vec!["read-tree", "--reset", "HEAD"], GuardedCommand::ReadTree),
        (vec!["read-tree", "-m", "-u", "HEAD"], GuardedCommand::ReadTree),
        (vec!["read-tree", "HEAD"], GuardedCommand::ReadTree),
        (vec!["merge", "--abort"], GuardedCommand::AbortMerge),
        (vec!["rebase", "--abort"], GuardedCommand::AbortRebase),
        (vec!["cherry-pick", "--abort"], GuardedCommand::AbortPick),
        (vec!["revert", "--abort"], GuardedCommand::AbortPick),
        (vec!["am", "--abort"], GuardedCommand::AbortAm),
    ] {
        assert_eq!(
            decide(values.as_slice()),
            LinkedWorktreeDecision::NeedsEffectiveTarget(command),
            "{values:?}"
        );
    }
}

/// A named target moves `HEAD` and is rejected only while the worktree is dirty.
#[test]
fn head_moves_need_the_dirtiness() {
    for (values, command) in [
        (vec!["checkout", "topic"], GuardedCommand::CheckoutMove),
        (vec!["checkout", "-"], GuardedCommand::CheckoutMove),
        (vec!["switch", "topic"], GuardedCommand::SwitchMove),
        (vec!["switch", "--detach", "HEAD~1"], GuardedCommand::SwitchMove),
    ] {
        assert_eq!(
            decide(values.as_slice()),
            LinkedWorktreeDecision::WhenDirty(command),
            "{values:?}"
        );
    }
}

/// Each form asks the question about the object it can destroy:
///  a clean that deletes untracked files reads tracked and untracked work,
///  a clean that deletes ignored files only asks nothing at all,
/// a stash drop asks the repository,
/// and every other form loses tracked changes only.
#[test]
fn each_form_asks_about_the_state_it_can_destroy() {
    for command in GuardedCommand::ALL {
        let expected: Option<DirtyQuestion> = match command {
            GuardedCommand::Clean | GuardedCommand::CleanIgnored => {
                Some(DirtyQuestion::Worktree(DirtyScope::TrackedAndUntracked))
            }
            GuardedCommand::CleanIgnoredOnly => None,
            GuardedCommand::StashDrop => Some(DirtyQuestion::RepositoryStash),
            _ => Some(DirtyQuestion::Worktree(DirtyScope::Tracked)),
        };
        if command == GuardedCommand::CleanIgnoredOnly {
            // A clean that names its own exclusion patterns is asked of Git itself, because the status cannot see
            //  them; without them it deletes ignored files and nothing else, which is nothing to ask about.
            assert_eq!(
                dirty_question(command, os_arguments(&["-fdx", "--exclude=notes.txt"]).as_slice()),
                Some(DirtyQuestion::CleanDryRun),
                "{command:?}"
            );
            for region in [vec!["-fdx"], vec!["-fdX"], vec!["-X"]] {
                assert_eq!(dirty_question(command, os_arguments(&region).as_slice()), None, "{region:?}");
            }
            continue;
        }
        assert_eq!(dirty_question(command, &[]), expected, "{command:?}");
    }
}

/// Other commands,
///  dry runs,
///  index-only resets and regions Git refuses pass without a measurement.
#[test]
fn harmless_forms_pass() {
    for values in [
        vec![],
        vec!["--version"],
        vec!["status"],
        vec!["add", "stash"],
        vec!["stash", "list"],
        vec!["stash", "list", "--stat"],
        vec!["stash", "show"],
        vec!["stash", "show", "-p", "stash@{1}"],
        vec!["stash", "create"],
        vec!["--no-pager", "stash", "list"],
        vec!["clean", "-n"],
        vec!["clean", "--dry-run", "-fd"],
        vec!["clean", "-nx"],
        vec!["clean", "-nX"],
        vec!["clean", "-i", "-n"],
        vec!["reset", "--soft", "HEAD~1"],
        vec!["reset", "--hard", "--soft"],
        vec!["restore", "-h"],
        vec!["checkout"],
        vec!["checkout", "-b", "topic"],
        vec!["switch", "-c", "topic"],
        vec!["rm", "a.txt"],
        vec!["rm", "--", "-f"],
        vec!["mv", "a", "b"],
        vec!["merge", "topic"],
        vec!["rebase", "main"],
        vec!["read-tree", "-n", "-m", "-u", "HEAD"],
        vec!["read-tree", "--dry-run", "--reset", "-u", "HEAD"],
        vec!["read-tree", "--debug-unpack", "-m", "-u", "HEAD"],
        vec!["read-tree", "--merge", "HEAD"],
        vec!["checkout", "--no-such-option"],
        vec!["switch", "--no-such-option"],
        vec!["clean", "--no-such-option"],
        vec!["reset", "--hard", "--no-such-option"],
        vec!["--no-such-global", "reset", "--hard"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            LinkedWorktreeDecision::Pass,
            "{values:?}"
        );
    }
}

/// Only the main worktree and a place without a worktree are rejected,
///  each with its own text.
#[test]
fn only_main_and_absent_worktrees_are_rejected() {
    for command in GuardedCommand::ALL {
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::MainWorktree),
            Some(main_worktree_message(command))
        );
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::OutsideWorktree),
            Some(outside_worktree_message(command))
        );
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::LinkedWorktree),
            None
        );
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::Allowlisted),
            None
        );
        assert_ne!(
            main_worktree_message(command),
            outside_worktree_message(command)
        );
    }
}

/// Every text names the form,
/// what it destroys,
/// a sanctioned alternative,
/// and the deliberate way to discard.
#[test]
fn every_text_names_the_loss_an_alternative_and_the_hatch() {
    for command in GuardedCommand::ALL {
        let main = main_worktree_message(command);
        let dirty = dirty_worktree_message(command);
        let outside = outside_worktree_message(command);
        assert!(main.starts_with("monochromatic-git-policy-cli: git "), "{command:?}: {main}");
        assert!(dirty.starts_with("monochromatic-git-policy-cli: git "), "{command:?}: {dirty}");
        assert!(outside.starts_with("monochromatic-git-policy-cli: git "), "{command:?}: {outside}");
        assert!(main.contains("in the main git worktree"), "{command:?}");
        assert!(dirty.contains("has uncommitted"), "{command:?}");
        assert!(outside.contains("-C <linked-worktree-root>"), "{command:?}");
        for text in [&main, &dirty] {
            assert!(text.contains("--no-enforce-worktree"), "{command:?}: {text}");
            assert!(text.contains("git diff > backup.patch"), "{command:?}: {text}");
            assert!(text.contains(" (allowed)") || text.contains("git commit --message")
                || text.contains("git rm <destination>") || text.contains("git rebase --continue")
                || text.contains("--continue") || text.contains("git diff -- <path>"),
                "{command:?} names no alternative: {text}");
        }
        assert_ne!(main, dirty);
        assert_ne!(main, outside);
    }
}

/// Each intent gets the alternative that fits it.
#[test]
fn each_intent_gets_its_own_alternative() {
    let unstage = main_worktree_message(GuardedCommand::RestoreStaged);
    assert!(unstage.contains("git commit --message <message> -- <paths>"));
    assert!(unstage.contains("git diff --cached --name-only"));
    let revert = main_worktree_message(GuardedCommand::RestoreWorktree);
    assert!(revert.contains("git show HEAD:<path>"));
    assert!(revert.contains("git diff -- <path> > backup.patch"));
    assert!(main_worktree_message(GuardedCommand::ResetHard).contains("git reset --soft <commit>"));
    assert!(main_worktree_message(GuardedCommand::Stash).contains("git worktree add -b <branch> <path>"));
    assert!(main_worktree_message(GuardedCommand::StashDrop).contains("git stash branch <branch>"));
    let stash: String = repository_stash_message(GuardedCommand::StashDrop);
    assert!(stash.contains("this repository holds a stash entry"), "{stash}");
    assert!(stash.contains("not this worktree's"), "{stash}");
    assert!(stash.contains("--no-enforce-worktree"), "{stash}");
    assert!(main_worktree_message(GuardedCommand::Clean).contains("git clean -n"));
    // An ignored file is not what a linked worktree's clean is refused over, so the sentence that reads after
    //  "has uncommitted" names only the work such a clean still loses.
    assert!(
        dirty_worktree_message(GuardedCommand::CleanIgnored)
            .contains("has uncommitted changes to tracked files or untracked files")
    );
    assert!(dirty_worktree_message(GuardedCommand::CleanIgnoredOnly).contains("--exclude"));
    assert!(dirty_worktree_message(GuardedCommand::Clean).contains("untracked files"));
    assert!(!dirty_worktree_message(GuardedCommand::RestoreWorktree).contains("untracked"));
    assert!(main_worktree_message(GuardedCommand::AbortMerge).contains("git merge --quit"));
    assert!(main_worktree_message(GuardedCommand::AbortRebase).contains("git rebase --quit"));
    assert!(main_worktree_message(GuardedCommand::AbortPick).contains("--quit"));
    assert!(main_worktree_message(GuardedCommand::AbortAm).contains("git am --quit"));
    assert!(main_worktree_message(GuardedCommand::CheckoutIndex).contains("--prefix=<directory>/"));
    assert!(main_worktree_message(GuardedCommand::Remove).contains("git rm --cached <path>"));
    assert!(dirty_worktree_message(GuardedCommand::SwitchMove).contains("git worktree add <path> <branch>"));
}

/// The README documents the alternative that each rejection text names,
/// in the same words.
#[test]
fn the_readme_documents_what_the_texts_recommend() {
    let readme: &str = include_str!("../../../README.md");
    assert!(readme.contains("### Discarding commands"));
    for (command, phrase) in [
        (GuardedCommand::Stash, "git worktree add -b <branch> <path>"),
        (GuardedCommand::StashDrop, "git stash branch <branch>"),
        (GuardedCommand::Clean, "git clean -n"),
        (GuardedCommand::CleanIgnored, "git clean -nx"),
        (GuardedCommand::CleanIgnoredOnly, "git clean -nX"),
        (GuardedCommand::ResetHard, "git reset --soft <commit>"),
        (GuardedCommand::ResetMixed, "git diff --cached --name-only"),
        (GuardedCommand::RestoreStaged, "git commit --message <message> -- <paths>"),
        (GuardedCommand::RestoreWorktree, "git show HEAD:<path>"),
        (GuardedCommand::CheckoutPaths, "git show HEAD:<path>"),
        (GuardedCommand::CheckoutMove, "git worktree add <path> <branch>"),
        (GuardedCommand::SwitchForce, "git worktree add <path> <branch>"),
        (GuardedCommand::SwitchMove, "git worktree add <path> <branch>"),
        (GuardedCommand::Remove, "git rm --cached <path>"),
        (GuardedCommand::Move, "git rm <destination>"),
        (GuardedCommand::CheckoutIndex, "git checkout-index --prefix=<directory>/ --all"),
        (GuardedCommand::ReadTree, "git read-tree --dry-run"),
        (GuardedCommand::AbortMerge, "--quit"),
        (GuardedCommand::AbortRebase, "--quit"),
        (GuardedCommand::AbortPick, "--quit"),
        (GuardedCommand::AbortAm, "--quit"),
    ] {
        assert!(
            main_worktree_message(command).contains(phrase),
            "{command:?} text lacks {phrase}"
        );
        assert!(readme.contains(phrase), "{command:?}: README lacks {phrase}");
    }
    assert!(readme.contains("git diff > backup.patch"));
    assert!(readme.contains("--no-enforce-worktree"));
}

/// The exemption sentence names the variable and the directory it points at,
/// and how to run the command without it.
#[test]
fn the_exemption_sentence_names_the_variable_that_exempted_the_repository() {
    let notice: String = exemption_notice("MISE_DATA_DIR", std::path::Path::new("/work/repo"));
    assert!(notice.contains("exempt from the worktree guard"), "{notice}");
    assert!(notice.contains("MISE_DATA_DIR"), "{notice}");
    assert!(notice.contains("/work/repo"), "{notice}");
    assert!(notice.contains("Nothing was measured and nothing was saved"), "{notice}");
    assert!(notice.contains("Hint: if MISE_DATA_DIR does not describe a tool cache"), "{notice}");
    assert!(notice.ends_with('\n'), "{notice}");
    assert_ne!(notice, exemption_notice("UV_CACHE_DIR", std::path::Path::new("/work/repo")));
}

/// A forced checkout,
/// a forced switch and a hard reset ask which tree they write;
/// every other form writes no tree and asks nothing.
#[test]
fn only_the_forms_that_write_a_tree_ask_which_tree() {
    let cases: [(GuardedCommand, &[&str], Option<Option<&str>>); 10] = [
        (GuardedCommand::ResetHard, &["--hard", "tracks"], Some(Some("tracks"))),
        (GuardedCommand::ResetHard, &["--hard"], Some(None)),
        (GuardedCommand::ResetHard, &["--hard", "--", "tracks"], Some(None)),
        (GuardedCommand::CheckoutPaths, &["--force", "tracks"], Some(Some("tracks"))),
        (GuardedCommand::CheckoutPaths, &["--force", "--", "tracked.txt"], Some(None)),
        (GuardedCommand::CheckoutPaths, &["--force", "tracks", "--", "tracked.txt"], Some(Some("tracks"))),
        (GuardedCommand::SwitchForce, &["--force", "tracks"], Some(Some("tracks"))),
        (GuardedCommand::SwitchForce, &["--force"], Some(None)),
        (GuardedCommand::Clean, &["-fdx"], None),
        (GuardedCommand::RestoreWorktree, &["--source", "tracks", "tracked.txt"], None),
    ];
    for (command, region, expected) in cases {
        let answer: Option<TargetTree> = target_tree(command, os_arguments(region).as_slice());
        assert_eq!(
            answer,
            expected.map(|revision| return TargetTree { revision: revision.map(OsString::from) }),
            "{command:?} {region:?}"
        );
    }
}

/// The text of an ignored file the target tree tracks names the form,
/// the paths,
/// the tree and the way
///  forward,
/// and it does not claim the worktree is dirty.
#[test]
fn the_ignored_file_text_names_the_paths_and_the_tree() {
    let one: String = ignored_file_message(GuardedCommand::ResetHard, Some(std::ffi::OsStr::new("tracks")), &[String::from(".env")]);
    assert!(one.contains("git reset --hard"), "{one}");
    assert!(one.contains("ignores .env"), "{one}");
    assert!(one.contains("the tree of tracks tracks"), "{one}");
    assert!(one.contains("would replace it"), "{one}");
    assert!(!one.contains("uncommitted changes to tracked files"), "{one}");
    assert!(one.contains("git reset --soft"), "{one}");
    assert!(one.contains("--no-enforce-worktree"), "{one}");
    // A form that names no revision writes HEAD.
    let head: String = ignored_file_message(GuardedCommand::CheckoutPaths, None, &[String::from(".env")]);
    assert!(head.contains("the tree of HEAD"), "{head}");
    // A worktree that ignores more shared paths than the text names counts the rest instead of listing them.
    let many: Vec<String> = (0..7).map(|index| return format!("ignored/{index}.bin")).collect();
    let listed: String = ignored_file_message(GuardedCommand::SwitchForce, None, many.as_slice());
    assert!(listed.contains("ignored/0.bin, ignored/1.bin"), "{listed}");
    assert!(!listed.contains("ignored/5.bin"), "{listed}");
    assert!(listed.contains("and 2 more"), "{listed}");
    assert!(listed.contains("would replace them"), "{listed}");
}

/// How real Git answers one `git read-tree` region,
/// measured in a repository that holds each kind of work a read can discard.
/// An `enum` is a closed set of named alternatives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GitAnswer {
    /// Git wrote the index,
    ///  a worktree file,
    ///  or both.
    Writes,
    /// Git wrote nothing.
    WritesNothing,
    /// Git refused the region before writing anything.
    Refuses,
}

/// What:
///  A repository holding the three kinds of uncommitted work a `git read-tree` can discard:
///  a staged edit,
///  a staged new file and an unstaged edit.
/// Why:
///  Each kind is what one column of `git status` shows,
///  and a reading that only saw staged edits would miss
///  the worktree file `-u` writes over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // write second.txt, add it, commit it, stage an edit plus new.txt, then edit tracked.txt
/// ```
fn repository_with_staged_work(directory: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = repository(directory, name);
    std::fs::write(repo.join("second.txt"), b"SECOND\n").expect("second file");
    git(repo.as_path(), &["add", "--", "second.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    std::fs::write(repo.join("second.txt"), b"SECOND-STAGED\n").expect("staged edit");
    std::fs::write(repo.join("new.txt"), b"NEW-STAGED\n").expect("staged new file");
    git(repo.as_path(), &["add", "--", "second.txt", "new.txt"]);
    std::fs::write(repo.join("tracked.txt"), b"UNSTAGED\n").expect("unstaged edit");
    return repo;
}

/// The index entries,
///  the porcelain status and every top-level file of a repository,
///  as one text.
/// Why:
///  A read that rewrites the index,
///  the index and the worktree,
///  or one worktree file alone must show in at least one of the three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // ls-files --stage + status --porcelain + readFileSync of each top-level file
/// ```
fn repository_state(repo: &Path) -> String {
    let mut state: String = String::new();
    for arguments in [
        vec!["ls-files", "--stage"],
        vec!["status", "--porcelain=v1", "--untracked-files=all"],
    ] {
        state.push_str(String::from_utf8_lossy(git_status(repo, arguments.as_slice()).stdout.as_slice()).as_ref());
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(repo)
        .expect("read repository")
        .map(|entry| return entry.expect("directory entry").path())
        .filter(|path| return path.is_file())
        .collect();
    paths.sort();
    for path in paths {
        state.push_str(format!("{}={:?}\n", path.display(), std::fs::read(&path).expect("file")).as_str());
    }
    return state;
}

/// What:
///  Every `git read-tree` form is decided the way real Git acts:
///  the forms that write the index or a worktree file are guarded,
///  the two that write nothing are not,
///  and a region Git refuses changes nothing.
/// Why:
///  `git read-tree -m -u HEAD` reverted a staged edit and deleted a staged new file from the worktree
///  with no guard,
///  because only `--reset` was read as destructive;
///  a reading written from Git's option table alone can miss a writing form,
///  so the table is measured against the repository each form leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // decide(['read-tree', '-m', '-u', 'HEAD']) is guarded, and git read-tree -m -u HEAD changes the worktree
/// ```
#[test]
fn every_read_tree_form_is_decided_the_way_real_git_acts() {
    let directory: PathBuf = fixture("read-tree-forms");
    for (index, (region, answer)) in [
        (vec!["HEAD"], GitAnswer::Writes),
        (vec!["-m", "HEAD"], GitAnswer::Writes),
        (vec!["-m", "-u", "HEAD"], GitAnswer::Writes),
        (vec!["-u", "-m", "HEAD"], GitAnswer::Writes),
        (vec!["--reset", "HEAD"], GitAnswer::Writes),
        (vec!["--res", "HEAD"], GitAnswer::Writes),
        (vec!["--reset", "-u", "HEAD"], GitAnswer::Writes),
        (vec!["-m", "-i", "HEAD"], GitAnswer::Writes),
        (vec!["--empty"], GitAnswer::Writes),
        (vec![], GitAnswer::Writes),
        (vec!["--no-reset", "HEAD"], GitAnswer::Writes),
        (vec!["--no-empty", "HEAD"], GitAnswer::Writes),
        (vec!["--prefix=sub/", "HEAD"], GitAnswer::Writes),
        (vec!["--prefix=sub/", "-u", "HEAD"], GitAnswer::Writes),
        (vec!["-v", "-m", "-u", "HEAD"], GitAnswer::Writes),
        (vec!["-q", "-m", "-u", "HEAD"], GitAnswer::Writes),
        (vec!["--super-prefix=super/", "HEAD"], GitAnswer::Writes),
        (vec!["-n", "-m", "-u", "HEAD"], GitAnswer::WritesNothing),
        (vec!["--dry-run", "--reset", "-u", "HEAD"], GitAnswer::WritesNothing),
        (vec!["--debug-unpack", "-m", "-u", "HEAD"], GitAnswer::WritesNothing),
        (vec!["-u", "HEAD"], GitAnswer::Refuses),
        (vec!["-m", "--prefix=sub/", "HEAD"], GitAnswer::Refuses),
        (vec!["--merge", "HEAD"], GitAnswer::Refuses),
    ]
    .into_iter()
    .enumerate()
    {
        let repo: PathBuf = repository_with_staged_work(directory.as_path(), format!("form-{index}").as_str());
        let before: String = repository_state(repo.as_path());
        let mut invocation: Vec<&str> = vec!["read-tree"];
        invocation.extend(region.iter().copied());
        let output: std::process::Output = git_status(repo.as_path(), invocation.as_slice());
        let after: String = repository_state(repo.as_path());
        let measured: GitAnswer = if before != after {
            GitAnswer::Writes
        } else if !output.status.success() {
            GitAnswer::Refuses
        } else {
            GitAnswer::WritesNothing
        };
        assert_eq!(
            measured,
            answer,
            "{region:?}: git said {}",
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        let decision: LinkedWorktreeDecision = decide(invocation.as_slice());
        match answer {
            GitAnswer::Writes => assert_eq!(
                decision,
                LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ReadTree),
                "{region:?} writes and must be guarded"
            ),
            GitAnswer::WritesNothing => assert_eq!(
                decision,
                LinkedWorktreeDecision::Pass,
                "{region:?} writes nothing and must pass"
            ),
            // A region Git refuses changes nothing; the reading may still see a writing form in it, which only
            //  rejects a command Git would refuse anyway.
            GitAnswer::Refuses => assert_eq!(after, before, "{region:?} changed a repository Git refused"),
        }
    }
    remove(directory.as_path());
}

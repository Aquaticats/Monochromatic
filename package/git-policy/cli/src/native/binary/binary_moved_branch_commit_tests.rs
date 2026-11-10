//! What:
//!  The three commits a moved branch still refused after a conclusion and an amend learned to land on it:
//!  a first commit on a branch that did not exist yet,
//!  a `--patch` commit whose selection the person made at the terminal,
//!  and a commit whose editor ran on a line that borrows a message and an author (`-C`) or names a fixup.
//! Why:
//!  The owner's words (2026-11-09),
//! beside "Commits should never fail.":
//! "I would consider this to be a defect in our wrapper.
//! Agents should never wait on each other on commits or merges."
//!  Each of these prepares a commit from bytes it captured and lands it on the branch as it is now,
//!  so a commit another session landed in the window between the preparation and the landing
//!  costs the person nothing they typed or chose.
//!  Every control runs the built executable against real Git in a disposable repository,
//!  held at the phase marker between preparation and landing,
//!  so the other command lands in exactly that window.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = heldCommit(fixture, repo, name, markers, { arguments: ['commit', '-m', 'first', '--', 'x.txt'] });
//! await waitHeld(held); await anotherSessionCommits(); release(held);
//! expect(await held).toLandOnTheNewTip();
//! ```

use super::commit::{git_text, remove_settled};
use super::hold_at_prepare::{Ended, Hold, Held, author_of, held_commit, marker_directory, parents, release, wait_held, write};
use super::service_commit::{Running, start, through_service};
use super::support::{Fixture, executable, fixture, git, repository};
use std::ffi::OsStr;
use std::path::PathBuf;

/// What:
///  A repository whose branch has no commit yet,
///  which is what `git commit` on a branch nobody has committed to finds.
/// Why:
///   `repository` makes an initial commit;
///   a first commit needs a branch that names nothing,
///   and the wrapper prepares such a commit against the all-zero name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(repo); await git('init', '--initial-branch=main');
/// ```
fn unborn_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = fixture.root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// What:
///  A first commit on a branch another session created while this one was prepared,
///  lands on that branch with both changes.
/// Why:
///   The captured base of such a commit is the all-zero name,
///  which `git merge-tree --merge-base` refuses;
///  the empty tree is what that base holds,
///  and the tree this commit captured is a whole root tree,
///  so replaying it from the empty tree keeps every path either commit has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldCommit({ arguments: ['commit', '-m', 'first', '--', 'x.txt'] }); await waitHeld(held);
/// await throughService(['commit', '-m', 'unrelated', '--', 'y.txt']); release(held);
/// expect(head().parents).toEqual([other]); expect(head().files).toEqual({ 'x.txt': 'mine\n', 'y.txt': 'theirs\n' });
/// ```
#[test]
fn a_first_commit_lands_on_a_branch_another_session_created_meanwhile() {
    let fixture: Fixture = fixture("unborn-commit-moved");
    let repo: PathBuf = unborn_repository(&fixture);
    write(&repo, "x.txt", "mine\n");
    let name: String = format!("ubm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(
        &fixture,
        &repo,
        name.as_str(),
        markers.as_path(),
        &Hold { arguments: &["commit", "--message=first", "--", "x.txt"], stdin: None, environment: &[] },
    );
    wait_held(&mut held);
    write(&repo, "y.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "y.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved.clone()], "the first commit sits on the branch as it is now");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "mine\n", "the file this commit captured");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:y.txt"]), "landed by another session\n", "nothing the other commit landed is lost");
    assert_eq!(git_text(&fixture, &repo, &["rev-list", "--count", "HEAD"]), "2\n", "the other commit, then this one");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1", "--untracked-files=all"]), "", "the index and worktree agree with the commit");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// What:
///  A repository with `picked.txt` and `left.txt` both edited and unstaged,
///  `x.txt` as a tracked file the selection does not touch,
///  and the answers that take the first file's only hunk and leave the second's.
/// Why:
///   `git commit --patch` asks the person which hunks to take,
///  so a control drives it with the answers on standard input exactly as a person types them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('picked.txt', 'picked\n'); write('left.txt', 'left\n'); // both tracked and unmodified at the last commit
/// ```
fn repository_for_a_patch_commit(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["picked.txt", "left.txt", "x.txt"] {
        write(&repo, file, "base\n");
    }
    git(fixture, &repo, &["add", "--", "picked.txt", "left.txt", "x.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    write(&repo, "picked.txt", "picked\n");
    write(&repo, "left.txt", "left\n");
    return repo;
}

/// The answers a person gives `git add --patch` here:
/// decline the hunk of `left.txt`,
/// take the hunk of `picked.txt`.
/// Why:
///   Git offers the files the worktree changed in path order,
///  and `x.txt` is not offered because nothing changed it.
const PATCH_ANSWERS: &[u8] = b"n\ny\n";

/// What:
///  A `--patch` commit whose branch moved while it was prepared lands the hunks the person chose,
///  and only those,
///  with the message the editor settled.
/// Why:
///   The person's choice is already in the private index the preparation captured,
///  so replaying that tree invents nothing and asks nothing again,
///  and the message the editor settled is built again from the file it was saved in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldCommit({ arguments: ['commit', '--patch'], stdin: 'n\ny\n', environment: [['GIT_EDITOR', editor]] });
/// await waitHeld(held); await throughService(['commit', '-m', 'unrelated', '--', 'x.txt']); release(held);
/// expect(head().files['picked.txt']).toBe('picked\n'); expect(head().files['left.txt']).toBe('base\n');
/// ```
#[test]
fn a_patch_commit_lands_the_selection_and_the_message_it_captured_on_a_branch_that_moved() {
    let fixture: Fixture = fixture("patch-commit-moved");
    let repo: PathBuf = repository_for_a_patch_commit(&fixture);
    let editor: PathBuf = fixture.root.join("editing-editor");
    executable(editor.as_path(), EDITOR_SCRIPT);
    let name: String = format!("pcm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(
        &fixture,
        &repo,
        name.as_str(),
        markers.as_path(),
        &Hold {
            arguments: &["commit", "--patch"],
            stdin: Some(PATCH_ANSWERS),
            environment: &[("GIT_EDITOR", editor.to_str().expect("the editor's path is text"))],
        },
    );
    wait_held(&mut held);
    write(&repo, "x.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved], "the commit sits on the branch as it is now");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:picked.txt"]), "picked\n", "the hunk the person took");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:left.txt"]), "base\n", "the hunk the person left out");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "landed by another session\n", "nothing the other commit landed is lost");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]), "written in the editor\n", "the message the editor settled");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%b"]).trim_end(), "", "and nothing after it");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// What:
///  A `--patch` commit with its message on the command line and something the service does not build
///  lands the same selection on a branch that moved,
///  with nothing asked at the terminal.
/// Why:
///   The service builds such a commit itself only from the command lines it covers;
///  `--verbose` sends this one to Git,
///  and a selection Git made is captured in the private index either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldCommit({ arguments: ['commit', '--patch', '--verbose', '-m', 'chosen'], stdin: 'n\ny\n' }); await waitHeld(held);
/// await throughService(['commit', '-m', 'unrelated', '--', 'x.txt']); release(held); expect(head().files['picked.txt']).toBe('picked\n');
/// ```
#[test]
fn a_patch_commit_git_builds_lands_its_selection_on_a_branch_that_moved() {
    let fixture: Fixture = fixture("patch-commit-git-built-moved");
    let repo: PathBuf = repository_for_a_patch_commit(&fixture);
    let name: String = format!("pgm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(
        &fixture,
        &repo,
        name.as_str(),
        markers.as_path(),
        &Hold {
            arguments: &["commit", "--patch", "--verbose", "--message=chosen"],
            stdin: Some(PATCH_ANSWERS),
            environment: &[],
        },
    );
    wait_held(&mut held);
    write(&repo, "x.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved], "the commit sits on the branch as it is now");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:picked.txt"]), "picked\n", "the hunk the person took");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:left.txt"]), "base\n", "the hunk the person left out");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "landed by another session\n", "nothing the other commit landed is lost");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]), "chosen\n", "the message the command line named");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The editor every control here gives Git:
/// it writes one line into the message Git prepared and keeps everything else.
const EDITOR_SCRIPT: &[u8] = b"#!/bin/sh\nprintf '\\nwritten in the editor\\n' >> \"$1\"\n";

/// What:
///  A repository whose last commit carries an author of its own and a date of its own,
///  with `a.txt` edited so a commit of that path has something to record.
/// Why:
///   `-C` takes the message and the authorship of the commit it names,
///  so a control can only say the authorship survived if the commit it reuses has one nothing else would produce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git('commit', '--author=Reused Author <reused@example.com>', '-m', 'reused message', { env: { GIT_AUTHOR_DATE: '1234567890 +0000' } });
/// ```
fn repository_for_a_reused_message(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["a.txt", "b.txt", "c.txt"] {
        write(&repo, file, "base\n");
    }
    git(fixture, &repo, &["add", "--", "a.txt", "b.txt", "c.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    write(&repo, "c.txt", "reused\n");
    git(
        fixture,
        &repo,
        &[
            "commit",
            "--quiet",
            "--author=Reused Author <reused@example.com>",
            "--date=1234567890 +0000",
            "--message=reused message",
            "--",
            "c.txt",
        ],
    );
    write(&repo, "a.txt", "one\ntwo\n");
    return repo;
}

/// What:
///  A commit that reuses another commit's message and author (`-C`) with an editor
///  lands on a branch that moved,
///  keeping the message the person left in the editor,
///  the author and author date the reuse took,
///  and the tree the replay settled.
/// Why:
///   Once the editor has run the message is settled,
///  and `-C`'s authorship is settled too:
///  both are read back from the commit Git built,
///  so nothing the person saw has to be typed again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldCommit({ arguments: ['commit', '--edit', '-C', 'reused', '--', 'a.txt'], environment: [['GIT_EDITOR', editor]] });
/// await waitHeld(held); await throughService(['commit', '-m', 'unrelated', '--', 'b.txt']); release(held);
/// expect(head().author).toBe('Reused Author <reused@example.com>'); expect(head().message).toContain('written in the editor');
/// ```
#[test]
fn a_commit_that_reuses_another_commit_lands_on_a_branch_that_moved() {
    let fixture: Fixture = fixture("reused-message-on-a-moved-branch");
    let repo: PathBuf = repository_for_a_reused_message(&fixture);
    let reused: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let editor: PathBuf = fixture.root.join("editing-editor");
    executable(editor.as_path(), EDITOR_SCRIPT);
    let name: String = format!("rms-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(
        &fixture,
        &repo,
        name.as_str(),
        markers.as_path(),
        &Hold {
            arguments: &["commit", "--edit", "-C", reused.as_str(), "--", "a.txt"],
            stdin: None,
            environment: &[("GIT_EDITOR", editor.to_str().expect("the editor's path is text"))],
        },
    );
    wait_held(&mut held);
    write(&repo, "b.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "b.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved], "the commit sits on the branch as it is now");
    let reused_author: String = author_of(&fixture, &repo, reused.as_str());
    assert!(reused_author.starts_with("Reused Author <reused@example.com>|"), "the control gave the reused commit its own author: {reused_author}");
    assert_eq!(author_of(&fixture, &repo, "HEAD"), reused_author, "the author and author date the reuse took");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]), "reused message\n", "the reused message");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%b"]).trim_end(), "written in the editor", "the line the person added");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:a.txt"]), "one\ntwo\n", "the change this commit captured");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:b.txt"]), "landed by another session\n", "nothing the other commit landed is lost");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// What:
///  A `--fixup=reword:<commit>` commit whose branch moved lands on the branch as it is now,
///  carrying that branch's own tree and the message the editor settled,
///  with the person's staged change still staged.
/// Why:
///  "Commits should never fail.",
///  and the acceptance bar of `SPEC.md`:
/// "nothing the machine or other processes do ends them with a nonzero status".
///  Another session moving the branch is exactly that case.
///  Git's own commit for this option carries the branch's tree and takes the index out of it,
///  so the commit made on the moved branch keeps the moved branch's tree
///  (`doc/decision/cli-git-concurrent-commits.md`,
/// "Serial landing":
/// a replay never lands changes the person did not choose,
/// and a replay of the captured tree here would land changes the command itself discards).
///  The message is the one the editor settled,
///  so the commit is written from the fields of the commit Git built rather than by running the line again,
///  which would ask the person a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldCommit({ arguments: ['commit', '--fixup=reword:<target>', '--no-enforce-only'] }); await waitHeld(held);
/// await throughService(['commit', '-m', 'unrelated', '--', 'x.txt']); release(held);
/// expect(head().tree).toBe(head().parent.tree); expect(head().subject).toBe('amend! target');
/// ```
#[test]
fn a_reword_fixup_lands_on_a_branch_that_moved_with_the_branchs_own_tree() {
    let fixture: Fixture = fixture("reword-fixup-on-a-moved-branch");
    let repo: PathBuf = fixture.root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(&fixture, &repo, &["init", "--quiet", "--initial-branch=main"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["a.txt", "x.txt"] {
        write(&repo, file, "base\n");
    }
    git(&fixture, &repo, &["add", "--", "a.txt", "x.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=target"]);
    let target: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    write(&repo, "a.txt", "staged by the person\n");
    git(&fixture, &repo, &["add", "--", "a.txt"]);
    // The target is named by the option itself: `--fixup=reword:<commit>` is one token.
    let reword: String = format!("--fixup=reword:{target}");
    let editor: PathBuf = fixture.root.join("editing-editor");
    executable(editor.as_path(), EDITOR_SCRIPT);
    let name: String = format!("rfm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(
        &fixture,
        &repo,
        name.as_str(),
        markers.as_path(),
        &Hold {
            arguments: &["commit", reword.as_str(), "--no-enforce-only"],
            stdin: None,
            environment: &[("GIT_EDITOR", editor.to_str().expect("the editor's path is text"))],
        },
    );
    wait_held(&mut held);
    write(&repo, "x.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved], "the commit sits on the branch as it is now");
    assert_eq!(
        git_text(&fixture, &repo, &["rev-parse", "HEAD^{tree}"]).trim(),
        git_text(&fixture, &repo, &["rev-parse", "HEAD^^{tree}"]).trim(),
        "the commit carries the branch's own tree"
    );
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]), "amend! target\n", "the message the editor settled");
    assert_eq!(
        git_text(&fixture, &repo, &["log", "-1", "--format=%b"]).trim_end(),
        "target\n\nwritten in the editor",
        "the target's own message, and the line the person added"
    );
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:a.txt"]), "base\n", "the person's staged change is not in the commit");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "landed by another session\n", "nothing the other commit landed is lost");
    assert_eq!(git_text(&fixture, &repo, &["diff", "--cached", "--name-only"]), "a.txt\n", "the staged change is still staged for its owner");
    assert_eq!(git_text(&fixture, &repo, &["rev-list", "--count", "HEAD"]), "3\n", "the target commit, the other session's, and this one");
    assert!(!ended.stderr.contains("while this amend was being prepared"), "no amendment is named: {}", ended.stderr);
    assert!(ended.stderr.contains("did not land"), "the build Git printed before the branch moved is named as lost: {}", ended.stderr);
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

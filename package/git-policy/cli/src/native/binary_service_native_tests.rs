//! What:
//!  The commits real Git builds for the service (an editor, a signature, a configuration or an option the service does
//!  not embed), compared field by field with the commit the transaction makes of the same input, and the cases where
//!  nothing may land.
//! Why:
//!  The service lands a commit Git built by moving the branch to it only while the branch is still where the commit was
//!  made. A difference in a stored field, in what is left behind, or a landing after an abort or a lost race would be a
//!  silent change of what an editor or a signature means.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::{Running, query, start, through_service};
use super::support::{Fixture, executable, fixture, git, repository};
use git_policy_cli::service_journal::DONE_EXTENSION;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

fn finished(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

/// The stored fields that must agree: tree, whole message, author, author date, committer, parents, and whether the
///  commit is signed. A signature's bytes are not compared, because a signer may put its own time in them.
fn stored(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let formats: [&str; 6] = ["%T", "%B", "%an <%ae>", "%ad", "%cn <%ce>", "%P"];
    let mut fields: Vec<String> =
        formats.map(|format| { return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str()]); }).to_vec();
    fields.push(git_text(fixture, repo, &["cat-file", "commit", "HEAD"]).contains("\ngpgsig ").to_string());
    return fields;
}

/// What a commit left behind: the status, the staged difference from the commit and how many commits the branch has.
fn left_behind(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return vec![
        git_text(fixture, repo, &["status", "--short"]),
        git_text(fixture, repo, &["diff", "--cached", "--name-status"]),
        git_text(fixture, repo, &["rev-list", "--count", "HEAD"]),
    ];
}

fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("write");
}

/// A repository with two tracked files and an editor that puts `edited: ` in front of the first line of the message, a
///  signer that makes a placeholder signature, and a worktree edit of one file ready to commit.
fn repository_with_programs(fixture: &Fixture) -> (PathBuf, String) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    write(&repo, "a.txt", "base\n");
    write(&repo, "b.txt", "other\n");
    git(fixture, &repo, &["add", "a.txt", "b.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=root\n\nroot body"]);
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let editor: PathBuf = fixture.root.join("typing-editor");
    executable(editor.as_path(), b"#!/bin/sh\nsed -i '1s/^/edited: /' \"$1\"\n");
    git(fixture, &repo, &["config", "core.editor", editor.to_str().expect("UTF-8")]);
    let signer: PathBuf = fixture.root.join("stand-in-gpg");
    executable(
        signer.as_path(),
        b"#!/bin/sh\ncat > /dev/null\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nplaceholder\\n-----END PGP SIGNATURE-----\\n'\n",
    );
    git(fixture, &repo, &["config", "gpg.program", signer.to_str().expect("UTF-8")]);
    let start_commit: String = query(fixture, &repo, &["rev-parse", "HEAD"]);
    return (repo, start_commit);
}

fn edit_a(repo: &Path) {
    write(repo, "a.txt", "base\nedited\n");
}

fn edit_a_and_sign_by_configuration(repo: &Path) {
    edit_a(repo);
    let output = std::process::Command::new("/usr/bin/git").current_dir(repo).args(["config", "commit.gpgsign", "true"]).output().expect("config");
    assert!(output.status.success());
}

fn edit_a_and_clean_up_by_configuration(repo: &Path) {
    edit_a(repo);
    let output = std::process::Command::new("/usr/bin/git").current_dir(repo).args(["config", "commit.cleanup", "verbatim"]).output().expect("config");
    assert!(output.status.success());
}

/// One command line, the state it starts from, and whether the author date is a field the line keeps.
struct Case {
    name: &'static str,
    command: &'static [&'static str],
    prepare: fn(&Path),
    date_kept: bool,
}

fn case(name: &'static str, command: &'static [&'static str], prepare: fn(&Path)) -> Case {
    return Case { name, command, prepare, date_kept: false };
}

/// Every line real Git builds for the service gives the commit the transaction gives, leaves the repository as it leaves
///  it, and the service (not the transaction) landed it.
#[test]
fn commits_git_builds_for_the_service_equal_the_transaction_commit() {
    let fixture: Fixture = fixture("service-native");
    let (repo, start_commit) = repository_with_programs(&fixture);
    let cases: Vec<Case> = vec![
        case("editor, named path", &["commit", "--", "a.txt"], edit_a),
        case("editor, seeded message", &["commit", "-e", "-m", "seed", "--", "a.txt"], edit_a),
        case("editor, index", &["commit", "--no-enforce-only"], |repo| {
            edit_a(repo);
            let output = std::process::Command::new("/usr/bin/git").current_dir(repo).args(["add", "a.txt"]).output().expect("add");
            assert!(output.status.success());
        }),
        case("editor, squash", &["commit", "--squash=HEAD", "--", "a.txt"], edit_a),
        Case { date_kept: true, ..case("editor, amend", &["commit", "--amend", "--", "a.txt"], edit_a) },
        case("signed by an option", &["commit", "-S", "-m", "signed", "--", "a.txt"], edit_a),
        case("signed by configuration", &["commit", "-m", "signed", "--", "a.txt"], edit_a_and_sign_by_configuration),
        Case { date_kept: true, ..case("signed amend", &["commit", "--amend", "--no-edit", "-S", "--", "a.txt"], edit_a) },
        case("clean-up mode by configuration", &["commit", "-m", "kept   \n\n\n", "--", "a.txt"], edit_a_and_clean_up_by_configuration),
        case("scissors clean-up", &["commit", "--cleanup=scissors", "-m", "above\n# ------------------------ >8 ------------------------\nbelow", "--", "a.txt"], edit_a),
        case("author found in history", &["commit", "--author=Fixture", "-m", "found", "--", "a.txt"], edit_a),
    ];
    let name: String = format!("native-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in &cases {
        let mut outcomes: Vec<(Vec<String>, Vec<String>, Option<i32>)> = Vec::new();
        for through in [true, false] {
            git(&fixture, &repo, &["reset", "--hard", "--quiet", start_commit.as_str()]);
            let _unset = std::process::Command::new("/usr/bin/git").current_dir(&repo).args(["config", "--unset-all", "commit.gpgsign"]).output();
            let _unset_cleanup = std::process::Command::new("/usr/bin/git").current_dir(&repo).args(["config", "--unset-all", "commit.cleanup"]).output();
            git(&fixture, &repo, &["clean", "--force", "--quiet", "-d"]);
            (entry.prepare)(&repo);
            let before: usize = finished(&fixture);
            let output: Output = if through {
                through_service(&fixture, &repo, name.as_str(), entry.command)
            } else {
                wrapped_commit(&fixture, &repo, entry.command).output().expect("transaction")
            };
            if through {
                assert_eq!(finished(&fixture), before + 1, "{}: the service landed it: {}", entry.name, String::from_utf8_lossy(&output.stderr));
            }
            outcomes.push((stored(&fixture, &repo), left_behind(&fixture, &repo), output.status.code()));
        }
        assert_eq!(outcomes[0].2, outcomes[1].2, "{}: exit status", entry.name);
        assert_eq!(outcomes[0].2, Some(0), "{}", entry.name);
        assert_eq!(outcomes[0].1, outcomes[1].1, "{}: what is left behind", entry.name);
        for (index, (left, right)) in outcomes[0].0.iter().zip(&outcomes[1].0).enumerate() {
            // Field 3 is the author date: the time of the run unless an amend kept the replaced commit's.
            if index == 3 && !entry.date_kept {
                continue;
            }
            assert_eq!(left, right, "{}: stored field {index}", entry.name);
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// An editor that leaves an empty message, or fails, ends the commit with Git's status; nothing lands, nothing moves, and
///  the commit's scratch directory is gone.
#[test]
fn an_abandoned_editor_lands_nothing_and_leaves_no_scratch_behind() {
    let fixture: Fixture = fixture("service-native-abort");
    let (repo, start_commit) = repository_with_programs(&fixture);
    let name: String = format!("native-abort-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let empties: PathBuf = fixture.root.join("empty-editor");
    executable(empties.as_path(), b"#!/bin/sh\n: > \"$1\"\n");
    let failing: PathBuf = fixture.root.join("failing-editor");
    executable(failing.as_path(), b"#!/bin/sh\nexit 3\n");
    for (label, program) in [("an empty message", empties), ("an editor that fails", failing)] {
        git(&fixture, &repo, &["config", "core.editor", program.to_str().expect("UTF-8")]);
        edit_a(&repo);
        let before: usize = finished(&fixture);
        let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--", "a.txt"]);
        assert_eq!(output.status.code(), Some(1), "{label}: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(finished(&fixture), before, "{label}: nothing landed");
        assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD"]), start_commit, "{label}: the branch did not move");
        let scratch: Vec<String> = std::fs::read_dir(repo.join(".git"))
            .expect("git directory")
            .flatten()
            .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
            .filter(|entry| { return entry.starts_with("cli-git-service-"); })
            .collect();
        assert!(scratch.is_empty(), "{label}: left {scratch:?}");
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A commit Git built is not moved onto a tip it was not made on: while the editor is open another commit lands on the
///  branch, and the commit that was typed is refused with the plain failure and does not land.
#[test]
fn a_branch_that_moves_while_the_editor_is_open_refuses_the_commit_that_was_typed() {
    let fixture: Fixture = fixture("service-native-moved");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let name: String = format!("native-moved-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let racing: PathBuf = fixture.root.join("racing-editor");
    let script: String = format!(
        "#!/bin/sh\nsed -i '1s/^/typed: /' \"$1\"\nenv -u GIT_DIR -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY -u GIT_WORK_TREE -u GIT_COMMON_DIR \
         GIT_AUTHOR_NAME=other GIT_AUTHOR_EMAIL=other@example.invalid GIT_COMMITTER_NAME=other GIT_COMMITTER_EMAIL=other@example.invalid \
         /usr/bin/git -C '{}' commit --quiet --allow-empty --message='moved by someone'\n",
        repo.display()
    );
    executable(racing.as_path(), script.as_bytes());
    git(&fixture, &repo, &["config", "core.editor", racing.to_str().expect("UTF-8")]);
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--", "a.txt"]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("moved while this commit was being prepared"), "{stderr}");
    assert_eq!(finished(&fixture), before, "nothing landed");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "moved by someone");
    assert_eq!(query(&fixture, &repo, &["log", "--format=%s", "--grep=typed"]), "", "the typed commit is not on the branch");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A policy that rewrites a file keeps the commit in the transaction, whose preparation installs the rewrite into the
///  worktree: the service is not used, and the commit still lands with the editor's message.
#[test]
fn a_policy_that_rewrites_a_file_keeps_the_editor_commit_in_the_transaction() {
    let fixture: Fixture = fixture("service-native-rewrite");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let name: String = format!("native-rewrite-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    write(&repo, "a.txt", "base\nno final newline");
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-e", "-m", "seed", "--", "a.txt"]);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(finished(&fixture), before, "the service was not used");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "edited: seed");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

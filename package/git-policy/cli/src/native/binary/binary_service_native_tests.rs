//! What:
//!  The commits real Git builds for the service (an editor,
//! a signature,
//! a configuration or an option the service does
//!  not embed),
//! compared field by field with the commit the transaction makes of the same input,
//! and the cases where
//!  nothing may land.
//! Why:
//!  The service lands a commit Git built by moving the branch to it only while the branch is still where the commit was
//!  made.
//! A difference in a stored field,
//! in what is left behind,
//! or a landing after an abort or a lost race would be a
//!  silent change of what an editor or a signature means.

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, query, start, through_service, through_service_with};
use super::support::{Fixture, direct, executable, fixture, git, repository};
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

/// The stored fields that must agree:
/// tree,
/// whole message,
/// author,
/// author date,
/// committer,
/// parents,
/// and whether the
///  commit is signed.
/// A signature's bytes are not compared,
/// because a signer may put its own time in them.
fn stored(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let formats: [&str; 6] = ["%T", "%B", "%an <%ae>", "%ad", "%cn <%ce>", "%P"];
    let mut fields: Vec<String> =
        formats.map(|format| { return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str()]); }).to_vec();
    fields.push(git_text(fixture, repo, &["cat-file", "commit", "HEAD"]).contains("\ngpgsig ").to_string());
    return fields;
}

/// What a commit left behind:
/// the status,
/// the staged difference from the commit and how many commits the branch has.
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

/// A repository with two tracked files and an editor that puts `edited: ` in front of the first line of the message,
/// a
///  signer that makes a placeholder signature,
/// and a worktree edit of one file ready to commit.
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

fn edit_a_and_stage(repo: &Path) {
    edit_a(repo);
    let output = std::process::Command::new("/usr/bin/git").current_dir(repo).args(["add", "a.txt"]).output().expect("add");
    assert!(output.status.success());
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

/// One command line,
/// the state it starts from,
/// and whether the author date is a field the line keeps.
struct Case {
    name: &'static str,
    command: &'static [&'static str],
    prepare: fn(&Path),
    date_kept: bool,
}

fn case(name: &'static str, command: &'static [&'static str], prepare: fn(&Path)) -> Case {
    return Case { name, command, prepare, date_kept: false };
}

/// Every line real Git builds for the service gives the commit the transaction gives,
/// leaves the repository as it leaves
///  it,
/// and the service (not the transaction) landed it.
#[test]
fn commits_git_builds_for_the_service_equal_the_transaction_commit() {
    let fixture: Fixture = fixture("service-native");
    let (repo, start_commit) = repository_with_programs(&fixture);
    let cases: Vec<Case> = vec![
        case("editor, named path", &["commit", "--", "a.txt"], edit_a),
        case("editor, seeded message", &["commit", "-e", "-m", "seed", "--", "a.txt"], edit_a),
        case("editor, index", &["commit", "--no-enforce-only"], edit_a_and_stage),
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
                direct(&fixture).current_dir(&repo).args(entry.command.iter().copied().filter(|word| { return *word != "--no-enforce-only"; })).output().expect("transaction")
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

/// The signature's verdict as Git reports it:
/// the status letter,
/// the signing key and the signer.
fn verdict(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return ["%G?", "%GK", "%GS"].map(|format| { return git_text(fixture, repo, &["log", "-1", format!("--format={format}").as_str()]); }).to_vec();
}

/// The way this machine signs:
/// SSH signing with a key file and an allowed-signers file,
/// set by `commit.gpgsign` alone.
/// A
///  throwaway key made in the fixture signs a commit that Git builds,
/// the service lands,
/// and `git verify-commit` accepts,
///  with the same verdict and the same stored fields as the transaction's signed commit.
#[test]
fn an_ssh_signed_commit_lands_through_the_service_and_verifies() {
    let fixture: Fixture = fixture("service-native-ssh");
    let (repo, start_commit) = repository_with_programs(&fixture);
    let key: PathBuf = fixture.root.join("signing-key");
    let made = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "fixture", "-f"])
        .arg(&key)
        .output()
        .expect("ssh-keygen starts");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let public: String = std::fs::read_to_string(key.with_extension("pub")).expect("public key");
    let allowed: PathBuf = fixture.root.join("allowed-signers");
    std::fs::write(&allowed, format!("fixture@example.invalid {}", public.trim())).expect("allowed signers");
    for (setting, value) in [
        ("gpg.format", "ssh"),
        ("user.signingkey", key.to_str().expect("UTF-8")),
        ("gpg.ssh.allowedsignersfile", allowed.to_str().expect("UTF-8")),
        ("commit.gpgsign", "true"),
    ] {
        git(&fixture, &repo, &["config", setting, value]);
    }
    let name: String = format!("native-ssh-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let mut outcomes: Vec<(Vec<String>, Vec<String>, Vec<String>)> = Vec::new();
    for through in [true, false] {
        git(&fixture, &repo, &["reset", "--hard", "--quiet", start_commit.as_str()]);
        edit_a(&repo);
        let before: usize = finished(&fixture);
        let output: Output = if through {
            through_service(&fixture, &repo, name.as_str(), &["commit", "-m", "signed by ssh", "--", "a.txt"])
        } else {
            direct(&fixture).current_dir(&repo).args(["commit", "-m", "signed by ssh", "--", "a.txt"]).output().expect("transaction")
        };
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        if through {
            assert_eq!(finished(&fixture), before + 1, "the service landed it");
        }
        git(&fixture, &repo, &["verify-commit", "HEAD"]);
        assert!(
            git_text(&fixture, &repo, &["cat-file", "commit", "HEAD"]).contains("\ngpgsig -----BEGIN SSH SIGNATURE-----"),
            "the signature is Git's own SSH signature"
        );
        outcomes.push((stored(&fixture, &repo), left_behind(&fixture, &repo), verdict(&fixture, &repo)));
    }
    assert_eq!(outcomes[0].2, outcomes[1].2, "the verdict");
    assert_eq!(outcomes[0].2[0].trim(), "G", "a good signature by a known signer");
    assert_eq!(outcomes[0].1, outcomes[1].1, "what is left behind");
    for (index, (left, right)) in outcomes[0].0.iter().zip(&outcomes[1].0).enumerate() {
        if index != 3 {
            assert_eq!(left, right, "stored field {index}");
        }
    }
    assert_eq!(outcomes[0].0[6], "true", "signed");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Turns on SSH signing for every commit of `repo` with a throwaway key made in the fixture,
/// and sends the signer through a
///  stand-in for `ssh-keygen` that first lets `race` happen.
/// It counts its own calls in the returned file.
fn ssh_signing_through(fixture: &Fixture, repo: &Path, race: &str) -> PathBuf {
    let key: PathBuf = fixture.root.join("signing-key");
    let made = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "fixture", "-f"])
        .arg(&key)
        .output()
        .expect("ssh-keygen starts");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let public: String = std::fs::read_to_string(key.with_extension("pub")).expect("public key");
    let allowed: PathBuf = fixture.root.join("allowed-signers");
    std::fs::write(&allowed, format!("fixture@example.invalid {}", public.trim())).expect("allowed signers");
    let calls: PathBuf = fixture.root.join("signer-calls");
    let signer: PathBuf = fixture.root.join("racing-signer");
    let script: String = format!("#!/bin/sh\n{race}\necho call >> '{}'\nexec /usr/bin/ssh-keygen \"$@\"\n", calls.display());
    executable(signer.as_path(), script.as_bytes());
    for (setting, value) in [
        ("gpg.format", "ssh"),
        ("gpg.ssh.program", signer.to_str().expect("UTF-8")),
        ("user.signingkey", key.to_str().expect("UTF-8")),
        ("gpg.ssh.allowedsignersfile", allowed.to_str().expect("UTF-8")),
        ("commit.gpgsign", "true"),
    ] {
        git(fixture, repo, &["config", setting, value]);
    }
    return calls;
}

/// The shell line that makes another commit on the branch,
/// unless the signer was started by that very commit,
/// and,
/// when a
///  marker is given,
/// only the first time.
fn racing_commit(repo: &Path, only_once_marker: Option<&Path>) -> String {
    let guard: String = match only_once_marker {
        Some(marker) => format!("[ -z \"$RACING\" ] && [ ! -e '{0}' ] && : > '{0}' && ", marker.display()),
        None => String::from("[ -z \"$RACING\" ] && "),
    };
    return format!(
        "{guard}RACING=1 env -u GIT_DIR -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY -u GIT_WORK_TREE -u GIT_COMMON_DIR \
         GIT_AUTHOR_NAME=other GIT_AUTHOR_EMAIL=fixture@example.invalid GIT_COMMITTER_NAME=other GIT_COMMITTER_EMAIL=fixture@example.invalid \
         /usr/bin/git -C '{}' commit --quiet --allow-empty --message=raced",
        repo.display()
    );
}

/// The shell line that makes another commit on the branch the first `times` times it runs,
/// counted in `marker`,
/// unless the signer was started by that very commit.
fn racing_commit_times(repo: &Path, marker: &Path, times: usize) -> String {
    let guard: String = format!("[ -z \"$RACING\" ] && [ \"$(cat '{0}' 2>/dev/null | wc -l)\" -lt {times} ] && echo race >> '{0}' && ", marker.display());
    return racing_commit(repo, None).replacen("[ -z \"$RACING\" ] && ", guard.as_str(), 1);
}

/// The number the plain failure names:
/// "changed N times".
fn builds_named(stderr: &str) -> usize {
    let after: &str = stderr.split("changed ").nth(1).expect("the failure names the builds");
    return after.split(' ').next().expect("a number").parse::<usize>().expect("a number of builds");
}

/// The ceiling the controls give the rebuilding,
/// in seconds:
/// far below the measured twenty minutes,
/// so a branch that never stops moving ends the control.
const CONTROL_REBUILD_CEILING: &str = "3";

fn lines_in(path: &Path) -> usize {
    return std::fs::read_to_string(path).map_or(0, |text| { return text.lines().count(); });
}

/// A signed commit with no editor is built again,
/// and signed again,
/// on the new tip when another commit lands on the branch
///  while it is being signed:
/// the person sees success,
/// the commit sits on the other commit,
/// and it is signed.
#[test]
fn a_signed_commit_is_built_again_on_the_new_tip_when_the_branch_moves() {
    let fixture: Fixture = fixture("service-native-rebuild");
    let (repo, start_commit) = repository_with_programs(&fixture);
    let marker: PathBuf = fixture.root.join("raced-once");
    let calls: PathBuf = ssh_signing_through(&fixture, &repo, racing_commit(&repo, Some(marker.as_path())).as_str());
    let name: String = format!("native-rebuild-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-m", "signed and moved", "--", "a.txt"]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    assert_eq!(finished(&fixture), before + 1, "the service landed it once");
    assert_eq!(lines_in(&calls), 3, "the first build, the other commit's signature, and the build made again");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "signed and moved");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s", "HEAD^"]), "raced", "it sits on the other commit");
    assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD~2"]), start_commit);
    assert!(git_text(&fixture, &repo, &["cat-file", "commit", "HEAD"]).contains("\ngpgsig -----BEGIN SSH SIGNATURE-----"), "signed again");
    assert!(String::from_utf8_lossy(&output.stdout).contains("signed and moved"), "one summary names the commit that landed: {}", String::from_utf8_lossy(&output.stdout));
    assert_eq!(query(&fixture, &repo, &["show", "HEAD:a.txt"]), "base\nedited");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The commit name a summary line `[<branch> <name>] <subject>` carries.
fn summary_name(line: &str) -> String {
    return line.split_whitespace().nth(1).expect("a name after the branch").trim_end_matches(']').to_string();
}

/// The note the wrapper adds,
/// on standard error,
/// after a commit Git built was built again because the branch moved:
/// it names the
///  builds that did not land and the one that did,
/// by the names Git's own summary lines carry.
fn rebuilt_note(unlanded: &[String], landed: &str) -> String {
    return format!("monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; {} did not land, {landed} did.", unlanded.join(" and "));
}

/// A commit Git builds again because the branch moved prints Git's summary for each build,
/// as Git prints them (the owner's
///  decision:
/// the wrapper adds no `--quiet`,
/// "Letting native git emit whatever it wants to is expected."),
/// and the last one
///  names the commit that landed.
/// The first names a build that never landed,
/// and the wrapper adds one note on standard error that
///  names both (owner decision:
/// "Add a wrapper note (Recommended)");
/// Git's own output is not touched.
#[test]
fn a_commit_built_again_prints_gits_summary_for_each_build_and_the_last_names_the_landed_commit() {
    let fixture: Fixture = fixture("service-native-one-summary");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let marker: PathBuf = fixture.root.join("raced-once");
    let _calls: PathBuf = ssh_signing_through(&fixture, &repo, racing_commit(&repo, Some(marker.as_path())).as_str());
    let name: String = format!("native-one-summary-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    edit_a(&repo);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-m", "signed and moved", "--", "a.txt"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let summaries: Vec<&str> = stdout.lines().filter(|line| { return line.starts_with('['); }).collect();
    assert_eq!(summaries.len(), 2, "Git's summary of the build that lost the race, then of the build that landed: {stdout}");
    let landed: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    assert!(landed.starts_with(summary_name(summaries[1]).as_str()), "the last line names the landed commit {landed}: {stdout}");
    assert!(!landed.starts_with(summary_name(summaries[0]).as_str()), "the first line names the build that never landed: {stdout}");
    assert!(summaries.iter().all(|line| { return line.contains("signed and moved"); }), "{stdout}");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    let note: String = rebuilt_note(&[summary_name(summaries[0])], summary_name(summaries[1]).as_str());
    assert_eq!(stderr.lines().filter(|line| { return line.contains("so it was built again"); }).collect::<Vec<&str>>(), vec![note.as_str()], "{stderr}");
    assert!(!stdout.contains("so it was built again"), "Git's output is not touched: {stdout}");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A branch that moves more often than the five builds a count once allowed still lets the commit land,
/// on the last tip,
/// signed again each time,
/// and each rebuild says so on standard error:
/// nothing conflicted,
/// so the commit is not refused (the owner's rule "Commits should never fail").
#[test]
fn a_branch_that_moves_more_often_than_a_count_allowed_still_lands_the_commit() {
    let fixture: Fixture = fixture("service-native-rebuild-many");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let marker: PathBuf = fixture.root.join("races");
    let moves: usize = 7;
    let calls: PathBuf = ssh_signing_through(&fixture, &repo, racing_commit_times(&repo, marker.as_path(), moves).as_str());
    let name: String = format!("native-rebuild-many-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-m", "lands after many moves", "--", "a.txt"]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    assert_eq!(finished(&fixture), before + 1, "the service landed it once");
    assert_eq!(lines_in(&calls), 2 * moves + 1, "a build and the other commit's signature for each move, and the build that landed");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "lands after many moves");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s", "HEAD^"]), "raced", "it sits on the last other commit");
    assert_eq!(stderr.lines().filter(|line| { return line.contains("moved while this commit was being built"); }).count(), moves, "each rebuild is said: {stderr}");
    assert_eq!(query(&fixture, &repo, &["show", "HEAD:a.txt"]), "base\nedited");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A branch that moves every time ends in the plain failure when the rebuilding reaches its ceiling,
/// naming the builds made,
/// and nothing lands.
#[test]
fn a_branch_that_keeps_moving_stops_the_rebuilds_at_the_ceiling() {
    let fixture: Fixture = fixture("service-native-rebuild-bound");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let calls: PathBuf = ssh_signing_through(&fixture, &repo, racing_commit(&repo, None).as_str());
    let name: String = format!("native-rebuild-bound-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service_with(&fixture, &repo, name.as_str(), &["commit", "-m", "never lands", "--", "a.txt"], &[(git_policy_cli::service_commit_rebuild_ceiling::REBUILD_CEILING_VARIABLE, CONTROL_REBUILD_CEILING)]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let builds: usize = builds_named(stderr.as_str());
    assert!(builds >= 2, "the ceiling is not a count of one build: {stderr}");
    assert_eq!(finished(&fixture), before, "nothing landed");
    assert_eq!(lines_in(&calls), 2 * builds, "a build and the other commit's signature, for each attempt");
    assert_eq!(query(&fixture, &repo, &["log", "--format=%s", "--grep=never lands"]), "");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// An amend is never built again on a new tip,
/// signed or not:
/// the person asked to replace the commit they saw.
#[test]
fn a_signed_amend_is_not_built_again_when_the_branch_moves() {
    let fixture: Fixture = fixture("service-native-rebuild-amend");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let calls: PathBuf = ssh_signing_through(&fixture, &repo, racing_commit(&repo, None).as_str());
    let name: String = format!("native-rebuild-amend-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--amend", "--no-edit", "--cleanup=scissors", "--", "a.txt"]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("moved while this commit was being prepared"), "{stderr}");
    assert_eq!(finished(&fixture), before);
    assert_eq!(lines_in(&calls), 2, "one build, and the other commit's signature");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// An editor that leaves an empty message,
/// or fails,
/// ends the commit with Git's status;
/// nothing lands,
/// nothing moves,
/// and
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
        assert!(!String::from_utf8_lossy(&output.stderr).contains("commit service"), "{label}: the service was never asked: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(finished(&fixture), before, "{label}: nothing landed");
        assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD"]), start_commit, "{label}: the branch did not move");
        let scratch: Vec<String> = super::support::left_behind(repo.as_path());
        assert!(scratch.is_empty(), "{label}: left {scratch:?}");
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// After the editor has run the message is fixed:
/// when another commit lands on the branch while the editor is open,
/// Git builds the
///  commit again on the new tip from `-F <the typed message>`,
/// with no editor and no person,
/// and it lands on top of the other commit.
#[test]
fn a_branch_that_moves_while_the_editor_is_open_is_built_again_from_the_typed_message() {
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
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert_eq!(finished(&fixture), before + 1, "it landed once");
    let stdout: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let names: Vec<String> = stdout.lines().filter(|line| { return line.starts_with('['); }).map(summary_name).collect();
    assert_eq!(names.len(), 2, "Git's summary of each build: {stdout}");
    let note: String = rebuilt_note(&names[..1], names[1].as_str());
    assert_eq!(stderr.lines().filter(|line| { return line.contains("so it was built again"); }).collect::<Vec<&str>>(), vec![note.as_str()], "{stderr}");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "typed:", "the typed message is the commit's");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s", "HEAD^"]), "moved by someone", "it sits on the other commit");
    assert_eq!(query(&fixture, &repo, &["show", "HEAD:a.txt"]), "base\nedited");
    let leftover: Vec<String> = super::support::left_behind(repo.as_path());
    assert!(leftover.is_empty(), "nothing is left behind: {leftover:?}");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The rebuilds from the typed message stop at the ceiling:
/// while the editor's message is fixed,
/// a branch that moves on every
///  build ends in the plain failure that names the file the message was saved in,
/// and `git commit -F <file>` makes the commit
///  afterwards.
#[test]
fn a_typed_message_is_saved_only_when_the_branch_never_stops_moving() {
    let fixture: Fixture = fixture("service-native-moved-bound");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let calls: PathBuf = ssh_signing_through(&fixture, &repo, racing_commit(&repo, None).as_str());
    let name: String = format!("native-moved-bound-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let typing: PathBuf = fixture.root.join("typing-only-editor");
    executable(typing.as_path(), b"#!/bin/sh\nsed -i '1s/^/typed: /' \"$1\"\n");
    git(&fixture, &repo, &["config", "core.editor", typing.to_str().expect("UTF-8")]);
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service_with(&fixture, &repo, name.as_str(), &["commit", "-e", "-m", "seed", "--", "a.txt"], &[(git_policy_cli::service_commit_rebuild_ceiling::REBUILD_CEILING_VARIABLE, CONTROL_REBUILD_CEILING)]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("moved while this commit was being prepared"), "{stderr}");
    assert_eq!(finished(&fixture), before, "nothing landed");
    let builds: usize = stderr.matches("moved while this commit was being built").count() + 1;
    assert_eq!(lines_in(&calls), 2 * builds, "a build and the other commit's signature, for each attempt: {stderr}");
    let saved: PathBuf = std::fs::read_dir(repo.join(".git"))
        .expect("git directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .find(|path| { return path.file_name().is_some_and(|file| { return file.to_string_lossy().starts_with("monochromatic-git-policy-cli-saved-message-"); }); })
        .expect("the typed message was saved");
    assert!(stderr.contains(&format!("git commit -F {}", saved.display())), "{stderr}");
    assert!(std::fs::read_to_string(&saved).expect("saved message").starts_with("typed: seed"), "the typed text is what was saved");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A policy that rewrites a file keeps its preparation in the wrapper,
/// which installs the rewrite into the worktree after
///  the landing:
/// the service lands the commit,
/// and it carries the editor's message.
#[test]
fn a_policy_that_rewrites_a_file_lands_the_editor_commit_through_the_service_and_installs_the_rewrite() {
    let fixture: Fixture = fixture("service-native-rewrite");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    let name: String = format!("native-rewrite-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    write(&repo, "a.txt", "base\nno final newline");
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-e", "-m", "seed", "--", "a.txt"]);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(finished(&fixture), before + 1, "the service landed it");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("worktree copy"), b"base\nno final newline\n", "the rewrite reached the worktree");
    assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s"]), "edited: seed");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A worktree that switches to another branch while the editor is open fails the commit the way native Git does,
/// with nothing
///  changed on either branch,
/// and the typed message is kept in a file:
/// the commit is never built again on the branch it
///  switched to.
#[test]
fn a_switch_to_another_branch_while_the_editor_is_open_fails_the_commit_and_keeps_the_message() {
    let fixture: Fixture = fixture("service-native-switch");
    let (repo, _start_commit) = repository_with_programs(&fixture);
    git(&fixture, &repo, &["switch", "--quiet", "--create", "side"]);
    git(&fixture, &repo, &["commit", "--quiet", "--allow-empty", "--message=side work"]);
    git(&fixture, &repo, &["switch", "--quiet", "main"]);
    let main_tip: String = query(&fixture, &repo, &["rev-parse", "main"]);
    let side_tip: String = query(&fixture, &repo, &["rev-parse", "side"]);
    let name: String = format!("native-switch-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let switching: PathBuf = fixture.root.join("switching-editor");
    let script: String = format!(
        "#!/bin/sh\nsed -i '1s/^/typed: /' \"$1\"\nenv -u GIT_DIR -u GIT_INDEX_FILE -u GIT_OBJECT_DIRECTORY -u GIT_WORK_TREE -u GIT_COMMON_DIR \
         /usr/bin/git -C '{}' switch --quiet side\n",
        repo.display()
    );
    executable(switching.as_path(), script.as_bytes());
    git(&fixture, &repo, &["config", "core.editor", switching.to_str().expect("UTF-8")]);
    edit_a(&repo);
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "-e", "-m", "seed", "--", "a.txt"]);
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_ne!(output.status.code(), Some(0), "the commit fails: {stderr}");
    assert_eq!(finished(&fixture), before, "nothing landed");
    assert_eq!(query(&fixture, &repo, &["rev-parse", "main"]), main_tip, "the branch the commit was made on did not move");
    assert_eq!(query(&fixture, &repo, &["rev-parse", "side"]), side_tip, "the branch switched to did not move");
    assert_eq!(query(&fixture, &repo, &["symbolic-ref", "HEAD"]), "refs/heads/side", "the worktree is where the person switched it");
    let saved: PathBuf = std::fs::read_dir(repo.join(".git"))
        .expect("git directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .find(|path| { return path.file_name().is_some_and(|file| { return file.to_string_lossy().starts_with("monochromatic-git-policy-cli-saved-message-"); }); })
        .expect("the typed message was saved");
    assert!(std::fs::read_to_string(&saved).expect("saved message").starts_with("typed: seed"), "the typed text is what was saved");
    assert!(stderr.contains(&saved.display().to_string()), "the message names the file: {stderr}");
    let scratch: Vec<String> = super::support::left_behind(repo.as_path());
    assert!(scratch.is_empty(), "left {scratch:?}");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

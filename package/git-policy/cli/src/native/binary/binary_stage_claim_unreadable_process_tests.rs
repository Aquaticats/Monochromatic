//! What:
//!  The stage claim of a worktree add whose owner's process entry the platform will not report on,
//!  through the built executable:
//!  `git worktree add` makes its worktree and exits 0,
//!  the claim and its stage stay,
//!  and past the patience the command says which record it is and the two commands that remove it.
//! Why:
//!  Every `git worktree` command runs the recovery of the copy transactions first, so an error there failed a command that
//!  was never asked about the claim: a `git worktree add` stopped and made no worktree while the claim proved nothing about a
//!  live owner (the review of a303cb638f, finding 2). Reading such a claim as a dead owner's while it lived was the other
//!  direction, and the decision of 2026-11-16 keeps it and tells the person.
//!  A process entry the platform refuses cannot be made on demand on this host, so the control makes one the way the
//!  platform does: in a private user, mount and PID namespace, a file of garbage is bind-mounted over the entry of a
//!  process the control started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // unshare --user --map-root-user --mount --pid --fork --mount-proc bash -c 'sleep 300 & mount --bind garbage /proc/$!/stat; git worktree add ...'
//! ```

/// The shared fixtures and the bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, remove, repository};
/// `OsStr` for the fixture's names.
use std::ffi::OsStr;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Whether a program is on the search path,
///  so a control that needs it can say plainly why it did not run.
/// Why:
///   `unshare` is not on every host the suite runs on, and a control that cannot enter its namespace must not fail as a
///  defect of the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const onPath = (program) => (process.env.PATH ?? '').split(':').some((d) => existsSync(join(d, program)));
/// ```
fn on_path(program: &str) -> bool {
    return std::env::var_os("PATH").is_some_and(|paths| {
        return std::env::split_paths(&paths).any(|directory| { return directory.join(program).is_file(); });
    });
}

/// What:
///  Run one shell script inside a private user, mount and PID namespace,
///  with the fixture's `bin/git` as the wrapper under test.
/// Why:
///   The garbage record is mounted over `/proc/<pid>/stat`, which only a private mount namespace may do; the stand-in owner
///  and the mount are the script's own, and the script kills the owner it started by the number it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = spawnSync('unshare', ['--user','--map-root-user','--mount','--pid','--fork','--mount-proc','bash','-c',script], { cwd, env });
/// ```
fn in_a_private_namespace(fixture: &Fixture, script: &str) -> Observed {
    let mut command: std::process::Command = std::process::Command::new("unshare");
    command.args(["--user", "--map-root-user", "--mount", "--pid", "--fork", "--mount-proc", "bash", "-c"]).arg(script);
    super::support::hermetic(&mut command, fixture, super::support::wrapper_first_path(fixture).as_os_str());
    let ran: std::process::Output = command.output().expect("unshare runs");
    return Observed { code: ran.status.code(), stdout: ran.stdout, stderr: ran.stderr };
}

/// What:
///  A repository with one commit and the journal root the stage claims live in.
/// Why:
///  The claim is written where every command of the worktree reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(fixture, 'repo'); git(fixture, repo, ['init']); ...
/// ```
fn repository_with_a_journal_root(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=first"]);
    let parent: PathBuf = repo.join(".git/monochromatic-git-policy-cli-worktree-copy");
    std::fs::create_dir_all(parent.join("v1")).expect("the journal root");
    std::fs::set_permissions(parent.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("the journal parent");
    std::fs::set_permissions(parent.join("v1").as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("the journal root");
    return repo;
}

/// What:
///  The number a stand-in owner's claim names is the one whose entry is made unreadable,
///  and `git worktree add` makes its worktree and exits 0.
/// Why:
///  Read as an error the claim stopped the command (the review's finding 2); read as a dead owner's it took a live command's
///  stage. The decision of 2026-11-16 keeps the claim, says nothing inside the patience, and lets the add go on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(addYoung.code).toBe(0); expect(exists(worktree)).toBe(true); expect(exists(claim)).toBe(true); expect(stderr).toBe('');
/// expect(addAged.code).toBe(0); expect(stderr).toContain('rm --'); expect(stderr).toContain('rm --recursive --');
/// ```
#[cfg(target_os = "linux")]
#[test]
fn an_add_beside_a_claim_whose_process_entry_cannot_be_read_makes_its_worktree() {
    if !on_path("unshare") {
        eprintln!("skipped: unshare is not installed, so no namespace was entered");
        return;
    }
    if !on_path("mount") {
        eprintln!("skipped: mount is not installed, so no process entry can be made unreadable");
        return;
    }
    let fixture: Fixture = fixture("stage-claim-unreadable-process");
    let repo: PathBuf = repository_with_a_journal_root(&fixture);
    let garbage: PathBuf = fixture.root.join("garbage-stat.txt");
    std::fs::write(garbage.as_path(), b"this is not a stat record\n").expect("the garbage record");
    let claim: PathBuf = repo.join(".git/monochromatic-git-policy-cli-worktree-copy/v1/fixture-claim.stage");
    let container: PathBuf = fixture.root.join(".cli-git-worktree-copy-fixture");
    let young_worktree: PathBuf = fixture.root.join("wt-young");
    let aged_worktree: PathBuf = fixture.root.join("wt-aged");
    // One stand-in owner for both arms, started inside the namespace, and its process entry made unreadable there.
    let script: String = format!(
        "set -u\ncd '{repo}' || exit 2\nsleep 300 &\nowner=$!\ngarbage='{garbage}'\nmount --bind \"$garbage\" \"/proc/$owner/stat\" || {{ echo 'the bind mount was refused'; kill \"$owner\"; exit 3; }}\n\
         head -c 40 \"/proc/$owner/stat\"; echo\n\
         printf '%s\\ncontainerHex=%s\\nowner=%s~linux:1~%s\\n' '{container}' '{container_hex}' \"$owner\" \"$(readlink /proc/self/ns/pid)\" > '{claim}'\n\
         chmod 600 '{claim}'\n\
         '{git}' worktree add '{young_worktree}' > '{fixture}/young.out' 2> '{fixture}/young.err'; echo \"young exit $?\"\n\
         touch -d '20 minutes ago' '{claim}'\n\
         '{git}' worktree add '{aged_worktree}' > '{fixture}/aged.out' 2> '{fixture}/aged.err'; echo \"aged exit $?\"\n\
         kill \"$owner\"; wait \"$owner\" 2> /dev/null\n",
        garbage = garbage.display(),
        container = container.display(),
        container_hex = container.to_string_lossy().bytes().map(|byte| { return format!("{byte:02x}"); }).collect::<String>(),
        claim = claim.display(),
        git = fixture.root.join("bin/git").display(),
        young_worktree = young_worktree.display(),
        aged_worktree = aged_worktree.display(),
        fixture = fixture.root.display(),
        repo = repo.display(),
    );
    let ran: Observed = in_a_private_namespace(&fixture, script.as_str());
    // The script's own lines are its standard output; the wrapper's diagnostics are its standard error.
    let said: String = format!(
        "{}\n{}",
        String::from_utf8_lossy(ran.stdout.as_slice()),
        String::from_utf8_lossy(ran.stderr.as_slice())
    );
    if !ran.code.unwrap_or(1).eq(&0) && said.contains("unshare:") {
        eprintln!("skipped: this system does not allow the namespace: {}", said.trim());
        remove(&fixture);
        return;
    }
    let young: String = std::fs::read_to_string(fixture.root.join("young.err")).unwrap_or_default();
    let aged: String = std::fs::read_to_string(fixture.root.join("aged.err")).unwrap_or_default();
    assert!(said.contains("young exit 0") && said.contains("aged exit 0"), "both adds exit 0: {said}\n{young}\n{aged}");
    assert!(young_worktree.is_dir(), "the worktree of the young claim was made: {said}\n{young}");
    assert!(aged_worktree.is_dir(), "the worktree of the aged claim was made: {said}\n{aged}");
    assert!(claim.is_file(), "the claim of a command that may run stays");
    assert!(!young.contains("rm --"), "inside the patience nothing is said about the claim: {young}");
    assert!(
        aged.contains(claim.to_string_lossy().as_ref()) && aged.contains("minutes old") && aged.contains("rm -- "),
        "past the patience the words name the claim, its age and the command that removes it: {aged}"
    );
    assert!(
        aged.contains("rm --recursive -- "),
        "and they name the command that removes the stage the claim leaves behind: {aged}"
    );
    remove(&fixture);
}

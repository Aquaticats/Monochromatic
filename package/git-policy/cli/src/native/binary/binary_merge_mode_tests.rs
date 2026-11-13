//! What:
//!  The mode a merge leaves on a file whose content it changes,
//!  for every shape of that file and of the person's own mode on it,
//!  compared with the mode real Git writes for the same entry.
//! Why:
//!  The owner decided on 2026-11-12T12:26:24-05:00 that this command writes the mode real Git would write,
//!  "As real Git,
//!  both cases",
//!  because the wrapper does not go beyond Git to keep a mode by itself and a person gets one behaviour they can
//!  predict
//!  (`doc/handover/cli-git-rust-implementation.md`;
//!  `SPEC.md`,
//!  "Merges and commits never fail").
//!  Real Git writes the mode its entry and the process's umask give,
//!  and it writes nothing at a path its update does not touch,
//!  so a mode a person set on a path the merge changes goes with the content,
//!  and the mode they set on a path the merge leaves alone stays.
//!  Every control compares the file's bits with what `git checkout-index` writes for the entry the index ends with,
//!  under the umask the control itself sets,
//!  rather than with a number,
//!  because a repository that records file modes writes `100644` as `0644` only where the umask allows it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! chmod('victim.txt', 0o600); await wrapped(['merge', '--no-edit', 'feature']); expect(mode('victim.txt')).toBe(gitWritesFor('victim.txt'));
//! ```

use super::commit::COMMIT_BOUND_SECONDS;
use super::support::{Fixture, Observed, bounded_for, fixture, git, observe, remove, repository, spawn_owned, wrapper_first_path};
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};

/// The environment variable that marks the child `ran_in_child` starts.
const CHILD_VARIABLE: &str = "CLI_GIT_MERGE_MODE_CHILD";

/// What:
///  This test process's own executable,
///  for the child process the umask controls start from the test binary.
/// Why:
///  A control starts the test binary,
///  never the wrapper:
///  the ban on `std::env::current_exe` (`clippy.toml`) is about the wrapper starting itself,
///  so this is this module's one place the lint is excused.
#[allow(clippy::disallowed_methods, reason = "identifies the running test binary so a control can start a helper copy of it under another umask; the wrapper is never started this way")]
fn the_test_binary() -> std::io::Result<PathBuf> {
    return std::env::current_exe();
}

/// What:
///  Whether this call ran the test in a child process under `mask`,
///  and asserted that the child passed;
///  `false` in the child itself,
///  where the body follows.
/// Why:
///  A umask is one value for a whole process,
///  and this harness runs many controls on the threads of one process,
///  so a control that set it would decide the mode every other control's writes get.
///  The child is the same test binary run with one test name,
///  which is why this answer is `true` in the parent:
///  the parent's assertion is the child's result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ranInChild = (module, name, mask) => { if (process.env.CHILD) return false; const child = spawn(self, ['--exact', `${module}::${name}`, '--nocapture'], { env: { CHILD: '1' }, preExec: () => umask(mask) }); const said = run(child); assert(said.includes('1 passed')); return true; };
/// ```
fn ran_in_child(module: &str, name: &str, mask: u32) -> bool {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        return false;
    }
    // The crate's own name is the first segment of `module_path!()` and is not part of the name libtest answers to,
    //  so it is dropped here as the library's own helper does.
    let path: String = format!("{}::{name}", module.split_once("::").map_or(module, |(_, rest)| { return rest }));
    let mut command: std::process::Command = std::process::Command::new(the_test_binary().expect("the test binary's own path"));
    command.args(["--exact", path.as_str(), "--nocapture"]).env(CHILD_VARIABLE, "1");
    // SAFETY: `umask` only sets the file creation mask of the child between its fork and its exec, and it is async-signal-safe.
    unsafe {
        command.pre_exec(move || {
            libc::umask(mask as libc::mode_t);
            return Ok(());
        });
    }
    let mut started: std::process::Command = command;
    started.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    let child: std::process::Child = spawn_owned(&mut started).expect("start the child test");
    let output: std::process::Output = child.wait_with_output().expect("the child test's end");
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    assert!(output.status.success(), "the child test {path} under umask {mask:o} failed:\n{said}\n{}", String::from_utf8_lossy(output.stderr.as_slice()));
    // A name that matches no test would end with success and run nothing.
    assert!(said.contains("running 1 test") && said.contains("1 passed"), "the child ran no test named {path}:\n{said}");
    return true;
}

/// What the merge does to the file the person set a mode on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheMergesChange = 'modify' | 'delete' | 'rename' | 'conflict' | 'touch-another-path';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WhatTheMergeDoes {
    /// `feature` writes new bytes into the file.
    Modify,
    /// `feature` deletes the file.
    Delete,
    /// `feature` moves the file to another name.
    Rename,
    /// Both sides write different bytes into the file,
    /// so the merge leaves markers in it.
    Conflict,
    /// `feature` writes another file,
    /// and this one is not the merge's business.
    AnotherPath,
}

/// The bytes the branch the person is on holds at the file.
const ON_THE_BRANCH: &str = "the branch's own content\n";

/// What `feature` writes into the file where the two sides conflict.
const ON_THE_FEATURE: &str = "the feature's own content\n";

/// The line the person adds to the file where the control needs an edit of their own.
const THE_PERSONS_LINE: &str = "the person's own line\n";

/// The name `feature` moves the file to.
const THE_NEW_NAME: &str = "moved-beside.txt";

/// The file the person set a mode on.
const THE_FILE: &str = "victim.txt";

/// What:
///  One repository whose branch and `feature` branch diverge,
///  with the file the merge changes at its base content,
///  and `main` one commit ahead so the merge is a true merge.
/// Why:
///  Every control needs the same shape and differs only in what the merge does to the file and in the person's own
///  state,
///  so the shape is built once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(name); write('victim.txt', ON_THE_BRANCH); write('other.txt', 'other\n'); commit('base'); branch('feature'); write('other.txt', 'other on main\n'); commit('main work'); switch('feature'); whatTheMergeDoes(...); commit('feature work'); switch('main');
/// ```
fn diverged(fixture: &Fixture, name: &str, what_the_merge_does: WhatTheMergeDoes) -> PathBuf {
    let repo: PathBuf = repository(fixture, name.as_ref());
    std::fs::write(repo.join(THE_FILE), ON_THE_BRANCH).expect(THE_FILE);
    std::fs::write(repo.join("other.txt"), b"other\n").expect("other.txt");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["branch", "feature"]);
    std::fs::write(repo.join("other.txt"), b"other on main\n").expect("other.txt");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "feature"]);
    match what_the_merge_does {
        WhatTheMergeDoes::Modify | WhatTheMergeDoes::Conflict => std::fs::write(repo.join(THE_FILE), ON_THE_FEATURE).expect(THE_FILE),
        WhatTheMergeDoes::Delete => {
            git(fixture, repo.as_path(), &["rm", "--quiet", "--", THE_FILE]);
        }
        WhatTheMergeDoes::Rename => {
            git(fixture, repo.as_path(), &["mv", "--", THE_FILE, THE_NEW_NAME]);
        }
        WhatTheMergeDoes::AnotherPath => {
            std::fs::write(repo.join("beside.txt"), b"beside, on the feature\n").expect("beside.txt");
        }
    }
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    if what_the_merge_does == WhatTheMergeDoes::Conflict {
        std::fs::write(repo.join(THE_FILE), ON_THE_FEATURE.replace("feature", "main")).expect(THE_FILE);
        git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main writes the file too"]);
    }
    return repo;
}

/// What:
///  A wrapped command of this control,
///  under a bound a merge over the person's work needs.
/// Why:
///  The shared hang guard bounds a command by its own progress and the harness's short bound is meant for one Git
///  start;
///  a merge carries files and writes records,
///  so it gets the same bound a wrapped commit gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = await run(wrapped(dir, ['merge', '--no-edit', 'feature']));
/// ```
fn merge(fixture: &Fixture, repo: &Path) -> Observed {
    let mut command: std::process::Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    // A runtime directory of the fixture's own, so no command here can find or leave anything in the session's.
    let runtime: PathBuf = fixture.root.join("runtime");
    if !runtime.exists() {
        std::fs::create_dir(runtime.as_path()).expect("the fixture's runtime directory");
    }
    command.env("XDG_RUNTIME_DIR", runtime);
    command.current_dir(repo).args(["merge", "--no-edit", "feature"]);
    return observe(&mut command, b"");
}

/// What:
///  The permission bits Git's own checkout writes for the entry the index holds at one path,
///  under this run's umask.
/// Why:
///  The mode Git writes is a fact about Git's own program,
///  which a control reads from Git rather than naming:
///  a repository that records file modes writes `100644` as `0644` only where the umask allows it,
///  and the person's own mode is what this command must not add to the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitWrites = (repo, path) => { git(['checkout-index', `--prefix=${elsewhere}/`, '--', path]); return mode(join(elsewhere, path)); };
/// ```
fn the_mode_git_writes(fixture: &Fixture, repo: &Path, path: &str) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;
    let elsewhere: PathBuf = repo.parent().map_or_else(|| { return repo.to_path_buf() }, |parent| { return parent.join("git-writes"); });
    std::fs::create_dir_all(elsewhere.as_path()).expect("a directory Git checks the entry out into");
    git(fixture, repo, &["checkout-index", format!("--prefix={}/", elsewhere.display()).as_str(), "--", path]);
    let written: u32 = std::fs::symlink_metadata(elsewhere.join(path)).expect("Git's own checkout of the entry").permissions().mode() & 0o7777;
    std::fs::remove_file(elsewhere.join(path)).expect("the checkout is read and removed");
    return written;
}

/// The bits one path's entry carries on the disk,
/// or nothing when nothing stands there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modeAt = (repo, path) => existsSync(join(repo, path)) ? lstatSync(join(repo, path)).mode & 0o7777 : undefined;
/// ```
fn mode_at(repo: &Path, path: &str) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt as _;
    return std::fs::symlink_metadata(repo.join(path)).ok().map(|state| { return state.permissions().mode() & 0o7777 });
}

/// The bits of `mode` that say the owner,
/// the group or anyone else may run the file.
const RUNNABLE_BITS: u32 = 0o111;

/// The name of the file the person staged an edit in,
/// at a path the merge never writes.
const THE_ASIDE_FILE: &str = "aside.txt";

/// What:
///  Stage an edit at a path the merge never touches.
/// Why:
///  Native Git refuses every merge that creates a commit while the index differs from `HEAD` at all,
///  so a staged edit is what makes this command complete the merge itself,
///  which is the one route where the carry writes the files of the merge's paths
///  (`merge_isolation.rs`,
///  "the staged-work check").
///  A control without it takes the route where Git's own update writes the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('aside.txt', 'the person staged a line\n'); run(['add', '--', 'aside.txt']);
/// ```
fn an_edit_the_person_staged(fixture: &Fixture, repo: &Path) {
    std::fs::write(repo.join(THE_ASIDE_FILE), b"the person staged a line\n").expect(THE_ASIDE_FILE);
    git(fixture, repo, &["add", "--", THE_ASIDE_FILE]);
}

/// What:
///  A merge that changes a file whose mode the person set writes the mode Git writes,
///  under the umask this control sets.
/// Why:
///  The mode a person set that Git does not record is not work a merge carries:
///  real Git writes the merge's content with the mode its entry and the umask give,
///  and this command does the same
///  (`SPEC.md`,
///  "Merges and commits never fail").
///
///  The person's staged edit at another path is what puts the merge on the route where this command completes it
///  itself and writes the file,
///  so the write this control judges is the carry's own.
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('victim.txt', 0o600); merge(); expect(mode('victim.txt')).toBe(gitWrites('victim.txt')); expect(read('victim.txt')).toBe(ON_THE_FEATURE);
/// ```
#[test]
fn a_merge_writes_the_mode_git_writes_over_the_private_mode_a_person_set() {
    if ran_in_child(module_path!(), "a_merge_writes_the_mode_git_writes_over_the_private_mode_a_person_set", 0o022) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-private");
    let repo: PathBuf = diverged(&fixture, "private", WhatTheMergeDoes::Modify);
    use std::os::unix::fs::PermissionsExt as _;
    an_edit_the_person_staged(&fixture, repo.as_path());
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge lands: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    assert_eq!(std::fs::read(repo.join(THE_FILE)).expect(THE_FILE), ON_THE_FEATURE.as_bytes(), "the merge's content stands at the path");
    let written: Option<u32> = mode_at(repo.as_path(), THE_FILE);
    let gits: u32 = the_mode_git_writes(&fixture, repo.as_path(), THE_FILE);
    assert_eq!(written, Some(gits), "the file ends at the mode Git writes ({gits:o}), not the {written:?} it held: a mode Git does not record is not a merge's to keep");
    remove(&fixture);
}

/// What:
///  The same under a private umask,
///  where Git's `0666` becomes `0600` and a command that named `0644` would be caught.
/// Why:
///  The mode Git writes is the one its entry and the umask give,
///  so a mask of `077` is where a write that names a finished mode instead of the one Git creates with shows itself.
///
///  The person's staged edit at another path is what puts the merge on the route where this command completes it
///  itself and writes the file,
///  so the write this control judges is the carry's own.
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('victim.txt', 0o600); merge(); expect(mode('victim.txt')).toBe(gitWrites('victim.txt')); expect(mode('victim.txt')).toBe(0o600);
/// ```
#[test]
fn a_merge_under_a_private_umask_writes_the_mode_git_writes() {
    if ran_in_child(module_path!(), "a_merge_under_a_private_umask_writes_the_mode_git_writes", 0o077) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-umask");
    let repo: PathBuf = diverged(&fixture, "umask", WhatTheMergeDoes::Modify);
    use std::os::unix::fs::PermissionsExt as _;
    an_edit_the_person_staged(&fixture, repo.as_path());
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o640)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge lands: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    let written: Option<u32> = mode_at(repo.as_path(), THE_FILE);
    let gits: u32 = the_mode_git_writes(&fixture, repo.as_path(), THE_FILE);
    assert_eq!(written, Some(gits), "the file ends at the mode Git writes under this mask ({gits:o}), not {written:?}");
    assert_eq!(written.map(|bits| { return bits & RUNNABLE_BITS }), Some(0), "a merge of a plain entry writes no executable bit, whatever mask is set");
    remove(&fixture);
}

/// What:
///  A mode that makes a plain entry runnable on the worktree alone is not a mode a merge keeps:
///  the entry Git records is not executable,
///  so the merge writes it as Git writes it.
/// Why:
///  `git status` compares the executable bit alone,
///  so a `chmod 0700` on a tracked `0644` file is work Git refuses to merge over and a merge writes the file with
///  its entry's bits;
///  this command carries the person's file for that reason and then writes nothing at the path,
///  so what stands there is Git's own mode
///  (`SPEC.md`,
///  "Merges and commits never fail").
///
///  The person's staged edit at another path is what puts the merge on the route where this command completes it
///  itself and writes the file,
///  so the write this control judges is the carry's own.
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('victim.txt', 0o700); merge(); expect(mode('victim.txt')).toBe(gitWrites('victim.txt')); expect(mode('victim.txt') & 0o111).toBe(0);
/// ```
#[test]
fn a_merge_of_a_plain_entry_writes_no_executable_bit_for_a_mode_the_worktree_alone_holds() {
    if ran_in_child(module_path!(), "a_merge_of_a_plain_entry_writes_no_executable_bit_for_a_mode_the_worktree_alone_holds", 0o077) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-runnable");
    let repo: PathBuf = diverged(&fixture, "runnable", WhatTheMergeDoes::Modify);
    use std::os::unix::fs::PermissionsExt as _;
    an_edit_the_person_staged(&fixture, repo.as_path());
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o700)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge lands: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    let written: Option<u32> = mode_at(repo.as_path(), THE_FILE);
    let gits: u32 = the_mode_git_writes(&fixture, repo.as_path(), THE_FILE);
    assert_eq!(written, Some(gits), "the file ends at the mode Git writes ({gits:o}), not the {written:?} the worktree held: Git records no such bit for this entry");
    assert_eq!(written.map(|bits| { return bits & RUNNABLE_BITS }), Some(0), "a plain entry stays unrunnable after a merge");
    remove(&fixture);
}

/// What:
///  A merge that moves a file away deletes the old path,
///  and the new path carries the mode Git writes.
/// Why:
///  Real Git deletes the file at the path its update removes,
///  whatever mode the person set on it,
///  and a command that kept the old file at their mode would leave a path Git does not have.
///
///  The person's staged edit at another path is what puts the merge on the route where this command completes it
///  itself and writes the file,
///  so the write this control judges is the carry's own.
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('victim.txt', 0o600); merge(); expect(existsSync('victim.txt')).toBe(false); expect(mode('moved-beside.txt')).toBe(gitWrites('moved-beside.txt'));
/// ```
#[test]
fn a_merge_that_moves_a_file_deletes_the_old_path_and_writes_the_mode_git_writes() {
    if ran_in_child(module_path!(), "a_merge_that_moves_a_file_deletes_the_old_path_and_writes_the_mode_git_writes", 0o022) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-rename");
    let repo: PathBuf = diverged(&fixture, "rename", WhatTheMergeDoes::Rename);
    use std::os::unix::fs::PermissionsExt as _;
    an_edit_the_person_staged(&fixture, repo.as_path());
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge lands: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    assert_eq!(mode_at(repo.as_path(), THE_FILE), None, "the path the merge moved away is gone, as it is with real Git");
    let written: Option<u32> = mode_at(repo.as_path(), THE_NEW_NAME);
    let gits: u32 = the_mode_git_writes(&fixture, repo.as_path(), THE_NEW_NAME);
    assert_eq!(written, Some(gits), "the moved file carries the mode Git writes ({gits:o}), not {written:?}");
    remove(&fixture);
}

/// What:
///  A merge that deletes the file the person set a mode on deletes it,
///  and the file keeps no mode because it is not there.
/// Why:
///  Real Git deletes such a file:
///  a mode it does not record is no change at all to its own update,
///  so the deletion takes the file and the mode with it.
///
///  The person's staged edit at another path is what puts the merge on the route where this command completes it
///  itself and writes the file,
///  so the write this control judges is the carry's own.
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('victim.txt', 0o600); merge(); expect(existsSync('victim.txt')).toBe(false);
/// ```
#[test]
fn a_merge_that_deletes_the_file_deletes_a_mode_the_person_set() {
    if ran_in_child(module_path!(), "a_merge_that_deletes_the_file_deletes_a_mode_the_person_set", 0o022) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-delete");
    let repo: PathBuf = diverged(&fixture, "delete", WhatTheMergeDoes::Delete);
    use std::os::unix::fs::PermissionsExt as _;
    an_edit_the_person_staged(&fixture, repo.as_path());
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge lands: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    assert_eq!(mode_at(repo.as_path(), THE_FILE), None, "the file the merge deletes is gone, with the mode a person set on it");
    remove(&fixture);
}

/// What:
///  A merge that conflicts and writes markers into the file writes them at the mode Git writes.
/// Why:
///  Real Git writes a conflicted file as a fresh entry:
///  its markers land in a file created with `0666` less the umask,
///  and the mode the person set on the file that stood there goes.
///  The index holds the path at three stages while it is conflicted,
///  so `checkout-index` cannot read an entry to compare with;
///  the answer is read from a second repository of the same shape where real Git runs the merge itself,
///  which is the same comparison every other control makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const twin = diverged('conflict-real'); chmod(twin, 0o600); native(twin, ['merge', '--no-edit', 'feature']);
/// chmod('victim.txt', 0o600); merge(); expect(read('victim.txt')).toContain('<<<<<<<'); expect(mode('victim.txt')).toBe(mode(twin, 'victim.txt')); expect(code).toBe(1);
/// ```
#[test]
fn a_merge_that_conflicts_writes_the_markers_at_the_mode_git_writes() {
    if ran_in_child(module_path!(), "a_merge_that_conflicts_writes_the_markers_at_the_mode_git_writes", 0o022) {
        return;
    }
    use std::os::unix::fs::PermissionsExt as _;
    let fixture: Fixture = fixture("merge-mode-conflict");
    // Real Git's own answer, in a repository of the same shape, under the same mask.
    let twin: PathBuf = diverged(&fixture, "conflict-real", WhatTheMergeDoes::Conflict);
    std::fs::set_permissions(twin.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let gits_run: Observed = observe(super::support::direct(&fixture).current_dir(twin.as_path()).args(["merge", "--no-edit", "feature"]), b"");
    let gits: Option<u32> = mode_at(twin.as_path(), THE_FILE);
    let repo: PathBuf = diverged(&fixture, "conflict", WhatTheMergeDoes::Conflict);
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, gits_run.code, "the merge ends as real Git's ends ({}): {}", String::from_utf8_lossy(gits_run.stderr.as_slice()), String::from_utf8_lossy(ran.stderr.as_slice()));
    assert_eq!(ran.stdout, gits_run.stdout, "the same words reach the person as with real Git");
    let written_text: String = String::from_utf8_lossy(std::fs::read(repo.join(THE_FILE)).expect(THE_FILE).as_slice()).into_owned();
    assert!(written_text.contains("<<<<<<<"), "the conflicted file holds both sides: {written_text}");
    assert_eq!(mode_at(repo.as_path(), THE_FILE), gits, "the conflicted file ends at the mode real Git writes, not the one the person set");
    remove(&fixture);
}

/// What:
///  A path the merge does not touch keeps the mode the person set,
///  because real Git writes nothing there.
/// Why:
///  The rule is that the mode follows Git,
///  and Git's own update never writes a file it does not change,
///  so the person's mode on such a path stands exactly as they left it.
///
///  The person's staged edit at another path is what puts the merge on the route where this command completes it
///  itself and writes the file,
///  so the write this control judges is the carry's own.
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('victim.txt', 0o600); merge(); expect(mode('victim.txt')).toBe(0o600); expect(read('victim.txt')).toBe(ON_THE_BRANCH);
/// ```
#[test]
fn a_merge_leaves_a_path_it_does_not_change_at_the_mode_the_person_set() {
    if ran_in_child(module_path!(), "a_merge_leaves_a_path_it_does_not_change_at_the_mode_the_person_set", 0o022) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-untouched");
    let repo: PathBuf = diverged(&fixture, "untouched", WhatTheMergeDoes::AnotherPath);
    use std::os::unix::fs::PermissionsExt as _;
    an_edit_the_person_staged(&fixture, repo.as_path());
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge lands: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    assert_eq!(std::fs::read(repo.join(THE_FILE)).expect(THE_FILE), ON_THE_BRANCH.as_bytes(), "the merge writes nothing at the path");
    assert_eq!(mode_at(repo.as_path(), THE_FILE), Some(0o600), "the mode the person set on a path Git does not write stands");
    remove(&fixture);
}

/// What:
///  A file the person edited,
///  at a path the merge deletes,
///  stays with their bytes and every permission bit the capture read.
/// Why:
///  That path is a deletion against an edit,
///  so the person's side is kept whole and named,
///  and nothing of Git's stands there to give the file a mode:
///  the bits come from the entry the capture read,
///  which is the one place a mode Git does not record is still written
///  (`SPEC.md`,
///  "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('victim.txt', ON_THE_BRANCH + THE_PERSONS_LINE); chmod('victim.txt', 0o600); merge(); expect(read('victim.txt')).toContain(THE_PERSONS_LINE); expect(mode('victim.txt')).toBe(0o600);
/// ```
#[test]
fn a_merge_that_keeps_the_persons_edit_at_a_deleted_path_keeps_the_mode_they_set() {
    if ran_in_child(module_path!(), "a_merge_that_keeps_the_persons_edit_at_a_deleted_path_keeps_the_mode_they_set", 0o022) {
        return;
    }
    let fixture: Fixture = fixture("merge-mode-kept-edit");
    let repo: PathBuf = diverged(&fixture, "kept-edit", WhatTheMergeDoes::Delete);
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::write(repo.join(THE_FILE), format!("{ON_THE_BRANCH}{THE_PERSONS_LINE}")).expect(THE_FILE);
    std::fs::set_permissions(repo.join(THE_FILE), std::fs::Permissions::from_mode(0o600)).expect("the person's mode");
    let ran: Observed = merge(&fixture, repo.as_path());
    assert_eq!(ran.code, Some(0), "the merge is made and their file named: {}", String::from_utf8_lossy(ran.stderr.as_slice()));
    let kept: String = String::from_utf8_lossy(std::fs::read(repo.join(THE_FILE)).unwrap_or_default().as_slice()).into_owned();
    assert!(kept.contains(THE_PERSONS_LINE), "their own line is still in the file they wrote: {kept:?}");
    assert_eq!(mode_at(repo.as_path(), THE_FILE), Some(0o600), "the bits they set on the entry that stands are theirs");
    remove(&fixture);
}

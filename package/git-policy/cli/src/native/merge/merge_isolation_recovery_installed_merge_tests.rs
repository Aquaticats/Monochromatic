//! What:
//!  A merge whose index an earlier attempt wrote:
//!  what stops the next command when a file is foreign,
//!  a file cannot be read,
//!  the checkout changed branch,
//!  the index stays held,
//!  or another command touched it,
//!  what a retry reports and a ref lookup that cannot start still finds,
//!  and what a take or a write that failed along the way leaves.
//! Why:
//!  The index is the one half of a merge that cannot be put back afterwards,
//!  so a merge is never finished again on a guess about what is installed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refused.text).toContain('partly installed'); expect(exists(record)).toBe(true);
//! ```

use super::super::*;
use super::fixtures::{
    A_CONTROLS_MERGE,
    World,
    aside_names,
    deferred_record_of,
    directory_to_file_world,
    place,
    stranded_name,
    world,
};
use super::readings::judged_changes;
use crate::merge_isolation_changes::file_identity;
/// The install of a merge's index entries and files,
/// which a placement of the merge's own bytes makes.
use crate::merge_isolation_recovery_steps::install_entries;
use crate::test_support::{
    git,
    git_context,
    git_text,
    remove,
};
use std::path::PathBuf;

/// What:
///  A saved merge whose index and files are installed stops the command when a file is foreign,
///  instead of letting the command read the merge as ordinary staged work.
/// Why:
///  The foreign-content refusal was wrapped as one that wrote nothing,
/// so the saved merge's catch let the command
///  go on although the index already held the merge beside the old branch
/// (a cross-vendor review of the fourth round,
/// the brief's item 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged, { updateWorktree: true }); const record = deferredRecord(); write('f.txt', foreign); await expect(recover(...)).rejects.toMatch(/f.txt/);
/// ```
#[test]
fn a_saved_merge_whose_installed_state_meets_a_foreign_file_stops_the_command() {
    let world: World = world("merge-recovery-installed-foreign");
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "27272727-0000-4000-8000-000000000027",
        "refs/cli-git/kept/27",
    );
    // An editor saves something else into a path the merge changes.
    std::fs::write(world.repo.join("f.txt"), b"an editor's own save\n").expect("the editor's file");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a begun merge stops the command");
    assert!(
        refused.0.contains("f.txt"),
        "the path is named: {}",
        refused.0
    );
    assert!(record.exists(), "the record is kept");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the branch did not move"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge that is partly installed keeps its record when the branch advances,
///  and the command is told.
/// Why:
///  The branch-moved branch dropped the record because a side ref existed,
/// without looking at what was installed,
/// so a
///  file under its private name and the merge's staged entry were left with nothing that would ever recover them
///  (a cross-vendor review of the fourth round,
/// the brief's item 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); deferredRecord(); commitUnrelated(); await expect(recover(...)).rejects.toMatch(/partly installed/); expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_saved_merge_that_is_partly_installed_and_whose_branch_advanced_keeps_its_record() {
    let world: World = world("merge-recovery-advanced-partly-installed");
    // The merge's index is installed and its files are not: the command that installed them was killed here.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "28282828-0000-4000-8000-000000000028",
        "refs/cli-git/kept/28",
    );
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let staged_entry: String = git_text(
        world.repo.as_path(),
        &["ls-files", "--stage", "--", "f.txt"],
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a partly installed merge stops the command");
    assert!(
        refused.0.contains("partly installed"),
        "it says the merge is partly installed: {}",
        refused.0
    );
    assert!(
        refused.0.contains(record.to_string_lossy().as_ref()),
        "the record is named: {}",
        refused.0
    );
    assert!(
        record.exists(),
        "the record is kept, so the install can still be settled"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the moved one"
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["ls-files", "--stage", "--", "f.txt"]
        ),
        staged_entry,
        "the installed entry is kept"
    );
    remove(world.parent.as_path());
}

/// What:
///  A take whose content read fails after the rename puts the file back and reports nothing installed.
/// Why:
///  The rename had already happened when the read failed,
/// so returning an error without restoring the file left it under
///  its private name while recovery said nothing of the merge was in the checkout
///  (a cross-vendor review of the fourth round,
/// the brief's item 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitattributes('f.txt filter=broken'); const (changes, judged) = judge(world); gitConfig('filter.broken.clean', "exit 1"); gitConfig('filter.broken.required', 'true'); await expect(installEntries(...)).rejects; expect(read('f.txt')).toEqual(old); expect(asides()).toEqual([]);
/// ```
#[test]
fn a_take_whose_read_fails_after_the_rename_puts_the_file_back() {
    let world: World = world("merge-recovery-take-read-fails");
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=broken\n")
        .expect("the attribute file");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let old: Vec<u8> = std::fs::read(world.repo.join("f.txt")).expect("the old content");
    // From here the clean filter is required and fails, so the read after the rename cannot run.
    git(
        world.repo.as_path(),
        &["config", "filter.broken.clean", "false"],
    );
    git(
        world.repo.as_path(),
        &["config", "filter.broken.required", "true"],
    );
    let mut said: Vec<u8> = Vec::new();
    let failed = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("a read that fails after the rename stops the merge");
    assert!(!failed.began, "nothing of the merge was installed");
    assert!(!failed.error.0.is_empty(), "the failure is named");
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("the file is back"),
        old,
        "the file the take moved is put back"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A taken file whose appended bytes a clean filter drops is kept rather than removed.
/// Why:
///  `may_remove` compared only the filtered hash,
/// so a write a clean filter hides left that hash as it was,
/// and the
///  file that held the person's bytes was deleted with its private name
/// (a cross-vendor review of the fourth round,
/// the brief's item 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanFilter('strip', 'sed /^NOTE:/d'); const take = takeAside(file); append(take.aside, 'NOTE: locally saved data\n'); finishAsides([take], sink); expect(exists(take.aside)).toBe(true);
/// ```
#[test]
fn a_taken_file_whose_appended_bytes_a_filter_drops_is_kept_and_named() {
    let world: World = world("merge-recovery-filtered-append");
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=strip\n")
        .expect("the attribute file");
    git(
        world.repo.as_path(),
        &["config", "filter.strip.clean", "sed /^NOTE:/d"],
    );
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let judged: &Judged = &judgements[changes
        .iter()
        .position(|change| {
            return change.path == b"f.txt";
        })
        .expect("the path")];
    let expected: String = judged
        .content
        .clone()
        .expect("the object the judgement read");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    // A writer that kept its descriptor appends a line the clean filter drops.
    let mut file: std::fs::File = std::fs::File::options()
        .append(true)
        .open(aside.as_path())
        .expect("the writer's descriptor");
    file.write_all(b"NOTE: locally saved data\n")
        .expect("the appended line");
    drop(file);
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected,
    }];
    let mut said: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said)
        .expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        aside.exists(),
        "a file whose bytes changed behind a filter is kept"
    );
    assert!(
        text.contains(aside.to_string_lossy().as_ref()),
        "the private name is said: {text}"
    );
    assert!(
        std::fs::read(aside.as_path())
            .expect("the aside")
            .ends_with(b"NOTE: locally saved data\n"),
        "the appended bytes are kept"
    );
    remove(world.parent.as_path());
}

/// What:
///  A write that arrives after the holder scan is seen by the last look,
///  so the file that holds it is kept.
/// Why:
///  The identity was read before the scan,
/// so a program that kept its descriptor open could append a line the clean filter drops
///  after that read and close it,
/// and the scan then found no holder while the filtered bytes still matched
///  (a cross-vendor review of the fifth round,
/// its first finding).
/// The seam is the point a control reaches after the scan and before the last looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = takeAside(file); hooks.set('aside-scanned', () => append(take.aside, 'NOTE: kept\n')); finishAsides([take], sink); expect(exists(take.aside)).toBe(true);
/// ```
#[test]
fn a_write_that_arrives_after_the_holder_scan_is_seen_by_the_last_look() {
    let world: World = world("merge-recovery-write-after-scan");
    // A clean filter that drops the line the writer appends, so only the identity can see the change.
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=strip\n")
        .expect("the attribute file");
    git(
        world.repo.as_path(),
        &["config", "filter.strip.clean", "sed /^NOTE:/d"],
    );
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let judged: &Judged = &judgements[changes
        .iter()
        .position(|change| {
            return change.path == b"f.txt";
        })
        .expect("the path")];
    let expected: String = judged
        .content
        .clone()
        .expect("the object the judgement read");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected,
    }];
    // The program that kept its descriptor writes at the instant the scan has just finished.
    let written: PathBuf = aside.clone();
    crate::service_fault::hooks::set(
        "aside-scanned",
        std::rc::Rc::new(move || {
            let mut file: std::fs::File = std::fs::File::options()
                .append(true)
                .open(written.as_path())
                .expect("the writer's descriptor");
            file.write_all(b"NOTE: locally saved data\n")
                .expect("the appended line");
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let outcome = finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said);
    crate::service_fault::hooks::clear();
    outcome.expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "a file written after the scan is kept");
    assert!(
        text.contains(aside.to_string_lossy().as_ref()),
        "the private name is said: {text}"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose index was installed stops the command when the worktree has another branch checked out.
/// Why:
///  The branch-mismatch refusal was wrapped as one that wrote nothing,
/// so a saved merge whose index entries were already
///  installed was reported as having nothing in the checkout and the command went on
///  (a cross-vendor review of the fifth round,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); const record = deferredRecord(); await git(['branch', 'other']); await git(['switch', 'other']); await expect(recover(...)).rejects.toMatch(/checked out/);
/// ```
#[test]
fn a_saved_merge_whose_index_was_installed_and_whose_checkout_changed_branch_stops_the_command() {
    let world: World = world("merge-recovery-installed-other-branch");
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "29292929-0000-4000-8000-000000000029",
        "refs/cli-git/kept/29",
    );
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    git(world.repo.as_path(), &["switch", "--quiet", "other"]);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("an installed merge stops the command");
    assert!(
        refused.0.contains("refs/heads/main"),
        "the branch the merge is for is named: {}",
        refused.0
    );
    assert!(record.exists(), "the record is kept");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the merge's branch did not move"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose index was installed stops the command when a file cannot be read.
/// Why:
///  The worktree read failure was wrapped as one that wrote nothing,
/// so a merge whose index entries were installed was
///  reported as not installed and the command went on
///  (a cross-vendor review of the fifth round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); deferredRecord(); gitConfig('filter.broken.required', 'true'); await expect(recover(...)).rejects;
/// ```
#[test]
fn a_saved_merge_whose_index_was_installed_and_a_file_cannot_be_read_stops_the_command() {
    let world: World = world("merge-recovery-installed-unreadable");
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "30303030-0000-4000-8000-000000000030",
        "refs/cli-git/kept/30",
    );
    // A clean filter the path requires now fails, so the worktree cannot be read.
    std::fs::write(world.repo.join(".gitattributes"), b"f.txt filter=broken\n")
        .expect("the attribute file");
    git(
        world.repo.as_path(),
        &["config", "filter.broken.clean", "false"],
    );
    git(
        world.repo.as_path(),
        &["config", "filter.broken.required", "true"],
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("an installed merge stops the command");
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A retry whose take fails reports the index entries an earlier attempt installed,
///  so the command stops.
/// Why:
///  The take failure computed `began` from its own rollback alone,
/// so a merge whose index was installed by an
///  earlier attempt was reported as not installed and the command went on
///  (a cross-vendor review of the fifth round,
/// its sixth finding).
/// The failing take is a path in a directory that refuses the rename,
/// which the control skips where it runs as root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); await readTree(repo, old, merged); deferredRecord(); chmod('dir', 0o555); await expect(recover(...)).rejects; expect(exists(record)).toBe(true);
/// ```
#[cfg(unix)]
#[test]
fn a_retry_after_an_earlier_install_that_fails_reports_the_install() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        eprintln!(
            "skipped: a directory that refuses writes does not stop root, so this control proves nothing here"
        );
        return;
    }
    let world: World = directory_to_file_world("merge-recovery-retry-installed");
    // An earlier attempt installed the merge's index entries and died before it wrote the files.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "31313131-0000-4000-8000-000000000031",
        "refs/cli-git/kept/31",
    );
    // The take of the file the merge deletes cannot rename into the directory that holds it.
    std::fs::set_permissions(
        world.repo.join("dir"),
        std::fs::Permissions::from_mode(0o555),
    )
    .expect("a directory that refuses writes");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("the retry's failure stops the command");
    let _writable = std::fs::set_permissions(
        world.repo.join("dir"),
        std::fs::Permissions::from_mode(0o755),
    );
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A recovery whose ref lookup cannot start still says the merge is partly installed.
/// Why:
///  The branch and `HEAD` reads were wrapped as refusals that wrote nothing,
/// so a saved merge whose index an earlier
///  attempt had installed was reported as having nothing in the checkout when the lookup's child could not
///  start,
/// and the requested command went on against the half-applied merge
///  (a cross-vendor review of the sixth round,
/// its second finding).
/// The seam is a Git that removes itself while the last file is read,
/// so the lookup that follows cannot start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); deferredRecord(); const place = withGit(vanishingShim); await expect(recover(place)).rejects; expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_ref_lookup_that_cannot_start_reports_the_install_already_found() {
    let world: World = world("merge-recovery-ref-lookup-fails");
    // An earlier attempt installed the merge's index entries and died before moving the branch.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "32323232-0000-4000-8000-000000000032",
        "refs/cli-git/kept/32",
    );
    // A Git that removes itself while the last judged file is read: the ref lookup that follows cannot start.
    let shim: PathBuf = world.parent.join("vanishing-git");
    crate::test_support::executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\nfor argument in \"$@\"; do\n  case \"$argument\" in\n    --path=gone.txt) rm --force -- '{shim}' ;;\n  esac\ndone\nexec {real} \"$@\"\n",
            shim = shim.display(),
            real = crate::test_support::REAL_GIT
        )
        .as_bytes(),
    );
    // Named options make the in-process readers decline, so every step here is a child and the last one can be missing.
    let mut context: crate::transaction_git::GitContext = git_context();
    context.real_git = shim.clone();
    context.global_prefix = vec![
        std::ffi::OsString::from("--git-dir"),
        world.git_dir.as_os_str().to_owned(),
        std::ffi::OsString::from("--work-tree"),
        world.repo.as_os_str().to_owned(),
    ];
    let place: RecoveryPlace = RecoveryPlace {
        git: context,
        cwd: world.repo.clone(),
        common_dir: world.git_dir.clone(),
        git_dir: world.git_dir.clone(),
    };
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place, world.git_dir.as_path(), &mut said)
        .expect_err("the refusal must report the install");
    assert!(!shim.exists(), "the seam ran");
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A worktree that switches branch in the instant before the branch move keeps the record,
///  and the branch is put back.
/// Why:
///  The branch write verifies the branch and not the worktree's `HEAD`,
/// so a switch that lands after the last look moved `main`
///  while `other` owned the checkout,
/// and recovery then removed the only record of it
///  (a cross-vendor review of the sixth round,
/// its first finding).
/// The seam is the instant between the last look and the write,
/// which no other test can reach.
/// The line that undo names is a command the person runs where they are,
/// so it carries the flag this wrapper's linked-worktree policy asks for (3147d271df).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); await git(['branch', 'other']); hooks.set('before-the-branch-write', () => git(['symbolic-ref', 'HEAD', 'refs/heads/other'])); await expect(recover(...)).rejects; expect(await revParse('main')).toEqual(oldOid); expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_worktree_that_switches_before_the_branch_write_keeps_the_record_and_the_branch() {
    let world: World = world("merge-recovery-switch-before-write");
    let record: PathBuf = deferred_record_of(
        &world,
        "34343434-0000-4000-8000-000000000034",
        "refs/cli-git/kept/34",
    );
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    let repository: PathBuf = world.repo.clone();
    crate::service_fault::hooks::set(
        "before-the-branch-write",
        std::rc::Rc::new(move || {
            git(
                repository.as_path(),
                &["symbolic-ref", "HEAD", "refs/heads/other"],
            );
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let attempted = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said);
    crate::service_fault::hooks::clear();
    let refused = attempted.expect_err("a switched worktree stops the command");
    assert!(
        refused.0.contains("refs/heads/other"),
        "the branch the worktree switched to is named: {}",
        refused.0
    );
    assert!(
        refused
            .0
            .contains("index entries and files are still in this worktree"),
        "it says the merge's entries and files stay: {}",
        refused.0
    );
    assert!(
        !refused.0.contains("nothing of this merge stays staged"),
        "it does not claim the checkout is clean: {}",
        refused.0
    );
    let undo: String = format!(
        "`git restore {} --staged --worktree -- <path>`",
        crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH
    );
    assert!(
        refused.0.contains(undo.as_str()),
        "it names the way to undo the staged content, with the flag this wrapper's policy asks for, in the words a person can run: {}",
        refused.0
    );
    assert!(
        !refused.0.contains(".."),
        "the sentence the refusal itself ends with is not followed by a second period: {}",
        refused.0
    );
    assert!(record.exists(), "the record is kept");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the branch was put back"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]),
        "refs/heads/other",
        "the worktree is where the other command put it"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge whose index entries an earlier attempt installed stops the command when the index stays held,
///  instead of letting it read the checkout.
/// Why:
///  The install returned `Installed::IndexHeld` whether or not the merge was already installed,
/// so the startup gate let a
///  read-only `status` or `diff --cached` observe the unfinished merge as ordinary staged work
///  (a cross-vendor review of the seventh round,
/// its fourth finding).
/// The lock is one the removal proof refuses,
/// so the wait ends at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); const record = deferredRecord(); write('.git/index.lock', content); write('.git/index~pid.lock', 'pid 1\n'); await expect(recover(...)).rejects; expect(exists(record)).toBe(true);
/// ```
#[test]
fn an_installed_merge_whose_index_stays_held_stops_the_command() {
    let world: World = world("merge-recovery-installed-index-held");
    // An earlier attempt installed the merge's index entries and died.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = deferred_record_of(
        &world,
        "35353535-0000-4000-8000-000000000035",
        "refs/cli-git/kept/35",
    );
    // A lock the removal proof does not cover, in the shape a person's editor-phase commit leaves.
    std::fs::write(
        world.git_dir.join("index.lock"),
        b"an index written into the lock",
    )
    .expect("a lock with content");
    std::fs::write(world.git_dir.join("index~pid.lock"), b"pid 4294967\n")
        .expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("an installed merge behind a held index stops the command");
    assert!(!refused.0.is_empty(), "the failure is named");
    assert!(
        refused.0.contains("index.lock"),
        "the lock is named: {}",
        refused.0
    );
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A directory a killed recovery left empty is removed by the retry,
///  so the merge that puts a file there finishes.
/// Why:
///  The kill sits between finishing the child's private name and removing the emptied directory,
/// and the retry judged the
///  missing child as already at its end state,
/// so it never scheduled the removal and `checkout-index` refused the
///  existing directory on every attempt
///  (a cross-vendor review of the seventh round,
/// its fifth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); await readTree(repo, old, merged); rm('dir/child.txt'); deferredRecord(); const outcomes = await recover(...); expect(read('dir')).toEqual(mergeFile);
/// ```
#[test]
fn a_directory_left_empty_by_a_killed_recovery_is_removed_on_the_retry() {
    let world: World = directory_to_file_world("merge-recovery-empty-directory-retry");
    // An earlier attempt installed the merge's index, finished the child's private name and died before removing the directory.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    std::fs::remove_file(world.repo.join("dir/child.txt"))
        .expect("the child's file goes, as a finished take leaves it");
    assert!(
        world.repo.join("dir").is_dir(),
        "the emptied directory is still there"
    );
    assert_eq!(
        std::fs::read_dir(world.repo.join("dir"))
            .expect("the directory")
            .count(),
        0,
        "and it is empty"
    );
    let _record: PathBuf = deferred_record_of(
        &world,
        "36363636-0000-4000-8000-000000000036",
        "refs/cli-git/kept/36",
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the retry finishes the merge");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(
        std::fs::read(world.repo.join("dir")).expect("the merge's file"),
        b"the merge's file\n"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid
    );
    remove(world.parent.as_path());
}

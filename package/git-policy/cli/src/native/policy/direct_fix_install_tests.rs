//! What:
//!  Controls for installing corrections:
//!  all files swapped with their modes and owners and the
//!       index untouched,
//!  or no file this command still owns changed,
//!  whichever step fails,
//!  and a journal
//!       for every byte a fix cannot prove it may remove.
//! Why:
//!  A fix that left half its files replaced,
//!  a lost write,
//!  a changed index or a journal that vanished
//!      with the only copy of a person's edit would damage the worktree it was asked to tidy.
//!  Every
//!      failure and every kill is provoked for real on a disposable directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await expect(installCorrections(root, index, changes)).rejects.toThrow('changed while');
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{
    InstallChange, Prepared, content_of, install_corrections, install_prepared, read_index,
    recover_interrupted_installs,
};
use crate::candidate_object::CandidateMode;
use crate::direct_fix_install_journal::{RECORD_FILE, read_record};
use crate::direct_fix_install_writes::{Kept, write_new};
use crate::test_support::{fixture, git, remove, repository};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A worktree with a regular file at the top,
///  an executable one below,
///  and an index file beside them.
struct Tree {
    /// The disposable fixture,
    ///  removed at the end.
    root: PathBuf,
    /// The worktree's top level.
    top: PathBuf,
    /// The index file,
    ///  whose directory is also where the journals go.
    index: PathBuf,
}

/// A fresh tree under a fixture named `name`.
fn tree(name: &str) -> Tree {
    let root: PathBuf = fixture(name);
    let top: PathBuf = root.join("top");
    std::fs::create_dir_all(top.join("sub")).expect("worktree");
    std::fs::write(top.join("a.txt"), b"a").expect("a");
    std::fs::write(top.join("sub/run.sh"), b"#!/bin/sh").expect("script");
    std::fs::set_permissions(
        top.join("sub/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("executable");
    let index: PathBuf = root.join("index");
    std::fs::write(&index, b"index bytes").expect("index");
    return Tree { root, top, index };
}

/// The two corrections of a tree.
fn changes() -> Vec<InstallChange> {
    return vec![
        InstallChange {
            path: b"a.txt".to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(&b"a"[..]),
            replacement: Rc::from(&b"a\n"[..]),
        },
        InstallChange {
            path: b"sub/run.sh".to_vec(),
            mode: CandidateMode::Executable,
            original: Rc::from(&b"#!/bin/sh"[..]),
            replacement: Rc::from(&b"#!/bin/sh\n"[..]),
        },
    ];
}

/// One correction of the top-level file.
fn one_change() -> Vec<InstallChange> {
    return vec![InstallChange {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a"[..]),
        replacement: Rc::from(&b"a\n"[..]),
    }];
}

/// The two worktree files' bytes,
///  in a fixed order.
fn contents(top: &Path) -> (Vec<u8>, Vec<u8>) {
    return (
        std::fs::read(top.join("a.txt")).expect("a"),
        std::fs::read(top.join("sub/run.sh")).expect("script"),
    );
}

/// The untouched bytes of a tree.
fn untouched() -> (Vec<u8>, Vec<u8>) {
    return (b"a".to_vec(), b"#!/bin/sh".to_vec());
}

/// Every name below `directory` whose name starts with the journal prefix,
///  and every name that starts with
///       the prefix a fix once wrote beside a file.
fn leftovers(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("directory") {
        let path: PathBuf = entry.expect("entry").path();
        let name: String = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        if name.starts_with(".cli-git-direct-fix-") {
            found.push(name);
        }
        if path.is_dir() {
            found.extend(leftovers(path.as_path()));
        }
    }
    found.sort();
    return found;
}

/// The journals below a Git directory,
///  in name order.
fn journals(git_dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(git_dir).expect("git directory") {
        let path: PathBuf = entry.expect("entry").path();
        if path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .starts_with("cli-git-direct-fix-")
        {
            found.push(path);
        }
    }
    found.sort();
    return found;
}

/// What:
///  Every file is swapped with its mode,
///  the index is the same,
///  and nothing is left beside
///       the files or in the Git directory.
/// Why:
///  A clean fix must leave the worktree exactly as Git sees it and no journal behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await installCorrections(root, index, changes); expect(await journals(gitDir)).toEqual([]);
/// ```
#[test]
fn corrections_are_installed_with_their_modes() {
    let fixture_tree: Tree = tree("install-ok");
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Ok(super::InstallOutcome::default())
    );
    assert_eq!(
        contents(&fixture_tree.top),
        (b"a\n".to_vec(), b"#!/bin/sh\n".to_vec())
    );
    let regular: u32 = std::fs::metadata(fixture_tree.top.join("a.txt"))
        .expect("a")
        .permissions()
        .mode();
    let executable: u32 = std::fs::metadata(fixture_tree.top.join("sub/run.sh"))
        .expect("script")
        .permissions()
        .mode();
    assert_eq!(regular & 0o111, 0, "{regular:o}");
    assert_eq!(executable & 0o100, 0o100, "{executable:o}");
    assert_eq!(
        std::fs::read(&fixture_tree.index).expect("index"),
        b"index bytes"
    );
    assert_eq!(leftovers(&fixture_tree.top), Vec::<String>::new());
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    // A repository without an index file still has none afterwards, and nothing to install installs nothing.
    std::fs::remove_file(&fixture_tree.index).expect("no index");
    assert_eq!(read_index(&fixture_tree.index).expect("absent"), None);
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, &[]),
        Ok(super::InstallOutcome::default())
    );
    let again: Vec<InstallChange> = vec![InstallChange {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a\n"[..]),
        replacement: Rc::from(&b"b\n"[..]),
    }];
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, again.as_slice()),
        Ok(super::InstallOutcome::default())
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"b\n"
    );
    assert!(!fixture_tree.index.exists());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A file kept private stays private,
///  the set-user-id and set-group-id bits survive,
///  and a
///       symbolic link is refused instead of replaced.
/// Why:
///  A replacement created as the running user with the umask's mode would publish a `0600` file,
///  drop the privilege a script was given,
///  and turn a link into a regular file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await chmod(file, 0o6755); await installCorrections(...); expect((await stat(file)).mode & 0o7777).toBe(0o6755)
/// ```
#[test]
fn the_mode_owner_and_kind_of_a_file_survive_a_fix() {
    let fixture_tree: Tree = tree("install-private");
    let private: PathBuf = fixture_tree.top.join("a.txt");
    let script: PathBuf = fixture_tree.top.join("sub/run.sh");
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o6400)).expect("private");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o6755)).expect("special bits");
    let before: std::fs::Metadata = std::fs::metadata(&private).expect("a");
    let (owner_before, group_before): (u32, u32) = (before.uid(), before.gid());
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Ok(super::InstallOutcome::default())
    );
    let kept: u32 = std::fs::metadata(&private).expect("a").permissions().mode() & 0o7777;
    assert_eq!(kept, 0o6400, "{kept:o}");
    let kept_script: u32 = std::fs::metadata(&script).expect("script").permissions().mode() & 0o7777;
    assert_eq!(kept_script, 0o6755, "{kept_script:o}");
    let after: std::fs::Metadata = std::fs::metadata(&private).expect("a");
    assert_eq!((after.uid(), after.gid()), (owner_before, group_before));
    remove(fixture_tree.root.as_path());
}

/// A selected path that is not a regular file stops the fix before any file changes.
#[test]
fn a_file_that_is_not_a_regular_file_stops_before_any_change() {
    let fixture_tree: Tree = tree("install-symlink");
    let linked: PathBuf = fixture_tree.top.join("sub/run.sh");
    std::fs::remove_file(&linked).expect("remove");
    std::os::unix::fs::symlink("../a.txt", &linked).expect("link");
    let refused: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("a symbolic link");
    assert_eq!(
        refused,
        String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file is not a regular file (a symbolic link, a directory or a device), and putting corrected bytes at its path would remove what is there, so no file was changed."
        )
    );
    assert!(std::fs::symlink_metadata(&linked)
        .expect("link")
        .file_type()
        .is_symlink());
    assert_eq!(std::fs::read(fixture_tree.top.join("a.txt")).expect("a"), b"a");
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}

/// A file that changed under the fix,
///  or is missing,
///  stops the fix before any file changes,
/// and leaves no journal.
#[test]
fn a_file_that_is_not_as_read_stops_before_any_change() {
    let fixture_tree: Tree = tree("install-stale");
    std::fs::write(fixture_tree.top.join("sub/run.sh"), b"edited").expect("edit");
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Err(String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file changed while monochromatic-git-policy-cli fix ran, so no file was changed. Run monochromatic-git-policy-cli fix again."
        ))
    );
    assert_eq!(
        contents(&fixture_tree.top),
        (b"a".to_vec(), b"edited".to_vec())
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    let missing: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("missing file");
    assert_eq!(
        missing,
        String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file is gone, and putting corrected bytes at its path would create a file the fix never read, so no file was changed."
        )
    );
    assert!(!missing.contains("run.sh"), "{missing}");
    let mut nameless: Vec<InstallChange> = changes();
    nameless[1].path = Vec::new();
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, nameless.as_slice()),
        Err(String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: Git named a corrected file this system cannot represent, so no file was changed."
        ))
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A permission change made after the fix read the file is refused,
///  like a change of its bytes.
/// Why:
///  The fix installs the mode and owner it snapshotted;
///  a person narrowing a file to `0600` while the fix
///  runs would get the broader mode back,
///  and the displaced comparison of bytes alone cannot see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const prepared = prepare(...); await chmod(file, 0o600); expect(install(prepared)).rejects.toThrow('changed while'); expect((await stat(file)).mode & 0o777).toBe(0o600)
/// ```
#[test]
fn a_permission_change_made_while_the_fix_ran_is_refused_and_kept() {
    let fixture_tree: Tree = tree("install-mode-change");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    std::fs::set_permissions(
        fixture_tree.top.join("a.txt"),
        std::fs::Permissions::from_mode(0o600),
    )
    .expect("narrow");
    let mut notices: Vec<String> = Vec::new();
    let refused: String = install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("a mode changed under the fix");
    assert!(
        refused.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file changed while monochromatic-git-policy-cli fix ran"
        ),
        "{refused}"
    );
    let kept: u32 = std::fs::metadata(fixture_tree.top.join("a.txt"))
        .expect("a")
        .permissions()
        .mode()
        & 0o7777;
    assert_eq!(kept, 0o600, "the person's own mode is back: {kept:o}");
    assert_eq!(
        contents(&fixture_tree.top),
        (b"a".to_vec(), b"#!/bin/sh".to_vec())
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A write that lands at a corrected path before a rollback is kept,
///  and the rollback says where the
///       bytes it could not restore are.
/// Why:
///  A blind swap back would move the newer save into the journal and then remove it,
///  which is the
///  exact loss the owner forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // afterSwap = (path) => { if (path.endsWith('a.txt')) writeFileSync(path, 'newer edit\n'); rmSync(secondContent); };
/// // await expect(install(prepared)).rejects.toThrow('does not own'); expect(readFileSync(a)).toBe('newer edit\n');
/// ```
#[test]
fn a_save_to_a_corrected_path_before_a_rollback_is_kept() {
    let fixture_tree: Tree = tree("install-rollback-save");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, changes().as_slice())
            .expect("prepared");
    let second_content: PathBuf = content_of(prepared[1].journal.as_path());
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |destination: &Path| {
            if destination.ends_with("a.txt") {
                // The person saves over the corrected file, and the second site then cannot be swapped at all.
                std::fs::write(destination, b"newer edit\n").expect("a save");
                std::fs::remove_file(second_content.as_path()).expect("break the second site");
            }
        }));
    });
    let mut notices: Vec<String> = Vec::new();
    let refused: String = install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("the second site fails");
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    assert!(
        refused.contains("the corrected files this command still owned were restored, and what it does not own stays"),
        "{refused}"
    );
    // The person's save is exactly where they put it.
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"newer edit\n".to_vec()
    );
    // And the bytes the fix could not restore are named, inside the Git directory.
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].starts_with("monochromatic-git-policy-cli kept ")
            && notices[0].contains("cli-git-direct-fix-")
            && notices[0].contains("/content"),
        "{notices:?}"
    );
    assert_eq!(journals(&fixture_tree.root).len(), 1, "one journal stays");
    remove(fixture_tree.root.as_path());
}

/// What:
///  A write through a descriptor opened before the swap is kept and named,
///  not removed with
///       the journal.
/// Why:
///  The writer holds the old inode;
///  its bytes land in the displaced file,
///  and removing that file would
///  lose them when the descriptor closes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const fd = openSync(a, 'r+'); afterSwap = () => writeSync(fd, 'late'); await install(...); expect(notices[0]).toContain('kept')
/// ```
#[test]
fn a_write_through_an_open_descriptor_is_kept_and_named() {
    let fixture_tree: Tree = tree("install-descriptor");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let destination: PathBuf = fixture_tree.top.join("a.txt");
    /// `Write` adds `write_all` to the borrowed file.
    use std::io::Write;
    let mut open: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .open(&destination)
        .expect("the person's editor holds the file");
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |swapped: &Path| {
            if swapped.ends_with("a.txt") {
                // Written through the descriptor the writer opened before the swap: it lands in the displaced file.
                open.write_all(b"# a late save\n").expect("a late write");
            }
        }));
    });
    let mut notices: Vec<String> = Vec::new();
    install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect("the correction goes in");
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    assert_eq!(
        std::fs::read(&destination).expect("a"),
        b"a\n".to_vec(),
        "the correction is in place"
    );
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].starts_with("monochromatic-git-policy-cli kept "),
        "{notices:?}"
    );
    // What the descriptor wrote is still on disk, inside the journal that stays.
    let kept: Vec<PathBuf> = journals(&fixture_tree.root);
    assert_eq!(kept.len(), 1, "{kept:?}");
    // The descriptor's offset is where the writer left it, so its write begins at the start of the file it holds.
    assert_eq!(
        std::fs::read(content_of(kept[0].as_path())).expect("kept bytes"),
        b"# a late save\n".to_vec()
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A fix killed between the swap and the judgement is undone by the next fix,
///  and the person's
///       newest bytes are back at the file.
/// Why:
///  That state is the one the review named as unrecoverable:
///  the file holds a correction nothing
///  verified,
///  and the only copy of the edit it displaced sits under a name nobody looks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const p = prepare(...); exchange(contentOf(p[0]), p[0].destination); markOwnerDead(p[0]); installCorrections(...); expect(readFile(a)).toBe('newer edit\n')
/// ```
#[test]
fn a_kill_between_the_swap_and_the_judgement_is_undone_by_the_next_fix() {
    let fixture_tree: Tree = tree("install-killed");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    // The person's newer edit is what the swap displaces.
    std::fs::write(fixture_tree.top.join("a.txt"), b"newer edit\n").expect("a save");
    super::exchange(
        content_of(prepared[0].journal.as_path()).as_path(),
        &fixture_tree.top.join("a.txt"),
    )
    .expect("the swap a kill interrupts after");
    // The kill: the record still names this process, so the test tells the sweeper its writer is gone.
    mark_writer_dead(prepared[0].journal.as_path());
    let mut notices: Vec<String> = Vec::new();
    recover_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"newer edit\n".to_vec(),
        "the newest bytes are back"
    );
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].contains("undid an interrupted correction"),
        "{notices:?}"
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A journal whose file was never touched is removed,
///  and one whose file has been written since is kept and named.
/// Why:
///  The first holds only a correction,
/// which is recomputable;
///  the second may hold the person's only copy
///  of a state,
///  and nothing this command can prove allows removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // killed before the swap: remove; killed and then written to: keep and name
/// ```
#[test]
fn a_journal_is_removed_only_when_nothing_of_the_person_is_in_it() {
    let fixture_tree: Tree = tree("install-journal-judged");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    mark_writer_dead(prepared[0].journal.as_path());
    let mut notices: Vec<String> = Vec::new();
    recover_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    assert!(
        notices[0].contains("had not reached its file, so nothing had changed"),
        "{notices:?}"
    );
    // Now a swap happens, the person saves again, and the owner dies: the journal must stay.
    let swapped: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    std::fs::write(fixture_tree.top.join("a.txt"), b"newer edit\n").expect("a save");
    super::exchange(
        content_of(swapped[0].journal.as_path()).as_path(),
        &fixture_tree.top.join("a.txt"),
    )
    .expect("swap");
    // The file is written again after the swap: its bytes are newer than anything the journal holds.
    std::fs::write(fixture_tree.top.join("a.txt"), b"newest edit\n").expect("another save");
    mark_writer_dead(swapped[0].journal.as_path());
    let mut second_notices: Vec<String> = Vec::new();
    recover_interrupted_installs(fixture_tree.root.as_path(), &mut second_notices);
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"newest edit\n".to_vec()
    );
    assert_eq!(journals(&fixture_tree.root).len(), 1, "the journal stays");
    assert!(
        second_notices[0].contains("has been written since the interrupted fix"),
        "{second_notices:?}"
    );
    remove(fixture_tree.root.as_path());
}

/// Rewrite a journal's record so its writer is a process no platform can have.
fn mark_writer_dead(journal: &Path) {
    let text: String =
        std::fs::read_to_string(journal.join(RECORD_FILE)).expect("the record of a prepared site");
    let dead: crate::worktree_copy_owner::CopyOwner = crate::worktree_copy_owner::CopyOwner {
        pid: 4_194_304,
        birth_identity: String::from("0"),
        pid_namespace: crate::pid_namespace::current_pid_namespace(),
    };
    let mut rewritten: String = String::new();
    for line in text.lines() {
        if line.starts_with("owner ") {
            rewritten.push_str(format!("owner {}\n", dead.token()).as_str());
        } else {
            rewritten.push_str(format!("{line}\n").as_str());
        }
    }
    std::fs::write(journal.join(RECORD_FILE), rewritten.as_bytes()).expect("a dead writer");
    assert!(read_record(journal).is_some(), "the record still decodes");
}

/// What:
///  A destination that cannot take its correction stops the install,
///  restores every file
///  this command still owns,
///  and leaves nothing behind.
/// Why:
///  A file deleted,
///  renamed or turned into a directory while the fix ran is not the file the fix
///  read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // prepareReplacements(); await rm(second); await expect(installPrepared()).rejects.toThrow('in place failed')
/// ```
#[test]
fn a_destination_that_cannot_take_the_correction_restores_every_file() {
    let fixture_tree: Tree = tree("install-vanished");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, changes().as_slice())
            .expect("prepared");
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    let mut notices: Vec<String> = Vec::new();
    let failed: String = install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("the destination is gone");
    assert!(
        failed.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: putting a corrected file in place failed ("
        ) && failed.contains("so every corrected file was restored"),
        "{failed}"
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}

/// An index that changed,
///  cannot be read again,
///  or cannot be read at all restores every file this command owns.
#[test]
fn an_index_that_is_not_as_read_restores_every_file() {
    let fixture_tree: Tree = tree("install-index");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, changes().as_slice())
            .expect("prepared");
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    let mut notices: Vec<String> = Vec::new();
    assert_eq!(
        install_prepared(
            prepared.as_slice(),
            &fixture_tree.index,
            &before,
            &mut notices
        ),
        Err(String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: the index changed while monochromatic-git-policy-cli fix ran, so every corrected file was restored. Run monochromatic-git-policy-cli fix again."
        ))
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    // An index that turned into a directory cannot be read again.
    std::fs::remove_file(&fixture_tree.index).expect("remove index");
    std::fs::create_dir(&fixture_tree.index).expect("index directory");
    let prepared_again: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, changes().as_slice())
            .expect("prepared");
    let mut again_notices: Vec<String> = Vec::new();
    let unreadable: String = install_prepared(
        prepared_again.as_slice(),
        &fixture_tree.index,
        &before,
        &mut again_notices,
    )
    .expect_err("unreadable index");
    assert!(
        unreadable.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: reading the index again failed ("
        ) && unreadable.contains("so every corrected file was restored"),
        "{unreadable}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    // The first read failing changes nothing at all.
    let first: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("unreadable index");
    assert!(
        first.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: reading the index failed ("
        ) && first.ends_with("), so no file was changed."),
        "{first}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A Git directory that cannot hold a journal stops the fix before any file changes.
/// Why:
///  The record is what a later command reads to put a file back;
///  a fix that cannot write it must not
///  replace anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // gitDir = readOnlyDir(); await expect(installCorrections(...)).rejects.toThrow('journal cannot be written')
/// ```
#[test]
fn a_git_directory_that_cannot_hold_a_journal_stops_before_any_change() {
    let fixture_tree: Tree = tree("install-journal-unwritable");
    let read_only: PathBuf = fixture_tree.root.join("read-only");
    std::fs::create_dir(&read_only).expect("directory");
    std::fs::write(read_only.join("index"), b"index bytes").expect("index");
    std::fs::set_permissions(&read_only, std::fs::Permissions::from_mode(0o555)).expect("read-only");
    let probe: std::io::Result<std::fs::File> =
        std::fs::File::create(read_only.join("probe"));
    // Positive control: a process that ignores permissions cannot run this control.
    if probe.is_ok() {
        std::fs::set_permissions(&read_only, std::fs::Permissions::from_mode(0o755)).expect("restore");
        remove(fixture_tree.root.as_path());
        panic!("the read-only directory accepted a new file, so this control cannot run here");
    }
    let index: PathBuf = read_only.join("index");
    let refused: String =
        install_corrections(&fixture_tree.top, &index, changes().as_slice()).expect_err("unwritable");
    std::fs::set_permissions(&read_only, std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert!(
        refused.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: its journal cannot be written inside the Git directory ("
        ) && refused.ends_with("), so no file was changed."),
        "{refused}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A fix leaves no file of its own in the worktree,
///  so no `.gitignore` can expose one and no `git add`
///       can stage one.
/// Why:
///  A rule in `.gitignore` outranks `info/exclude` (measured here:
///  `!.cli-git-direct-fix-*` makes
///       `git add --all` stage the leftover),
///  so the only safe place for a fix's bytes is the Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await writeFile('.gitignore', '!.cli-git-direct-fix-*\n'); await installCorrections(...); await git('add', '--all'); expect(await git('ls-files')).toEqual(['keep.txt'])
/// ```
#[test]
fn a_fix_leaves_nothing_in_the_worktree_that_git_could_stage() {
    let root: PathBuf = fixture("install-no-worktree-files");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b"!.cli-git-direct-fix-*\n").expect("a rule that outranks");
    std::fs::write(repo.join("keep.txt"), b"a").expect("keep");
    git(repo.as_path(), &["add", "--", "keep.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=keep"]);
    let index: PathBuf = repo.join(".git/index");
    let changes: Vec<InstallChange> = vec![InstallChange {
        path: b"keep.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a"[..]),
        replacement: Rc::from(&b"a\n"[..]),
    }];
    // A writer holding the file open makes the fix keep its journal, so this control sees where a journal lives.
    let git_dir: PathBuf = repo.join(".git");
    let for_the_hook: (PathBuf, PathBuf) = (git_dir.clone(), repo.clone());
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |swapped: &Path| {
            if !swapped.ends_with("keep.txt") {
                return;
            }
            // Every journal, wherever this build keeps it, so the control's write is the same in both designs.
            for directory in [for_the_hook.0.as_path(), for_the_hook.1.as_path()] {
                for entry in std::fs::read_dir(directory).expect("a directory") {
                    let path: PathBuf = entry.expect("entry").path();
                    if path
                        .file_name()
                        .expect("name")
                        .to_string_lossy()
                        .starts_with("cli-git-direct-fix-")
                    {
                        std::fs::write(path.join("content"), b"a late save\n").expect("a late write");
                    }
                }
            }
        }));
    });
    let outcome: super::InstallOutcome =
        install_corrections(repo.as_path(), &index, changes.as_slice()).expect("the correction goes in");
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    assert_eq!(outcome.notices.len(), 1, "{outcome:?}");
    assert_eq!(
        leftovers(repo.as_path()),
        Vec::<String>::new(),
        "a fix keeps nothing in the worktree, not even a journal it must keep"
    );
    assert_eq!(journals(&git_dir).len(), 1, "the journal is inside the Git directory");
    git(repo.as_path(), &["add", "--all"]);
    let staged: std::process::Output = git(repo.as_path(), &["ls-files"]);
    assert_eq!(
        String::from_utf8_lossy(staged.stdout.as_slice()).into_owned(),
        String::from(".gitignore\nkeep.txt\n")
    );
    remove(root.as_path());
}

/// What:
///  A replacement never changes who owns the file it stands in for:
///  the owner is kept where this
///       process may set it,
///  and the write is refused where it may not.
/// Why:
///  The replacement is created as the user running the fix;
///  a failed ownership restore was
///       discarded before,
///  so the file quietly changed owner and the original inode was removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const kept = { mode: 0o644, uid: 0, gid: 0 }; const r = writeNew(path, bytes, kept); if (r.ok) expect(stat(path).uid).toBe(0); else expect(stat(path).uid).toBe(geteuid());
/// ```
#[test]
fn a_replacement_never_changes_the_owner() {
    let root: PathBuf = fixture("install-owner");
    let path: PathBuf = root.join("kept.txt");
    let asked: Kept = Kept {
        mode: 0o644,
        uid: 0,
        gid: 0,
    };
    match write_new(path.as_path(), b"x\n", &asked) {
        // A process that may give the file away gives it.
        Ok(()) => {
            let owner: u32 = std::fs::metadata(&path).expect("written").uid();
            assert_eq!(owner, 0, "an accepted write keeps the owner it was asked for");
        }
        Err(refusal) => {
            let text: String = refusal.to_string();
            assert!(text.contains("owner"), "{text}");
            // Nothing was handed to another account: the file, if it exists, is still ours.
            if let Ok(described) = std::fs::metadata(&path) {
                assert_eq!(described.uid(), unsafe { libc::geteuid() });
            }
        }
    }
    let _ = std::fs::remove_file(path.as_path());
    remove(root.as_path());
}

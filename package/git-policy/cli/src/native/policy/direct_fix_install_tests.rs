//! What:
//!  Controls for installing corrections:
//!  all files replaced with their modes and the
//!       index untouched,
//!  or no file changed,
//!  whichever step fails.
//! Why:
//!  A fix that left half its files replaced,
//!  a stray sibling file,
//!  or a changed index
//!      would damage the worktree it was asked to tidy.
//!  Every failure is provoked for real
//!      on a disposable directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await expect(installCorrections(root, index, changes)).rejects.toThrow('changed while');
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{
    INSTALL_SIBLING_PREFIX, InstallChange, PREPARED_FILE_EXCLUDE_PATTERN, Replacement,
    install_corrections, install_prepared, keep_prepared_files_out_of_git, prepare_replacements,
    read_index, rollback,
};
use crate::direct_fix_install_writes::{Kept, write_new};
use crate::candidate_object::CandidateMode;
use crate::test_support::{fixture, git, remove, repository};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A worktree with a regular file at the top,
///  an executable one below,
///  and an index file.
struct Tree {
    /// The disposable fixture,
    ///  removed at the end.
    root: PathBuf,
    /// The worktree's top level.
    top: PathBuf,
    /// The index file.
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

/// Every file name below `directory` that starts with the sibling prefix.
fn siblings(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("directory") {
        let path: PathBuf = entry.expect("entry").path();
        if path.is_dir() {
            found.extend(siblings(path.as_path()));
            continue;
        }
        let name: String = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        if name.starts_with(INSTALL_SIBLING_PREFIX) {
            found.push(name);
        }
    }
    return found;
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

/// Every file is replaced with its mode,
///  the index is the same,
///  and nothing is left beside them.
#[test]
fn corrections_are_installed_with_their_modes() {
    let fixture_tree: Tree = tree("install-ok");
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Ok(())
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
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    // A repository without an index file still has none afterwards, and nothing to install
    // installs nothing.
    std::fs::remove_file(&fixture_tree.index).expect("no index");
    assert_eq!(read_index(&fixture_tree.index).expect("absent"), None);
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, &[]),
        Ok(())
    );
    let again: Vec<InstallChange> = vec![InstallChange {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a\n"[..]),
        replacement: Rc::from(&b"b\n"[..]),
    }];
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, again.as_slice()),
        Ok(())
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"b\n"
    );
    assert!(!fixture_tree.index.exists());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A file kept private stays private:
///  the correction takes the destination's own
///       permission bits,
///  never a mode of its own,
///  and a symbolic link is refused instead of
///       replaced.
/// Why:
///   The person's token file,
///  key or local configuration sits at `0600`;
///  a replacement
///       created at `0644` would publish it to every local user (the owner's "never discard user
///       data").
///  A symbolic link at the path would be replaced by a regular file,
///  which is not the
///       file the policy read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await chmod(file, 0o600); await installCorrections(...); expect((await stat(file)).mode & 0o777).toBe(0o600)
/// ```
#[test]
fn a_private_file_stays_private() {
    let fixture_tree: Tree = tree("install-private");
    let private: PathBuf = fixture_tree.top.join("a.txt");
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o600)).expect("private");
    let script: PathBuf = fixture_tree.top.join("sub/run.sh");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).expect("private script");
    /// `MetadataExt` adds the numeric owner and group.
    use std::os::unix::fs::MetadataExt;
    let before: std::fs::Metadata = std::fs::metadata(&private).expect("a");
    let (owner_before, group_before): (u32, u32) = (before.uid(), before.gid());
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Ok(())
    );
    let kept: u32 = std::fs::metadata(&private).expect("a").permissions().mode() & 0o7777;
    assert_eq!(kept, 0o600, "{kept:o}");
    let kept_script: u32 = std::fs::metadata(&script).expect("script").permissions().mode() & 0o7777;
    assert_eq!(kept_script, 0o700, "{kept_script:o}");
    let after: std::fs::Metadata = std::fs::metadata(&private).expect("a");
    assert_eq!((after.uid(), after.gid()), (owner_before, group_before));
    assert_eq!(
        contents(&fixture_tree.top),
        (b"a\n".to_vec(), b"#!/bin/sh\n".to_vec())
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A selected file that is a symbolic link is left as it is and the fix refuses.
/// Why:
///   Reading it follows the link,
///  so the corrected bytes belong to another file,
///  and writing
///       them at the link's path would remove the link itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await symlink('elsewhere', path); await expect(installCorrections(...)).rejects.toThrow('symbolic link')
/// ```
#[test]
fn a_symbolic_link_is_refused_instead_of_replaced() {
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
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file is a symbolic link, and putting corrected bytes at its path would remove the link, so no file was changed."
        )
    );
    assert!(std::fs::symlink_metadata(&linked).expect("link").file_type().is_symlink());
    assert_eq!(std::fs::read(fixture_tree.top.join("a.txt")).expect("a"), b"a");
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// A file that changed under the fix,
///  is missing,
///  or has no name stops the fix before
/// any file changes,
///  and leaves nothing beside the files.
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
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    let missing: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("missing file");
    assert!(
        missing.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: reading a corrected file failed ("
        ) && missing.ends_with("), so no file was changed."),
        "{missing}"
    );
    assert!(!missing.contains("run.sh"), "{missing}");
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    let mut nameless: Vec<InstallChange> = changes();
    nameless[1].path = Vec::new();
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, nameless.as_slice()),
        Err(String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: Git named a corrected file this system cannot represent, so no file was changed."
        ))
    );
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    remove(fixture_tree.root.as_path());
}

/// A sibling that cannot be written stops the fix before any change.
#[test]
fn an_unwritable_directory_stops_before_any_change() {
    let fixture_tree: Tree = tree("install-unwritable");
    let sub: PathBuf = fixture_tree.top.join("sub");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o555)).expect("read-only");
    let probe: std::io::Result<std::fs::File> = std::fs::File::create(sub.join("probe"));
    // Positive control: a process that ignores permissions cannot run this control.
    if probe.is_ok() {
        std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).expect("restore");
        remove(fixture_tree.root.as_path());
        panic!("the read-only directory accepted a new file, so this control cannot run here");
    }
    let refused: String =
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice())
            .expect_err("unwritable");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert!(
        refused.starts_with("monochromatic-git-policy-cli fix could not install its corrections: writing a corrected file beside its original failed (")
            && refused.ends_with("), so no file was changed."),
        "{refused}"
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// An index that changed,
///  cannot be read again,
///  or cannot be read at all restores every file.
#[test]
fn an_index_that_is_not_as_read_restores_every_file() {
    let fixture_tree: Tree = tree("install-index");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    assert_eq!(
        install_prepared(prepared.as_slice(), &fixture_tree.index, &before),
        Err(String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: the index changed while monochromatic-git-policy-cli fix ran, so every corrected file was restored. Run monochromatic-git-policy-cli fix again."
        ))
    );
    assert_eq!(contents(&fixture_tree.top), untouched());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    // An index that turned into a directory cannot be read again.
    std::fs::remove_file(&fixture_tree.index).expect("remove index");
    std::fs::create_dir(&fixture_tree.index).expect("index directory");
    let prepared_again: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    let unreadable: String =
        install_prepared(prepared_again.as_slice(), &fixture_tree.index, &before)
            .expect_err("unreadable index");
    assert!(
        unreadable.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: reading the index again failed ("
        ) && unreadable
            .ends_with("), so every corrected file was restored. Run monochromatic-git-policy-cli fix again."),
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
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A write that lands after the fix read a file is kept:
///  the install refuses,
///  the
///       newer bytes stay,
///  and nothing is left beside the files.
/// Why:
///  The person's editor,
///  a formatter or another agent can save between the read and the
///       replacement;
///  those bytes are the newest state of the file and the owner's "never discard
///       user data" forbids losing them silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // prepareReplacements(); await write(current); await expect(installPrepared()).rejects.toThrow('changed while')
/// ```
#[test]
fn a_write_that_arrives_after_preparation_is_kept() {
    let fixture_tree: Tree = tree("install-late-write");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    std::fs::write(fixture_tree.top.join("a.txt"), b"newer edit\n").expect("late write");
    let refused: String = install_prepared(prepared.as_slice(), &fixture_tree.index, &before)
        .expect_err("a write that arrived after preparation");
    assert_eq!(
        refused,
        String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file changed while monochromatic-git-policy-cli fix ran, so every corrected file was restored. Run monochromatic-git-policy-cli fix again."
        )
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"newer edit\n".to_vec()
    );
    assert_eq!(contents(&fixture_tree.top).1, b"#!/bin/sh".to_vec());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A destination that cannot take its correction stops the install,
///  restores every file
///  already swapped,
///  and leaves nothing beside them.
/// Why:
///   A file deleted,
///  renamed or turned into a directory while the fix ran is not the file the
///       fix read;
///  the worktree must be as it was and the person must be told to run the fix again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // prepareReplacements(); await rm(second); await expect(installPrepared()).rejects.toThrow('in place failed')
/// ```
#[test]
fn a_destination_that_cannot_take_the_correction_restores_every_file() {
    let fixture_tree: Tree = tree("install-vanished");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    let failed: String = install_prepared(prepared.as_slice(), &fixture_tree.index, &before)
        .expect_err("the destination is gone");
    assert!(
        failed.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: putting a corrected file in place failed ("
        ) && failed.ends_with("), so every corrected file was restored. Run monochromatic-git-policy-cli fix again."),
        "{failed}"
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A destination that changed its kind is refused the same way a changed file is.
/// Why:
///   A directory at the path holds no bytes to compare,
///  so the correction must not stay there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // prepareReplacements(); await rm(second); await mkdir(second); await expect(installPrepared()).rejects.toThrow('changed while')
/// ```
#[test]
fn a_destination_that_changed_its_kind_stops_the_install() {
    let fixture_tree: Tree = tree("install-kind");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Replacement> =
        prepare_replacements(&fixture_tree.top, changes().as_slice()).expect("prepared");
    std::fs::remove_file(fixture_tree.top.join("sub/run.sh")).expect("remove");
    std::fs::create_dir_all(fixture_tree.top.join("sub/run.sh/inside")).expect("directory");
    let failed: String = install_prepared(prepared.as_slice(), &fixture_tree.index, &before)
        .expect_err("the destination became a directory");
    assert_eq!(
        failed,
        String::from(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file changed while monochromatic-git-policy-cli fix ran, so every corrected file was restored. Run monochromatic-git-policy-cli fix again."
        )
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    // The directory is back where it was, and nothing of the fix stays beside it.
    assert!(fixture_tree.top.join("sub/run.sh/inside").is_dir());
    assert_eq!(siblings(&fixture_tree.top), Vec::<String>::new());
    remove(fixture_tree.root.as_path());
}

/// What:
///  A restore that cannot swap the displaced bytes back keeps them beside the file and says so.
/// Why:
///   The bytes a fix displaced are the newest state of that file;
///  a restore that fails must
///       never remove them,
///  and the person must be told where they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await rollback([{ destination, prepared: missing, original }]) // throws naming the prefix
/// ```
#[test]
fn a_restore_that_cannot_swap_back_keeps_the_original_bytes_beside_the_file() {
    let fixture_tree: Tree = tree("install-lost-backup");
    let lost: Vec<Replacement> = vec![Replacement {
        destination: fixture_tree.top.join("a.txt"),
        prepared: fixture_tree.top.join(".cli-git-direct-fix-gone.new"),
        original: Rc::from(&b"a"[..]),
    }];
    let refused: String = rollback(lost.as_slice()).expect_err("nothing to swap back");
    assert!(
        refused.starts_with("restoring a corrected file failed (")
            && refused.ends_with(
                "); its original bytes remain beside it in a file whose name starts with .cli-git-direct-fix-"
            ),
        "{refused}"
    );
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a"
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  The rule that keeps a fix's prepared files out of Git is written once,
///  keeps what the
///       person wrote,
///  and makes `git add --all` stage nothing of theirs at any depth.
/// Why:
///  A killed fix leaves its prepared file beside a corrected file,
///  and a fix's scope is the whole
///       worktree:
///  without the rule the next fix reads the leftover as content and a later
///       `git add --all` publishes it under a name the person never chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await keepPreparedFilesOutOfGit('.git'); await write('sub/.cli-git-direct-fix-1-0.new'); await git('add', '--all'); expect(staged).toEqual(['keep.txt'])
/// ```
#[test]
fn the_prepared_files_of_a_fix_are_never_staged() {
    let root: PathBuf = fixture("install-exclude");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let exclude: PathBuf = repo.join(".git/info/exclude");
    std::fs::write(&exclude, b"# the person's own rule\n*.tmp").expect("exclude");
    keep_prepared_files_out_of_git(repo.join(".git").as_path()).expect("the rule is written");
    let written: String = std::fs::read_to_string(&exclude).expect("exclude");
    assert!(written.contains("# the person's own rule"), "{written}");
    // A last line without a line break still ends before the block this writes.
    assert!(
        written.contains("*.tmp\n# monochromatic-git-policy-cli:"),
        "{written}"
    );
    assert!(written.contains(&format!("\n{PREPARED_FILE_EXCLUDE_PATTERN}*\n")), "{written}");
    // A second fix writes nothing: the rule is already there.
    let before: Vec<u8> = std::fs::read(&exclude).expect("exclude");
    keep_prepared_files_out_of_git(repo.join(".git").as_path()).expect("the rule is written");
    assert_eq!(std::fs::read(&exclude).expect("exclude"), before);
    // A leftover at the top, one level down and two levels down is invisible to `git add --all`.
    std::fs::write(repo.join("keep.txt"), b"keep\n").expect("keep");
    std::fs::write(repo.join(".cli-git-direct-fix-1-2-0.new"), b"one\n").expect("top");
    std::fs::create_dir_all(repo.join("sub/deeper")).expect("directories");
    std::fs::write(repo.join("sub/.cli-git-direct-fix-1-2-1.new"), b"two\n").expect("sub");
    std::fs::write(repo.join("sub/deeper/.cli-git-direct-fix-1-2-2.new"), b"three\n").expect("deeper");
    git(repo.as_path(), &["add", "--all"]);
    let staged: std::process::Output = git(repo.as_path(), &["ls-files"]);
    assert_eq!(
        String::from_utf8_lossy(staged.stdout.as_slice()).into_owned(),
        String::from("keep.txt\n")
    );
    remove(root.as_path());
}

/// What:
///  The set-user-id and set-group-id bits survive a fix.
/// Why:
///  Restoring the owner clears those bits on Linux even when the owner and group are unchanged
///  (`chmod 4755 f; chown $(id -u):$(id -g) f` leaves `755`),
///  so a fix that
///       restored the mode first and the owner second silently removed the privilege a file was
///       given on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await chmod(file, 0o4755); await installCorrections(...); expect((await stat(file)).mode & 0o7777).toBe(0o4755)
/// ```
#[test]
fn the_special_permission_bits_survive_a_fix() {
    let fixture_tree: Tree = tree("install-special-bits");
    let private: PathBuf = fixture_tree.top.join("a.txt");
    let script: PathBuf = fixture_tree.top.join("sub/run.sh");
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o4755)).expect("set-user-id");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o2755)).expect("set-group-id");
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.index, changes().as_slice()),
        Ok(())
    );
    let kept_private: u32 = std::fs::metadata(&private).expect("a").permissions().mode() & 0o7777;
    assert_eq!(kept_private, 0o4755, "{kept_private:o}");
    let kept_script: u32 = std::fs::metadata(&script).expect("script").permissions().mode() & 0o7777;
    assert_eq!(kept_script, 0o2755, "{kept_script:o}");
    remove(fixture_tree.root.as_path());
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
///  Only privilege can give
///       a file to another account,
///  so this control accepts either answer and refuses each wrong one:
///  kept, or refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const kept = { mode: 0o644, uid: 0, gid: 0 }; const r = writeNew(path, bytes, kept); if (r.ok) expect(stat(path).uid).toBe(0); else expect(stat(path).uid).toBe(geteuid());
/// ```
#[test]
fn a_replacement_never_changes_the_owner() {
    let root: PathBuf = fixture("install-owner");
    let path: PathBuf = root.join("kept.txt");
    /// `MetadataExt` adds the numeric owner;
    ///  `libc::geteuid` is this process's.
    use std::os::unix::fs::MetadataExt;
    let asked: Kept = Kept {
        mode: 0o644,
        uid: 0,
        gid: 0,
    };
    match write_new(path.as_path(), b"x\n", &asked) {
        // A process that may give the file away gives it, and the bits survive the change.
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

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
};
use crate::direct_fix_install_journal_sweep::reap_interrupted_installs;
use crate::candidate_object::CandidateMode;
use crate::direct_fix_install_attributes::{Attribute, attributes_of};
use crate::direct_fix_install_journal::{RECORD_FILE, read_record};
use crate::direct_fix_install_writes::{Kept, write_new};
use crate::test_support::{fixture, git, remove, repository};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// What:
///  Give `path` the extended attribute `name` with `value`;
///  false when the filesystem refuses it.
/// Why:
///  A control that plants an attribute must know whether the filesystem can hold one,
///  so a refusal is reported as a control that did not run rather than as a defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const planted = (path, name, value) => { try { setxattrSync(path, name, value); return true } catch { return false } };
/// ```
fn set_attribute(path: &Path, name: &[u8], value: &[u8]) -> bool {
    let (Ok(name_text), Ok(path_text)) = (CString::new(name), CString::new(path.as_os_str().as_bytes())) else {
        return false;
    };
    // SAFETY: both strings are NUL-terminated and outlive the call; `value` holds `value.len()` initialised bytes.
    let result: libc::c_int = unsafe {
        libc::lsetxattr(
            path_text.as_ptr(),
            name_text.as_ptr(),
            value.as_ptr().cast(),
            value.len(),
            0,
        )
    };
    return result == 0;
}

/// What:
///  The value of the extended attribute `name` of `path`,
///  or nothing when it has none.
/// Why:
///  The control must read the person's data back from the same name and by the same rule the fix does,
///  never through the file's own bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const attribute = (path, name) => { try { return getxattrSync(path, name) } catch { return undefined } };
/// ```
fn attribute(path: &Path, name: &[u8]) -> Option<Vec<u8>> {
    let (Ok(name_text), Ok(path_text)) = (CString::new(name), CString::new(path.as_os_str().as_bytes())) else {
        return None;
    };
    // SAFETY: a null buffer with size 0 asks the system for the length of the value.
    let size: isize = unsafe { libc::lgetxattr(path_text.as_ptr(), name_text.as_ptr(), std::ptr::null_mut(), 0) };
    if size < 0 {
        return None;
    }
    let mut value: Vec<u8> = vec![0; usize::try_from(size).ok()?];
    // SAFETY: `value` holds `size` writable bytes.
    let read: isize = unsafe {
        libc::lgetxattr(
            path_text.as_ptr(),
            name_text.as_ptr(),
            value.as_mut_ptr().cast(),
            value.len(),
        )
    };
    if read < 0 {
        return None;
    }
    value.truncate(usize::try_from(read).ok()?);
    return Some(value);
}

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


/// What:
///  An access-control list that grants a second entry beside the owner's,
/// as the bytes the kernel stores under `system.posix_acl_access`.
/// Why:
///  An access-control list decides who may read the file,
///  and it is the one extended attribute with a shape of its own;
///  a control that plants it must build exactly what the kernel checks,
///  so the fix is shown to carry a list and not only a user attribute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const blob = pack('<I', 2) + [[1, 6, -1], [2, 4, owner], [4, 4, -1], [0x10, 4, -1], [0x20, 0, -1]].map(([tag, perm, id]) => pack('<HHI', tag, perm, id)).join('');
/// ```
fn access_list() -> Vec<u8> {
    return access_list_with_owner(0o6, 0);
}

/// What:
///  The same list with the owner's own permission given,
/// as the bytes a directory stores under `system.posix_acl_default`.
/// Why:
///  A default list is what a directory hands to every file created inside it,
///  and the owner's own permission in it decides what the created object may be used for:
///  a list whose owner may not enter leaves a created directory unusable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const blob = acl(2, [[1, ownerPermission, -1], [2, 4, namedAccount], [4, 4, -1], [0x10, 4, -1], [0x20, 0, -1]])
/// ```
fn access_list_with_owner(owner_permission: u16, named_account: u32) -> Vec<u8> {
    let mut blob: Vec<u8> = u32::to_le_bytes(2).to_vec();
    let entries: [(u16, u16, u32); 5] = [
        (0x01, owner_permission, u32::MAX),
        (0x02, 0o4, named_account),
        (0x04, 0o4, u32::MAX),
        (0x10, 0o4, u32::MAX),
        (0x20, 0o0, u32::MAX),
    ];
    for (tag, permission, identifier) in entries {
        blob.extend_from_slice(u16::to_le_bytes(tag).as_slice());
        blob.extend_from_slice(u16::to_le_bytes(permission).as_slice());
        blob.extend_from_slice(u32::to_le_bytes(identifier).as_slice());
    }
    return blob;
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice()),
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, &[]),
        Ok(super::InstallOutcome::default())
    );
    let again: Vec<InstallChange> = vec![InstallChange {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a\n"[..]),
        replacement: Rc::from(&b"b\n"[..]),
    }];
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, again.as_slice()),
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice()),
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

/// What:
///  The extended attributes a file carries survive a fix,
///  and one the replacement cannot be given stops the fix with a word.
/// Why:
///  A user attribute is the person's data outside the file's bytes,
///  which Git never stores and no commit can bring back;
///  a replacement that drops it deletes the only copy without saying so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // setxattr(a, 'user.draft', 'only-copy-of-draft'); await installCorrections(...); expect(getxattr(a, 'user.draft')).toBe('only-copy-of-draft')
/// ```
#[test]
fn the_extended_attributes_of_a_file_survive_a_fix() {
    let fixture_tree: Tree = tree("install-attributes");
    let path: PathBuf = fixture_tree.top.join("a.txt");
    // The value is the person's only copy: nothing in Git, and nothing in the fix's journal, holds it.
    if !set_attribute(path.as_path(), b"user.draft", b"only-copy-of-draft") {
        eprintln!(
            "XATTR CONTROL NOT RUN: this filesystem refuses user extended attributes ({})",
            std::io::Error::last_os_error()
        );
        remove(fixture_tree.root.as_path());
        return;
    }
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice()),
        Ok(super::InstallOutcome::default())
    );
    assert_eq!(
        std::fs::read(&path).expect("a"),
        b"a\n",
        "the fix installed its correction"
    );
    assert_eq!(
        attribute(path.as_path(), b"user.draft"),
        Some(b"only-copy-of-draft".to_vec()),
        "the data the file carried outside its bytes is still there"
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}


/// What:
///  The access-control list of a file survives a fix,
///  so who may read the file does not change under the person's hands.
/// Why:
///  The list is an extended attribute the kernel reads to decide access;
///  a replacement that carried only the mode would widen or narrow the file
///  for every account the person granted or denied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // setxattr(a, 'system.posix_acl_access', acl); await installCorrections(...); expect(getxattr(a, 'system.posix_acl_access')).toEqual(acl)
/// ```
#[test]
fn the_access_control_list_of_a_file_survives_a_fix() {
    let fixture_tree: Tree = tree("install-access-list");
    let path: PathBuf = fixture_tree.top.join("a.txt");
    let list: Vec<u8> = access_list();
    if !set_attribute(path.as_path(), b"system.posix_acl_access", list.as_slice()) {
        eprintln!(
            "ACL CONTROL NOT RUN: this filesystem refuses an access-control list ({})",
            std::io::Error::last_os_error()
        );
        remove(fixture_tree.root.as_path());
        return;
    }
    assert_eq!(
        attribute(path.as_path(), b"system.posix_acl_access"),
        Some(list.clone()),
        "the list is on the file before the fix"
    );
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, one_change().as_slice()),
        Ok(super::InstallOutcome::default())
    );
    assert_eq!(std::fs::read(&path).expect("a"), b"a\n", "the fix installed its correction");
    assert_eq!(
        attribute(path.as_path(), b"system.posix_acl_access"),
        Some(list),
        "who may read the file is unchanged"
    );
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    remove(fixture_tree.root.as_path());
}

/// An account the person's file never named,
///  and one this control can plant without owning it.
const ANOTHER_ACCOUNT: u32 = 4321;

/// What:
///  A fix never grants an account access the file never granted,
///  by the list a directory hands to every file created inside it.
/// Why:
///  The replacement is created in the Git directory,
///  and a default access-control list there is copied onto it;
///  the corrected file would then be readable by an account the person's file never named,
///  and comparing the displaced original cannot see it,
///  because the unexpected list belongs to the replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // setxattr(gitDir, 'system.posix_acl_default', acl); await installCorrections(...); expect(getxattr(a, 'system.posix_acl_access')).toBeUndefined()
/// ```
#[test]
fn a_replacement_never_gains_an_access_control_list_the_file_never_had() {
    let fixture_tree: Tree = tree("install-inherited-access-list");
    let path: PathBuf = fixture_tree.top.join("a.txt");
    // The Git directory holds every journal, so a default list there is copied to every replacement created in it.
    if !set_attribute(
        fixture_tree.root.as_path(),
        b"system.posix_acl_default",
        access_list_with_owner(0o7, ANOTHER_ACCOUNT).as_slice(),
    ) {
        eprintln!(
            "ACL CONTROL NOT RUN: this filesystem refuses a default access-control list ({})",
            std::io::Error::last_os_error()
        );
        remove(fixture_tree.root.as_path());
        return;
    }
    let carried: Vec<Attribute> = attributes_of(path.as_path(), "reading a file").expect("read");
    assert_eq!(
        attribute(path.as_path(), b"system.posix_acl_access"),
        None,
        "the file names no account before the fix"
    );
    let mode: u32 = std::fs::metadata(&path).expect("a").permissions().mode() & 0o7777;
    assert_eq!(
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, one_change().as_slice()),
        Ok(super::InstallOutcome::default())
    );
    assert_eq!(std::fs::read(&path).expect("a"), b"a\n", "the fix installed its correction");
    assert_eq!(
        attribute(path.as_path(), b"system.posix_acl_access"),
        None,
        "the corrected file grants nobody access the original did not"
    );
    assert_eq!(
        attributes_of(path.as_path(), "reading a file").expect("read"),
        carried,
        "the corrected file carries exactly what the file it replaced did"
    );
    let after: u32 = std::fs::metadata(&path).expect("a").permissions().mode() & 0o7777;
    assert_eq!(after, mode, "and the permission bits it had");
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice())
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice()),
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice())
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, nameless.as_slice()),
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
///  An attribute the person changed while the fix ran is refused and kept,
///  like a change of its bytes or its mode.
/// Why:
///  The fix installs the attributes it read;
///  a person who saves a value into the file's own attribute while the fix
///  runs would get the older value back,
///  and the displaced comparison of bytes,
///  mode and owner cannot see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const prepared = prepare(...);
/// // setxattr(a, 'user.draft', 'second');
/// // const refused = install(prepared); // 'changed while'
/// // expect(getxattr(a, 'user.draft')).toBe('second');
/// ```
#[test]
fn an_attribute_change_made_while_the_fix_ran_is_refused_and_kept() {
    let fixture_tree: Tree = tree("install-attribute-change");
    let path: PathBuf = fixture_tree.top.join("a.txt");
    if !set_attribute(path.as_path(), b"user.draft", b"first") {
        eprintln!(
            "XATTR CONTROL NOT RUN: this filesystem refuses user extended attributes ({})",
            std::io::Error::last_os_error()
        );
        remove(fixture_tree.root.as_path());
        return;
    }
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    assert!(set_attribute(path.as_path(), b"user.draft", b"second"), "the person saves a newer value");
    let mut notices: Vec<String> = Vec::new();
    let refused: String = install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("an attribute changed under the fix");
    assert!(
        refused.starts_with(
            "monochromatic-git-policy-cli fix could not install its corrections: a selected file changed while monochromatic-git-policy-cli fix ran"
        ),
        "{refused}"
    );
    assert_eq!(
        attribute(path.as_path(), b"user.draft"),
        Some(b"second".to_vec()),
        "the person's newer value is back on the file"
    );
    assert_eq!(std::fs::read(&path).expect("a"), b"a", "the fix left the person's bytes alone");
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
    // What: The rollback puts back what it displaced, so the file holds the bytes the fix found; the save the swap
    //       took out of it is in the journal the message names.
    // Why:  A rollback decides nothing about a write it did not make, and a swap is what takes the bytes out.
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a".to_vec(),
        "the file is back to what the fix found"
    );
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].starts_with("monochromatic-git-policy-cli kept ")
            && notices[0].contains("cli-git-direct-fix-")
            && notices[0].contains("/content"),
        "{notices:?}"
    );
    // What: The message says where the save is, which is the journal it names, and never that the file holds it.
    // Why:  The file was just put back to the bytes the fix found, so a person who read "the file itself holds what the
    //       other writer put there" would look at a file holding the opposite. Finding of the fifth review.
    assert!(
        !notices[0].contains("the file itself holds"),
        "the notice says the file holds the other writer's save, and the file holds what the fix found: {}",
        notices[0]
    );
    assert!(
        notices[0].contains("in that journal"),
        "the notice does not say the save is in the journal: {}",
        notices[0]
    );
    let kept: Vec<PathBuf> = journals(&fixture_tree.root);
    assert_eq!(kept.len(), 1, "one journal stays");
    assert_eq!(
        std::fs::read(content_of(kept[0].as_path())).expect("the kept bytes"),
        b"newer edit\n".to_vec(),
        "the person's save is kept"
    );
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
///  The state a kill between the swap and the judgement leaves is reported by the next fix,
///  and nothing
///       of the person's is moved or removed.
/// Why:
///  The file holds a correction nothing verified,
///  and the only copy of the edit it displaced sits in the
///       journal;
///  the person must be told exactly that,
///  and no exchange may decide for them which of the two
///       belongs at the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // exchange(contentOf(p.journal), p.destination); markOwnerDead(p); reap(); expect(readFile(a)).toBe('a\n'); expect(contentOf(journal)).toBe('newer edit\n')
/// ```
#[test]
fn a_kill_between_the_swap_and_the_judgement_is_reported_and_kept() {
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
    mark_writer_dead(prepared[0].journal.as_path());
    let mut notices: Vec<String> = Vec::new();
    reap_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a\n".to_vec(),
        "the file keeps what the interrupted fix put there"
    );
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].starts_with("monochromatic-git-policy-cli left ")
            && notices[0].contains("cli-git-direct-fix-"),
        "{notices:?}"
    );
    assert_eq!(journals(&fixture_tree.root).len(), 1, "the journal stays");
    assert_eq!(
        std::fs::read(content_of(journals(&fixture_tree.root)[0].as_path())).expect("kept"),
        b"newer edit\n".to_vec(),
        "the person's edit is kept where the message says"
    );
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
    reap_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    assert_eq!(journals(&fixture_tree.root), Vec::<PathBuf>::new());
    assert!(
        notices[0].contains("had not reached its file, so nothing had changed"),
        "{notices:?}"
    );
    // Now a swap happens and the owner dies: the journal holds the person's bytes and must stay.
    let swapped: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    std::fs::write(fixture_tree.top.join("a.txt"), b"newer edit\n").expect("a save");
    super::exchange(
        content_of(swapped[0].journal.as_path()).as_path(),
        &fixture_tree.top.join("a.txt"),
    )
    .expect("swap");
    mark_writer_dead(swapped[0].journal.as_path());
    let mut second_notices: Vec<String> = Vec::new();
    reap_interrupted_installs(fixture_tree.root.as_path(), &mut second_notices);
    assert_eq!(journals(&fixture_tree.root).len(), 1, "the journal stays");
    assert!(
        second_notices[0].contains("holds bytes from a fix whose process ended"),
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
        install_corrections(&fixture_tree.top, &fixture_tree.root, &fixture_tree.index, changes().as_slice())
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
        install_corrections(&fixture_tree.top, &read_only, &index, changes().as_slice())
            .expect_err("unwritable");
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
        install_corrections(repo.as_path(), repo.join(".git").as_path(), &index, changes.as_slice()).expect("the correction goes in");
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
///  A journal lands in the worktree's own Git directory,
///  never beside an alternate index a caller named.
/// Why:
///  `GIT_INDEX_FILE` can point inside the worktree,
///  and the parent of that index is then an ordinary
///       directory a later `git add --all` stages:
///  the journal's `content` is displaced user content,
/// so
///       it would enter the index.
///  The Git directory is the only place Git never stages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const alt = join(worktree, 'alt-index'); writeIndex(alt); installCorrections(root, gitDir, alt, changes); expect(readdir(worktree)).not.toContain('cli-git-direct-fix-*')
/// ```
#[test]
fn a_journal_never_lands_beside_an_alternate_index() {
    let root: PathBuf = fixture("install-alternate-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"a").expect("a");
    // An alternate index inside the worktree, as `GIT_INDEX_FILE` allows.
    let alternate: PathBuf = repo.join("alt-index");
    std::fs::write(&alternate, b"index bytes").expect("the alternate index");
    let changes: Vec<InstallChange> = vec![InstallChange {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        original: Rc::from(&b"a"[..]),
        replacement: Rc::from(&b"a\n"[..]),
    }];
    // A writer holding the file open makes the fix keep its journal, so this control sees where a journal lives.
    let for_the_hook: (PathBuf, PathBuf) = (repo.join(".git"), repo.clone());
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |swapped: &Path| {
            if !swapped.ends_with("a.txt") {
                return;
            }
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
    let outcome: super::InstallOutcome = install_corrections(
        repo.as_path(),
        repo.join(".git").as_path(),
        alternate.as_path(),
        changes.as_slice(),
    )
    .expect("the correction goes in");
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    assert_eq!(outcome.notices.len(), 1, "a journal stays: {outcome:?}");
    assert_eq!(
        std::fs::read(repo.join("a.txt")).expect("a"),
        b"a\n".to_vec()
    );
    // Nothing of the fix is beside the alternate index or anywhere else in the worktree.
    assert_eq!(
        leftovers(repo.as_path()),
        Vec::<String>::new(),
        "the worktree holds no journal"
    );
    assert_eq!(
        journals(repo.join(".git").as_path()).len(),
        1,
        "the journal this fix must keep is in the Git directory"
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
        attributes: Vec::new(),
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

/// What:
///  A journal whose content is not the correction is never exchanged into its file,
///  and the person's
///       saved bytes stay where they are.
/// Why:
///  A kill between creating `content` and writing its bytes leaves an empty file,
///  and a recovery that
///  exchanges on the strength of the destination's digest would put that empty file over a save.
///  Repeating the recovery must change nothing either,
/// which is also what keeps two recoveries
///  from undoing each other's exchange of one journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // truncate(contentOf(p.journal)); writeFile(p.destination, replacement); markOwnerDead(p); recover(); recover(); expect(readFile(p.destination)).toBe(replacement)
/// ```
#[test]
fn a_recovery_never_exchanges_and_changes_nothing_twice() {
    let fixture_tree: Tree = tree("install-recovery-safe");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let content: PathBuf = content_of(prepared[0].journal.as_path());
    // A kill in the middle of writing the corrected bytes: the journal's content says nothing.
    std::fs::write(&content, b"").expect("a half-written content");
    // The person saves exactly the bytes the fix meant to write.
    std::fs::write(fixture_tree.top.join("a.txt"), b"a\n").expect("a save");
    mark_writer_dead(prepared[0].journal.as_path());
    let mut notices: Vec<String> = Vec::new();
    reap_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    reap_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    assert_eq!(
        std::fs::read(fixture_tree.top.join("a.txt")).expect("a"),
        b"a\n".to_vec(),
        "no exchange may put the journal's bytes over the person's save"
    );
    assert_eq!(
        std::fs::read(&content).expect("kept content"),
        b"".to_vec(),
        "the journal's bytes are untouched"
    );
    assert_eq!(journals(&fixture_tree.root).len(), 1, "the journal stays");
    assert_eq!(notices.len(), 2, "each fix names it: {notices:?}");
    for notice in &notices {
        assert!(notice.contains("cli-git-direct-fix-"), "{notice}");
    }
    remove(fixture_tree.root.as_path());
}

/// What:
///  A save that lands after a rollback's ownership check is never deleted:
///  the rollback judges what it
///       took out of the exchange,
/// which is the bytes that were really there.
/// Why:
///  A read of the destination followed by an exchange has a window,
///  and a save that lands in it was
///  moved into the journal and then removed with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // beforeRollback = () => writeFileSync(destination, 'newer edit\n'); install(...); expect(theBytesExistSomewhere).toBe(true)
/// ```
#[test]
fn a_save_that_lands_after_a_rollbacks_check_is_kept() {
    let fixture_tree: Tree = tree("install-rollback-late-save");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let destination: PathBuf = fixture_tree.top.join("a.txt");
    super::BEFORE_ROLLBACK_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |rolled_back: &Path| {
            if rolled_back.ends_with("a.txt") {
                // The rollback read this file and found the fix's bytes; an editor saves before its swap.
                std::fs::write(rolled_back, b"newer edit\n").expect("a save in the window");
            }
        }));
    });
    // The index changed, so the fix must roll back everything it installed.
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    let mut notices: Vec<String> = Vec::new();
    let failed: String = install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("the index changed");
    super::BEFORE_ROLLBACK_SWAP.with(|hook| { *hook.borrow_mut() = None });
    assert!(failed.contains("the index changed"), "{failed}");
    // The save is never deleted: it is at the file or in a journal the message names.
    let at_the_file: bool = std::fs::read(&destination).is_ok_and(|bytes| { return bytes == b"newer edit\n"; });
    let in_a_journal: bool = journals(&fixture_tree.root).iter().any(|journal| {
        return std::fs::read(content_of(journal.as_path())).is_ok_and(|bytes| { return bytes == b"newer edit\n"; });
    });
    assert!(
        at_the_file || in_a_journal,
        "the saved bytes are gone: at_the_file={at_the_file} journals={:?}",
        journals(&fixture_tree.root)
    );
    assert!(!notices.is_empty(), "the person is told where they are");
    remove(fixture_tree.root.as_path());
}

/// What:
///  A rollback never takes a permission change the fix did not make,
///  and never follows a symbolic link
///       to decide a file is its own.
/// Why:
///  Bytes alone do not say whose bytes they are:
///  a file narrowed to `0600` after the install was the
///       person's act,
///  and a link whose target happens to hold the correction is not the correction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // afterSwap = () => chmod(destination, 0o600); writeIndex(other); install(); expect(journalKept).toBe(true); expect(modeOfTheKeptFile).toBe(0o600)
/// ```
#[test]
fn a_rollback_never_takes_a_mode_change_and_never_follows_a_link() {
    let fixture_tree: Tree = tree("install-rollback-mode");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |swapped: &Path| {
            if swapped.ends_with("a.txt") {
                // The person narrows the installed file; its bytes do not change.
                std::fs::set_permissions(swapped, std::fs::Permissions::from_mode(0o600))
                    .expect("narrow");
            }
        }));
    });
    // The index is not what the fix read, so it must roll back what it installed.
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    let mut notices: Vec<String> = Vec::new();
    install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("the index changed");
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    // The narrowed inode is on disk, in the journal the message names.
    let kept: Vec<PathBuf> = journals(&fixture_tree.root);
    assert_eq!(kept.len(), 1, "{notices:?}");
    let kept_mode: u32 = std::fs::metadata(content_of(kept[0].as_path()))
        .expect("the kept bytes")
        .permissions()
        .mode()
        & 0o7777;
    assert_eq!(kept_mode, 0o600, "the person's mode is kept: {kept_mode:o}");
    assert!(
        notices.iter().any(|notice| return notice.contains("cli-git-direct-fix-")),
        "{notices:?}"
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A symbolic link a person put at a corrected path is never removed by a rollback.
/// Why:
///  Reading through the link finds the correction's bytes in its target,
///  so an ownership test that
///       follows links takes the person's link for the fix's own file and unlinks it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // afterSwap = () => { unlink(destination); symlink(elsewhere, destination) }; writeIndex(other); install(); expect(theLinkStillExists)
/// ```
#[test]
fn a_rollback_never_removes_a_symbolic_link_it_did_not_make() {
    let fixture_tree: Tree = tree("install-rollback-link");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let destination: PathBuf = fixture_tree.top.join("a.txt");
    // A file elsewhere whose bytes are the correction, and a link to it where the corrected file is.
    let elsewhere: PathBuf = fixture_tree.top.join("elsewhere.txt");
    std::fs::write(&elsewhere, b"a\n").expect("the target");
    let target: PathBuf = elsewhere.clone();
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |swapped: &Path| {
            if swapped.ends_with("a.txt") {
                std::fs::remove_file(swapped).expect("remove the corrected file");
                std::os::unix::fs::symlink(target.as_path(), swapped).expect("the person's link");
            }
        }));
    });
    std::fs::write(&fixture_tree.index, b"changed index").expect("change index");
    let mut notices: Vec<String> = Vec::new();
    install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    )
    .expect_err("the index changed");
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    // The link is not the fix's file: it stays, either at the path or in the journal the message names.
    let at_the_path: bool = std::fs::symlink_metadata(&destination)
        .is_ok_and(|found| return found.file_type().is_symlink());
    let in_a_journal: bool = journals(&fixture_tree.root).iter().any(|journal| {
        return std::fs::symlink_metadata(content_of(journal.as_path()))
            .is_ok_and(|found| return found.file_type().is_symlink());
    });
    assert!(
        at_the_path || in_a_journal,
        "the person's link is gone: journals={:?}",
        journals(&fixture_tree.root)
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A journal a fix cannot remove is named,
///  so a later command never acts on a successful fix behind the
///       person's back.
/// Why:
///  A removal that fails leaves the journal behind;
///  a fix that says nothing about it reports success
///  while the worktree's evidence of what it did is out of its hands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // afterSwap = () => chmod(journal, 0o500); install(...); expect(notices.some(n => n.includes('could not be removed'))).toBe(true)
/// ```
#[test]
fn a_journal_a_fix_cannot_remove_is_named() {
    let fixture_tree: Tree = tree("install-cleanup-fails");
    let before: Option<Vec<u8>> = read_index(&fixture_tree.index).expect("index");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let journal: PathBuf = prepared[0].journal.clone();
    let for_the_hook: PathBuf = journal.clone();
    super::AFTER_SWAP.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move |swapped: &Path| {
            if swapped.ends_with("a.txt") {
                // Write permission leaves the journal's directory while every name in it stays readable.
                std::fs::set_permissions(&for_the_hook, std::fs::Permissions::from_mode(0o500))
                    .expect("make the journal unremovable");
            }
        }));
    });
    let mut notices: Vec<String> = Vec::new();
    let outcome: Result<(), String> = install_prepared(
        prepared.as_slice(),
        &fixture_tree.index,
        &before,
        &mut notices,
    );
    super::AFTER_SWAP.with(|hook| { *hook.borrow_mut() = None });
    std::fs::set_permissions(&journal, std::fs::Permissions::from_mode(0o700)).expect("restore");
    assert_eq!(outcome, Ok(()), "the correction is installed");
    assert!(
        notices.iter().any(|notice| return notice.contains("could not be removed")),
        "the journal that stays is named: {notices:?}"
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A replacement keeps the group of the file it stands in for,
///  even where the file system gives the new file
///       another one.
/// Why:
///  A directory with its set-group-ID bit gives every new file that directory's group,
///  so the
///       replacement's group is not the process's:
///  comparing the two lets a `0640` file change which group
///  may read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // setgidDir(groupB); writeNew(file, bytes, { gid: groupA }); expect(stat(file).gid).toBe(groupA)
/// ```
#[test]
fn a_replacement_keeps_the_group_the_file_had() {
    let root: PathBuf = fixture("install-group");
    let shared: PathBuf = root.join("shared");
    std::fs::create_dir(&shared).expect("directory");
    // A group this process belongs to and that is not its effective group, so the directory can hand it out;
    // `getgroups` fills the caller's supplementary groups.
    let mut groups: [libc::gid_t; 32] = [0; 32];
    // SAFETY: `getgroups` writes at most 32 ids into the array the caller owns.
    let counted: libc::c_int = unsafe { libc::getgroups(32, groups.as_mut_ptr()) };
    assert!(counted >= 0, "the group list cannot be read");
    let own: u32 = unsafe { libc::getegid() };
    let other: Option<u32> = groups[..usize::try_from(counted).expect("a count")].iter().map(|id| return *id).find(|id| return *id != own);
    let Some(other_gid) = other else {
        // Positive control: without a second group this control cannot run, and says so.
        remove(root.as_path());
        panic!("this process belongs to one group only, so a set-group-ID directory cannot hand out another");
    };
    let mut shared_for_chgrp: std::process::Command = std::process::Command::new("chgrp");
    let changed: std::process::Output = shared_for_chgrp
        .arg(other_gid.to_string())
        .arg(&shared)
        .output()
        .expect("chgrp runs");
    assert!(changed.status.success(), "{}", String::from_utf8_lossy(&changed.stderr));
    std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(0o2755)).expect("set-group-ID");
    let path: PathBuf = shared.join("kept.txt");
    let asked: Kept = Kept {
        mode: 0o640,
        uid: unsafe { libc::geteuid() },
        gid: own,
        attributes: carried_by_a_new_file_in(shared.as_path()),
    };
    write_new(path.as_path(), b"x\n", &asked).expect("the replacement is written");
    /// `MetadataExt` adds the group.
    use std::os::unix::fs::MetadataExt;
    let actual: u32 = std::fs::metadata(&path).expect("written").gid();
    assert_eq!(actual, own, "the replacement keeps the group it was asked for");
    remove(root.as_path());
}

/// What:
///  The extended attributes a file this directory creates carries,
///  as the list a fix reads from a file that has none of its own.
/// Why:
///  A replacement is created in that directory and must end with exactly the list it is given,
///  and the file system gives every file here an identity of its own;
///  a control that named an empty list would describe a file this directory cannot create.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // writeFile(prototype, 'x'); const carried = attributesOf(prototype); unlink(prototype); return carried;
/// ```
fn carried_by_a_new_file_in(directory: &Path) -> Vec<Attribute> {
    let prototype: PathBuf = directory.join("prototype.txt");
    std::fs::write(&prototype, b"x\n").expect("a prototype");
    let carried: Vec<Attribute> = attributes_of(prototype.as_path(), "reading a file").expect("read");
    std::fs::remove_file(&prototype).expect("the prototype is removed");
    return carried;
}

/// What:
///  A journal is reaped only when it is exactly the correction the record names,
///  bits and owner
///       included,
///  and a permission change of the person's keeps it.
/// Why:
///  A digest match does not say the journal holds the fix's own replacement:
///  the person can save the
///       correction's bytes themselves,
///  and a file they narrowed to `0600` moved into the journal by the swap
///       both matches on content and is theirs.
///  Reaping it would discard the mode they chose and leave the
///       broader `0644` replacement in the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // writeFile(a, 'a\n'); chmod(a, 0o600); exchange(contentOf(j), a); markOwnerDead(j); reap(); expect(modeOf(contentOf(j))).toBe(0o600)
/// ```
#[test]
fn a_reap_never_removes_a_permission_change_the_person_made() {
    let fixture_tree: Tree = tree("install-reap-mode");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let journal: PathBuf = prepared[0].journal.clone();
    let destination: PathBuf = fixture_tree.top.join("a.txt");
    // The person saves the correction's own bytes and narrows the file.
    std::fs::write(&destination, b"a\n").expect("a save");
    std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o600)).expect("narrow");
    // The swap installs the fix's 0644 replacement and moves the person's file into the journal.
    super::exchange(content_of(journal.as_path()).as_path(), &destination).expect("the swap");
    mark_writer_dead(journal.as_path());
    let mut notices: Vec<String> = Vec::new();
    reap_interrupted_installs(fixture_tree.root.as_path(), &mut notices);
    assert_eq!(journals(&fixture_tree.root).len(), 1, "the journal stays: {notices:?}");
    let kept_mode: u32 = std::fs::metadata(content_of(journal.as_path()))
        .expect("the kept bytes")
        .permissions()
        .mode()
        & 0o7777;
    assert_eq!(kept_mode, 0o600, "the person's mode is kept: {kept_mode:o}");
    assert!(
        notices.iter().any(|notice| return notice.contains("cli-git-direct-fix-")),
        "{notices:?}"
    );
    remove(fixture_tree.root.as_path());
}

/// What:
///  A journal whose `content` is not a regular file is neither read nor reaped,
///  and the sweep returns
///       at once.
/// Why:
///  The swap moves whatever stands at the destination into the journal,
///  and opening a FIFO with no
///       writer blocks for ever:
///  a recovery that read its bytes would hold every later fix,
/// against the owner's
///       "nothing should wait forever".
///  The check is the file's own type,
///  read without following a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // mkfifo(a); exchange(contentOf(j), a); markOwnerDead(j); race(reap(), sleep(10)) // the reap must win
/// ```
#[test]
fn a_journal_holding_a_fifo_never_waits_and_is_kept() {
    let fixture_tree: Tree = tree("install-reap-fifo");
    let prepared: Vec<Prepared> =
        super::prepare_replacements(&fixture_tree.top, &fixture_tree.root, one_change().as_slice())
            .expect("prepared");
    let journal: PathBuf = prepared[0].journal.clone();
    let destination: PathBuf = fixture_tree.top.join("a.txt");
    // Another command puts a FIFO where the corrected file is, and the swap moves it into the journal.
    std::fs::remove_file(&destination).expect("remove the file");
    let made: std::process::Output = std::process::Command::new("mkfifo")
        .arg(&destination)
        .output()
        .expect("mkfifo runs");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    super::exchange(content_of(journal.as_path()).as_path(), &destination).expect("the swap");
    let displaced: std::fs::FileType = std::fs::symlink_metadata(content_of(journal.as_path()))
        .expect("the displaced entry")
        .file_type();
    assert!(!displaced.is_file(), "the control needs a non-regular entry");
    mark_writer_dead(journal.as_path());
    // The sweep runs on another thread, so a sweep that blocks cannot hold this control for ever.
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<String>>();
    let git_dir: PathBuf = fixture_tree.root.clone();
    let _sweeper = std::thread::spawn(move || {
        let mut noticed: Vec<String> = Vec::new();
        reap_interrupted_installs(git_dir.as_path(), &mut noticed);
        // A receiver that is gone means the control already failed; nobody is left to tell.
        let _sent = sender.send(noticed);
    });
    let notices: Vec<String> = match receiver.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(found) => found,
        Err(_) => panic!("the sweep opened the FIFO and never returned"),
    };
    assert_eq!(journals(&fixture_tree.root).len(), 1, "the journal stays: {notices:?}");
    assert!(
        notices.iter().any(|notice| return notice.contains("cli-git-direct-fix-")),
        "{notices:?}"
    );
    remove(fixture_tree.root.as_path());
}

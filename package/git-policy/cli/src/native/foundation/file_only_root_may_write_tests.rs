//! What:
//!  Controls of the reader that only follows a file this program's account owns
//!  and only the account may write:
//!  a plain file passes,
//!  and a link,
//!  a group or other write bit,
//!  another owner and an absent file each answer their own refusal.
//! Why:
//!  The handler's configuration names the datasets root acts on,
//!  so each term of the reader is a way an account could choose them;
//!  every control here is reachable as an ordinary account,
//!  and the positive control proves the reader can answer,
//! not only refuse
//!  (`file_only_root_may_write`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readFileOnlyRootMayWrite(own)).toBe('{}'); expect(() => readFileOnlyRootMayWrite(link)).toThrow('link');
//! ```

use super::{
    FileControlRefusal,
    read_file_only_root_may_write,
    read_file_only_root_may_write_owned_by,
};
use crate::test_support::{
    fixture,
    remove,
};
use rustix::process::geteuid;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The account this control runs as,
/// which owns every fixture it writes.
fn this_account() -> u32 {
    return geteuid().as_raw();
}

/// One fixture file holding `text` at `mode`,
/// owned by this control's account.
fn file_at(
    name: &str,
    text: &str,
    mode: u32,
) -> PathBuf {
    let root: PathBuf = fixture(name);
    let path: PathBuf = root.join("git-worktree-clone-mounts.jsonc");
    std::fs::write(&path, text).expect("the fixture file is written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
        .expect("the fixture mode is set");
    return path;
}

/// A file this account owns and no other account may write is the one answer
/// that reads,
/// so the refusals below are refusals and not a reader that never reads.
#[test]
fn a_file_this_account_alone_may_write_is_read() {
    let path: PathBuf = file_at("root-write-plain", "{\"cloneMounts\":{}}", 0o644);
    assert_eq!(
        read_file_only_root_may_write(path.as_path()),
        Ok(Some(String::from("{\"cloneMounts\":{}}")))
    );
    let private: PathBuf = file_at("root-write-owner-only", "", 0o600);
    assert_eq!(
        read_file_only_root_may_write(private.as_path()),
        Ok(Some(String::new())),
        "an empty file is text, and the caller refuses it as empty"
    );
    remove(path.parent().unwrap_or(path.as_path()));
    remove(private.parent().unwrap_or(private.as_path()));
}

/// No file at the path is no refusal:
/// the caller's own answer for a missing file is what the project answered in
/// 2026-11-22.
#[test]
fn an_absent_file_is_no_text_and_no_refusal() {
    let root: PathBuf = fixture("root-write-absent");
    assert_eq!(
        read_file_only_root_may_write(root.join("nothing-here.jsonc").as_path()),
        Ok(None)
    );
    remove(root.as_path());
}

/// A link at the path is refused,
/// and the target it names is readable,
/// so the refusal is the reader's rule and not the target's state.
#[test]
fn a_link_at_the_path_is_refused_and_its_target_is_not_read() {
    let target: PathBuf = file_at("root-write-link", "{\"cloneMounts\":{}}", 0o644);
    let root: PathBuf = target.parent().expect("the fixture folder").to_path_buf();
    let link: PathBuf = root.join("git-worktree-clone-mounts-link.jsonc");
    std::os::unix::fs::symlink(&target, &link).expect("the fixture link is made");
    assert_eq!(
        read_file_only_root_may_write(target.as_path()),
        Ok(Some(String::from("{\"cloneMounts\":{}}"))),
        "the target reads, so the link's refusal is the rule"
    );
    assert_eq!(
        read_file_only_root_may_write(link.as_path()),
        Err(FileControlRefusal::Link {
            path: link.clone()
        })
    );
    assert!(
        read_file_only_root_may_write(link.as_path())
            .expect_err("a link is refused")
            .reason()
            .contains("link"),
        "the refusal names the link"
    );
    remove(root.as_path());
}

/// A group or other write bit is refused,
/// and the refusal names the bits it found,
/// so a person knows which bit to clear.
#[test]
fn a_write_bit_for_another_account_is_refused() {
    for mode in [0o664, 0o646, 0o666, 0o647] {
        let path: PathBuf = file_at("root-write-bits", "{}", mode);
        let refusal: FileControlRefusal = read_file_only_root_may_write(path.as_path())
            .expect_err("another account could write it");
        assert_eq!(
            refusal,
            FileControlRefusal::WritableByOthers {
                path: path.clone(),
                mode
            }
        );
        assert!(
            refusal.reason().contains(&format!("{mode:04o}")),
            "the refusal names the mode it found: {}",
            refusal.reason()
        );
        remove(path.parent().unwrap_or(path.as_path()));
    }
}

/// A file another account owns is refused,
/// which is the term the release path reads from the kernel:
/// the handler runs as root,
/// so a file an account owns is one that account could rewrite.
#[test]
fn a_file_another_account_owns_is_refused() {
    let path: PathBuf = file_at("root-write-other-owner", "{}", 0o644);
    let another: u32 = this_account().wrapping_add(1);
    let refusal: FileControlRefusal =
        read_file_only_root_may_write_owned_by(path.as_path(), another)
            .expect_err("another account owns it");
    assert_eq!(
        refusal,
        FileControlRefusal::Owner {
            path: path.clone(),
            found: this_account(),
            wanted: another
        }
    );
    let reason: String = refusal.reason();
    assert!(
        reason.contains(&format!("user {}", this_account())),
        "{reason}"
    );
    assert!(
        reason.contains(&format!("runs as user {another}")),
        "{reason}"
    );
    assert_eq!(
        read_file_only_root_may_write_owned_by(path.as_path(), this_account()),
        Ok(Some(String::from("{}"))),
        "the same file answers its own owner"
    );
    remove(path.parent().unwrap_or(path.as_path()));
}

/// A folder at the path is refused as unreadable text,
/// which is what the readers of this file answered before the reader asked
/// whose it was.
#[test]
fn a_folder_at_the_path_is_refused() {
    let root: PathBuf = fixture("root-write-folder");
    let refusal: FileControlRefusal =
        read_file_only_root_may_write(root.as_path()).expect_err("a folder is not a file");
    assert!(
        matches!(refusal, FileControlRefusal::Open { .. }),
        "{refusal:?}"
    );
    remove(root.as_path());
}

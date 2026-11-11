//! What:
//!  Controls for putting one planned worktree write in place:
//!  the entry the write judged,
//!  the private name the entry is taken to,
//!  and the write that can still arrive through a descriptor opened before the take.
//! Why:
//!  A judgement and a write are separate steps,
//!  so the bytes a person's program writes into the entry the take renamed are the thing every
//!  control here asks about:
//!  the path either holds them or names where they are,
//!  and the merge never reports a write that threw them away.

use super::{JudgedFile, place_file_awaiting};
use crate::merge_isolation_carry_writes::FileWrite;
use crate::merge_isolation_changes::file_identity;
use crate::merge_isolation_take_asides::ASIDE_PREFIX;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::io::Write;
use std::path::{Path, PathBuf};

/// The bytes the branch holds at the path the merge carries.
const BEFORE: &str = "the line the branch had\n";

/// The bytes the merge carries to the path.
const MERGED: &str = "the line the merge brings\n";

/// The bytes a program that kept its descriptor writes into the entry the take renamed.
const THE_EDIT: &str = "an edit written through a descriptor that was opened before the take\n";

/// The file's mode.
const MODE: &str = "100644";

/// The private names a take of `location` left in its directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asidesBeside = (location) => readdirSync(dirname(location)).filter((name) => name.startsWith(ASIDE_PREFIX));
/// ```
fn asides_beside(location: &Path) -> Vec<PathBuf> {
    let Some(directory) = location.parent() else {
        return Vec::new();
    };
    let Ok(listing) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(ASIDE_PREFIX) {
            found.push(entry.path());
        }
    }
    found.sort();
    return found;
}

/// What:
///  A file a process holds open is never dropped with the private name of a carry's take,
///  and the bytes a write through that descriptor puts there are still readable afterwards.
/// Why:
///  `take_judged_awaiting` renames the worktree file to a private name and checks its bytes there,
///  and a process that opened the file before that rename keeps a descriptor to the same inode.
///  Removing the private name after the merge's own entry is placed leaves those newly written
///  bytes on an inode with no name at all,
/// which is how a merge succeeds and loses a save
///  (a Kimi K3.1 Pro audit of every wait since `0.6.1-alpha.8`,
/// its first finding).
///  The proof the second pass already uses is the same one:
/// the identity the take checked is
///  unchanged,
/// no process this command can read holds the file open,
/// and its bytes are still the
///  ones the merge's capture holds (`merge_isolation_take_asides.rs`,
/// `may_remove_awaiting`).
///  The holder is a child of this process,
///  because the scan for open descriptors leaves this process out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holderOf(location); const placed = await placeFile({ judged, bytes: MERGED }); expect(asidesBesides(location)).toHaveLength(1); write(held, THE_EDIT); expect(read(asidesBesides(location)[0])).toEqual(THE_EDIT);
/// ```
#[test]
fn a_file_a_process_holds_open_is_kept_when_a_carry_replaces_it() {
    let root: PathBuf = fixture("carry-place-held-open");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("f.txt"), BEFORE).expect("the branch's file");
    git(repo.as_path(), &["add", "--", "f.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let location: PathBuf = repo.join("f.txt");
    let object: String = git_text(repo.as_path(), &["hash-object", "--", "f.txt"]).trim().to_owned();
    let judged: JudgedFile = JudgedFile { identity: file_identity(location.as_path()), content: Some(object) };
    // The descriptor the writer keeps is this process's, so the write it makes can be asked about after the placement; the
    //  child holds the same inode open, which is what the scan for holders can see.
    let mut writer: std::fs::File = std::fs::File::options().write(true).open(location.as_path()).expect("the writer's descriptor");
    let held_open: std::fs::File = std::fs::File::open(location.as_path()).expect("the descriptor to hand on");
    let mut holder: std::process::Command = std::process::Command::new("sleep");
    holder.arg("120").stdin(std::process::Stdio::from(held_open));
    let mut holding: std::process::Child = holder.spawn().expect("a program holding the file open");
    drop(holder);
    let placed = crate::lazy_runtime::run_on_runtime(async {
        return place_file_awaiting(&git_context(), repo.as_path(), b"f.txt", &judged, &FileWrite::Bytes(MERGED.as_bytes().to_vec(), String::from(MODE))).await;
    })
    .expect("the runtime runs the placement")
    .expect("the placement goes through");
    let kept: Vec<PathBuf> = asides_beside(location.as_path());
    // The descriptor the writer kept writes now, into the inode the take renamed.
    writer.write_all(THE_EDIT.as_bytes()).expect("the write through the descriptor the writer kept");
    writer.flush().expect("the write reaches the file");
    let said: String = format!("{placed:?}");
    assert_eq!(kept.len(), 1, "the entry a process holds open is kept under its private name, not removed: {kept:?} ({said})");
    assert_eq!(std::fs::read(kept[0].as_path()).expect("the kept entry"), THE_EDIT.as_bytes(), "the bytes written through the descriptor are still readable");
    assert_eq!(std::fs::read(location.as_path()).expect("the merge's entry"), MERGED.as_bytes(), "the merge's entry is at the path");
    holding.kill().expect("the holder ends");
    holding.wait().expect("reaped");
    remove(root.as_path());
}

/// What:
///  A file a carry deletes is removed from the path,
///  and the entry it held is kept and nameable when a process can still write into it.
/// Why:
///  The removal path removes the private name of the entry it took aside,
///  and that entry is the only name a write through a descriptor opened before the take can reach
///  (a Kimi K3.1 Pro audit of every wait since `0.6.1-alpha.8`,
/// its first finding,
/// which named the removal at line 353 as well as the placing).
///  What the path loses is the merge's decision,
///  never the bytes a program wrote into the entry beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holderOf(location); const placed = await placeFile({ judged, write: { kind: 'remove' } }); expect(exists(location)).toBe(false); expect(asidesBesides(location)).toHaveLength(1); write(held, AN_EDIT); expect(read(asidesBesides(location)[0])).toEqual(AN_EDIT);
/// ```
#[test]
fn a_file_a_process_holds_open_is_kept_when_a_carry_deletes_it() {
    let root: PathBuf = fixture("carry-place-held-open-delete");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("f.txt"), BEFORE).expect("the branch's file");
    git(repo.as_path(), &["add", "--", "f.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let location: PathBuf = repo.join("f.txt");
    let object: String = git_text(repo.as_path(), &["hash-object", "--", "f.txt"]).trim().to_owned();
    let judged: JudgedFile = JudgedFile { identity: file_identity(location.as_path()), content: Some(object) };
    let mut writer: std::fs::File = std::fs::File::options().write(true).open(location.as_path()).expect("the writer's descriptor");
    let held_open: std::fs::File = std::fs::File::open(location.as_path()).expect("the descriptor to hand on");
    let mut holder: std::process::Command = std::process::Command::new("sleep");
    holder.arg("120").stdin(std::process::Stdio::from(held_open));
    let mut holding: std::process::Child = holder.spawn().expect("a program holding the file open");
    drop(holder);
    let placed = crate::lazy_runtime::run_on_runtime(async {
        return place_file_awaiting(&git_context(), repo.as_path(), b"f.txt", &judged, &FileWrite::Remove).await;
    })
    .expect("the runtime runs the removal")
    .expect("the removal goes through");
    let said: String = format!("{placed:?}");
    assert!(std::fs::symlink_metadata(location.as_path()).is_err(), "the path the merge deletes is free");
    let kept: Vec<PathBuf> = asides_beside(location.as_path());
    writer.write_all(THE_EDIT.as_bytes()).expect("the write through the descriptor the writer kept");
    writer.flush().expect("the write reaches the file");
    assert_eq!(kept.len(), 1, "the entry the removal took aside is kept when a program holds it open: {kept:?} ({said})");
    assert_eq!(std::fs::read(kept[0].as_path()).expect("the kept entry"), THE_EDIT.as_bytes(), "the bytes written through the descriptor are still readable");
    holding.kill().expect("the holder ends");
    holding.wait().expect("reaped");
    remove(root.as_path());
}

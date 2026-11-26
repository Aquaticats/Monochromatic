//! What:
//!  The one decision every command makes about a copy a merge kept:
//!  a copy an earlier release left of the merge's own file is put back,
//!  one left of newer bytes stays and is recorded,
//!  one the person's file holds again is removed,
//!  and a copy decided once more ends when the file holds its bytes.
//! Why:
//!  A copy nobody decides on is a file the person never asked for,
//!  and a copy deleted too early loses the bytes it was kept for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(exists(copy)).toBe(false); expect(read(path)).toEqual(theBytesOfACommit(world, oid, path));
//! ```

use super::super::*;
use super::fixtures::{
    World,
    an_earlier_releases_name,
    dead_owner,
    place,
    record_of,
    world,
};
use super::readings::{
    judged_changes,
    the_bytes_of_a_commit,
};
/// The put-back of the files a killed take left beside their paths.
use crate::merge_isolation_recovery_steps::put_back_stranded_takes_awaiting;
use crate::test_support::remove;
use std::path::PathBuf;

/// What:
///  A copy an earlier release left of the file the merge holds at a free path is put back,
///  and the merge goes on from the checkout it expected.
/// Why:
///  The name an earlier release wrote carries no merge key,
/// so no record could say whether an unfinished merge took
///  it;
///  the bytes can,
/// and a copy of the branch's own file or the merge's own file is
///  the file the merge judged there.
/// Without this,
/// a merge interrupted before this release was never finished:
///  the path stayed empty,
/// the copy stayed beside it,
/// and every later merge read the emptiness as the
///  person's own deletion (`merge-carry-review-round-2-findings-1`,
/// its second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record(deadOwner(...)); rename('f.txt', earlierName('f.txt')); await putBackStrandedTakes(...); expect(exists('f.txt')).toBe(true); expect(sink).toContain('put back');
/// ```
#[test]
fn a_copy_an_earlier_release_left_of_the_merges_own_file_is_put_back() {
    let world: World = world("merge-recovery-earlier-release-copy");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let _record: PathBuf = record_of(
        &world,
        dead_owner("23232323-0000-4000-8000-000000000023"),
        Some("refs/heads/main"),
    );
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    // The copy holds the branch's own file, which is what a take of an interrupted install moved aside.
    std::fs::write(
        aside.as_path(),
        the_bytes_of_a_commit(&world, world.old_oid.as_str(), "f.txt"),
    )
    .expect("the copy");
    std::fs::remove_file(location.as_path()).expect("the path a killed take emptied");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        "2a2a2a2a-0000-4000-8000-00000000002a",
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        location.exists(),
        "the path holds the file the merge judged again"
    );
    assert_eq!(
        std::fs::read(location.as_path()).expect("the file is back"),
        the_bytes_of_a_commit(&world, world.old_oid.as_str(), "f.txt"),
        "its bytes are the ones it had"
    );
    assert!(!aside.exists(), "the copy is not a copy any more");
    assert!(
        text.contains("put back"),
        "the putting back is said: {text}"
    );
    remove(world.parent.as_path());
}

/// What:
///  A copy an earlier release left of bytes nothing else holds is never put back on a guess,
///  and it is recorded so a later command ends it.
/// Why:
///  The bytes are the person's newest,
/// and nothing else holds them:
///  not a commit,
/// and not the capture the merge's record names.
/// Putting them back would
///  overwrite whatever the person put at the path afterwards,
/// so the copy stays,
/// the line names
///  it,
/// and the record makes the next command look at it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename('f.txt', earlierName('f.txt')); write(aside, 'the person\'s own newer bytes\n'); await putBackStrandedTakes(...); expect(exists(location)).toBe(false); expect(exists(aside)).toBe(true); expect(records(gitDir)).toHaveLength(1);
/// ```
#[test]
fn a_copy_an_earlier_release_left_of_newer_bytes_stays_and_is_recorded() {
    let world: World = world("merge-recovery-earlier-release-newer");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let _record: PathBuf = record_of(
        &world,
        dead_owner("24242424-0000-4000-8000-000000000024"),
        Some("refs/heads/main"),
    );
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(aside.as_path(), b"the person's own newer bytes\n").expect("the copy");
    std::fs::remove_file(location.as_path()).expect("the path a killed take emptied");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        "2a2a2a2a-0000-4000-8000-00000000002a",
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        !location.exists(),
        "the path the person emptied stays empty"
    );
    assert!(aside.exists(), "the bytes nothing else holds are kept");
    assert_eq!(
        std::fs::read(aside.as_path()).expect("the copy"),
        b"the person's own newer bytes\n",
        "and they are whole"
    );
    assert!(
        text.contains(aside.to_string_lossy().as_ref()),
        "the copy is named: {text}"
    );
    assert!(
        text.contains("mv --"),
        "the line says how to put it back: {text}"
    );
    assert!(
        crate::merge_isolation_kept_copies::record_of_a_control(
            world.git_dir.as_path(),
            aside.as_path()
        )
        .exists(),
        "the copy is recorded, so a later command decides it again"
    );
    remove(world.parent.as_path());
}

/// What:
///  A copy whose bytes the person's file holds again is removed,
///  and the line says the file holds them.
/// Why:
///  Two files holding one run of bytes are one too many,
/// and the copy beside the path is the one to go:
///  the person's file is where they look,
/// and a copy that stayed would be a name nothing explains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', 'mine\n'); rename('f.txt', aside); write('f.txt', 'mine\n'); await putBackStrandedTakes(...); expect(exists(aside)).toBe(false); expect(sink).toContain('holds those same bytes again');
/// ```
#[test]
fn a_copy_the_persons_file_holds_again_is_removed() {
    let world: World = world("merge-recovery-copy-the-file-holds");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(aside.as_path(), b"mine\n").expect("the copy");
    std::fs::write(location.as_path(), b"mine\n").expect("the person's file holds the same bytes");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        "2a2a2a2a-0000-4000-8000-00000000002a",
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(!aside.exists(), "the second copy goes");
    assert_eq!(
        std::fs::read(location.as_path()).expect("the person's file"),
        b"mine\n",
        "their file is untouched"
    );
    assert!(
        text.contains("holds those same bytes again"),
        "the line says why the copy went: {text}"
    );
    remove(world.parent.as_path());
}

/// What:
///  A copy a merge kept is decided by a command that has no merge record at all,
///  and it ends the moment the person's file holds its bytes.
/// Why:
///  A copy is left beside a path one merge touched,
/// and a later merge that touches another path says nothing
///  about it;
/// without this the copy is a file no command ever ends
///  (`merge-carry-review-round-2-findings-1`,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await putBackStrandedTakes(...); expect(records).toHaveLength(1); const first = await recover(noRecord); expect(sink).toContain(aside); write('f.txt', copyBytes); const second = await recover(noRecord); expect(exists(aside)).toBe(false); expect(sink).toContain('removed');
/// ```
#[test]
fn a_kept_copy_is_decided_once_more_and_ends_when_the_file_holds_its_bytes() {
    let world: World = world("merge-recovery-kept-copy-end");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    let kept: &[u8] = b"the person's own newer bytes\n";
    std::fs::write(aside.as_path(), kept).expect("the copy");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        "2a2a2a2a-0000-4000-8000-00000000002a",
        &mut said,
    ))
    .expect("the runtime of the controls");
    let record: PathBuf = crate::merge_isolation_kept_copies::record_of_a_control(
        world.git_dir.as_path(),
        aside.as_path(),
    );
    assert!(record.exists(), "the copy is recorded");
    // A later command with no merge record in this worktree: the file the merge held is gone, so nothing judges the copy's
    //  bytes again, and the copy still ends.
    let mut later: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut later)
        .expect("recovery");
    assert!(outcomes.is_empty(), "no merge record is left to settle");
    let text: String = String::from_utf8(later).expect("the notices are UTF-8");
    assert!(
        text.contains(aside.to_string_lossy().as_ref()),
        "the copy is named again: {text}"
    );
    assert_eq!(
        std::fs::read(location.as_path()).expect("the world's file"),
        b"f.txt base\n",
        "nothing was put back on a guess"
    );
    // The person copies their bytes into the file themselves, which is the end the line named.
    std::fs::write(location.as_path(), kept).expect("the person's file holds the bytes again");
    let mut last: Vec<u8> = Vec::new();
    let _outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut last)
        .expect("recovery");
    let ending: String = String::from_utf8(last).expect("the notices are UTF-8");
    assert!(
        !aside.exists(),
        "the copy goes once the file holds its bytes"
    );
    assert!(!record.exists(), "and its record goes with it");
    assert!(
        ending.contains("removed"),
        "the line says the copy went: {ending}"
    );
    assert_eq!(
        std::fs::read(location.as_path()).expect("the person's file"),
        kept,
        "their file is untouched"
    );
    remove(world.parent.as_path());
}

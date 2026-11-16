//! Controls of two costs of the migration's own bookkeeping:
//! a name that is linked to an earlier copy holds the bytes the pass read,
//! and replacing many entries does not cost a durability sync for each of them.
//! The first was written against `9ef0169599`, where the second name of a file was linked to a copy made before a write the
//! pass had already read, so both names stood on bytes older than the source;
//! the second against the same tip, where every replaced entry cost four `fsync` calls.

use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_file_write_steps::{SyncCounts, take_sync_counts};
use crate::subvolume_preflight::{Layout, layout_of};
use crate::subvolume_sync::{SyncOptions, SyncStats, sync_tree, watch_entries};
use crate::subvolume_platform::Platform;
use crate::subvolume_test_support::{Flushing, PlainPlatform, checkout_with_worktree, migrate_arguments, run_command};
use crate::test_support::remove;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The exit status a finished command gives.
const DONE: i32 = 0;

/// The exit status of a failure after the journal began.
const FAILED: i32 = 2;

/// Whether the file at `path` holds exactly `bytes`.
fn holds(path: &Path, bytes: &[u8]) -> bool {
    return std::fs::read(path).map(|found| return found == bytes).unwrap_or(false);
}

/// The arguments of the deletion of the old tree.
fn remove_old_arguments() -> MigrateArguments {
    return MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() };
}

/// Run one command to completion and answer its status and everything it said.
fn run(platform: &dyn Platform, arguments: &MigrateArguments, root: &Path) -> (i32, String) {
    let (code, lines) = run_command(platform, arguments, root, &Interrupt::never());
    return (code, lines.join("\n"));
}

/// The review, finding 4:
/// a name linked to an earlier copy holds the bytes the pass read,
/// because a copy is only linked when the source still reads as it did when that copy was made.
#[test]
fn linked_the_second_name_of_a_file_holds_the_bytes_the_pass_read() {
    let parent: PathBuf = crate::test_support::fixture("linked-second-name");
    let source: PathBuf = parent.join("source");
    let destination: PathBuf = parent.join("destination");
    std::fs::create_dir(source.as_path()).expect("source");
    std::fs::create_dir(destination.as_path()).expect("destination");
    std::fs::write(source.join("a.txt"), b"one\n").expect("a.txt");
    std::fs::hard_link(source.join("a.txt"), source.join("b.txt")).expect("b.txt, the second name");
    // The write lands after the pass examined the first name, so that name's copy is older than the source by the time
    //  the second name is reached. Whichever name is examined second carries the write.
    let newest: &[u8] = b"TWO, WRITTEN BETWEEN THE TWO NAMES\n";
    let seen: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    let source_for_the_watch: PathBuf = source.clone();
    let written_at: &'static [u8] = newest;
    watch_entries(Some(Box::new(move |examined: &Path| {
        let is_one_of_the_two: bool = examined.parent() == Some(source_for_the_watch.as_path());
        if !is_one_of_the_two || seen.fetch_add(1, Ordering::SeqCst) == 0 {
            return;
        }
        std::fs::write(source_for_the_watch.join("b.txt"), written_at).expect("a write between the two names");
    })));
    let platform: PlainPlatform = PlainPlatform::new();
    let stats: SyncStats = sync_tree(
        &platform,
        source.as_path(),
        destination.as_path(),
        SyncOptions { never_replaces: false, delete: false, update_only: false, excluded: &[], created_since: None },
        &Interrupt::never(),
        None,
        None,
    )
    .expect("the pass ran");
    watch_entries(None);
    println!("pass: {} entries, {} copied, {} hard-linked", stats.entries, stats.files_copied, stats.hard_linked);
    let first: bool = holds(destination.join("a.txt").as_path(), newest);
    let second: bool = holds(destination.join("b.txt").as_path(), newest);
    remove(parent.as_path());
    assert!(
        first || second,
        "one of the names holds the bytes the pass read after the write, so no name is left standing on older bytes alone"
    );
}

/// What:
///  The syncs one deletion of the old tree costs,
///  whose late pass replaces every entry the person wrote into the old tree.
/// Why:
///   The cost a person waits on is measured by difference rather than by a total,
///  so nothing here depends on how many syncs the rest of the run takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const syncs = syncsOfDeleting(count);
/// ```
fn syncs_of_deleting(written: usize) -> SyncCounts {
    let (parent, root) = checkout_with_worktree(format!("parking-syncs-{written}").as_str());
    // The stand-in keeps its writes in memory, and a run of it would cost no sync however much it parks; this one flushes
    //  as the real platform does, so the count is the cost a person waits on.
    let platform: Flushing = Flushing { inner: PlainPlatform::new(), on_question: Box::new(|| {}) };
    let (migrated, migrated_said) = run(&platform, &migrate_arguments(), root.as_path());
    assert_eq!(migrated, DONE, "{migrated_said}");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    for name in ["README.md", "script.sh", "empty", "src/main.rs", "src/deep/er/leaf.txt", "src/deep/er/leaf.txt", "link", "src/deep/er/leaf.txt"].iter().take(written) {
        std::fs::write(layout.old_path.join(name), format!("a write into the old tree at {name}\n").as_bytes()).expect("a write into the old tree");
    }
    let _cleared: SyncCounts = take_sync_counts();
    let (code, said) = run(&platform, &remove_old_arguments(), root.as_path());
    let counts: SyncCounts = take_sync_counts();
    println!("{written} write(s) into the old tree: removal code {code}, {} sync(s) made", counts.made);
    assert_ne!(code, FAILED, "{said}");
    remove(parent.as_path());
    return counts;
}

/// The review, finding 3:
/// deleting the old tree after a program wrote into it costs no more durability syncs for eight entries than for one.
#[test]
fn linked_replacing_entries_does_not_cost_a_sync_for_each_one() {
    let one: SyncCounts = syncs_of_deleting(1);
    let eight: SyncCounts = syncs_of_deleting(8);
    assert!(
        eight.made <= one.made,
        "the record of what a pass parks is written once for the pass, not once for each entry: 1 entry cost {} sync(s), 8 entries cost {}",
        one.made,
        eight.made
    );
}

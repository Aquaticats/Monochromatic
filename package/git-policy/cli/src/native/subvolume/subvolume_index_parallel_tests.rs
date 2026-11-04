//! The rewrite of every index of a checkout side by side: the rewrites overlap, the report keeps the order of the checkout's
//!  index first and then each linked worktree's, one index that cannot be rewritten leaves the others rewritten, and the host
//!  budget bounds the overlap.

use super::*;
use crate::parallel_test_support::{Rendezvous, SERIAL_PATIENCE};
use crate::subvolume_test_support::make_checkout;
use crate::test_support::{fixture, git, remove};
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many linked worktrees the checkout of these controls has; the checkout's own index makes one more.
const LINKED: usize = 3;

/// A host large enough that neither the CPUs nor the memory bound a run over `LINKED + 1` indexes.
const LARGE_HOST: HostResources = HostResources { parallelism: (LINKED + 1) * 8, memory_available: None };

/// A checkout with `LINKED` linked worktrees `w0`, `w1`, ... inside it, each with an index.
fn checkout_with_linked(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    for index in 0..LINKED {
        git(root.as_path(), &["worktree", "add", "--quiet", "--detach", format!("w{index}").as_str()]);
    }
    return (parent, root);
}

/// Rewrite every index of `root` with `rewrite` on `host` and return the count and the lines.
fn refresh_with(root: &Path, host: &HostResources, rewrite: &RewriteOne<'_>) -> (usize, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let count: usize = refresh_all_indexes_with(&AllIndexes { root, lock: IndexLock::Acquire, references: &[], host, rewrite }, &mut |text: &str| { lines.push(text.to_string()); });
    return (count, lines);
}

/// The rewrites of all the indexes start before any has finished: a point that needs all of them at once is passed only by
///  parallel work.
#[test]
fn the_indexes_are_rewritten_side_by_side() {
    let (parent, root) = checkout_with_linked("index-parallel-overlap");
    let point: Rendezvous = Rendezvous::new(LINKED + 1);
    let met: AtomicUsize = AtomicUsize::new(0);
    let (count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &|index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        if point.arrive() {
            met.fetch_add(1, Ordering::SeqCst);
        }
        return refresh_index_file_with(index, tree, options);
    });
    assert_eq!(count, LINKED + 1);
    assert_eq!(met.load(Ordering::SeqCst), LINKED + 1, "{lines:#?}");
    remove(parent.as_path());
}

/// The checkout's index is reported first and then each worktree's by path, though the checkout's index finishes last.
#[test]
fn the_report_keeps_the_order_of_the_checkout_then_its_worktrees() {
    let (parent, root) = checkout_with_linked("index-parallel-order");
    let others_done: Rendezvous = Rendezvous::impatient(LINKED, crate::parallel_test_support::PATIENCE);
    let main_index: PathBuf = root.join(".git").join("index");
    let (_count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &|index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        if index == main_index.as_path() {
            assert!(others_done.wait_for_target(), "the worktrees' indexes were rewritten while the checkout's waited");
        }
        let line: String = refresh_index_file_with(index, tree, options);
        if index != main_index.as_path() {
            others_done.arrive_without_waiting();
        }
        return line;
    });
    assert_eq!(lines.len(), LINKED + 1, "{lines:#?}");
    assert!(lines[0].starts_with(format!("index {}:", main_index.display()).as_str()), "{lines:#?}");
    for index in 0..LINKED {
        assert!(lines[index + 1].starts_with(format!("index {}:", root.join(format!(".git/worktrees/w{index}/index")).display()).as_str()), "{lines:#?}");
    }
    remove(parent.as_path());
}

/// An index whose lock a Git command holds is skipped and said so; every other index is rewritten.
#[test]
fn an_index_that_cannot_be_rewritten_leaves_the_others_rewritten() {
    let (parent, root) = checkout_with_linked("index-parallel-skip");
    std::fs::write(root.join(".git/worktrees/w0/index.lock"), b"another git").expect("lock");
    let (count, lines) = refresh_with(root.as_path(), &LARGE_HOST, &refresh_index_file_with);
    assert_eq!(count, LINKED + 1);
    assert_eq!(lines.iter().filter(|line: &&String| { return line.contains("skipped (index.lock exists"); }).count(), 1, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.ends_with("entries refreshed"); }).count(), LINKED, "{lines:#?}");
    remove(parent.as_path());
}

/// The host's budget bounds the overlap: on a host that may run few threads the overlap reaches that many and no more.
#[test]
fn the_budget_of_the_host_bounds_the_overlap() {
    let (parent, root) = checkout_with_linked("index-parallel-budget");
    let host: HostResources = HostResources { parallelism: 3, memory_available: None };
    let allowed: usize = worker_budget(&BudgetRequest { items: LINKED + 1, per_item_memory: 1, ceiling: None, cap: None, host: &host });
    assert!(allowed > 1 && allowed < LINKED + 1, "the control needs a bound between one and the indexes: {allowed}");
    let running: AtomicUsize = AtomicUsize::new(0);
    let busiest: AtomicUsize = AtomicUsize::new(0);
    let point: Rendezvous = Rendezvous::impatient(allowed, SERIAL_PATIENCE * 10);
    let (_count, lines) = refresh_with(root.as_path(), &host, &|index: &Path, tree: &Path, options: &RefreshOptions<'_>| {
        let now: usize = running.fetch_add(1, Ordering::SeqCst) + 1;
        busiest.fetch_max(now, Ordering::SeqCst);
        point.arrive();
        let line: String = refresh_index_file_with(index, tree, options);
        running.fetch_sub(1, Ordering::SeqCst);
        return line;
    });
    assert_eq!(busiest.load(Ordering::SeqCst), allowed, "{lines:#?}");
    remove(parent.as_path());
}

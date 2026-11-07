//! Controls for the per-case copies and the worker runner.

use super::*;
use crate::command_test_support::{fixture, remove};

/// A copy holds every nested file with the same bytes and shares nothing with the source.
#[test]
fn a_copy_holds_every_nested_file_and_shares_nothing() {
    let directory = fixture("cases-copy");
    let source = directory.join("source");
    std::fs::create_dir_all(source.join("inner/deeper")).expect("source directories");
    std::fs::write(source.join("top.txt"), b"top").expect("top file");
    std::fs::write(source.join("inner/deeper/leaf.txt"), b"leaf").expect("leaf file");
    let copy = directory.join("copy");
    copy_tree(source.as_path(), copy.as_path());
    assert_eq!(std::fs::read(copy.join("top.txt")).expect("copied top"), b"top");
    assert_eq!(std::fs::read(copy.join("inner/deeper/leaf.txt")).expect("copied leaf"), b"leaf");
    std::fs::write(copy.join("top.txt"), b"changed").expect("change the copy");
    assert_eq!(std::fs::read(source.join("top.txt")).expect("source top"), b"top");
    remove(directory.as_path());
}

/// A link in the fixture is refused instead of followed or copied as a regular file.
#[cfg(unix)]
#[test]
fn a_link_in_the_source_is_refused() {
    let directory = fixture("cases-link");
    let source = directory.join("source");
    std::fs::create_dir_all(&source).expect("source directory");
    std::os::unix::fs::symlink("elsewhere", source.join("link")).expect("link");
    let outcome = std::panic::catch_unwind(|| copy_tree(source.as_path(), directory.join("copy").as_path()));
    assert!(outcome.is_err());
    remove(directory.as_path());
}

/// Every item is handled exactly once,
/// whatever the number of items against the number of workers.
#[test]
fn every_item_is_handled_exactly_once() {
    for count in [0, 1, WORKER_CAP, WORKER_CAP * 3 + 1] {
        let items: Vec<usize> = (0..count).collect();
        let results: Vec<Vec<usize>> = across_workers(
            &items,
            |_| return Vec::new(),
            |item, seen| {
                seen.push(*item);
            },
        );
        let mut handled: Vec<usize> = results.into_iter().flatten().collect();
        handled.sort_unstable();
        assert_eq!(handled, items, "{count} items");
    }
}

/// Each worker starts from the result its own number creates.
#[test]
fn each_worker_gets_its_own_result() {
    let items: Vec<usize> = (0..WORKER_CAP * 2).collect();
    let results: Vec<usize> = across_workers(&items, |number| { return number; }, |_, _| {});
    let mut numbers: Vec<usize> = results.clone();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(numbers.len(), results.len());
}

/// A panic in a worker reaches the caller with its own message.
#[test]
fn a_panic_in_a_worker_reaches_the_caller() {
    let items: Vec<usize> = vec![1];
    let outcome = std::panic::catch_unwind(|| {
        return across_workers(&items, |_| (), |_, _| panic!("the case failed"));
    });
    let payload = outcome.expect_err("the panic resumes on the caller");
    assert_eq!(payload.downcast_ref::<&str>().copied(), Some("the case failed"));
}

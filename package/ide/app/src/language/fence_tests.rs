//! Stale-result fencing without a server:
//!  each identity component is moved on its own.

use super::{Fence, FenceCounts, FenceVerdict};
use crate::language::identity::{DocumentStamp, ServerIdentity};

fn server(name: &str, instance: u64) -> ServerIdentity {
    return ServerIdentity {
        name: name.to_string(),
        instance,
    };
}

fn fence() -> Fence {
    let mut fence = Fence::default();
    fence.display(Some(DocumentStamp {
        file: 4,
        revision: 7,
    }));
    fence.set_servers(vec![server("rust-analyzer", 2), server("typos", 1)]);
    return fence;
}

#[test]
fn current_result_is_accepted() {
    let mut fence = fence();
    let stamp = DocumentStamp {
        file: 4,
        revision: 7,
    };
    assert_eq!(
        fence.verdict(stamp, Some(&server("rust-analyzer", 2))),
        FenceVerdict::Accepted
    );
    assert!(
        fence.admit(stamp, Some(&server("typos", 1))),
        "a current result was dropped"
    );
    assert!(
        fence.admit(stamp, None),
        "a current result without a server was dropped"
    );
}

#[test]
fn result_for_an_old_revision_is_dropped() {
    let mut fence = fence();
    let stale = DocumentStamp {
        file: 4,
        revision: 6,
    };
    assert_eq!(
        fence.verdict(stale, Some(&server("rust-analyzer", 2))),
        FenceVerdict::StaleRevision,
        "a result for an old revision was accepted"
    );
    assert!(
        !fence.admit(stale, Some(&server("rust-analyzer", 2))),
        "a result for an old revision was accepted"
    );
}

#[test]
fn result_for_an_old_file_generation_is_dropped() {
    let mut fence = fence();
    let other = DocumentStamp {
        file: 3,
        revision: 7,
    };
    assert_eq!(
        fence.verdict(other, Some(&server("rust-analyzer", 2))),
        FenceVerdict::OtherFile,
        "a result for an old file generation was accepted"
    );
    assert!(
        !fence.admit(other, Some(&server("rust-analyzer", 2))),
        "a result for an old file generation was accepted"
    );
}

#[test]
fn result_from_a_replaced_server_process_is_dropped() {
    let mut fence = fence();
    let stamp = DocumentStamp {
        file: 4,
        revision: 7,
    };
    assert_eq!(
        fence.verdict(stamp, Some(&server("rust-analyzer", 1))),
        FenceVerdict::OldServer,
        "a result from a replaced server process was accepted"
    );
    assert!(
        !fence.admit(stamp, Some(&server("rust-analyzer", 1))),
        "a result from a replaced server process was accepted"
    );
}

#[test]
fn nothing_is_accepted_while_no_file_is_displayed() {
    let mut fence = fence();
    fence.display(None);
    let stamp = DocumentStamp {
        file: 4,
        revision: 7,
    };
    assert_eq!(fence.verdict(stamp, None), FenceVerdict::OtherFile);
    assert!(
        !fence.admit(stamp, None),
        "a result was accepted while no file is displayed"
    );
}

#[test]
fn file_generation_outranks_revision_and_server() {
    let fence = fence();
    let other = DocumentStamp {
        file: 9,
        revision: 1,
    };
    assert_eq!(
        fence.verdict(other, Some(&server("rust-analyzer", 1))),
        FenceVerdict::OtherFile
    );
}

#[test]
fn every_verdict_is_counted() {
    let mut fence = fence();
    let current = DocumentStamp {
        file: 4,
        revision: 7,
    };
    fence.admit(current, None);
    fence.admit(
        DocumentStamp {
            file: 4,
            revision: 1,
        },
        None,
    );
    fence.admit(
        DocumentStamp {
            file: 1,
            revision: 7,
        },
        None,
    );
    fence.admit(current, Some(&server("typos", 9)));
    assert_eq!(
        fence.counts(),
        FenceCounts {
            accepted: 1,
            other_file: 1,
            stale_revision: 1,
            old_server: 1
        }
    );
}

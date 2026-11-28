//! What:
//!  Controls of the names the route's clone parent has:
//!  a dataset name that answers for both,
//!  a name with no component below its pool,
//!  a pool name alone,
//!  and an empty name.
//! Why:
//!  The filter compares a delivery against these two names,
//!  and a name that resolved to a pool with no dataset would accept or drop
//!  events the route never meant
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  veto entry V2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ParentScope.ofDataset('zpcachyos/nosnap/clones')).toEqual({ pool: 'zpcachyos', dataset: 'zpcachyos/nosnap/clones' });
//! ```

use super::ParentScope;

/// The pinned clone parent dataset the route's configuration carries.
const PARENT: &str = "zpcachyos/nosnap/zfsroute-clones";

/// A dataset name answers for both names.
#[test]
fn a_dataset_name_answers_for_its_pool_and_itself() {
    let parent: ParentScope = ParentScope::of_dataset(PARENT).expect("a dataset name");
    assert_eq!(parent.pool(), "zpcachyos");
    assert_eq!(parent.dataset(), PARENT);
}

/// A pool name alone names no parent this route could enumerate.
#[test]
fn a_pool_name_alone_resolves_to_nothing() {
    assert_eq!(ParentScope::of_dataset("zpcachyos"), None);
}

/// A name with nothing below its pool resolves to nothing.
#[test]
fn a_name_with_no_dataset_below_its_pool_resolves_to_nothing() {
    assert_eq!(ParentScope::of_dataset("zpcachyos/"), None);
    assert_eq!(ParentScope::of_dataset("/nosnap/zfsroute-clones"), None);
}

/// An empty name resolves to nothing.
#[test]
fn an_empty_name_resolves_to_nothing() {
    assert_eq!(ParentScope::of_dataset(""), None);
}

//! What:
//!  Controls of the parent's child listing and of the bound one sweep takes.
//! Why:
//!  The listing is the sweep's only unbounded loop,
//!  and a control cannot open `/dev/zfs`,
//!  so a scripted reader proves the cursor is the kernel's own,
//!  the end of the listing is distinct from a refusal,
//!  and a parent longer than the bound is refused rather than truncated.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(childrenOf(reader, PARENT)).toEqual(['pool/clones/one', 'pool/clones/two']);
//! ```

use super::{ChildrenRefusal, DatasetReader, ListedChild, Listing, MAXIMUM_CHILDREN, each_child};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

/// The parent the controls list.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// A reader that answers from a script and names the cursor it answered,
/// which is how a control sees the sweep's own loop.
struct ScriptedReader {
    /// The children the scripted pool holds.
    children: Vec<String>,
    /// The cursor the pool answers with a failure,
    /// or nothing when it never fails.
    fail_at: Option<u64>,
    /// How many generated children the pool answers,
    /// which is how the bound is exercised without writing four thousand names.
    generated: Option<usize>,
}

impl DatasetReader for ScriptedReader {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, String> {
        return Err(format!("this control lists children only, and cannot read `{dataset}`"));
    }

    fn next_child(&self, parent: &str, cursor: u64) -> Result<Option<ListedChild>, String> {
        assert_eq!(parent, PARENT, "the sweep asks the configured parent and no other");
        if self.fail_at == Some(cursor) {
            return Err(String::from("the device answered EIO"));
        }
        if let Some(bound) = self.generated {
            if usize::try_from(cursor).unwrap_or(usize::MAX) >= bound {
                return Ok(None);
            }
            return Ok(Some(ListedChild { name: format!("{PARENT}/generated-{cursor}"), cursor: cursor + 1 }));
        }
        return Ok(self.children.get(usize::try_from(cursor).unwrap_or(usize::MAX)).map(|name| return ListedChild { name: name.clone(), cursor: cursor + 1 }));
    }
}

/// A reader holding `children` below the parent.
fn reader_of(children: &[&str]) -> ScriptedReader {
    return ScriptedReader { children: children.iter().map(|name| return String::from(*name)).collect(), fail_at: None, generated: None };
}

/// Every child is listed in the order the kernel's own cursor gives them.
#[test]
fn every_child_is_listed_in_the_kernels_order() {
    let reader: ScriptedReader = reader_of(&[&format!("{PARENT}/one"), &format!("{PARENT}/two"), &format!("{PARENT}/three")]);
    let mut children: Vec<String> = Vec::new();
    let listing: Listing = each_child(&reader, PARENT, &mut |name| return children.push(String::from(name)));
    assert!(listing.refusal.is_none(), "the listing ends: {listing:?}");
    assert_eq!(listing.seen, 3);
    assert_eq!(children, vec![format!("{PARENT}/one"), format!("{PARENT}/two"), format!("{PARENT}/three")]);
}

/// A parent with no children lists nothing,
/// which is a finished sweep rather than a refusal.
#[test]
fn a_parent_with_no_children_lists_nothing() {
    let mut children: Vec<String> = Vec::new();
    let listing: Listing = each_child(&reader_of(&[]), PARENT, &mut |name| return children.push(String::from(name)));
    assert!(listing.refusal.is_none(), "{listing:?}");
    assert_eq!(listing.seen, 0);
    assert!(children.is_empty());
}

/// A step that refused is reported with the cursor it was asked after,
/// and the parent it walked.
#[test]
fn a_step_that_refused_is_reported_with_its_cursor() {
    let reader: ScriptedReader = ScriptedReader { children: vec![format!("{PARENT}/one"), format!("{PARENT}/two")], fail_at: Some(1), generated: None };
    let mut children: Vec<String> = Vec::new();
    let listing: Listing = each_child(&reader, PARENT, &mut |name| return children.push(String::from(name)));
    let Some(refusal) = listing.refusal else { panic!("the second step refused: {listing:?}") };
    assert_eq!(children, vec![format!("{PARENT}/one")], "the child the listing reached was visited before the stop");
    assert_eq!(listing.seen, 1);
    assert_eq!(refusal, ChildrenRefusal::Read { parent: String::from(PARENT), cursor: 1, reason: String::from("the device answered EIO") });
    let said: String = refusal.message();
    assert!(said.contains(PARENT), "the refusal names the parent: {said}");
    assert!(said.contains("after cursor 1"), "the refusal names the cursor: {said}");
    assert!(said.contains("keep their mounts"), "the refusal says what stands: {said}");
}

/// A parent longer than the bound is refused rather than truncated,
/// so a misconfigured parent is reported instead of half swept.
#[test]
fn a_parent_longer_than_the_bound_is_refused() {
    let endless: ScriptedReader = ScriptedReader { children: Vec::new(), fail_at: None, generated: Some(MAXIMUM_CHILDREN + 1) };
    let mut seen: usize = 0;
    let listing: Listing = each_child(&endless, PARENT, &mut |_name| return seen += 1);
    let Some(refusal) = listing.refusal else { panic!("a listing that never ends is bounded: {listing:?}") };
    assert_eq!(seen, MAXIMUM_CHILDREN);
    assert_eq!(refusal, ChildrenRefusal::TooMany { parent: String::from(PARENT), bound: MAXIMUM_CHILDREN });
    assert!(refusal.message().contains(&MAXIMUM_CHILDREN.to_string()), "the refusal names the bound: {}", refusal.message());
}

/// A listing exactly at the bound still ends,
/// which is what tells the bound from a truncation.
#[test]
fn a_listing_at_the_bound_ends() {
    let reader: ScriptedReader = ScriptedReader { children: Vec::new(), fail_at: None, generated: Some(MAXIMUM_CHILDREN) };
    let mut seen: usize = 0;
    let listing: Listing = each_child(&reader, PARENT, &mut |_name| return seen += 1);
    assert!(listing.refusal.is_none(), "a parent at the bound is listed whole: {listing:?}");
    assert_eq!(seen, MAXIMUM_CHILDREN);
}

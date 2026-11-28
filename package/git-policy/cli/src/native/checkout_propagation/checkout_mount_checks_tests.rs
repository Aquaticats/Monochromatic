//! What:
//!  The five checks that prove which mount was read,
//!  each refusing on its own disagreement:
//!  another point,
//!  another type,
//!  another dataset,
//!  no namespace,
//!  and a table with no row for the instance or a row describing another
//!  mount.
//! Why:
//!  The step sets a mount private on the strength of these checks,
//!  so a check that admitted one disagreement would change a mount that is
//!  not the checkout's:
//!  a bind of another dataset at the path,
//!  a tmpfs there,
//!  a replacement between the read and the table,
//!  or an instance a lazy unmount detached
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "What it verifies before",
//!  steps 3 to 5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkTheFacts(facts({ type: 'tmpfs' }), checkout, dataset)).toEqual({ word: 'type', why: /tmpfs/ });
//! ```

use super::*;
use crate::checkout_propagation::mount_descriptor_read::DescriptorMount;
use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::subvolume::fs_mount_options::MountFacts;
use std::path::{
    Path,
    PathBuf,
};

/// The checkout path every control here is about.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// The dataset the checkout's mount must serve.
const DATASET: &str = "pool/home/monochromatic-git-checkout";

/// The checkout's path,
/// as the value the checks compare against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkout = () => '/home/user/Monochromatic';
/// ```
fn checkout() -> &'static Path {
    return Path::new(CHECKOUT);
}

/// What:
///  One descriptor's facts,
///  with the fields a control changes named after it.
/// Why:
///  Every check compares three or four fields,
///  and a control states the one it disagrees on so nothing else moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = (over: Partial<DescriptorMount> = {}): DescriptorMount => ({ mountId: 9001, ... , ...over });
/// ```
fn facts(over: Option<DescriptorMount>) -> DescriptorMount {
    return over.unwrap_or(DescriptorMount {
        mount_id: 9001,
        mount_id_old: 41,
        namespace_id: Some(7),
        propagation: MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("shared"),
        peer_group: Some(247),
        master: None,
        mount_point: PathBuf::from(CHECKOUT),
        filesystem_type: String::from("zfs"),
        source: Some(String::from(DATASET)),
    });
}

/// What:
///  One mount table row for the checkout's instance.
/// Why:
///  The table check compares the row's three names with the facts',
///  so a control states the row and lets the check find it by the reused
///  number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (over: Partial<MountFacts> = {}): MountFacts => ({ mountId: 41, ... });
/// ```
fn row(over: Option<MountFacts>) -> MountFacts {
    return over.unwrap_or(MountFacts {
        mount_id: 41,
        parent_id: 1,
        root: PathBuf::from("/"),
        optional_fields: vec![String::from("shared:247")],
        mount_point: PathBuf::from(CHECKOUT),
        filesystem_type: String::from("zfs"),
        source: String::from(DATASET),
        mount_options: vec![String::from("rw")],
        super_options: Vec::new(),
    });
}

#[test]
fn the_checkout_own_instance_passes_every_check() {
    assert_eq!(check_the_facts(&facts(None), checkout(), DATASET), None);
    let table: Vec<MountFacts> = vec![row(None)];
    let found: &MountFacts = check_the_row(&facts(None), table.as_slice(), DATASET)
        .expect("the table holds the instance's row");
    assert_eq!(found.mount_id, 41);
}

#[test]
fn another_mount_point_refuses_with_the_point_word() {
    let mut one: DescriptorMount = facts(None);
    one.mount_point = PathBuf::from("/home/user/Monochromatic/.claude");
    let refusal: MountRefusal = check_the_facts(&one, checkout(), DATASET).expect("a refusal");
    assert_eq!(refusal.word, "point");
    assert!(
        refusal.why.contains(".claude"),
        "the refusal names both paths: {}",
        refusal.why
    );
}

#[test]
fn another_filesystem_type_refuses_with_the_type_word() {
    let mut one: DescriptorMount = facts(None);
    one.filesystem_type = String::from("tmpfs");
    let refusal: MountRefusal = check_the_facts(&one, checkout(), DATASET).expect("a refusal");
    assert_eq!(refusal.word, "type");
    assert!(refusal.why.contains("tmpfs"));
}

#[test]
fn another_dataset_refuses_with_the_source_word_where_the_kernel_reports_one() {
    let mut one: DescriptorMount = facts(None);
    one.source = Some(String::from("pool/home/another-dataset"));
    let refusal: MountRefusal = check_the_facts(&one, checkout(), DATASET).expect("a refusal");
    assert_eq!(refusal.word, "source");
    assert!(refusal.why.contains("another-dataset"));
}

#[test]
fn a_kernel_that_reports_no_source_leaves_the_table_to_answer_it() {
    let mut one: DescriptorMount = facts(None);
    one.source = None;
    assert_eq!(
        check_the_facts(&one, checkout(), DATASET),
        None,
        "the descriptor's answer cannot disagree about a source it does not report"
    );
    let mut elsewhere: MountFacts = row(None);
    elsewhere.source = String::from("pool/home/another-dataset");
    let table: Vec<MountFacts> = vec![elsewhere];
    let refusal: MountRefusal =
        check_the_row(&one, table.as_slice(), DATASET).expect_err("the row names another dataset");
    assert_eq!(refusal.word, "source");
}

#[test]
fn an_instance_with_no_namespace_refuses_with_the_namespace_word() {
    let mut one: DescriptorMount = facts(None);
    one.namespace_id = None;
    let refusal: MountRefusal = check_the_facts(&one, checkout(), DATASET).expect("a refusal");
    assert_eq!(refusal.word, "namespace");
    assert!(
        refusal.why.contains("lazy unmount"),
        "the refusal says what such an instance is: {}",
        refusal.why
    );
}

#[test]
fn a_table_with_no_row_for_the_instance_refuses_with_the_before_table_word() {
    let table: Vec<MountFacts> = vec![row(None)];
    let mut other: DescriptorMount = facts(None);
    other.mount_id_old = 42;
    let refusal: MountRefusal =
        check_the_row(&other, table.as_slice(), DATASET).expect_err("no row names mount 42");
    assert_eq!(refusal.word, "before-table");
    assert!(refusal.why.contains("42"));
}

#[test]
fn a_row_that_describes_another_mount_refuses_because_the_mount_was_replaced() {
    let mut elsewhere: MountFacts = row(None);
    elsewhere.source = String::from("pool/home/yet-another");
    let table: Vec<MountFacts> = vec![elsewhere];
    let refusal: MountRefusal = check_the_row(&facts(None), table.as_slice(), DATASET)
        .expect_err("the row names another mount at the same number");
    assert_eq!(refusal.word, "before-table");
    assert!(
        refusal.why.contains("replaced"),
        "the refusal says the two reads met a replacement: {}",
        refusal.why
    );
}

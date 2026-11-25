//! Controls for the preflight:
//!  a clean read set leaves a route,
//!  the protected set refuses before anything else,
//!  a blind read refuses,
//!  a locked holder refuses before any signal,
//!  a structural refusal names its namespace and reason,
//!  and the route falls to way 7 where way 8 does not fit.

use crate::worktree_clone_forced_removal::caller_chain_walk::SparedBy;
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_preflight::*;
use crate::worktree_clone_forced_removal::receiver_closure::{MountRow, NamespaceTable};

/// The dataset the clone's own mount carries.
const DATASET: &str = "tank/one";

/// One mount row with the tags the closure reads.
fn row(mount_id: u32, parent_id: u32, mount_point: &str, shared: &[u32], master: Option<u32>) -> MountRow {
    return MountRow {
        mount_id,
        parent_id,
        mount_point: String::from(mount_point),
        root: String::from("/"),
        filesystem: String::from("ext4"),
        source: String::from("/dev/vda1"),
        shared: shared.to_vec(),
        master,
    };
}

/// The clone's own mount at the slot.
fn the_copy(mount_id: u32, parent_id: u32) -> MountRow {
    return MountRow {
        mount_id,
        parent_id,
        mount_point: String::from("/checkout/one"),
        root: String::from("/"),
        filesystem: String::from("zfs"),
        source: String::from(DATASET),
        shared: Vec::new(),
        master: None,
    };
}

/// What:
///  One holder namespace whose parent shares with a group of its own,
///  which is the shape way 8 fits,
///  with every refusal's own input empty.
/// Why:
///  Each control changes one read and nothing else,
///  so the refusal it shows is the read it changed.
struct Fixture {
    /// The holder namespace's table.
    table: NamespaceTable,
    /// The copy the act is about.
    copy: MountRow,
    /// The namespaces the act handles.
    handled: Vec<String>,
}

impl Fixture {
    /// A clean fixture of one holder namespace.
    fn new() -> Fixture {
        return Fixture {
            table: NamespaceTable {
                namespace: String::from("mnt:u"),
                rows: vec![row(10, 1, "/checkout", &[7], None), the_copy(11, 10)],
                at_the_slot: Some(the_copy(11, 10)),
            },
            copy: the_copy(11, 10),
            handled: vec![String::from("mnt:u")],
        };
    }

    /// The reads a clean fixture answers.
    fn reads(&self) -> PreflightReads<'_> {
        return PreflightReads {
            dataset: DATASET,
            tables: std::slice::from_ref(&self.table),
            acting: "mnt:u",
            copy: &self.copy,
            handled: self.handled.as_slice(),
            protected: None,
            blind: &[],
            unenterable: &[],
            locked: &[],
            parents_not_the_checkout: &[],
            structural: &[],
            unendable: &[],
            no_destination: &[],
            handle_recorded: true,
        };
    }
}

/// A clean read set whose parent shares with its own group and whose handle is recorded takes way 8.
#[test]
fn a_clean_set_takes_way_eight() {
    let fixture: Fixture = Fixture::new();
    let plan: ForcedPlan = plan_the_forced_removal(&fixture.reads()).expect("the read set is clean");
    assert_eq!(plan.route, Route::InPlaceByTheHandle);
    assert_eq!(plan.route.word(), "in-place-by-the-handle");
    assert_eq!(plan.receivers.receivers.len(), 1);
}

/// The same set without the recorded handle takes way 7,
/// and way 7's own destination read then decides.
#[test]
fn no_handle_takes_way_seven_and_its_destination_read_decides() {
    let fixture: Fixture = Fixture::new();
    let mut reads: PreflightReads<'_> = fixture.reads();
    reads.handle_recorded = false;
    let plan: ForcedPlan = plan_the_forced_removal(&reads).expect("way 7 fits");
    assert_eq!(plan.route, Route::SlaveMoveAndUnmount);
    let no_destination: [(String, &'static str); 1] = [(String::from("mnt:u"), "the parent is read-only")];
    reads.no_destination = &no_destination;
    let refused: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("no destination refuses");
    assert_eq!(refused.word(), RefusalWord::HolderUnendable);
    assert!(matches!(refused, ForcedRefusal::NoDestination { .. }));
}

/// A holder in the protected set refuses with the design's word,
/// and it refuses before a blind read is looked at.
#[test]
fn the_protected_set_refuses_first() {
    let fixture: Fixture = Fixture::new();
    let mut reads: PreflightReads<'_> = fixture.reads();
    let blind: [String; 1] = [String::from("a namespace could not be read")];
    reads.blind = &blind;
    reads.protected = Some((4242, SparedBy::Session));
    let refused: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("a protected holder refuses");
    assert_eq!(refused.word(), RefusalWord::CallerProtectedHolds);
    assert_eq!(refused, ForcedRefusal::ProtectedHolds { holder: 4242, spared_by: SparedBy::Session });
}

/// A blind read refuses as `holder-blind`,
/// and a scan that could not read a namespace is one of those reads.
#[test]
fn a_blind_read_refuses() {
    let fixture: Fixture = Fixture::new();
    let mut reads: PreflightReads<'_> = fixture.reads();
    let blind: [String; 1] = [String::from("a namespace with no live thread may hold the clone")];
    reads.blind = &blind;
    let refused: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("a blind read refuses");
    assert_eq!(refused.word(), RefusalWord::HolderBlind);
    let ForcedRefusal::Blind { reads: named } = refused else {
        panic!("the refusal names the reads");
    };
    assert_eq!(named.len(), 1);
    assert!(named[0].contains("no live thread"), "{named:?}");
}

/// A locked holder refuses with its own word before any signal,
/// and the report names the holder and the namespace.
#[test]
fn a_locked_holder_refuses_before_any_signal() {
    let fixture: Fixture = Fixture::new();
    let mut reads: PreflightReads<'_> = fixture.reads();
    let locked: [(String, String); 1] = [(String::from("mnt:u"), String::from("unit watcher.service"))];
    reads.locked = &locked;
    let refused: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("a locked holder refuses");
    assert_eq!(refused.word(), RefusalWord::HolderLocked);
    assert_eq!(
        refused,
        ForcedRefusal::Locked { namespace: String::from("mnt:u"), holder: String::from("unit watcher.service") }
    );
}

/// A structural refusal names its namespace and the design's own reason,
/// and a copy whose parent is not the checkout's refuses as its own cause.
#[test]
fn a_structural_refusal_names_its_namespace_and_reason() {
    let fixture: Fixture = Fixture::new();
    let mut reads: PreflightReads<'_> = fixture.reads();
    let structural: [(String, &'static str); 1] = [(String::from("mnt:u"), "a mount under the copy")];
    reads.structural = &structural;
    let refused: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("a mount under the copy refuses");
    assert_eq!(
        refused,
        ForcedRefusal::Structure { namespace: String::from("mnt:u"), because: "a mount under the copy" }
    );
    let elsewhere: [String; 1] = [String::from("mnt:u")];
    reads.structural = &[];
    reads.parents_not_the_checkout = &elsewhere;
    let refused_2: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("the parent refuses");
    assert!(matches!(refused_2, ForcedRefusal::ParentIsNotTheCheckout { .. }));
    assert_eq!(refused_2.word(), RefusalWord::HolderUnendable);
}

/// A holder the endability read proved cannot be ended refuses as `holder-unendable`,
/// and a namespace the act cannot enter answers the same word.
#[test]
fn an_unendable_holder_or_unenterable_namespace_refuses() {
    let fixture: Fixture = Fixture::new();
    let mut reads: PreflightReads<'_> = fixture.reads();
    let unendable: [(i32, String); 1] = [(4242, String::from("a kernel thread"))];
    reads.unendable = &unendable;
    let refused: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("an unendable holder refuses");
    assert_eq!(refused.word(), RefusalWord::HolderUnendable);
    let unenterable: [String; 1] = [String::from("mnt:gone")];
    reads.unendable = &[];
    reads.unenterable = &unenterable;
    let refused_2: ForcedRefusal = plan_the_forced_removal(&reads).expect_err("an unenterable namespace refuses");
    assert_eq!(refused_2.word(), RefusalWord::HolderUnendable);
    assert_eq!(refused_2, ForcedRefusal::Unenterable { namespace: String::from("mnt:gone") });
}

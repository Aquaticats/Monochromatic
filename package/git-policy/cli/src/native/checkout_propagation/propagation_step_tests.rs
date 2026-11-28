//! What:
//!  The step's controls, one per state it can reach:
//!  a shared mount set private,
//!  a shared and slave mount set private with the master recorded,
//!  a private mount already as the route wants it,
//!  a slave and an unbindable mount left alone,
//!  and a refusal at each act and each check,
//!  with the record each one files.
//! Why:
//!  The step is the piece that changes a mount on this machine,
//!  so every state the kernel can hand it is proved here on fixture facts
//!  before the guest arm shows the same states on a real kernel
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The tests a build must have",
//!  tests 1 to 7 and 21 and 22).
//!  A control that only proved the repaired state would pass on a step that
//!  set a private or a slave mount private as well,
//!  which is the state the project decided to report and leave.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const calls = new Scripted({ reads: [shared(), privateAgain()] });
//! expect(runTheStep(places, at(checkout), calls).outcome).toBe('applied');
//! expect(calls.acts).toEqual(['open', 'read', 'table', 'set', 'read']);
//! ```

use super::*;
use crate::checkout_propagation::mount_descriptor_read::DescriptorMount;
use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::checkout_propagation::propagation_calls::{
    CallFailure,
    PropagationCalls,
};
use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_record_file::PropagationFile;
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use crate::subvolume::fs_mount_options::MountFacts;
use crate::testing::test_support::fixture;
use std::cell::RefCell;
use std::path::{
    Path,
    PathBuf,
};

/// The checkout path every control here is about.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// The dataset the checkout's mount must serve.
const DATASET: &str = "pool/home/monochromatic-git-checkout";

/// What:
///  The step's kernel acts, answered from a script,
///  with every act it was asked for and every refusal it was told to give.
/// Why:
///  No test on this machine may change a mount,
///  so the step's order and its decisions are proved by the acts the double
///  was asked for rather than by a kernel's answer,
///  and a control that says which acts ran catches a step that set a mount it
///  had decided to leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Scripted implements PropagationCalls { acts: string[] = []; reads: DescriptorMount[] = []; ... }
/// ```
struct Scripted {
    /// The acts the step asked for,
    /// in the order it asked.
    acts: RefCell<Vec<&'static str>>,
    /// What each read answers,
    /// taken from the front so the read after a set answers differently.
    reads: RefCell<Vec<DescriptorMount>>,
    /// This process's mount table.
    table: Vec<MountFacts>,
    /// The number a refused open answers,
    /// where the control wants one refused.
    refuse_open: Option<i32>,
    /// The number a refused set answers.
    refuse_set: Option<i32>,
    /// The number a refused table read answers.
    refuse_table: Option<i32>,
}

impl Scripted {
    /// What:
    ///  A double that answers every act well,
    ///  with the reads and the table the control states.
    /// Why:
    ///  Each control states only what its state needs,
    ///  so a well-answering double is the one every control starts from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const calls = new Scripted({ reads: [shared()], table: [rowFor(41)] });
    /// ```
    fn answering(
        reads: Vec<DescriptorMount>,
        table: Vec<MountFacts>,
    ) -> Self {
        return Scripted {
            acts: RefCell::new(Vec::new()),
            reads: RefCell::new(reads),
            table,
            refuse_open: None,
            refuse_set: None,
            refuse_table: None,
        };
    }

    /// The acts the step asked for,
    /// which a control compares with the order it expects.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const acts = (): string[] => this.acts;
    /// ```
    fn acts(&self) -> Vec<&'static str> {
        return self.acts.borrow().clone();
    }
}

impl PropagationCalls for Scripted {
    fn open_the_checkout(
        &self,
        _path: &Path,
    ) -> Result<std::fs::File, CallFailure> {
        self.acts.borrow_mut().push("open");
        if let Some(errno) = self.refuse_open {
            return Err(CallFailure::Open {
                errno,
            });
        }
        // A stand-in descriptor: no control reads it, and the double answers every read itself.
        let opened: std::fs::File =
            std::fs::File::open("/dev/null").expect("a descriptor to hand back");
        return Ok(opened);
    }

    fn mount_of(
        &self,
        _descriptor: &std::fs::File,
    ) -> Result<DescriptorMount, CallFailure> {
        self.acts.borrow_mut().push("read");
        let mut reads = self.reads.borrow_mut();
        if reads.is_empty() {
            panic!("the step read the mount more times than this control scripted")
        }
        let answer: DescriptorMount = reads.remove(0);
        return Ok(answer);
    }

    fn make_private(
        &self,
        _descriptor: &std::fs::File,
    ) -> Result<(), CallFailure> {
        self.acts.borrow_mut().push("set");
        return match self.refuse_set {
            Some(errno) => Err(CallFailure::Set {
                errno,
            }),
            None => Ok(()),
        };
    }

    fn read_the_table(&self) -> Result<Vec<MountFacts>, CallFailure> {
        self.acts.borrow_mut().push("table");
        return match self.refuse_table {
            Some(errno) => Err(CallFailure::Table {
                errno,
            }),
            None => Ok(self.table.clone()),
        };
    }
}

/// What:
///  The places one control's run reads and writes:
///  a fixture directory this account owns,
///  and a boot identifier the control states.
/// Why:
///  The release place is `/run`, which a control never writes,
///  and the record is read back from the fixture so a control proves the
///  file as well as the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = (): PropagationPlaces => ({ record: PropagationFile.at(fixture('...'), uid()), boot: 'b0ot' });
/// ```
fn places() -> PropagationPlaces {
    let directory: PathBuf = fixture("checkout-propagation-step");
    // SAFETY: `getuid` reads the calling process's own number and touches no memory.
    let owner: u32 = unsafe { libc::getuid() };
    return PropagationPlaces {
        record: PropagationFile::at(directory, owner),
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
    };
}

/// The checkout's resolution,
/// as a caller whose configuration named it hands the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = (): CheckoutResolution => ({ kind: 'at', checkout: '/home/user/Monochromatic', dataset: 'pool/home/monochromatic-git-checkout' });
/// ```
fn at() -> CheckoutResolution {
    return CheckoutResolution::At {
        checkout: PathBuf::from(CHECKOUT),
        dataset: String::from(DATASET),
    };
}

/// What:
///  The descriptor's facts for one set,
///  with the checkout's own names.
/// Why:
///  A control states the set it is about and nothing else moves,
///  so the same facts serve every state's test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = (propagation: MountPropagation, master?: number): DescriptorMount => ({ ... });
/// ```
fn facts(
    propagation: MountPropagation,
    master: Option<u64>,
) -> DescriptorMount {
    return DescriptorMount {
        mount_id: 9001,
        mount_id_old: 41,
        namespace_id: Some(7),
        propagation,
        peer_group: match propagation.carries_shared() {
            true => Some(247),
            false => None,
        },
        master,
        mount_point: PathBuf::from(CHECKOUT),
        filesystem_type: String::from("zfs"),
        source: Some(String::from(DATASET)),
    };
}

/// The mount table's one row for the checkout's instance,
/// with the tags the set carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (propagation: MountPropagation): MountFacts => ({ mountId: 41, ... });
/// ```
fn row(propagation: MountPropagation) -> MountFacts {
    let mut tags: Vec<String> = Vec::new();
    if propagation.carries_shared() {
        tags.push(String::from("shared:247"));
    }
    if propagation.word() == "slave" || propagation.word() == "shared+slave" {
        tags.push(String::from("master:3"));
    }
    return MountFacts {
        mount_id: 41,
        parent_id: 1,
        root: PathBuf::from("/"),
        optional_fields: tags,
        mount_point: PathBuf::from(CHECKOUT),
        filesystem_type: String::from("zfs"),
        source: String::from(DATASET),
        mount_options: vec![String::from("rw")],
        super_options: Vec::new(),
    };
}

/// The private set,
/// which every answer ends in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const private = (): MountPropagation => MountPropagation.fromKernel(MS_PRIVATE);
/// ```
fn private() -> MountPropagation {
    return MountPropagation::of_the_kernel_set(libc::MS_PRIVATE).expect("private");
}

/// The shared set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shared = (): MountPropagation => MountPropagation.fromKernel(MS_SHARED);
/// ```
fn shared() -> MountPropagation {
    return MountPropagation::of_the_kernel_set(libc::MS_SHARED).expect("shared");
}

/// The shared and slave set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const both = (): MountPropagation => MountPropagation.fromKernel(MS_SHARED | MS_SLAVE);
/// ```
fn both() -> MountPropagation {
    return MountPropagation::of_the_kernel_set(libc::MS_SHARED | libc::MS_SLAVE)
        .expect("shared+slave");
}

/// The slave set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slave = (): MountPropagation => MountPropagation.fromKernel(MS_SLAVE);
/// ```
fn slave() -> MountPropagation {
    return MountPropagation::of_the_kernel_set(libc::MS_SLAVE).expect("slave");
}

/// The unbindable set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unbindable = (): MountPropagation => MountPropagation.fromKernel(MS_UNBINDABLE);
/// ```
fn unbindable() -> MountPropagation {
    return MountPropagation::of_the_kernel_set(libc::MS_UNBINDABLE).expect("unbindable");
}

/// The record the run filed,
/// which a control reads back from the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filed = (places: PropagationPlaces): PropagationRecord => places.record.read().unwrap();
/// ```
fn filed(places: &PropagationPlaces) -> PropagationRecord {
    return places
        .record
        .read()
        .expect("a readable record")
        .expect("a record the run filed");
}

#[test]
fn a_shared_checkout_is_set_private_through_the_descriptor_and_the_record_says_so() {
    let places: PropagationPlaces = places();
    let calls: Scripted = Scripted::answering(
        vec![facts(shared(), None), facts(private(), None)],
        vec![row(shared())],
    );
    let done: PropagationDone = run_the_step(&places, &at(), &calls).expect("the repair");
    assert_eq!(done.outcome, PropagationOutcome::Applied);
    assert_eq!(
        calls.acts(),
        vec!["open", "read", "table", "set", "read"],
        "the set is made through the descriptor the read verified, and proved on the same one"
    );
    let record: PropagationRecord = filed(&places);
    assert_eq!(record.outcome, PropagationOutcome::Applied);
    assert_eq!(record.step, PropagationStep::NoStep);
    assert_eq!(record.before.map(|set| return set.word()), Some("shared"));
    assert_eq!(record.after.map(|set| return set.word()), Some("private"));
    assert_eq!(record.beneath.word(), "0");
    assert!(done.line.contains("applied"), "the line: {}", done.line);
}

#[test]
fn a_shared_and_slave_checkout_is_set_private_and_the_master_it_detaches_is_recorded() {
    let places: PropagationPlaces = places();
    let calls: Scripted = Scripted::answering(
        vec![facts(both(), Some(3)), facts(private(), None)],
        vec![row(both())],
    );
    let done: PropagationDone = run_the_step(&places, &at(), &calls).expect("the repair");
    assert_eq!(done.outcome, PropagationOutcome::Applied);
    let record: PropagationRecord = filed(&places);
    assert_eq!(
        record.before.map(|set| return set.word()),
        Some("shared+slave")
    );
    assert_eq!(
        record.master,
        Some(3),
        "the cost of setting such a mount private is the master it stops receiving"
    );
    assert!(
        done.line
            .contains("no longer receives that master's mounts"),
        "the line says the cost where a person meets it: {}",
        done.line
    );
}

#[test]
fn a_private_checkout_is_reported_as_already_private_and_nothing_is_set() {
    let places: PropagationPlaces = places();
    let calls: Scripted = Scripted::answering(vec![facts(private(), None)], vec![row(private())]);
    let done: PropagationDone = run_the_step(&places, &at(), &calls).expect("the report");
    assert_eq!(done.outcome, PropagationOutcome::AlreadyPrivate);
    assert_eq!(
        calls.acts(),
        vec!["open", "read", "table"],
        "a mount that is already private is never set"
    );
    let record: PropagationRecord = filed(&places);
    assert_eq!(done.outcome, PropagationOutcome::AlreadyPrivate);
    assert_eq!(
        record.before, record.after,
        "a run that changed nothing has one set"
    );
}

#[test]
fn a_slave_checkout_and_an_unbindable_one_are_reported_and_left() {
    for set in [slave(), unbindable()] {
        let places: PropagationPlaces = places();
        let calls: Scripted = Scripted::answering(
            vec![facts(
                set,
                match set.word() {
                    "slave" => Some(3),
                    _ => None,
                },
            )],
            vec![row(set)],
        );
        let done: PropagationDone = run_the_step(&places, &at(), &calls).expect("the report");
        assert_eq!(
            done.outcome,
            PropagationOutcome::Left,
            "a {} mount is the state the step reports and does not change",
            set.word()
        );
        assert_eq!(
            calls.acts(),
            vec!["open", "read", "table"],
            "the step sets nothing on a {} mount",
            set.word()
        );
        let record: PropagationRecord = filed(&places);
        assert_eq!(record.step, PropagationStep::NoStep);
        assert_eq!(record.before, record.after);
    }
}

#[test]
fn a_refused_set_refuses_with_its_number_and_changes_nothing() {
    let places: PropagationPlaces = places();
    let mut calls: Scripted = Scripted::answering(vec![facts(shared(), None)], vec![row(shared())]);
    calls.refuse_set = Some(libc::EPERM);
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("the kernel's refusal");
    assert_eq!(refusal.step, PropagationStep::Set);
    assert_eq!(refusal.errno, Some(libc::EPERM));
    assert!(
        refusal.message.contains("set refused"),
        "the sentence names the act: {}",
        refusal.message
    );
    let record: PropagationRecord = filed(&places);
    assert_eq!(record.outcome, PropagationOutcome::Refused);
    assert_eq!(record.step, PropagationStep::Set);
    assert_eq!(record.errno, Some(libc::EPERM));
    assert_eq!(record.after, None, "a refused set reached no set after it");
}

#[test]
fn a_set_that_reports_success_and_leaves_the_set_standing_is_refused() {
    let places: PropagationPlaces = places();
    let calls: Scripted = Scripted::answering(
        vec![facts(shared(), None), facts(shared(), None)],
        vec![row(shared())],
    );
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("the read after the set");
    assert_eq!(refusal.step, PropagationStep::VerifyAfter);
    let record: PropagationRecord = filed(&places);
    assert_eq!(record.step, PropagationStep::VerifyAfter);
    assert_eq!(
        record.after.map(|set| return set.word()),
        Some("shared"),
        "the record says what the read after the set found"
    );
}

#[test]
fn a_set_on_another_instance_is_refused_because_the_unique_number_moved() {
    let places: PropagationPlaces = places();
    let mut replaced: DescriptorMount = facts(private(), None);
    replaced.mount_id = 9002;
    let calls: Scripted =
        Scripted::answering(vec![facts(shared(), None), replaced], vec![row(shared())]);
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("another instance's answer");
    assert_eq!(
        refusal.step,
        PropagationStep::VerifyAfter,
        "an answer about another mount does not prove this one"
    );
    let record: PropagationRecord = filed(&places);
    assert_eq!(
        record.mount_id,
        Some(9001),
        "the record names the instance the before read gave"
    );
}

#[test]
fn an_open_that_refuses_files_a_record_with_none_in_every_field_it_never_reached() {
    let places: PropagationPlaces = places();
    let mut calls: Scripted = Scripted::answering(Vec::new(), Vec::new());
    calls.refuse_open = Some(libc::EACCES);
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("the path could not be opened");
    assert_eq!(refusal.step, PropagationStep::Open);
    assert_eq!(refusal.errno, Some(libc::EACCES));
    let record: PropagationRecord = filed(&places);
    assert_eq!(record.outcome, PropagationOutcome::Refused);
    assert_eq!(record.checkout, None);
    assert_eq!(record.beneath.word(), "unread");
}

#[test]
fn each_fact_that_disagrees_refuses_with_its_own_word() {
    let cases: [(&'static str, fn(&mut DescriptorMount)); 4] = [
        ("point", |facts| {
            return facts.mount_point = PathBuf::from("/home/user/other");
        }),
        ("type", |facts| {
            return facts.filesystem_type = String::from("btrfs");
        }),
        ("source", |facts| {
            return facts.source = Some(String::from("pool/home/another-dataset"));
        }),
        ("namespace", |facts| return facts.namespace_id = None),
    ];
    for (word, disagree) in cases {
        let places: PropagationPlaces = places();
        let mut one: DescriptorMount = facts(shared(), None);
        disagree(&mut one);
        let calls: Scripted = Scripted::answering(vec![one], vec![row(shared())]);
        let refusal: PropagationRefusal =
            run_the_step(&places, &at(), &calls).expect_err("a disagreement");
        assert_eq!(refusal.step.word(), word);
        assert_eq!(
            calls.acts(),
            vec!["open", "read"],
            "a check that refuses authorizes no table read and no set"
        );
    }
}

#[test]
fn a_table_the_step_could_not_read_refuses_with_the_before_table_word() {
    let places: PropagationPlaces = places();
    let mut calls: Scripted = Scripted::answering(vec![facts(shared(), None)], Vec::new());
    calls.refuse_table = Some(libc::E2BIG);
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("the table refused");
    assert_eq!(refusal.step, PropagationStep::BeforeTable);
    assert_eq!(
        calls.acts(),
        vec!["open", "read", "table"],
        "no set follows a table the step could not read"
    );
}

#[test]
fn an_instance_the_table_does_not_describe_refuses_and_records_unread_beneath() {
    let places: PropagationPlaces = places();
    let mut elsewhere: MountFacts = row(shared());
    elsewhere.mount_id = 42;
    let calls: Scripted = Scripted::answering(vec![facts(shared(), None)], vec![elsewhere]);
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("no row for the instance");
    assert_eq!(refusal.step, PropagationStep::BeforeTable);
    let record: PropagationRecord = filed(&places);
    assert_eq!(
        record.beneath.word(),
        "unread",
        "a run that refused before it read the table says it never looked"
    );
}

#[test]
fn a_caller_whose_resolution_refused_files_that_refusal_with_the_resolve_word() {
    let places: PropagationPlaces = places();
    let calls: Scripted = Scripted::answering(Vec::new(), Vec::new());
    let refused: CheckoutResolution = CheckoutResolution::Refused {
        why: String::from(
            "the configuration names no checkout and the mount table holds two datasets whose last name is monochromatic-git-checkout",
        ),
    };
    let refusal: PropagationRefusal =
        run_the_step(&places, &refused, &calls).expect_err("the caller's refusal");
    assert_eq!(refusal.step, PropagationStep::Resolve);
    assert!(
        calls.acts().is_empty(),
        "a checkout that was not resolved opens nothing and reads nothing"
    );
    let record: PropagationRecord = filed(&places);
    assert_eq!(record.step, PropagationStep::Resolve);
    assert_eq!(record.checkout, None);
}

#[test]
fn the_shared_mounts_beneath_are_counted_for_an_answer_and_for_a_refused_set() {
    let mut nested: MountFacts = row(shared());
    nested.mount_id = 267;
    nested.parent_id = 41;
    nested.mount_point = PathBuf::from("/home/user/Monochromatic/package/vm-builder/cli/output");
    for refuse_set in [None, Some(libc::EPERM)] {
        let places: PropagationPlaces = places();
        let mut calls: Scripted = Scripted::answering(
            vec![facts(shared(), None), facts(private(), None)],
            vec![row(shared()), nested.clone()],
        );
        calls.refuse_set = refuse_set;
        let _answer: Result<PropagationDone, PropagationRefusal> =
            run_the_step(&places, &at(), &calls);
        let record: PropagationRecord = filed(&places);
        assert_eq!(
            record.beneath.word(),
            "1",
            "one shared mount stands beneath the checkout, and the record counts it whether the set ran or refused"
        );
        assert_eq!(
            record.beneath.paths(),
            &[String::from(
                "/home/user/Monochromatic/package/vm-builder/cli/output"
            )]
        );
    }
}

#[test]
fn a_record_that_cannot_be_filed_is_its_own_refusal() {
    let directory: PathBuf = fixture("checkout-propagation-unwritable");
    // SAFETY: `getuid` reads the calling process's own number and touches no memory.
    let owner: u32 = unsafe { libc::getuid() };
    // The place names a file rather than a folder, so no directory can be made or proved there.
    let place: PathBuf = directory.join("a-file");
    std::fs::write(place.as_path(), "not a folder").expect("a file where a folder must be");
    let places: PropagationPlaces = PropagationPlaces {
        record: PropagationFile::at(place, owner),
        boot: String::from("6f1c2b8a-2b9f-4b4d-9a1f-0e5e6f7a8b9c"),
    };
    let calls: Scripted = Scripted::answering(vec![facts(private(), None)], vec![row(private())]);
    let refusal: PropagationRefusal =
        run_the_step(&places, &at(), &calls).expect_err("the answer could not be filed");
    assert_eq!(refusal.step, PropagationStep::Record);
    assert!(
        refusal.message.contains("could not be filed"),
        "the sentence says the answer was lost: {}",
        refusal.message
    );
}

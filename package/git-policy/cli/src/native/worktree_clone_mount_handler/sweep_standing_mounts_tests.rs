//! What:
//!  Controls of the standing-mount answer:
//!  the poll that keeps the table copy right,
//!  the ways that poll cannot be made,
//!  a re-read that fails,
//!  and the one mount's source,
//!  which the kernel is asked for rather than read from the copy.
//! Why:
//!  The rename gap this round closes is a source that moves without a mount
//!  event,
//!  so the controls mount a tmpfs inside a namespace of their own and shape it
//!  as a clone's own mount,
//!  and the answers say what the kernel and the copy each report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(standing.standingMountOf(request, CLONE)).toBeUndefined(); // the kernel named another source
//! ```

use crate::worktree_clone_mount_handler::sweep_standing_mount_source::{KernelMountSource, MountSourceAnswer, MountSourceByNumber, STATMOUNT_NUMBER};
use crate::worktree_clone_mount_handler::standing_mount_source_call_refusal::{refuse_the_calls, the_call_must_answer};
use super::{LiveMountTableSource, LiveStandingMounts, MountTableSource, StandingMounts, is_this_clones_own_mount, the_facts_are_a_datasets_own_mount};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::LookAgainAtSlotRequest;
use std::cell::RefCell;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// The variable each control's inner half sets,
/// because a tmpfs is mounted in the control's own namespace.
const INSIDE: &str = "CLI_GIT_STANDING_MOUNTS_ROOT";

/// The clone the controls ask about.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// The source the controls mount the tmpfs at a slot with,
/// which the kernel reports for that mount while the mount-source call answers.
const THE_SOURCE_THE_SLOT_IS_MOUNTED_WITH: &str = "zfs-step-8-standing-mounts-clone";

/// One table source a control owns:
/// the tables it answers in order,
/// whether the poll says the table changed,
/// and how many reads it was asked for.
#[derive(Clone)]
struct ScriptedTable {
    /// The tables in order;
    /// the last one answers every later read.
    tables: std::rc::Rc<RefCell<Vec<Result<Vec<MountFacts>, String>>>>,
    /// Whether the poll says the table changed,
    /// which the control sets when it does.
    changed: std::rc::Rc<RefCell<bool>>,
    /// How many reads were asked for.
    reads: std::rc::Rc<RefCell<usize>>,
}

impl ScriptedTable {
    /// A source holding `tables`,
    /// with the poll quiet.
    fn new(tables: Vec<Result<Vec<MountFacts>, String>>) -> Self {
        return Self { tables: std::rc::Rc::new(RefCell::new(tables)), changed: std::rc::Rc::new(RefCell::new(false)), reads: std::rc::Rc::new(RefCell::new(0)) };
    }

    /// Say that one change happened since the last poll.
    fn change(&self) {
        *self.changed.borrow_mut() = true;
    }

    /// How many reads have been asked for.
    fn read_count(&self) -> usize {
        return *self.reads.borrow();
    }
}

impl MountTableSource for ScriptedTable {
    fn changed(&self) -> bool {
        let mut changed = self.changed.borrow_mut();
        let answered: bool = *changed;
        *changed = false;
        return answered;
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        *self.reads.borrow_mut() += 1;
        let mut tables = self.tables.borrow_mut();
        if tables.len() > 1 {
            return tables.remove(0);
        }
        return tables.first().cloned().unwrap_or_else(|| return Ok(Vec::new()));
    }
}

/// A source of one mount's source that answers from a script.
struct ScriptedSource {
    /// What it says about any mount.
    answer: MountSourceAnswer,
    /// How many times it was asked.
    asked: std::rc::Rc<RefCell<usize>>,
}

impl MountSourceByNumber for ScriptedSource {
    fn source_of(&self, _unique_mount_number: Option<u64>) -> MountSourceAnswer {
        *self.asked.borrow_mut() += 1;
        return self.answer.clone();
    }
}

/// One table mount the controls build.
fn table_mount(id: u64, point: &str, source: &str) -> MountFacts {
    return MountFacts {
        mount_id: id,
        parent_id: 40,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: PathBuf::from(point),
        filesystem_type: String::from("zfs"),
        source: String::from(source),
        mount_options: vec![String::from("rw"), String::from("nosuid"), String::from("nodev")],
        super_options: Vec::new(),
    };
}

/// The mount id the kernel gave the tmpfs at `path`.
fn mount_number_of(path: &Path) -> u64 {
    let opened: std::fs::File = std::fs::File::open(path).expect("the folder opens");
    return crate::fs_beneath_mount_facts::mount_id_of(&opened).expect("the folder's mount number reads");
}

/// The unique mount number of the mount the tmpfs at `path` is on,
/// which the mount-source call takes.
fn unique_number_of(path: &Path) -> u64 {
    let opened: std::fs::File = std::fs::File::open(path).expect("the folder opens");
    return crate::fs_beneath_mount_facts::unique_mount_number_of(&opened).expect("the folder's unique mount number reads");
}

/// One tmpfs mount a control makes,
/// taken down however the control ends.
struct MountedSlot {
    /// The mount point.
    path: PathBuf,
}

impl Drop for MountedSlot {
    fn drop(&mut self) {
        let target: std::ffi::CString = std::ffi::CString::new(self.path.as_os_str().as_bytes()).expect("no NUL in the path");
        // SAFETY: `target` is a NUL-terminated path that outlives the call, and no flag is passed.
        let _taken: libc::c_int = unsafe { libc::umount2(target.as_ptr(), 0) };
    }
}

/// Mount one tmpfs named `source` at `path` with `flags`.
fn mount_tmpfs(source: &str, path: &Path, flags: libc::c_ulong) {
    let source_name: std::ffi::CString = std::ffi::CString::new(source).expect("no NUL in the source");
    let target: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in the path");
    let kind: std::ffi::CString = std::ffi::CString::new("tmpfs").expect("no NUL in the type");
    // SAFETY: both strings outlive the call, the type is a NUL-terminated name, and no data pointer is read.
    let mounted: libc::c_int = unsafe { libc::mount(source_name.as_ptr(), target.as_ptr(), kind.as_ptr(), flags, std::ptr::null()) };
    assert_eq!(mounted, 0, "the control's tmpfs mounts: {}", std::io::Error::last_os_error());
}

/// One fixture checkout with one slot folder,
/// and a tmpfs at the slot shaped as a clone's own mount would be.
struct ShapedSlot {
    /// The checkout the walk starts at.
    checkout: PathBuf,
    /// The slot's folder.
    slot: PathBuf,
    /// The mount,
    /// taken down when the control ends.
    _guard: MountedSlot,
}

impl ShapedSlot {
    /// Build one,
    /// with the tmpfs carrying both enforced flags.
    fn build(name: &str) -> Self {
        let checkout: PathBuf = crate::test_support::fixture(name).join("checkout");
        let slot: PathBuf = checkout.join(".claude").join("worktrees").join("one");
        std::fs::create_dir_all(slot.as_path()).expect("the slot folder is made");
        mount_tmpfs(THE_SOURCE_THE_SLOT_IS_MOUNTED_WITH, slot.as_path(), libc::MS_NOSUID | libc::MS_NODEV);
        return Self { checkout, _guard: MountedSlot { path: slot.clone() }, slot };
    }

    /// The facts of this control's tmpfs,
    /// shaped as a dataset's own mount of `source`.
    fn shaped(&self, source: &str) -> MountFacts {
        let mut facts: MountFacts = table_mount(mount_number_of(self.slot.as_path()), self.slot.display().to_string().as_str(), source);
        facts.parent_id = 40;
        return facts;
    }

    /// The request the answer is asked for.
    fn request(&self) -> LookAgainAtSlotRequest<'_> {
        return LookAgainAtSlotRequest { checkout: self.checkout.as_path(), slot: "one" };
    }
}

/// What:
///  A source the kernel names for the mount at the slot decides the answer,
///  even where the copy calls that mount the clone's own.
/// Why:
///  A rename of a mounted `legacy` dataset moves the source the kernel prints
///  and raises no mount event,
///  so a copy that says `own` can name another dataset's mount;
///  this is the control of the check that closes it.
#[test]
fn a_copy_that_calls_a_mount_own_is_refused_when_the_kernel_names_another_source() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mounts::tests::a_copy_that_calls_a_mount_own_is_refused_when_the_kernel_names_another_source",
    ) {
        return;
    }
    let shaped: ShapedSlot = ShapedSlot::build("standing-source-renamed-away");
    let table: ScriptedTable = ScriptedTable::new(vec![Ok(vec![shaped.shaped(CLONE)])]);
    let asked: std::rc::Rc<RefCell<usize>> = std::rc::Rc::new(RefCell::new(0));
    let sources: ScriptedSource = ScriptedSource { answer: MountSourceAnswer::Source(String::from("zpcachyos/nosnap/clones/moved-away")), asked: asked.clone() };
    let standing: LiveStandingMounts = LiveStandingMounts::from_sources(Box::new(table), Box::new(sources));
    assert!(is_this_clones_own_mount(&shaped.shaped(CLONE), CLONE), "the copy this control writes calls the mount the clone's own");
    assert!(standing.standing_mount_of(&shaped.request(), CLONE).is_none(), "the kernel's source is another dataset's, so the mount is not this clone's");
    assert_eq!(*asked.borrow(), 1, "the kernel was asked once about the one mount at the slot");
}

/// What:
///  Where the kernel names the clone's own source,
///  the answer is the mount at the slot.
/// Why:
///  A control that only proves refusals cannot tell the check from one that
///  refuses everything.
#[test]
fn a_copy_that_calls_a_mount_own_is_accepted_when_the_kernel_names_the_clones_own_source() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mounts::tests::a_copy_that_calls_a_mount_own_is_accepted_when_the_kernel_names_the_clones_own_source",
    ) {
        return;
    }
    let shaped: ShapedSlot = ShapedSlot::build("standing-source-own");
    let table: ScriptedTable = ScriptedTable::new(vec![Ok(vec![shaped.shaped(CLONE)])]);
    let sources: ScriptedSource = ScriptedSource { answer: MountSourceAnswer::Source(String::from(CLONE)), asked: std::rc::Rc::new(RefCell::new(0)) };
    let standing: LiveStandingMounts = LiveStandingMounts::from_sources(Box::new(table), Box::new(sources));
    let found = standing.standing_mount_of(&shaped.request(), CLONE).expect("the kernel names this clone's own source");
    assert_eq!(found.source, CLONE);
    assert_eq!(found.observed_path, shaped.slot, "the mount the answer names is the one the walk found");
}

/// What:
///  Where the kernel cannot name a mount's source,
///  the answer is the live table's whole check,
///  which is the call `main` makes.
/// Why:
///  A kernel before 6.13 reports no source at all,
///  and there the route must answer what it answered before the copy existed,
///  not refuse a mount that stands.
#[test]
fn a_kernel_that_cannot_name_a_source_is_answered_from_the_live_table() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mounts::tests::a_kernel_that_cannot_name_a_source_is_answered_from_the_live_table",
    ) {
        return;
    }
    let shaped: ShapedSlot = ShapedSlot::build("standing-source-unavailable");
    let table: ScriptedTable = ScriptedTable::new(vec![Ok(vec![shaped.shaped(CLONE)])]);
    let sources: ScriptedSource = ScriptedSource { answer: MountSourceAnswer::NoAnswer(String::from("this kernel has no sb_source")), asked: std::rc::Rc::new(RefCell::new(0)) };
    let standing: LiveStandingMounts = LiveStandingMounts::from_sources(Box::new(table), Box::new(sources));
    let found = standing.standing_mount_of(&shaped.request(), CLONE).expect("the live table names this clone's own mount, as it did before the copy");
    assert_eq!(found.source, CLONE, "the answer carries the live table's source");
}

/// What:
///  A call a seccomp filter refuses is answered from the live table,
///  and that fallback's answer is the one the table's whole predicate gives.
/// Why:
///  The fallback is the half a kernel before 6.13 and a container whose profile
///  refuses the call both take,
///  and only a filter can make the call answer nothing on a kernel that has it:
///  this control reads the call's answer first,
///  refuses it with a filter on this thread,
///  and then proves the release reader answers from the table,
///  so the same process shows both halves and the denial is known to be the
///  filter's.
#[test]
fn a_call_the_filter_refuses_is_answered_from_the_live_table() {
    let control: &str = "worktree_clone_mount_handler::sweep_standing_mounts::tests::a_call_the_filter_refuses_is_answered_from_the_live_table";
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        control,
    ) {
        return;
    }
    let shaped: ShapedSlot = ShapedSlot::build("standing-source-refused");
    let unique: u64 = unique_number_of(shaped.slot.as_path());
    // The call answers before the filter, which is the positive control the denial below is read against.
    match KernelMountSource.source_of(Some(unique)) {
        MountSourceAnswer::Source(source) => assert_eq!(source, THE_SOURCE_THE_SLOT_IS_MOUNTED_WITH, "the kernel named the source the slot is mounted with"),
        MountSourceAnswer::NoAnswer(why) => {
            // A container whose own profile refuses the call reaches this arm, and the scripted controls cover the
            // fallback there; the run's statement decides whether that is acceptable.
            if the_call_must_answer(control, why.as_str()) {
                panic!("this run states that its containers allow statmount(2) through their seccomp profile, and the call answered nothing before any filter was installed: {why}");
            }
            return;
        }
    }
    // What: the call number this build sends, which the filter has to refuse by name.
    let statmount_number: libc::c_long = STATMOUNT_NUMBER.expect("this architecture declares the call");
    refuse_the_calls(&[statmount_number]);
    // The call now answers ENOSYS, which is what a refused call looks like to the release reader.
    let refused: MountSourceAnswer = KernelMountSource.source_of(Some(unique));
    assert!(matches!(refused, MountSourceAnswer::NoAnswer(_)), "the filter refuses the call, so it names no source: {refused:?}");
    let table: ScriptedTable = ScriptedTable::new(vec![Ok(vec![shaped.shaped(CLONE)])]);
    let standing: LiveStandingMounts = LiveStandingMounts::from_sources(Box::new(table), Box::new(KernelMountSource));
    let found = standing.standing_mount_of(&shaped.request(), CLONE).expect("the live table names this clone's own mount, as it did before the copy");
    assert_eq!(found.source, CLONE, "the answer carries the live table's source");
    assert!(
        standing.standing_mount_of(&shaped.request(), "zpcachyos/nosnap/clones/two").is_none(),
        "the live table's source is another clone's, so this is not the clone asked about"
    );
}

/// What:
///  A kernel that cannot name a source,
///  where the live table does not call the mount the clone's own,
///  answers nothing.
/// Why:
///  The fallback is the whole predicate on a fresh read,
///  so the source clause has to be enforced there too.
#[test]
fn the_live_fallback_refuses_a_mount_the_table_does_not_call_the_clones_own() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mounts::tests::the_live_fallback_refuses_a_mount_the_table_does_not_call_the_clones_own",
    ) {
        return;
    }
    let shaped: ShapedSlot = ShapedSlot::build("standing-source-fallback-refused");
    let table: ScriptedTable = ScriptedTable::new(vec![Ok(vec![shaped.shaped(CLONE)])]);
    let sources: ScriptedSource = ScriptedSource { answer: MountSourceAnswer::NoAnswer(String::from("this kernel has no sb_source")), asked: std::rc::Rc::new(RefCell::new(0)) };
    let standing: LiveStandingMounts = LiveStandingMounts::from_sources(Box::new(table), Box::new(sources));
    assert!(standing.standing_mount_of(&shaped.request(), "zpcachyos/nosnap/clones/another").is_none(), "the live table's source is another clone's, so this is not the clone asked about");
}

/// What:
///  A table whose re-read fails does not go on answering from the copy.
/// Why:
///  The copy then describes a moment nobody can vouch for,
///  and `main` would read the live table for that mount;
///  the sweep clears the copy and reads live.
#[test]
fn a_reread_that_fails_does_not_keep_the_stale_copy() {
    let table: ScriptedTable = ScriptedTable::new(vec![Ok(vec![table_mount(7_000_011, "/fixture/one", CLONE)]), Err(String::from("the table could not be read"))]);
    let sources: ScriptedSource = ScriptedSource { answer: MountSourceAnswer::NoAnswer(String::from("unused by this control")), asked: std::rc::Rc::new(RefCell::new(0)) };
    let standing: LiveStandingMounts = LiveStandingMounts::from_sources(Box::new(table.clone()), Box::new(sources));
    assert_eq!(standing.facts_of(7_000_011).expect("the first read holds this mount").source, CLONE);
    table.change();
    let after: Result<MountFacts, String> = standing.facts_of(7_000_011);
    assert!(after.is_err(), "the failed re-read leaves the copy empty, so this mount is read live and the live table does not hold it: {after:?}");
    assert_eq!(table.read_count(), 2, "the change made the sweep read once more");
}

/// What:
///  A descriptor that cannot be opened counts as a change,
///  so the sweep keeps reading the table rather than trusting a copy nothing
///  watches.
/// Why:
///  This is the fail-open path:
///  a call that could not be made proves nothing about the table,
///  so without a descriptor the answer must be `main`'s,
///  not the copy's.
#[test]
fn a_watch_with_no_descriptor_counts_as_a_change() {
    let watch: LiveMountTableSource = LiveMountTableSource::without_a_descriptor();
    assert!(MountTableSource::changed(&watch), "a poll this build cannot make is not evidence that nothing changed");
}

/// What:
///  A poll that refuses counts as a change.
/// Why:
///  A call that did not answer proves nothing about the table,
///  so the next answer reads it again.
#[test]
fn a_watch_whose_poll_refuses_counts_as_a_change() {
    let watch: LiveMountTableSource = LiveMountTableSource::whose_poll_answers(None);
    assert!(MountTableSource::changed(&watch), "a poll that refused is a change, not a quiet table");
}

/// What:
///  A poll that answers with nothing watched counts as a change.
/// Why:
///  The kernel's answer `POLLNVAL` says the descriptor it was asked about is
///  gone,
///  so nothing proves the table stood still.
#[test]
fn a_watch_whose_descriptor_the_kernel_lost_counts_as_a_change() {
    let watch: LiveMountTableSource = LiveMountTableSource::whose_poll_answers(Some(libc::POLLNVAL));
    assert!(MountTableSource::changed(&watch), "a descriptor the kernel no longer knows is a change");
}

/// What:
///  The real poll is quiet when nothing changes,
///  fires once after a change,
///  and is quiet again without a read in between.
/// Why:
///  The kernel remembers the event its open saw,
///  so a fire is consumed by the poll that reported it;
///  a control that never saw the quiet answer could not tell a working watch
///  from one that always says the table changed (which would cost `main`'s
///  reads) or never says it (which would keep a wrong copy).
#[test]
fn the_real_poll_is_quiet_until_a_change_and_quiet_again_after_it() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_standing_mounts::tests::the_real_poll_is_quiet_until_a_change_and_quiet_again_after_it",
    ) {
        return;
    }
    let root: PathBuf = crate::test_support::fixture("standing-poll-quiet");
    let slot: PathBuf = root.join("slot");
    std::fs::create_dir_all(slot.as_path()).expect("the folder is made");
    let watch: LiveMountTableSource = LiveMountTableSource::open();
    assert!(!MountTableSource::changed(&watch), "a table nothing changed is quiet");
    assert!(!MountTableSource::changed(&watch), "and it stays quiet when asked again");
    mount_tmpfs("zfs-step-8-standing-poll", slot.as_path(), libc::MS_NOSUID | libc::MS_NODEV);
    let _guard: MountedSlot = MountedSlot { path: slot.clone() };
    assert!(MountTableSource::changed(&watch), "a mount raises the event the poll reports");
    assert!(!MountTableSource::changed(&watch), "the fire is consumed by the poll that reported it, with no read between");
}

/// What:
///  The answer needs the two attributes and the root the copy carries,
///  and the source it does not.
/// Why:
///  The predicate is split so the source can come from the kernel;
///  this control pins the half that still comes from the table.
#[test]
fn a_dataset_mount_is_told_by_its_type_root_and_two_attributes() {
    let own: MountFacts = table_mount(7_000_012, "/fixture/one", CLONE);
    assert!(the_facts_are_a_datasets_own_mount(&own));
    assert!(is_this_clones_own_mount(&own, CLONE));
    let bind: MountFacts = MountFacts { root: PathBuf::from("/sub"), ..own.clone() };
    assert!(!the_facts_are_a_datasets_own_mount(&bind));
    let without_nodev: MountFacts = MountFacts { mount_options: vec![String::from("rw"), String::from("nosuid")], ..own.clone() };
    assert!(!the_facts_are_a_datasets_own_mount(&without_nodev));
    let another_type: MountFacts = MountFacts { filesystem_type: String::from("tmpfs"), ..own.clone() };
    assert!(!the_facts_are_a_datasets_own_mount(&another_type));
    let another_source: MountFacts = MountFacts { source: String::from("zpcachyos/nosnap/clones/two"), ..own };
    assert!(the_facts_are_a_datasets_own_mount(&another_source), "the dataset is not this half's question");
    assert!(!is_this_clones_own_mount(&another_source, CLONE), "the whole predicate still compares the source");
}

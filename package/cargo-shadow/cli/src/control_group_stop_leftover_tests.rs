//! What:
//! The leftovers a stop of a scope can report,
//! each driven through the decision the run is refused on (`refusal_of_scope`),
//! and the bound that cuts the listing short:
//! a member the listing read as one that has ended,
//! a listing cut before it read one member,
//! a listing cut after it read one,
//! a group below the scope that could not be read,
//! a member whose threads were not all read within the bound,
//! a listing that read no member while the kernel said the scope was populated,
//! and the order the listing names the groups of a scope in.
//! Why:
//! Release read of `0.1.5-alpha.12` (Medium 2 and Medium 3,
//! 2026-11-27):
//! `refusal_of_scope` returning `None` for a `NoWriter` or a `Cut` leftover would have stayed green,
//! because both leftovers appeared in the tests only as hand-built values read through `headline` and `sentence`
//! (`control_group_stop_report_tests.rs`),
//! never through the decision;
//! and the listing's bound was reached by no test at all.
//! Each control here reaches the decision with a report a stop made of a real scope,
//! and each says which failpoint stands for an outcome nothing outside a test build can hold a process in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the leftovers a stop refuses on', () => { it('a member that ended after it was listed', ...) })
//! ```

/// Reads a program's state,
/// and writes members into a scope and a group below it.
use std::fs;
/// Borrows and owns the paths of the scope's directory and of the group below it.
use std::path::{
    Path,
    PathBuf,
};
/// Starts the programs a control moves into a scope,
/// and reaps them when the control ends.
use std::process::{
    Child,
    Command,
};
/// The bound the stop is given,
/// and the point on the clock that is already past.
use std::time::Instant;

/// The bound the stop of a scope waits for `populated 0`.
use crate::bounded_run::QUIESCE_BOUND;
/// The decision itself,
/// the wait for the direct child,
/// and what became of it.
use crate::bounded_run_containment::{
    Reaped,
    reap,
    refusal_of_scope,
};
/// A scope this test starts the way a run starts one,
/// the proof that this host can give it,
/// and the handshake that waits
/// for a member to be listed.
use crate::cgroup_test_support::{
    TestScope,
    start_in_scope,
    this_test_needs_a_delegated_scope,
    wait_until,
};
/// The scope whose directory a member is moved into and whose groups are made.
use crate::control_group::ControlGroup;
/// What the stop found,
/// and the read of one member's threads with a bound that has already passed.
use crate::control_group_stop::report::{
    StopReport,
    standing_of_within,
};
/// The switches of a test build:
/// the report a stop answers at once with,
/// the member that has ended by the time its threads
/// are read,
/// and the clock the listing's bound is read from.
use crate::failpoints::{
    Point,
    arm,
    arm_listing_clock_after_checks,
};

/// What:
/// `fn the_refusal_of(report: &StopReport, group: &ControlGroup, reaped: &Reaped) -> Option<String>` is the decision
/// a run's end makes on one stop:
/// the refusal,
/// or `None` when the scope was shown empty.
/// Why:
/// Each control below judges that one decision,
/// and none of them should repeat how the sentence before it is built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theRefusalOf(report: StopReport, group: ControlGroup, reaped: Reaped): string | undefined
/// ```
fn the_refusal_of(
    report: &StopReport,
    group: &ControlGroup,
    reaped: &Reaped,
) -> Option<String> {
    // What:     `report.sentence(group.directory())` is the stop's own words: the wait, the kernel's word, and the listing.
    // Why:      The decision reads that sentence beside the report, so a control gives it the same pair a run gives it.
    let words: String = report.sentence(group.directory());
    return refusal_of_scope(report, reaped, &words);
}

/// What:
/// `fn a_program_in_a_scope() -> TestScope` starts a program that lives until it is killed,
/// in a fresh delegated scope.
/// Why:
/// A member the listing must name has to be in a scope the stop of this test can read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aProgramInAScope(): TestScope  // startInScope(new Command('sleep', ['600']))
/// ```
fn a_program_in_a_scope() -> TestScope {
    return start_in_scope(Command::new("sleep").arg("600"));
}

/// What:
/// `fn the_direct_child_has_ended(scope: &mut TestScope) -> Reaped` kills the program of a scope and waits for it,
/// as the run's own stop does before it reads the scope.
/// Why:
/// A program that is still alive is a member the listing names as one that can still write,
/// so a control of another leftover
/// must have it reaped first,
/// and the decision then really is about a child that ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theDirectChildHasEnded(scope: TestScope): Reaped  // kill, then reap
/// ```
fn the_direct_child_has_ended(scope: &mut TestScope) -> Reaped {
    let _ = scope.child.kill();
    return reap(&mut scope.child);
}

/// What:
/// `struct MovedMember { child: Child }` is a program this control started outside the scope and moved into it,
/// and it is killed and reaped when the control ends however it ends.
/// Why:
/// A member left behind would keep its scope populated after the control,
/// and the next run of the suite would meet it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MovedMember { child: ChildProcess }
/// ```
struct MovedMember {
    /// The program this control started.
    child: Child,
}

/// What:
/// `impl MovedMember` starts the program and names its process.
/// Why:
/// The member must be this control's own child,
/// so the control can say it is really the process it moved into the scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a class with a static constructor and a pid getter
/// ```
impl MovedMember {
    /// What:
    /// `fn started() -> MovedMember` starts a program that lives until it is killed.
    /// Why:
    /// The move needs a process that is alive at the moment its number is written into the scope.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static started(): MovedMember  // spawn('sleep', ['600'])
    /// ```
    fn started() -> MovedMember {
        return MovedMember {
            child: Command::new("sleep")
                .arg("600")
                .spawn()
                .expect("starts the program that is moved into the scope"),
        };
    }

    /// What:
    /// `fn pid(&self) -> i32` is this program's process number.
    /// Why:
    /// The move writes the number,
    /// and the assertions name it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get pid(): number { return this.child.pid }
    /// ```
    fn pid(&self) -> i32 {
        return i32::try_from(self.child.id()).expect("a process id that fits in an i32");
    }
}

/// What:
/// `impl Drop for MovedMember` kills the program and reaps it.
/// Why:
/// A control that fails halfway still leaves no member behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose](): void { this.child.kill(); this.child.wait(); }
/// ```
impl Drop for MovedMember {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// What:
/// `fn moved_into(member: &MovedMember, directory: &Path)` writes the member's number into a group's `cgroup.procs`,
/// so the member stands in that group.
/// Why:
/// A process is moved into a group under the same slice by writing its number there,
/// and the control waits until the kernel
/// lists it,
/// so no assertion below rests on a move that did not happen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movedInto(member: MovedMember, directory: string): void  // write, then wait until listed
/// ```
fn moved_into(
    member: &MovedMember,
    directory: &Path,
) {
    let procs: PathBuf = directory.join("cgroup.procs");
    // What:     `fs::write(&procs, format!("{}\n", member.pid()))` writes the number the kernel reads as a move.
    // Why:      The group then holds the member, and the listing names it.
    fs::write(&procs, format!("{}\n", member.pid()))
        .unwrap_or_else(|error| panic!("{} cannot be written ({error})", procs.display()));
    let wanted: i32 = member.pid();
    wait_until("the member this control moved into the group", &|| {
        return fs::read_to_string(&procs).is_ok_and(|text| {
            return text
                .lines()
                .any(|line| return line.trim().parse::<i32>() == Ok(wanted));
        });
    });
}

/// What:
/// `fn a_group_below_the_scope(group: &ControlGroup, name: &str) -> GroupBelowTheScope` makes a group of that name below
/// the scope and moves a member into it.
/// Why:
/// The listing walks the groups below the scope,
/// so a group below it that cannot be read,
/// a cut before that group's
/// members are read,
/// and the order two such groups are named in
/// are all cases a control must be able to make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aGroupBelowTheScope(group: ControlGroup, name: string): GroupBelowTheScope
/// ```
fn a_group_below_the_scope(
    group: &ControlGroup,
    name: &str,
) -> GroupBelowTheScope {
    // What:     `let below: PathBuf = group.directory().join(name)` is the group made below the scope.
    // Why:      `populated` counts a descendant group's threads, so a refusal's listing walks into it.
    let below: PathBuf = group.directory().join(name);
    fs::create_dir(&below)
        .unwrap_or_else(|error| panic!("{} cannot be made ({error})", below.display()));
    let member: MovedMember = MovedMember::started();
    moved_into(&member, &below);
    return GroupBelowTheScope {
        member,
        path: below,
    };
}

/// What:
/// `struct GroupBelowTheScope { member: MovedMember, path: PathBuf }` is a group below a scope,
/// the member in it,
/// and its
/// directory.
/// Why:
/// The group's mode is restored and the group removed when the control ends,
/// so the scope can be removed after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GroupBelowTheScope { member: MovedMember; path: string }
/// ```
struct GroupBelowTheScope {
    /// The member that stands in the group.
    member: MovedMember,
    /// The group's directory.
    path: PathBuf,
}

/// What:
/// `impl GroupBelowTheScope` denies and restores the read of the group's `cgroup.procs`.
/// Why:
/// A mode of `000` is what makes that read fail with `EACCES` on this hierarchy (measured 2026-11-27),
/// and the directory cannot be removed while that mode stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a class with denyTheRead() and a disposer
/// ```
impl GroupBelowTheScope {
    /// What:
    /// `fn deny_the_read(&self)` sets the group's mode to `000`,
    /// so a read of `cgroup.procs` in it fails.
    /// Why:
    /// The listing reads that file for every group below the scope,
    /// and this is the one failure a control can produce
    /// without a failpoint.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// denyTheRead(): void { chmod(this.path, 0o000) }
    /// ```
    fn deny_the_read(&self) {
        use std::os::unix::fs::PermissionsExt;
        // What:     `fs::Permissions::from_mode(0o000)` is a permission set with no bit for anybody.
        // Why:      A read of the group's `cgroup.procs` then answers `EACCES`, which is not a group that was removed.
        let denied: fs::Permissions = fs::Permissions::from_mode(0o000);
        fs::set_permissions(&self.path, denied)
            .unwrap_or_else(|error| panic!("{} cannot be denied ({error})", self.path.display()));
    }
}

/// What:
/// `impl Drop for GroupBelowTheScope` restores the group's mode and removes it.
/// Why:
/// A group that kept `000` would stop its scope from being removed,
/// and the next run would meet a loaded empty scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose](): void { chmod(this.path, 0o755); rmdir(this.path) }
/// ```
impl Drop for GroupBelowTheScope {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let restored: fs::Permissions = fs::Permissions::from_mode(0o755);
        let _ = fs::set_permissions(&self.path, restored);
        let _ = fs::remove_dir(&self.path);
    }
}

/// Release read of `0.1.5-alpha.12`,
/// Medium 2:
/// a member that ends after the listing read it is a `NoWriter` leftover,
/// and a run with such a leftover is refused.
/// The member is a real one in a real scope,
/// and the failpoint stands for the one outcome no test can hold a process in:
/// the kernel takes a process out of `cgroup.procs` and out of `populated` at its last thread's end,
/// so a member can only be read as one that has ended in the window between the read of the list and the read of its
/// threads (measured 2026-11-27:
/// a process that has exited and waits to be reaped is not in its scope's `cgroup.procs`,
/// and a scope of such a member reads `populated 0`).
#[test]
fn a_member_that_ended_after_it_was_listed_is_a_leftover_the_run_is_refused_on() {
    this_test_needs_a_delegated_scope();
    let mut scope: TestScope = a_program_in_a_scope();
    let member: MovedMember = MovedMember::started();
    moved_into(&member, scope.group.directory());
    let ended: Reaped = the_direct_child_has_ended(&mut scope);
    // The stop answers at once with a report of the scope as it really is, and the member it lists reads as one that has
    // ended by the time its threads are read.
    arm(Point::ContainmentStopStuck);
    arm(Point::ContainmentMemberEndedAtRead);

    let report: StopReport = scope
        .group
        .stop(QUIESCE_BOUND)
        .expect("the stop of a scope that exists answers a report");
    let refusal: String = the_refusal_of(&report, &scope.group, &ended)
        .expect("a scope whose listed member cannot write is still not one shown empty");

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopStuck)
            && !crate::failpoints::fires(Point::ContainmentMemberEndedAtRead),
        "the stop never reached the report and the read this control arms, so the refusal has another cause"
    );
    assert!(
        refusal.contains("no member it listed can still write"),
        "the refusal does not open by saying that no member it listed can still write: {refusal}"
    );
    assert!(
        refusal.contains(&member.pid().to_string()),
        "the refusal does not name the member it read ({}): {refusal}",
        member.pid()
    );
    assert!(
        refusal.contains("it has exited and only waits to be reaped, so it can write nothing"),
        "the refusal does not say what the member it read can do: {refusal}"
    );
}

/// Release read of `0.1.5-alpha.12`,
/// Medium 3:
/// a listing cut before it read one member refuses the run,
/// and says which bound cut it and that it read nothing.
/// The clock is this control's own (`arm_listing_clock_after_checks`),
/// so the bound is reached without a scope of thousands of threads,
/// and the member the scope holds is one the cut keeps the listing from reading.
#[test]
fn a_listing_cut_before_it_read_one_member_refuses_the_run_and_says_so() {
    this_test_needs_a_delegated_scope();
    let mut scope: TestScope = a_program_in_a_scope();
    let member: MovedMember = MovedMember::started();
    moved_into(&member, scope.group.directory());
    let ended: Reaped = the_direct_child_has_ended(&mut scope);
    arm(Point::ContainmentStopStuck);
    // The first check is the one at the top of a group, so a clock that lets no check pass cuts before its members are read.
    arm_listing_clock_after_checks(0);

    let report: StopReport = scope
        .group
        .stop(QUIESCE_BOUND)
        .expect("the stop of a scope that exists answers a report");
    let refusal: String = the_refusal_of(&report, &scope.group, &ended)
        .expect("a listing cut at its bound is not a scope shown empty");

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopStuck),
        "the stop never reached the listing this control cuts, so the refusal has another cause"
    );
    assert!(
        refusal.contains("its members could not all be read within"),
        "the refusal does not open by saying that the listing was cut at its bound: {refusal}"
    );
    assert!(
        refusal.contains("the listing of its members was cut at its own bound of")
            && refusal.contains("before one was read"),
        "the refusal does not say the listing was cut before it read one member: {refusal}"
    );
    assert!(
        !refusal.contains(&member.pid().to_string()),
        "the refusal names the member the cut kept it from reading ({}): {refusal}",
        member.pid()
    );
}

/// Release read of `0.1.5-alpha.12`,
/// Medium 3:
/// a listing cut after it read a member names that member,
/// says which bound cut the rest,
/// and the run is refused.
/// The member that stands in the scope's own group reads as one that has ended (the failpoint above),
/// so the leftover is the cut itself and not a member that can still write;
/// the member below the scope is the one the cut keeps the listing from reading.
#[test]
fn a_listing_cut_after_one_member_names_it_and_refuses_the_run() {
    this_test_needs_a_delegated_scope();
    let mut scope: TestScope = a_program_in_a_scope();
    let member: MovedMember = MovedMember::started();
    moved_into(&member, scope.group.directory());
    let below: GroupBelowTheScope = a_group_below_the_scope(&scope.group, "nested");
    let ended: Reaped = the_direct_child_has_ended(&mut scope);
    arm(Point::ContainmentStopStuck);
    arm(Point::ContainmentMemberEndedAtRead);
    // The first check is the top of the scope's own group and the second is before its member, so a clock that lets two
    // checks pass reads that member and cuts before the top of the group below the scope.
    arm_listing_clock_after_checks(2);

    let report: StopReport = scope
        .group
        .stop(QUIESCE_BOUND)
        .expect("the stop of a scope that exists answers a report");
    let refusal: String = the_refusal_of(&report, &scope.group, &ended)
        .expect("a listing cut at its bound is not a scope shown empty");

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopStuck)
            && !crate::failpoints::fires(Point::ContainmentMemberEndedAtRead),
        "the stop never reached the listing and the read this control arms, so the refusal has another cause"
    );
    assert!(
        refusal.contains("after it listed 1 member")
            && refusal.contains(&member.pid().to_string())
            && refusal.contains("cut at its own bound"),
        "the refusal does not name the member it read before the cut ({}): {refusal}",
        member.pid()
    );
    assert!(
        !refusal.contains(&below.member.pid().to_string()),
        "the refusal names the member the cut kept it from reading ({}): {refusal}",
        below.member.pid()
    );
}

/// Release read of `0.1.5-alpha.12`,
/// Low 4:
/// a group below the scope that cannot be read keeps the members the listing already read,
/// so a live writer it found stands in the refusal and is named.
/// The group's `cgroup.procs` is denied with a mode,
/// so the read really fails (`EACCES`),
/// and no failpoint is needed for it.
#[test]
fn a_group_below_the_scope_that_cannot_be_read_keeps_the_members_already_read() {
    this_test_needs_a_delegated_scope();
    let mut scope: TestScope = a_program_in_a_scope();
    let member: MovedMember = MovedMember::started();
    moved_into(&member, scope.group.directory());
    let below: GroupBelowTheScope = a_group_below_the_scope(&scope.group, "nested");
    below.deny_the_read();
    let ended: Reaped = the_direct_child_has_ended(&mut scope);
    arm(Point::ContainmentStopStuck);

    let report: StopReport = scope
        .group
        .stop(QUIESCE_BOUND)
        .expect("the stop of a scope that exists answers a report");
    let refusal: String = the_refusal_of(&report, &scope.group, &ended)
        .expect("a group below the scope that could not be read is not a scope shown empty");

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopStuck),
        "the stop never reached the listing this control denies a group to, so the refusal has another cause"
    );
    assert!(
        refusal.contains("a member of the scope could not be shown stopped"),
        "the refusal does not open by naming the member it did read: {refusal}"
    );
    assert!(
        refusal.contains(&member.pid().to_string())
            && refusal.contains("it is alive, so it can still write"),
        "the refusal does not name the live member of the scope's own group ({}): {refusal}",
        member.pid()
    );
    assert!(
        refusal.contains("its members could not all be read"),
        "the refusal does not say that the rest could not be read: {refusal}"
    );
    assert!(
        !refusal.contains(&below.member.pid().to_string()),
        "the refusal names a member of the group it could not read ({}): {refusal}",
        below.member.pid()
    );
}

/// Release read of `0.1.5-alpha.12`,
/// Medium 3:
/// a member whose threads are not all read within the bound is never read as gone,
/// and the words say which bound it hit.
/// The deadline is one this control passes in,
/// so no read has to stall in the kernel.
#[test]
fn a_member_whose_threads_were_not_all_read_within_the_bound_is_never_read_as_gone() {
    let mut program: Child = Command::new("sleep")
        .arg("600")
        .spawn()
        .expect("starts the program whose threads are read");
    let member: i32 = i32::try_from(program.id()).expect("a process id that fits in an i32");
    // What:     `Instant::now()` is a point the clock is already at or past, so the first thread's read is past its bound.
    // Why:      The member really is alive, and the read must report that it could not be shown gone rather than "gone".
    let already_past: Instant = Instant::now();

    let standing = standing_of_within(member, already_past);
    let _ = program.kill();
    let _ = program.wait();

    assert!(
        standing.can_still_write(),
        "a member whose threads were not all read was read as one that can write nothing: {standing:?}"
    );
    assert!(
        standing.text().contains("could not all be read within"),
        "the words do not say which bound the read of its threads hit: {}",
        standing.text()
    );
}

/// Release read of `0.1.5-alpha.13`,
/// Low "The refusal's listing":
/// a listing that read no member of any group while the kernel's last word said the scope was populated is a `Nothing`
/// leftover,
/// and the run is refused on it.
/// The refusal says the listing found no member,
/// and says nothing about what a member it never read can do.
/// The member leaves between that word and the listing,
/// which nothing outside a test build can hold a process in:
/// a member that leaves before the `populated` read makes the scope read unpopulated,
/// and the stop then reports the scope empty at once instead of refusing.
#[test]
fn a_listing_that_read_no_member_while_the_kernel_said_populated_refuses_the_run() {
    this_test_needs_a_delegated_scope();
    let mut scope: TestScope = a_program_in_a_scope();
    let member: MovedMember = MovedMember::started();
    moved_into(&member, scope.group.directory());
    let ended: Reaped = the_direct_child_has_ended(&mut scope);
    // The stop answers at once with the report of a scope that really holds the member, and the listing reads none of it.
    arm(Point::ContainmentStopStuck);
    arm(Point::ContainmentListingFindsNoMember);

    let report: StopReport = scope
        .group
        .stop(QUIESCE_BOUND)
        .expect("the stop of a scope that exists answers a report");
    let refusal: String = the_refusal_of(&report, &scope.group, &ended)
        .expect("a scope the kernel reported populated is not one shown empty, even when the listing read no member");

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopStuck)
            && !crate::failpoints::fires(Point::ContainmentListingFindsNoMember),
        "the stop never reached the report and the listing this control arms, so the refusal has another cause"
    );
    assert!(
        matches!(
            report.leftover(),
            crate::control_group_stop::report::Leftover::Nothing
        ),
        "the report was not read as a listing that found no member: {refusal}"
    );
    assert!(
        refusal.contains("and it listed no member of the scope or of any group below it"),
        "the refusal does not open by saying that it listed no member: {refusal}"
    );
    assert!(
        !refusal.contains(&member.pid().to_string()),
        "the refusal names the member the listing did not read ({}): {refusal}",
        member.pid()
    );
}

/// Release read of `0.1.5-alpha.13`,
/// Low "The refusal's listing":
/// the listing names the scope's own group first,
/// then each group below it with the greatest name first,
/// and the processes of one group ascending by process number,
/// which is the order `Listing` states.
/// The numbers are compared before the order is judged:
/// the group below the scope whose name is read last holds the member started first,
/// so a listing in process number order would name these four the other way round.
#[test]
fn the_listing_names_the_scope_s_own_group_first_and_the_groups_below_it_greatest_name_first() {
    this_test_needs_a_delegated_scope();
    let mut scope: TestScope = a_program_in_a_scope();
    let at_home: MovedMember = MovedMember::started();
    moved_into(&at_home, scope.group.directory());
    let alpha: GroupBelowTheScope = a_group_below_the_scope(&scope.group, "alpha");
    let beta: GroupBelowTheScope = a_group_below_the_scope(&scope.group, "beta");
    let second_in_beta: MovedMember = MovedMember::started();
    moved_into(&second_in_beta, &beta.path);
    let ended: Reaped = the_direct_child_has_ended(&mut scope);
    arm(Point::ContainmentStopStuck);

    let report: StopReport = scope
        .group
        .stop(QUIESCE_BOUND)
        .expect("the stop of a scope that exists answers a report");
    let refusal: String = the_refusal_of(&report, &scope.group, &ended)
        .expect("a scope whose listed members are alive is not one shown empty");

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopStuck),
        "the stop never reached the report this control arms, so the refusal has another cause"
    );
    assert!(
        at_home.pid() < alpha.member.pid()
            && alpha.member.pid() < beta.member.pid()
            && beta.member.pid() < second_in_beta.pid(),
        "this host handed these numbers out in another order than the members were started in, so this control cannot \
         tell one listing order from the other: {} {} {} {}",
        at_home.pid(),
        alpha.member.pid(),
        beta.member.pid(),
        second_in_beta.pid()
    );
    // Each needle is the member's own words, so a number inside another number or inside the scope's path cannot answer for it.
    let at_home_at: usize = refusal
        .find(&format!("{} (", at_home.pid()))
        .unwrap_or_else(|| {
            panic!("the refusal does not name the member of the scope's own group: {refusal}")
        });
    let in_beta_at: usize = refusal
        .find(&format!("{} in beta (", beta.member.pid()))
        .unwrap_or_else(|| panic!("the refusal does not name the member in beta: {refusal}"));
    let second_in_beta_at: usize = refusal
        .find(&format!("{} in beta (", second_in_beta.pid()))
        .unwrap_or_else(|| {
            panic!("the refusal does not name the second member in beta: {refusal}")
        });
    let in_alpha_at: usize = refusal
        .find(&format!("{} in alpha (", alpha.member.pid()))
        .unwrap_or_else(|| panic!("the refusal does not name the member in alpha: {refusal}"));

    assert!(
        at_home_at < in_beta_at
            && in_beta_at < second_in_beta_at
            && second_in_beta_at < in_alpha_at,
        "the refusal does not name the scope's own group first, then the groups below it with the greatest name first, \
         and one group's members ascending: it names {} at {at_home_at}, {} at {in_beta_at}, {} at {second_in_beta_at}, {} at \
         {in_alpha_at}: {refusal}",
        at_home.pid(),
        beta.member.pid(),
        second_in_beta.pid(),
        alpha.member.pid()
    );
}

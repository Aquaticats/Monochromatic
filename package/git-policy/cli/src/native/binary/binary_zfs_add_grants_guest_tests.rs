//! What:
//!  Plan step 10's grant split as guest arms:
//!  one arm per decided permission withheld,
//!  refusing before the claim with nothing written for the six the add's own
//!  steps use,
//!  and printing the `zfs allow` line and finishing the add for the three only a
//!  removal uses,
//!  one arm on the whole set printing no line at all,
//!  and one arm under the delegated account showing that the pool's own
//!  answers for the add's requests are the same with each of the three taken
//!  away.
//! Why:
//!  The split's ground is a claim about which requests the add makes
//!  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
//!  choice 3C,
//!  "An arm in the guest,
//!  with the six granted and one of the three withheld,
//!  should confirm that the add's own requests never fail for want of the
//!  withheld one").
//!  The add's own command runs as the fixture's account,
//!  whose nine grants the pool hands the account that runs it,
//!  so withholding one asks the preflight the question a person's machine asks.
//!  The kernel's side is a second arm:
//!  root bypasses every delegation,
//!  so only the delegated account can say whether a withheld permission would
//!  have stopped a request,
//!  and that arm runs the add's four writing requests as `ubuntu` through the
//!  `zfs` tool.
//!  A run without `CLI_GIT_ZFS_ADD_GUEST` prints the skip line its parent
//!  module's arms print and returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const grant of DECIDED) { const added = await runAdd(grant); expect(added).toEqual(verdictOf(grant)) }
//! for (const grant of A_REMOVALS_GROUP) { expect(await asUbuntu().writes(grant)).toEqual('all succeed') }
//! ```

use std::path::PathBuf;
use std::process::{Command, Output};

use super::{Fixture, guest};

/// The account the design delegates the route to, which the guest's image creates.
const ACCOUNT_NAME: &str = "ubuntu";

/// That account's user id,
/// which every `zfs allow` and every finding names.
const ACCOUNT: u32 = 1000;

/// Where the design grants one decided permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Place {
    /// The checkout dataset.
    Checkout,
    /// Every descendant of the checkout dataset.
    CheckoutDescendants,
    /// The clone container itself.
    Container,
    /// Every descendant of the clone container,
    /// which is every clone.
    ContainerDescendants,
}

/// What the add does when one decided permission is missing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Verdict {
    /// Called by a step of this add, so the preflight refuses before the claim.
    Refuses,
    /// Called only by a removal, so the add prints the `zfs allow` line and goes on.
    Warns,
}

/// One decided permission the design scopes,
/// with the `zfs allow` scope that grants it and what the add does without it.
struct DecidedGrant {
    /// Where the design grants it.
    place: Place,
    /// The `zfs allow` scope: `-l` for the dataset, `-d` for its descendants.
    scope: &'static str,
    /// The kernel's permission name.
    permission: &'static str,
    /// What the add does without it.
    verdict: Verdict,
}

/// The nine permissions the design scopes
/// (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
/// "Scoped delegation"),
/// split by choice 3C:
/// the six the add's own steps use,
/// and the three only a removal uses while no step of the add destroys.
const DECIDED: [DecidedGrant; 9] = [
    DecidedGrant { place: Place::Checkout, scope: "-l", permission: "snapshot", verdict: Verdict::Refuses },
    DecidedGrant { place: Place::CheckoutDescendants, scope: "-d", permission: "clone", verdict: Verdict::Refuses },
    DecidedGrant { place: Place::CheckoutDescendants, scope: "-d", permission: "mount", verdict: Verdict::Warns },
    DecidedGrant { place: Place::CheckoutDescendants, scope: "-d", permission: "destroy", verdict: Verdict::Warns },
    DecidedGrant { place: Place::Container, scope: "-l", permission: "create", verdict: Verdict::Refuses },
    DecidedGrant { place: Place::Container, scope: "-l", permission: "mount", verdict: Verdict::Refuses },
    DecidedGrant { place: Place::ContainerDescendants, scope: "-d", permission: "mount", verdict: Verdict::Refuses },
    DecidedGrant { place: Place::ContainerDescendants, scope: "-d", permission: "userprop", verdict: Verdict::Refuses },
    DecidedGrant { place: Place::ContainerDescendants, scope: "-d", permission: "destroy", verdict: Verdict::Warns },
];

/// The dataset one place names.
fn dataset_of(fixture: &Fixture, place: Place) -> String {
    return match place {
        Place::Checkout | Place::CheckoutDescendants => fixture.checkout(),
        Place::Container | Place::ContainerDescendants => fixture.container(),
    };
}

/// The slot one arm runs at,
/// which names the permission and the scope so a failure reads at once.
fn arm_slot(grant: &DecidedGrant) -> String {
    return format!("grants-{}-{}", grant.permission, grant.scope.trim_start_matches('-'));
}

/// The `zfs` arguments that grant one decided permission where the design puts it.
fn allow_arguments(grant: &DecidedGrant, fixture: &Fixture) -> Vec<String> {
    return vec![String::from("allow"), String::from(grant.scope), String::from("-u"), ACCOUNT.to_string(), String::from(grant.permission), dataset_of(fixture, grant.place)];
}

/// The `zfs` arguments that take that grant away.
fn unallow_arguments(grant: &DecidedGrant, fixture: &Fixture) -> Vec<String> {
    return vec![String::from("unallow"), String::from(grant.scope), String::from("-u"), ACCOUNT.to_string(), String::from(grant.permission), dataset_of(fixture, grant.place)];
}

/// One grant of the decided nine handed to uid 0,
/// which is the account this guest's add runs as,
/// and put back when the arm ends.
struct Withheld<'a> {
    /// The fixture whose delegation is short one permission.
    fixture: &'a Fixture,
    /// The permission this arm is about.
    grant: &'a DecidedGrant,
}

impl<'a> Withheld<'a> {
    /// Take the one permission away from the account the add runs as.
    fn from_the_fixture(fixture: &'a Fixture, grant: &'a DecidedGrant) -> Withheld<'a> {
        let mut taken: Vec<String> = unallow_arguments(grant, fixture);
        taken[3] = String::from("0");
        let said: Output = fixture.zfs(taken.iter().map(|argument| return argument.as_str()).collect::<Vec<&str>>().as_slice());
        assert!(said.status.success(), "`{}` cannot be withheld: {}", grant.permission, String::from_utf8_lossy(said.stderr.as_slice()));
        return Withheld { fixture, grant };
    }
}

impl Drop for Withheld<'_> {
    fn drop(&mut self) {
        let mut given: Vec<String> = allow_arguments(self.grant, self.fixture);
        given[3] = String::from("0");
        let said: Output = self.fixture.zfs(given.iter().map(|argument| return argument.as_str()).collect::<Vec<&str>>().as_slice());
        assert!(said.status.success(), "`{}` cannot be granted back: {}", self.grant.permission, String::from_utf8_lossy(said.stderr.as_slice()));
    }
}

/// Clear one arm's slot before it runs,
/// and leave the world alone when the arm never made one.
/// Why:
///  The arms run at names of their own,
///  so an untouched slot is the usual case and `git worktree remove` on it
///  writes a `fatal` line into the log for nothing.
fn clear_a_fresh_slot(fixture: &Fixture, name: &str) {
    if fixture.slot(name).exists() || fixture.dataset_stands(fixture.clone(name).as_str()) {
        fixture.clear_the_slot(name);
    }
}

/// Every snapshot the checkout holds,
/// so a refusal arm can prove the add took none.
fn snapshots_of_the_checkout(fixture: &Fixture) -> String {
    let listed: Output = fixture.zfs(&["list", "-H", "-o", "name", "-t", "snapshot", fixture.checkout().as_str()]);
    return String::from_utf8_lossy(listed.stdout.as_slice()).into_owned();
}

/// What:
///  A missing permission of the add's own refuses before the claim,
///  names the permission and the exact `zfs allow`,
///  and writes nothing;
///  a missing permission only a removal uses prints that command,
///  says the add will finish,
///  and finishes.
/// Why:
///  This is choice 3C at the boundary a person meets:
///  the pool enforces the delegation the fixture gives,
///  the account is the one that runs the add,
///  and the finding names that account.
#[test]
fn every_withheld_permission_refuses_or_warns_as_the_record_says() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    for grant in &DECIDED {
        let name: String = arm_slot(grant);
        clear_a_fresh_slot(&fixture, name.as_str());
        let withheld: Withheld<'_> = Withheld::from_the_fixture(&fixture, grant);
        let output: Output = fixture.run_add(name.as_str());
        let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
        let dataset: String = dataset_of(&fixture, grant.place);
        let allow: String = format!("zfs allow {} -u 0 {} {}", grant.scope, grant.permission, dataset);
        match grant.verdict {
            Verdict::Refuses => {
                assert_eq!(output.status.code(), Some(git_policy_cli::subvolume_command::REFUSED_EXIT_CODE), "a missing `{}` on `{dataset}` must refuse the add: {said}", grant.permission);
                assert!(said.contains(&format!("`{}`", grant.permission)), "the refusal names the permission: {said}");
                assert!(said.contains(allow.as_str()), "the refusal names `{allow}`: {said}");
                assert!(said.contains("the add changed nothing"), "the refusal says it changed nothing: {said}");
                // Nothing written: no clone, no slot, no registration, no origin
                // snapshot and no record of this slot.
                assert!(!fixture.dataset_stands(fixture.clone(name.as_str()).as_str()), "the refused add made no clone: {said}");
                assert!(!fixture.slot(name.as_str()).exists(), "the refused add made no slot folder: {said}");
                assert!(!fixture.common().join("worktrees").join(name.as_str()).exists(), "the refused add wrote no registration: {said}");
                assert!(!fixture.home().join(format!("{name}.binding")).exists(), "the refused add wrote no binding: {said}");
                assert!(!snapshots_of_the_checkout(&fixture).contains("git-worktree-clone-origin-"), "the refused add took no origin snapshot: {said}");
                assert!(!said.contains("made the worktree as a ZFS clone"), "the refused add took no clone route: {said}");
            }
            Verdict::Warns => {
                assert!(output.status.success(), "a missing `{}` on `{dataset}` must not stop the add: {said}", grant.permission);
                assert!(said.contains("this add will finish"), "the warning says the add goes on: {said}");
                assert!(said.contains(allow.as_str()), "the warning names `{allow}`: {said}");
                assert!(said.contains("made the worktree as a ZFS clone"), "the add went on and made the clone: {said}");
                assert!(fixture.dataset_stands(fixture.clone(name.as_str()).as_str()), "the warned add made the clone: {said}");
                assert_eq!(std::fs::read_to_string(fixture.slot(name.as_str()).join("plain.txt")).expect("the filled worktree's file"), "plain\n", "the warned add filled the worktree");
            }
        }
        drop(withheld);
        fixture.clear_the_slot(name.as_str());
    }
}

/// What:
///  The whole decided delegation prints no line and finishes the add.
/// Why:
///  The warning must be about the missing grant and not about the arm:
///  an add under the nine prints neither the warning nor a `zfs allow`.
#[test]
fn the_whole_delegation_prints_no_line() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    let name: &str = "grants-all";
    clear_a_fresh_slot(&fixture, name);
    let output: Output = fixture.run_add(name);
    let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(output.status.success(), "the add under the whole delegation failed: {said}");
    assert!(!said.contains("this add will finish"), "a whole delegation earns no warning: {said}");
    assert!(!said.contains("zfs allow"), "a whole delegation earns no command: {said}");
    assert!(said.contains("made the worktree as a ZFS clone"), "the add made the clone: {said}");
    fixture.clear_the_slot(name);
}

//region The kernel's side, under the delegated account

/// One `zfs` command run as the delegated account,
/// with the pool's own binary and its own library.
fn zfs_as_the_account(fixture: &Fixture, arguments: &[&str]) -> Output {
    let mut command: Command = Command::new("/usr/sbin/runuser");
    command.args(["-u", ACCOUNT_NAME, "--", "/usr/bin/env"]);
    command.arg(format!("PATH={}:/usr/bin:/bin", fixture.sbin));
    command.arg(format!("LD_LIBRARY_PATH={}/lib", fixture.sbin.trim_end_matches("/sbin")));
    command.arg(format!("{}/zfs", fixture.sbin));
    command.args(arguments);
    return command.output().expect("the account's zfs starts");
}

/// Grant one decided permission to the delegated account.
fn grant_to_the_account(fixture: &Fixture, grant: &DecidedGrant) {
    let arguments: Vec<String> = allow_arguments(grant, fixture);
    let said: Output = fixture.zfs(arguments.iter().map(|argument| return argument.as_str()).collect::<Vec<&str>>().as_slice());
    assert!(said.status.success(), "`{}` cannot be granted to {ACCOUNT_NAME}: {}", grant.permission, String::from_utf8_lossy(said.stderr.as_slice()));
}

/// Take one decided permission away from the delegated account.
fn withhold_from_the_account(fixture: &Fixture, grant: &DecidedGrant) {
    let arguments: Vec<String> = unallow_arguments(grant, fixture);
    let said: Output = fixture.zfs(arguments.iter().map(|argument| return argument.as_str()).collect::<Vec<&str>>().as_slice());
    assert!(said.status.success(), "`{}` cannot be withheld from {ACCOUNT_NAME}: {}", grant.permission, String::from_utf8_lossy(said.stderr.as_slice()));
}

/// Where the account tries to mount a checkout snapshot,
/// which a removal's origin cleanup does and no step of the add does.
const ACCOUNT_MOUNT_POINT: &str = "/home/ubuntu/origin-mount";

/// Mount one checkout snapshot as the delegated account,
/// which needs `mount` on the checkout's descendants.
fn mount_a_snapshot_as_the_account(_fixture: &Fixture, snapshot: &str) -> Output {
    let folder: PathBuf = PathBuf::from(ACCOUNT_MOUNT_POINT);
    std::fs::create_dir_all(folder.as_path()).expect("the mount point");
    let handed: Output = Command::new("/usr/bin/chown").args(["--recursive", format!("{ACCOUNT_NAME}:{ACCOUNT_NAME}").as_str()]).arg(folder.as_path()).output().expect("chown starts");
    assert!(handed.status.success(), "the mount point cannot be handed to {ACCOUNT_NAME}: {}", String::from_utf8_lossy(handed.stderr.as_slice()));
    let mut command: Command = Command::new("/usr/sbin/runuser");
    command.args(["-u", ACCOUNT_NAME, "--", "/usr/bin/mount"]).args(["-t", "zfs", snapshot, ACCOUNT_MOUNT_POINT]);
    return command.output().expect("the account's mount starts");
}

/// One of the three the add never needs,
/// with the request that permission belongs to.
struct RemovalGrant {
    /// The permission.
    grant: DecidedGrant,
    /// What a removal does with it.
    what: &'static str,
}

/// What:
///  With each permission only a removal uses withheld,
///  every request the add makes succeeds for the delegated account except the
///  clone's own `mountpoint` property,
///  which is refused with the whole delegation too,
///  and the request that permission belongs to is refused.
/// Why:
///  Root bypasses every delegation,
///  so the add's own completion as root says nothing about a withheld
///  permission;
///  this arm runs the add's requests as the account the design delegates to,
///  one permission at a time.
///  The clone's `mountpoint=legacy` is not one of them:
///  `zfs clone -o mountpoint=legacy` needs the `mountpoint` delegation,
///  which the design forbids the account,
///  so the arm runs the propertyless clone the delegation carries as well and
///  states the refusal separately rather than folding it into a grant's verdict.
#[test]
fn the_adds_writes_succeed_for_the_account_with_each_removal_permission_withheld() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    for grant in DECIDED {
        grant_to_the_account(&fixture, &grant);
    }
    let removal_grants: [RemovalGrant; 3] = [
        RemovalGrant { grant: DecidedGrant { place: Place::ContainerDescendants, scope: "-d", permission: "destroy", verdict: Verdict::Warns }, what: "destroy a clone" },
        RemovalGrant { grant: DecidedGrant { place: Place::CheckoutDescendants, scope: "-d", permission: "mount", verdict: Verdict::Warns }, what: "mount a checkout snapshot for origin cleanup" },
        RemovalGrant { grant: DecidedGrant { place: Place::CheckoutDescendants, scope: "-d", permission: "destroy", verdict: Verdict::Warns }, what: "destroy an origin snapshot" },
    ];
    let counter: u32 = std::process::id();
    for (index, withheld) in removal_grants.iter().enumerate() {
        withhold_from_the_account(&fixture, &withheld.grant);
        let snapshot: String = format!("{}@grants-probe-{counter}-{index}", fixture.checkout());
        let bare: String = format!("{}/grants-probe-bare-{counter}-{index}", fixture.container());
        let snapshot_taken: Output = zfs_as_the_account(&fixture, &["snapshot", snapshot.as_str()]);
        assert!(snapshot_taken.status.success(), "the add's snapshot needs no `{}`: {}", withheld.grant.permission, String::from_utf8_lossy(snapshot_taken.stderr.as_slice()));
        let clone_made: Output = zfs_as_the_account(&fixture, &["clone", snapshot.as_str(), bare.as_str()]);
        assert!(clone_made.status.success(), "the add's clone needs no `{}`: {}", withheld.grant.permission, String::from_utf8_lossy(clone_made.stderr.as_slice()));
        let property_written: Output = zfs_as_the_account(&fixture, &["set", "org.monochromatic:git-worktree-clone-mount-desired-state=v1:00112233445566778899aabbccddeeff:mount", bare.as_str()]);
        assert!(property_written.status.success(), "the add's mount request needs no `{}`: {}", withheld.grant.permission, String::from_utf8_lossy(property_written.stderr.as_slice()));
        let identity_read: Output = zfs_as_the_account(&fixture, &["list", "-H", "-o", "name", bare.as_str()]);
        assert!(identity_read.status.success(), "the add's identity read needs no `{}`: {}", withheld.grant.permission, String::from_utf8_lossy(identity_read.stderr.as_slice()));
        // The request the withheld permission belongs to is refused.
        let refused: Output = match (withheld.grant.place, withheld.grant.permission) {
            (Place::CheckoutDescendants, "mount") => mount_a_snapshot_as_the_account(&fixture, snapshot.as_str()),
            (Place::CheckoutDescendants, "destroy") => zfs_as_the_account(&fixture, &["destroy", snapshot.as_str()]),
            _destroy => zfs_as_the_account(&fixture, &["destroy", bare.as_str()]),
        };
        assert!(!refused.status.success(), "a withheld `{}`, which a removal uses to {}, still let that request through", withheld.grant.permission, withheld.what);
        // Root puts the probe back, whatever the account could not do.
        let _destroyed: Output = fixture.zfs(&["destroy", bare.as_str()]);
        let _snapshot_destroyed: Output = fixture.zfs(&["destroy", snapshot.as_str()]);
        let _unmounted: Output = Command::new("/usr/bin/umount").arg(ACCOUNT_MOUNT_POINT).output().expect("umount starts");
        grant_to_the_account(&fixture, &withheld.grant);
    }
    // What:
    //  The add's own clone request, with the `mountpoint` the add writes, is
    //  refused for the account whatever is withheld,
    //  and its propertyless half is not.
    // Why:
    //  `mountpoint` is a delegation the design forbids,
    //  and the add asks for it at clone time,
    //  so this arm separates that refusal from the split's: it answers the same
    //  way with the whole nine granted.
    let snapshot: String = format!("{}@grants-probe-{counter}-property", fixture.checkout());
    let with_the_property: String = format!("{}/grants-probe-with-property-{counter}", fixture.container());
    let snapshot_taken: Output = zfs_as_the_account(&fixture, &["snapshot", snapshot.as_str()]);
    assert!(snapshot_taken.status.success(), "the control's snapshot: {}", String::from_utf8_lossy(snapshot_taken.stderr.as_slice()));
    let property_refused: Output = zfs_as_the_account(&fixture, &["clone", "-o", "mountpoint=legacy", snapshot.as_str(), with_the_property.as_str()]);
    let _partial: Output = fixture.zfs(&["destroy", "-r", "-f", with_the_property.as_str()]);
    let _snapshot_destroyed: Output = fixture.zfs(&["destroy", snapshot.as_str()]);
    assert!(
        !property_refused.status.success(),
        "the control: `zfs clone -o mountpoint=legacy` must be refused for the account under the nine, because `mountpoint` is a delegation the design forbids, and it was not: {}",
        String::from_utf8_lossy(property_refused.stdout.as_slice())
    );
    // The control that the harness can see a missing grant at all.
    let snapshot_grant: DecidedGrant = DecidedGrant { place: Place::Checkout, scope: "-l", permission: "snapshot", verdict: Verdict::Refuses };
    withhold_from_the_account(&fixture, &snapshot_grant);
    let snapshot_refused: Output = zfs_as_the_account(&fixture, &["snapshot", format!("{}@grants-probe-{counter}-control", fixture.checkout()).as_str()]);
    grant_to_the_account(&fixture, &snapshot_grant);
    assert!(!snapshot_refused.status.success(), "the positive control: with `snapshot` withheld the account's snapshot must be refused, and it was not");
    for grant in DECIDED {
        withhold_from_the_account(&fixture, &grant);
    }
}

//endregion The kernel's side, under the delegated account

//! What:
//!  Plan step 10's grant split as guest arms, run as the delegated account:
//!  one arm per decided permission withheld,
//!  refusing before the claim with nothing written for the six the add's own
//!  steps use,
//!  and printing the `zfs allow` line and finishing the add for the three only a
//!  removal uses,
//!  and one arm on the whole set printing no line at all.
//! Why:
//!  The split's ground is a claim about which requests the add makes,
//!  and only a run under the account the delegation names can answer it:
//!  as root every pool request succeeds whatever the delegation says,
//!  so a root run shows the preflight's sentence and nothing about the
//!  permissions behind it
//!  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
//!  choice 3C,
//!  "An arm in the guest,
//!  with the six granted and one of the three withheld,
//!  should confirm that the add's own requests never fail for want of the
//!  withheld one").
//!  Every arm borrows the fixture the add's root arms use,
//!  hands the checkout and the nine grants to the account,
//!  drops the one permission its arm is about,
//!  and puts both back before the next arm,
//!  so a failure names one permission rather than the world it left behind.
//!  A run without `CLI_GIT_ZFS_ADD_GUEST` prints the skip line its parent
//!  module's arms print and returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const grant of DECIDED) { const added = await runAddAsTheAccount(grant); expect(added).toEqual(verdictOf(grant)) }
//! ```

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::{Fixture, POLL, WAIT, WRAPPER, guest};

/// The account the design delegates the route to, which the guest's image creates.
const ACCOUNT_NAME: &str = "ubuntu";

/// That account's user id,
/// which every `zfs allow` and every finding names.
const ACCOUNT: u32 = 1000;

/// That account's home, where its own links, records and runtime folder live.
const ACCOUNT_HOME: &str = "/home/ubuntu";

/// The folder the account starts the wrapper from,
/// which is outside the checkout so the source stays clean.
const ACCOUNT_BIN: &str = "/home/ubuntu/zfs-add-bin";

/// The session runtime folder the account owns.
const ACCOUNT_RUNTIME: &str = "/home/ubuntu/zfs-add-runtime";

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
/// which names the permission so a failure reads at once.
fn arm_slot(grant: &DecidedGrant) -> String {
    return format!("grants-{}-{}", grant.permission, if grant.scope == "-l" { "local" } else { "descendants" });
}

/// Hand the fixture's checkout to the delegated account,
/// and put it back when the arm ends.
/// Why:
///  The route is the unprivileged lifecycle's:
///  the preflight compares the slot folder's owner with the account's own uid,
///  and every step writes into the checkout as the account,
///  so the arm must run as that account over that account's folder.
struct Lend<'a> {
    /// The fixture lent.
    fixture: &'a Fixture,
    /// The folder handed over,
    /// which goes back to root when the arm ends.
    checkout_root: PathBuf,
}

impl<'a> Lend<'a> {
    /// Grant the nine to the account and hand it the checkout,
    /// its worktrees folder and a home to run from.
    fn to_the_account(fixture: &'a Fixture) -> Lend<'a> {
        for grant in DECIDED {
            fixture.zfs_ok(&["allow", grant.scope, "-u", ACCOUNT.to_string().as_str(), grant.permission, dataset_of(fixture, grant.place).as_str()]);
        }
        let folder: PathBuf = PathBuf::from(ACCOUNT_BIN);
        std::fs::create_dir_all(folder.as_path()).expect("the account's bin folder");
        let link: PathBuf = folder.join("git");
        let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the account's git link");
        let runtime: PathBuf = PathBuf::from(ACCOUNT_RUNTIME);
        std::fs::create_dir_all(runtime.as_path()).expect("the account's runtime folder");
        let children: [&Path; 3] = [folder.as_path(), runtime.as_path(), Path::new(ACCOUNT_HOME)];
        for child in children {
            let handed: Output = Command::new("/usr/bin/chown").args(["--recursive", format!("{ACCOUNT_NAME}:{ACCOUNT_NAME}").as_str()]).arg(child).output().expect("chown starts");
            assert!(handed.status.success(), "{} cannot be handed to {ACCOUNT_NAME}: {}", child.display(), String::from_utf8_lossy(handed.stderr.as_slice()));
        }
        let mode: Output = Command::new("/usr/bin/chmod").args(["0700"]).arg(runtime.as_path()).output().expect("chmod starts");
        assert!(mode.status.success(), "the account's runtime folder takes mode 0700: {}", String::from_utf8_lossy(mode.stderr.as_slice()));
        let handed: Output = Command::new("/usr/bin/chown").args(["--recursive", format!("{ACCOUNT_NAME}:{ACCOUNT_NAME}").as_str()]).arg(fixture.root.as_path()).output().expect("chown starts");
        assert!(handed.status.success(), "the checkout cannot be handed to {ACCOUNT_NAME}: {}", String::from_utf8_lossy(handed.stderr.as_slice()));
        return Lend { fixture, checkout_root: fixture.root.clone() };
    }
}

impl Drop for Lend<'_> {
    fn drop(&mut self) {
        // The checkout goes back to root first, so the arm that runs next reads
        // the world the root arms left, and a `git` root starts there is not
        // refused as a repository somebody else owns.
        let handed: Output = Command::new("/usr/bin/chown").args(["--recursive", "root:root"]).arg(self.checkout_root.as_path()).output().expect("chown starts");
        assert!(handed.status.success(), "the checkout cannot be handed back to root: {}", String::from_utf8_lossy(handed.stderr.as_slice()));
        for grant in DECIDED {
            let said: Output = self.fixture.zfs(&["unallow", grant.scope, "-u", ACCOUNT.to_string().as_str(), grant.permission, dataset_of(self.fixture, grant.place).as_str()]);
            assert!(said.status.success(), "the account's `{}` grant on {} cannot be taken back: {}", grant.permission, dataset_of(self.fixture, grant.place), String::from_utf8_lossy(said.stderr.as_slice()));
        }
    }
}

/// One permission taken away for the length of one arm,
/// and put back when it ends.
struct Withheld<'a> {
    /// The fixture whose delegation is short one permission.
    fixture: &'a Fixture,
    /// The permission this arm is about.
    grant: &'a DecidedGrant,
}

impl<'a> Withheld<'a> {
    /// Take the one permission away from the account.
    fn from_the_account(fixture: &'a Fixture, grant: &'a DecidedGrant) -> Withheld<'a> {
        let said: Output = fixture.zfs(&["unallow", grant.scope, "-u", ACCOUNT.to_string().as_str(), grant.permission, dataset_of(fixture, grant.place).as_str()]);
        assert!(said.status.success(), "`{}` cannot be withheld: {}", grant.permission, String::from_utf8_lossy(said.stderr.as_slice()));
        return Withheld { fixture, grant };
    }
}

impl Drop for Withheld<'_> {
    fn drop(&mut self) {
        let said: Output = self.fixture.zfs(&["allow", self.grant.scope, "-u", ACCOUNT.to_string().as_str(), self.grant.permission, dataset_of(self.fixture, self.grant.place).as_str()]);
        assert!(said.status.success(), "`{}` cannot be granted back: {}", self.grant.permission, String::from_utf8_lossy(said.stderr.as_slice()));
    }
}

/// The add's command, started as the delegated account through `runuser`.
fn add_command_as_the_account(fixture: &Fixture, name: &str) -> Command {
    let mut command: Command = Command::new("/usr/sbin/runuser");
    let slot: String = fixture.slot(name).display().to_string();
    command
        .args(["-u", ACCOUNT_NAME, "--", "/usr/bin/env"])
        .arg(format!("PATH={ACCOUNT_BIN}:/usr/bin:/bin"))
        .arg(format!("HOME={ACCOUNT_HOME}"))
        .arg(format!("XDG_RUNTIME_DIR={ACCOUNT_RUNTIME}"))
        .args(["GIT_CONFIG_NOSYSTEM=1", "GIT_CONFIG_GLOBAL=/nonexistent-global-config", "GIT_AUTHOR_NAME=Fixture", "GIT_AUTHOR_EMAIL=fixture@example.invalid", "GIT_COMMITTER_NAME=Fixture", "GIT_COMMITTER_EMAIL=fixture@example.invalid"])
        .arg(format!("{ACCOUNT_BIN}/git"))
        .args(["worktree", "add", slot.as_str()])
        .current_dir(fixture.root.as_path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    return command;
}

/// Run one add as the account to its end,
/// mounting the clone at the slot as root when it appears.
fn run_add_as_the_account(fixture: &Fixture, name: &str) -> Output {
    let mut child: std::process::Child = add_command_as_the_account(fixture, name).spawn().expect("the account's add starts");
    let clone: String = fixture.clone(name);
    let slot: PathBuf = fixture.slot(name);
    let started: std::time::Instant = std::time::Instant::now();
    while started.elapsed() < WAIT {
        if child.try_wait().expect("the add is asked how it is").is_some() {
            break;
        }
        if fixture.dataset_stands(clone.as_str()) && slot.is_dir() && !fixture.mounted_at(slot.as_path()) {
            let mounted: Output = Command::new("/usr/bin/mount").args(["-t", "zfs", "-o", "nosuid,nodev", clone.as_str(), slot.to_str().expect("a text path")]).output().expect("mount starts");
            assert!(mounted.status.success(), "the clone {clone} cannot be mounted at {}: {}", slot.display(), String::from_utf8_lossy(mounted.stderr.as_slice()));
        }
        std::thread::sleep(POLL);
    }
    return child.wait_with_output().expect("the account's add ends");
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
///  This is choice 3C at the boundary a person meets,
///  with the account the finding names and a withheld permission the pool
///  really enforces.
#[test]
fn every_withheld_permission_refuses_or_warns_as_the_record_says() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    for grant in &DECIDED {
        let name: String = arm_slot(grant);
        fixture.clear_the_slot(name.as_str());
        let lend: Lend<'_> = Lend::to_the_account(&fixture);
        let withheld: Withheld<'_> = Withheld::from_the_account(&fixture, grant);
        let output: Output = run_add_as_the_account(&fixture, name.as_str());
        let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
        let dataset: String = dataset_of(&fixture, grant.place);
        let allow: String = format!("zfs allow {} -u {ACCOUNT} {} {}", grant.scope, grant.permission, dataset);
        match grant.verdict {
            Verdict::Refuses => {
                assert_eq!(output.status.code(), Some(git_policy_cli::subvolume_command::REFUSED_EXIT_CODE), "a missing `{}` on `{dataset}` must refuse the add: {said}", grant.permission);
                assert!(said.contains(&format!("`{}`", grant.permission)), "the refusal names the permission: {said}");
                assert!(said.contains(allow.as_str()), "the refusal names `{allow}`: {said}");
                assert!(said.contains("the add changed nothing"), "the refusal says it changed nothing: {said}");
                // Nothing written: no clone, no slot, no registration, no
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
        drop(lend);
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
    fixture.clear_the_slot(name);
    let lend: Lend<'_> = Lend::to_the_account(&fixture);
    let output: Output = run_add_as_the_account(&fixture, name);
    let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(output.status.success(), "the add under the whole delegation failed: {said}");
    assert!(!said.contains("this add will finish"), "a whole delegation earns no warning: {said}");
    assert!(!said.contains("zfs allow"), "a whole delegation earns no command: {said}");
    assert!(said.contains("made the worktree as a ZFS clone"), "the add made the clone: {said}");
    drop(lend);
    fixture.clear_the_slot(name);
}

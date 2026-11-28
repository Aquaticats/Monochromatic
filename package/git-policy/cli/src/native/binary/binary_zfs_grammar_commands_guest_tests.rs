//! What:
//!  The two management words of the ZFS route through the built wrapper,
//!  against a real file-backed pool in the throwaway guest:
//!  the cancel command's refusals and its reversal of a removal a kill left,
//!  and the settling command's refusals and its two settlements.
//! Why:
//!  Both commands read `/dev/zfs` and this process's mount table,
//!  so no control on a person's machine may run them against a pool it did not
//!  build,
//!  and the words are the grammar's,
//!  so only a run through the wrapper shows what a person is told and what status
//!  they read
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 17").
//!  The states are the ones the two commands' own arms make:
//!  a removal the entry wrote and a kill left standing,
//!  a clone that stands,
//!  a clone a person destroyed outside the wrapper,
//!  and a slot a checkout has begun in.
//!  The handler answers the reversing request on its own sweep,
//!  which the arm starts as the wrapper waits for the receipt,
//!  so the mount the cancellation asks for is made by the shipped handler and
//!  not by a script.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = wrapperGit(checkout, ['monochromatic-git-policy-cli', 'cancel-worktree-removal', SLOT]);
//! expect(said.code).toBe(0); expect(said.stderr).toContain('cancelled the removal of');
//! ```

use super::zfs_slot_state_guest_fixture::{
    Place,
    publish,
};
use super::zfs_sweep_guest::{
    NONCE,
    Run,
    clone_the_checkouts_snapshot,
    command_ok,
    make_the_designs_layout,
    mount_request,
    prepare_the_store,
    required,
    sweep,
    unmount_at,
    write_configuration,
    write_passing_probe_record,
    zfs,
    zfs_ok,
};
use git_policy_cli::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use git_policy_cli::worktree_clone_binding_record::binding_words::Phase;
use git_policy_cli::worktree_clone_binding_record::registration_home::plans_folder;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use git_policy_cli::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use git_policy_cli::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use git_policy_cli::worktree_clone_removal_proof_and_plan::removal_clock::TheLiveClock;
use git_policy_cli::worktree_clone_removal_proof_and_plan::removal_entry::{
    RemovalEntry,
    enter_the_removal,
};
use git_policy_cli::worktree_clone_removal_proof_and_plan::removal_reads::LiveRemovalReads;
use git_policy_cli::worktree_clone_removal_proof_and_plan::unmount_request::{
    RemovalUnmount,
    UnmountAnswer,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
};

/// The variable naming the guest this arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_GRAMMAR_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_GRAMMAR_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_GRAMMAR_SBIN";
/// The variable naming the checkout the design's layout mounts.
const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_GRAMMAR_CHECKOUT";

/// The slot both commands act on.
const SLOT: &str = "fix-flaky-test";
/// The clone's own snapshot,
/// which the kill's removal planned and the arm destroys.
const OWN_SNAPSHOT: &str = "sanoid-hourly-2026-11-27_03";

/// The wrapper program the driver built,
/// which the arm reaches through a `git` link so it runs as the person's `git`.
const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// A process id no process can have on this kernel,
/// which is the owner a plan of a killed removal records.
const A_PID_NO_PROCESS_CAN_HAVE: i64 = 9_000_001;

/// Whether this run is inside the guest.
fn in_the_guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP worktree_clone_cancel_and_settle_commands::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One checkout path as the argument Git takes.
fn shown(path: &Path) -> &str {
    return path.to_str().expect("the guest's paths are UTF-8");
}

/// What:
///  The built wrapper started the way a person's shell starts it,
///  with no environment of this arm's own reaching it.
/// Why:
///  The two words are the grammar's,
///  so what a person reads and the status they get are what this arm proves,
///  and a wrapper that inherited a runtime directory could reach a service
///  manager the arm never meant to use.
fn the_wrapper() -> Command {
    let bin: PathBuf = PathBuf::from("/root/grammar-bin");
    std::fs::create_dir_all(bin.as_path()).expect("the wrapper's bin folder");
    let home: PathBuf = PathBuf::from("/root/grammar-home");
    std::fs::create_dir_all(home.as_path()).expect("the wrapper's home folder");
    let runtime: PathBuf = PathBuf::from("/root/grammar-runtime");
    std::fs::create_dir_all(runtime.as_path()).expect("the wrapper's runtime folder");
    let link: PathBuf = bin.join("git");
    let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
    std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's git link");
    let mut command: Command = Command::new(link.as_path());
    command
        .env_clear()
        .env(
            "PATH",
            format!(
                "{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                bin.display()
            ),
        )
        .env("HOME", home.as_path())
        .env("XDG_RUNTIME_DIR", runtime.as_path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config");
    return command;
}

/// One command's answer from the built wrapper,
/// both streams read.
fn said(
    checkout: &Path,
    arguments: &[&str],
) -> Output {
    return the_wrapper()
        .args(arguments)
        .current_dir(checkout)
        .output()
        .unwrap_or_else(|error| panic!("the wrapper `{arguments:?}` could not start: {error}"));
}

/// One answer's status,
/// which the words are read beside.
fn code_of(output: &Output) -> i32 {
    return output.status.code().unwrap_or_else(|| return -1);
}

/// One answer's two streams as text.
fn text_of(output: &Output) -> String {
    return format!(
        "{}{}",
        String::from_utf8_lossy(output.stdout.as_slice()),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}

/// What:
///  The fixture the design's layout gives both commands:
///  a real repository whose slot Git registered,
///  the shape a sweep attaches,
///  and the add's records naming the clone.
/// Why:
///  Both commands read a slot below `.claude/worktrees`,
///  the records under the common directory and the pool,
///  and the state they act on is the add's own output rather than a record this
///  arm wrote.
struct TheFixture {
    /// The checkout the design's layout mounts.
    checkout: PathBuf,
    /// The OpenZFS prefix's `sbin`.
    sbin: String,
    /// The clone's dataset.
    clone: String,
    /// The slot's place in the record grammar.
    place: Place,
}

impl TheFixture {
    /// What:
    ///  Build the layout,
    ///  the registration and the clone,
    ///  attach the clone at its slot,
    ///  and publish the add's record of it.
    /// Why:
    ///  The mount and the record are what both commands prove,
    ///  and the slot's underlay identity is read before the mount because the
    ///  record holds it.
    fn build(
        checkout: PathBuf,
        pool: String,
        sbin: String,
    ) -> TheFixture {
        // One arm's fixtures are torn down before the next arm's, so a run that repeats an arm never meets it.
        let _previous: std::io::Result<()> = std::fs::remove_dir_all(checkout.as_path());
        std::fs::create_dir_all(checkout.as_path()).expect("the checkout folder is made");
        let (checkout_dataset, container): (String, String) =
            make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
        let worktrees: PathBuf = checkout.join(".claude").join("worktrees");
        std::fs::create_dir_all(worktrees.as_path()).expect("the slot parent is made");
        command_ok("git", &["init", "--quiet", shown(checkout.as_path())]);
        command_ok(
            "git",
            &[
                "-c",
                "user.name=the arm",
                "-c",
                "user.email=arm@example.invalid",
                "-C",
                shown(checkout.as_path()),
                "commit",
                "--quiet",
                "--allow-empty",
                "--message",
                "the arm's base",
            ],
        );
        let slot: PathBuf = worktrees.join(SLOT);
        command_ok(
            "git",
            &[
                "-C",
                shown(checkout.as_path()),
                "worktree",
                "add",
                "--quiet",
                "--no-checkout",
                shown(slot.as_path()),
                "HEAD",
            ],
        );
        let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path())
            .expect("the underlay's identity reads before the mount");
        let pointer: Vec<u8> =
            std::fs::read(slot.join(".git")).expect("the underlay pointer reads before the mount");
        write_passing_probe_record();
        prepare_the_store(checkout.as_path());
        write_configuration(checkout.as_path(), None, None);
        let clone: String = clone_the_checkouts_snapshot(
            sbin.as_str(),
            checkout_dataset.as_str(),
            container.as_str(),
            SLOT,
            NONCE,
            Some(mount_request(NONCE).as_str()),
        );
        zfs_ok(
            sbin.as_str(),
            &["snapshot", format!("{clone}@{OWN_SNAPSHOT}").as_str()],
        );
        let common: PathBuf = checkout.join(".git");
        let place: Place = Place {
            common: common.clone(),
            pool: pool.clone(),
            checkout_dataset: checkout_dataset.clone(),
            clone_dataset: clone.clone(),
            slot: slot.clone(),
            slot_identity,
            pointer,
            registration: common.join("worktrees").join(SLOT),
        };
        let attached: Run = sweep();
        assert_eq!(
            attached.status,
            Some(0),
            "the sweep attached the clone: {}",
            attached.said
        );
        publish(&place, Phase::Filled, sbin.as_str(), SLOT);
        return TheFixture {
            checkout,
            sbin,
            clone,
            place,
        };
    }

    /// The clone's GUID,
    /// read from the pool the record answers with.
    fn guid(&self) -> DatasetGuid {
        return super::zfs_slot_state_guest_fixture::guid_of(
            self.sbin.as_str(),
            self.clone.as_str(),
        );
    }
}

/// The unmount answer the entry is given,
/// so the arm judges the words and never the mount work.
struct ADoneUnmount {
    /// The clone's GUID.
    clone_guid: DatasetGuid,
}

impl RemovalUnmount for ADoneUnmount {
    fn request_the_unmount(
        &self,
        intent: &MountIntent,
    ) -> Result<
        UnmountAnswer,
        git_policy_cli::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop,
    > {
        return Ok(UnmountAnswer {
            dataset: self.clone_guid,
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            outcome: ReceiptOutcome::Done,
            mount_id: None,
            failed_step: None,
            errno: None,
        });
    }
}

/// What:
///  The state a removal the wrapper was killed in leaves:
///  the entry's own plan,
///  its `unmount` request,
///  the clone unmounted,
///  and a plan whose owner no longer runs.
/// Why:
///  A live owner is a running removal,
///  which the cancel command refuses by design,
///  and a pid above `pid_max` cannot be alive,
///  so the record is the same one the entry wrote with a dead owner.
fn the_state_a_kill_leaves(fixture: &TheFixture) {
    let guid: DatasetGuid = fixture.guid();
    let entry: RemovalEntry<'_> = RemovalEntry {
        common_dir: fixture.place.common.as_path(),
        slot: SLOT,
    };
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let unmount: ADoneUnmount = ADoneUnmount {
        clone_guid: guid,
    };
    enter_the_removal(&entry, &reads, &unmount, &TheLiveClock)
        .expect("the removal's own entry writes its plan");
    let mut plan: DestructionPlan =
        git_policy_cli::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan(
            fixture.place.common.as_path(),
            guid,
        )
        .expect("the entry's plan reads")
        .expect("the entry wrote a plan");
    plan.facts.owner = format!("{A_PID_NO_PROCESS_CAN_HAVE}:linux:0");
    let folder: PathBuf = plans_folder(fixture.place.common.as_path());
    std::fs::write(
        folder.join(guid.file_name()),
        plan.encode().expect("the plan encodes"),
    )
    .expect("the plan file");
    // The kill left the clone unmounted, because the entry's request was the last thing it wrote.
    unmount_at(fixture.place.slot.as_path());
    zfs_ok(
        fixture.sbin.as_str(),
        &[
            "destroy",
            format!("{}@{OWN_SNAPSHOT}", fixture.clone).as_str(),
        ],
    );
}

/// Whether the clone's request names an operation this arm did not write,
/// which is how it sees that the wrapper asked for the reversing mount.
fn the_request_changed(
    fixture: &TheFixture,
    was: &str,
) -> bool {
    let now: (bool, String, String) = zfs(
        fixture.sbin.as_str(),
        "zfs",
        &[
            "get",
            "-H",
            "-o",
            "value",
            DESIRED_STATE_PROPERTY,
            fixture.clone.as_str(),
        ],
        None,
    );
    return now.0 && now.1.trim() != was;
}

/// The clone's own request as text.
fn the_request(fixture: &TheFixture) -> String {
    let read: (bool, String, String) = zfs(
        fixture.sbin.as_str(),
        "zfs",
        &[
            "get",
            "-H",
            "-o",
            "value",
            DESIRED_STATE_PROPERTY,
            fixture.clone.as_str(),
        ],
        None,
    );
    return read.1.trim().to_string();
}

/// What:
///  That a slot no removal has touched is refused by the cancel word itself,
///  with the caller's status and a sentence naming the slot.
/// Why:
///  This is the answer the plan's step 17 owes while the removal's command path
///  is still cut:
///  the word is admitted,
///  it proves the records,
///  and it says there is no removal to cancel rather than printing a usage text.
#[test]
fn the_cancel_word_refuses_a_slot_no_removal_has_touched() {
    if !in_the_guest() {
        return;
    }
    let fixture: TheFixture = TheFixture::build(
        PathBuf::from(required(CHECKOUT_VARIABLE)),
        required(POOL_VARIABLE),
        required(SBIN_VARIABLE),
    );
    let answer: Output = said(
        fixture.checkout.as_path(),
        &[
            "monochromatic-git-policy-cli",
            "cancel-worktree-removal",
            "no-such-slot",
        ],
    );
    let text: String = text_of(&answer);
    assert_eq!(
        code_of(&answer),
        2,
        "a slot that is no removal is the caller's mistake: {text}"
    );
    assert!(
        text.contains("the slot `no-such-slot` is not being removed"),
        "{text}"
    );
    assert!(
        !text.contains("Usage: git monochromatic-git-policy-cli check"),
        "the word answers its own sentence rather than the namespace's usage: {text}"
    );
    println!("GRAMMAR cancel-untouched=refused status=2");
    println!("GRAMMAR done");
}

/// What:
///  That the settling word settles the two states its design gives it,
///  through the wrapper,
///  and refuses the clone that still stands.
/// Why:
///  The command removes evidence,
///  so a run that printed it removed nothing while the pool still held the
///  clone would be the worst answer it can give,
///  and the refusal beside the settlement shows the report is about the state
///  it read.
#[test]
fn the_settling_word_settles_a_slot_whose_clone_is_gone() {
    if !in_the_guest() {
        return;
    }
    let fixture: TheFixture = TheFixture::build(
        PathBuf::from(required(CHECKOUT_VARIABLE)),
        required(POOL_VARIABLE),
        required(SBIN_VARIABLE),
    );
    let guid: DatasetGuid = fixture.guid();
    let digits: String = {
        let whole: String = guid.file_name();
        String::from(&whole[whole.len() - 4..])
    };
    let the_words: [&str; 6] = [
        "monochromatic-git-policy-cli",
        "settle-worktree-clone-record",
        SLOT,
        "--confirm-removal-of",
        SLOT,
        "--confirm-clone-guid",
    ];
    let mut arguments: Vec<&str> = the_words.to_vec();
    arguments.push(digits.as_str());
    let stands: Output = said(fixture.checkout.as_path(), arguments.as_slice());
    let said_standing: String = text_of(&stands);
    assert_eq!(
        code_of(&stands),
        1,
        "a clone that answers the pool refuses the settlement: {said_standing}"
    );
    assert!(
        said_standing.contains("so its absence is not proved"),
        "{said_standing}"
    );
    assert!(
        told(&fixture, &fixture.clone),
        "nothing was removed while the clone stands"
    );
    println!("GRAMMAR settling-stands=refused status=1");
    // A person takes the mount down and then destroys the clone outside the wrapper,
    // which is the state the command exists for: the pool refuses to destroy a dataset a mount holds.
    unmount_at(fixture.place.slot.as_path());
    zfs_ok(
        fixture.sbin.as_str(),
        &["destroy", "-r", fixture.clone.as_str()],
    );
    let gone: Output = said(fixture.checkout.as_path(), arguments.as_slice());
    let said_gone: String = text_of(&gone);
    assert_eq!(code_of(&gone), 0, "a gone clone settles: {said_gone}");
    assert!(
        said_gone.contains("the clone's absence is proved"),
        "{said_gone}"
    );
    assert!(
        !told(&fixture, &fixture.clone),
        "the clone's name no longer answers the pool"
    );
    assert!(
        !fixture.place.registration.exists(),
        "the registration went with the records"
    );
    println!("GRAMMAR settling-gone=settled status=0");
    println!("GRAMMAR done");
}

/// Whether the pool still answers a dataset by this name.
fn told(
    fixture: &TheFixture,
    dataset: &str,
) -> bool {
    let asked: (bool, String, String) = zfs(
        fixture.sbin.as_str(),
        "zfs",
        &["list", "-H", "-o", "name", dataset],
        None,
    );
    return asked.0 && asked.1.trim() == dataset;
}

/// What:
///  That the cancel word reverses a removal a kill left,
///  through the wrapper,
///  as the shipped handler answers its request.
/// Why:
///  This is the manual way back the project kept the command for,
///  and only a run shows the report a person reads and the mount it asks for.
///  The handler's sweep runs while the wrapper waits for the receipt,
///  which is the order a person's own run has when the handler's timer fires.
#[test]
fn the_cancel_word_reverses_a_removal_a_kill_left() {
    if !in_the_guest() {
        return;
    }
    let fixture: TheFixture = TheFixture::build(
        PathBuf::from(required(CHECKOUT_VARIABLE)),
        required(POOL_VARIABLE),
        required(SBIN_VARIABLE),
    );
    the_state_a_kill_leaves(&fixture);
    let before: String = the_request(&fixture);
    assert!(
        before.ends_with(":unmount"),
        "the kill left the clone's own `unmount` request: {before}"
    );
    let waiting: std::process::Child = the_wrapper()
        .args([
            "monochromatic-git-policy-cli",
            "cancel-worktree-removal",
            SLOT,
        ])
        .current_dir(fixture.checkout.as_path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the wrapper starts");
    // The command writes its `mount` request and waits, so the handler's own sweep is what answers it.
    let mut answered: bool = false;
    for _look in 0..30 {
        if the_request_changed(&fixture, before.as_str()) {
            let ran: Run = sweep();
            assert_eq!(
                ran.status,
                Some(0),
                "the handler's sweep answered the request: {}",
                ran.said
            );
            answered = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    let output: Output = waiting
        .wait_with_output()
        .expect("the wrapper's answer is read");
    let text: String = text_of(&output);
    assert!(
        answered,
        "the wrapper wrote a request the handler answered: {text}"
    );
    assert_eq!(
        code_of(&output),
        0,
        "the cancellation reverses the removal: {text}"
    );
    assert!(
        text.contains("cancelled the removal of `fix-flaky-test`"),
        "{text}"
    );
    assert!(
        text.contains("do not come back"),
        "the report names the snapshot the removal destroyed: {text}"
    );
    assert!(
        super::zfs_sweep_guest::mount_row(fixture.place.slot.as_path()).is_some(),
        "the clone stands at its slot again"
    );
    println!("GRAMMAR cancel-killed-removal=reversed status=0");
    println!("GRAMMAR done");
}

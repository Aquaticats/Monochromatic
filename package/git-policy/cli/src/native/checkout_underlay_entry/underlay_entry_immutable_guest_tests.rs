//! What:
//!  The immutable covered entry's guest arm:
//!  the verdict is read against a real folder that carries the kernel's
//!  immutable flag on a real dataset,
//!  a real mount over the same path,
//!  and the account's own attempts to move and remove the flagged entry.
//! Why:
//!  the verdict's controls judge shapes a test hands it,
//!  and the project's measured facts (VM item 8) are the kernel's answers on a
//!  machine:
//!  the flag refused the account's rename and removal while the checkout was
//!  unmounted,
//!  a mount at the path answers another device than its parent,
//!  and a rollback to a snapshot taken before the flag loses it.
//!  So this arm takes the reading and the verdict together,
//! where the kernel
//!  answers,
//! and nowhere else (`MHX`).
//!  On any other machine it prints `SKIPPED` and returns.
//!  Its fixture folder is made clean before it runs:
//!  a run that stopped before its last line keeps the flag on the entry,
//!  and the driver that starts this arm clears it and takes down whatever mount
//!  was left,
//!  so the first reading is always a plain folder.
//!
//! The driver sets:
//!  `CLI_GIT_ZFS_STEP19_GUEST` to the guest's name,
//!  `CLI_GIT_ZFS_STEP19_UNDERLAY` to a folder on the guest's home dataset,
//!  which stands for the home underlay the flag is set on,
//!  and `CLI_GIT_ZFS_STEP19_UNDERLAY_DATASET` to a dataset the arm may mount at
//!  the checkout path and unmount again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! exec('chattr +i', underlayCheckout);
//! expect(verify(new LiveEntryReader().read(underlayCheckout), uid).isProtected()).toBe(true);
//! ```

use crate::checkout_underlay_entry::underlay_entry_read::{
    CoveredEntryFacts,
    EntryReader,
    LiveEntryReader,
};
use crate::checkout_underlay_entry::underlay_entry_verdict::{
    UnderlayVerdict,
    UnprotectedReason,
    verify,
};
use crate::test_support::spawn_owned;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
    Stdio,
};
use std::time::{
    Duration,
    Instant,
};

/// The variable that says this run is the step 19 guest.
pub const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_GUEST";

/// The folder that stands for the home underlay.
pub const UNDERLAY_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_UNDERLAY";

/// The program that sets and clears the kernel's immutable flag.
pub const CHATTR: &str = "/usr/bin/chattr";

/// The program that mounts and unmounts a dataset.
pub const MOUNT: &str = "/usr/bin/mount";

/// The program that unmounts a dataset.
pub const UMOUNT: &str = "/usr/bin/umount";

/// The program that changes an entry's owner.
pub const CHOWN: &str = "/usr/bin/chown";

/// How long one child may take.
/// Value:
///  120 s,
/// because each call here is a kernel call that answers at once,
/// and a
///  mount that waits for a device is what the bound exists for.
const COMMAND_DEADLINE: Duration = Duration::from_secs(120);

/// Run one command and answer its output,
/// ending the child that overruns the deadline.
fn run_bounded(
    program: &str,
    arguments: &[&str],
) -> Output {
    let mut command: Command = Command::new(program);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child: std::process::Child =
        spawn_owned(&mut command).unwrap_or_else(|error| panic!("{program} starts: {error}"));
    let began: Instant = Instant::now();
    loop {
        match child
            .try_wait()
            .unwrap_or_else(|error| panic!("{program} answers whether it ended: {error}"))
        {
            Some(_) => {
                return child
                    .wait_with_output()
                    .unwrap_or_else(|error| panic!("{program}'s output reads: {error}"));
            }
            None if began.elapsed() >= COMMAND_DEADLINE => {
                crate::child_ended_with_its_whole_group::end_with_its_group(child);
                panic!(
                    "{program} {} did not answer within {COMMAND_DEADLINE:?}",
                    arguments.join(" ")
                );
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// Run one command that must succeed,
/// and hand back what it said.
fn must(
    program: &str,
    arguments: &[&str],
) -> String {
    let output: Output = run_bounded(program, arguments);
    let mut said: String = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
    said.push_str(String::from_utf8_lossy(output.stderr.as_slice()).as_ref());
    assert!(
        output.status.success(),
        "`{program} {}` failed: {said}",
        arguments.join(" ")
    );
    return said;
}

/// The entry's path the arm protects:
/// the checkout's leaf name under the
/// underlay the driver named.
fn covered_entry() -> PathBuf {
    let underlay: String = std::env::var(UNDERLAY_VARIABLE)
        .unwrap_or_else(|_| panic!("{UNDERLAY_VARIABLE} names the guest's home underlay"));
    return Path::new(underlay.as_str()).join("monochromatic-git-checkout");
}

/// Read the entry and judge it for this test's own account.
fn read_verdict() -> (CoveredEntryFacts, UnderlayVerdict) {
    let entry: PathBuf = covered_entry();
    let facts: CoveredEntryFacts = LiveEntryReader
        .read(entry.as_path())
        .unwrap_or_else(|refusal| panic!("{}", refusal.message()));
    let wanted: u32 = rustix::process::geteuid().as_raw();
    let verdict: UnderlayVerdict = verify(&facts, wanted);
    return (facts, verdict);
}

/// Assert the reading is the verdict a person acts on,
/// and say what was compared.
fn expect_verdict(
    verdict: &UnderlayVerdict,
    facts: &CoveredEntryFacts,
    expected: &UnderlayVerdict,
    label: &str,
) {
    assert_eq!(
        verdict,
        expected,
        "`{label}` read {} and the verdict says {}",
        facts.device,
        verdict.message(facts)
    );
    println!(
        "STEP19-UNDERLAY {label} verdict={} same-device-as-parent={} immutable={} attributes-reported={} owner={}",
        match verdict {
            UnderlayVerdict::Protected => "protected",
            UnderlayVerdict::Unprotected(_) => "unprotected",
            UnderlayVerdict::Unproven(_) => "unproven",
        },
        facts.same_device_as_parent,
        facts.immutable,
        facts.attributes_reported,
        facts.owner
    );
}

#[test]
fn the_verdict_reads_a_real_immutable_folder_in_a_guest() {
    if std::env::var_os(GUEST_VARIABLE).is_none() {
        println!(
            "SKIPPED the immutable entry's guest arm: {GUEST_VARIABLE} is unset, so this is not the step 19 guest"
        );
        return;
    }
    let entry: PathBuf = covered_entry();
    std::fs::create_dir_all(entry.as_path()).expect("the covered entry is made");
    let plain: (CoveredEntryFacts, UnderlayVerdict) = read_verdict();
    assert!(
        plain.0.attributes_reported,
        "the home dataset reports the immutable attribute, which is the proof this check needs"
    );
    expect_verdict(
        &plain.1,
        &plain.0,
        &UnderlayVerdict::Unprotected(UnprotectedReason::ImmutableClear),
        "plain",
    );

    // The flag the setup sets, and the account's own attempts against it.
    let entry_text: String = entry.display().to_string();
    must(CHATTR, &["+i", entry_text.as_str()]);
    let flagged: (CoveredEntryFacts, UnderlayVerdict) = read_verdict();
    expect_verdict(
        &flagged.1,
        &flagged.0,
        &UnderlayVerdict::Protected,
        "flagged",
    );
    let removal: std::io::Error = std::fs::remove_dir(entry.as_path())
        .expect_err("an immutable folder refuses the account's removal");
    println!(
        "STEP19-UNDERLAY removal-refused={}",
        removal.raw_os_error().unwrap_or(0)
    );
    assert_eq!(
        removal.kind(),
        std::io::ErrorKind::PermissionDenied,
        "the kernel refuses it as a permission error: {removal}"
    );
    let moved: PathBuf = entry.with_file_name("monochromatic-git-checkout-moved");
    let rename: std::io::Error = std::fs::rename(entry.as_path(), moved.as_path())
        .expect_err("an immutable folder refuses the account's rename");
    println!(
        "STEP19-UNDERLAY rename-refused={}",
        rename.raw_os_error().unwrap_or(0)
    );
    assert!(
        !moved.exists(),
        "the refused rename left nothing behind: {}",
        moved.display()
    );

    // A mount at the path is another device, and the verdict names that rather
    // than believing the underlay's flag off the dataset.
    must(MOUNT, &["-t", "tmpfs", "tmpfs", entry_text.as_str()]);
    let mounted: (CoveredEntryFacts, UnderlayVerdict) = read_verdict();
    expect_verdict(
        &mounted.1,
        &mounted.0,
        &UnderlayVerdict::Unprotected(UnprotectedReason::MountedOver),
        "mounted",
    );
    must(UMOUNT, &[entry_text.as_str()]);
    let after_unmount: (CoveredEntryFacts, UnderlayVerdict) = read_verdict();
    expect_verdict(
        &after_unmount.1,
        &after_unmount.0,
        &UnderlayVerdict::Protected,
        "unmounted-again",
    );

    // The owner the route requires, and an entry that answers another one.
    // The flag is cleared first, because the kernel refuses `chown` on an
    // immutable entry with the same `EPERM` it refused the removal with.
    must(CHATTR, &["-i", entry_text.as_str()]);
    let wanted: u32 = rustix::process::geteuid().as_raw();
    let other: String = if wanted == 65534 {
        String::from("0")
    } else {
        String::from("65534")
    };
    must(CHOWN, &["--recursive", other.as_str(), entry_text.as_str()]);
    // The flag is set again, because the verdict answers a cleared flag before it
    // answers an owner,
    // and this case is about the owner alone.
    must(CHATTR, &["+i", entry_text.as_str()]);
    let owned: (CoveredEntryFacts, UnderlayVerdict) = read_verdict();
    expect_verdict(
        &owned.1,
        &owned.0,
        &UnderlayVerdict::Unprotected(UnprotectedReason::Owner {
            found: owned.0.owner,
            wanted,
        }),
        "other-owner",
    );
    must(
        CHOWN,
        &[
            "--recursive",
            wanted.to_string().as_str(),
            entry_text.as_str(),
        ],
    );

    // A rollback to a snapshot taken before the flag loses it, which is why the
    // check runs after every reimport, restore and rollback and not only at
    // installation.
    must(CHATTR, &["-i", entry_text.as_str()]);
    let rolled_back: (CoveredEntryFacts, UnderlayVerdict) = read_verdict();
    expect_verdict(
        &rolled_back.1,
        &rolled_back.0,
        &UnderlayVerdict::Unprotected(UnprotectedReason::ImmutableClear),
        "rolled-back",
    );
    println!("STEP19-UNDERLAY done");
}

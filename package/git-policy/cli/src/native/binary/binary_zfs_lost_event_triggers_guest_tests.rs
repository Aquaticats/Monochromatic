//! What:
//!  Plan step 19's lost-event cases in the throwaway guest:
//!  an event a ZED start replays,
//!  a dropped event with and without a live caller,
//!  zedlets run at once,
//!  a stale receipt,
//!  and a pending unmount across a restart.
//! Why:
//!  The handler side of the lost-event triggers is a ZED delivery,
//!  a filter that decides on it,
//!  and the merge device around one pass,
//!  so all of it has to meet a real ZED,
//!  a real kernel queue and a real `/run`
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "Enable real ZED in the guest and exercise the configured ordering",
//!  its "replay,
//!  dropped events with and without a live caller,
//!  concurrent zedlets,
//!  stale receipts,
//!  and pending unmount across a restart").
//!  No command reaches the filter or the device in this release,
//!  so the installed zedlet here is the arm's own:
//!  a link to this test binary whose one test runs per delivery,
//!  reads the delivery ZED handed it,
//!  decides it through the released filter and,
//!  when the event passes,
//!  runs the released merge device around a stand-in pass that files a real
//!  receipt.
//!  The stand-in is the pass's *work* alone:
//!  it reads the request the driver wrote on disk rather than the clone's intent
//!  property,
//!  because the sweep that reads that property is plan step 8's and is not this
//!  round's.
//!  Everything else is the product's own:
//!  the filter's rule and words,
//!  the device's folder,
//!  claim and mark,
//!  and the receipt writer.
//!  The cases run only where `CLI_GIT_ZFS_LOST_EVENTS_GUEST` names the guest,
//!  and every other machine reads `SKIP`,
//!  which is why no test here carries `#[ignore]`:
//! a harness that skipped an ignored test would run it nowhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the zedlet: one process per delivery
//! const decision = decideAboutTheEvent(EventIdentity.live(), parent); if (decision.passes) runAMergePass(folder, passThatFilesAReceipt);
//! // the arm, after the driver ran every case:
//! expect(deliveries.some((d) => d.decided === 'drop' && passesAfter === passesBefore)).toBe(true);
//! ```

use git_policy_cli::worktree_clone_mount_event_filter::{
    EventIdentity,
    ParentScope,
    decide_about_the_event,
};
use git_policy_cli::worktree_clone_mount_protocol::CloneMountReceipt;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use git_policy_cli::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use git_policy_cli::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use git_policy_cli::worktree_clone_mount_sweep_merge::SweepsFolder;
use git_policy_cli::worktree_clone_mount_sweep_merge::merge_pass::run_a_merge_pass;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// The variable that names the guest these cases run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_LOST_EVENTS_GUEST";

/// The folder the driver prepared every fixture in.
const FIXTURE: &str = "/opt/lost-events";

/// One line per delivery the installed zedlet took,
/// with the decision and the device's answer beside it.
const DELIVERY_LOG: &str = "delivery.log";

/// One line per pass the device ran,
/// which is what a drop must leave untouched.
const PASS_LOG: &str = "pass.log";

/// The one request file the stand-in pass answers,
/// as `nonce operation guid dataset`.
const REQUEST: &str = "request";

/// How long the stand-in pass works,
/// so two zedlets ZED forked at once are inside the device together.
/// Value:
///  a fixture's own length and no shipped limit:
///  the design's arm asks for a pass that lasts long enough to be caught,
///  and its measurement sizes the real one (M4).
const PASS_LASTS: Duration = Duration::from_millis(400);

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP binary::zfs_lost_event_triggers_guest: {GUEST_VARIABLE} is unset, so these cases run only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "these cases run only in the guest {name}");
    return true;
}

/// Append one line to a fixture log,
/// created when it is not there yet.
fn append(
    path: &Path,
    line: &str,
) {
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|error| {
            panic!(
                "{} could not be opened for this fixture: {error}",
                path.display()
            )
        });
    writeln!(file, "{line}").expect("a fixture log takes one line");
}

/// One fixture file's whole text,
/// trimmed,
/// or a panic naming the file.
fn fixture_file(name: &str) -> String {
    let path: PathBuf = Path::new(FIXTURE).join(name);
    return std::fs::read_to_string(path.as_path())
        .unwrap_or_else(|error| panic!("{} could not be read: {error}", path.display()))
        .trim()
        .to_string();
}

/// One fixture file's text,
/// or nothing when the driver wrote none.
fn maybe_fixture_file(name: &str) -> Option<String> {
    let path: PathBuf = Path::new(FIXTURE).join(name);
    return std::fs::read_to_string(path.as_path())
        .ok()
        .map(|text| return text.trim().to_string());
}

/// The lines of one fixture log,
/// and none when the driver has not made it yet.
fn log_lines(name: &str) -> Vec<String> {
    return maybe_fixture_file(name).map_or_else(Vec::new, |text| {
        return text
            .lines()
            .map(|line| return String::from(line))
            .collect::<Vec<String>>();
    });
}

/// The clone parent the driver's configuration names.
fn parent() -> ParentScope {
    return ParentScope::of_dataset(fixture_file("parent").as_str())
        .expect("the driver writes a parent dataset name");
}

/// The boot this guest runs,
/// which every receipt carries.
fn boot() -> String {
    return std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
}

/// The one request the stand-in pass answers,
/// as its nonce,
/// its operation,
/// its dataset and the clone's GUID.
fn request() -> (RequestNonce, MountOperation, String, DatasetGuid) {
    let text: String = fixture_file(REQUEST);
    let mut fields = text.split(' ');
    let nonce: &str = fields.next().expect("a request names a nonce");
    let operation: &str = fields.next().expect("a request names an operation");
    let dataset: &str = fields.next().expect("a request names a dataset");
    let guid: &str = fields.next().expect("a request names the clone's GUID");
    return (
        RequestNonce::parse(nonce).expect("the driver writes a nonce"),
        MountOperation::from_word(operation).expect("the driver writes an operation"),
        String::from(dataset),
        DatasetGuid::parse(guid).expect("the driver writes a GUID"),
    );
}

/// The one test the installed zedlet runs,
/// one process per delivery,
/// as ZED's own child.
#[test]
fn zfs_lost_event_zedlet_arm() {
    // This process is ZED's child, and ZED builds a zedlet's whole environment from the event and a fixed list, so no
    // variable of the driver's reaches it: the fixture folder the driver prepared is what proves this is its zedlet.
    let fixture: PathBuf = PathBuf::from(FIXTURE);
    if !fixture.join("parent").exists() {
        println!("SKIP binary::zfs_lost_event_zedlet_arm: no fixture at {FIXTURE}");
        return;
    }
    let delivery: EventIdentity = EventIdentity::live();
    let decided = decide_about_the_event(&delivery, Some(&parent()));
    append(
        fixture.join(DELIVERY_LOG).as_path(),
        format!(
            "start pid={} pool={} dataset={} decided={}",
            std::process::id(),
            or_absent(delivery.pool()),
            or_absent(delivery.dataset()),
            if decided.passes() { "pass" } else { "drop" }
        )
        .as_str(),
    );
    if !decided.passes() {
        append(
            fixture.join(DELIVERY_LOG).as_path(),
            format!("drop {}", decided.line()).as_str(),
        );
        append(fixture.join(DELIVERY_LOG).as_path(), "end");
        return;
    }
    // The pass the device runs is the arm's own body: it reads the request the driver wrote and files the product's
    // receipt. The sweep that reads the clone's intent property is plan step 8's and is not this round's.
    let answer: Result<String, String> = run_a_merge_pass(&SweepsFolder::live(), || {
        append(
            fixture.join(PASS_LOG).as_path(),
            format!("pass pid={}", std::process::id()).as_str(),
        );
        std::thread::sleep(PASS_LASTS);
        return file_a_receipt();
    });
    append(
        fixture.join(DELIVERY_LOG).as_path(),
        format!(
            "answer {}",
            match &answer {
                Ok(report) => report.as_str(),
                Err(refusal) => refusal.as_str(),
            }
            .replace('\n', " | ")
        )
        .as_str(),
    );
    append(fixture.join(DELIVERY_LOG).as_path(), "end");
}

/// One name,
/// or the word an absent one is written as in the log.
fn or_absent(name: Option<&str>) -> &str {
    return name.unwrap_or("absent");
}

/// What:
///  The work the stand-in pass does:
///  read the driver's request and file the product's receipt for it.
/// Why:
///   The cases need a receipt that answers the request's nonce,
///  and a stale receipt must be replaced rather than read as the answer,
///  so the receipt is written by the crate's own writer from the request the
///  driver wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [nonce, operation, dataset, guid] = request(); return publishReceipt({ dataset: guid, operation, nonce });
/// ```
fn file_a_receipt() -> Result<String, String> {
    let (nonce, operation, dataset, guid) = request();
    // The receipt carries the nonce by value, and the answer below names it again, so its text is kept first.
    let digits: String = String::from(nonce.as_str());
    let receipt = CloneMountReceipt {
        dataset: guid,
        operation,
        nonce,
        outcome: ReceiptOutcome::Done,
        mount: None,
        failed_step: None,
        errno: None,
        boot: boot(),
        time: git_policy_cli::iso_time::now_iso(),
    };
    ReceiptsFolder::live()
        .publish(&receipt)
        .map_err(|refusal| return format!("the receipt was not published: {refusal:?}"))?;
    return Ok(format!(
        "filed {} for {} on {}",
        digits,
        operation.word(),
        dataset
    ));
}

/// What:
///  Every case the driver ran,
///  judged from the deliveries ZED made and the passes the device ran.
/// Why:
///   A real ZED's own log is ZED's,
///  and the handler's lines reach nobody there,
///  so the evidence a person reads is the installed zedlet's own lines and the
///  receipts under `/run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(deliveries.filter((d) => d.decided === 'drop').length).toBe(2);
/// ```
#[test]
fn zfs_lost_event_triggers_guest() {
    if !guest() {
        return;
    }
    let delivered: Vec<String> = log_lines(DELIVERY_LOG);
    let passes: Vec<String> = log_lines(PASS_LOG);
    let delivered_text: String = delivered.join("\n");
    let start_lines: Vec<&String> = delivered
        .iter()
        .filter(|line| return line.starts_with("start "))
        .collect();
    println!(
        "MEASUREMENT lost-event-deliveries={} passes={}",
        start_lines.len(),
        passes.len()
    );

    // 1. An event a ZED start replayed was delivered and the device ran one pass for it.
    assert!(
        delivered_text.contains("decided=pass"),
        "a delivery the route's own scope accepts was decided as a pass:\n{delivered_text}"
    );
    assert!(
        !passes.is_empty(),
        "the device ran at least one pass for the deliveries it accepted:\n{delivered_text}"
    );
    assert!(
        maybe_fixture_file("count-replay").as_deref() == Some("answered"),
        "the replay's request was answered: the driver writes `count-replay` when the receipt named its nonce"
    );

    // 2. A dropped event ran no pass.
    let dropped: Vec<&String> = delivered
        .iter()
        .filter(|line| return line.starts_with("drop "))
        .collect();
    assert_eq!(
        dropped.len(),
        2,
        "one dropped event without a live caller and one with it:\n{delivered_text}"
    );
    for line in &dropped {
        assert!(
            line.contains("no pass ran") && line.contains("`/dev/zfs` was not opened"),
            "a drop says no pass ran and nothing was opened: {line}"
        );
    }
    assert_eq!(
        maybe_fixture_file("passes-around-dropped").as_deref(),
        Some("equal"),
        "the driver compared the pass count before and after each dropped event, and a drop added none"
    );

    // 3. A dropped event with a live caller left that caller waiting.
    assert_eq!(
        maybe_fixture_file("waiter").as_deref(),
        Some("no-receipt"),
        "a caller that waited through the dropped event got no receipt for it"
    );

    // 4. Zedlets ran at once, and the device let one pass run.
    let overlapping: usize = overlap_of(&delivered);
    assert!(
        overlapping >= 2,
        "two zedlet processes were alive together, which the start and end lines prove:\n{delivered_text}"
    );
    assert!(
        delivered_text.contains("holds the run claim") || delivered_text.contains("one more pass"),
        "a delivery that arrived while another pass ran was answered by the device, and not by a second pass of its \
         own:\n{delivered_text}"
    );

    // 5. A stale receipt was not the answer.
    assert_eq!(
        maybe_fixture_file("stale").as_deref(),
        Some("replaced"),
        "the driver read the receipt after a fresh request's event and found the fresh nonce, not the older one"
    );

    // 6. A pending unmount across a restart was answered.
    assert_eq!(
        maybe_fixture_file("pending-unmount").as_deref(),
        Some("answered"),
        "the driver found a receipt naming the unmount's own nonce after ZED was restarted"
    );
    println!("LOST-EVENTS every case held");
}

/// What:
///  The most zedlet processes the delivery log shows alive together.
/// Why:
///   ZED forks a zedlet per event and does not wait for it,
///  so two events it delivered together are two processes whose `start` lines
///  both precede their `end` lines;
///  counting the nesting is what proves they really ran at once rather than one
///  after the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let live = 0; let most = 0; for (const line of log) { if (line.startsWith('start ')) most = Math.max(most, ++live); if (line === 'end') --live; }
/// ```
fn overlap_of(delivered: &[String]) -> usize {
    let mut alive: usize = 0;
    let mut most: usize = 0;
    for line in delivered {
        if line.starts_with("start ") {
            alive += 1;
            most = most.max(alive);
            continue;
        }
        if line == "end" {
            alive = alive.saturating_sub(1);
        }
    }
    return most;
}

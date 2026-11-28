//! What:
//!  Controls for the progress file:
//!  its first stage and counter are published before the act reads anything,
//!  each stage advances the counter,
//!  the heartbeat moves it on its own and stops when the act ends,
//!  a file that answers another request,
//!  another mode or another owner is refused,
//!  and the give-up window is whole heartbeat periods.
//! Why:
//!  The wrapper's liveness answer rests on this file alone,
//!  and every rule it reads is a rule of the design
//!  ("The act's liveness"),
//!  so each is shown here with a heartbeat of milliseconds,
//!  which is a control's own value and never a shipped one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writing = file.begin(); await until(() => file.read().counter > first); writing.finish();
//! ```

use super::{
    ProgressFile,
    ProgressReading,
    ProgressRefusal,
    parse_progress,
    progress_text,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_forced_removal::forced_removal_words::Stage;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::time::{
    Duration,
    Instant,
};

/// The heartbeat a control gives its file,
/// which is short enough to watch move and is not the release's,
/// which is none.
const A_CONTROLS_HEARTBEAT: Duration = Duration::from_millis(20);

/// How long a control waits for the counter to move,
/// which is far longer than one period and still bounded.
const A_CONTROLS_DEADLINE: Duration = Duration::from_secs(10);

/// The dataset a control files its progress under.
fn a_dataset() -> DatasetGuid {
    return DatasetGuid::parse("18446744073709551615").expect("a canonical GUID");
}

/// The request's nonce,
/// as the intent writes it.
fn a_nonce() -> RequestNonce {
    return RequestNonce::parse("00112233445566778899aabbccddeeff").expect("a canonical nonce");
}

/// What:
///  Wait for one period without holding a thread's clock or spinning.
/// Why:
///  A control that busy-waited would be synthetic load,
///  and this is the same wait the heartbeat makes:
///  a channel with a timeout.
fn pause(period: Duration) {
    let (_sender, receiver) = std::sync::mpsc::channel::<()>();
    let _timed_out: Result<(), std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(period);
}

/// One file in a fixture folder this account owns,
/// which is the shape the live handler has with root.
fn a_file(folder: std::path::PathBuf) -> ProgressFile {
    use std::os::unix::fs::MetadataExt;
    let owner: u32 = std::fs::metadata(folder.as_path())
        .expect("the fixture folder's own facts")
        .uid();
    return ProgressFile::at(folder, owner, a_dataset(), a_nonce(), A_CONTROLS_HEARTBEAT);
}

#[test]
fn the_first_stage_and_its_counter_are_published_before_anything_is_read() {
    let folder: std::path::PathBuf = fixture("removal_progress_first");
    let file: ProgressFile = a_file(folder);
    let writing = file
        .begin()
        .expect("the fixture folder is the designed one");
    let first: ProgressReading = file
        .read()
        .expect("the file is there as soon as the act began");
    assert_eq!(first.stage, Stage::Preflight);
    assert_eq!(first.nonce, a_nonce().as_str());
    assert!(
        first.counter >= 1,
        "the first publish carries a counter, and it read {}",
        first.counter
    );
    assert_eq!(
        file.entry_name(),
        format!("{}{}", a_dataset().file_name(), ".removal-progress")
    );
    writing.finish().expect("the last stage is published");
}

#[test]
fn each_stage_advances_the_counter_and_is_read_back() {
    let folder: std::path::PathBuf = fixture("removal_progress_stages");
    let file: ProgressFile = a_file(folder);
    let writing = file
        .begin()
        .expect("the fixture folder is the designed one");
    let before: u64 = file.read().expect("the file is there").counter;
    writing
        .stage(Stage::Structural)
        .expect("a stage is published");
    let after: ProgressReading = file.read().expect("the file is still there");
    assert_eq!(after.stage, Stage::Structural);
    assert!(
        after.counter > before,
        "a stage advances the counter, and it read {} then {}",
        before,
        after.counter
    );
    writing.stage(Stage::Signals).expect("a stage is published");
    writing.stage(Stage::Unmount).expect("a stage is published");
    writing.finish().expect("the last stage is published");
    let ended: ProgressReading = file.read().expect("the file is still there after the act");
    assert_eq!(
        ended.stage,
        Stage::Done,
        "the act's end is its own stage word"
    );
}

#[test]
fn the_heartbeat_moves_the_counter_on_its_own_and_stops_when_the_act_ends() {
    let folder: std::path::PathBuf = fixture("removal_progress_heartbeat");
    let file: ProgressFile = a_file(folder);
    let writing = file
        .begin()
        .expect("the fixture folder is the designed one");
    let first: u64 = file.read().expect("the file is there").counter;
    let deadline: Instant = Instant::now() + A_CONTROLS_DEADLINE;
    let mut moved: u64 = first;
    while moved == first && Instant::now() < deadline {
        pause(A_CONTROLS_HEARTBEAT);
        moved = file.read().expect("the file is read again").counter;
    }
    assert!(
        moved > first,
        "the heartbeat advanced the counter within {A_CONTROLS_DEADLINE:?}, and it read {first} then {moved}"
    );
    writing.finish().expect("the last stage is published");
    let ended: ProgressReading = file.read().expect("the file after the act");
    assert_eq!(ended.stage, Stage::Done);
    pause(A_CONTROLS_HEARTBEAT * 5);
    let later: ProgressReading = file.read().expect("the file is read once more");
    assert_eq!(
        later.counter, ended.counter,
        "a counter that kept moving after the act ended would read as a live handler"
    );
}

#[test]
fn a_file_that_answers_another_request_is_refused() {
    let folder: std::path::PathBuf = fixture("removal_progress_other_nonce");
    let file: ProgressFile = a_file(folder.clone());
    let writing = file
        .begin()
        .expect("the fixture folder is the designed one");
    writing.finish().expect("the last stage is published");
    let other: RequestNonce =
        RequestNonce::parse("ffeeddccbbaa99887766554433221100").expect("a canonical nonce");
    use std::os::unix::fs::MetadataExt;
    let owner: u32 = std::fs::metadata(folder.as_path())
        .expect("the fixture folder's own facts")
        .uid();
    let stranger: ProgressFile =
        ProgressFile::at(folder, owner, a_dataset(), other, A_CONTROLS_HEARTBEAT);
    let refused: ProgressRefusal = stranger
        .read()
        .expect_err("a file for another request is never read as this one's");
    assert_eq!(refused.calling, "another-request");
    assert!(
        refused.said.contains("nonce"),
        "the refusal names the nonce, and it read {}",
        refused.said
    );
}

#[test]
fn a_mode_or_an_owner_a_person_changed_is_refused() {
    let folder: std::path::PathBuf = fixture("removal_progress_mode");
    let file: ProgressFile = a_file(folder.clone());
    let writing = file
        .begin()
        .expect("the fixture folder is the designed one");
    writing.finish().expect("the last stage is published");
    let entry: std::path::PathBuf = folder.join(file.entry_name());
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(entry.as_path(), std::fs::Permissions::from_mode(0o666))
        .expect("the control changes the mode");
    let refused: ProgressRefusal = file
        .read()
        .expect_err("a mode the route does not write is refused");
    assert_eq!(refused.calling, "read");
    assert!(
        refused.said.contains("mode"),
        "the refusal names the mode, and it read {}",
        refused.said
    );
}

#[test]
fn the_give_up_window_is_four_heartbeat_periods() {
    let folder: std::path::PathBuf = fixture("removal_progress_window");
    let file: ProgressFile = a_file(folder);
    assert!(
        !file.gives_up(Duration::from_millis(79)),
        "three periods and a half is not the window"
    );
    assert!(
        file.gives_up(A_CONTROLS_HEARTBEAT * 4),
        "four whole periods is the window"
    );
    assert!(file.gives_up(A_CONTROLS_HEARTBEAT * 5));
}

#[test]
fn a_text_line_this_build_does_not_write_is_refused() {
    let text: String = progress_text(a_dataset(), &a_nonce(), Stage::Unmount, 7);
    assert_eq!(
        parse_progress(text.as_str()).expect("a file this build wrote is one it reads"),
        (String::from(a_nonce().as_str()), Stage::Unmount, 7)
    );
    let broken: String = text.replace("stage unmount", "stage perhaps");
    assert!(
        parse_progress(broken.as_str()).is_err(),
        "a stage word this build does not write is refused"
    );
    let missing: String = text
        .lines()
        .filter(|line| return !line.starts_with("counter "))
        .collect::<Vec<&str>>()
        .join("\n");
    assert!(
        parse_progress(missing.as_str()).is_err(),
        "a file without its counter is refused"
    );
    assert!(
        parse_progress(
            text.replace("kind removal-progress", "kind something")
                .as_str()
        )
        .is_err(),
        "a kind this build does not write is refused"
    );
}

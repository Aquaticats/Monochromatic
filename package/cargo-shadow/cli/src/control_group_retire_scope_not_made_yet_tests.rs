//! What:
//! The one question this file settles:
//! what a run's last look at its scope answers when the scope's directory is not there at all,
//! because the launcher exited before the manager made the group.
//! Why:
//! The look reads the directory to tell a group the manager collected from a group that never existed,
//! and both answer "not there".
//! So a unit that is loaded while its directory was never made may be read as collected,
//! which asks the manager nothing and says nothing,
//! and the unit stays loaded with no member and no warning.
//! The control below makes that state and measures which answer the look gives,
//! because `README.md` claims that "a group that is gone was collected, and asks nothing either"
//! and the claim was read in the source, never observed at a run's end
//! (`doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md`,
//! "A directory that was never made reads as a collected scope").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the look at a scope whose directory was never made', () => { it('reads as collected', ...) })
//! ```

/// Writes the stand-in `systemctl` and reads its call log.
use std::fs;
/// Sets the stand-in's executable bit.
use std::os::unix::fs::PermissionsExt;
/// Borrows and owns the paths of the absent directory and of the stand-in.
use std::path::{
    Path,
    PathBuf,
};
/// Starts a real program in a real scope, and reads the manager's answer for its unit.
use std::process::Command;

/// The scope this control starts the way a run starts one,
/// the proof that this host can give it,
/// and the launcher's own handshake.
use crate::cgroup_test_support::{
    TestScope,
    start_in_scope,
    this_test_needs_a_delegated_scope,
};
/// The look at the scope and its bound.
use crate::control_group::retire::{
    RETIRE_BOUND,
    retire,
};
/// A directory of this control's own, beside the build output.
use crate::test_scratch::scratch;

/// What:
/// `fn recording_systemctl(directory: &Path) -> (String, PathBuf)` writes a program that appends its arguments to a log and
/// answers that it cannot stop anything,
/// and answers the program and the log.
/// Why:
/// The claim is that the look asks the manager nothing,
/// and that is read from a log that stays empty rather than from the manager's state.
/// The program is a stand-in and no claim about a real manager rests on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordingSystemctl(dir: string): [string, string]
/// ```
fn recording_systemctl(directory: &Path) -> (String, PathBuf) {
    let program: PathBuf = directory.join("recording-systemctl");
    let log: PathBuf = directory.join("systemctl-calls");
    // What:     `format!` builds the stand-in's text: it appends every call to the log and exits 1.
    // Why:      An empty log is the proof that no question was asked.
    let text: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nexit 1\n",
        log = log.display()
    );
    fs::write(&program, text).expect("write the stand-in");
    // What:     `fs::Permissions::from_mode(0o755)` is the mode a program must have to be started.
    // Why:      The look runs the program it is given, and a file without the executable bit cannot be started.
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return (program.display().to_string(), log);
}

/// What:
/// `fn calls(log: &Path) -> String` is what the recording program logged, or the empty text.
/// Why:
/// An empty log is the claim that no question was asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function calls(log: string): string { return readFileOrEmpty(log) }
/// ```
fn calls(log: &Path) -> String {
    return fs::read_to_string(log).unwrap_or_default();
}

/// What:
/// `fn active_state(unit: &str) -> String` is the manager's own `ActiveState` for a unit, trimmed.
/// Why:
/// The unit's state is the manager's fact, not a stand-in's,
/// and "left loaded" is about that state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function activeState(unit: string): string  // systemctl --user show <unit> --property=ActiveState --value
/// ```
fn active_state(unit: &str) -> String {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "--no-ask-password",
            "show",
            unit,
            "--property=ActiveState",
            "--value",
        ])
        .output()
        .expect("the real systemctl runs");
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// The control:
/// a scope unit that is loaded, read at a directory that was never made,
/// is a scope the manager collected:
/// the look answers no warning and asks the manager nothing.
/// The unit is a real one with a real member in it,
/// and the same look at that member's own directory says the unit was left loaded with a member,
/// which is what makes the absent directory the only thing the silent answer rests on.
#[test]
fn a_scope_whose_directory_was_never_made_is_read_as_collected_and_the_run_says_nothing() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let (systemctl, log) = recording_systemctl(directory.path());
    let mut scope: TestScope = start_in_scope(Command::new("sleep").arg("600"));
    let unit: String = scope.group.unit();
    // What:     `real_directory` is the scope's own directory, which the manager made when the launcher started the program.
    // Why:      It is put back after the look, because the scope's own stop needs it to find the member.
    let real_directory: PathBuf = scope.group.directory.clone();
    // What:     `never_made` is a path in the same slice that nothing made, which is the state of a scope whose launcher
    //           exited before the manager made its group.
    // Why:      The look reads the scope's directory to tell a collected group from one that is not there, and this is the
    //           second of the two.
    let never_made: PathBuf = scope.group.parent.join(format!(
        "cargo-shadow-test-{}-never-made.scope",
        std::process::id()
    ));
    assert!(
        !never_made.exists(),
        "the directory this control calls never made is there: {}",
        never_made.display()
    );

    // The positive control first: the same look at the member's own directory says the unit keeps its member, so a silent
    // answer at the absent one is not the look's only possible answer for this unit.
    let real_word: Option<String> = retire(&scope.group, &systemctl, RETIRE_BOUND);
    let real_said: String = real_word.expect("a scope with a member is reported as left loaded");
    assert!(
        real_said.contains("still has a member"),
        "the positive control did not report the member the scope holds: {real_said}"
    );

    // What:     `scope.group.directory = never_made.clone()` points the look at the directory that was never made.
    // Why:      The look reads `directory()` and nothing else about where the group is, so this is the state a launcher that
    //           exits before the manager makes the group leaves.
    scope.group.directory = never_made.clone();
    let word: Option<String> = retire(&scope.group, &systemctl, RETIRE_BOUND);
    scope.group.directory = real_directory;

    assert_eq!(
        word, None,
        "a scope whose directory was never made is reported as a unit that may be left loaded, so the look did not read it \
         as a collected scope: {word:?}"
    );
    assert_eq!(
        calls(&log),
        "",
        "the look asked the manager about a scope whose directory was never made"
    );
    assert_eq!(
        active_state(&unit),
        "active",
        "the unit of the scope is not loaded after the look, so this control cannot show what an absent directory leaves"
    );
}

//! What:
//!  The one command line of the root-run clone-mount handler:
//!  read the arguments as a mode,
//!  run that mode,
//!  write what it said,
//!  and answer with the process's exit status.
//! Why:
//!   Every mode ends in the same two answers to an operator or a unit:
//!  a line saying what happened,
//!  or a refusal saying why nothing happened,
//!  and the same exit status for each.
//!  A refusal is never a success:
//!  a boot unit's `ExecStart` that could not reconcile must fail,
//!  so a status of zero is written only after a mode did its work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(run(process.argv.slice(2))); // 0: done, 2: refused
//! ```

use super::adopt_worktree_parent_mode;
use super::attaching_sweep_mode::{self, SweepPlaces};
use super::boot_reconcile_mode::{self, ReconcilePlaces};
use super::handler_mode_arguments::{self, MODE_WORDS, HandlerMode};
use super::mount_path_descendant_mode;
use super::record_checkout_mode;
use super::root_mode_names::RootModeMisuse;
use super::holder_scan_mode;
use super::root_mode_caller::NotRootRefusal;
use super::shutdown_unmount_mode;
use crate::diagnostic_lines_off_the_calling_thread::write;
use super::handler_install_paths::CONFIGURATION_PATH;
use crate::start_probe::{FingerprintPaths, ProbeMode, RecordPlace, skip_after_gate_refusal};
use crate::worktree_clone_mount_protocol::ReadinessFile;
use crate::zfs_compatibility_gate::gate_verdict::Verdict;
use crate::zfs_compatibility_gate::{BeforeRequests, LiveGate, verdict_line};
use rustix::process::geteuid;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

/// The status of an invocation that did what it was asked.
pub const EXIT_DONE: i32 = 0;

/// The status of an invocation that refused:
/// an unreadable argument,
/// or a mode that would not act.
pub const EXIT_REFUSED: i32 = 2;

/// What:
///  The bytes of one whole line,
///  its newline included.
/// Why:
///   The door writes exactly the bytes it is handed,
///  so a caller that passed its sentence alone ran the next line into it:
///  two `--version` runs printed `mount-git-worktree-clone-datasets 0.6.1-alpha.44mount-git-worktree-clone-datasets 0.6.1-alpha.44`,
///  and a refusal followed the line before it with no break.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineToWrite = (line) => `${line}\n`;
/// ```
fn line_to_write(line: &str) -> String {
    return format!("{line}\n");
}

/// What:
///  Say one report line on the process's standard output,
///  which is where a command's answer belongs.
/// Why:
///   `--help` and `--version` are the program's answer,
///  and a person who pipes it (`mount-git-worktree-clone-datasets --version | grep`) reads standard output:
///  a report on standard error would print on the terminal and leave the pipe empty.
///  The write is small and line-terminated,
///  exactly what the wrapper's own exit report writes to the same stream (`invocation::write_exit`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reportLine = (line) => process.stdout.write(lineToWrite(line));
/// ```
fn report_line(line: &str) {
    let _written = std::io::stdout().write_all(line_to_write(line).as_bytes());
}

/// What:
///  Say one whole line to the process's standard error,
///  through the door every diagnostic line goes through.
/// Why:
///   `eprintln!` takes the stream handle every writer of this process shares,
///  and a reader that stopped draining it would hold this program where it stands;
///  the door hands the line to the thread that never holds its caller,
///  and the line reaches it with the newline that ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const say = (line) => write(lineToWrite(line)); // the diagnostic door
/// ```
fn say(line: &str) {
    write(line_to_write(line).as_str());
}

/// What:
///  One refusal as the process prints it:
///  opening with the words every root mode is reached through, once.
/// Why:
///   `SPEC.md` states that the modes' refusals open with the words,
///  and a mode's own sentence may name its cause first while another already
///  opens with them;
///  every mode's refusal passes through `report`,
///  so this is the one place that adds them where they are missing and never
///  twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const openingWithTheWords = (refusal) => refusal.startsWith(words + ': ') ? refusal : `${words}: ${refusal}`;
/// ```
fn opening_with_the_words(refusal: String) -> String {
    let named: String = super::handler_install_paths::root_modes_name();
    let opening: String = format!("{named}: ");
    if refusal.starts_with(opening.as_str()) {
        return refusal;
    }
    return format!("{opening}{refusal}");
}

/// What:
///  The status one mode's answer becomes,
///  writing its line first.
/// Why:
///   The success line and the refusal line are both a person's answer,
///  and only the success reaches zero;
///  the refusal opens with the words every root-mode refusal opens with,
///  whether the mode built that sentence itself or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = (outcome) => { if (outcome.ok) { say(outcome.line); return 0 } say(openingWithTheWords(outcome.refusal)); return 2 };
/// ```
fn report(outcome: Result<String, String>) -> i32 {
    return match outcome {
        Ok(line) => {
            say(line.as_str());
            EXIT_DONE
        }
        Err(refusal) => {
            say(opening_with_the_words(refusal).as_str());
            EXIT_REFUSED
        }
    };
}

/// What:
///  Run the compatibility gate before a mode that will make requests,
///  and answer
///  with the gate's line when it refuses.
/// Why:
///  No mode may send a raw request before the loaded module's declaration is
///  compared,
///  and a refusal is the whole answer:
///  the mode is not entered,
///  so it opens nothing,
///  mounts nothing,
///  unmounts nothing and records nothing,
///  and registrations and durable recovery state stay as they were.
fn behind_the_gate(
    gate: &dyn BeforeRequests,
    outcome: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    let verdict: Verdict = gate.verdict();
    let line: String = verdict_line(&verdict);
    if let Verdict::Refused(_) = verdict {
        // The gate's own sentence names the compatibility check and nothing ties it to the modes, so every mode's gate
        // refusal opens with the words a person or root's journal reads first.
        let named: String = super::handler_install_paths::root_modes_name();
        return Err(format!(
            "{named}: {line}\nno mode was entered and no raw request was made, so this invocation mounted nothing, unmounted nothing and recorded nothing"
        ));
    }
    return match outcome() {
        Ok(done) => Ok(format!("{line}\n{done}")),
        Err(refusal) => Err(format!("{line}\n{refusal}")),
    };
}

/// What:
///  Run the start probe behind the gate,
///  and answer
///  with the skip line and a skipped record when the gate refuses.
/// Why:
///  The probe's own mode is entered only on a compatible verdict,
///  and a refusal leaves the record saying the probe was skipped and that
///  nothing was mounted,
///  because the gate's refusal authorizes no mount
///  (`doc/planning/cli-git-zfs-start-probe-design.md`,
///  "Who runs it,
/// and where").
///  The mode still exits successfully:
/// a skipped probe is a verdict in the
///  record,
/// never a failed unit.
/// The place and the configuration path are arguments,
/// so a control runs the refused branch against a fixture directory and never
/// writes under `/run`.
fn behind_the_gate_for_the_probe(gate: &dyn BeforeRequests, place: &RecordPlace, configuration: &std::path::Path) -> Result<String, String> {
    let effective_user = geteuid();
    return behind_the_gate_for_the_probe_as(gate, place, configuration, effective_user.is_root(), effective_user.as_raw());
}

/// What:
///  Run the start probe behind the gate,
///  refusing first a caller that is not root,
///  with the caller's answer to whether this process runs as root.
/// Why:
///  Every write the probe makes is root's:
///  the record under `/run`,
///  the scratch tree,
///  the private mount namespace and the attach it proves,
///  so an account is refused before the gate is asked and before anything is
///  written,
///  and its status is the refusal status rather than the zero a probe that
///  reached its own end answers.
///  The effective user is an argument because a control cannot set it:
///  a control drives the not-root refusal without root,
///  and the program passes the user the kernel reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = behindTheGateForTheProbeAs(gate, place, configuration, false, 1000); // Err: not root
/// ```
fn behind_the_gate_for_the_probe_as(
    gate: &dyn BeforeRequests,
    place: &RecordPlace,
    configuration: &std::path::Path,
    running_as_root: bool,
    effective_user: u32,
) -> Result<String, String> {
    if !running_as_root {
        return Err(NotRootRefusal {
            mode: "start-probe",
            nothing: "read, mounted or recorded",
            because: "the record it writes decides whether a boot's clones are admitted for attachment, and an account that could write it could report a boot that never passed",
            way: "run the mode as root, from the boot unit or by hand",
            effective_user,
        }
        .sentence());
    }
    let verdict: Verdict = gate.verdict();
    let line: String = verdict_line(&verdict);
    if let Verdict::Refused(refusal) = &verdict {
        let skipped: Result<String, String> = skip_after_gate_refusal(place, &FingerprintPaths::live(), refusal.step.as_str());
        return Ok(format!("{line}\n{}", skipped.unwrap_or_else(|refusal_line| return refusal_line)));
    }
    let mode: ProbeMode = ProbeMode::new(place, configuration);
    return match mode.run() {
        Ok(done) => Ok(format!("{line}\n{done}")),
        Err(refusal) => Ok(format!("{line}\n{refusal}")),
    };
}

/// What:
///  Where one command line's modes read and write,
///  with the installed paths by default.
/// Why:
///   The release path takes the installed places,
///  and a control hands in fixture places,
///  so no control reads this machine's configuration,
///  store,
///  probe record or readiness folder,
///  and the order the modes run in stays the one the release path takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DispatchPlaces = { sweep: SweepPlaces; reconcile: ReconcilePlaces; readiness: ReadinessFile; probe: RecordPlace; probeConfiguration: string };
/// ```
pub struct DispatchPlaces {
    /// The sweep's places.
    pub sweep: SweepPlaces,
    /// The boot reconciliation's places.
    pub reconcile: ReconcilePlaces,
    /// The readiness file the boot reconciliation publishes.
    pub readiness: ReadinessFile,
    /// The start probe's record place.
    pub probe: RecordPlace,
    /// The start probe's configuration file.
    pub probe_configuration: PathBuf,
}

impl DispatchPlaces {
    /// The installed places,
    /// which is what a boot or a ZED event reaches.
    pub fn installed() -> Self {
        return DispatchPlaces {
            sweep: SweepPlaces::installed(),
            reconcile: ReconcilePlaces::installed(),
            readiness: ReadinessFile::live(),
            probe: RecordPlace::live(),
            probe_configuration: PathBuf::from(CONFIGURATION_PATH),
        };
    }
}

/// What:
///  Refuse a start the words or the one name could not honour,
///  before the modes' grammar sees it.
/// Why:
///   A person who typed a word under the installed name reads why that name
///  answers no word but its own,
///  and a person who typed the group word alone reads which mode word it needs;
///  the usage follows either refusal,
///  and the status is the one every refusal answers,
///  so no argument vector at that path reaches Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exit(refuseRootModeMisuse({ kind: 'word-under-the-zedlet-name', word: 'boot-reconcile' }));
/// ```
pub fn refuse_root_mode_misuse(misuse: RootModeMisuse<'_>) -> i32 {
    let named: String = super::handler_install_paths::root_modes_name();
    let sentence: String = match misuse {
        RootModeMisuse::NoModeWordAfterTheGroupWord => format!(
            "{named}: `{}` needs a mode word after it. Nothing was done. The modes are {}, and the clone-mount sweep has no mode word: ZED starts the file under the link name {} with no argument, and that start runs it. Ways forward: run `{named} --help` for the modes this build accepts.",
            super::root_mode_names::ROOT_GROUP_WORD,
            MODE_WORDS.join(", "),
            super::root_mode_names::ZEDLET_START_NAME
        ),
        RootModeMisuse::WordUnderTheZedletName(word) => format!(
            "{named}: `{}` is an argument this start does not take: under this file's own name only no argument, `--help` and `--version` are answered. Nothing was done. Ways forward: run the file with no argument for the clone-mount sweep, with `--help` for the modes' usage, or with `--version` alone; every mode with a word is reached through the words, as `{named} <mode>`.",
            word.to_string_lossy()
        ),
    };
    say(sentence.as_str());
    say(handler_mode_arguments::usage().as_str());
    return EXIT_REFUSED;
}

/// What:
///  Read one invocation and run it.
/// Why:
///   The argument grammar is the only way in:
///  an unreadable invocation prints its cause and the usage and refuses,
///  and each mode that has landed runs,
///  while each mode whose step has not prints that step and refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = run(argv); // 0 when the mode did its work, 2 otherwise
/// ```
pub fn run(arguments: &[OsString]) -> i32 {
    return run_behind(&LiveGate, arguments);
}

/// What:
///  Read one invocation and run it behind the given gate.
/// Why:
///   The release path's gate reads this machine's kernel files and device,
///  so a
///  control hands in a gate that answers without them,
///  and the order the modes run
///  in stays the one the release path takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = runBehind(gate, argv);
/// ```
pub fn run_behind(gate: &dyn BeforeRequests, arguments: &[OsString]) -> i32 {
    return run_behind_at(gate, &DispatchPlaces::installed(), arguments);
}

/// What:
///  Read one invocation and run it behind the given gate,
///  from the places it is given.
/// Why:
///   The installed places hold this machine's configuration,
///  store and `/run` folders,
///  so a control hands in fixtures and no mode reads or writes anything the
///  machine owns;
///  the gate is an argument for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = runBehindAt(gate, places, argv);
/// ```
pub fn run_behind_at(gate: &dyn BeforeRequests, places: &DispatchPlaces, arguments: &[OsString]) -> i32 {
    let mode: HandlerMode = match handler_mode_arguments::parse(arguments) {
        Ok(mode) => mode,
        Err(refusal) => {
            say(refusal.message().as_str());
            say(handler_mode_arguments::usage().as_str());
            return EXIT_REFUSED;
        }
    };
    return match mode {
        HandlerMode::Help => {
            report_line(handler_mode_arguments::usage().as_str());
            EXIT_DONE
        }
        HandlerMode::Version => {
            report_line(super::handler_install_paths::version_line().as_str());
            EXIT_DONE
        }
        HandlerMode::AttachingSweep => report(behind_the_gate(gate, || return attaching_sweep_mode::run_at(&places.sweep))),
        HandlerMode::BootReconcile => report(behind_the_gate(gate, || return boot_reconcile_mode::run_at(&places.reconcile, &places.readiness))),
        HandlerMode::StartProbe => report(behind_the_gate_for_the_probe(gate, &places.probe, places.probe_configuration.as_path())),
        HandlerMode::ShutdownUnmount => report(behind_the_gate(gate, || {
            return shutdown_unmount_mode::run().map_err(|awaiting| return awaiting.message());
        })),
        HandlerMode::HolderScan(invocation) => report(behind_the_gate(gate, || return holder_scan_mode::run(&invocation))),
        HandlerMode::AdoptWorktreeParent(request) => report(adopt_worktree_parent_mode::run(&request).map_err(|failure| return failure.message())),
        HandlerMode::RecordCheckout(request) => report(behind_the_gate(gate, || return record_checkout_mode::run(&request).map_err(|failure| return failure.message()))),
        HandlerMode::MountPathDescendant(request) => report(behind_the_gate(gate, || return mount_path_descendant_mode::run(&request).map_err(|failure| return failure.message()))),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_mode_dispatch_tests.rs"]
mod tests;

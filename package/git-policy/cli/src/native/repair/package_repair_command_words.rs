//! What:
//!  The words a tool's run is reported in:
//! the end of what it printed,
//! how its status is said,
//! the line that announces a tool still running,
//! and the name its quiet gap is kept under.
//! Why:
//!  A repair words every end so a person reads one plain verdict,
//! and the same words are built whether the run happened here or was abandoned;
//! keeping them apart from the run lets a control read them without touching
//! the process's standard error,
//! and lets the ceiling that gives up a write be read beside the line it bounds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tailOf = (bytes: Uint8Array) => string;
//! const failureText = (bytes: Uint8Array) => string;
//! const describeStatus = (status: ExitStatus) => string;
//! const whatItWaitsFor = (program: string, command: ToolCommand, limits: ToolLimits) => string;
//! const toolStep = (program: string, command: ToolCommand) => string;
//! ```

/// Import the command and the bounds one run is asked for,
/// which the notice and the quiet gap's name both read.
use crate::package_repair_command_answers::{
    ToolCommand,
    ToolLimits,
};
/// Import the process's own standard error,
/// written without Rust's shared standard-error lock.
use crate::standard_error_descriptor::StandardErrorDescriptor;
/// Import the bound of the notice's write.
use crate::wait_ceilings::REPAIR_NOTICE_WRITE_CEILING;
/// Import the write that gives up at a ceiling,
/// because a stream whose reader stopped draining waits in the kernel until it resumes.
use crate::work_awaited_within_a_ceiling::write_within;
/// Import the owned,
/// platform-native string one argument of a command line is.
use std::ffi::OsString;
/// Import the borrowed filesystem path a program is named by.
use std::path::Path;
/// The most bytes of a program's output kept for its failure text.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub(super) const OUTPUT_TAIL_BYTES: usize = 600;

/// What:
///  The last bytes of what a program printed,
/// as text,
/// with the cut ragged edge dropped,
/// and started at a sentence when the tool printed one line and the cut fell inside it.
/// Why:
///   A failure is worded with the end of the output,
/// where tools print their reason;
/// the start is progress noise.
/// A person reads the failure as one line,
/// so a kept text that began in the middle of a sentence named neither the tool's reason nor its own:
/// `package/dsh-plugin/cost-dismissal/node_modules/zod". The link at "/var/` was all a service journal showed of pnpm's
///  reason,
/// and the sentence that said why came after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tailOf(bytes: Uint8Array): string;
/// ```
pub(super) fn tail_of(bytes: &[u8]) -> String {
    let cut: usize = bytes.len().saturating_sub(OUTPUT_TAIL_BYTES);
    let text: String = String::from_utf8_lossy(&bytes[cut..]).into_owned();
    // Lines that only say a tool waited for another process say nothing about why it failed.
    let mut informative_lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        if !line
            .trim_start()
            .starts_with("Blocking waiting for file lock")
        {
            informative_lines.push(line);
        }
    }
    let informative: String = informative_lines.join("\n");
    let trimmed: &str = informative.trim();
    if cut == 0 {
        return trimmed.to_owned();
    }
    let whole_lines: &str = match trimmed.split_once('\n') {
        Some((_ragged, rest)) => rest,
        // One line with no break in it has no whole line to start at, so it starts at the first sentence that follows the
        //  cut. A line that holds no sentence at all keeps its end: the reason a tool prints last is the point of the
        //  text, and the sentence beside it says how much was left out.
        None => {
            let sentence: &str = sentence_after_the_cut(trimmed);
            if sentence.is_empty() {
                trimmed
            } else {
                sentence
            }
        }
    };
    return whole_lines.trim().to_owned();
}

/// What:
///  The text from its first sentence on,
///  or nothing when it holds no sentence start at all.
/// Why:
///   A tool that prints its reason as one line cannot be cut at a line break,
///   and cutting inside a sentence leaves words that name no tool's claim;
///   the first `. ` after a cut is where a sentence of that line begins again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentenceAfterTheCut = (text) => { const at = text.indexOf('. '); return at < 0 ? '' : text.slice(at + 2) };
/// ```
fn sentence_after_the_cut(text: &str) -> &str {
    return match text.find(". ") {
        Some(end) => &text[end + 2..],
        None => "",
    };
}

/// What:
///  The words one failed tool's output is reported with:
/// the kept end of it,
/// and one sentence saying that it printed more than is kept.
/// Why:
///   The kept text is bounded,
///   and a bound the person cannot see is a bound that reads as the whole output;
///   the sentence ends the report at a full stop wherever the kept text was cut,
///   and a tool whose output fit says nothing extra.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failureText = (bytes) => bytes.length <= OUTPUT_TAIL_BYTES ? tailOf(bytes) : `${tailOf(bytes)} The tool printed ${bytes.length} bytes and only its end is kept here.`;
/// ```
pub(super) fn failure_text(bytes: &[u8]) -> String {
    let kept: String = tail_of(bytes);
    if bytes.len() <= OUTPUT_TAIL_BYTES {
        return kept;
    }
    return format!(
        "{kept} The tool printed {} bytes and only its end is kept here.",
        bytes.len()
    );
}

/// What:
///  How a finished program's status is worded.
/// Why:
///   A signal has no code,
/// and the person should see which of the two ended the program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeStatus(status: ExitStatus): string;
/// ```
pub(super) fn describe_status(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    return match (status.code(), status.signal()) {
        (Some(code), _) => format!("exit status {code}"),
        (None, Some(signal)) => format!("stopped by signal {signal}"),
        (None, None) => String::from("ended without a status"),
    };
}

/// What:
///  The line that says a repair tool is still running,
/// what it runs,
/// where,
/// and what ends it.
/// Why:
///   The two bounds that end a tool (the quiet window and the ceiling) are the only thing that ends it,
/// and a person who
///  sees nothing for ten minutes cannot tell a tool that works from one the wrapper lost;
/// the tool,
/// its
///  arguments and the worktree it runs in are what tells them apart.
///  This write runs on the thread that drives the loop,
/// so a reader that stopped draining would hold the loop inside the kernel and neither bound would ever fire:
/// the tool the repair owns would never be stopped,
/// its process group would keep running,
/// and an add ended with a signal would leave it behind.
/// The line is advisory beside the tool's own result,
/// so it is given up at `REPAIR_NOTICE_WRITE_CEILING`
/// (`wait_ceilings.rs`),
/// and the loop goes on reading the bounds that end the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayWhatItWaitsFor(pnpm, { arguments: ['install', '--offline'], directory: worktree }, { window: 600, ceiling: 1080 })
/// ```
pub(super) fn say_what_it_waits_for(
    program: &Path,
    command: &ToolCommand,
    limits: ToolLimits,
) {
    let words: String = what_it_waits_for(program, command, limits);
    #[cfg(test)]
    NOTICES_SAID.lock().expect("notices").push(words.clone());
    // Terminal output: this line is the person's, not a log record, and there is nowhere to report a closed standard error.
    //  The descriptor is written directly, so a write this run abandons holds no lock the run's next line would wait for.
    let _written: Option<std::io::Result<()>> = write_within(
        REPAIR_NOTICE_WRITE_CEILING,
        StandardErrorDescriptor,
        format!("{words}\n"),
    );
}

/// What:
///  The words of that line,
/// which a control reads without touching the process's standard error.
/// Why:
///   A repair's waits are announced on standard error,
/// which a control of a test binary that runs its tests side by side cannot
///  take for itself;
/// the words are built here and printed by the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = whatItWaitsFor(pnpm, command, limits);
/// ```
fn what_it_waits_for(
    program: &Path,
    command: &ToolCommand,
    limits: ToolLimits,
) -> String {
    let arguments: Vec<String> = command.arguments.iter().map(argument_text).collect();
    return format!(
        "monochromatic-git-policy-cli: {} {} is still running in {} for the repair of the new worktree; it is stopped when it shows no sign of work for {} seconds, or after {} minutes at the latest.",
        program.display(),
        arguments.join(" "),
        command.directory.display(),
        limits.window.as_secs(),
        limits.ceiling.as_secs() / 60
    );
}

/// What:
///  One argument of a tool's command line,
/// as the text a person reads.
/// Why:
///  The parameters of a repair are what tells two runs of the same tool apart
/// (`--offline`,
/// `--frozen-lockfile`),
/// and the notice is the only place a person sees them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// argumentText(OsString('--offline')) // '--offline'
/// ```
fn argument_text(word: &OsString) -> String {
    return word.to_string_lossy().into_owned();
}

/// What:
///  What the waiting notices said,
/// so a control reads them without taking the process's standard error.
/// Why:
///   A tool's wait runs as a task of the process's runtime,
/// on a worker of its own,
/// so a record of one thread would miss it;
/// a control of a test process that runs its tests side by side reads its own line by the
///  fixture path in it.
#[cfg(test)]
pub(crate) fn notices_said() -> Vec<String> {
    return NOTICES_SAID.lock().expect("notices").clone();
}

/// The notices the controls read.
#[cfg(test)]
static NOTICES_SAID: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// What:
///  The name a tool's quiet gap is kept under:
/// the program's file name and its first argument,
/// as in `pnpm install`.
/// Why:
///   Two uses of one program (`cargo metadata`,
/// `cargo clean`) have different gaps,
/// and the arguments after the first are
///  paths that would make a name for every worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [basename(program), args[0]].filter(Boolean).join(' ')
/// ```
pub(super) fn tool_step(
    program: &Path,
    command: &ToolCommand,
) -> String {
    let name: String = match program.file_name() {
        Some(file) => file.to_string_lossy().into_owned(),
        None => String::new(),
    };
    return match command.arguments.first() {
        Some(first) => format!("{name} {}", first.to_string_lossy()),
        None => name,
    };
}

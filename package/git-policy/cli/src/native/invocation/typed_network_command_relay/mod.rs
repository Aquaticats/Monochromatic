//! What:
//!  The relay a typed network command gets when ssh carries its remote:
//!  read the remote first,
//!  run Git as this process's child with its error stream on a pseudo-terminal or a pipe,
//!  relay those bytes as they arrive,
//!  and print the sentence that names the next step after a refusal.
//! Why:
//!  `doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md` decides it:
//!  only a command ssh carries is relayed,
//!  an `https` or `file` remote keeps the process replacement it has today,
//!  the child is published to the existing signal relay,
//!  and the person reads one sentence after OpenSSH's refusal at a new host key or a key no agent holds.
//!  The four readers are Git's own answers, so `insteadOf` and a push URL are applied exactly as the command applies them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // relayWhenSshCarries(git, argv, env) : { outcome, code } | undefined  // undefined = keep today's process replacement
//! ```

/// The four command words, the remote URL Git would use, and whether ssh carries it.
pub mod the_remote_a_typed_command_names;
/// The first of a command's words that is not an option, read with Git's own option table.
pub mod the_options_before_the_remote;
/// The host a remote names as the person typed it, and the words an `ssh` command needs to connect there.
pub mod a_target_the_sentence_may_name;
/// The child started with its error stream on a terminal or a pipe, and the wait that reads it to the child's end.
pub mod git_error_stream_relayed_as_it_arrives;
/// The pseudo-terminal a child writes its error stream to when this process's standard error is a terminal.
pub mod a_terminal_the_child_writes_its_error_stream_to;
/// The relayed bytes: read as they arrive, written on, and kept as a bounded tail.
pub mod the_relayed_bytes;
/// The sentence printed after a refusal, and the record that one was printed.
pub mod the_sentence_after_the_refusal;

// What: `OsString` is owned operating-system text of raw bytes (its sibling `String` must be UTF-8).
// Why:  The person's arguments and the overlay are passed to Git without being decoded.
//
// In TS you'd write (pseudocode):
// ```ts
// // process.argv entries, byte-preserving.
// ```
use std::ffi::OsString;
// What: `Path` is a borrowed filesystem path of raw bytes.
use std::path::Path;
// What: Who a real-Git start runs for, which decides the monitor setting the child gets.
use crate::hooks_off::StartOwner;
// What: The start of a real Git, with the marker and the hooks-off prefix every start carries.
use crate::forwarding::git_command;
// What: The guard that owns this process's relaying handlers, and how the child ended.
use crate::signal_relay::SignalGuard;
use crate::forwarding::ChildOutcome;
// What: The two halves of the error-stream relay, and the shape K2 decides between.
use git_error_stream_relayed_as_it_arrives::{ErrorStreamShape, RelayedErrorStream, read_to_the_child_s_end, shape_of_this_process, start_relaying};
// What: The refusal reading, the sentence, and the record of printing one.
use the_sentence_after_the_refusal::print_after_a_refusal;
// What: The command, its remote read and the host the sentence names, plus whether ssh carries the remote.
use the_remote_a_typed_command_names::{RemoteRead, TypedNetworkCommand, remote_read_for, remote_url, ssh_carries, typed_command};
// What: The host the sentence names, built from the URL's own grammar.
// Why:  A sentence without a host it may print names no next step.
//
// In TS you'd write (pseudocode):
// ```ts
// import { type Target, targetOf } from './a-target-the-sentence-may-name';
// ```
use a_target_the_sentence_may_name::{Target, target_of};

/// What:
///  How a typed network command ended when the wrapper kept Git as its child.
/// Why:
///   A code passes through, and a signal is this process's to end by,
///   exactly as the process replacement let Git's caller see it
///   (`forwarding::end_as_git_ended`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TypedNetworkCommandEnding = { ran: number } | { signalled: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedNetworkCommandEnding {
    /// Git exited with this code, and the wrapper's exit is the same code.
    Ran(i32),
    /// A signal ended Git; the wrapper ends by that signal, and this number is the fallback code.
    Signalled(i32),
}

impl TypedNetworkCommandEnding {
    /// What:
    ///  The code this process ends with, after ending by Git's own signal when a signal ended Git.
    /// Why:
    ///   The plain forward replaced this process with Git, so its caller saw `WIFSIGNALED` when Git was signalled;
    ///   a route that keeps a process around Git has to end the same way or a script takes the wrong branch
    ///   (`forwarding::end_as_git_ended`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// exitCode(): number { return this.ran ?? endAsGitEnded(this.signalled); }
    /// ```
    pub fn exit_code(self) -> i32 {
        // `match` picks by variant and binds the number the variant carries.
        return match self {
            TypedNetworkCommandEnding::Ran(code) => code,
            TypedNetworkCommandEnding::Signalled(signal) => crate::forwarding::end_as_git_ended(signal),
        };
    }
}

/// What:
///  The relay for a typed network command whose remote ssh carries, or nothing when this command keeps its process
///  replacement.
/// Why:
///  Every caller asks this before it replaces the process or starts a child,
///  so one decision serves the plain forward, the coordinated run, the settled copy and the wrapper's own steps:
///  a command ssh does not carry, a remote Git cannot name, or a URL this wrapper cannot word all keep today's behaviour.
///  An `Err` says Git could not be started, which each caller reports in its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relayWhenSshCarries(git, argv, overlay, owner): Result<TypedNetworkCommandEnding> | undefined
/// ```
pub fn relay_when_ssh_carries(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    owner: StartOwner,
) -> Option<std::io::Result<TypedNetworkCommandEnding>> {
    // What: `(command, region, prefix)` names the typed command, its own words, and Git's global options before it.
    // Why:   Only the four network commands are relayed, and the remote is read in the repository those options name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const typed = typedCommand(argv); if (!typed) return undefined;
    // ```
    let (command, region, prefix): (TypedNetworkCommand, &[OsString], &[OsString]) = match typed_command(arguments) {
        Some(named) => named,
        // What: `None` ends this function with no relay, and the line says which command word was read.
        // Why:   A decision that nothing relays must be readable when a person asks why a command named no next step.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // debug('typedNetworkCommandRelay', 'not one of the four network commands'); return undefined;
        // ```
        None => {
            crate::diagnostic_log::debug("typedNetworkCommandRelay", "not one of the four typed network commands, so the process replacement it has stays");
            return None;
        }
    };
    // What: `RemoteRead` says where the URL comes from: a clone's own word, or a remote the repository knows.
    // Why:   The two need different questions to Git.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const read = remoteReadFor(command, region); if (!read) return undefined;
    // ```
    let read: RemoteRead<'_> = match remote_read_for(command, region) {
        Some(named) => named,
        None => {
            crate::diagnostic_log::debug("typedNetworkCommandRelay", "the command's remote could not be read from its words, so the process replacement it has stays");
            return None;
        }
    };
    // What: `String` is the URL Git answers with; `None` means Git refused the question.
    // Why:   An unknown remote keeps today's behaviour instead of a relay over a transport nobody read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const url = remoteUrl(command, read, git, overlay); if (!url) return undefined;
    // ```
    let url: String = match remote_url(command, &read, prefix, real_git, overlay) {
        Some(read_url) => read_url,
        None => {
            crate::diagnostic_log::debug("typedNetworkCommandRelay", "Git answered no URL for this command's remote, so the process replacement it has stays");
            return None;
        }
    };
    crate::diagnostic_log::debug("typedNetworkCommandRelay", format!("the remote is {url}").as_str());
    if !ssh_carries(url.as_str()) {
        crate::diagnostic_log::debug("typedNetworkCommandRelay", "ssh does not carry this remote, so the process replacement it has stays");
        return None;
    }
    // What: `Target` is the host as the person's remote names it, and the words an `ssh` command needs.
    // Why:   The sentence cannot name a next step without a host it may print.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const target = targetOf(url); if (!target) return undefined;
    // ```
    let target: Target = match target_of(url.as_str()) {
        Some(named) => named,
        None => {
            crate::diagnostic_log::debug("typedNetworkCommandRelay", "the remote names no host a sentence may print, so the process replacement it has stays");
            return None;
        }
    };
    // What: `Ok(guard)` owns this process's relaying handlers; the sibling answer is a system call that failed.
    // Why:   A signal sent to the wrapper must reach Git, and where no handler can be installed the command keeps the
    //        process replacement it has today, which lets the signal reach Git as it always did.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let guard; try { guard = SignalGuard.install(); } catch (error) { warn(...); return undefined; }
    // ```
    let guard: SignalGuard = match SignalGuard::install() {
        Ok(installed) => installed,
        Err(error) => {
            crate::diagnostic_log::warn(
                "typedNetworkCommandRelay",
                format!("no signal relay could be installed ({error}), so this command keeps the process replacement it had").as_str(),
            );
            return None;
        }
    };
    // What: `shape` is K2's one answer, from one `isatty` call on this process's standard error.
    // Why:   The child's stream is decided before it starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shape = isatty(2) ? 'terminal' : 'pipe';
    // ```
    let shape: ErrorStreamShape = shape_of_this_process();
    // What: `git_command` builds the child exactly as the process replacement would have, so the arguments, the hooks-off
    //       prefix and the marker are the same in both shapes.
    // Why:   A relayed command must be the same command, with one stream connected differently.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const command = gitCommand(git, argv, overlay, owner);
    // ```
    let mut git_to_run: std::process::Command = git_command(real_git, arguments, overlay, owner);
    // What: `Err(io::Error)` is the failure variant: the operating system refused the start, which the caller reports.
    // Why:   A refused start is an engine failure and not a Git answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let start; try { start = startRelaying(shape, guard, command); } catch (error) { return { error }; }
    // ```
    let started: git_error_stream_relayed_as_it_arrives::RelayedStart = match start_relaying(shape, Some(&guard), &mut git_to_run) {
        Ok(running) => running,
        Err(error) => return Some(Err(error)),
    };
    // What: `RelayedErrorStream` is the child's end and the tail of what it wrote.
    // Why:   The sentence reads that tail, and the end is Git's answer to the person.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relayed = readToTheChildSEnd(start);
    // ```
    let relayed: RelayedErrorStream = match read_to_the_child_s_end(started, &mut std::io::stderr()) {
        Ok(finished) => finished,
        Err(error) => return Some(Err(error)),
    };
    // The repository's record names this Git, so a lock it takes is explainable after this process is gone.
    crate::lock_holder_records::note_child_started(relayed.child_number);
    // What: `match` picks by variant: a code passes through and a signal is kept for the caller to end by.
    // Why:   The sentence is printed for every non-zero end before the caller decides how to end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const code = 'exited' in relayed.outcome ? relayed.outcome.exited : 128 + relayed.outcome.signaled;
    // printAfterARefusal(relayed.kept, code, target);
    // return 'exited' in relayed.outcome ? { ran: code } : { signalled: relayed.outcome.signaled };
    // ```
    let (ended_with, ending): (i32, TypedNetworkCommandEnding) = match relayed.outcome {
        ChildOutcome::Exited(code) => (code, TypedNetworkCommandEnding::Ran(code)),
        ChildOutcome::Signaled(signal) => (crate::forwarding::SIGNAL_EXIT_BASE.saturating_add(signal), TypedNetworkCommandEnding::Signalled(signal)),
    };
    print_after_a_refusal(relayed.kept.as_slice(), ended_with, &target);
    // What: `Some(Ok(..))` says the command ran the relayed way; the ending says how it ended.
    // Why:   A caller that reads nothing keeps the process replacement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { outcome: ending };
    // ```
    return Some(Ok(ending));
}

/// What:
///  Print the sentence when the wrapper's own probe of a typed push stopped at a refusal of ssh's.
/// Why:
///  A typed `push` is probed before it runs: the wrapper's own `push --dry-run --porcelain` reads the updates, and at a new host
///  key or a key no agent holds that probe is the process that meets the refusal, so Git's own push never starts.
///  The person then reads OpenSSH's line in the wrapper's words and no next step;
///  the parent that owned the child is this process still, so it is the one to name it
///  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`, and the reading `lfs/lfs_pre_push.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteARefusedProbeOfAPush(git, cwd, overlay, postSubcommand, refusal.stderr)
/// ```
pub fn sentence_after_a_refused_probe_of_a_push(
    real_git: &Path,
    cwd: &Path,
    overlay: &[(OsString, OsString)],
    post_subcommand: &[OsString],
    said: &str,
) -> Option<String> {
    the_sentence_after_the_refusal::refusal_in(said)?;
    // What: `-C <cwd>` is Git's own global option for the directory to read the repository in.
    // Why:  The probe ran there, so the remote is that repository's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const prefix = ['-C', cwd];
    // ```
    let prefix: [OsString; 2] = [OsString::from("-C"), cwd.as_os_str().to_os_string()];
    let mut arguments: Vec<OsString> = Vec::new();
    arguments.push(OsString::from("push"));
    arguments.extend(post_subcommand.iter().cloned());
    // What: `None` ends this function, because a remote Git cannot name here names no host a sentence may print.
    // Why:   A sentence that named no host would help nobody.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const url = remoteUrl('push', read, prefix, git, overlay); if (!url || !sshCarries(url)) return;
    // ```
    let (command, region, _own_prefix): (TypedNetworkCommand, &[OsString], &[OsString]) = typed_command(arguments.as_slice())?;
    let read: RemoteRead<'_> = remote_read_for(command, region)?;
    let url: String = remote_url(command, &read, prefix.as_slice(), real_git, overlay)?;
    if !ssh_carries(url.as_str()) {
        return None;
    }
    let target: Target = target_of(url.as_str())?;
    // What: `match` keeps the refusal the transcript shows; a key failure gets the passphrase sentence, and the rest is silent.
    // Why:   The sentence must fit what OpenSSH printed, or it misleads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const refusal = refusalIn(said); const sentence = sentenceFor(refusal, target);
    // ```
    let refusal: the_sentence_after_the_refusal::Refusal = the_sentence_after_the_refusal::refusal_in(said)?;
    the_sentence_after_the_refusal::remember_printed();
    return Some(the_sentence_after_the_refusal::sentence_for(refusal, &target));
}

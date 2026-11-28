//! What:
//!  The gate that asks the repository link check before a command touches the repository.
//! Why:
//!  Nothing may be created in the wrong place when the link dangles.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gate = repositoryLinkGate({ home, paths });
//! ```

use crate::repository_link_check_walk::{
    DEFAULT_DEADLINE,
    check_repository_link_within,
};
use crate::repository_link_identity::{
    IdentityRead,
    LinkIdentity,
    LinkPhase,
    read_identity,
};
use crate::repository_link_paths::{
    named_paths,
    touches_repository,
};
use crate::repository_link_probe::{
    EntryKind,
    LinkProbe,
    RealLinkProbe,
};
use std::ffi::OsString;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;

/// What:
///  What the gate decided.
/// Why:
///  The caller continues or prints the text and exits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkGate = { kind: 'clear' } | { kind: 'stop'; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkGate {
    /// The command goes on.
    Clear,
    /// The command stops with this standard error and status 128.
    Stop(String),
}

/// What:
///  The identity whose repository a full check already found usable in this process.
/// Why:
///  A Git command meets the gate twice (at the entry and in the startup step),
/// and a full check reads the mount table and the by-uuid links;
/// the second meeting asks nothing when the first found the same identity usable.
/// A refusal is never remembered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let readyFor: LinkIdentity | undefined;
/// ```
pub struct ReadyMemory(std::sync::Mutex<Option<LinkIdentity>>);

/// The memory of this process for the real system:
/// the entry gate and the startup step share it.
static PROCESS_MEMORY: ReadyMemory = ReadyMemory::new();

impl ReadyMemory {
    /// An empty memory.
    pub const fn new() -> ReadyMemory {
        return ReadyMemory(std::sync::Mutex::new(None));
    }
}

impl Default for ReadyMemory {
    fn default() -> Self {
        return Self::new();
    }
}

/// What:
///  `gate_for_identity` that skips the check when `memory` holds this identity,
/// and remembers a usable answer it had to walk for.
/// Why:
///  See `ReadyMemory`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gateRemembering({ probe, identity, paths, memory })
/// ```
pub(crate) fn gate_remembering(
    probe: Arc<dyn LinkProbe + Send + Sync>,
    identity: &LinkIdentity,
    paths: &[PathBuf],
    memory: &ReadyMemory,
) -> LinkGate {
    if let Ok(held) = memory.0.lock()
        && held.as_ref() == Some(identity)
    {
        return LinkGate::Clear;
    }
    return match decide(probe, identity, paths) {
        Decision::Ready => {
            if let Ok(mut held) = memory.0.lock() {
                *held = Some(identity.clone());
            }
            LinkGate::Clear
        }
        Decision::Passed => LinkGate::Clear,
        Decision::Refused(text) => LinkGate::Stop(text),
    };
}

/// What the gate decided for an identity and the paths of a command.
enum Decision {
    /// No path lies under the repository,
    /// or phase `before` found its folder:
    /// the command goes on and nothing is worth remembering.
    Passed,
    /// A full check found the repository usable.
    Ready,
    /// The check refused;
    /// this is the text to print.
    Refused(String),
}

/// What:
///  What to do once the identity is known:
/// nothing when no path lies under the repository,
/// one `lstat` in phase `before`,
/// the full check in phase `after`.
/// Why:
///  The gate runs on every Git command,
/// forwarded read-only ones included,
/// so each step is the cheapest test that can end it:
/// phase `before` must stay one `lstat` and a return,
/// and only phase `after` walks the link.
/// Do not call the full check for phase `before` here:
/// it reads the mount table and the by-uuid links,
/// and nothing in phase `before` needs them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gateForIdentity({ probe, identity, paths })
/// ```
fn decide(
    probe: Arc<dyn LinkProbe + Send + Sync>,
    identity: &LinkIdentity,
    paths: &[PathBuf],
) -> Decision {
    let mut touched: bool = false;
    for path in paths {
        if touches_repository(identity, path.as_path()) {
            touched = true;
            break;
        }
    }
    if !touched {
        return Decision::Passed;
    }
    if identity.phase == LinkPhase::Before
        && probe.lstat(Path::new(identity.link_path.as_str())) == EntryKind::Directory
    {
        return Decision::Passed;
    }
    let answer = check_repository_link_within(identity, probe, DEFAULT_DEADLINE);
    if answer.ready {
        return Decision::Ready;
    }
    return Decision::Refused(format!("fatal: {}\n", answer.message));
}

/// `decide` as the plain gate answer,
/// with no memory.
fn gate_for_identity(
    probe: Arc<dyn LinkProbe + Send + Sync>,
    identity: &LinkIdentity,
    paths: &[PathBuf],
) -> LinkGate {
    return match decide(probe, identity, paths) {
        Decision::Refused(text) => LinkGate::Stop(text),
        Decision::Passed | Decision::Ready => LinkGate::Clear,
    };
}

/// What:
///  The words that open the sentence a person reads when the identity file cannot be used,
///  as the message names what is wrong and not only that something is.
/// Why:
///  `recorded_identity` says it,
///  and the control that counts how often one command says it matches on this text,
///  so the words a control looks for are the words the gate writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNUSABLE_IDENTITY_LEAD = 'the repository link identity cannot be used';
/// ```
const UNUSABLE_IDENTITY_LEAD: &str = "the repository link identity cannot be used";

/// What:
///  The reason this process has already said the identity file cannot be used,
///  which is how the sentence each gate says is said once for a command.
/// Why:
///  A command meets the gate twice,
///  at the entry and in the startup step,
///  and each meeting reads the file on its own,
///  so both said the same sentence about one file that cannot be used,
///  and the person read one fault as two
///  (measured 2026-11-26 against the released alpha.48 and alpha.49:
///  `git status --short` printed the line twice in both).
///  A process is one command,
///  which is the same reason the ready answer is remembered here once
///  (`ReadyMemory`).
///  The reason is kept rather than a flag,
///  so a file that changed between the two readings is still said twice:
///  two different sentences are two faults and not one repeated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let said: string | undefined;
/// ```
static SAID_WHY: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// What:
///  Tells the person the identity file could not be used,
/// on standard error,
/// once for this process and this reason.
/// Why:
///  Without the identity the gate cannot name the repository,
/// so the command goes on;
/// saying so keeps the missing protection visible,
/// and saying it once keeps a reader from counting one fault twice.
///  The entry gate and the startup step each read the identity file and neither reads the other's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sayIdentityUnreadableOnce(why: string): boolean { const first = !said || said !== why; said = why; if (first) write(2, `monochromatic-git-policy-cli: ${LEAD} (${why}), ...`); return first; }
/// ```
fn say_identity_unreadable(why: &str) -> bool {
    // What: the reason this process has said, which the guard below replaces with this one.
    // Why:  a mutex that cannot be taken says nothing about this fault, so the sentence is written rather than dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = lock(); if (!held) { write(2, sentence(why)); return true; }
    // ```
    let Ok(mut said) = SAID_WHY.lock() else {
        write_the_sentence(why);
        return true;
    };
    if said.as_deref() == Some(why) {
        return false;
    }
    *said = Some(why.to_owned());
    // What: the lock is given back before the write, which may wait on a reader.
    // Why:  a slow stream must not hold the sentence the next gate asks about in this process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // drop(held); write(2, sentence(why)); return true;
    // ```
    drop(said);
    write_the_sentence(why);
    return true;
}

/// What:
///  The sentence about an identity file that cannot be used,
///  written to standard error.
/// Why:
///  One place writes it,
///  so the words a control counts are the words both gates say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(`monochromatic-git-policy-cli: ${UNUSABLE_IDENTITY_LEAD} (${why}), so ...`);
/// ```
fn write_the_sentence(why: &str) {
    let _noted = std::io::stderr().write_all(
        format!("monochromatic-git-policy-cli: {UNUSABLE_IDENTITY_LEAD} ({why}), so the repository link is not checked for this command\n").as_bytes(),
    );
}

/// What:
///  The recorded identity below `home`,
/// nothing when there is no home or no identity file.
/// Why:
///  An absent file is the usual case and costs one failed open;
/// one that cannot be read is said once for this command on standard error and the command goes on,
/// because without it the gate cannot name the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordedIdentity(home) // LinkIdentity | undefined
/// ```
fn recorded_identity(home: Option<&Path>) -> Option<LinkIdentity> {
    let directory: &Path = home?;
    return match read_identity(directory) {
        IdentityRead::Absent => None,
        IdentityRead::Unreadable(why) => {
            let _said: bool = say_identity_unreadable(why.as_str());
            None
        }
        IdentityRead::Found(identity) => Some(identity),
    };
}

/// What:
///  Runs the gate for explicit paths with a probe:
/// reads the identity file below `home` first,
/// and returns at once when there is none.
/// Why:
///  Tests count the probe's questions;
/// a home without the file costs one failed open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gateWithProbe({ probe, home, paths })
/// ```
pub fn link_gate_with_probe(
    probe: Arc<dyn LinkProbe + Send + Sync>,
    home: Option<&Path>,
    paths: &[PathBuf],
) -> LinkGate {
    let Some(identity) = recorded_identity(home) else {
        return LinkGate::Clear;
    };
    return gate_for_identity(probe, &identity, paths);
}

/// What:
///  Runs the gate for explicit paths against the running system.
/// Why:
///  The paths a layout resolved (a Git directory,
/// a common directory,
/// a work tree) are known when the startup step runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLinkGate({ home, paths })
/// ```
pub fn link_gate(
    home: Option<&Path>,
    paths: &[PathBuf],
) -> LinkGate {
    let Some(identity) = recorded_identity(home) else {
        return LinkGate::Clear;
    };
    return gate_remembering(
        Arc::new(RealLinkProbe::default()),
        &identity,
        paths,
        &PROCESS_MEMORY,
    );
}

/// What:
///  Runs the gate for the paths an invocation names:
/// its `-C` chain,
/// `--git-dir`,
/// `--work-tree` and the Git environment variables.
/// Why:
///  The invocation entry runs this before anything else of a wrapped command,
/// so a dangling link is named before Git prints its own `fatal:` line.
/// The identity file is read first and the paths are made only when it exists:
/// with no file the cost is one failed open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// invocationLinkGate({ home, cwd, globalPrefix, environment })
/// ```
pub fn invocation_link_gate(
    home: Option<&Path>,
    cwd: &Path,
    global_prefix: &[OsString],
    environment: &[(OsString, OsString)],
) -> LinkGate {
    let Some(identity) = recorded_identity(home) else {
        return LinkGate::Clear;
    };
    let paths: Vec<PathBuf> = named_paths(cwd, global_prefix, environment);
    return gate_remembering(
        Arc::new(RealLinkProbe::default()),
        &identity,
        paths.as_slice(),
        &PROCESS_MEMORY,
    );
}

/// Gate controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_gate_tests.rs"]
mod tests;

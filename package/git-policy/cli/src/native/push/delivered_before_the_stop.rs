//! What:
//!  Whether a push that was stopped had already delivered every reference it was asked to deliver.
//! Why:
//!  A push to a local path or a `file://` remote runs the remote's own `receive-pack` inside the push's
//!  process tree,
//! and that program runs the remote's hooks.
//! A `post-receive` hook runs after the references are written,
//! so a hook that shows no sign of work stops a push whose refs are already on the remote.
//!  Saying then that the push failed would be false,
//! telling the person to push again would be useless,
//! and a push with nothing to update runs no remote hook,
//! so the interrupted work would never finish
//!  while the branch read as unpublished.
//! The wrapper cannot see the remote's phase,
//! but it can ask the remote what it holds:
//! for each refspec the push named,
//! the local object and the object the remote has for that reference.
//! Equal means the delivery is done,
//! and then the stop's report says so and names the remote's own work as what was cut short.
//!  The questions are bounded like every other query of the push,
//! so a remote that does not answer leaves the report as it was:
//! the proof never turns into a wait.
//!
//! The objects compared are the ones the push was going to send,
//! recorded before it started:
//! a local reference that moves while the push runs
//! names another object afterwards,
//! and asking it then would compare the remote against something the push never sent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const question = DeliveryQuestion.of(pushArguments); // undefined when the push names no comparable ref
//! const sending = question && await objectsThePushWillSend(git, question); // recorded before the push starts
//! const delivered = sending && await refsAlreadyDelivered(git, question, sending); // undefined unless every ref matched
//! ```

/// A query of the push,
/// bounded by what it shows.
use crate::bounded_query::run_bounded_query_awaiting;
/// Git's own `push` option table,
/// which decides where the destination and the refspecs sit.
use crate::command_push::push_shape;
/// Git's own reading of the options that come before the subcommand,
/// which decide which repository and configuration a start reads.
use crate::global_arguments::{GlobalOutcome, global_layout};
use std::ffi::{OsStr, OsString};
/// The program the push ran,
/// and the directory it ran in.
use std::path::PathBuf;
/// The command that asks the remote.
use std::process::Command;
/// How long the questions may show no sign of work,
/// and how long a stopped one gets to end.
use std::time::Duration;

/// The prefix of every reference the proof read must name,
/// so the question is asked about a reference and not about a pattern.
const REFERENCE_PREFIX: &str = "refs/";

/// What:
///  The references one stopped push was asked to deliver,
/// and where it was delivering them.
/// Why:
///   The two questions per reference are the whole proof:
/// what the local repository names,
/// and what the remote holds for the same
///  reference name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeliveryQuestion = { remote: string; refs: { source: string; destination: string }[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryQuestion {
    /// The destination as the push wrote it:
    /// a remote name or a location.
    pub remote: OsString,
    /// The reference on each side.
    pub refs: Vec<(OsString, OsString)>,
}

/// What:
///  The words before the subcommand: Git's global options,
/// or nothing when the arguments name no subcommand.
/// Why:
///   `-C` names another repository and `-c` another configuration,
/// so a question asked without them reads what the push
///  never read:
/// a mismatched repository answered about references the push did not touch.
///  Git reads its own prefix, so the wrapper does not approximate it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// globalLayout(arguments).outcome === 'command' ? arguments.slice(0, layout.prefixLen) : undefined;
/// ```
pub fn globals_before_the_command(arguments: &[OsString]) -> Option<&[OsString]> {
    let layout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return None;
    }
    return arguments.get(..layout.prefix_len);
}

/// What:
///  The arguments after the `push` word,
/// past the options that come before the subcommand.
/// Why:
///   Every start of the wrapper carries its `-c` switches before the subcommand,
/// and a global option may take a
///  value,
/// so the push region begins after the words Git reads as globals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = globalLayout(arguments);
/// return layout.outcome === 'command' && arguments[layout.prefixLen] === 'push' ? arguments.slice(layout.prefixLen + 1) : undefined;
/// ```
pub fn after_the_subcommand(arguments: &[OsString]) -> Option<&[OsString]> {
    let layout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return None;
    }
    if arguments.get(layout.prefix_len)? != "push" {
        return None;
    }
    return arguments.get(layout.prefix_len + 1..);
}

/// What:
///  The reference on each side of a refspec,
/// when the proof can ask about it.
/// Why:
///   A deletion delivers nothing,
/// a short name on the remote side is not a reference the remote lists,
/// and a `^`
///  revision expression names objects and not the reference that was updated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [source, destination] = refspec.replace(/^\+/, '').split(':', 2);
/// return source && destination.startsWith('refs/') ? [source, destination] : undefined;
/// ```
pub fn refspec_pair(refspec: &OsStr) -> Option<(OsString, OsString)> {
    let text: &str = refspec.to_str()?;
    let without_force: &str = text.strip_prefix('+').unwrap_or(text);
    if without_force.contains('^') {
        return None;
    }
    let (source, destination): (&str, &str) = match without_force.split_once(':') {
        Some((source, destination)) => (source, destination),
        None => (without_force, without_force),
    };
    if source.is_empty() || !destination.starts_with(REFERENCE_PREFIX) {
        return None;
    }
    return Some((OsString::from(source), OsString::from(destination)));
}

impl DeliveryQuestion {
    /// What:
    ///  The question a push's arguments ask,
    /// or nothing when they name no reference the proof can ask about.
    /// Why:
    ///   A matching push,
    /// `--all`,
    /// `--tags`,
    /// `--mirror` and a deletion name no destination reference,
    /// and one
    ///  refspec of such a push would make an answer about the others a guess.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// DeliveryQuestion.of(arguments); // DeliveryQuestion | undefined
    /// ```
    pub fn of(arguments: &[OsString]) -> Option<Self> {
        let region: &[OsString] = after_the_subcommand(arguments)?;
        let shape = push_shape(region)?;
        if shape.branches || shape.tags || shape.mirror || shape.deletes {
            return None;
        }
        let remote: OsString = shape.repository?;
        let mut refs: Vec<(OsString, OsString)> = Vec::new();
        for refspec in &shape.refspecs {
            refs.push(refspec_pair(refspec.as_os_str())?);
        }
        if refs.is_empty() {
            return None;
        }
        return Some(Self { remote, refs });
    }
}

/// What:
///  The program that asks a question of the push's repository and its remote,
/// with the environment the push ran with.
/// Why:
///   The questions are Git's,
/// started the way the push itself was started so that the remote's location,
/// its
///  configuration and the caller's settings are the ones the push saw.
/// The push's global options are repeated for the
///  same reason: `-C` names the repository the push read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AskingGit = { program: string; cwd?: string; environment: [string, string][]; globals: string[] };
/// ```
#[derive(Clone, Debug)]
pub struct AskingGit {
    /// The real Git the push ran.
    pub program: PathBuf,
    /// The directory the push ran in.
    pub cwd: Option<PathBuf>,
    /// The complete environment the push ran with.
    pub environment: Vec<(OsString, OsString)>,
    /// The global options the push carried,
    /// which name the repository and the configuration it read.
    pub globals: Vec<OsString>,
}

/// One question of the proof:
/// its standard output when the query exited zero,
/// and nothing when it failed or was stopped for showing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await ask(git, ['ls-remote', '--refs', remote, destination]);
/// ```
async fn ask(git: &AskingGit, arguments: &[OsString], label: &str, window: Duration, grace: Duration) -> Option<String> {
    let (code, text): (Option<i32>, String) = ask_answering(git, arguments, label, window, grace).await?;
    if code != Some(0) {
        return None;
    }
    return Some(text);
}

/// What:
///  One question of the proof and the exit status Git answered it with,
/// or nothing when the question could not be asked.
/// Why:
///   Some questions tell "not set" (status 1) from "set" (status 0),
/// and a question that could not be answered must not
///  read as an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await askAnswering(git, ['config', '--bool', '--get', key]);
/// ```
async fn ask_answering(
    git: &AskingGit,
    arguments: &[OsString],
    label: &str,
    window: Duration,
    grace: Duration,
) -> Option<(Option<i32>, String)> {
    let mut command: Command = Command::new(git.program.as_path());
    if let Some(cwd) = git.cwd.as_deref() {
        command.current_dir(cwd);
    }
    command.env_clear();
    for pair in &git.environment {
        command.env(&pair.0, &pair.1);
    }
    // The push's own global options: `-C` decides which repository is asked and `-c` which configuration.
    command.args(git.globals.as_slice());
    command.args(arguments);
    // The questions are bounded by what they show, like every other query of the push: one that stalls is no answer.
    let answer_outcome = run_bounded_query_awaiting(command, label, window, grace, git.environment.as_slice()).await;
    let output = match answer_outcome {
        Ok(done) => done,
        Err(_) => return None,
    };
    return Some((output.code, String::from_utf8_lossy(output.stdout.as_slice()).into_owned()));
}

/// What:
///  The object the local repository names for `source`,
/// or nothing when it names none.
/// Why:
///   The push sends the object the local side names,
/// so that is what the remote must hold afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await ask(git, ['rev-parse', '--verify', '--quiet', '--end-of-options', source]))?.trim();
/// ```
async fn local_object(git: &AskingGit, source: &OsStr, window: Duration, grace: Duration) -> Option<String> {
    let arguments: Vec<OsString> = [
        OsString::from("rev-parse"),
        OsString::from("--verify"),
        OsString::from("--quiet"),
        OsString::from("--end-of-options"),
        source.to_os_string(),
    ]
    .to_vec();
    let answer: String = ask(git, arguments.as_slice(), "rev-parse", window, grace).await?;
    let name: String = answer.trim().to_owned();
    if name.is_empty() {
        return None;
    }
    return Some(name);
}

/// What:
///  The object the remote holds for `destination`,
/// or nothing when it holds no such reference.
/// Why:
///   The remote's own list is the only statement about the remote that this process can read
/// without starting a hook or writing anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await ask(git, ['ls-remote', '--refs', remote, destination]))?.split('\t')[0];
/// ```
async fn remote_object(
    git: &AskingGit,
    remote: &OsStr,
    destination: &OsStr,
    window: Duration,
    grace: Duration,
) -> Option<String> {
    let arguments: Vec<OsString> = [
        OsString::from("ls-remote"),
        OsString::from("--refs"),
        remote.to_os_string(),
        destination.to_os_string(),
    ]
    .to_vec();
    let answer: String = ask(git, arguments.as_slice(), "ls-remote", window, grace).await?;
    for line in answer.lines() {
        let mut fields = line.split('\t');
        let (Some(object), Some(reference)) = (fields.next(), fields.next()) else {
            continue;
        };
        if reference == destination.to_string_lossy() {
            return Some(object.to_owned());
        }
    }
    return None;
}

/// What:
///  The location the push delivered to:
/// the URL of the remote it named,
/// or the location it named itself.
/// Why:
///   A push reads a remote's `pushurl` and only falls back to its fetch URL,
/// so asking the remote name asks
///  the repository the push fetched from;
/// with the two naming different repositories the stop could be called delivered while the
///  destination held nothing.
///  A destination that is not a remote name (a path or a URL, as in `git push ../remote.git`) is a location already,
/// and one that is a remote name with no readable push URL is no place this release can ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const url = await ask(git, ['remote', 'get-url', '--push', remote]) ?? (await ask(git, ['remote','get-url', remote]) ? undefined : remote);
/// ```
async fn destination(git: &AskingGit, remote: &OsStr, window: Duration, grace: Duration) -> Option<OsString> {
    let asked: Vec<OsString> = [OsString::from("remote"), OsString::from("get-url"), OsString::from("--push"), remote.to_os_string()].to_vec();
    if let Some(url) = ask(git, asked.as_slice(), "remote get-url", window, grace).await {
        // Git prints the URL and one line break; only that line break is removed, so a location that ends in one keeps it.
        let value: &str = url.strip_suffix('\n').unwrap_or(url.as_str());
        if value.is_empty() {
            return None;
        }
        return Some(OsString::from(value));
    }
    let fetched: Vec<OsString> = [OsString::from("remote"), OsString::from("get-url"), remote.to_os_string()].to_vec();
    if ask(git, fetched.as_slice(), "remote get-url", window, grace).await.is_some() {
        return None;
    }
    return Some(remote.to_os_string());
}

/// What:
///  The objects the push was going to send,
/// one per reference it names,
/// or nothing when the local repository names none of them.
/// Why:
///   The push chooses its objects when it starts,
/// and a reference that moves while the push runs names another object
///  afterwards;
/// recorded now, they are the objects the remote must hold for the delivery to be done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// objectsThePushWillSend(git, question); // string[] | undefined
/// ```
pub async fn objects_the_push_will_send(
    git: &AskingGit,
    question: &DeliveryQuestion,
    window: Duration,
    grace: Duration,
) -> Option<Vec<String>> {
    let mut sending: Vec<String> = Vec::new();
    for (source, _destination) in &question.refs {
        sending.push(local_object(git, source.as_os_str(), window, grace).await?);
    }
    return Some(sending);
}

/// What:
///  The references the remote already holds at the objects the push was sending,
/// or nothing unless every one of them does.
/// Why:
///   A partial answer is no proof:
/// the report says the push delivered what it was asked to deliver only when every
///  reference it named is on the remote,
/// and any question that cannot be answered leaves the report as it was.
///   `sending` is what was recorded before the push started,
/// so a reference that moved afterwards cannot stand for what was sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refsAlreadyDelivered(git, question, sending); // string[] | undefined
/// ```
pub async fn refs_already_delivered(
    git: &AskingGit,
    question: &DeliveryQuestion,
    sending: &[String],
    window: Duration,
    grace: Duration,
) -> Option<Vec<String>> {
    if sending.len() != question.refs.len() {
        return None;
    }
    let where_it_went: OsString = destination(git, question.remote.as_os_str(), window, grace).await?;
    let mut delivered: Vec<String> = Vec::new();
    for ((_source, destination), sent) in question.refs.iter().zip(sending) {
        let remote: String = remote_object(git, where_it_went.as_os_str(), destination.as_os_str(), window, grace).await?;
        if &remote != sent {
            return None;
        }
        delivered.push(destination.to_string_lossy().into_owned());
    }
    return Some(delivered);
}

/// The proof and its questions are controlled in the test module.
#[cfg(test)]
#[path = "delivered_before_the_stop_tests.rs"]
mod tests;

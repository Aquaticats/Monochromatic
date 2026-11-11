//! What:
//!  The server a push waits for,
//! named from the command's own destination,
//! so the wait and the stop that a person and the service's log read say which server held the
//!  connection.
//! Why:
//!  `git push origin` names a remote and not a server,
//! and a stop that says only "the remote" leaves its reader
//!  to find out which one.
//! The destination word is read from the command before the run starts,
//! a destination written as a URL answers with its own host,
//! and a remote name is read with one bounded `git remote get-url --push` query,
//! which applies the rewriting rules of the repository as Git's own push does.
//! The answer is asked for once,
//! before the child starts,
//! and every line the run prints uses it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const naming = ServerNaming.of(command, environment); await naming.ask(); naming.server(); // 'host:port'
//! ```

/// The host a destination names,
/// with no userinfo and no token.
use crate::lfs_output_mask::host_of;
/// The bounded query that answers where a remote name sends a push,
/// and the answer it gives.
use crate::bounded_query::{QueryOutput, query_window, run_bounded_query_awaiting};
/// The grace a stopped query gets.
use crate::bounded_push::STOP_GRACE;
/// Owned OS strings of the child environment.
use std::ffi::OsString;
/// The path the answer is asked in.
use std::path::PathBuf;
/// `Command` describes the run and `CommandArgs` walks its words.
use std::process::{Command, CommandArgs};

/// What:
///  How a wait and a stop name the server they are waiting for:
/// the word the command gave as its destination,
/// the program and directory that can say where that word goes,
/// the environment the answer is asked in,
/// and the answer once it is known.
/// Why:
///   The question costs nothing on a push that lands:
/// it is asked once,
/// before the run starts,
/// and only when a line about a wait or a stop needs it,
/// so a run that never stalls pays one local query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ServerNaming { static of(command, env); static none(env); async ask(); server() }
/// ```
pub struct ServerNaming<'a> {
    /// The word the run gave as its destination,
    /// when it gave one.
    destination: Option<String>,
    /// The Git executable that can say where the destination goes.
    program: PathBuf,
    /// The directory the answer is asked in.
    directory: Option<PathBuf>,
    /// The environment the answer is asked in,
    /// and the one the run itself is given.
    pub environment: &'a [(OsString, OsString)],
    /// Whether the answer has been asked for,
    /// so a line that asks twice pays once.
    asked: bool,
    /// The server;
    /// nothing when the destination names none or the question could not be answered.
    known: Option<String>,
}

impl<'a> ServerNaming<'a> {
    /// A run that names no server,
    /// and never asks.
    pub fn none(environment: &'a [(OsString, OsString)]) -> ServerNaming<'a> {
        return ServerNaming { destination: None, program: PathBuf::new(), directory: None, environment, asked: true, known: None };
    }

    /// What:
    ///  How a run of `command` names the server it waits for,
    /// taken from the command before it starts.
    /// Why:
    ///   The destination word and the program that resolves it are the command's,
    /// and the environment is the one the run was given.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { destination: destinationOf(command), program: command.program, directory: command.cwd, environment, asked: false }
    /// ```
    pub fn of(command: &Command, environment: &'a [(OsString, OsString)]) -> ServerNaming<'a> {
        return ServerNaming {
            destination: destination_of(command),
            program: command.get_program().into(),
            directory: command.get_current_dir().map(std::path::Path::to_path_buf),
            environment,
            asked: false,
            known: None,
        };
    }

    /// What:
    ///  Ask for the server once,
    /// before the run it names starts.
    /// Why:
    ///   The question is a Git query,
    /// and a query awaited from inside the wait loop would make that loop's own future recursive;
    /// asked once before the start,
    /// it costs one local query on a run whose push takes seconds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (!asked) { asked = true; known = await look(); }
    /// ```
    pub async fn ask(&mut self) {
        if self.asked {
            return;
        }
        self.asked = true;
        self.known = self.look().await;
    }

    /// The server the run's destination names,
    /// once it has been asked for.
    pub fn server(&self) -> Option<&str> {
        return self.known.as_deref();
    }

    /// What:
    ///  One bounded query for the destination's push URL,
    /// and the host of it with any userinfo left out.
    /// Why:
    ///   Git answers one location per line,
    /// and the first is the one a push would use;
    /// a destination that is already a URL answers without a query at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const first = answer.stdout.split('\n')[0]; return hostOf(first);
    /// ```
    async fn look(&self) -> Option<String> {
        let destination: &str = self.destination.as_deref()?;
        if let Some(host) = host_of(destination) {
            return Some(host);
        }
        let mut query: Command = Command::new(&self.program);
        query.args(["remote", "get-url", "--push", destination]);
        if let Some(directory) = &self.directory {
            query.current_dir(directory);
        }
        // The answer is bounded as every Git query of the wrapper's own is, by the same window and the same stop.
        let text: String = format!("remote get-url --push {destination}");
        let answer: QueryOutput = run_bounded_query_awaiting(query, text.as_str(), query_window(self.environment), STOP_GRACE, self.environment)
            .await
            .ok()?;
        if answer.code != Some(0) {
            return None;
        }
        let output: String = String::from_utf8_lossy(answer.stdout.as_slice()).into_owned();
        let first: &str = output.lines().next()?.trim();
        return host_of(first);
    }
}

/// What:
///  The word a run gave as its destination:
/// the argument after its subcommand,
/// which is a remote name,
/// a URL or a path.
/// Why:
///   A wait and a stop that name the server it is waiting for are actionable;
/// the word is all the command itself says,
/// and this module turns it into the server when a person reads the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = nonOptionWords(command.args); return words[1];
/// ```
fn destination_of(command: &Command) -> Option<String> {
    let mut arguments = command.get_args();
    // The subcommand (`push`, `fetch`) comes first, and the destination is the word after it.
    let _subcommand: String = first_word(&mut arguments)?;
    return first_word(&mut arguments);
}

/// What:
///  The first word of `arguments` that is not an option,
/// with the value of a `-c` or a `-C` skipped with it.
/// Why:
///   `command_label` names a run by this word and this module reads the word after it,
/// so both read the arguments by one rule,
/// and neither takes the value of an option for a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstWord(args) { for (const [i, word] of args.entries()) { if (['-c','-C'].includes(word)) { i++; continue; } if (!word.startsWith('-')) return word; } }
/// ```
pub fn first_word(arguments: &mut CommandArgs<'_>) -> Option<String> {
    while let Some(argument) = arguments.next() {
        let word: String = argument.to_string_lossy().into_owned();
        if word == "-c" || word == "-C" {
            let _value = arguments.next();
            continue;
        }
        if word.starts_with('-') {
            continue;
        }
        return Some(word);
    }
    return None;
}

/// Server naming is controlled in the test module.
#[cfg(test)]
#[path = "push_server_naming_tests.rs"]
mod tests;

//! What:
//!  Pure decision of the branch-worktree-only policy:
//!  a new branch starts in its own
//!       linked worktree,
//!  not in the current one.
//! Why:
//!  `git branch <name>`,
//!  `git checkout -b`,
//!  `git switch -c` and Git's guess of a remote
//!      branch all create a branch in the current worktree.
//!  The policy rejects them and
//!      points at `git worktree add -b`.
//!  This file decides from the arguments alone,
//!  names
//!      the one repository query the guess needs,
//!  and reads that query's answer.
//!  It starts
//!      no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideBranchWorktree(['switch', '-c', 'topic']) => { kind: 'creates', command: 'switch' }
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The creation facts come from Git's own option tables of the three commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseBranchCreationRegion } from './command_branch_create.ts';
/// ```
use super::command_branch_create::{
    BranchCreationCommand, BranchCreationRegion, branch_creation_command,
    parse_branch_creation_region,
};
use super::command_options::OptionError;
use super::escape_hatch::BRANCH_WORKTREE_ESCAPE_HATCH;
use super::global_arguments::command_tokens;
/// What:
///  `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
///  Sibling
///       the reader might expect:
///  `&str`/`String`,
///  which must be valid UTF-8.
/// Why:
///   A branch name is passed to Git with its exact bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::{OsStr, OsString};

/// What:
///  The finding code of a branch-worktree rejection.
///  `&str` is borrowed text baked
///       into the program.
/// Why:
///   Callers identify the finding by this stable code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BRANCH_CREATION_CODE = 'branch-creation-requires-worktree';
/// ```
pub const BRANCH_CREATION_CODE: &str = "branch-creation-requires-worktree";

/// Ref namespace of local branches.
const LOCAL_BRANCH_PREFIX: &[u8] = b"refs/heads/";

/// Ref namespace of remote-tracking branches.
const REMOTE_BRANCH_PREFIX: &[u8] = b"refs/remotes/";

/// What:
///  The outcome of looking at the argument list only.
///  An `enum` is a closed set of
///       named alternatives;
///  `NeedsRemoteGuess` carries the name Git may guess.
///       `#[derive(...)]` asks the compiler to generate cloning,
///  debug printing and `==`.
/// Why:
///   Explicit creation is decided here;
///  the guess depends on which branches exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BranchWorktreeDecision = { kind: 'pass' } | { kind: 'creates'; command } | { kind: 'needs-remote-guess'; command; target: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BranchWorktreeDecision {
    /// Not a guarded command,
    ///  a form that creates nothing,
    ///  or a region Git refuses.
    Pass,
    /// The command creates a branch explicitly.
    Creates(BranchCreationCommand),
    /// `git switch <target>` or `git checkout <target>`:
    ///  creation depends on the
    /// repository's branches;
    ///  the caller asks and then calls `resolve_remote_guess`.
    NeedsRemoteGuess {
        /// The command being run.
        command: BranchCreationCommand,
        /// The name Git may guess,
        ///  with its exact bytes.
        target: OsString,
    },
}

/// What:
///  Decide from the argument list.
///  `&[OsString]` borrows the arguments,
///  already free
///       of wrapper controls.
/// Why:
///   Only `branch`,
///  `checkout` and `switch` can create a branch outside
///       `git worktree add`.
///  A region Git itself refuses creates nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideBranchWorktree(args: string[]): BranchWorktreeDecision;
/// ```
pub fn decide_branch_worktree(arguments: &[OsString]) -> BranchWorktreeDecision {
    // `let Some((a, b)) = ... else { ... };` unpacks the command word and the tokens after
    // it, or returns when Git runs no subcommand.
    let Some((word, region)) = command_tokens(arguments) else {
        return BranchWorktreeDecision::Pass;
    };
    // `.as_encoded_bytes()` lends the raw bytes of the subcommand word;
    // `let Some(x) = ... else { ... };` unwraps the guarded command or returns.
    let Some(command) = branch_creation_command(word.as_encoded_bytes()) else {
        return BranchWorktreeDecision::Pass;
    };
    // `Result<A, B>` is "either success `A` or failure `B`"; `&[]` is an empty flag list.
    let parsed: Result<BranchCreationRegion, OptionError> =
        parse_branch_creation_region(command, region, &[]);
    // `let Ok(x) = ... else { ... };` unwraps the facts or returns for a refused region.
    let Ok(facts) = parsed else {
        return BranchWorktreeDecision::Pass;
    };
    if facts.creates_branch {
        return BranchWorktreeDecision::Creates(command);
    }
    // `if let Some(token) = ...` runs only when Git may guess a remote branch.
    if let Some(token) = facts.implicit_creation_target {
        return BranchWorktreeDecision::NeedsRemoteGuess {
            command,
            // `.clone()` copies the name's bytes into the decision.
            target: region[token].clone(),
        };
    }
    return BranchWorktreeDecision::Pass;
}

/// What:
///  The subcommand word of a guarded command.
///  `&'static str` is text baked into the
///       program.
/// Why:
///   The rejection names the command the caller typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = subcommand; // the union member already is the string
/// ```
fn command_word(command: BranchCreationCommand) -> &'static str {
    // `match` picks one arm per variant; the compiler refuses a forgotten variant.
    match command {
        BranchCreationCommand::Branch => return "branch",
        BranchCreationCommand::Checkout => return "checkout",
        BranchCreationCommand::Switch => return "switch",
    }
}

/// What:
///  Build the rejection text.
///  `Option<&OsStr>` is "a borrowed branch name or
///       nothing";
///  `String` is the owned result.
/// Why:
///   The text names the command,
///  the guessed branch when there is one,
///  the worktree
///       form to use instead,
///  and the one-invocation bypass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function branchCreationMessage({ subcommand, target }): string;
/// ```
pub fn branch_creation_message(command: BranchCreationCommand, target: Option<&OsStr>) -> String {
    // `String::new()` is empty owned text; `mut` allows filling it below.
    let mut target_clause: String = String::new();
    // `if let Some(name) = target` runs only for a guessed branch.
    if let Some(name) = target {
        // `.to_string_lossy()` renders the name as text, replacing undecodable bytes.
        target_clause = format!(" for {}", name.to_string_lossy());
    }
    return format!(
        "cli-git: git {}{target_clause} branch creation is rejected in the current worktree. \
         Use `git worktree add -b <branch> <path> [<start-point>]` so new branch work starts \
         in its own checkout, or pass {BRANCH_WORKTREE_ESCAPE_HATCH} to bypass for this \
         invocation.",
        command_word(command)
    );
}

/// What:
///  Build the real-Git argument list of the guess query:
///  the caller's global
///       options,
///  then one `for-each-ref` over the local branch of that name and the
///       remote branches of that name.
///  `Vec<OsString>` is the owned result.
/// Why:
///   One listing answers both questions the guess depends on,
///  so the check starts one
///       Git process.
///  The query must inspect the repository the command will run in,
///  so
///       the global options are forwarded unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...pre, 'for-each-ref', '--format=%(refname)', `refs/heads/${target}`, `refs/remotes/*/${target}`];
/// ```
pub fn remote_guess_query_arguments(global_prefix: &[OsString], target: &OsStr) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into an owned, growable list.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("for-each-ref"));
    arguments.push(OsString::from("--format=%(refname)"));
    let mut local: OsString = OsString::from("refs/heads/");
    // `.push` appends the name's bytes unchanged.
    local.push(target);
    arguments.push(local);
    let mut remote: OsString = OsString::from("refs/remotes/*/");
    remote.push(target);
    arguments.push(remote);
    return arguments;
}

/// Named predicate for splitting output into lines;
///  `&u8` borrows one byte.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Named predicate for finding the end of a remote's name.
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Whether a listed ref is `refs/remotes/<remote>/<target>` with a remote name of
///       one path segment.
///  `&[u8]` borrows the ref name and the branch name.
/// Why:
///   The query's patterns can also list refs that merely start with the name,
///  so
///       each line is compared exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRemote = line.startsWith('refs/remotes/') && line.slice(line.indexOf('/', 13) + 1) === target;
/// ```
fn is_remote_branch_of(line: &[u8], target: &[u8]) -> bool {
    // `.strip_prefix` is `Some(rest)` when the bytes start with the prefix.
    let Some(rest) = line.strip_prefix(REMOTE_BRANCH_PREFIX) else {
        return false;
    };
    // `.iter().position(f)` is the index of the first byte for which `f` is true.
    let Some(slash) = rest.iter().position(is_slash) else {
        return false;
    };
    // `&rest[n..]` borrows the bytes after the remote's name and its slash.
    return &rest[slash + 1..] == target;
}

/// What:
///  Read the guess query's listing:
///  no local branch of that name,
///  and exactly one
///       remote branch of that name.
///  `bool` is true or false.
/// Why:
///   That is when `git switch <name>` and `git checkout <name>` create a local branch.
///       A remote's `HEAD` pointer is not a branch and is never guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const creates = !lines.includes(`refs/heads/${target}`) && remoteMatches.length === 1 && target !== 'HEAD';
/// ```
pub fn remote_guess_creates_branch(stdout: &[u8], target: &[u8]) -> bool {
    if target == b"HEAD" {
        return false;
    }
    // `usize` counts the remote branches of that name.
    let mut remotes: usize = 0;
    // `.split(f)` cuts the bytes at each line break.
    for line in stdout.split(is_line_feed) {
        if line.strip_prefix(LOCAL_BRANCH_PREFIX) == Some(target) {
            return false;
        }
        if is_remote_branch_of(line, target) {
            remotes += 1;
        }
    }
    return remotes == 1;
}

/// Decisions,
///  the message,
///  and the guess query with real Git 2.56.0 controls.
#[cfg(test)]
#[path = "rule_branch_worktree_tests.rs"]
mod tests;

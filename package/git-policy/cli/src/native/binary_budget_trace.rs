//! Reads a `strace` trace of one wrapped commit into the process starts and the fsync-class calls of each process.
//! Why:
//!  A budget test needs to say which Git subcommands started, how many times, and which process made each sync call,
//!  so a regression names its cause.
//!  The method is the one of the scratch scripts that measured the commit: the trace lists `execve` calls, the
//!  `clone`-family calls that give a thread or a new process its parent, and the sync calls;
//!  a sync call belongs to the program its process last started, and a thread to the process that made it.

use std::collections::{BTreeMap, HashMap};

/// The syscalls the trace is asked for, as the value of `strace --trace=`.
pub(super) const TRACED_CALLS: &str = "execve,clone,clone3,fork,vfork,fsync,fdatasync,syncfs,sync_file_range";

/// How many parent links a process may be followed through before the trace is taken to be corrupt.
const MAX_PARENT_HOPS: usize = 64;

/// What a process is, for counting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Program {
    /// The test's own `timeout` that bounds the command; it is not part of the commit.
    Harness,
    /// The wrapper under test.
    Wrapper,
    /// A start of Git, by the subcommand it ran (with its first operand for the commands that need one).
    Git(String),
    /// A start of any other program, by file name.
    Other(String),
}

/// One line of the trace that matters.
#[derive(Debug)]
enum Event {
    /// A process replaced its program.
    Exec { tid: u32, program: Program, command: String },
    /// A process or thread made a new one.
    Spawn { parent: u32, child: u32 },
    /// A sync-class call.
    Sync { tid: u32 },
}

/// The counts one trace shows.
#[derive(Debug, Default)]
pub(super) struct Counts {
    /// Starts of Git by subcommand.
    pub(super) git_starts: BTreeMap<String, usize>,
    /// Starts of other programs by file name, the harness and the wrapper itself left out.
    pub(super) other_starts: BTreeMap<String, usize>,
    /// Sync-class calls made by the wrapper process and its threads.
    pub(super) wrapper_syncs: usize,
    /// Sync-class calls made by Git processes, by the subcommand that made them.
    pub(super) git_syncs: BTreeMap<String, usize>,
    /// Sync-class calls made by other programs, by file name.
    pub(super) other_syncs: BTreeMap<String, usize>,
    /// Every started Git command line, in trace order, for failure messages.
    pub(super) commands: Vec<String>,
}

/// Decode a quoted `strace` string at the start of `text`; returns the string and the text after it and any `...`.
fn quoted(text: &str) -> Option<(String, &str)> {
    let body: &str = text.strip_prefix('"')?;
    let mut decoded: String = String::new();
    let mut escaped: bool = false;
    for (position, character) in body.char_indices() {
        if escaped {
            decoded.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            let rest: &str = &body[position + 1..];
            return Some((decoded, rest.strip_prefix("...").unwrap_or(rest)));
        } else {
            decoded.push(character);
        }
    }
    return None;
}

/// The arguments of an `execve` call's `argv` array, from the text just after `[`.
fn arguments(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest: &str = text;
    while let Some((argument, after)) = quoted(rest) {
        found.push(argument);
        rest = after.strip_prefix(", ").unwrap_or(after);
    }
    return found;
}

/// Whether `token` is a Git global option that takes the next token as its value.
fn takes_value(token: &str) -> bool {
    return matches!(token, "-c" | "-C" | "--git-dir" | "--work-tree" | "--namespace" | "--config-env" | "--super-prefix");
}

/// The subcommands whose first operand names what they do, so a start reads `hook run` rather than `hook`.
const COMPOUND: [&str; 3] = ["hook", "maintenance", "multi-pack-index"];

/// The name a Git start counts under: its subcommand, plus the operand that completes the compound ones.
fn git_key(arguments_of_start: &[String]) -> String {
    let first: &str = arguments_of_start.first().map_or("git", String::as_str);
    let program_name: &str = first.rsplit('/').next().unwrap_or(first);
    if let Some(dashed) = program_name.strip_prefix("git-") {
        return String::from(dashed);
    }
    let mut operands: Vec<&str> = Vec::new();
    let mut index: usize = 1;
    while index < arguments_of_start.len() {
        let token: &str = arguments_of_start[index].as_str();
        if takes_value(token) {
            index += 2;
        } else {
            if !token.starts_with('-') {
                operands.push(token);
            }
            index += 1;
        }
    }
    let Some(subcommand) = operands.first() else {
        return String::from("(no subcommand)");
    };
    if COMPOUND.contains(subcommand) {
        if let Some(operand) = operands.get(1) {
            return format!("{subcommand} {operand}");
        }
    }
    return String::from(*subcommand);
}

/// What the program started at `path` is, given where the fixture keeps the wrapper and which file is real Git.
fn program_of(path: &str, arguments_of_start: &[String], fixture_root: &str, real_git: &str) -> Program {
    if path.starts_with(fixture_root) {
        return Program::Wrapper;
    }
    if path == real_git || path.contains("git-core") {
        return Program::Git(git_key(arguments_of_start));
    }
    let name: &str = path.rsplit('/').next().unwrap_or(path);
    if name == "timeout" {
        return Program::Harness;
    }
    return Program::Other(String::from(name));
}

/// The result of a call line: the text after its last `) = `, when the call returned.
fn result_of(rest: &str) -> Option<&str> {
    return rest.rsplit_once(") = ").map(|(_call, result)| { return result; });
}

/// The event a trace line shows, if it is one that matters.
fn parse_line(line: &str, fixture_root: &str, real_git: &str) -> Option<Event> {
    let (id, remainder) = line.split_once(char::is_whitespace)?;
    let tid: u32 = id.parse().ok()?;
    let call: &str = remainder.trim_start();
    if let Some(execve) = call.strip_prefix("execve(") {
        // A failed start (a search of PATH that missed) is not a start.
        if result_of(call).is_some_and(|result| { return result.starts_with('-'); }) {
            return None;
        }
        let (path, after) = quoted(execve)?;
        let argv: Vec<String> = arguments(after.strip_prefix(", [")?);
        let program: Program = program_of(path.as_str(), argv.as_slice(), fixture_root, real_git);
        return Some(Event::Exec { tid, program, command: argv.join(" ") });
    }
    let spawning: bool = ["clone(", "clone3(", "fork(", "vfork(", "<... clone resumed>", "<... clone3 resumed>",
        "<... fork resumed>", "<... vfork resumed>"]
        .iter()
        .any(|prefix| { return call.starts_with(prefix); });
    if spawning {
        let result: &str = result_of(call)?;
        let digits: String = result.chars().take_while(char::is_ascii_digit).collect();
        let child: u32 = digits.parse().ok()?;
        return Some(Event::Spawn { parent: tid, child });
    }
    let syncing: bool = ["fsync(", "fdatasync(", "syncfs(", "sync_file_range(", "<... fsync resumed>",
        "<... fdatasync resumed>", "<... syncfs resumed>", "<... sync_file_range resumed>"]
        .iter()
        .any(|prefix| { return call.starts_with(prefix); });
    if syncing {
        // The call may be printed in two halves; only the half that begins it counts.
        if call.starts_with("<...") {
            return None;
        }
        return Some(Event::Sync { tid });
    }
    return None;
}

/// The program that thread or process `tid` was running at event `before`, found by following parents when it has
///  not started one itself.
fn program_at<'a>(
    starts: &'a HashMap<u32, Vec<(usize, Program)>>,
    parents: &HashMap<u32, u32>,
    tid: u32,
    before: usize,
) -> &'a Program {
    let mut current: u32 = tid;
    for _hop in 0..MAX_PARENT_HOPS {
        if let Some(known) = starts.get(&current).and_then(|list| {
            return list.iter().rev().find(|(position, _program)| { return *position < before; });
        }) {
            return &known.1;
        }
        let Some(parent) = parents.get(&current) else {
            panic!("a sync call by thread {current} cannot be attributed: no program start and no parent in the trace");
        };
        current = *parent;
    }
    panic!("thread {tid}: parent chain longer than {MAX_PARENT_HOPS}, so the trace is corrupt");
}

/// Count the starts and the sync calls in `trace`, the text of a `strace --follow-forks` output file.
///  `fixture_root` marks the wrapper's files and `real_git` the Git executable.
pub(super) fn count(trace: &str, fixture_root: &str, real_git: &str) -> Counts {
    let events: Vec<Event> = trace
        .lines()
        .filter_map(|line| { return parse_line(line, fixture_root, real_git); })
        .collect();
    // A process's first events can be printed before its parent's `clone` returns, so parents are gathered first.
    let mut parents: HashMap<u32, u32> = HashMap::new();
    let mut starts: HashMap<u32, Vec<(usize, Program)>> = HashMap::new();
    for (position, event) in events.iter().enumerate() {
        match event {
            Event::Spawn { parent, child } => {
                parents.entry(*child).or_insert(*parent);
            }
            Event::Exec { tid, program, .. } => {
                starts.entry(*tid).or_default().push((position, program.clone()));
            }
            Event::Sync { .. } => {}
        }
    }
    let mut counts: Counts = Counts::default();
    for (position, event) in events.iter().enumerate() {
        match event {
            Event::Exec { program, command, .. } => match program {
                Program::Git(key) => {
                    *counts.git_starts.entry(key.clone()).or_default() += 1;
                    counts.commands.push(command.clone());
                }
                Program::Other(name) => *counts.other_starts.entry(name.clone()).or_default() += 1,
                Program::Harness | Program::Wrapper => {}
            },
            Event::Sync { tid } => match program_at(&starts, &parents, *tid, position) {
                Program::Wrapper => counts.wrapper_syncs += 1,
                Program::Git(key) => *counts.git_syncs.entry(key.clone()).or_default() += 1,
                Program::Other(name) => *counts.other_syncs.entry(name.clone()).or_default() += 1,
                Program::Harness => panic!("the harness made a sync call, so the trace is not of the command alone"),
            },
            Event::Spawn { .. } => {}
        }
    }
    return counts;
}

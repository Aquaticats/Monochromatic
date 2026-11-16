//! The scan that keeps blocked code off the runtime's workers.
//! What:
//!  A check of the crate's own source text that fails when release code starts a thread,
//!  pauses on a blocking clock,
//!  starts a child outside the retried start,
//!  or does blocking storage work or a blocking wait inside a task the runtime polls.
//! Why:
//!  The owner's move to tokio is "Full on tokio.
//! Multi-thread."
//! (`doc/planning/git-policy-cli-full-tokio.md`),
//! and the owner's rules that
//!  bind it are "nothing should ever wait forever or indefinitely" and "Commits should never fail."
//!  A worker that blocks stops the tasks that would end its wait,
//!  which is how a wait with a timer in it still outlives the timer.
//!  The plan's step T13 names three of these guards (no `std::thread`,
//!  no `std::process::Command` outside the guarded
//!  start,
//!  no `thread::sleep`),
//!  and the runtime's own rule adds the fourth (a blocking call reachable from a task).
//!  Each finding is either fixed or named in
//! `blocking_code_off_the_runtime_rules_tests.rs` with the reason it stays and what removes it,
//!  so the list shrinks as
//!  the move lands and a new one fails the suite.
//!
//! The scan is textual,
//! and it says where it is coarse:
//! it reads one file at a time,
//!  it follows no call at all,
//!  not even one to a helper of the same file,
//!  and a fork of one line across two lines is read
//!  only where the next line is joined to it.
//!  Everything else the crate can see in one file is covered:
//!  the file's code lines,
//!  the
//!  bodies of functions that build a `Command`,
//!  and the bodies of `async fn`s outside the closures that hand their work to the
//!  blocking pool.
//!  Comments and the characters inside string,
//!  character and raw string literals are replaced by spaces before any rule reads a line,
//!  so a `"{{"` in a format
//!  text neither opens a block nor hides one,
//!  a storage call a literal or a comment names is never a finding,
//!  and a file whose braces still do not balance is an error rather than a file the scan skips.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/blocking_code_off_the_runtime_tests.ts: one check over the crate's own text.
//! ```

use std::path::Path;

/// The forbidden work and the justified exceptions.
#[path = "blocking_code_off_the_runtime_rules_tests.rs"]
mod rules;
use rules::{ALLOWED, Allowed, BLOCKING_WAITS, Place, REACHED_FROM_A_TASK_KNOWN, RULES, Rule, STORAGE_CALLS, holds_any};

use crate::source_scan_tests::{CrateRoot, classified_sources, real_roots, source_root};

/// Controls of this scan on fixture trees.
#[path = "blocking_code_off_the_runtime_fixture_tests.rs"]
mod fixtures;

/// One rule match:
/// the source file as a path relative to `src/native`,
/// the rule that matched,
/// and the line number.
type Finding = (String, &'static str, usize);

/// The lines of one function,
/// with the next line joined so a call split over two lines still reads.
type BodyLines = Vec<(usize, String, String)>;

/// What:
///  `struct Function { ... }` is one function of a source file:
/// its name,
/// whether it is `async`,
/// whether its body
///  builds or names a `Command`,
/// and the lines of its body that run on the task the runtime polls.
/// Why:
///  The three rules that are not about a whole file need to know which function a line is in,
/// and an `async fn`
///  is the smallest unit the compiler lets one task poll.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Function = { name: string; isAsync: boolean; holdsACommand: boolean; taskLines: BodyLines }
/// ```
#[derive(Debug)]
struct Function {
    /// The name the signature gives.
    name: String,
    /// Whether the signature says `async`.
    is_async: bool,
    /// Whether the body builds or names `std::process::Command`,
    /// which is where a start belongs to a command.
    holds_a_command: bool,
    /// The body's lines that are not inside a closure handed to the blocking pool.
    task_lines: BodyLines,
}

/// The release source files under `root`,
/// as `/`-separated paths relative to it:
/// every file the module tree reaches from a crate root through
///  declarations that no `cfg(test)` gates.
fn release_sources_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in classified_sources(root, roots)? {
        if !file.test_only {
            found.push(file.path);
        }
    }
    return Ok(found);
}

/// The lines of a source file that are code,
/// with the following line joined so a call split over two lines still reads.
fn code_lines(read: &ReadCode) -> BodyLines {
    let mut code: BodyLines = Vec::new();
    for (index, line) in read.masked.iter().enumerate() {
        let trimmed: &str = line.trim_start();
        // A line that is only a comment or the inside of a literal runs nothing.
        if trimmed.is_empty() {
            continue;
        }
        let next: &str = read.masked.get(index + 1).map_or("", |following| {
            return following.trim_start();
        });
        code.push((index + 1, trimmed.to_owned(), format!("{trimmed} {next}")));
    }
    return code;
}

/// Where the reader of one source file is.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Scanner {
    /// Ordinary code.
    Code,
    /// A `//` comment,
    /// to the end of the line.
    LineComment,
    /// A `/* */` comment,
    /// with how many are open.
    BlockComment(usize),
    /// A `"..."` string.
    Text,
    /// A `'c'` character literal.
    Character,
    /// An `r"..."`,
    /// `r#"..."#` or `br#"..."#` raw string,
    /// with how many `#` it opened with.
    Raw(usize),
}

/// What:
///  One source file read as code:
///  its lines with every comment and every character inside a literal replaced by spaces,
///  and the braces each line opens and closes.
/// Why:
///  A rule that asks whether a line runs a call must read the code of that line:
///  a storage call a string literal or a comment names is text,
///  and reporting it rejects release source that runs nothing.
///  A replaced character keeps its place,
///  so every line number and every column stays where the file put it and a finding still names the line a person
///  reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadCode = {
///   masked: string[]
///   braces: [number, number][]
/// }
/// ```
struct ReadCode {
    /// One entry per line of the file:
    /// the line's code,
    /// with a space wherever a comment or a literal put a character.
    masked: Vec<String>,
    /// For each line:
    /// the `{` and `}` that are code rather than comment or literal text.
    braces: Vec<(usize, usize)>,
}

/// What:
///  `blank` puts one space in the masked text for each character a comment or a literal held.
/// Why:
///  A replaced character keeps its place,
///  so the line break it sat beside stays where the file put it and a finding still names the line a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blank(masked: string[], count: number): void
/// ```
fn blank(masked: &mut Vec<char>, count: usize) {
    for _ in 0..count {
        masked.push(' ');
    }
}

/// What:
///  `fn read_the_code_of(text: &str) -> Result<ReadCode, String>` reads one source file once:
///  its code without comments or literal characters,
///  and the braces that code holds.
/// Why:
///  A function's body is found by its braces,
/// and the crate writes `format!("{{\"schemaVersion\":{SCHEMA_VERSION}")`,
/// `b"@{"` and
///  `'\u{8}'`,
///  whose braces close nothing;
///  counting them would end a body early or lose the end of the file.
///  The same reading decides what a rule may match,
///  so the two can never disagree about where a literal ends.
///  An unterminated literal or block comment means the file cannot be read,
/// which is an error and not a file
///  the scan skips.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheCodeOf(text: string): ReadCode  // throws Error
/// ```
fn read_the_code_of(text: &str) -> Result<ReadCode, String> {
    let characters: Vec<char> = text.chars().collect();
    let mut masked: Vec<char> = Vec::with_capacity(characters.len());
    let mut counts: Vec<(usize, usize)> = vec![(0, 0)];
    let mut scanner: Scanner = Scanner::Code;
    let mut index: usize = 0;
    while index < characters.len() {
        let character: char = characters[index];
        let next: Option<char> = characters.get(index + 1).copied();
        if character == '\n' {
            masked.push('\n');
            counts.push((0, 0));
            if scanner == Scanner::LineComment {
                scanner = Scanner::Code;
            }
            index += 1;
            continue;
        }
        match scanner {
            Scanner::Code => match character {
                '/' if next == Some('/') => {
                    scanner = Scanner::LineComment;
                    blank(&mut masked, 2);
                    index += 2;
                }
                '/' if next == Some('*') => {
                    scanner = Scanner::BlockComment(1);
                    blank(&mut masked, 2);
                    index += 2;
                }
                '"' => {
                    scanner = Scanner::Text;
                    blank(&mut masked, 1);
                    index += 1;
                }
                'r' | 'b' => {
                    let mut hashes: usize = 0;
                    let mut cursor: usize = index + 1;
                    while characters.get(cursor) == Some(&'#') {
                        hashes += 1;
                        cursor += 1;
                    }
                    match characters.get(cursor) {
                        Some('"') => {
                            scanner = if character == 'r' { Scanner::Raw(hashes) } else { Scanner::Text };
                            blank(&mut masked, cursor + 1 - index);
                            index = cursor + 1;
                        }
                        Some('\'') if character == 'b' => {
                            scanner = Scanner::Character;
                            blank(&mut masked, cursor + 1 - index);
                            index = cursor + 1;
                        }
                        _ => {
                            masked.push(character);
                            index += 1;
                        }
                    }
                }
                '\'' => {
                    // A character literal is one character or one escape and then its own quote; a lifetime is
                    // neither, and the escape `\u{8}` carries braces of its own.
                    let mut cursor: usize = index + 1;
                    let mut closed: bool = false;
                    if characters.get(cursor) != Some(&'\\') && characters.get(cursor + 1) == Some(&'\'') {
                        cursor += 2;
                        closed = true;
                    } else if characters.get(cursor) == Some(&'\\') {
                        cursor += 1;
                        let mut inner_braces: usize = 0;
                        while let Some(inner) = characters.get(cursor).copied() {
                            if inner == '\n' {
                                break;
                            }
                            if inner == '{' {
                                inner_braces += 1;
                            }
                            if inner == '}' {
                                inner_braces = inner_braces.saturating_sub(1);
                            }
                            if inner == '\'' && inner_braces == 0 {
                                cursor += 1;
                                closed = true;
                                break;
                            }
                            cursor += 1;
                        }
                    }
                    let consumed: usize = if closed { cursor - index } else { 1 };
                    blank(&mut masked, consumed);
                    index += consumed;
                }
                '{' => {
                    if let Some(line) = counts.last_mut() {
                        line.0 += 1;
                    }
                    masked.push(character);
                    index += 1;
                }
                '}' => {
                    if let Some(line) = counts.last_mut() {
                        line.1 += 1;
                    }
                    masked.push(character);
                    index += 1;
                }
                _ => {
                    masked.push(character);
                    index += 1;
                }
            },
            Scanner::Text => match character {
                // A line continuation keeps the string open across the line break, and the break still starts a line.
                '\\' if next == Some('\n') => {
                    blank(&mut masked, 1);
                    index += 1;
                }
                '\\' => {
                    blank(&mut masked, 2);
                    index += 2;
                }
                '"' => {
                    scanner = Scanner::Code;
                    blank(&mut masked, 1);
                    index += 1;
                }
                _ => {
                    blank(&mut masked, 1);
                    index += 1;
                }
            },
            Scanner::Character => match character {
                '\\' => {
                    blank(&mut masked, 2);
                    index += 2;
                }
                '\'' => {
                    scanner = Scanner::Code;
                    blank(&mut masked, 1);
                    index += 1;
                }
                _ => {
                    blank(&mut masked, 1);
                    index += 1;
                }
            },
            Scanner::Raw(hashes) => {
                // A raw string ends at the first `"` its own count of `#` follows; any other `"` is its text.
                let mut cursor: usize = index + 1;
                let mut seen: usize = 0;
                while seen < hashes && characters.get(cursor) == Some(&'#') {
                    seen += 1;
                    cursor += 1;
                }
                if character == '"' && seen == hashes {
                    scanner = Scanner::Code;
                    blank(&mut masked, cursor - index);
                    index = cursor;
                } else {
                    blank(&mut masked, 1);
                    index += 1;
                }
            }
            Scanner::LineComment => {
                blank(&mut masked, 1);
                index += 1;
            }
            Scanner::BlockComment(depth) => {
                if character == '*' && next == Some('/') {
                    scanner = if depth <= 1 { Scanner::Code } else { Scanner::BlockComment(depth - 1) };
                    blank(&mut masked, 2);
                    index += 2;
                } else if character == '/' && next == Some('*') {
                    scanner = Scanner::BlockComment(depth + 1);
                    blank(&mut masked, 2);
                    index += 2;
                } else {
                    blank(&mut masked, 1);
                    index += 1;
                }
            }
        }
    }
    if scanner != Scanner::Code && scanner != Scanner::LineComment {
        return Err(String::from("a literal or a block comment is not closed, so this file cannot be read"));
    }
    let read_text: String = masked.into_iter().collect();
    let masked_lines: Vec<String> = read_text.split('\n').map(String::from).collect();
    return Ok(ReadCode { masked: masked_lines, braces: counts });
}

/// The braces of each line of `text` that are code rather than comment or literal text.
fn braces_outside_literals(text: &str) -> Result<Vec<(usize, usize)>, String> {
    return Ok(read_the_code_of(text)?.braces);
}

/// The `async` flag of a line that starts a function signature,
/// and the function's name;
/// `None` on every other line.
fn signature_of(line: &str) -> Option<(String, bool)> {
    let trimmed: &str = line.trim_start();
    let at: usize = trimmed.find("fn ")?;
    let prefix: &str = &trimmed[..at];
    let words: Vec<&str> = prefix.split_whitespace().collect();
    let allowed: [&str; 4] = ["pub", "async", "unsafe", "const"];
    if !words.iter().all(|word| {
        return allowed.contains(word) || word.starts_with("pub(");
    }) {
        return None;
    }
    let rest: &str = &trimmed[at + "fn ".len()..];
    let name: String = rest
        .chars()
        .take_while(|letter| {
            return letter.is_alphanumeric() || *letter == '_';
        })
        .collect();
    if name.is_empty() {
        return None;
    }
    return Some((name, words.contains(&"async")));
}

/// The closures of a file that hand their body to the blocking pool,
/// whose work is off the runtime by design.
const POOL_CALLS: &[&str] = &[
    "spawn_blocking(",
    "run_blocking_job(",
    "run_blocking_job_within_the_budget(",
    "run_awaiting_blocking_job(",
    // The service's own door: a job of the pool, awaited under a ceiling (`service_file_work_off_the_workers.rs`).
    "off_the_workers(",
    "run_a_job_once_awaited_no_longer_than(",
    "run_a_job_awaited_no_longer_than(",
];

/// Whether a line names a call that hands work to the blocking pool,
/// so the closure that follows is off the runtime even when its brace is on a later line.
fn names_a_pool_call(line: &str) -> bool {
    return POOL_CALLS.iter().any(|call_name| { return line.contains(call_name); });
}

/// Whether a line hands a closure to the blocking pool,
/// so the closure's body is off the runtime.
fn opens_a_pool_body(line: &str) -> bool {
    return POOL_CALLS.iter().any(|call_name| {
        return line.contains(call_name) && line.contains('{');
    });
}

/// What:
///  `fn functions_in(text: &str) -> Result<Vec<Function>, String>` splits one source file into its functions.
/// Why:
///  A rule that asks what runs on a task,
/// or where a `Command` is built,
/// needs the body of each function and
///  nothing of its neighbours;
///  a file whose braces do not balance is an error rather than a file the scan silently skips.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function functionsIn(text: string): Function[]  // throws Error
/// ```
fn functions_in(text: &str) -> Result<Vec<Function>, String> {
    let read: ReadCode = read_the_code_of(text)?;
    let code: BodyLines = code_lines(&read);
    let counts: Vec<(usize, usize)> = read.braces;
    let mut found: Vec<Function> = Vec::new();
    let mut depth: usize = 0;
    let mut building: Option<(String, bool, usize, bool, usize)> = None;
    for (index, (opened, closed)) in counts.iter().enumerate() {
        let number: usize = index + 1;
        // A signature is code, so a line that only a comment or a literal holds is never one.
        let line: &str = read.masked.get(index).map_or("", |masked| {
            return masked.as_str();
        });
        let signature: Option<(String, bool)> = if building.is_none() { signature_of(line) } else { None };
        if let Some((name, is_async)) = signature {
            building = Some((name, is_async, depth, false, number));
        }
        depth += opened;
        depth = depth.saturating_sub(*closed);
        let Some((_name, _is_async, starts_at, opened_body, _first_line)) = building.as_mut() else {
            continue;
        };
        if depth > *starts_at || *opened > 0 {
            *opened_body = true;
        }
        let ended: bool = (*opened_body && depth == *starts_at) || (line.trim_end().ends_with(';') && depth == *starts_at);
        if !ended {
            continue;
        }
        let (name, is_async, _, _, first_line) = building.take().expect("the function being built");
        let inside: BodyLines = code
            .iter()
            .filter(|(line_number, _, _)| {
                return *line_number >= first_line && *line_number <= number;
            })
            .cloned()
            .collect();
        found.push(function_of(name, is_async, inside)?);
    }
    if building.is_some() || depth != 0 {
        return Err(String::from("the braces of this file do not balance, so its functions cannot be read"));
    }
    return Ok(found);
}

/// What:
///  `fn function_of(name: String, is_async: bool, inside: BodyLines) -> Function` is one function,
///  with the lines of its body that run on the task the runtime polls.
/// Why:
///  A closure handed to the blocking pool is off the runtime,
/// so its lines are not the task's;
///  every other line of an `async fn` is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function functionOf(name: string, isAsync: boolean, inside: BodyLines): Function
/// ```
fn function_of(name: String, is_async: bool, inside: BodyLines) -> Result<Function, String> {
    let joined_text: String = inside
        .iter()
        .map(|(_, line, _)| {
            return line.as_str();
        })
        .collect::<Vec<&str>>()
        .join("\n");
    let counts: Vec<(usize, usize)> = braces_outside_literals(joined_text.as_str())?;
    let mut depth: usize = 0;
    let mut pool_from: Option<usize> = None;
    // What: A pool call whose closure opens on a line after the call itself.
    // Why:  `run_a_job_once_awaited_no_longer_than(ceiling, move || {` and its longer spellings put the brace on a
    // line of their own,
    //  and a closure that was not recognised as the pool's counted its file work as the task's: a guard that read
    // only the call's own line found 137 tasks whose work was already off the worker.
    let mut a_pool_body_is_coming: bool = false;
    let mut task_lines: BodyLines = Vec::new();
    for (index, (number, line, joined)) in inside.iter().enumerate() {
        let (opened, closed) = counts.get(index).copied().unwrap_or((0, 0));
        let trimmed: &str = line.trim_start();
        // What: The signature is not the body.
        // Why:  It names the function, and a rule that reads a call reads the signature as a call to itself.
        if index == 0 {
            depth += opened;
            depth = depth.saturating_sub(closed);
            continue;
        }
        depth += opened;
        depth = depth.saturating_sub(closed);
        let already: bool = pool_from.is_some();
        if !already && opens_a_pool_body(trimmed) {
            pool_from = Some(depth);
            a_pool_body_is_coming = false;
            continue;
        }
        if !already && !a_pool_body_is_coming && names_a_pool_call(trimmed) {
            a_pool_body_is_coming = true;
            continue;
        }
        if !already && a_pool_body_is_coming && opened > 0 {
            pool_from = Some(depth);
            a_pool_body_is_coming = false;
            continue;
        }
        if already {
            // The pool's closure closed on this line, so the task's own work starts again here.
            let left_the_pool: bool = pool_from.is_some_and(|pool_depth| {
                return depth < pool_depth;
            });
            if left_the_pool {
                pool_from = None;
                task_lines.push((*number, line.clone(), joined.clone()));
            }
            continue;
        }
        task_lines.push((*number, line.clone(), joined.clone()));
    }
    let holds_a_command: bool = inside.iter().any(|(_, _, joined)| {
        return joined.contains("Command");
    });
    return Ok(Function { name, is_async, holds_a_command, task_lines });
}

/// What:
///  The rule id of a blocking call reached from a task through the crate's own plain functions.
/// Why:
///  A task that calls a plain function which waits on a file holds the worker exactly as if it had written the call
/// itself,
///  and the rules that read one line cannot see through the call
///  (the third review of this branch, finding 4: a `canonicalize` two plain calls below an `async fn`).
const REACHED_FROM_A_TASK: &str = "blocking-function-on-a-task";

/// What:
///  Every `(relative path, rule id, line number)` the rules find in the release sources of `root`.
fn findings_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<Finding>, String> {
    let mut found: Vec<Finding> = Vec::new();
    let sources: Vec<String> = release_sources_in(root, roots)?;
    // What: The functions of the whole crate, read once, for the rule that looks through a call.
    // Why:  That rule needs the body of a function of another file as much as of the same one.
    let mut every_function: Vec<(String, Function)> = Vec::new();
    for name in &sources {
        let text: String = std::fs::read_to_string(root.join(name.as_str()))
            .map_err(|error| return format!("cannot read {name}: {error}"))?;
        for function in functions_in(text.as_str()).map_err(|error| return format!("{name}: {error}"))? {
            every_function.push((name.clone(), function));
        }
    }
    found.extend(through_the_crates_own_functions(every_function.as_slice()));
    for name in sources {
        let text: String = std::fs::read_to_string(root.join(name.as_str()))
            .map_err(|error| return format!("cannot read {name}: {error}"))?;
        let read: ReadCode = read_the_code_of(text.as_str()).map_err(|error| return format!("{name}: {error}"))?;
        let functions: Vec<Function> = functions_in(text.as_str()).map_err(|error| return format!("{name}: {error}"))?;
        let mut places: Vec<(Place, BodyLines)> = Vec::new();
        places.push((Place::Everywhere, code_lines(&read)));
        let command_lines: BodyLines = functions
            .iter()
            .filter(|function| {
                return function.holds_a_command;
            })
            .flat_map(|function| {
                return function.task_lines.iter().cloned();
            })
            .collect();
        places.push((Place::InACommandFunction, command_lines));
        let task_lines: BodyLines = functions
            .iter()
            .filter(|function| {
                return function.is_async;
            })
            .flat_map(|function| {
                return function.task_lines.iter().cloned();
            })
            .collect();
        places.push((Place::OnATask, task_lines));
        for (place, lines) in places {
            for (number, line, joined) in lines {
                for rule in RULES {
                    if rule.place != place {
                        continue;
                    }
                    if (rule.matches)(line.as_str(), joined.as_str()) {
                        found.push((name.clone(), rule.id, number));
                    }
                }
            }
        }
    }
    return Ok(found);
}

/// The findings in this crate's own release sources.
fn findings() -> Vec<Finding> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's release sources");
}

/// What:
///  The findings of a blocking call reached from an `async` body through the crate's own plain functions,
///  each as the file of the task, the rule's id and the line of the call in that task.
/// Why:
///  A rule that reads one line sees `canonicalize` in a plain function and says nothing about the task three calls
/// above it,
///  which is how a `std::fs::canonicalize` reached from an `async fn` passed this scan
///  (the third review of this branch, finding 4).
///  A function is blocking when its body reads or writes a file or waits on a thread,
/// or when it calls one that is, to as many steps as the crate's own calls go.
///  A closure handed to the blocking pool is off the worker, so a task's lines inside one are not its own.
fn through_the_crates_own_functions(functions: &[(String, Function)]) -> Vec<Finding> {
    // What the body of each function calls, by the crate's own names, read once.
    let mut called_by: std::collections::BTreeMap<&str, std::collections::BTreeSet<&str>> = std::collections::BTreeMap::new();
    for (_, function) in functions {
        let mut called: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for (_, line, joined) in &function.task_lines {
            for name in names_called(joined.as_str()) {
                called.insert(name);
            }
            let _reads_a_file: bool = holds_any(line, STORAGE_CALLS);
        }
        called_by.insert(function.name.as_str(), called);
    }
    // What: The names this crate defines once, which are the only ones a call can be followed through.
    // Why:  `new` names a hundred functions of this crate, and a `X::new(` of one of them is not a call to the
    //       landing board's, which takes a place in a journal.
    let mut definitions: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (_, function) in functions {
        *definitions.entry(function.name.as_str()).or_default() += 1;
    }
    let known: std::collections::BTreeSet<&str> = definitions.iter().filter(|(_, count)| { return **count == 1; }).map(|(name, _)| { return *name; }).collect();
    // What: The functions that wait on a file themselves, or reach one that does.
    // Why:  One worklist pass, so the scan stays quick on a crate of thousands of functions.
    let mut blocking: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut callers_of: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
    for (caller, called) in &called_by {
        for name in called {
            if known.contains(*name) {
                callers_of.entry(name).or_default().push(caller);
            }
        }
    }
    // What: The functions this crate hands to a job of the blocking pool by name, which are off the worker.
    // Why:  `run_a_job_awaited_no_longer_than(ceiling, state, read_the_head_file)` names its work as an argument,
    //       and a closure is not the only shape the pool takes: a function passed this way runs there too, so a file
    //       wait inside it is not the caller's.
    let mut off_the_worker: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for (_, function) in functions {
        for (_, _line, joined) in &function.task_lines {
            for name in jobs_handed_to_the_pool(joined.as_str()) {
                off_the_worker.insert(name);
            }
        }
    }
    let mut waiting: Vec<&str> = Vec::new();
    for (_, function) in functions {
        if off_the_worker.contains(function.name.as_str()) || hands_work_to_a_pool(function.name.as_str()) {
            continue;
        }
        let waits: bool = function.task_lines.iter().any(|(_, line, _joined)| {
            return holds_any(line, STORAGE_CALLS) || holds_any(line, BLOCKING_WAITS);
        });
        if waits && !blocking.contains(function.name.as_str()) {
            blocking.insert(function.name.as_str());
            waiting.push(function.name.as_str());
        }
    }
    while let Some(found) = waiting.pop() {
        for caller in callers_of.get(found).map_or_else(Vec::new, |callers| { return callers.clone(); }) {
            if blocking.insert(caller) {
                waiting.push(caller);
            }
        }
    }
    let mut found: Vec<Finding> = Vec::new();
    for (name, function) in functions {
        if !function.is_async || !holds_the_service_s_tasks(name.as_str()) {
            continue;
        }
        for (number, _line, joined) in &function.task_lines {
            let reaches: bool = names_called(joined.as_str()).iter().any(|called| { return blocking.contains(*called); });
            if reaches {
                found.push((name.clone(), REACHED_FROM_A_TASK, *number));
                break;
            }
        }
    }
    return found;
}

/// What:
///  Whether this file holds the service's own tasks:
/// the modules of the resident service and of the after-commit work.
/// Why:
///   The rule that looks through a call is asked of these files,
/// because they are where the service's tasks are spawned and where a wait on a file stops every client of a service
/// (`doc/planning/git-policy-cli-full-tokio.md`,
///  and the third review of this branch, finding 4).
///  The rest of the crate has tasks of its own, on the runtimes of the commands that run them;
/// a scan of the whole crate by this rule answers a list its owners have to take up one by one,
/// and that list is reported rather than excused here
/// (`doc/handover/cli-git-rust-implementation.md`).
fn holds_the_service_s_tasks(path: &str) -> bool {
    return path.starts_with("service/") || path.starts_with("landing/");
}

/// What:
///  The functions `text` hands to a job of the blocking pool by name:
/// the bare identifiers in the argument list of one of the pool runners.
/// Why:
///  Those functions run off the worker, so a file wait inside one is not one the caller's task bears.
///  A closure in the same argument list is already off the worker by the rule that drops its lines.
fn jobs_handed_to_the_pool(text: &str) -> Vec<&str> {
    let mut handed: Vec<&str> = Vec::new();
    for runner in POOL_CALLS {
        let Some(at) = text.find(runner) else { continue };
        let arguments: &str = &text[at + runner.len()..];
        let bytes: &[u8] = arguments.as_bytes();
        let mut position: usize = 0;
        while position < bytes.len() {
            let starts_a_name: bool = (bytes[position] as char).is_ascii_alphabetic() || bytes[position] == b'_';
            if !starts_a_name {
                position += 1;
                continue;
            }
            let start: usize = position;
            while position < bytes.len() && ((bytes[position] as char).is_ascii_alphanumeric() || bytes[position] == b'_') {
                position += 1;
            }
            let name: &str = &arguments[start..position];
            // A bare name is a function passed as the job; one followed by `(` is called here, not handed over.
            if bytes.get(position) != Some(&b'(') && !is_a_common_word(name) {
                handed.push(name);
            }
        }
    }
    return handed;
}

/// Whether `name` is one of the calls that hand work to the blocking pool,
/// which is off the worker by what it does and not only by where its closure is.
fn hands_work_to_a_pool(name: &str) -> bool {
    return POOL_CALLS.iter().any(|call_name| { return call_name.strip_suffix('(') == Some(name); });
}

/// Whether `name` is a word of the language rather than a function of this crate,
/// which an argument list holds as `move`, `true` or `None`.
fn is_a_common_word(name: &str) -> bool {
    return matches!(name, "move" | "true" | "false" | "None" | "Some" | "Ok" | "Err" | "async" | "await" | "if" | "let" | "match" | "return" | "mut" | "ref");
}

/// What:
///  The names `text` calls,
///  as one identifier followed by an opening parenthesis and not by another name's letters.
/// Why:
///  The rule that looks through a call needs the names a body names once, not every name of the crate against every
/// line.
fn names_called(text: &str) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    let bytes: &[u8] = text.as_bytes();
    let mut at: usize = 0;
    while at < bytes.len() {
        let starts_a_name: bool = (bytes[at] as char).is_ascii_alphabetic() || bytes[at] == b'_';
        if !starts_a_name {
            at += 1;
            continue;
        }
        let start: usize = at;
        while at < bytes.len() && ((bytes[at] as char).is_ascii_alphanumeric() || bytes[at] == b'_') {
            at += 1;
        }
        if at < bytes.len() && bytes[at] == b'(' && bytes[start] != b'_' {
            names.push(&text[start..at]);
        }
    }
    return names;
}

/// The findings no `allowed` entry excuses,
/// each as a `file:line [rule] reason` text.
fn unexplained(found: &[Finding], allowed: &[Allowed]) -> Vec<String> {
    return found
        .iter()
        .filter(|(file, rule, _)| {
            return !allowed.iter().any(|entry| {
                return entry.file == file && entry.rule == *rule;
            });
        })
        .map(|(file, rule, number)| {
            let reason: &str = RULES
                .iter()
                .find(|candidate| {
                    return candidate.id == *rule;
                })
                .map_or("", |matched| {
                    return matched.reason;
                });
            return format!("{file}:{number} [{rule}] {reason}");
        })
        .collect();
}

/// The `allowed` entries no finding matches,
/// each as a `file [rule]: why` text.
fn stale(found: &[Finding], allowed: &[Allowed]) -> Vec<String> {
    return allowed
        .iter()
        .filter(|entry| {
            return !found.iter().any(|(file, rule, _)| {
                return entry.file == file && entry.rule == *rule;
            });
        })
        .map(|entry| format!("{} [{}]: {}", entry.file, entry.rule, entry.why))
        .collect();
}

/// No release code starts a thread,
/// pauses on a blocking clock,
/// starts a child outside the retried start,
/// or does blocking
///  work inside a task,
/// beyond the listed exceptions,
/// and the rule that looks through a call finds a task of a file no one has converted yet only where that file is
/// recorded.
#[test]
fn the_release_code_keeps_blocking_work_off_the_runtime() {
    let found: Vec<Finding> = findings();
    let new_files: Vec<String> = found
        .iter()
        .filter(|(file, rule, _)| {
            return *rule == REACHED_FROM_A_TASK && !REACHED_FROM_A_TASK_KNOWN.iter().any(|known| { return known == file; });
        })
        .map(|(file, _rule, number)| { return format!("{file}:{number}"); })
        .collect();
    assert!(
        new_files.is_empty(),
        "a task of this file reaches a wait on a file through a plain function of this crate, on the worker it is polled \
         on: run it through `service_file_work_off_the_workers.rs`, or record the file in REACHED_FROM_A_TASK_KNOWN with \
         the owner that converts it:\n{}",
        new_files.join("\n")
    );
    let converted: Vec<String> = REACHED_FROM_A_TASK_KNOWN
        .iter()
        .filter(|known| { return !found.iter().any(|(file, rule, _)| { return file == *known && *rule == REACHED_FROM_A_TASK; }); })
        .map(|known| { return String::from(*known); })
        .collect();
    assert!(
        converted.is_empty(),
        "these files' tasks reach no wait on a file any more, so they come out of REACHED_FROM_A_TASK_KNOWN:\n{}",
        converted.join("\n")
    );
    // The rule that looks through a call is held to its own list, which says which files its findings stand for.
    let besides: Vec<Finding> = found.iter().filter(|(_, rule, _)| { return *rule != REACHED_FROM_A_TASK; }).cloned().collect();
    let unexplained: Vec<String> = unexplained(besides.as_slice(), ALLOWED);
    assert!(
        unexplained.is_empty(),
        "the runtime's workers only await: a thread, a sleeping clock, a child start outside `start_retry` or a blocking \
         call inside a task stops the tasks that would end its wait (doc/planning/git-policy-cli-full-tokio.md, step T13); \
         fix it or name it in ALLOWED with what removes it:\n{}",
        unexplained.join("\n")
    );
}

/// Every listed exception still matches code,
/// so the list shrinks as the move lands.
#[test]
fn every_listed_exception_still_matches_code() {
    let found: Vec<Finding> = findings();
    let besides: Vec<Finding> = found.iter().filter(|(_, rule, _)| { return *rule != REACHED_FROM_A_TASK; }).cloned().collect();
    let stale: Vec<String> = stale(besides.as_slice(), ALLOWED);
    assert!(stale.is_empty(), "remove these entries from ALLOWED, their code is gone:\n{}", stale.join("\n"));
}

/// The rules fire on the text they exist for,
/// so a rule that stopped matching would fail here.
#[test]
fn each_rule_fires_on_its_own_forbidden_text() {
    let samples: [(&str, &str); 9] = [
        ("thread-start", "let started = std::thread::Builder::new().spawn(move || work());"),
        ("blocking-sleep", "std::thread::sleep(POLL_PAUSE);"),
        ("unretried-start", "let output = command.output();"),
        ("unretried-built-start", "let output = git_command(real_git, arguments, overlay).output();"),
        ("storage-read-on-a-task", "let head = std::fs::read_to_string(git_dir.join(\"HEAD\"));"),
        ("blocking-wait-on-a-task", "let answered = replies.recv_timeout(ceiling);"),
        ("blocking-helper-on-a-task", "let (retries, watch) = restored_within(ceiling, plan, watch, instance);"),
        ("plain-tokio-fs", "let metadata = tokio::fs::metadata(path).await;"),
        ("block-in-place", "let value = tokio::task::block_in_place(|| read_it());"),
    ];
    for (id, sample) in samples {
        let rule: &Rule = RULES
            .iter()
            .find(|candidate| {
                return candidate.id == id;
            })
            .expect("rule exists");
        assert!((rule.matches)(sample, sample), "{id} did not match {sample}");
    }
    assert_eq!(RULES.len(), samples.len());
}

/// The scan reads the release sources of this crate,
/// and it sees the functions of a real file with their `async` flag.
#[test]
fn the_scan_reads_the_release_sources_and_their_functions() {
    let root = source_root();
    let sources: Vec<String> = release_sources_in(&root, &real_roots()).expect("the crate's release sources");
    assert!(sources.iter().any(|path| {
        return Path::new(path).ends_with("start_retry.rs");
    }));
    assert!(sources.iter().all(|path| {
        return !path.ends_with("_tests.rs");
    }));
    let text: String = std::fs::read_to_string(root.join("foundation/blocking_job.rs")).expect("a release file");
    let functions: Vec<Function> = functions_in(text.as_str()).expect("the functions of a release file");
    let awaiting: Vec<&Function> = functions
        .iter()
        .filter(|function| {
            return function.is_async;
        })
        .collect();
    assert_eq!(awaiting.len(), 1, "{:?}", awaiting.iter().map(|function| return function.name.as_str()).collect::<Vec<&str>>());
    assert_eq!(awaiting[0].name, "run_blocking_job");
}

/// The files whose text holds a brace inside a string or a character literal still split into functions,
/// so the
/// reader of literals cannot end a body early and the scan cannot fall out of step.
#[test]
fn a_file_with_braces_in_its_literals_still_splits_into_functions() {
    let root = source_root();
    let files: [&str; 4] =
        ["command/branch_delete_audit.rs", "policy/diagnostics.rs", "policy/policy_events.rs", "repo_read/repo_read_refs.rs"];
    for file in files {
        let text: String = std::fs::read_to_string(root.join(file)).expect("a release file");
        let functions: Vec<Function> = functions_in(text.as_str()).unwrap_or_else(|error| {
            panic!("{file}: {error}");
        });
        assert!(!functions.is_empty(), "{file} holds functions");
    }
}

/// A file whose braces do not balance is an error,
/// never a file that is silently skipped.
#[test]
fn a_file_with_unbalanced_braces_is_an_error() {
    let outcome = functions_in("fn half_a_function() {\n    let value = 1;\n");
    assert!(outcome.is_err(), "{outcome:?}");
}

/// A file with a literal that never closes is an error,
/// so an unterminated string hides no code from the scan.
#[test]
fn a_file_with_an_unterminated_literal_is_an_error() {
    let outcome = braces_outside_literals("fn a() {\n    let text = \"starts and never ends;\n");
    assert!(outcome.is_err(), "{outcome:?}");
}

/// The scan of the real crate reads every release file,
/// so a file it cannot read is an error and not a pass.
#[test]
fn the_scan_of_the_real_crate_answers_findings_and_never_an_empty_pass() {
    let found: Vec<Finding> = findings();
    assert!(!found.is_empty(), "the crate still holds blocking work the move has not converted");
    assert!(found.iter().all(|(file, rule, number)| {
        return !file.is_empty() && !rule.is_empty() && *number > 0;
    }));
}

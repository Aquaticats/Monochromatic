//! The scan that keeps blocked code off the runtime's workers.
//! What:
//!  A check of the crate's own source text that fails when release code starts a thread,
//!  pauses on a blocking clock,
//!  starts a child outside the retried start,
//!  or does blocking storage work or a blocking wait inside a task the runtime polls.
//! Why:
//!  The owner's move to tokio is "Full on tokio.
//! Multi-thread." (`doc/planning/git-policy-cli-full-tokio.md`),
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
//!  it follows no call into another file,
//!  and a fork of one line across two lines is read only where the
//!  next line is joined to it.
//!  Everything else the crate can see in one file is covered:
//!  the file's code lines,
//!  the
//!  bodies of functions that build a `Command`,
//!  and the bodies of `async fn`s outside the closures that hand their work to the
//!  blocking pool.
//!  Braces are counted outside comments,
//!  strings,
//!  character literals and raw strings,
//!  so a `"{{"` in a format
//!  text neither opens a block nor hides one,
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
use rules::{ALLOWED, Allowed, Place, RULES, Rule};

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
fn code_lines(text: &str) -> BodyLines {
    let lines: Vec<&str> = text.lines().collect();
    let mut code: BodyLines = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed: &str = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        let next: &str = lines.get(index + 1).map_or("", |following| {
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
    /// An `r"..."`, `r#"..."#` or `br#"..."#` raw string,
    /// with how many `#` it opened with.
    Raw(usize),
}

/// What:
///  `fn braces_outside_literals(text: &str) -> Result<Vec<(usize, usize)>, String>` counts,
///  for each line,
///  the `{` and `}` that are code rather than comment or literal text.
/// Why:
///  A function's body is found by its braces,
/// and the crate writes `format!("{{\"schemaVersion\":{SCHEMA_VERSION}")`,
/// `b"@{"` and
///  `'\u{8}'`,
///  whose braces close nothing;
///  counting them would end a body early or lose the end of the file.
///  An unterminated literal or block comment means the file cannot be read,
/// which is an error and not a file
///  the scan skips.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bracesOutsideLiterals(text: string): [number, number][]  // throws Error
/// ```
fn braces_outside_literals(text: &str) -> Result<Vec<(usize, usize)>, String> {
    let characters: Vec<char> = text.chars().collect();
    let mut counts: Vec<(usize, usize)> = vec![(0, 0)];
    let mut scanner: Scanner = Scanner::Code;
    let mut index: usize = 0;
    while index < characters.len() {
        let character: char = characters[index];
        let next: Option<char> = characters.get(index + 1).copied();
        if character == '\n' {
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
                    index += 2;
                }
                '/' if next == Some('*') => {
                    scanner = Scanner::BlockComment(1);
                    index += 2;
                }
                '"' => {
                    scanner = Scanner::Text;
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
                            index = cursor + 1;
                        }
                        Some('\'') if character == 'b' => {
                            scanner = Scanner::Character;
                            index = cursor + 1;
                        }
                        _ => index += 1,
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
                        let mut braces: usize = 0;
                        while let Some(inner) = characters.get(cursor).copied() {
                            if inner == '\n' {
                                break;
                            }
                            if inner == '{' {
                                braces += 1;
                            }
                            if inner == '}' {
                                braces = braces.saturating_sub(1);
                            }
                            if inner == '\'' && braces == 0 {
                                cursor += 1;
                                closed = true;
                                break;
                            }
                            cursor += 1;
                        }
                    }
                    index = if closed { cursor } else { index + 1 };
                }
                '{' => {
                    if let Some(line) = counts.last_mut() {
                        line.0 += 1;
                    }
                    index += 1;
                }
                '}' => {
                    if let Some(line) = counts.last_mut() {
                        line.1 += 1;
                    }
                    index += 1;
                }
                _ => index += 1,
            },
            Scanner::Text => match character {
                // A line continuation keeps the string open across the line break, and the break still starts a line.
                '\\' if next == Some('\n') => index += 1,
                '\\' => index += 2,
                '"' => {
                    scanner = Scanner::Code;
                    index += 1;
                }
                _ => index += 1,
            },
            Scanner::Character => match character {
                '\\' => index += 2,
                '\'' => {
                    scanner = Scanner::Code;
                    index += 1;
                }
                _ => index += 1,
            },
            Scanner::Raw(hashes) => {
                if character != '"' {
                    index += 1;
                    continue;
                }
                let mut cursor: usize = index + 1;
                let mut seen: usize = 0;
                while seen < hashes && characters.get(cursor) == Some(&'#') {
                    seen += 1;
                    cursor += 1;
                }
                if seen == hashes {
                    scanner = Scanner::Code;
                    index = cursor;
                } else {
                    index += 1;
                }
            }
            Scanner::LineComment => index += 1,
            Scanner::BlockComment(depth) => {
                if character == '*' && next == Some('/') {
                    scanner = if depth <= 1 { Scanner::Code } else { Scanner::BlockComment(depth - 1) };
                    index += 2;
                } else if character == '/' && next == Some('*') {
                    scanner = Scanner::BlockComment(depth + 1);
                    index += 2;
                } else {
                    index += 1;
                }
            }
        }
    }
    if scanner != Scanner::Code && scanner != Scanner::LineComment {
        return Err(String::from("a literal or a block comment is not closed, so this file cannot be read"));
    }
    return Ok(counts);
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
];

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
    let counts: Vec<(usize, usize)> = braces_outside_literals(text)?;
    let lines: Vec<&str> = text.lines().collect();
    let code: BodyLines = code_lines(text);
    let mut found: Vec<Function> = Vec::new();
    let mut depth: usize = 0;
    let mut building: Option<(String, bool, usize, bool, usize)> = None;
    for (index, (opened, closed)) in counts.iter().enumerate() {
        let number: usize = index + 1;
        let line: &str = lines.get(index).copied().unwrap_or("");
        if building.is_none() {
            if let Some((name, is_async)) = signature_of(line) {
                building = Some((name, is_async, depth, false, number));
            }
        }
        depth = depth + opened;
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
    let mut task_lines: BodyLines = Vec::new();
    for (index, (number, line, joined)) in inside.iter().enumerate() {
        let (opened, closed) = counts.get(index).copied().unwrap_or((0, 0));
        let trimmed: &str = line.trim_start();
        depth = depth + opened;
        depth = depth.saturating_sub(closed);
        let already: bool = pool_from.is_some();
        if !already && opens_a_pool_body(trimmed) {
            pool_from = Some(depth);
            continue;
        }
        if already {
            if let Some(pool_depth) = pool_from {
                if depth < pool_depth {
                    pool_from = None;
                    task_lines.push((*number, line.clone(), joined.clone()));
                }
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

/// Every `(relative path, rule id, line number)` the rules find in the release sources of `root`.
fn findings_in(root: &Path, roots: &[CrateRoot]) -> Result<Vec<Finding>, String> {
    let mut found: Vec<Finding> = Vec::new();
    for name in release_sources_in(root, roots)? {
        let text: String = std::fs::read_to_string(root.join(name.as_str()))
            .map_err(|error| return format!("cannot read {name}: {error}"))?;
        let functions: Vec<Function> = functions_in(text.as_str()).map_err(|error| return format!("{name}: {error}"))?;
        let mut places: Vec<(Place, BodyLines)> = Vec::new();
        places.push((Place::Everywhere, code_lines(text.as_str())));
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
/// beyond the listed exceptions.
#[test]
fn the_release_code_keeps_blocking_work_off_the_runtime() {
    let unexplained: Vec<String> = unexplained(findings().as_slice(), ALLOWED);
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
    let stale: Vec<String> = stale(findings().as_slice(), ALLOWED);
    assert!(stale.is_empty(), "remove these entries from ALLOWED, their code is gone:\n{}", stale.join("\n"));
}

/// The rules fire on the text they exist for,
/// so a rule that stopped matching would fail here.
#[test]
fn each_rule_fires_on_its_own_forbidden_text() {
    let samples: [(&str, &str); 7] = [
        ("thread-start", "let started = std::thread::Builder::new().spawn(move || work());"),
        ("blocking-sleep", "std::thread::sleep(POLL_PAUSE);"),
        ("unretried-start", "let output = command.output();"),
        ("storage-read-on-a-task", "let head = std::fs::read_to_string(git_dir.join(\"HEAD\"));"),
        ("blocking-wait-on-a-task", "let answered = replies.recv_timeout(ceiling);"),
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

//! The mechanical ban on the wrapper starting itself or writing shell text.
//! The wrapper never starts its own executable (a child,
//! a hook command,
//! a shim,
//! an `argv[0]` dispatch),
//! never passes
//!  management words to a process it starts,
//! and never writes shell text or scripts.
//! This scan reads the source of the
//!  release code and fails on any line that does,
//! unless an `ALLOWED` entry justifies it.
//! Each entry must still match
//!  something,
//! so a cleaned file also shrinks the list.
//! The scan is textual by design:
//! the clippy ban on
//!  `std::env::current_exe` (`clippy.toml`) covers the typed call,
//! and this covers the spellings clippy cannot see.
//! Explanations come from the parent process that owns a child;
//! see `doc/planning/cli-git-no-self-invocation.md`.

use std::path::Path;

/// The forbidden texts and the justified exceptions.
#[path = "no_self_invocation_rules_tests.rs"]
mod rules;
use rules::{
    ALLOWED,
    Allowed,
    RULES,
    Rule,
};

use super::release_sources_in;
use crate::source_scan_tests::{
    real_roots,
    source_root,
};

/// Filesystem-backed controls of this scan on nested layouts.
#[path = "no_self_invocation_fixture_tests.rs"]
mod fixtures;

/// One rule match:
/// the source file as a path relative to `src/native`,
/// the rule that matched,
/// and the line number.
type Finding = (String, &'static str, usize);

/// The squeezed text of each line of a source that holds anything at all,
/// with its line number.
/// Why nothing is cut from a line first:
///  a forbidden read is usually a string,
/// as `"/proc/self/cmdline"` is,
/// so cutting literals would
///  hide exactly what this scan looks for;
///  and cutting comments would let a reader that disagrees with rustc hide a line,
/// which is what the
///  sixth revision's lexer did.
///  A comment that spells a needle is therefore a finding,
/// and each such line is a declared `ALLOWED`
///  row:
/// release code holds one today,
/// and it is in `main.rs`.
fn code_lines(source: &str) -> Vec<(usize, String)> {
    let mut code: Vec<(usize, String)> = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let squeezed_line: String = super::squeezed(line);
        if squeezed_line.is_empty() {
            continue;
        }
        code.push((index + 1, squeezed_line));
    }
    return code;
}

/// Every `(relative path, rule id, line number)` the rules find in the release sources of `root`.
fn findings_in(
    root: &Path,
    roots: &[crate::source_scan_tests::CrateRoot],
) -> Result<Vec<Finding>, String> {
    let mut found: Vec<Finding> = Vec::new();
    for name in release_sources_in(root, roots)? {
        let text: String = std::fs::read_to_string(root.join(name.as_str()))
            .map_err(|error| return format!("cannot read {name}: {error}"))?;
        for (number, line) in code_lines(text.as_str()) {
            for rule in RULES {
                if (rule.matches)(line.as_str()) {
                    found.push((name.clone(), rule.id, number));
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
fn unexplained(
    found: &[Finding],
    allowed: &[Allowed],
) -> Vec<String> {
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
fn stale(
    found: &[Finding],
    allowed: &[Allowed],
) -> Vec<String> {
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

/// No release code starts the wrapper,
/// passes it management words,
/// or writes shell text,
/// beyond the justified list.
#[test]
fn the_release_code_never_starts_the_wrapper_or_writes_shell_text() {
    let unexplained: Vec<String> = unexplained(findings().as_slice(), ALLOWED);
    assert!(
        unexplained.is_empty(),
        "the wrapper never starts itself and never writes shell text; explanations come from the parent that owns the \
         child (doc/planning/cli-git-no-self-invocation.md):\n{}",
        unexplained.join("\n")
    );
}

/// Every justified exception still matches code,
/// so the list shrinks as the code is cleaned.
#[test]
fn every_justified_exception_still_matches_code() {
    let stale: Vec<String> = stale(findings().as_slice(), ALLOWED);
    assert!(
        stale.is_empty(),
        "remove these entries from ALLOWED, their code is gone:\n{}",
        stale.join("\n")
    );
}

/// The scan reads the release files,
/// and no release code defines a hook command at all.
#[test]
fn the_scan_reads_the_release_sources_and_no_hook_command_is_defined() {
    let root = source_root();
    let sources: Vec<String> =
        release_sources_in(&root, &real_roots()).expect("the crate's release sources");
    assert!(sources.iter().any(|path| {
        return Path::new(path).ends_with("lfs_pre_push.rs");
    }));
    assert!(sources.iter().all(|path| {
        return !path.ends_with("_tests.rs");
    }));
    let mut definitions: Vec<String> = Vec::new();
    for path in sources {
        let text: String = std::fs::read_to_string(root.join(path)).expect("source file");
        for (_, line) in code_lines(text.as_str()) {
            if line.contains(".command=") {
                definitions.push(line);
            }
        }
    }
    assert_eq!(definitions, Vec::<String>::new());
}

/// The rules fire on the text they exist for,
/// so a rule that stopped matching would fail here.
#[test]
fn each_rule_fires_on_its_own_forbidden_text() {
    let samples: [(&str, &str); 7] = [
        ("own-executable", "let exe = std::env::current_exe()?;"),
        ("argv0-dispatch", "let name = std::env::args_os().next();"),
        ("argv-vector-read", "use std::env::args_os as the_vector;"),
        ("shell-text", "let script = \"#!/bin/sh\\nexit 0\\n\";"),
        (
            "hook-command",
            "format!(\"hook.x.command={exe} monochromatic-git-policy-cli sub\")",
        ),
        (
            "management-word",
            "Command::new(program).arg(\"monochromatic-git-policy-cli\").arg(\"x\")",
        ),
        (
            "spawn-own-executable",
            "Command::new(request.own_executable)",
        ),
    ];
    for (id, sample) in samples {
        let rule: &Rule = RULES
            .iter()
            .find(|candidate| {
                return candidate.id == id;
            })
            .expect("rule exists");
        assert!((rule.matches)(sample), "{id} did not match {sample}");
    }
    assert_eq!(RULES.len(), samples.len());
}

/// Each added needle fires on its own witness and on no release read,
/// so a needle that grew too wide fails here before it fails the tree.
#[test]
fn each_added_needle_fires_on_its_own_witness_and_no_release_read() {
    let dispatch: &Rule = RULES
        .iter()
        .find(|rule| {
            return rule.id == "argv0-dispatch";
        })
        .expect("argv0-dispatch");
    let vector: &Rule = RULES
        .iter()
        .find(|rule| {
            return rule.id == "argv-vector-read";
        })
        .expect("argv-vector-read");
    for witness in [
        "std::fs::read(\"/proc/self/cmdline\")",
        "std::fs::read_to_string(\"/proc/self/comm\")",
        "std::fs::read_to_string(\"/proc/thread-self/comm\")",
    ] {
        assert!(
            (dispatch.matches)(witness),
            "argv0-dispatch does not fire on {witness}"
        );
    }
    for witness in [
        "use std::env::args_os as the_vector;",
        "let all = std::env::args().collect::<Vec<_>>();",
        "let name = e::args();",
        "use std::env::args as the_vector;",
    ] {
        assert!(
            (vector.matches)(witness),
            "argv-vector-read does not fire on {witness}"
        );
    }
    for control in [
        "std::fs::read_to_string(\"/proc/self/stat\")",
        "std::fs::read_to_string(format!(\"/proc/self/task/{tid}/io\"))",
        "std::fs::read_to_string(format!(\"/proc/{pid}/comm\"))",
    ] {
        assert!(
            !(dispatch.matches)(control),
            "argv0-dispatch fires on {control}, which release code reads today"
        );
    }
    assert!(
        !(vector.matches)("let taken = request.arguments.as_slice();"),
        "argv-vector-read fires only on the process's own vector, which every other file takes as a parameter"
    );
}

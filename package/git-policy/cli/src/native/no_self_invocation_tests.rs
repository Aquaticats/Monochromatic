//! The mechanical ban on the wrapper starting itself or writing shell text.
//! The wrapper never starts its own executable (a child, a hook command, a shim, an `argv[0]` dispatch), never passes
//!  management words to a process it starts, and never writes shell text or scripts. This scan reads the source of the
//!  release code and fails on any line that does, unless an `ALLOWED` entry justifies it. Each entry must still match
//!  something, so a cleaned file also shrinks the list. The scan is textual by design: the clippy ban on
//!  `std::env::current_exe` (`clippy.toml`) covers the typed call, and this covers the spellings clippy cannot see.
//! Explanations come from the parent process that owns a child; see `doc/planning/cli-git-no-self-invocation.md`.

use std::path::{Path, PathBuf};

/// One kind of forbidden text.
struct Rule {
    /// The name an `ALLOWED` entry gives.
    id: &'static str,
    /// Why the text is forbidden.
    reason: &'static str,
    /// Whether a source line holds the forbidden text; the second text is the line with the next one joined, for a
    ///  call split over two lines.
    matches: fn(&str, &str) -> bool,
}

/// The forbidden texts.
const RULES: &[Rule] = &[
    Rule {
        id: "own-executable",
        reason: "asks the operating system for the running executable, the first step of starting the wrapper again",
        matches: |line, _joined| {
            return line.contains("current_exe") || line.contains("/proc/self/exe");
        },
    },
    Rule {
        id: "argv0-dispatch",
        reason: "reads the program name the wrapper was started as, which only matters for a wrapper that is started under another name",
        matches: |line, _joined| {
            return ["args_os().next()", "args().next()", ".nth(0)", "argv[0]", "arg0("]
                .iter()
                .any(|needle| {
                    return line.contains(needle);
                });
        },
    },
    Rule {
        id: "shell-text",
        reason: "writes or starts shell text; the wrapper writes no shell script, function or `sh -c`",
        matches: |line, _joined| {
            return ["/bin/sh", "/bin/bash", "#!/", "Command::new(\"sh\")", "Command::new(\"bash\")", "\"$@\"", "exec \\\"$"]
                .iter()
                .any(|needle| {
                    return line.contains(needle);
                });
        },
    },
    Rule {
        id: "hook-command",
        reason: "defines a Git hook command; the only one allowed is the plain `git-lfs pre-push` of `lfs_pre_push.rs`",
        matches: |line, _joined| {
            return line.contains(".command=") && !line.contains(".command={HOOK_COMMAND}");
        },
    },
    Rule {
        id: "management-word",
        reason: "passes the management words to a process, which would start the wrapper's own management entry",
        matches: |line, _joined| {
            return line.contains(".arg(\"cli-git\"")
                || line.contains(".arg(MANAGEMENT_COMMAND")
                || line.contains("args([\"cli-git\"")
                || line.contains("HOOK_ENTRY");
        },
    },
    Rule {
        id: "spawn-own-executable",
        reason: "starts a process from the wrapper's own executable path",
        matches: |line, joined| {
            return line.contains("Command::new(")
                && ["own_executable", "executable.as_path()", "inputs.own", "WRAPPER", "request.own"]
                    .iter()
                    .any(|needle| {
                        return joined.contains(needle);
                    });
        },
    },
];

/// One justified exception: the file, the rule, and why it stays.
struct Allowed {
    /// The file name under `src/native`.
    file: &'static str,
    /// The rule that file is excused from.
    rule: &'static str,
    /// Why it is not the design the ban forbids, or when it goes.
    why: &'static str,
}

/// The exceptions. Remove an entry when its code goes; the test fails on an entry that matches nothing.
const ALLOWED: &[Allowed] = &[
    Allowed {
        file: "real_git.rs",
        rule: "own-executable",
        why: "identification only: the path is excluded from the search for the real Git, and nothing is started from it",
    },
    Allowed {
        file: "doctor.rs",
        rule: "shell-text",
        why: "the text git-lfs itself writes as its hook shim, held only to recognise that shim; the wrapper never writes it",
    },
    Allowed {
        file: "main.rs",
        rule: "argv0-dispatch",
        why: "the argv[0] hook dispatcher, removed by phase 2 of doc/planning/cli-git-no-self-invocation.md",
    },
    Allowed {
        file: "commit_command.rs",
        rule: "own-executable",
        why: "names the executable for the commit hook dispatcher, removed by phase 2 of doc/planning/cli-git-no-self-invocation.md",
    },
    Allowed {
        file: "manual_push_probe.rs",
        rule: "shell-text",
        why: "the probe's recording hook is a shell script written by the wrapper; the owner's no-shell rule forbids it and its replacement is open, see the planning document",
    },
];

/// The modules `lib.rs` compiles for tests only: a `#[cfg(test)]` attribute with no blank line before the `mod` line.
fn test_only_modules() -> Vec<String> {
    let text: String =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/lib.rs")).expect("lib.rs");
    let mut names: Vec<String> = Vec::new();
    let mut test_only: bool = false;
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() {
            test_only = false;
        } else if trimmed.starts_with("#[cfg(test)]") {
            test_only = true;
        } else if let Some(name) = trimmed
            .strip_prefix("pub mod ")
            .or_else(|| {
                return trimmed.strip_prefix("pub(crate) mod ");
            })
            .or_else(|| {
                return trimmed.strip_prefix("mod ");
            })
            && test_only
        {
            names.push(name.trim_end_matches(';').to_owned());
        }
    }
    return names;
}

/// The release source files: every `.rs` file under `src` except test sidecars, binary-level controls and support
///  that only tests compile.
fn release_sources() -> Vec<PathBuf> {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let test_only: Vec<String> = test_only_modules();
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("source directory").flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let name: String = entry.file_name().to_string_lossy().into_owned();
            let release: bool = name.ends_with(".rs")
                && !name.ends_with("_tests.rs")
                && !name.starts_with("binary_")
                && !test_only.contains(&name.trim_end_matches(".rs").to_owned());
            if release {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// The lines of a source file that are code, with the following line joined so a call split over two lines still reads.
fn code_lines(text: &str) -> Vec<(usize, String, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut code: Vec<(usize, String, String)> = Vec::new();
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

/// Every `(file name, rule id, line number)` the rules find in the release sources.
fn findings() -> Vec<(String, &'static str, usize)> {
    let mut found: Vec<(String, &'static str, usize)> = Vec::new();
    for path in release_sources() {
        let name: String = path.file_name().expect("file name").to_string_lossy().into_owned();
        let text: String = std::fs::read_to_string(&path).expect("source file");
        for (number, line, joined) in code_lines(text.as_str()) {
            for rule in RULES {
                if (rule.matches)(line.as_str(), joined.as_str()) {
                    found.push((name.clone(), rule.id, number));
                }
            }
        }
    }
    return found;
}

/// No release code starts the wrapper, passes it management words, or writes shell text, beyond the justified list.
#[test]
fn the_release_code_never_starts_the_wrapper_or_writes_shell_text() {
    let found: Vec<(String, &'static str, usize)> = findings();
    let unexplained: Vec<String> = found
        .iter()
        .filter(|(file, rule, _)| {
            return !ALLOWED.iter().any(|allowed| {
                return allowed.file == file && allowed.rule == *rule;
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
    assert!(
        unexplained.is_empty(),
        "the wrapper never starts itself and never writes shell text; explanations come from the parent that owns the \
         child (doc/planning/cli-git-no-self-invocation.md):\n{}",
        unexplained.join("\n")
    );
}

/// Every justified exception still matches code, so the list shrinks as the code is cleaned.
#[test]
fn every_justified_exception_still_matches_code() {
    let found: Vec<(String, &'static str, usize)> = findings();
    let stale: Vec<String> = ALLOWED
        .iter()
        .filter(|allowed| {
            return !found.iter().any(|(file, rule, _)| {
                return allowed.file == file && allowed.rule == *rule;
            });
        })
        .map(|allowed| format!("{} [{}]: {}", allowed.file, allowed.rule, allowed.why))
        .collect();
    assert!(stale.is_empty(), "remove these entries from ALLOWED, their code is gone:\n{}", stale.join("\n"));
}

/// The scan reads the release files: it sees the hook command definition and the source directory is not empty.
#[test]
fn the_scan_reads_the_release_sources_and_sees_the_plain_hook_command() {
    let sources: Vec<PathBuf> = release_sources();
    assert!(sources.iter().any(|path| {
        return path.ends_with("lfs_pre_push.rs");
    }));
    assert!(sources.iter().all(|path| {
        return !path.to_string_lossy().ends_with("_tests.rs");
    }));
    // The one hook command definition of the whole source is the plain command constant, which the rule excuses by text.
    let mut definitions: Vec<String> = Vec::new();
    for path in sources {
        let text: String = std::fs::read_to_string(&path).expect("source file");
        for (_, line, _) in code_lines(text.as_str()) {
            if line.contains(".command=") {
                definitions.push(line);
            }
        }
    }
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert!(definitions[0].contains(".command={HOOK_COMMAND}"), "{definitions:?}");
}

/// The rules fire on the text they exist for, so a rule that stopped matching would fail here.
#[test]
fn each_rule_fires_on_its_own_forbidden_text() {
    let samples: [(&str, &str); 6] = [
        ("own-executable", "let exe = std::env::current_exe()?;"),
        ("argv0-dispatch", "let name = std::env::args_os().next();"),
        ("shell-text", "let script = \"#!/bin/sh\\nexit 0\\n\";"),
        ("hook-command", "format!(\"hook.x.command={exe} cli-git sub\")"),
        ("management-word", "Command::new(program).arg(\"cli-git\").arg(\"x\")"),
        ("spawn-own-executable", "Command::new(request.own_executable)"),
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

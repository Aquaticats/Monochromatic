//! The forbidden texts and the justified exceptions of the scan in `no_self_invocation_tests.rs`.

/// One kind of forbidden text.
pub(super) struct Rule {
    /// The name an `ALLOWED` entry gives.
    pub(super) id: &'static str,
    /// Why the text is forbidden.
    pub(super) reason: &'static str,
    /// Whether a source line holds the forbidden text;
    /// the second text is the line with the next one joined,
    /// for a
    ///  call split over two lines.
    pub(super) matches: fn(&str, &str) -> bool,
}

/// The forbidden texts.
pub(super) const RULES: &[Rule] = &[
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
        reason: "defines a Git hook or hook command; the wrapper injects no hook at all, the upload is a child the parent runs",
        matches: |line, _joined| {
            return line.contains(".command=") || line.contains(".event=pre-push");
        },
    },
    Rule {
        id: "management-word",
        reason: "passes the management words to a process, which would start the wrapper's own management entry",
        matches: |line, _joined| {
            return line.contains(".arg(\"monochromatic-git-policy-cli\"")
                || line.contains(".arg(MANAGEMENT_COMMAND")
                || line.contains("args([\"monochromatic-git-policy-cli\"")
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

/// One justified exception:
/// the file's relative path,
/// the rule,
/// and why it stays.
pub(super) struct Allowed {
    /// The file's path relative to `src/native`,
    /// `/`-separated (`real_git.rs`,
    /// or `git/real_git.rs` for a nested one);
    ///  a file name alone excuses nothing in a subdirectory.
    pub(super) file: &'static str,
    /// The rule that file is excused from.
    pub(super) rule: &'static str,
    /// Why it is not the design the ban forbids,
    /// or when it goes.
    pub(super) why: &'static str,
}

/// The exceptions.
/// Remove an entry when its code goes;
/// the test fails on an entry that matches nothing.
pub(super) const ALLOWED: &[Allowed] = &[
    Allowed {
        file: "invocation/real_git.rs",
        rule: "own-executable",
        why: "identification only: the path is excluded from the search for the real Git, and nothing is started from it",
    },
    Allowed {
        file: "health/lfs_shims.rs",
        rule: "shell-text",
        why: "the text git-lfs itself writes as its hook shim, held only to recognise that shim; the wrapper never writes it",
    },
];

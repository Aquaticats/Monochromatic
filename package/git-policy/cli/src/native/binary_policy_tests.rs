//! What:
//!  Configuration loading and the read-only fast path through the built executable.
//! Why:
//!  Configuration errors must stop a guarded command before Git,
//!  and a read-only
//!      command must never read configuration nor start more Git processes than the one
//!      question of where it runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['add', 'file']).status).toBe(2); expect(staged()).toEqual([]);
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, bounded, direct, executable, fixture, git, observe, remove, repository,
    wrapped,
};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The only configuration file the native wrapper reads.
const CONFIG_FILE_NAME: &str = "cli-git.config.jsonc";

/// The one question a command asks Git about where it runs,
///  as the spawn log records it.
const LOCATION_QUERY: &str = "rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir --show-toplevel --show-prefix";

/// Run a wrapped command in a repository and return what the caller saw.
fn run_wrapped(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).current_dir(repo).args(arguments), b"");
}

/// Real Git's porcelain status of a repository,
///  as raw bytes.
fn status(fixture: &Fixture, repo: &Path) -> Vec<u8> {
    return git(
        fixture,
        repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )
    .stdout;
}

/// Run a wrapped command whose real Git is a shim that logs every start,
///  and return what
/// the caller saw with the log:
///  one line per Git process,
///  holding its arguments.
fn run_counted(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> (Observed, String) {
    let log: PathBuf = fixture.root.join("spawn.log");
    std::fs::write(&log, b"").expect("empty spawn log");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("shim"));
    let observed: Observed = observe(
        bounded(
            fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
        )
        .env("CLI_GIT_TEST_SPAWN_LOG", &log)
        .current_dir(repo)
        .args(arguments),
        b"",
    );
    let logged: String = std::fs::read_to_string(&log).expect("spawn log");
    return (observed, logged);
}

/// A read-only command starts Git once to ask where it runs,
///  or not at all,
///  and never reads configuration.
#[test]
fn read_only_commands_start_at_most_the_location_query() {
    let fixture: Fixture = fixture("fast-path-count");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::create_dir(fixture.root.join("shim")).expect("shim directory");
    executable(
        fixture.root.join("shim/git").as_path(),
        // The hooks-off option the wrapper puts first is not part of the count's question, so the log leaves it out.
        b"#!/bin/sh\nargs=\"$*\"\nargs=\"${args#-c core.hooksPath=/dev/null/cli-git-no-hooks }\"\nprintf '%s\\n' \"$args\" >> \"$CLI_GIT_TEST_SPAWN_LOG\"\nexec /usr/bin/git \"$@\"\n",
    );
    // Invalid configuration and a legacy file: reading either would stop the command.
    std::fs::write(repo.join(CONFIG_FILE_NAME), "this is not JSONC").expect("invalid");
    std::fs::write(
        repo.join("cli-git.config.ts"),
        "throw new Error('executed');\n",
    )
    .expect("legacy");
    // Commands exempt from require-root: the forwarded command is the only Git process.
    for arguments in [
        vec!["--version"],
        vec!["version"],
        vec!["--exec-path"],
        vec!["help", "--no-such-option"],
    ] {
        let (observed, logged) = run_counted(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(
            logged,
            format!("{}\n", arguments.join(" ")),
            "{arguments:?}"
        );
        assert_eq!(
            observed,
            observe(
                direct(&fixture)
                    .current_dir(&repo)
                    .args(arguments.as_slice()),
                b""
            ),
            "{arguments:?}"
        );
    }
    // Other read-only commands: one question about the location, then the forwarded command.
    for (arguments, forwarded) in [
        (vec!["log", "--oneline"], "log --oneline"),
        (vec!["rev-parse", "HEAD"], "rev-parse HEAD"),
        (vec!["branch", "--list"], "branch --list"),
        (vec!["tag", "--list"], "tag --list"),
        (vec!["diff", "--stat"], "diff --stat"),
        (
            vec!["status", "--porcelain=v1"],
            "-c advice.statusHints=false status --porcelain=v1",
        ),
        (vec!["--cli-git-keep-going", "ls-files"], "ls-files"),
    ] {
        let (observed, logged) = run_counted(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(observed.code, Some(0), "{arguments:?}");
        assert_eq!(
            logged,
            format!("{LOCATION_QUERY}\n{forwarded}\n"),
            "{arguments:?}"
        );
    }
    // Positive control: a guarded command does read the configuration, and stops on it
    // after the same single question, without ever starting the command itself.
    let (stopped, stopped_log) =
        run_counted(&fixture, repo.as_path(), &["reset", "--soft", "HEAD"]);
    assert_eq!(stopped.code, Some(2));
    assert!(
        String::from_utf8_lossy(&stopped.stderr).contains("\"code\":\"config-invalid\""),
        "{:?}",
        String::from_utf8_lossy(&stopped.stderr)
    );
    assert_eq!(stopped_log, format!("{LOCATION_QUERY}\n"));
    // Positive control: with valid configuration the same guarded command is counted twice.
    std::fs::write(repo.join(CONFIG_FILE_NAME), "{}").expect("valid");
    std::fs::remove_file(repo.join("cli-git.config.ts")).expect("remove legacy");
    let (ran, ran_log) = run_counted(&fixture, repo.as_path(), &["reset", "--soft", "HEAD"]);
    assert_eq!(ran.code, Some(0));
    assert_eq!(ran_log, format!("{LOCATION_QUERY}\nreset --soft HEAD\n"));
    remove(&fixture);
}

/// Invalid,
///  legacy and unknown-key configuration stops a changing command with the key or file named.
#[test]
fn configuration_errors_stop_before_git() {
    let fixture: Fixture = fixture("config-errors");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("file.txt"), b"content\n").expect("file");
    let source: PathBuf = repo.join(CONFIG_FILE_NAME);
    for (content, fragment) in [
        (
            r#"{ "plugins": {} }"#,
            "Configuration key plugins is retired",
        ),
        (r#"{ "unknown": 1 }"#, "Unknown configuration key: unknown."),
        (
            r#"{ "policies": { "no-such-policy": "error" } }"#,
            "Unknown policy ID: no-such-policy.",
        ),
        (
            r#"{ "hooks": {}, "hooks": {} }"#,
            "Configuration key hooks is defined more than once",
        ),
        (
            r#"{ "policies": { "final-newline": "fatal" } }"#,
            "Configuration key policies.final-newline has an invalid severity",
        ),
        (
            r#"{ "landing": { "reserveAfterLostRaces": 0 } }"#,
            "Configuration key landing.reserveAfterLostRaces must be a whole number from 1",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "executable": "./x" }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].executable is retired",
        ),
        (
            r#"{ "policies": null }"#,
            "Configuration key policies must not be null",
        ),
        ("export default {};", "JSONC syntax error at byte 0"),
    ] {
        std::fs::write(&source, content).expect("write configuration");
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "file.txt"]);
        let stderr: String = String::from_utf8_lossy(&observed.stderr).into_owned();
        assert_eq!(observed.code, Some(2), "{content}");
        assert_eq!(observed.stdout, Vec::<u8>::new(), "{content}");
        // Exactly one config-invalid event, on standard error for a wrapped command.
        assert!(
            stderr.starts_with(format!("{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"{}: ", source.display()).as_str()),
            "{content}\n  got: {stderr}"
        );
        assert!(stderr.ends_with("\"}\n"), "{content}\n  got: {stderr}");
        assert_eq!(stderr.matches('\n').count(), 1, "{content}");
        assert!(stderr.contains(fragment), "{content}\n  got: {stderr}");
        assert!(
            !stderr.contains("not implemented"),
            "{content}\n  got: {stderr}"
        );
    }
    // A legacy file alone: the migration diagnostic, and the legacy file is never executed.
    std::fs::remove_file(&source).expect("remove JSONC");
    std::fs::write(
        repo.join("cli-git.config.mjs"),
        "import { writeFileSync } from 'node:fs'; writeFileSync('executed', ''); export default {};\n",
    )
    .expect("legacy configuration");
    let legacy: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "file.txt"]);
    let legacy_stderr: String = String::from_utf8_lossy(&legacy.stderr).into_owned();
    assert_eq!(legacy.code, Some(2));
    assert!(
        legacy_stderr.starts_with(
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"Legacy configuration {} is not executed or read by the native cli-git.",
                repo.join("cli-git.config.mjs").display()
            )
            .as_str()
        ),
        "{legacy_stderr}"
    );
    assert!(!repo.join("executed").exists());
    assert!(!fixture.root.join("executed").exists());
    assert_eq!(
        status(&fixture, repo.as_path()),
        b"?? cli-git.config.mjs\n?? file.txt\n"
    );
    remove(&fixture);
}

/// Inspection commands are identical to real Git even beside invalid and legacy configuration.
#[test]
fn inspection_commands_never_read_configuration() {
    let fixture: Fixture = fixture("fast-path");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(CONFIG_FILE_NAME), "this is not JSONC").expect("invalid");
    std::fs::write(
        repo.join("cli-git.config.ts"),
        "throw new Error('executed');\n",
    )
    .expect("legacy");
    for arguments in [
        vec!["status", "--porcelain=v1"],
        vec!["log", "--oneline"],
        vec!["rev-parse", "--show-toplevel"],
        vec!["branch", "--list"],
        vec!["tag", "--list"],
        vec!["--version"],
    ] {
        let through_wrapper: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let through_git: Observed = observe(
            direct(&fixture)
                .current_dir(&repo)
                .args(arguments.as_slice()),
            b"",
        );
        assert_eq!(through_git.code, Some(0), "{arguments:?}");
        assert_eq!(through_wrapper, through_git, "{arguments:?}");
    }
    remove(&fixture);
}

/// A legacy file beside a valid JSONC file is reported by `git cli-git check` only:
///  an
/// ordinary configuration-loading command and `git cli-git fix` print nothing about it.
#[test]
fn legacy_file_beside_jsonc_is_reported_by_check_only() {
    let fixture: Fixture = fixture("legacy-notice");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(
        repo.join(CONFIG_FILE_NAME),
        "{ \"policies\": { \"mono/dependent-version-bump\": \"off\" } }\n",
    )
    .expect("valid configuration");
    std::fs::write(repo.join("cli-git.config.ts"), "export default {};\n").expect("legacy");
    std::fs::write(repo.join("cli-git.config.mjs"), "export default {};\n").expect("legacy");
    // Positive control: the configuration really is loaded by this ordinary command, whose
    // content policy then cannot predict an add of a file that does not exist.
    let ordinary: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "anything"]);
    assert_eq!(ordinary.code, Some(2));
    assert_eq!(ordinary.stdout, Vec::<u8>::new());
    let ordinary_stderr: String = String::from_utf8_lossy(&ordinary.stderr).into_owned();
    assert!(
        ordinary_stderr.contains("\"code\":\"content-unavailable\""),
        "{ordinary_stderr}"
    );
    assert!(!ordinary_stderr.contains("egacy"), "{ordinary_stderr}");
    assert!(
        !ordinary_stderr.contains("cli-git.config.ts"),
        "{ordinary_stderr}"
    );
    assert!(
        !ordinary_stderr.contains("configuration-warning"),
        "{ordinary_stderr}"
    );
    // Fix stays silent about the legacy files, and finds nothing to correct.
    let fix: Observed = run_wrapped(&fixture, repo.as_path(), &["cli-git", "fix", "--all"]);
    assert_eq!(fix.code, Some(0));
    assert_eq!(fix.stdout, Vec::<u8>::new());
    assert!(
        !String::from_utf8_lossy(&fix.stderr).contains("egacy"),
        "{:?}",
        String::from_utf8_lossy(&fix.stderr)
    );
    let check: Observed = run_wrapped(&fixture, repo.as_path(), &["cli-git", "check", "--all"]);
    assert_eq!(check.code, Some(0));
    assert_eq!(
        String::from_utf8_lossy(&check.stdout),
        format!(
            "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration {mjs} is ignored: {jsonc} is authoritative for the native cli-git. Remove the legacy file once no TypeScript cli-git reads it.\",\"path\":\"{mjs}\"}}\n\
             {{\"schemaVersion\":1,\"sequence\":1,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration {ts} is ignored: {jsonc} is authoritative for the native cli-git. Remove the legacy file once no TypeScript cli-git reads it.\",\"path\":\"{ts}\"}}\n",
            mjs = repo.join("cli-git.config.mjs").display(),
            ts = repo.join("cli-git.config.ts").display(),
            jsonc = repo.join(CONFIG_FILE_NAME).display()
        )
    );
    assert!(
        !String::from_utf8_lossy(&check.stderr).contains("egacy"),
        "{:?}",
        String::from_utf8_lossy(&check.stderr)
    );
    // Without the JSONC file the legacy files are a migration error on every command.
    std::fs::remove_file(repo.join(CONFIG_FILE_NAME)).expect("remove JSONC");
    for arguments in [
        vec!["add", "anything"],
        vec!["cli-git", "fix", "--all"],
        vec!["cli-git", "check", "--all"],
    ] {
        let migration: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(migration.code, Some(2), "{arguments:?}");
        let text: String = format!(
            "{}{}",
            String::from_utf8_lossy(&migration.stdout),
            String::from_utf8_lossy(&migration.stderr)
        );
        assert!(
            text.contains("\"code\":\"config-invalid\",\"message\":\"Legacy configuration "),
            "{arguments:?}: {text}"
        );
    }
    remove(&fixture);
}

/// The line a forbidden-strings content match prints,
///  as event number `sequence`.
fn forbidden_match(trigger: &str, sequence: u64, path: &str, line: u64) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"{trigger}\",\"policyId\":\"security/forbidden-strings\",\"severity\":\"error\",\"code\":\"security/forbidden-strings/forbidden-string\",\"message\":\"Forbidden string matched at line {line} (rule 0).\",\"path\":\"{path}\",\"fix\":\"none\"}}\n"
    );
}

/// The forbidden-strings policy scans what `git add` would stage and what a direct check
/// selects,
///  with rules from the configuration or `FORBIDDEN_STRINGS_RULES`,
///  and never
/// prints the matched text;
///  a named rules file that is missing stops the add.
#[test]
fn forbidden_strings_scan_candidates_from_each_rules_source() {
    let fixture: Fixture = fixture("forbidden-strings");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let needle: String = ["PLANTED", "BINARY", "NEEDLE"].join("_");
    std::fs::write(repo.join("a.txt"), format!("first\n{needle}\n")).expect("needle");
    std::fs::write(repo.join("clean.txt"), b"clean\n").expect("clean");
    std::fs::create_dir(repo.join("rules")).expect("rules directory");
    std::fs::write(repo.join("rules/private.txt"), format!("{needle}\n")).expect("rules");
    std::fs::write(
        repo.join(CONFIG_FILE_NAME),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false, \"rulesFile\": \"rules/private.txt\" }] } }\n",
    )
    .expect("configuration");
    // The configured rules file: the add stops, names the line and rule, and stages nothing.
    let stopped: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    assert_eq!(
        stopped,
        Observed {
            code: Some(1),
            stdout: Vec::<u8>::new(),
            stderr: forbidden_match("pre-forward", 0, "a.txt", 2).into_bytes(),
        }
    );
    // A clean file is staged.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["add", "--", "clean.txt"]),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    // A direct check reads the worktree and reports on standard output.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["cli-git", "check", "--", "a.txt"]
        ),
        Observed {
            code: Some(1),
            stdout: forbidden_match("direct-check", 0, "a.txt", 2).into_bytes(),
            stderr: Vec::<u8>::new(),
        }
    );
    // The variable, when the configuration names no file; relative to the top level.
    std::fs::write(
        repo.join(CONFIG_FILE_NAME),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false }] } }\n",
    )
    .expect("configuration without a file");
    let from_variable: Observed = observe(
        wrapped(&fixture)
            .env("FORBIDDEN_STRINGS_RULES", "rules/private.txt")
            .current_dir(&repo)
            .args(["add", "--", "a.txt"]),
        b"",
    );
    assert_eq!(
        from_variable.stderr,
        forbidden_match("pre-forward", 0, "a.txt", 2).into_bytes()
    );
    assert_eq!(from_variable.code, Some(1));
    // Neither, and no default file: the rules cannot load, so the add stops as incomplete.
    let missing: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    assert_eq!(missing.code, Some(2));
    let missing_text: String = String::from_utf8_lossy(&missing.stderr).into_owned();
    assert!(
        missing_text.contains("\"type\":\"engine-failure\",\"code\":\"policy-incomplete\""),
        "{missing_text}"
    );
    // No output of any run holds the matched text.
    for observed in [&stopped, &from_variable, &missing] {
        assert!(!String::from_utf8_lossy(&observed.stdout).contains(needle.as_str()));
        assert!(!String::from_utf8_lossy(&observed.stderr).contains(needle.as_str()));
    }
    // A configured name that leaves the repository is a configuration error before Git runs.
    std::fs::write(
        repo.join(CONFIG_FILE_NAME),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"rulesFile\": \"../outside.txt\" }] } }\n",
    )
    .expect("escaping configuration");
    let escaping: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    assert_eq!(escaping.code, Some(2));
    assert!(
        String::from_utf8_lossy(&escaping.stderr)
            .contains("rulesFile must name a file relative to the repository's top level that stays inside it, but the value has a . or .. component."),
        "{}",
        String::from_utf8_lossy(&escaping.stderr)
    );
    // Only the clean file was ever staged.
    assert_eq!(
        String::from_utf8_lossy(&status(&fixture, repo.as_path())),
        "A  clean.txt\n?? a.txt\n?? cli-git.config.jsonc\n?? rules/private.txt\n"
    );
    remove(&fixture);
}

/// `--no-enforce-security/forbidden-strings` skips the policy for the one command that carries it:
///  the add stages a file the policy would stop,
///  a commit without the flag still stops it,
///  and a commit with the flag records it.
#[test]
fn forbidden_strings_escape_skips_the_policy_for_one_command() {
    let fixture: Fixture = fixture("forbidden-strings-escape");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let needle: String = ["PLANTED", "ESCAPE", "NEEDLE"].join("_");
    std::fs::write(repo.join("a.txt"), format!("first\n{needle}\n")).expect("needle");
    std::fs::create_dir(repo.join("rules")).expect("rules directory");
    std::fs::write(repo.join("rules/private.txt"), format!("{needle}\n")).expect("rules");
    std::fs::write(
        repo.join(CONFIG_FILE_NAME),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false, \"rulesFile\": \"rules/private.txt\" }] } }\n",
    )
    .expect("configuration");
    // Without the flag the add stops and stages nothing.
    let stopped: Observed = run_wrapped(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    assert_eq!(stopped.code, Some(1));
    // With the flag the add stages the file and prints nothing.
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["add", "--no-enforce-security/forbidden-strings", "--", "a.txt"]
        ),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        String::from_utf8_lossy(&status(&fixture, repo.as_path())),
        "A  a.txt\n?? cli-git.config.jsonc\n?? rules/private.txt\n"
    );
    // The flag does not carry over: a commit without it stops on the staged file.
    let blocked: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=blocked", "--", "a.txt"],
    );
    assert_eq!(blocked.code, Some(1), "{}", String::from_utf8_lossy(&blocked.stderr));
    assert!(
        String::from_utf8_lossy(&blocked.stderr).contains("security/forbidden-strings"),
        "{}",
        String::from_utf8_lossy(&blocked.stderr)
    );
    // With the flag the commit records it.
    let committed: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &[
            "commit",
            "--quiet",
            "--no-enforce-security/forbidden-strings",
            "--message=escaped",
            "--",
            "a.txt",
        ],
    );
    assert_eq!(committed.code, Some(0), "{}", String::from_utf8_lossy(&committed.stderr));
    assert_eq!(
        String::from_utf8_lossy(&status(&fixture, repo.as_path())),
        "?? cli-git.config.jsonc\n?? rules/private.txt\n"
    );
    remove(&fixture);
}

/// A direct fix corrects the selected worktree files,
///  executable ones included,
///  reads
/// the worktree rather than the index,
///  reports only its summary,
///  and never changes the
/// index;
///  a listed dependent-version policy plans nothing for this worktree, so the fix proceeds.
#[test]
fn direct_fix_corrects_only_worktree_files() {
    let fixture: Fixture = fixture("direct-fix");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("t.txt"), b"t\n").expect("tracked");
    git(&fixture, repo.as_path(), &["add", "t.txt"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=tracked"],
    );
    std::fs::write(repo.join("t.txt"), b"t\n\n").expect("modified");
    std::fs::write(repo.join("u.txt"), b"u").expect("untracked");
    executable(repo.join("run.sh").as_path(), b"#!/bin/sh");
    // Staged without a final newline, canonical in the worktree: nothing to correct.
    std::fs::write(repo.join("s.txt"), b"s").expect("staged");
    git(&fixture, repo.as_path(), &["add", "s.txt"]);
    std::fs::write(repo.join("s.txt"), b"s\n").expect("worktree");
    let index_before: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    // The fix itself.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["cli-git", "fix", "--all"]),
        Observed {
            code: Some(0),
            stdout: b"{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\"passes\":1,\"changedPaths\":[\"run.sh\",\"t.txt\",\"u.txt\"]}\n".to_vec(),
            stderr: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before,
        "a direct fix never changes the index"
    );
    assert_eq!(std::fs::read(repo.join("t.txt")).expect("t"), b"t\n");
    assert_eq!(std::fs::read(repo.join("u.txt")).expect("u"), b"u\n");
    assert_eq!(std::fs::read(repo.join("s.txt")).expect("s"), b"s\n");
    assert_eq!(
        std::fs::read(repo.join("run.sh")).expect("run"),
        b"#!/bin/sh\n"
    );
    let mode: u32 = {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(repo.join("run.sh"))
            .expect("run")
            .permissions()
            .mode()
    };
    assert_eq!(mode & 0o100, 0o100, "{mode:o}");
    // Nothing is left beside the corrected files, and Git sees only the worktree change.
    let mut leftovers: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&repo).expect("worktree") {
        let name: String = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.starts_with(".cli-git-") {
            leftovers.push(name);
        }
    }
    assert_eq!(leftovers, Vec::<String>::new());
    assert_eq!(
        String::from_utf8_lossy(&status(&fixture, repo.as_path())),
        "AM s.txt\n?? run.sh\n?? u.txt\n"
    );
    // A second fix finds nothing to do.
    assert_eq!(
        run_wrapped(&fixture, repo.as_path(), &["cli-git", "fix", "--all"]),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    remove(&fixture);
}

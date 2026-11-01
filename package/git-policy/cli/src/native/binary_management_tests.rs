//! What:
//!  `git cli-git` controls through the built executable.
//! Why:
//!  The management namespace belongs to the wrapper:
//!  help and retired commands
//!      answer without Git,
//!  malformed scopes exit 2,
//!  and nothing in the namespace is
//!      ever handed to real Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['cli-git', '--help'])).toMatchObject({ status: 0, stdout: HELP });
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, bounded, fixture, observe, registered_transaction, remove, repository,
    wrapped,
};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The only configuration file the native wrapper reads.
const CONFIG_FILE_NAME: &str = "cli-git.config.jsonc";

/// Run `git <arguments>` through the wrapper in a directory.
fn run(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).current_dir(directory).args(arguments), b"");
}

/// Standard error as text for message assertions.
fn stderr_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stderr).into_owned();
}

/// Standard output as text for message assertions.
fn stdout_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stdout).into_owned();
}

/// Help is printed on standard output with exit status 0,
///  even when PATH holds no Git at all.
#[test]
fn help_needs_neither_git_nor_a_repository() {
    let fixture: Fixture = fixture("management-help");
    let only_wrapper: OsString = fixture.root.join("bin").into_os_string();
    for arguments in [
        vec!["cli-git", "--help"],
        vec!["-C", "/nonexistent", "cli-git", "-h"],
    ] {
        let observed: Observed = observe(
            bounded(
                &fixture,
                fixture.root.join("bin/git").as_path(),
                only_wrapper.as_os_str(),
            )
            .args(arguments.as_slice()),
            b"",
        );
        assert_eq!(observed.code, Some(0), "{arguments:?}");
        assert_eq!(observed.stderr, Vec::<u8>::new(), "{arguments:?}");
        let help: String = stdout_text(&observed);
        assert!(
            help.starts_with("Usage: git cli-git <command> [options]\n"),
            "{help}"
        );
        assert!(
            help.contains("  check    Check policies over an explicit scope.\n"),
            "{help}"
        );
        assert!(
            help.contains("  fix      Apply policy fixes over an explicit scope.\n"),
            "{help}"
        );
        assert!(
            help.contains("           1 when any other hook is found, 2 outside a repository.\n"),
            "{help}"
        );
    }
    remove(&fixture);
}

/// Malformed invocations exit 2 with their remedy on standard error and nothing on standard output.
#[test]
fn malformed_invocations_exit_two() {
    let fixture: Fixture = fixture("management-refusals");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    for (arguments, message) in [
        (
            vec!["cli-git", "check"],
            "git cli-git check requires exactly one scope: --all or non-empty pathspecs after --.\n",
        ),
        (
            vec!["cli-git", "check", "--all", "--", "a.txt"],
            "git cli-git check requires exactly one scope: --all or non-empty pathspecs after --.\n",
        ),
        (
            vec!["cli-git", "check", "a.txt"],
            "git cli-git check pathspecs must follow --.\n",
        ),
        (
            vec!["cli-git", "fix", "a.txt", "--all"],
            "git cli-git fix pathspecs must follow --.\n",
        ),
    ] {
        assert_eq!(
            run(&fixture, repo.as_path(), arguments.as_slice()),
            Observed {
                code: Some(2),
                stdout: Vec::<u8>::new(),
                stderr: message.as_bytes().to_vec(),
            },
            "{arguments:?}"
        );
    }
    for arguments in [
        vec!["cli-git"],
        vec!["cli-git", "unknown"],
        vec!["cli-git", "check", "--bogus", "--all"],
        vec!["cli-git", "check", "--policy"],
        vec!["cli-git", "--help", "extra"],
    ] {
        let observed: Observed = run(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(observed.code, Some(2), "{arguments:?}");
        assert_eq!(observed.stdout, Vec::<u8>::new(), "{arguments:?}");
        assert!(
            stderr_text(&observed).starts_with(
                "Usage: git cli-git check (--all | -- <pathspec>...) [--policy <id>]...\n"
            ),
            "{arguments:?}"
        );
    }
    remove(&fixture);
}

/// Retired trust commands explain the retirement,
///  exit 0,
///  and neither read nor write trust records.
#[test]
fn retired_trust_commands_explain_and_change_nothing() {
    let fixture: Fixture = fixture("management-retired");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    // A legacy executable configuration that would fail loudly if anything ran it.
    std::fs::write(repo.join("cli-git.config.mjs"), "process.exit(97);\n").expect("legacy");
    for (arguments, name) in [
        (vec!["cli-git", "trust"], "trust"),
        (vec!["cli-git", "trust", "--yes"], "trust"),
        (vec!["cli-git", "untrust"], "untrust"),
        (vec!["cli-git", "status"], "status"),
    ] {
        let observed: Observed = run(&fixture, repo.as_path(), arguments.as_slice());
        assert_eq!(
            observed,
            Observed {
                code: Some(0),
                stdout: Vec::<u8>::new(),
                stderr: format!(
                    "git cli-git {name} is retired. cli-git now reads cli-git.config.jsonc as \
                     data and runs no repository-supplied code, so there is no code execution to \
                     approve, revoke or report. Nothing was changed; existing trust records are \
                     left in place and are no longer read.\n"
                )
                .into_bytes(),
            },
            "{arguments:?}"
        );
    }
    let help: Observed = run(&fixture, repo.as_path(), &["cli-git", "trust", "--help"]);
    assert_eq!(help.code, Some(0));
    assert_eq!(help.stderr, Vec::<u8>::new());
    assert!(stdout_text(&help).starts_with("git cli-git trust is retired."));
    // The fixture home, where a trust registry would live, is still empty.
    assert_eq!(
        std::fs::read_dir(fixture.root.join("home"))
            .expect("fixture home")
            .count(),
        0
    );
    remove(&fixture);
}

/// The final-newline warning of a direct check about `file.txt`,
///  as event number `sequence`.
fn direct_warning(sequence: u64) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"direct-check\",\"policyId\":\"final-newline\",\"severity\":\"warn\",\"code\":\"final-newline/noncanonical-final-newline\",\"message\":\"Non-empty text file must end with exactly one LF byte.\",\"path\":\"file.txt\",\"fix\":\"none\"}}\n"
    );
}

/// A direct command reports configuration problems as events on standard output,
///  runs the
/// ported policies over the selected worktree files,
///  and refuses what it cannot do yet.
#[test]
fn direct_commands_validate_run_ported_policies_and_refuse_the_rest() {
    let fixture: Fixture = fixture("management-direct");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    std::fs::write(repo.join("file.txt"), b"no final newline").expect("file");
    // Well-formed, from another directory through the global prefix: the content policy
    // reads the selected worktree file and reports it on standard output.
    let repo_text: String = repo.to_string_lossy().into_owned();
    for arguments in [
        vec!["-C", repo_text.as_str(), "cli-git", "check", "--all"],
        vec![
            "-C",
            repo_text.as_str(),
            "cli-git",
            "check",
            "--policy",
            "final-newline",
            "--",
            "file.txt",
        ],
    ] {
        assert_eq!(
            run(&fixture, fixture.root.as_path(), arguments.as_slice()),
            Observed {
                code: Some(0),
                stdout: direct_warning(0).into_bytes(),
                stderr: Vec::<u8>::new(),
            },
            "{arguments:?}"
        );
    }
    // A ported policy answers for real: clean at the top level, a finding on standard output below it.
    assert_eq!(
        run(
            &fixture,
            repo.as_path(),
            &["cli-git", "check", "--all", "--policy", "require-root"]
        ),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    let finding: String = format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"direct-check\",\"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\",\"message\":\"cli-git: not at the root of the git repository. Repo root is {root} but effective cwd is {root}/nested. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to {root} or pass -C {root} before the subcommand.\",\"fix\":\"none\"}}\n",
        root = repo.display()
    );
    assert_eq!(
        run(
            &fixture,
            nested.as_path(),
            &["cli-git", "check", "--all", "--policy", "require-root"]
        ),
        Observed {
            code: Some(1),
            stdout: finding.clone().into_bytes(),
            stderr: Vec::<u8>::new(),
        }
    );
    // The first error stops the pass; keep-going written before the namespace reaches the
    // content policy, which reports after the finding.
    assert_eq!(
        run(&fixture, nested.as_path(), &["cli-git", "check", "--all"]),
        Observed {
            code: Some(1),
            stdout: finding.clone().into_bytes(),
            stderr: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        run(
            &fixture,
            nested.as_path(),
            &["--cli-git-keep-going", "cli-git", "check", "--all"]
        ),
        Observed {
            code: Some(1),
            stdout: format!("{finding}{}", direct_warning(1)).into_bytes(),
            stderr: Vec::<u8>::new(),
        }
    );
    // An escape written before the namespace skips its policy.
    assert_eq!(
        run(
            &fixture,
            nested.as_path(),
            &[
                "--no-enforce-require-root",
                "cli-git",
                "check",
                "--all",
                "--policy",
                "require-root"
            ]
        ),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    // A direct fix has no command policy to run.
    assert_eq!(
        run(
            &fixture,
            repo.as_path(),
            &["cli-git", "fix", "--all", "--policy", "require-root"]
        ),
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    // The content policy corrects the worktree file and reports only the summary.
    let index_before: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    assert_eq!(
        run(&fixture, repo.as_path(), &["cli-git", "fix", "--all"]),
        Observed {
            code: Some(0),
            stdout: b"{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\"passes\":1,\"changedPaths\":[\"file.txt\"]}\n".to_vec(),
            stderr: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        std::fs::read(repo.join("file.txt")).expect("fixed file"),
        b"no final newline\n"
    );
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before,
        "a direct fix changes only the worktree"
    );
    std::fs::write(repo.join("file.txt"), b"no final newline").expect("file again");
    // An unknown selected policy: one config-invalid event on standard output.
    let unknown: Observed = run(
        &fixture,
        repo.as_path(),
        &["cli-git", "check", "--all", "--policy", "no-such-policy"],
    );
    assert_eq!(unknown.code, Some(2));
    assert_eq!(unknown.stderr, Vec::<u8>::new());
    assert!(
        stdout_text(&unknown).starts_with(
            "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"Unknown policy ID: no-such-policy. Shipped policies: require-root, "
        ),
        "{}",
        stdout_text(&unknown)
    );
    assert_eq!(stdout_text(&unknown).matches('\n').count(), 1);
    // Invalid repository configuration: the same event shape, naming the file and key.
    let source: PathBuf = repo.join(CONFIG_FILE_NAME);
    std::fs::write(&source, r#"{ "plugins": {} }"#).expect("invalid config");
    let invalid: Observed = run(&fixture, repo.as_path(), &["cli-git", "fix", "--all"]);
    assert_eq!(invalid.code, Some(2));
    assert_eq!(invalid.stderr, Vec::<u8>::new());
    assert!(
        stdout_text(&invalid).starts_with(
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"{}: Configuration key plugins is retired",
                source.display()
            )
            .as_str()
        ),
        "{}",
        stdout_text(&invalid)
    );
    // A live commit transaction no longer stops a direct command: the check holds the landing lock.
    std::fs::write(&source, "{}").expect("valid config");
    let _registry: PathBuf = registered_transaction(repo.as_path(), true);
    let beside_live: Observed = run(
        &fixture,
        repo.as_path(),
        &["cli-git", "check", "--all", "--policy", "require-root"],
    );
    assert_eq!(beside_live.code, Some(0), "{}", String::from_utf8_lossy(&beside_live.stderr).into_owned());
    assert!(
        !String::from_utf8_lossy(&beside_live.stderr).into_owned().contains("recovering or waiting for the commit transactions"),
        "{}",
        String::from_utf8_lossy(&beside_live.stderr).into_owned()
    );
    // Nothing a direct command did changed the file.
    assert_eq!(
        std::fs::read(repo.join("file.txt")).expect("file"),
        b"no final newline"
    );
    remove(&fixture);
}

/// The namespace word is only a command in the subcommand position;
///  elsewhere it is an ordinary argument.
#[test]
fn namespace_word_elsewhere_is_an_ordinary_argument() {
    let fixture: Fixture = fixture("management-word");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("cli-git"), b"a file named like the namespace\n").expect("file");
    // An inspection command naming the file is forwarded to Git unchanged.
    let listed: Observed = run(
        &fixture,
        repo.as_path(),
        &["ls-files", "--others", "--", "cli-git"],
    );
    assert_eq!(
        listed,
        Observed {
            code: Some(0),
            stdout: b"cli-git\n".to_vec(),
            stderr: Vec::<u8>::new(),
        }
    );
    // `git add cli-git` is an `add`: its content is checked and Git stages the file.
    let add: Observed = run(&fixture, repo.as_path(), &["add", "cli-git"]);
    assert_eq!(
        add,
        Observed {
            code: Some(0),
            stdout: Vec::<u8>::new(),
            stderr: Vec::<u8>::new(),
        }
    );
    let staged: Observed = run(&fixture, repo.as_path(), &["ls-files", "--", "cli-git"]);
    assert_eq!(staged.stdout, b"cli-git\n".to_vec());
    remove(&fixture);
}

/// `git cli-git doctor` lists the traditional hooks of the default and the configured hooks directory, one path per
///  line, and exits 1 when it finds any and 0 when it finds none; samples are ignored.
#[test]
fn doctor_lists_traditional_hooks_and_exits_by_what_it_finds() {
    let fixture: Fixture = fixture("management-doctor");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    super::support::executable(repo.join(".git/hooks/pre-commit.sample").as_path(), b"#!/bin/sh\n");
    let clean: Observed = run(&fixture, repo.as_path(), &["cli-git", "doctor"]);
    assert_eq!(clean.code, Some(0), "{}", stderr_text(&clean));
    assert_eq!(stdout_text(&clean), "");
    assert_eq!(stderr_text(&clean), "");
    super::support::executable(repo.join(".git/hooks/commit-msg").as_path(), b"#!/bin/sh\n");
    std::fs::create_dir(repo.join("custom")).expect("custom hooks");
    super::support::executable(repo.join("custom/pre-commit").as_path(), b"#!/bin/sh\n");
    super::support::git(&fixture, repo.as_path(), &["config", "core.hooksPath", "custom"]);
    let found: Observed = run(&fixture, repo.as_path(), &["cli-git", "doctor"]);
    assert_eq!(found.code, Some(1), "{}", stderr_text(&found));
    assert_eq!(
        stdout_text(&found),
        format!("{}\n{}\n", repo.join(".git/hooks/commit-msg").display(), repo.join("custom/pre-commit").display())
    );
    assert!(stderr_text(&found).contains("unsupported"), "{}", stderr_text(&found));
    let refused: Observed = run(&fixture, repo.as_path(), &["cli-git", "doctor", "--all"]);
    assert_eq!(refused.code, Some(2), "{}", stderr_text(&refused));
    remove(&fixture);
}

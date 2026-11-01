//! What:
//!  Decision controls for one wrapper invocation against real Git.
//! Why:
//!  Inspection commands must be forwarded without reading configuration,
//!  every
//!      other command must validate configuration and pass its policies or stop,
//!  and
//!      the management namespace must never reach Git.
//!  The invocations that stop before
//!      a Git is trusted are controlled in `entry_stop_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(planInvocation(['-C', repo, 'status'], env, inputs)).toEqual({ kind: 'forward', realGit: '/usr/bin/git', ... });
//! ```
#![cfg(unix)]

/// Import the decision under test and shared fixtures.
use crate::config_schema::DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS;
use super::{MANAGEMENT_COMMAND, plan_invocation};
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE};
use crate::child_environment::FORWARD_TARGET_VARIABLE;
use crate::config_file::CONFIG_FILE_NAME;
use crate::management_arguments::MANAGEMENT_HELP;
use crate::real_git::{Platform, ResolutionInputs};
use crate::test_support::{REAL_GIT, executable, fixture, remove, repository};

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  Inputs selecting the image's real Git,
///  with a fixture file standing in for this wrapper.
/// Why:
///  `pub(super)` lets the sibling stop controls in `entry_stop_tests.rs` reuse these helpers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function inputs(root: string): ResolutionInputs { /* ... */ }
/// ```
pub(super) fn inputs(root: &Path) -> ResolutionInputs {
    let own: PathBuf = root.join("own-wrapper");
    executable(own.as_path(), b"\x7fELF fixture wrapper");
    return ResolutionInputs {
        platform: Platform::Unix,
        path: OsString::from("/usr/bin"),
        path_extensions: OsString::new(),
        current_directory: root.to_path_buf(),
        common_paths: Vec::<PathBuf>::new(),
        own_executable: own,
    };
}

/// Build owned arguments from text.
pub(super) fn text(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// Build `-C <directory> <rest...>` as owned arguments.
fn in_directory(directory: &Path, rest: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> =
        vec![OsString::from("-C"), directory.as_os_str().to_os_string()];
    result.extend(text(rest));
    return result;
}

/// The forward action for these fixtures:
///  real Git,
///  the lock-PID overlay and no warning.
pub(super) fn forward(arguments: Vec<OsString>) -> Action {
    return Action::Forward {
        real_git: PathBuf::from(REAL_GIT),
        arguments,
        overlay: vec![
            (OsString::from("GIT_CONFIG_COUNT"), OsString::from("1")),
            (
                OsString::from("GIT_CONFIG_KEY_0"),
                OsString::from("core.lockfilePid"),
            ),
            (OsString::from("GIT_CONFIG_VALUE_0"), OsString::from("true")),
            (
                OsString::from(FORWARD_TARGET_VARIABLE),
                OsString::from(REAL_GIT),
            ),
        ],
        stderr: String::new(),
    };
}

/// Return the stderr of a stop action with exit code 2 and empty stdout;
///  a forward fails the test.
pub(super) fn stopped(action: Action) -> String {
    match action {
        Action::Exit {
            code,
            stdout,
            stderr,
        } => {
            assert_eq!(code, ENGINE_FAILURE_EXIT_CODE);
            assert_eq!(stdout, "", "wrapped commands report on standard error");
            return stderr;
        }
        Action::Forward { .. } => panic!("expected a stop action, got {action:?}"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
    }
}

/// What:
///  Check that a planned `git add` in a repository is coordinated,
///  with the given arguments,
///  warnings,
///  real Git and lock-PID overlay,
///  and the default unproven-owner budget.
/// Why:
///   The identity and the lease are the repository's own,
///  so only the fixed fields are compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(plan).toMatchObject({ kind: 'forward-coordinated', args, stderr, unprovenOwnerTimeoutMs: 1000 });
/// ```
fn assert_coordinated_add(action: Action, arguments: &[OsString], warnings: &str) {
    let Action::ForwardCoordinated { forward: planned, stderr } = action else {
        panic!("an index writer is coordinated, got a different action");
    };
    assert_eq!(stderr, warnings);
    assert_eq!(planned.arguments.as_slice(), arguments);
    assert_eq!(planned.real_git, PathBuf::from(REAL_GIT));
    let Action::Forward {
        overlay: expected_overlay,
        ..
    } = forward(arguments.to_vec())
    else {
        panic!("the fixture is a plain forward");
    };
    assert_eq!(planned.overlay, expected_overlay);
    assert_eq!(planned.timeout_ms, DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS);
}

/// The final-newline warning of `git add` about `file`.
const FILE_WARNING: &str = "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"final-newline\",\"severity\":\"warn\",\"code\":\"final-newline/noncanonical-final-newline\",\"message\":\"Non-empty text file must end with exactly one LF byte.\",\"hint\":\"Hint: if you are committing or checking this file, edit it so it ends with exactly one newline, then stage it again.\",\"path\":\"file\",\"fix\":\"none\"}\n";

/// Native queries other than version and help (those are in `entry_announced_tests.rs`),
///  option errors and a bare `git` are forwarded exactly as written.
#[test]
fn invocations_git_answers_itself_are_forwarded_unchanged() {
    let root: PathBuf = fixture("entry-forward");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for arguments in [
        vec!["--exec-path"],
        vec!["--html-path"],
        vec!["--no-such-global-option", "commit"],
        vec!["-C"],
        vec![],
        vec!["--no-pager"],
        // The namespace word after a query is an ordinary argument.
        vec!["--man-path", "cli-git"],
    ] {
        assert_eq!(
            plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution),
            forward(text(arguments.as_slice())),
            "{arguments:?}"
        );
    }
    remove(root.as_path());
}

/// Inspection commands are forwarded with their fixed transform,
///  and controls never reach Git.
#[test]
fn inspection_commands_are_forwarded_without_wrapper_controls() {
    let root: PathBuf = fixture("entry-inspection");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(repo.join("nested")).expect("nested directory");
    let resolution: ResolutionInputs = inputs(root.as_path());
    for (rest, forwarded) in [
        (
            vec!["status"],
            vec!["-c", "advice.statusHints=false", "status"],
        ),
        (vec!["log", "--oneline"], vec!["log", "--oneline"]),
        (vec!["branch", "--list"], vec!["branch", "--list"]),
        // The namespace word in any position but the subcommand is an ordinary argument.
        (vec!["log", "cli-git"], vec!["log", "cli-git"]),
        (
            vec!["status", "--cli-git-keep-going", "--short"],
            vec!["-c", "advice.statusHints=false", "status", "--short"],
        ),
    ] {
        assert_eq!(
            plan_invocation(
                in_directory(repo.as_path(), rest.as_slice()).as_slice(),
                &[],
                &resolution
            ),
            forward(in_directory(repo.as_path(), forwarded.as_slice())),
            "{rest:?}"
        );
    }
    // A control written before the subcommand is removed there too.
    let mut leading: Vec<OsString> = text(&["--cli-git-keep-going"]);
    leading.extend(in_directory(repo.as_path(), &["log"]));
    assert_eq!(
        plan_invocation(leading.as_slice(), &[], &resolution),
        forward(in_directory(repo.as_path(), &["log"]))
    );
    // Below the top level require-root stops the same command, with or without the control.
    for prefix in [vec![], vec!["--cli-git-keep-going"]] {
        let mut arguments: Vec<OsString> = text(prefix.as_slice());
        arguments.extend(in_directory(repo.join("nested").as_path(), &["status"]));
        match plan_invocation(arguments.as_slice(), &[], &resolution) {
            Action::Exit {
                code,
                stdout,
                stderr,
            } => {
                assert_eq!(code, 1, "{prefix:?}");
                assert_eq!(stdout, "", "{prefix:?}");
                assert!(
                    stderr.starts_with(
                        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\","
                    ),
                    "{prefix:?}: {stderr}"
                );
                assert_eq!(stderr.matches('\n').count(), 1, "{prefix:?}: {stderr}");
            }
            Action::Forward { .. } => {
                panic!("{prefix:?}: a command below the top level is stopped")
            }
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
        }
    }
    remove(root.as_path());
}

/// `git cli-git ...` is answered by the wrapper,
///  also behind global options,
///  and never forwarded.
#[test]
fn management_namespace_is_answered_by_the_wrapper() {
    let root: PathBuf = fixture("entry-management");
    let mut resolution: ResolutionInputs = inputs(root.as_path());
    // No Git on PATH: help must not need it.
    resolution.path = OsString::from("nothing-here");
    assert_eq!(MANAGEMENT_COMMAND, "cli-git");
    for arguments in [
        vec!["cli-git", "--help"],
        vec!["-C", "/somewhere", "-c", "a.b=c", "cli-git", "-h"],
        vec!["--no-pager", "cli-git", "--help"],
        // A wrapper control before the namespace word does not hide it.
        vec!["--cli-git-keep-going", "cli-git", "--help"],
        vec![
            "-C",
            "/somewhere",
            "--no-enforce-require-root",
            "cli-git",
            "-h",
        ],
    ] {
        assert_eq!(
            plan_invocation(text(arguments.as_slice()).as_slice(), &[], &resolution),
            Action::Exit {
                code: 0,
                stdout: String::from(MANAGEMENT_HELP),
                stderr: String::new(),
            },
            "{arguments:?}"
        );
    }
    // A control after the namespace word is not part of the management grammar.
    match plan_invocation(
        text(&["cli-git", "--cli-git-keep-going", "check", "--all"]).as_slice(),
        &[],
        &resolution,
    ) {
        Action::Exit { code, stdout, .. } => {
            assert_eq!(code, 2);
            assert_eq!(stdout, "");
        }
        Action::Forward { .. } => panic!("the management namespace is never forwarded"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
    }
    // The global prefix reaches the direct command: it selects the repository whose file is read.
    resolution.path = OsString::from("/usr/bin");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(CONFIG_FILE_NAME), r#"{ "unknown": 1 }"#).expect("invalid config");
    match plan_invocation(
        in_directory(repo.as_path(), &["cli-git", "check", "--all"]).as_slice(),
        &[],
        &resolution,
    ) {
        Action::Exit {
            code,
            stdout,
            stderr,
        } => {
            assert_eq!(code, 2);
            assert!(stdout.contains("\"code\":\"config-invalid\""), "{stdout}");
            assert!(
                stdout.contains("Unknown configuration key: unknown."),
                "{stdout}"
            );
            assert_eq!(stderr, "");
        }
        Action::Forward { .. } => panic!("the management namespace is never forwarded"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
    }
    remove(root.as_path());
}

/// An inspection command never reads repository configuration,
///  even when it is invalid.
#[test]
fn inspection_commands_do_not_load_configuration() {
    let root: PathBuf = fixture("entry-skip-config");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(CONFIG_FILE_NAME), "not jsonc at all").expect("invalid config");
    std::fs::write(repo.join("cli-git.config.ts"), "throw 1;").expect("legacy config");
    let resolution: ResolutionInputs = inputs(root.as_path());
    assert_eq!(
        plan_invocation(
            in_directory(repo.as_path(), &["log"]).as_slice(),
            &[],
            &resolution
        ),
        forward(in_directory(repo.as_path(), &["log"]))
    );
    // The same repository stops a configuration-requiring command on that invalid file.
    let message: String = stopped(plan_invocation(
        in_directory(repo.as_path(), &["add", "file"]).as_slice(),
        &[],
        &resolution,
    ));
    assert!(message.contains("JSONC syntax error"), "{message}");
    remove(root.as_path());
}

/// A configuration-requiring command validates configuration,
///  then runs its policies.
#[test]
fn policy_commands_validate_configuration_then_run_policies() {
    let root: PathBuf = fixture("entry-required");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let resolution: ResolutionInputs = inputs(root.as_path());
    let arguments: Vec<OsString> = in_directory(repo.as_path(), &["add", "file"]);
    let source: PathBuf = repo.join(CONFIG_FILE_NAME);
    let legacy: PathBuf = repo.join("cli-git.config.ts");
    std::fs::write(repo.join("file"), b"no final newline").expect("file");
    // No configuration file: the defaults apply, and the built-in content policy reads
    // what the add would stage and warns before Git runs.
    assert_coordinated_add(
        plan_invocation(arguments.as_slice(), &[], &resolution),
        arguments.as_slice(),
        FILE_WARNING,
    );
    // A valid file is accepted and its settings are used.
    std::fs::write(&source, r#"{ "policies": { "final-newline": "off" } }"#).expect("valid config");
    assert_coordinated_add(
        plan_invocation(arguments.as_slice(), &[], &resolution),
        arguments.as_slice(),
        "",
    );
    // A legacy file beside it is not mentioned by an ordinary command: only
    // `git cli-git check` reports it.
    std::fs::write(&legacy, "export default {};").expect("legacy config");
    assert_coordinated_add(
        plan_invocation(arguments.as_slice(), &[], &resolution),
        arguments.as_slice(),
        "",
    );
    // An invalid key is exactly one config-invalid event and no refusal notice.
    std::fs::write(&source, r#"{ "policies": { "unknown": "off" } }"#).expect("invalid config");
    assert_eq!(
        stopped(plan_invocation(arguments.as_slice(), &[], &resolution)),
        format!(
            "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"{}: Unknown policy ID: unknown. Shipped policies: require-root, linked-worktree-only, branch-worktree-only, add-explicit, final-newline, markdown/autofix, mono/forbidden-root-context, mono/dependent-version-bump, security/forbidden-strings.\",\"hint\":\"Hint: if you just edited cli-git.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}}\n",
            source.display()
        )
    );
    // A legacy file alone demands migration, as a config-invalid event.
    std::fs::remove_file(&source).expect("remove JSONC");
    let migration: String = stopped(plan_invocation(arguments.as_slice(), &[], &resolution));
    assert!(
        migration.starts_with(
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"Legacy configuration {} is not executed or read by the native cli-git.",
                legacy.display()
            )
            .as_str()
        ),
        "{migration}"
    );
    assert!(migration.ends_with("\"}\n"), "{migration}");
    assert_eq!(migration.matches('\n').count(), 1);
    remove(root.as_path());
}

/// Outside a worktree there is no configuration file:
///  the defaults apply and Git answers.
#[test]
fn policy_commands_outside_a_worktree_use_defaults() {
    let root: PathBuf = fixture("entry-outside");
    let resolution: ResolutionInputs = inputs(root.as_path());
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    // An invalid file outside any repository is never read.
    std::fs::write(plain.join(CONFIG_FILE_NAME), "invalid").expect("stray file");
    for rest in [vec!["init"], vec!["add", "file"]] {
        assert_eq!(
            plan_invocation(
                in_directory(plain.as_path(), rest.as_slice()).as_slice(),
                &[],
                &resolution,
            ),
            forward(in_directory(plain.as_path(), rest.as_slice())),
            "{rest:?}"
        );
    }
    // A commit outside a repository ends on Git's own refusal to find one, and creates nothing.
    match plan_invocation(
        in_directory(plain.as_path(), &["commit", "-m", "x"]).as_slice(),
        &[],
        &resolution,
    ) {
        Action::Exit { code, .. } => assert_eq!(code, 1),
        other => panic!("expected the commit to end, got {other:?}"),
    }
    assert!(!plain.join(".git").exists());
    remove(root.as_path());
}

/// A repository whose configuration names the rules file, with a linked worktree `linked`.
fn repository_with_rules_file(root: &Path) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(
        repo.join(CONFIG_FILE_NAME),
        "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"rulesFile\": \".cache/forbidden-strings.rules.txt\" }] } }\n",
    )
    .expect("configuration");
    crate::test_support::git(repo.as_path(), &["add", "--", CONFIG_FILE_NAME]);
    crate::test_support::git(repo.as_path(), &["commit", "--quiet", "--message=configure"]);
    let linked: PathBuf = root.join("linked");
    crate::test_support::git(
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", "../linked"],
    );
    return (repo, linked);
}

/// The settled-forward facts of a plan, or a failure naming the action that was planned instead.
fn settled_facts(action: Action) -> crate::worktree_copy_forward::CopyFacts {
    match action {
        Action::ForwardSettled { copy, .. } => return *copy,
        other => panic!("expected a settled forward, got {other:?}"),
    }
}

/// From a linked worktree every forwarded command is settled by the copy:
///  a worktree creation (also through an alias) creates, anything else only recovers.
#[test]
fn commands_in_a_linked_worktree_are_settled_by_the_copy() {
    let root: PathBuf = fixture("entry-settled-linked");
    let (_repo, linked): (PathBuf, PathBuf) = repository_with_rules_file(root.as_path());
    let resolution: ResolutionInputs = inputs(root.as_path());
    for (rest, creates) in [
        (vec!["worktree", "add", "--quiet", "-b", "second", "../second"], true),
        (vec!["-c", "alias.wta=worktree add", "wta", "-b", "second", "../second"], true),
        (vec!["worktree", "move", "../second", "../third"], true),
        (vec!["worktree", "list"], false),
        (vec!["log", "--oneline"], false),
    ] {
        let facts = settled_facts(plan_invocation(
            in_directory(linked.as_path(), rest.as_slice()).as_slice(),
            &[],
            &resolution,
        ));
        assert_eq!(facts.creates_or_moves, creates, "{rest:?}");
        assert!(matches!(facts.identity, crate::worktree_identity::WorktreeIdentity::LinkedWorktree { .. }));
        assert!(facts.main_checkout_paths.is_empty(), "a linked source copies everything it ignores");
    }
    // The inherited environment is handed on, so a nested command can present an outer lease.
    let environment: Vec<(OsString, OsString)> = vec![(
        OsString::from("CLI_GIT_WORKTREE_COPY_LEASE"),
        OsString::from("token"),
    )];
    let nested = settled_facts(plan_invocation(
        in_directory(linked.as_path(), &["status"]).as_slice(),
        environment.as_slice(),
        &resolution,
    ));
    assert_eq!(nested.environment, environment);
    remove(root.as_path());
}

/// The opt-out, and Git's help and version forms, keep the plain forward even from a linked worktree.
#[test]
fn the_opt_out_and_short_circuit_forms_are_not_settled() {
    let root: PathBuf = fixture("entry-settled-opt-out");
    let (_repo, linked): (PathBuf, PathBuf) = repository_with_rules_file(root.as_path());
    let resolution: ResolutionInputs = inputs(root.as_path());
    assert_eq!(
        plan_invocation(
            in_directory(
                linked.as_path(),
                &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "second", "../second"]
            )
            .as_slice(),
            &[],
            &resolution
        ),
        forward(in_directory(
            linked.as_path(),
            &["worktree", "add", "--quiet", "-b", "second", "../second"]
        ))
    );
    // Commands that need no repository are never asked about their location.
    for rest in [vec!["version"], vec!["help", "worktree"], vec!["config", "--global", "--list"]] {
        assert_eq!(
            plan_invocation(
                in_directory(linked.as_path(), rest.as_slice()).as_slice(),
                &[],
                &resolution
            ),
            forward(in_directory(linked.as_path(), rest.as_slice())),
            "{rest:?}"
        );
    }
    assert_eq!(
        plan_invocation(
            in_directory(linked.as_path(), &["--exec-path"]).as_slice(),
            &[],
            &resolution
        ),
        forward(in_directory(linked.as_path(), &["--exec-path"]))
    );
    remove(root.as_path());
}

/// From the main checkout, only a worktree creation with a configured rules file is settled,
///  and it hands over exactly that file.
#[test]
fn the_main_checkout_is_settled_only_for_a_creation_with_a_rules_file() {
    let root: PathBuf = fixture("entry-settled-main");
    let (repo, _linked): (PathBuf, PathBuf) = repository_with_rules_file(root.as_path());
    let resolution: ResolutionInputs = inputs(root.as_path());
    for rest in [
        vec!["worktree", "add", "--quiet", "-b", "second", "../second"],
        vec!["-c", "alias.wta=worktree add", "wta", "-b", "second", "../second"],
        vec!["worktree", "move", "../linked", "../moved"],
    ] {
        let facts = settled_facts(plan_invocation(
            in_directory(repo.as_path(), rest.as_slice()).as_slice(),
            &[],
            &resolution,
        ));
        assert!(facts.creates_or_moves, "{rest:?}");
        assert_eq!(
            facts.main_checkout_paths,
            vec![String::from(".cache/forbidden-strings.rules.txt")],
            "{rest:?}"
        );
    }
    // Commands that create nothing keep their plain forward from the main checkout.
    for rest in [vec!["worktree", "list"], vec!["status"], vec!["log", "--oneline"]] {
        assert!(
            matches!(
                plan_invocation(
                    in_directory(repo.as_path(), rest.as_slice()).as_slice(),
                    &[],
                    &resolution
                ),
                Action::Forward { .. }
            ),
            "{rest:?}"
        );
    }
    remove(root.as_path());
}

/// A main checkout whose configuration names no rules file stays a plain forward for a creation too.
#[test]
fn a_main_checkout_without_a_rules_file_is_forwarded_plainly() {
    let root: PathBuf = fixture("entry-settled-main-unconfigured");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let resolution: ResolutionInputs = inputs(root.as_path());
    let arguments: Vec<&str> = vec!["worktree", "add", "--quiet", "-b", "second", "../second"];
    assert_eq!(
        plan_invocation(
            in_directory(repo.as_path(), arguments.as_slice()).as_slice(),
            &[],
            &resolution
        ),
        forward(in_directory(repo.as_path(), arguments.as_slice()))
    );
    remove(root.as_path());
}

/// An index writer in a linked worktree is coordinated and also carries the copy,
///  while the same writer in the main checkout carries none.
#[test]
fn an_index_writer_carries_the_copy_only_from_a_linked_worktree() {
    let root: PathBuf = fixture("entry-coordinated-copy");
    let (repo, linked): (PathBuf, PathBuf) = repository_with_rules_file(root.as_path());
    let resolution: ResolutionInputs = inputs(root.as_path());
    for directory in [linked.as_path(), repo.as_path()] {
        std::fs::write(directory.join("file"), b"content\n").expect("file");
        std::fs::create_dir_all(directory.join(".cache")).expect("cache directory");
        std::fs::write(
            directory.join(".cache/forbidden-strings.rules.txt"),
            b"UNMATCHED_RULE_TEXT\n",
        )
        .expect("rules file the scanner needs");
    }
    for (directory, carries) in [(linked.as_path(), true), (repo.as_path(), false)] {
        match plan_invocation(
            in_directory(directory, &["add", "--", "file"]).as_slice(),
            &[],
            &resolution,
        ) {
            Action::ForwardCoordinated { forward: planned, .. } => {
                assert_eq!(planned.copy.is_some(), carries, "{directory:?}");
            }
            other => panic!("expected a coordinated forward, got {other:?}"),
        }
    }
    remove(root.as_path());
}

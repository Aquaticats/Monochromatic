//! What:
//!  Decision controls for `git cli-git` against real Git.
//! Why:
//!  Help and retired commands must answer without Git or a repository;
//!  direct
//!      commands must reject unknown policies and invalid configuration on standard
//!      output with exit status 2,
//!  run the policies that are ported,
//!  and refuse instead
//!      of reporting a clean result for a policy that cannot read its files yet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(planManagement(['--help'], [], controls, env, inputs)).toEqual({ kind: 'exit', code: 0, stdout: HELP, stderr: '' });
//! ```
#![cfg(unix)]

/// Import the decision under test and shared fixtures.
use super::{plan_management, retired_explanation};
use crate::action::Action;
use crate::config_file::CONFIG_FILE_NAME;
use crate::management_arguments::{MANAGEMENT_HELP, MANAGEMENT_USAGE, RetiredCommand};
use crate::policy_registry::PolicyId;
use crate::real_git::{Platform, ResolutionInputs};
use crate::test_support::{executable, fixture, remove, repository};
use crate::wrapper_controls::{Controls, no_controls};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Inputs whose PATH holds nothing:
///  any attempt to resolve Git fails visibly.
fn no_git(root: &Path) -> ResolutionInputs {
    return ResolutionInputs {
        platform: Platform::Unix,
        path: OsString::from("empty-directory"),
        path_extensions: OsString::new(),
        current_directory: root.to_path_buf(),
        common_paths: Vec::<PathBuf>::new(),
        own_executable: root.join("own-wrapper"),
    };
}

/// Inputs selecting the image's real Git.
fn with_git(root: &Path) -> ResolutionInputs {
    let own: PathBuf = root.join("own-wrapper");
    executable(own.as_path(), b"\x7fELF fixture wrapper");
    let mut inputs: ResolutionInputs = no_git(root);
    inputs.path = OsString::from("/usr/bin");
    inputs.own_executable = own;
    return inputs;
}

/// Plan management text arguments with `-C <directory>` as the Git global prefix and these controls.
fn plan_with(
    values: &[&str],
    directory: &Path,
    controls: &Controls,
    inputs: &ResolutionInputs,
) -> Action {
    let mut arguments: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        arguments.push(OsString::from(value));
    }
    let global: Vec<OsString> = vec![OsString::from("-C"), directory.as_os_str().to_os_string()];
    return plan_management(
        arguments.as_slice(),
        global.as_slice(),
        controls,
        &[],
        inputs,
    );
}

/// Plan management text arguments without any wrapper control before `cli-git`.
fn plan(values: &[&str], directory: &Path, inputs: &ResolutionInputs) -> Action {
    return plan_with(values, directory, &no_controls(), inputs);
}

/// Build the expected stop action.
fn exit(code: i32, stdout: &str, stderr: &str) -> Action {
    return Action::Exit {
        code,
        stdout: String::from(stdout),
        stderr: String::from(stderr),
    };
}

/// The fix summary of a direct fix that corrected `a.txt` in one pass.
const FIXED_A: &str = "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\"passes\":1,\"changedPaths\":[\"a.txt\"]}\n";

/// The final-newline warning of a direct check about `a.txt`,
///  as event number `sequence`.
fn final_newline_warning(sequence: u64) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"direct-check\",\"policyId\":\"final-newline\",\"severity\":\"warn\",\"code\":\"final-newline/noncanonical-final-newline\",\"message\":\"Non-empty text file must end with exactly one LF byte.\",\"hint\":\"Hint: if you are committing or checking this file, edit it so it ends with exactly one newline, then stage it again.\",\"path\":\"a.txt\",\"fix\":\"none\"}}\n"
    );
}

/// The standard output of a direct command whose scope could not be projected,
///  which the
/// control requires to be one `transaction-failed` event for `trigger`,
///  its message
/// starting with `message`.
fn projection_failed(action: Action, trigger: &str, message: &str) {
    match action {
        Action::Exit {
            code,
            stdout,
            stderr,
        } => {
            assert_eq!(code, 2);
            assert_eq!(stderr, "");
            let head: String = format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"transaction-failed\",\"message\":\"{message}"
            );
            assert!(stdout.starts_with(head.as_str()), "{stdout}");
            assert!(
                stdout.ends_with(format!("\",\"trigger\":\"{trigger}\"}}\n").as_str()),
                "{stdout}"
            );
            assert_eq!(stdout.matches('\n').count(), 1, "{stdout}");
        }
        Action::Forward { .. } => panic!("a direct command is never forwarded"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
    }
}

/// The require-root finding of a direct check run in `nested/` of `repo`,
///  as event number `sequence`.
fn not_at_root(sequence: u64, repo: &Path) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"direct-check\",\"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\",\"message\":\"cli-git: not at the root of the git repository. Repo root is {root} but effective cwd is {root}/nested. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to {root} or pass -C {root} before the subcommand.\",\"fix\":\"none\"}}\n",
        root = repo.display()
    );
}

/// Help,
///  usage refusals and scope refusals answer without resolving Git.
#[test]
fn help_and_refusals_need_no_git() {
    let root: PathBuf = fixture("management-no-git");
    let inputs: ResolutionInputs = no_git(root.as_path());
    assert_eq!(
        plan(&["--help"], root.as_path(), &inputs),
        exit(0, MANAGEMENT_HELP, "")
    );
    assert_eq!(
        plan(&["-h"], root.as_path(), &inputs),
        exit(0, MANAGEMENT_HELP, "")
    );
    for arguments in [
        vec![],
        vec!["unknown"],
        vec!["check", "--bogus"],
        vec!["--help", "x"],
    ] {
        assert_eq!(
            plan(arguments.as_slice(), root.as_path(), &inputs),
            exit(2, "", MANAGEMENT_USAGE),
            "{arguments:?}"
        );
    }
    assert_eq!(
        plan(&["check", "a.txt"], root.as_path(), &inputs),
        exit(2, "", "git cli-git check pathspecs must follow --.\n")
    );
    assert_eq!(
        plan(&["fix", "a.txt"], root.as_path(), &inputs),
        exit(2, "", "git cli-git fix pathspecs must follow --.\n")
    );
    assert_eq!(
        plan(&["check"], root.as_path(), &inputs),
        exit(
            2,
            "",
            "git cli-git check requires exactly one scope: --all or non-empty pathspecs after --.\n"
        )
    );
    assert_eq!(
        plan(&["fix", "--all", "--", "a.txt"], root.as_path(), &inputs),
        exit(
            2,
            "",
            "git cli-git fix requires exactly one scope: --all or non-empty pathspecs after --.\n"
        )
    );
    remove(root.as_path());
}

/// Retired trust commands explain the retirement with exit status 0 and need no Git.
#[test]
fn retired_commands_explain_and_succeed() {
    let root: PathBuf = fixture("management-retired");
    let inputs: ResolutionInputs = no_git(root.as_path());
    for (arguments, name, command) in [
        (vec!["trust"], "trust", RetiredCommand::Trust),
        (vec!["trust", "--yes"], "trust", RetiredCommand::Trust),
        (vec!["untrust"], "untrust", RetiredCommand::Untrust),
        (vec!["status"], "status", RetiredCommand::Status),
    ] {
        let explanation: String = format!(
            "git cli-git {name} is retired. cli-git now reads cli-git.config.jsonc as data and \
             runs no repository-supplied code, so there is no code execution to approve, revoke \
             or report. Nothing was changed; existing trust records are left in place and are no \
             longer read.\n"
        );
        assert_eq!(retired_explanation(command), explanation);
        assert_eq!(
            plan(arguments.as_slice(), root.as_path(), &inputs),
            exit(0, "", explanation.as_str()),
            "{arguments:?}"
        );
    }
    // Help for the retired command goes to standard output.
    assert_eq!(
        plan(&["trust", "--help"], root.as_path(), &inputs),
        exit(0, retired_explanation(RetiredCommand::Trust).as_str(), "")
    );
    remove(root.as_path());
}

/// An unknown selected policy is a `config-invalid` event on standard output before Git is resolved.
#[test]
fn unknown_selected_policy_is_reported_on_stdout() {
    let root: PathBuf = fixture("management-unknown-policy");
    let inputs: ResolutionInputs = no_git(root.as_path());
    for (arguments, id) in [
        (
            vec![
                "check",
                "--all",
                "--policy",
                "final-newline",
                "--policy",
                "nope",
            ],
            "nope",
        ),
        (
            vec!["fix", "--policy=security/forbidden-string", "--", "a"],
            "security/forbidden-string",
        ),
        (vec!["check", "--all", "--policy="], ""),
        (vec!["check", "--all", "--policy", "a\"b"], "a\\\"b"),
    ] {
        assert_eq!(
            plan(arguments.as_slice(), root.as_path(), &inputs),
            exit(
                2,
                format!(
                    "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"Unknown policy ID: {id}. Shipped policies: require-root, linked-worktree-only, branch-worktree-only, add-explicit, final-newline, markdown/autofix, mono/forbidden-root-context, mono/dependent-version-bump, security/forbidden-strings.\",\"hint\":\"Hint: if you just edited cli-git.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}}\n"
                )
                .as_str(),
                ""
            ),
            "{arguments:?}"
        );
    }
    remove(root.as_path());
}

/// A direct command projects its scope before any policy runs,
///  then runs the policies
/// over the selected worktree files;
///  a scope Git refuses is a lifecycle failure.
#[test]
fn direct_commands_project_their_scope_and_run_the_policies() {
    let root: PathBuf = fixture("management-direct");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    std::fs::write(repo.join("a.txt"), b"no final newline").expect("file");
    let index_before: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    let inputs: ResolutionInputs = with_git(root.as_path());
    // Default settings: the built-in content policy reads the selected worktree file.
    assert_eq!(
        plan(&["check", "--all"], repo.as_path(), &inputs),
        exit(0, final_newline_warning(0).as_str(), "")
    );
    assert_eq!(
        plan(&["check", "--", "a.txt"], repo.as_path(), &inputs),
        exit(0, final_newline_warning(0).as_str(), "")
    );
    // A fix corrects the worktree file, reports only the summary, and a second fix finds
    // nothing left to do.
    assert_eq!(
        plan(
            &[
                "fix",
                "--policy",
                "final-newline",
                "--policy",
                "security/forbidden-strings",
                "--",
                "a.txt"
            ],
            repo.as_path(),
            &inputs
        ),
        exit(0, FIXED_A, "")
    );
    assert_eq!(
        std::fs::read(repo.join("a.txt")).expect("fixed"),
        b"no final newline\n"
    );
    assert_eq!(
        plan(&["fix", "--", "a.txt"], repo.as_path(), &inputs),
        exit(0, "", "")
    );
    std::fs::write(repo.join("a.txt"), b"no final newline").expect("file again");
    // A pathspec Git refuses stops the command before any policy, whichever is selected.
    projection_failed(
        plan(
            &["check", "--policy", "require-root", "--", "missing.txt"],
            repo.as_path(),
            &inputs,
        ),
        "direct-check",
        "cli-git could not read the selected worktree files: git add failed: fatal: pathspec",
    );
    projection_failed(
        plan(&["fix", "--", "missing.txt"], repo.as_path(), &inputs),
        "direct-fix",
        "cli-git could not read the selected worktree files: git add failed: fatal: pathspec",
    );
    // Outside a repository there is no index to project into.
    projection_failed(
        plan(&["check", "--all"], root.as_path(), &inputs),
        "direct-check",
        "cli-git could not read the selected worktree files: git rev-parse failed:",
    );
    // Selecting only command policies gives their answer: clean at the top level ...
    for arguments in [
        vec!["check", "--all", "--policy", "require-root"],
        vec!["check", "--policy=require-root", "--", "a.txt"],
        // ... and nothing at all for policies that do not run for a direct command.
        vec!["check", "--all", "--policy", "linked-worktree-only"],
        vec!["fix", "--all", "--policy", "require-root"],
        vec!["fix", "--all", "--policy", "add-explicit"],
    ] {
        assert_eq!(
            plan(arguments.as_slice(), repo.as_path(), &inputs),
            exit(0, "", ""),
            "{arguments:?}"
        );
    }
    // ... and a finding on standard output, with exit status 1, below the top level.
    assert_eq!(
        plan(
            &["check", "--all", "--policy", "require-root"],
            nested.as_path(),
            &inputs
        ),
        exit(1, not_at_root(0, repo.as_path()).as_str(), "")
    );
    // A direct fix has no require-root policy to run, wherever it is typed.
    assert_eq!(
        plan(
            &["fix", "--all", "--policy", "require-root"],
            nested.as_path(),
            &inputs
        ),
        exit(0, "", "")
    );
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before,
        "a direct command never changes the index"
    );
    remove(root.as_path());
}

/// The first error stops a direct check;
///  keep-going and escapes written before `cli-git` apply.
#[test]
fn controls_before_the_namespace_reach_a_direct_command() {
    let root: PathBuf = fixture("management-controls");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    let inputs: ResolutionInputs = with_git(root.as_path());
    // Without a control the require-root error ends the pass before the content policy.
    assert_eq!(
        plan(&["check", "--all"], nested.as_path(), &inputs),
        exit(1, not_at_root(0, repo.as_path()).as_str(), "")
    );
    // Keep-going reports the finding and then reaches the content policy, which reads the
    // selected files: one without a final newline is reported after the finding.
    std::fs::write(repo.join("a.txt"), b"no final newline").expect("file");
    let mut keep_going: Controls = no_controls();
    keep_going.keep_going = true;
    assert_eq!(
        plan_with(&["check", "--all"], nested.as_path(), &keep_going, &inputs),
        exit(
            1,
            format!(
                "{}{}",
                not_at_root(0, repo.as_path()),
                final_newline_warning(1)
            )
            .as_str(),
            ""
        )
    );
    // An escape skips its policy.
    let mut escaped: Controls = no_controls();
    escaped.escaped.push(PolicyId::RequireRoot);
    assert_eq!(
        plan_with(
            &["check", "--all", "--policy", "require-root"],
            nested.as_path(),
            &escaped,
            &inputs
        ),
        exit(0, "", "")
    );
    remove(root.as_path());
}

/// Configuration decides which policies a direct command runs,
///  and `check` alone reports a legacy file.
#[test]
fn direct_commands_use_the_repository_configuration() {
    let root: PathBuf = fixture("management-configured");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let nested: PathBuf = repo.join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    let inputs: ResolutionInputs = with_git(root.as_path());
    // Invalid configuration is an event on standard output, with no refusal notice.
    let source: PathBuf = repo.join(CONFIG_FILE_NAME);
    std::fs::write(&source, r#"{ "trust": { "children": true } }"#).expect("invalid config");
    assert_eq!(
        plan(&["check", "--all"], repo.as_path(), &inputs),
        exit(
            2,
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"{}: Configuration key trust is retired: JSONC configuration is data and needs no code-execution approval. Remove the key.\",\"hint\":\"Hint: if you just edited cli-git.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}}\n",
                source.display()
            )
            .as_str(),
            ""
        )
    );
    // With every content policy off, an unfiltered check and fix are complete and clean.
    std::fs::write(&source, r#"{ "policies": { "final-newline": "off" } }"#).expect("valid config");
    assert_eq!(
        plan(&["check", "--all"], repo.as_path(), &inputs),
        exit(0, "", "")
    );
    assert_eq!(
        plan(&["fix", "--all"], repo.as_path(), &inputs),
        exit(0, "", "")
    );
    // A listed optional policy is on and reads the selected files.
    std::fs::write(
        &source,
        "{ \"policies\": { \"final-newline\": \"off\", \"mono/forbidden-root-context\": \"error\" } }\n",
    )
    .expect("valid config");
    assert_eq!(
        plan(&["check", "--all"], repo.as_path(), &inputs),
        exit(0, "", "")
    );
    std::fs::write(repo.join("CONTEXT.md"), b"context\n").expect("context");
    assert_eq!(
        plan(&["check", "--all"], repo.as_path(), &inputs),
        exit(
            1,
            "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"direct-check\",\"policyId\":\"mono/forbidden-root-context\",\"severity\":\"error\",\"code\":\"mono/forbidden-root-context/root-context-forbidden\",\"message\":\"Root CONTEXT.md is forbidden; read source code directly.\",\"hint\":\"Hint: if you meant to keep project notes or terms for later readers, put them in a file under doc/; to drop CONTEXT.md from this commit and keep the file on disk, run `git rm --cached CONTEXT.md`.\",\"path\":\"CONTEXT.md\",\"fix\":\"none\"}\n",
            ""
        )
    );
    // That policy does not run for a direct fix, so the fix is complete.
    assert_eq!(
        plan(&["fix", "--all"], repo.as_path(), &inputs),
        exit(0, "", "")
    );
    // A legacy file beside valid configuration is a warning event on standard output of
    // `check`, and policy events continue its numbering.
    std::fs::write(&source, r#"{ "policies": { "final-newline": "off" } }"#).expect("valid config");
    std::fs::write(repo.join("cli-git.config.ts"), "export default {};").expect("legacy config");
    let legacy_warning: String = format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration {legacy} is ignored: {jsonc} is authoritative for the native cli-git. Remove the legacy file once no TypeScript cli-git reads it.\",\"path\":\"{legacy}\",\"hint\":\"Hint: if you are still keeping the old configuration files, delete them once cli-git.config.jsonc holds the settings; they are no longer read.\"}}\n",
        legacy = repo.join("cli-git.config.ts").display(),
        jsonc = source.display()
    );
    assert_eq!(
        plan(&["check", "--all"], repo.as_path(), &inputs),
        exit(0, legacy_warning.as_str(), "")
    );
    assert_eq!(
        plan(&["check", "--all"], nested.as_path(), &inputs),
        exit(
            1,
            format!("{legacy_warning}{}", not_at_root(1, repo.as_path())).as_str(),
            ""
        )
    );
    assert_eq!(
        plan(&["fix", "--all"], repo.as_path(), &inputs),
        exit(0, "", "")
    );
    remove(root.as_path());
}

/// A direct command refuses beside a live commit transaction and recovers a dead one.
#[test]
fn direct_commands_refuse_beside_registered_transactions() {
    let root: PathBuf = fixture("management-pending");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    crate::transaction_registry::ensure_transaction_root(registry.as_path()).expect("registry");
    let id: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";
    crate::transaction_registry::publish_transaction_directory(
        registry.as_path(),
        id,
        crate::recovery_inspect::tests::owner_bytes(id, "2026-10-06T00:00:00.000Z", true)
            .as_slice(),
    )
    .expect("live transaction");
    // An interrupted worktree copy does not concern a direct command.
    std::fs::create_dir_all(repo.join(".git/cli-git-worktree-copy/v1")).expect("journals");
    std::fs::write(repo.join(".git/cli-git-worktree-copy/v1/id.json"), b"{}").expect("journal");
    for (arguments, command) in [
        (
            vec!["check", "--all", "--policy", "require-root"],
            "cli-git check",
        ),
        (
            vec!["fix", "--all", "--policy", "require-root"],
            "cli-git fix",
        ),
    ] {
        // A live transaction no longer stops a direct command: the fix holds the landing lock instead.
        let Action::Exit { stderr, .. } = plan(arguments.as_slice(), repo.as_path(), &inputs) else {
            panic!("{command} is answered by the wrapper, never forwarded");
        };
        assert!(
            !stderr.contains("recovering or waiting for the commit transactions"),
            "{command}: {stderr}"
        );
    }
    // A dead owner's transaction is recovered first, and the command runs.
    std::fs::write(
        registry.join(id).join("owner.json"),
        crate::recovery_inspect::tests::owner_bytes(id, "2026-10-06T00:00:00.000Z", false),
    )
    .expect("dead owner");
    assert_eq!(
        plan(
            &["check", "--all", "--policy", "require-root"],
            repo.as_path(),
            &inputs
        ),
        exit(0, "", "")
    );
    assert!(!registry.join(id).exists());
    // A registry recovery cannot read stops the command with the failure.
    std::fs::create_dir(registry.join("stray")).expect("stray entry");
    match plan(
        &["check", "--all", "--policy", "require-root"],
        repo.as_path(),
        &inputs,
    ) {
        Action::Exit { code, stderr, .. } => {
            assert_eq!(code, 2);
            assert!(
                stderr.contains("Unexpected transaction registry entry"),
                "{stderr}"
            );
        }
        Action::Forward { .. } => panic!("a direct command never forwards"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
    }
    remove(root.as_path());
}

/// A direct command without a usable real Git,
///  or with uninterpretable Git output,
///  stops in prose.
#[test]
fn direct_commands_report_git_failures() {
    let root: PathBuf = fixture("management-git-failure");
    assert_eq!(
        plan(&["check", "--all"], root.as_path(), &no_git(root.as_path())),
        exit(
            2,
            "",
            "cli-git: Could not find a real Git executable after examining 1 PATH candidates and \
             skipping 0 cli-git wrappers. Ensure Git is installed and PATH/PATHEXT expose its \
             executable.\n"
        )
    );
    let mut fake: ResolutionInputs = with_git(root.as_path());
    std::fs::create_dir(root.join("fake")).expect("directory");
    executable(
        root.join("fake/git").as_path(),
        b"#!/bin/sh\nprintf 'false\\n/only-two-lines\\n'\nexit 0\n",
    );
    fake.path = OsString::from("fake");
    match plan(&["check", "--all"], root.as_path(), &fake) {
        Action::Exit {
            code,
            stdout,
            stderr,
        } => {
            assert_eq!(code, 2);
            assert_eq!(stdout, "");
            assert!(
                stderr.starts_with("cli-git: cli-git could not classify the Git worktree"),
                "{stderr}"
            );
        }
        Action::Forward { .. } => panic!("a direct command is never forwarded"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
    }
    remove(root.as_path());
}

/// Unpack an exit action into its status, standard output and standard error.
fn exit_parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else {
        panic!("a management command ends in an exit action");
    };
    return (code, stdout, stderr);
}

/// `doctor` lists the traditional hooks with status 1, prints nothing with status 0 when there are none, and does
/// not load the configuration, so a broken `cli-git.config.jsonc` does not stop it.
#[test]
fn doctor_lists_traditional_hooks_without_loading_the_configuration() {
    let root: PathBuf = fixture("management-doctor");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(CONFIG_FILE_NAME), b"{ not valid").expect("a broken configuration");
    let inputs: ResolutionInputs = with_git(root.as_path());
    assert_eq!(plan(&["doctor"], repo.as_path(), &inputs), exit(0, "", ""));
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(plan(&["doctor"], repo.as_path(), &inputs));
    assert_eq!(code, 1);
    assert_eq!(stdout, format!("{}\n", repo.join(".git/hooks/pre-commit").display()));
    assert!(stderr.contains("unsupported"), "{stderr}");
    remove(root.as_path());
}

/// `doctor` outside a repository reports the failure with status 2.
#[test]
fn doctor_outside_a_repository_fails_with_status_two() {
    let root: PathBuf = fixture("management-doctor-outside");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let (code, stdout, stderr) = exit_parts(plan(&["doctor"], root.as_path(), &inputs));
    assert_eq!((code, stdout.as_str()), (2, ""));
    assert!(stderr.starts_with("git cli-git doctor: "), "{stderr}");
    remove(root.as_path());
}

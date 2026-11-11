//! What:
//!  Decision controls for `git monochromatic-git-policy-cli` against real Git.
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
use super::{Selection, plan_management, plan_management_with_section, retired_explanation, status_git_dir};
use crate::service_instance::{SOCKET_DIRECTORY, socket_for};
use crate::action::Action;
use crate::control_terminal::Terminal;
use crate::config_file::CONFIG_FILE_NAME;
use crate::management_arguments::{MANAGEMENT_HELP, MANAGEMENT_USAGE, RetiredCommand};
use crate::policy_registry::PolicyId;
use crate::real_git::{Platform, ResolutionInputs};
use crate::test_support::{REAL_GIT, executable, fixture, git, isolated_overlay, remove, repository};
use crate::wrapper_controls::{Controls, no_controls};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Direct runs that select a policy which is off,
/// warns or blocks.
#[path = "management_off_policy_tests.rs"]
mod off_policy_cases;

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
        service_unit: None,
        // Management commands are answered by the wrapper and start no Git child, so the terminal decides nothing.
        terminal: Terminal::Present,
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

/// Plan management text arguments without any wrapper control before `monochromatic-git-policy-cli`.
fn plan(values: &[&str], directory: &Path, inputs: &ResolutionInputs) -> Action {
    return plan_with(values, directory, &no_controls(), inputs);
}

/// The standard error without its first line,
/// which must be the fsmonitor line.
///  These controls run the doctor in the test process's own environment,
/// so the host's global configuration decides
///  whether the line says off or on;
/// the line's wording is controlled in `doctor_fsmonitor_tests.rs` with an injected
///  global file.
fn without_fsmonitor_line(stderr: &str) -> String {
    let prefix: &str = "git monochromatic-git-policy-cli doctor: Git's fsmonitor is ";
    assert!(stderr.starts_with(prefix), "{stderr}");
    let line_end: usize = stderr.find('\n').expect("the fsmonitor line ends with a line feed");
    return String::from(&stderr[line_end + 1..]);
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
        Action::ExitBytes { .. } => panic!("a report of pathnames is not expected in this control"),
    }
}

/// The require-root finding of a direct check run in `nested/` of `repo`,
///  as event number `sequence`.
fn not_at_root(sequence: u64, repo: &Path) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"direct-check\",\"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\",\"message\":\"monochromatic-git-policy-cli: not at the root of the git repository. Repo root is {root} but effective cwd is {root}/nested. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to {root} or pass -C {root} before the subcommand.\",\"fix\":\"none\"}}\n",
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
        exit(2, "", "git monochromatic-git-policy-cli check pathspecs must follow --.\n")
    );
    assert_eq!(
        plan(&["fix", "a.txt"], root.as_path(), &inputs),
        exit(2, "", "git monochromatic-git-policy-cli fix pathspecs must follow --.\n")
    );
    assert_eq!(
        plan(&["check"], root.as_path(), &inputs),
        exit(
            2,
            "",
            "git monochromatic-git-policy-cli check requires exactly one scope: --all or non-empty pathspecs after --.\n"
        )
    );
    assert_eq!(
        plan(&["fix", "--all", "--", "a.txt"], root.as_path(), &inputs),
        exit(
            2,
            "",
            "git monochromatic-git-policy-cli fix requires exactly one scope: --all or non-empty pathspecs after --.\n"
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
            "git monochromatic-git-policy-cli {name} is retired. monochromatic-git-policy-cli now reads monochromatic-git-policy-cli.config.jsonc as data and \
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
                    "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"Unknown policy ID: {id}. Shipped policies: require-root, linked-worktree-only, branch-worktree-only, add-explicit, final-newline, markdown/autofix, mono/forbidden-root-context, mono/dependent-version-bump, mono/catalog-specifiers, security/forbidden-strings.\",\"hint\":\"Hint: if you just edited monochromatic-git-policy-cli.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}}\n"
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
        "monochromatic-git-policy-cli could not read the selected worktree files: git add failed: fatal: pathspec",
    );
    projection_failed(
        plan(&["fix", "--", "missing.txt"], repo.as_path(), &inputs),
        "direct-fix",
        "monochromatic-git-policy-cli could not read the selected worktree files: git add failed: fatal: pathspec",
    );
    // Outside a repository there is no index to project into.
    projection_failed(
        plan(&["check", "--all"], root.as_path(), &inputs),
        "direct-check",
        "monochromatic-git-policy-cli could not read the selected worktree files: git rev-parse failed:",
    );
    // Selecting only command policies gives their answer: clean at the top level ...
    for arguments in [
        vec!["check", "--all", "--policy", "require-root"],
        vec!["check", "--policy=require-root", "--", "a.txt"],
    ] {
        assert_eq!(
            plan(arguments.as_slice(), repo.as_path(), &inputs),
            exit(0, "", ""),
            "{arguments:?}"
        );
    }
    // ... and a refusal, never a clean exit, for policies that do not run for a direct command.
    for arguments in [
        vec!["check", "--all", "--policy", "linked-worktree-only"],
        vec!["fix", "--all", "--policy", "require-root"],
        vec!["fix", "--all", "--policy", "add-explicit"],
    ] {
        let Action::Exit { code, stdout, stderr } = plan(arguments.as_slice(), repo.as_path(), &inputs) else {
            panic!("a direct command ends in an exit action");
        };
        assert_eq!((code, stderr.as_str()), (2, ""), "{arguments:?}");
        assert!(stdout.contains("\"code\":\"config-invalid\""), "{stdout}");
        assert!(stdout.contains("would check nothing and exit clean"), "{stdout}");
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
    // A direct fix has no require-root policy to run, wherever it is typed, and says so.
    let Action::Exit { code: fix_code, .. } = plan(&["fix", "--all", "--policy", "require-root"], nested.as_path(), &inputs) else {
        panic!("a direct command ends in an exit action");
    };
    assert_eq!(fix_code, 2);
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before,
        "a direct command never changes the index"
    );
    remove(root.as_path());
}

/// The first error stops a direct check;
///  keep-going and escapes written before `monochromatic-git-policy-cli` apply.
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
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"{}: Configuration key trust is retired: JSONC configuration is data and needs no code-execution approval. Remove the key.\",\"hint\":\"Hint: if you just edited monochromatic-git-policy-cli.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}}\n",
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
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration {legacy} is ignored: {jsonc} is authoritative for the native monochromatic-git-policy-cli. Remove the legacy file once no TypeScript monochromatic-git-policy-cli reads it.\",\"path\":\"{legacy}\",\"hint\":\"Hint: if you are still keeping the old configuration files, delete them once monochromatic-git-policy-cli.config.jsonc holds the settings; they are no longer read.\"}}\n",
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

/// A direct command goes on beside the records of an earlier release,
/// and a commit journal of the previous executable's format
///  stops it.
#[test]
fn direct_commands_go_on_beside_records_an_earlier_release_left() {
    let root: PathBuf = fixture("management-pending");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let registry: PathBuf = repo.join(".git/cli-git-transactions");
    let id: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";
    std::fs::create_dir_all(registry.join(id)).expect("a transaction directory an earlier release left");
    // An interrupted worktree copy does not concern a direct command.
    std::fs::create_dir_all(repo.join(".git/monochromatic-git-policy-cli-worktree-copy/v1")).expect("journals");
    std::fs::write(repo.join(".git/monochromatic-git-policy-cli-worktree-copy/v1/id.json"), b"{}").expect("journal");
    for (arguments, command) in [
        (
            vec!["check", "--all", "--policy", "require-root"],
            "monochromatic-git-policy-cli check",
        ),
        (
            vec!["fix", "--all", "--policy", "require-root"],
            "monochromatic-git-policy-cli fix",
        ),
    ] {
        // A transaction an earlier release left does not stop a direct command.
        let Action::Exit { stderr, .. } = plan(arguments.as_slice(), repo.as_path(), &inputs) else {
            panic!("{command} is answered by the wrapper, never forwarded");
        };
        assert!(
            !stderr.contains("recovering or waiting for the commit transactions"),
            "{command}: {stderr}"
        );
    }
    assert!(registry.join(id).exists(), "nothing was removed");
    // A commit journal of the previous executable's format stops the command with the way to recover it.
    std::fs::create_dir(repo.join(".git/cli-git-transaction")).expect("legacy journal directory");
    match plan(
        &["check", "--all", "--policy", "require-root"],
        repo.as_path(),
        &inputs,
    ) {
        Action::Exit { code, stderr, .. } => {
            assert_eq!(code, 2);
            assert!(
                stderr.contains("holds a commit journal in a format this executable does not recover"),
                "{stderr}"
            );
        }
        Action::Forward { .. } => panic!("a direct command never forwards"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
        Action::ExitBytes { .. } => panic!("a report of pathnames is not expected in this control"),
    }
    remove(root.as_path());
}

/// A direct command without a usable real Git,
///  or with uninterpretable Git output,
///  stops in prose.
#[test]
fn direct_commands_report_git_failures() {
    let root: PathBuf = fixture("management-git-failure");
    // The evidence is long (every candidate and every remedy); the head names the counts and the PATH it read.
    let stopped: Action = plan(&["check", "--all"], root.as_path(), &no_git(root.as_path()));
    let Action::Exit { code: stop_code, stdout: stop_out, stderr: stop_err } = stopped else { panic!("expected a stop") };
    assert_eq!(stop_code, 2);
    assert_eq!(stop_out, "");
    assert!(
        stop_err.starts_with(
            "monochromatic-git-policy-cli: Could not find a real Git executable after examining 1 PATH candidates and \
             skipping 0 monochromatic-git-policy-cli wrappers, so no Git command was run. PATH is "
        ),
        "{stop_err}"
    );
    assert!(stop_err.contains("install Git"), "{stop_err}");
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
                stderr.starts_with("monochromatic-git-policy-cli: monochromatic-git-policy-cli could not classify the Git worktree"),
                "{stderr}"
            );
        }
        Action::Forward { .. } => panic!("a direct command is never forwarded"),
        Action::ForwardCoordinated { .. } => panic!("a coordinated forward is not expected in this control"),
        Action::ForwardSettled { .. } => panic!("a settled forward is not expected in this control"),
        Action::ForwardPush { .. } => panic!("a stacked push is not expected in this control"),
        Action::ForwardAnnounced { .. } => panic!("an announced query is not expected in this control"),
        Action::ExitBytes { .. } => panic!("a report of pathnames is not expected in this control"),
    }
    remove(root.as_path());
}

/// Unpack an exit action into its status,
/// standard output and standard error.
fn exit_parts(action: Action) -> (i32, String, String) {
    return match action {
        Action::Exit { code, stdout, stderr } => (code, stdout, stderr),
        Action::ExitBytes { code, stdout, stderr } => (
            code,
            String::from_utf8(stdout).expect("a report of text paths"),
            String::from_utf8(stderr).expect("an explanation of text paths"),
        ),
        other => panic!("a management command ends in an exit action, not {other:?}"),
    };
}

/// `doctor` lists the traditional hooks with status 1,
/// prints nothing with status 0 when there are none,
/// and does
/// not load the configuration,
/// so a broken `monochromatic-git-policy-cli.config.jsonc` does not stop it.
#[test]
fn doctor_lists_traditional_hooks_without_loading_the_configuration() {
    let root: PathBuf = fixture("management-doctor");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(CONFIG_FILE_NAME), b"{ not valid").expect("a broken configuration");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let (quiet_code, quiet_stdout, quiet_stderr) = exit_parts(plan(&["doctor"], repo.as_path(), &inputs));
    assert_eq!((quiet_code, quiet_stdout.as_str(), without_fsmonitor_line(quiet_stderr.as_str()).as_str()), (0, "", ""));
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let (code, stdout, stderr) = exit_parts(plan(&["doctor"], repo.as_path(), &inputs));
    assert_eq!(code, 1);
    assert_eq!(stdout, format!("{}\n", repo.join(".git/hooks/pre-commit").display()));
    assert!(stderr.contains("never run through monochromatic-git-policy-cli"), "{stderr}");
    remove(root.as_path());
}

/// `doctor` outside a repository reports the failure with status 2.
#[test]
fn doctor_outside_a_repository_fails_with_status_two() {
    let root: PathBuf = fixture("management-doctor-outside");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let (code, stdout, stderr) = exit_parts(plan(&["doctor"], root.as_path(), &inputs));
    assert_eq!((code, stdout.as_str()), (2, ""));
    assert!(stderr.starts_with("git monochromatic-git-policy-cli doctor: "), "{stderr}");
    remove(root.as_path());
}

/// The doctor appends the supplied filesystem section to standard error and keeps its status and standard output;
/// the
///  supplier is called with the repository's common Git directory,
/// and no other command calls it.
#[test]
fn doctor_appends_the_supplied_filesystem_section_and_no_other_command_asks_for_it() {
    let root: PathBuf = fixture("management-doctor-section");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let global: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    let asked: std::cell::RefCell<Vec<PathBuf>> = std::cell::RefCell::new(Vec::new());
    let supplier = |viewpoint: &crate::doctor::CheckoutViewpoint<'_>| -> String {
        asked.borrow_mut().push(viewpoint.checkout.map(Path::to_path_buf).unwrap_or_default());
        return String::from("section text\n");
    };
    let doctor: Action = plan_management_with_section(&[OsString::from("doctor")], global.as_slice(), &no_controls(), &[], &inputs, &supplier);
    let (code, stdout, stderr) = exit_parts(doctor);
    assert_eq!((code, stdout.as_str(), without_fsmonitor_line(stderr.as_str()).as_str()), (0, "", "section text\n"));
    assert_eq!(*asked.borrow(), vec![repo.clone()]);
    let help: Action = plan_management_with_section(&[OsString::from("--help")], global.as_slice(), &no_controls(), &[], &inputs, &supplier);
    assert!(matches!(help, Action::Exit { code: 0, .. }));
    assert_eq!(asked.borrow().len(), 1, "help does not ask for the section");
    remove(root.as_path());
}

/// The doctor reports,
/// read-only,
/// how many batches the earlier release's commit service left open for this worktree,
/// and names
///  the command that settles them;
/// the exit status and standard output do not change,
/// and a worktree with none gets no note.
#[test]
fn doctor_tells_how_many_batches_the_earlier_release_left_open_and_names_the_command() {
    let root: PathBuf = fixture("management-doctor-earlier-batches");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let state: PathBuf = root.join("state");
    let mut environment: Vec<(OsString, OsString)> = isolated_overlay();
    environment.push((OsString::from("XDG_STATE_HOME"), state.as_os_str().to_os_string()));
    let global: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    let doctor = || -> (i32, String, String) {
        return exit_parts(plan_management_with_section(&[OsString::from("doctor")], global.as_slice(), &no_controls(), environment.as_slice(), &inputs, &|_: &crate::doctor::CheckoutViewpoint<'_>| { return String::new(); }));
    };
    let (quiet_code, _, quiet_stderr) = doctor();
    assert_eq!(quiet_code, 0);
    assert!(!quiet_stderr.contains("settle-earlier-batches"), "{quiet_stderr}");
    let git_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("git directory");
    let journal: PathBuf = state.join("cli-git/service").join(crate::service_instance::instance_of(git_dir.as_path()));
    std::fs::create_dir_all(&journal).expect("the earlier journal");
    let intent = crate::service_journal::Intent {
        seq: 1,
        repository: repo.clone(),
        common_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: "a".repeat(40),
        new: "b".repeat(40),
        ids: vec![String::from("id")],
        commits: vec!["c".repeat(40)],
        paths: vec![b"a.txt".to_vec()],
        derived: false,
        cleanup: None,
        index: git_dir.join("index"),
        git_dir: git_dir.clone(),
    };
    crate::service_journal::write_intent(journal.as_path(), &intent).expect("an open batch");
    let (code, stdout, stderr) = doctor();
    assert_eq!((code, stdout.as_str()), (0, ""));
    assert!(stderr.contains("1 batch") && stderr.contains("git monochromatic-git-policy-cli service settle-earlier-batches"), "{stderr}");
    assert!(crate::service_journal::batch_path(journal.as_path(), 1, crate::service_journal::OPEN_EXTENSION).exists(), "the doctor changed nothing");
    remove(root.as_path());
}

/// The environment of a user whose own file holds `settings`,
/// with the system configuration file left in play:
/// the reader then
/// learns that file's path from the real Git it is told about.
fn user_environment(file: &Path, settings: &str) -> Vec<(OsString, OsString)> {
    std::fs::write(file, settings).expect("user configuration");
    let mut environment: Vec<(OsString, OsString)> = isolated_overlay()
        .into_iter()
        .filter(|(name, _)| { return name != "GIT_CONFIG_GLOBAL" && name != "GIT_CONFIG_NOSYSTEM"; })
        .collect();
    environment.push((OsString::from("GIT_CONFIG_GLOBAL"), file.as_os_str().to_os_string()));
    return environment;
}

/// The global options of `git -C <directory>`.
fn in_directory(directory: &Path) -> Vec<OsString> {
    return vec![OsString::from("-C"), directory.as_os_str().to_os_string()];
}

/// `git monochromatic-git-policy-cli service status` asks about the service of the worktree it runs in,
/// however the user's configuration reads:
/// it needs no valid configuration.
/// A user-level `core.fsmonitor = true` and a key the typed proof refuses must not turn the
/// worktree into "any service",
/// which is what a declined layout meant.
#[test]
fn the_status_names_the_worktree_for_a_user_the_typed_proof_cannot_vouch_for() {
    let root: PathBuf = fixture("management-status-user");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let expected: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("git directory");
    for settings in ["[core]\n\tfsmonitor = true\n", "[diff]\n\tdirstat = changes\n", "[core]\n\tfilemode = maybe\n"] {
        let environment: Vec<(OsString, OsString)> = user_environment(root.join("user.gitconfig").as_path(), settings);
        assert_eq!(status_git_dir(in_directory(repo.as_path()).as_slice(), environment.as_slice(), Some(Path::new(REAL_GIT))), Selection::Found(expected.clone()), "{settings:?}");
    }
    remove(root.as_path());
}

/// What:
///  A status asked of Git names the Git directory by the bytes Git printed.
/// Why:
///   The answer is a pathname,
///  and a name that is not UTF-8 spelled through lossy text named a directory that does not exist:
///  a commit that asked whether a service held the worktree was then refused as
///  "the Git directory cannot be resolved".
#[test]
fn a_status_asked_of_git_keeps_a_git_directory_that_is_not_text() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("management-status-not-utf8");
    let repo: PathBuf = root.join(std::ffi::OsString::from_vec(b"rep\xefo".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let environment: Vec<(OsString, OsString)> = user_environment(root.join("user.gitconfig").as_path(), "[user]\n\tname = Someone\n");
    // `--git-dir` is a selection the in-process reader leaves to Git, so this asks Git itself.
    let selected: Vec<OsString> = vec![OsString::from("--git-dir"), repo.join(".git").into_os_string()];
    assert_eq!(
        status_git_dir(selected.as_slice(), environment.as_slice(), Some(Path::new(REAL_GIT))),
        Selection::Found(std::fs::canonicalize(repo.join(".git")).expect("git directory")),
        "the Git directory keeps the bytes Git printed"
    );
    remove(root.as_path());
}

/// What:
///  A status asked of Git keeps a Git directory whose name ends in a space.
/// Why:
///   The answer is a pathname and Git ends it with one line feed,
///  so every trailing whitespace byte taken from it took the space that belongs to the name:
/// the status then asked for the service of a directory that does not exist,
///  refused the question as "the Git directory ... cannot be resolved",
/// and named a service that was running as one that was not.
///
/// `--git-dir` is a selection the in-process reader leaves to Git,
/// so this asks Git itself.
#[test]
fn a_status_asked_of_git_keeps_a_git_directory_that_ends_in_a_space() {
    let root: PathBuf = fixture("management-status-trailing-space");
    let repo: PathBuf = root.join("repo");
    let administered: PathBuf = root.join("metadata ");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(
        root.as_path(),
        &[
            "init",
            "--quiet",
            "--initial-branch=main",
            "--separate-git-dir",
            administered.to_str().expect("a UTF-8 fixture path"),
            repo.to_str().expect("a UTF-8 fixture path"),
        ],
    );
    let environment: Vec<(OsString, OsString)> = user_environment(root.join("user.gitconfig").as_path(), "[user]\n\tname = Someone\n");
    let selected: Vec<OsString> = vec![OsString::from("--git-dir"), administered.clone().into_os_string()];
    assert_eq!(
        status_git_dir(selected.as_slice(), environment.as_slice(), Some(Path::new(REAL_GIT))),
        Selection::Found(std::fs::canonicalize(administered.as_path()).expect("git directory")),
        "the Git directory keeps the space its name ends in"
    );
    remove(root.as_path());
}

/// The reader learns the system configuration file from the real Git,
/// so a status that names none cannot tell a plain layout
/// from one it must leave to Git:
/// this is why the command resolves the real Git first.
#[test]
fn the_status_needs_the_real_git_to_name_the_worktree() {
    let root: PathBuf = fixture("management-status-nameless");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let environment: Vec<(OsString, OsString)> = user_environment(root.join("user.gitconfig").as_path(), "[user]\n\tname = Someone\n");
    assert!(matches!(status_git_dir(in_directory(repo.as_path()).as_slice(), environment.as_slice(), Some(Path::new(REAL_GIT))), Selection::Found(_)));
    assert!(matches!(status_git_dir(in_directory(repo.as_path()).as_slice(), environment.as_slice(), None), Selection::Unresolved(_)), "no real Git, no answer, and not \"outside a repository\"");
    remove(root.as_path());
}

/// The wording of the status for the worktree whose service answers,
/// and for a worktree whose does not.
const THIS_WORKTREE: &str = "The commit service of this worktree";

/// `git -C /path/B monochromatic-git-policy-cli service status` run from anywhere asks about B:
/// the global options reach the status,
/// and `-c` options do not change the worktree,
/// while an option that could name another repository leaves the status counting any service.
#[test]
fn the_status_follows_the_dash_c_options_of_the_invocation() {
    let root: PathBuf = fixture("management-status-dash-c");
    let first: PathBuf = repository(root.as_path(), "first");
    let second: PathBuf = repository(root.as_path(), "second");
    // A socket path holds about a hundred bytes,
    // which a fixture root could use up,
    // so the runtime directory is short.
    let runtime: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join(format!("temp/agent/ms-{}", std::process::id()));
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("runtime directory");
    let second_git_dir: PathBuf = std::fs::canonicalize(second.join(".git")).expect("git directory");
    let listener = std::os::unix::net::UnixListener::bind(socket_for(runtime.as_path(), second_git_dir.as_path())).expect("the second worktree's service");
    let mut environment: Vec<(OsString, OsString)> = isolated_overlay();
    environment.push((OsString::from("XDG_RUNTIME_DIR"), runtime.as_os_str().to_os_string()));
    let inputs: ResolutionInputs = with_git(root.as_path());
    let ceiling: (OsString, OsString) = (OsString::from("GIT_CEILING_DIRECTORIES"), root.as_os_str().to_os_string());
    let status_in = |global: &[OsString], extra: &[(OsString, OsString)]| -> (i32, String, String) {
        let mut seen: Vec<(OsString, OsString)> = environment.clone();
        seen.extend(extra.iter().cloned());
        let planned: Action = plan_management_with_section(&[OsString::from("service"), OsString::from("status")], global, &no_controls(), seen.as_slice(), &inputs, &|_: &crate::doctor::CheckoutViewpoint<'_>| { return String::new(); });
        let Action::Exit { code, stdout, stderr } = planned else {
            panic!("the status exits");
        };
        return (code, stdout, stderr);
    };
    let status = |global: &[OsString]| -> (i32, String, String) { return status_in(global, &[]); };
    let (second_code, second_out, _) = status(in_directory(second.as_path()).as_slice());
    assert_eq!((second_code, second_out.starts_with(THIS_WORKTREE)), (0, true), "-C names the worktree that has a service: {second_out}");
    let (first_code, _, first_err) = status(in_directory(first.as_path()).as_slice());
    assert_eq!((first_code, first_err.starts_with(THIS_WORKTREE)), (1, true), "-C names the worktree that has none: {first_err}");
    let mut with_config: Vec<OsString> = vec![OsString::from("-c"), OsString::from("user.name=Someone")];
    with_config.extend(in_directory(second.as_path()));
    let (config_code, config_out, _) = status(with_config.as_slice());
    assert_eq!((config_code, config_out.starts_with(THIS_WORKTREE)), (0, true), "-c does not change the worktree: {config_out}");
    // `--git-dir` is not resolved by the reader,
    // so Git is asked:
    // it names the repository the option selects,
    // not any service.
    let git_dir_option: Vec<OsString> = vec![OsString::from("--git-dir"), second.join(".git").into_os_string()];
    let (named_code, named_out, _) = status(git_dir_option.as_slice());
    assert_eq!((named_code, named_out.starts_with(THIS_WORKTREE)), (0, true), "--git-dir names the worktree that has a service: {named_out}");
    let mut elsewhere: Vec<OsString> = in_directory(first.as_path());
    elsewhere.extend([OsString::from("--git-dir"), OsString::from(".git")]);
    let (none_code, _, none_err) = status(elsewhere.as_slice());
    assert_eq!((none_code, none_err.starts_with(THIS_WORKTREE)), (1, true), "-C and --git-dir select the repository without a service, and another repository's service is not counted: {none_err}");
    // Outside a repository the answer counts any service.
    // The ceiling keeps Git from finding the repository this test is in.
    let outside: PathBuf = root.join("outside");
    std::fs::create_dir(&outside).expect("a directory outside any repository");
    let (any_code, any_out, _) = status_in(in_directory(outside.as_path()).as_slice(), std::slice::from_ref(&ceiling));
    assert_eq!((any_code, any_out.starts_with("A commit service")), (0, true), "outside a repository any service counts: {any_out}");
    // With no real Git to ask,
    // an invocation the reader does not resolve is unresolved,
    // never counted as any service.
    let nameless: ResolutionInputs = no_git(root.as_path());
    let unresolved = plan_management_with_section(&[OsString::from("service"), OsString::from("status")], git_dir_option.as_slice(), &no_controls(), environment.as_slice(), &nameless, &|_: &crate::doctor::CheckoutViewpoint<'_>| { return String::new(); });
    let Action::Exit { code: unresolved_code, stdout: unresolved_out, stderr: unresolved_err } = unresolved else {
        panic!("the status exits");
    };
    assert_eq!((unresolved_code, unresolved_out.is_empty(), unresolved_err.contains("no service was counted")), (1, true, true), "{unresolved_err}");
    drop(listener);
    std::fs::remove_dir_all(runtime).expect("runtime directory");
    remove(root.as_path());
}

/// An invocation the reader does not resolve and Git cannot answer for another reason than "not a git repository" (a
/// malformed value Git dies on) is unresolved:
/// it is not outside a repository,
/// so no service is counted for it.
#[test]
fn a_selection_git_fails_on_is_unresolved_and_not_outside_a_repository() {
    let root: PathBuf = fixture("management-status-git-fails");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let environment: Vec<(OsString, OsString)> = user_environment(root.join("user.gitconfig").as_path(), "[core]\n\tfilemode = maybe\n");
    let selected: Vec<OsString> = vec![OsString::from("--git-dir"), repo.join(".git").into_os_string()];
    let found: Selection = status_git_dir(selected.as_slice(), environment.as_slice(), Some(Path::new(REAL_GIT)));
    assert!(matches!(&found, Selection::Unresolved(reason) if reason.contains("filemode")), "{found:?}");
    remove(root.as_path());
}

/// The doctor's host section ends with the line about the repository's link to the 860 drive when an identity file records one,
/// and adds nothing when none does (plan item W15).
#[cfg(target_os = "linux")]
#[test]
fn the_doctors_host_section_carries_the_link_line_only_with_an_identity_file() {
    use super::filesystem_section_with_link_line;
    let root: PathBuf = fixture("doctor-host-section-link").canonicalize().expect("root");
    let home: PathBuf = root.join("home");
    let not_a_mount: PathBuf = root.join("not-a-mount");
    std::fs::create_dir_all(&not_a_mount).expect("mount point stand-in");
    let link: PathBuf = root.join("repolink");
    std::os::unix::fs::symlink(not_a_mount.join("r/M"), &link).expect("dangling link");
    let nowhere: crate::doctor::CheckoutViewpoint<'_> = crate::doctor::CheckoutViewpoint {
        checkout: None,
        layout: crate::checkout_layout::CheckoutLayout::Main,
        home: Some(home.as_path()),
    };
    assert_eq!(filesystem_section_with_link_line(&nowhere), "", "no checkout and no identity file, no line");
    let identity: PathBuf = home.join(".config/monochromatic-dev/repository-link.json");
    std::fs::create_dir_all(identity.parent().expect("parent")).expect("config folder");
    std::fs::write(
        &identity,
        format!(
            "{{\"linkPath\":\"{}\",\"linkText\":\"{}\",\"canonicalRoot\":\"{}\",\"mountPoint\":\"{}\",\"driveUuid\":\"D\",\"nvmeUuid\":\"N\",\"rootCommit\":\"abc\",\"phase\":\"after\"}}",
            link.display(),
            not_a_mount.join("r/M").display(),
            not_a_mount.join("r/M").display(),
            not_a_mount.display()
        ),
    )
    .expect("identity file");
    let section: String = filesystem_section_with_link_line(&nowhere);
    assert!(section.starts_with("repository link: the 860 drive is not mounted yet ("), "{section}");
    assert!(section.ends_with("run this again\n"), "{section}");
    remove(root.as_path());
}

/// The doctor's link line reads the home the invocation's environment names,
/// not this process's:
/// an identity file below that home records a link that dangles,
/// and the doctor says so.
#[cfg(target_os = "linux")]
#[test]
fn the_doctors_link_line_reads_the_home_the_invocation_names() {
    use super::host_filesystem_section;
    let root: PathBuf = fixture("doctor-invocation-home").canonicalize().expect("root");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let inputs: ResolutionInputs = with_git(root.as_path());
    let home: PathBuf = root.join("home");
    let mount_point: PathBuf = root.join("not-a-mount");
    let link: PathBuf = root.join("repolink");
    let target: PathBuf = mount_point.join("repository/Monochromatic");
    std::fs::create_dir_all(mount_point.as_path()).expect("mount point stand-in");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("dangling link");
    let identity: PathBuf = home.join(".config/monochromatic-dev/repository-link.json");
    std::fs::create_dir_all(identity.parent().expect("parent")).expect("config folder");
    std::fs::write(
        identity,
        format!(
            "{{\"linkPath\":\"{}\",\"linkText\":\"{}\",\"canonicalRoot\":\"{}\",\"mountPoint\":\"{}\",\"driveUuid\":\"D\",\"nvmeUuid\":\"N\",\"rootCommit\":\"abc\",\"phase\":\"after\"}}",
            link.display(),
            target.display(),
            target.display(),
            mount_point.display()
        ),
    )
    .expect("identity file");
    let mut environment: Vec<(OsString, OsString)> = isolated_overlay();
    environment.push((OsString::from("HOME"), home.into_os_string()));
    let global: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    let action: Action = plan_management_with_section(
        &[OsString::from("doctor")],
        global.as_slice(),
        &no_controls(),
        environment.as_slice(),
        &inputs,
        &host_filesystem_section,
    );
    let (code, _stdout, stderr) = exit_parts(action);
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.contains("repository link: the 860 drive is not mounted yet ("), "{stderr}");
    remove(root.as_path());
}

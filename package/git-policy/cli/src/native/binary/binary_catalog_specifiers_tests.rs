//! What:
//!  `mono/catalog-specifiers` through the built executable:
//! a real `git commit` that a listed `error` policy blocks and
//!  a `warn` policy reports,
//! the same commit landing untouched under an older configuration,
//! and the direct
//!  `git monochromatic-git-policy-cli check --policy` runs for an off,
//! warning and blocking policy.
//! Why:
//!  The policy ships switched off,
//! so what a repository sees is decided by its configuration file.
//!  These are the commands a person or a CI step runs;
//! the controls read exit status,
//! events and the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['commit', '--message=x', '--', 'package.json']).code).toBe(1);
//! ```

/// Import the shared fixtures and the commit controls' helpers.
use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit};
use super::support::{Fixture, Observed, fixture, repository, run_wrapped, stderr_of, without_wait_notices};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A manifest with a literal version.
const LITERAL: &str = "{\n  \"dependencies\": {\n    \"zod\": \"^4.6.7\"\n  }\n}\n";

/// The same manifest written as the rule wants it.
const CATALOG: &str = "{\n  \"dependencies\": {\n    \"zod\": \"catalog:\"\n  }\n}\n";

/// The configuration that turns the policy on at `error`.
const ERROR_CONFIGURATION: &str = "{ \"policies\": { \"mono/catalog-specifiers\": \"error\" } }\n";

/// The pre-forward finding of `LITERAL` at `severity`,
/// as the event line with its newline.
fn literal_finding(trigger: &str, severity: &str) -> String {
    return format!(
        "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"{trigger}\",\
         \"policyId\":\"mono/catalog-specifiers\",\"severity\":\"{severity}\",\
         \"code\":\"mono/catalog-specifiers/disallowed-specifier\",\
         \"message\":\"package.json: dependencies `zod` is \\\"^4.6.7\\\"; move the version into the `catalog:` section \
         of pnpm-workspace.yaml and write `catalog:`, or write `workspace:*` for a package of this workspace\",\
         \"path\":\"package.json\",\"fix\":\"none\"}}\n"
    );
}

/// A repository with one commit and the configuration text,
/// when there is one,
/// and a manifest in the worktree.
fn configured(fixture: &Fixture, configuration: Option<&str>, manifest: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    if let Some(text) = configuration {
        std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), text).expect("repository configuration");
    }
    std::fs::write(repo.join("package.json"), manifest).expect("manifest");
    return repo;
}

/// The wrapped commit of the manifest alone.
fn commit_manifest(fixture: &Fixture, repo: &Path) -> Observed {
    return run_commit(fixture, repo, &["commit", "--message=manifest", "--", "package.json"]);
}

/// A listed `error` policy blocks a real commit that holds a literal version and nothing lands;
/// the same commit with
///  the specifier written as `catalog:` lands and prints nothing.
#[test]
fn an_error_policy_blocks_a_literal_version_and_lets_a_catalog_one_land() {
    let fixture: Fixture = fixture("catalog-commit-error");
    let repo: PathBuf = configured(&fixture, Some(ERROR_CONFIGURATION), LITERAL);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let blocked: Observed = commit_manifest(&fixture, repo.as_path());
    assert_eq!((blocked.code, stderr_of(&blocked)), (Some(1), literal_finding("pre-forward", "error")));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_nothing_left(&fixture, repo.as_path());
    std::fs::write(repo.join("package.json"), CATALOG).expect("fixed manifest");
    let landed: Observed = commit_manifest(&fixture, repo.as_path());
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    // A loaded host stretches the service's branch write past a beat, and the client then reads the service's
    //  notice about the write it is inside; this control means the policy's words, so it drops that notice.
    assert_eq!(without_wait_notices(stderr_of(&landed).as_bytes()), b"", "a commit the policy lets through prints nothing of its own: {}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:package.json"]), CATALOG);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A tracked `package.yaml` blocks a commit at `error` too,
/// and its bytes are never read.
#[test]
fn an_error_policy_blocks_an_unsupported_manifest_name() {
    let fixture: Fixture = fixture("catalog-commit-yaml");
    let repo: PathBuf = configured(&fixture, Some(ERROR_CONFIGURATION), CATALOG);
    std::fs::write(repo.join("package.yaml"), b"\xff\xfe not yaml, never read").expect("yaml");
    let blocked: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=yaml", "--", "package.yaml"]);
    let stderr: String = stderr_of(&blocked);
    assert_eq!(blocked.code, Some(1), "{stderr}");
    assert!(stderr.contains("\"code\":\"mono/catalog-specifiers/unsupported-manifest-format\""), "{stderr}");
    assert!(stderr.contains("\"path\":\"package.yaml\""), "{stderr}");
    assert_eq!(stderr.matches('\n').count(), 1, "{stderr}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// At `warn` the commit lands with the finding and the unsafe-warn notice on standard error.
#[test]
fn a_warn_policy_reports_and_lets_the_commit_land() {
    let fixture: Fixture = fixture("catalog-commit-warn");
    let repo: PathBuf = configured(&fixture, Some("{ \"policies\": { \"mono/catalog-specifiers\": \"warn\" } }\n"), LITERAL);
    let landed: Observed = commit_manifest(&fixture, repo.as_path());
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(stderr.starts_with(literal_finding("pre-forward", "warn").as_str()), "{stderr}");
    assert!(stderr.contains("\"code\":\"warn-unsafe\""), "{stderr}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:package.json"]), LITERAL);
    remove_settled(&fixture);
}

/// A repository with no configuration,
/// an empty one,
/// or one that lists only the older policies commits the same
///  manifest as it did before the policy shipped:
/// exit 0,
/// nothing printed,
/// the manifest in the commit.
#[test]
fn a_configuration_that_does_not_name_the_policy_commits_as_before() {
    for (name, configuration) in [
        ("none", None),
        ("empty", Some("{}\n")),
        ("older", Some("{ \"policies\": { \"mono/forbidden-root-context\": \"error\", \"mono/dependent-version-bump\": \"error\" } }\n")),
        ("off", Some("{ \"policies\": { \"mono/catalog-specifiers\": \"off\" } }\n")),
    ] {
        let fixture: Fixture = fixture(format!("catalog-commit-before-{name}").as_str());
        let repo: PathBuf = configured(&fixture, configuration, LITERAL);
        let landed: Observed = commit_manifest(&fixture, repo.as_path());
        assert_eq!((landed.code, stderr_of(&landed)), (Some(0), String::new()), "{name}");
        assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:package.json"]), LITERAL, "{name}");
        remove_settled(&fixture);
    }
}

/// `git monochromatic-git-policy-cli check --all --policy mono/catalog-specifiers` exits 2 and names the key while the policy is off,
///  exits 0 with the finding at `warn`,
/// and exits 1 at `error`.
#[test]
fn a_direct_check_of_the_policy_follows_its_severity() {
    let fixture: Fixture = fixture("catalog-direct");
    let repo: PathBuf = configured(&fixture, None, LITERAL);
    let arguments: [&str; 5] = ["monochromatic-git-policy-cli", "check", "--all", "--policy", "mono/catalog-specifiers"];
    let off: Observed = run_wrapped(&fixture, repo.as_path(), &arguments);
    let stdout: String = String::from_utf8_lossy(off.stdout.as_slice()).into_owned();
    assert_eq!((off.code, stderr_of(&off)), (Some(2), String::new()));
    assert!(stdout.contains("\"code\":\"config-invalid\""), "{stdout}");
    assert!(stdout.contains("Policy mono/catalog-specifiers is off, so --policy mono/catalog-specifiers would check nothing"), "{stdout}");
    assert!(stdout.contains("\\\"policies\\\": { \\\"mono/catalog-specifiers\\\": \\\"error\\\" } in monochromatic-git-policy-cli.config.jsonc"), "{stdout}");
    for (severity, code) in [("warn", 0), ("error", 1)] {
        std::fs::write(
            repo.join("monochromatic-git-policy-cli.config.jsonc"),
            format!("{{ \"policies\": {{ \"mono/catalog-specifiers\": \"{severity}\" }} }}\n"),
        )
        .expect("configuration");
        let ran: Observed = run_wrapped(&fixture, repo.as_path(), &arguments);
        let printed: String = String::from_utf8_lossy(ran.stdout.as_slice()).into_owned();
        assert_eq!((ran.code, stderr_of(&ran)), (Some(code), String::new()), "{severity}");
        assert!(printed.starts_with(literal_finding("direct-check", severity).as_str()), "{severity}: {printed}");
    }
    // The policy is on at `error`, but a direct fix does not run it: that is a refusal, not a clean exit.
    let fix: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "fix", "--all", "--policy", "mono/catalog-specifiers"],
    );
    let refusal: String = String::from_utf8_lossy(fix.stdout.as_slice()).into_owned();
    assert_eq!((fix.code, stderr_of(&fix)), (Some(2), String::new()), "{refusal}");
    assert!(refusal.contains("does not run for monochromatic-git-policy-cli fix"), "{refusal}");
    remove_settled(&fixture);
}

/// This repository's own settings file (found by the wrapper's lookup,
/// current name then earlier),
/// read by the built executable:
/// it loads,
/// the older optional
///  policies it lists run (a clean check of one exits 0),
/// and selecting the catalog policy does what the file says about it:
/// listed,
/// it runs and refuses the literal specifier (exit 1);
/// not listed,
/// it exits 2.
/// The expectation is read from the file,
/// so turning the policy on or off in the repository changes no test.
#[test]
fn the_repository_configuration_loads_and_the_catalog_policy_follows_it() {
    let fixture: Fixture = fixture("catalog-real-configuration");
    let repo: PathBuf = configured(&fixture, None, LITERAL);
    // The file is found by the wrapper's own lookup (current name, then the earlier one) and copied under the current name.
    let shipped: PathBuf = git_policy_cli::config_file::settings_file_of(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").as_path());
    std::fs::copy(&shipped, repo.join(git_policy_cli::config_file::CONFIG_FILE_NAME)).expect("the repository's own configuration");
    let shipped_text: String = std::fs::read_to_string(&shipped).expect("the repository's own configuration text");
    // On means listed with a value other than "off"; a line's `//` comment is not configuration.
    let lists_catalog: bool = shipped_text.lines().any(|line| {
        let code: &str = line.split("//").next().unwrap_or_default();
        return code.contains("\"mono/catalog-specifiers\"") && !code.contains("\"off\"");
    });
    let listed: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "check", "--all", "--policy", "mono/forbidden-root-context"],
    );
    assert_eq!((listed.code, stderr_of(&listed)), (Some(0), String::new()));
    assert!(listed.stdout.is_empty(), "{}", String::from_utf8_lossy(listed.stdout.as_slice()));
    let selected: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "check", "--all", "--policy", "mono/catalog-specifiers"],
    );
    let stdout: String = String::from_utf8_lossy(selected.stdout.as_slice()).into_owned();
    if lists_catalog {
        assert_eq!(selected.code, Some(1), "{stdout}");
        assert!(stdout.contains("mono/catalog-specifiers/disallowed-specifier"), "{stdout}");
    } else {
        // Not listed and listed as "off" both refuse with exit 2 and name the policy.
        assert_eq!(selected.code, Some(2), "{stdout}");
        assert!(stdout.contains("mono/catalog-specifiers"), "{stdout}");
    }
    remove_settled(&fixture);
}

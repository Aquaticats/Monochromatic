//! What:
//!  The linked-worktree ignored-state copy through the built executable:
//!  `git worktree add` from the main checkout and from a linked worktree brings the generated forbidden-strings
//!  rules file into the new worktree, and the scanner there finds its rules.
//! Why:
//!  The user decision of 2026-10-12 is that a fresh worktree must get `.cache/forbidden-strings.rules.txt` when the
//!  wrapper creates it; a source without the file copies nothing, so the scanner fails closed instead of scanning
//!  with fewer rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // run(wrappedGit, ['worktree', 'add', ...]); expect(run(wrappedGit, ['cli-git', 'check', '--all']).status).toBe(1);
//! ```

/// Import the shared fixtures and the bounded process helpers.
use super::support::{
    Fixture, Observed, fixture, git, observe, remove, repository, run_wrapped, stderr_of, wrapped,
};
use git_policy_cli::worktree_copy_journal::{create_journal, read_pending_journals};
use git_policy_cli::worktree_copy_selection::CopySelection;
use git_policy_cli::worktree_copy_stage::{StagedSnapshot, new_stage_container, stage_ignored_snapshot};
use git_policy_cli::worktree_copy_stage_claim::{claim_stage, stage_claims_exist};
use git_policy_cli::transaction_git::GitContext;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Where the generated rules file lives, as the configuration's `rulesFile` names it.
const RULES_FILE: &str = ".cache/forbidden-strings.rules.txt";

/// The text the rules file forbids and the tracked file `a.txt` contains.
fn needle() -> String {
    return ["PLANTED", "COPY", "NEEDLE"].join("_");
}

/// A repository whose committed configuration names the rules file, with `a.txt` holding the forbidden text.
/// The rules file and a second ignored directory exist only when `with_rules` is set.
fn repository_with_rules(fixture: &Fixture, with_rules: bool) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        format!(
            "{{ \"policies\": {{ \"security/forbidden-strings\": [\"error\", {{ \"builtinRules\": false, \"rulesFile\": \"{RULES_FILE}\" }}] }} }}\n"
        ),
    )
    .expect("configuration");
    std::fs::write(repo.join(".gitignore"), b".cache/\nheavy/\n").expect("gitignore");
    std::fs::write(repo.join("a.txt"), format!("first\n{}\n", needle())).expect("needle file");
    git(
        fixture,
        repo.as_path(),
        &["add", "--", "cli-git.config.jsonc", ".gitignore", "a.txt"],
    );
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=configuration"],
    );
    if with_rules {
        std::fs::create_dir_all(repo.join(".cache")).expect("cache directory");
        std::fs::write(repo.join(RULES_FILE), format!("{}\n", needle())).expect("rules");
        std::fs::write(repo.join(".cache/other.txt"), b"other\n").expect("other cache file");
        std::fs::create_dir_all(repo.join("heavy")).expect("heavy directory");
        std::fs::write(repo.join("heavy/data"), b"payload\n").expect("heavy data");
    }
    return repo;
}

/// Run the forbidden-strings check over every file of a worktree.
fn check_all(fixture: &Fixture, worktree: &Path) -> Observed {
    return run_wrapped(
        fixture,
        worktree,
        &["cli-git", "check", "--policy", "security/forbidden-strings", "--all"],
    );
}

/// The check found the forbidden text with the copied rules.
fn assert_scanner_has_rules(observed: &Observed) {
    let stdout: String = String::from_utf8_lossy(&observed.stdout).into_owned();
    assert_eq!(observed.code, Some(1), "{stdout} {}", stderr_of(observed));
    assert!(
        stdout.contains("security/forbidden-strings/forbidden-string"),
        "{stdout}"
    );
}

/// The check could not load rules and scanned nothing; the message names the missing path.
fn assert_scanner_failed_closed(observed: &Observed, worktree: &Path) {
    let stdout: String = String::from_utf8_lossy(&observed.stdout).into_owned();
    assert_eq!(observed.code, Some(2), "{stdout}");
    assert!(stdout.contains("\"code\":\"policy-incomplete\""), "{stdout}");
    assert!(
        stdout.contains(worktree.join(RULES_FILE).to_string_lossy().as_ref()),
        "{stdout}"
    );
}

/// A worktree added from the main checkout receives the rules file and nothing else the checkout ignores,
/// and the scanner there finds its rules;
///  the same add with the opt-out leaves the scanner without rules.
#[test]
fn worktree_added_from_the_main_checkout_receives_the_rules_file() {
    let fixture: Fixture = fixture("copy-main");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(
        stderr_of(&created),
        format!(
            "cli-git: copied 1 ignored filesystem entry from \"{}\" into 1 new worktree.\n",
            repo.display()
        )
    );
    let one: PathBuf = fixture.root.join("one");
    assert_eq!(
        std::fs::read(one.join(RULES_FILE)).expect("copied rules"),
        format!("{}\n", needle()).into_bytes()
    );
    assert!(!one.join(".cache/other.txt").exists(), "siblings stay behind");
    assert!(!one.join("heavy").exists(), "other ignored trees stay behind");
    assert_scanner_has_rules(&check_all(&fixture, one.as_path()));
    // Control: the opt-out copies nothing, and then the scanner has no rules and fails closed.
    let opted_out: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(opted_out.code, Some(0), "{}", stderr_of(&opted_out));
    assert!(opted_out.stderr.is_empty(), "{}", stderr_of(&opted_out));
    let two: PathBuf = fixture.root.join("two");
    assert!(!two.join(".cache").exists());
    assert_scanner_failed_closed(&check_all(&fixture, two.as_path()), two.as_path());
    remove(&fixture);
}

/// A worktree added from a linked worktree receives every ignored path of that worktree.
#[test]
fn worktree_added_from_a_linked_worktree_receives_all_ignored_state() {
    let fixture: Fixture = fixture("copy-linked");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["worktree", "add", "--quiet", "-b", "one", "../one"],
        )
        .code,
        Some(0)
    );
    let one: PathBuf = fixture.root.join("one");
    std::fs::create_dir_all(one.join("heavy")).expect("heavy in the source");
    std::fs::write(one.join("heavy/data"), b"payload\n").expect("heavy data");
    let created: Observed = run_wrapped(
        &fixture,
        one.as_path(),
        &["worktree", "add", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(
        stderr_of(&created),
        format!(
            "cli-git: copied 4 ignored filesystem entries from \"{}\" into 1 new worktree.\n",
            one.display()
        )
    );
    let two: PathBuf = fixture.root.join("two");
    assert!(two.join(RULES_FILE).exists());
    assert_eq!(
        std::fs::read(two.join("heavy/data")).expect("heavy copied"),
        b"payload\n"
    );
    assert_scanner_has_rules(&check_all(&fixture, two.as_path()));
    remove(&fixture);
}

/// A main checkout without a rules file copies nothing, and the scanner in the new worktree fails closed.
#[test]
fn a_source_without_the_rules_file_copies_nothing_and_the_scanner_fails_closed() {
    let fixture: Fixture = fixture("copy-absent");
    let repo: PathBuf = repository_with_rules(&fixture, false);
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let one: PathBuf = fixture.root.join("one");
    assert!(!one.join(".cache").exists());
    assert_scanner_failed_closed(&check_all(&fixture, one.as_path()), one.as_path());
    remove(&fixture);
}

/// A main checkout whose configuration names no rules file stays a bypass: nothing is copied and nothing is printed.
#[test]
fn a_main_checkout_without_a_configured_rules_file_is_forwarded_plainly() {
    let fixture: Fixture = fixture("copy-unconfigured");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b".cache/\n").expect("gitignore");
    std::fs::create_dir_all(repo.join(".cache")).expect("cache");
    std::fs::write(repo.join(RULES_FILE), b"rules\n").expect("rules");
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert!(created.stderr.is_empty(), "{}", stderr_of(&created));
    assert!(!fixture.root.join("one/.cache").exists());
    remove(&fixture);
}

/// A command Git expands from an alias into `worktree add` is copied too, and a failed Git keeps its own status.
#[test]
fn an_aliased_worktree_add_is_copied_and_git_failures_keep_their_status() {
    let fixture: Fixture = fixture("copy-alias");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let aliased: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["-c", "alias.wta=worktree add", "wta", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(aliased.code, Some(0), "{}", stderr_of(&aliased));
    assert!(fixture.root.join("one").join(RULES_FILE).exists());
    // Git refuses a branch that exists: its status is kept and nothing is copied or summarized.
    let refused: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../again"],
    );
    assert_eq!(refused.code, Some(255), "{}", stderr_of(&refused));
    assert!(!stderr_of(&refused).contains("cli-git: copied"));
    assert!(!fixture.root.join("again").exists());
    remove(&fixture);
}

/// A copy interrupted before it finished is recovered by the next command run from a linked worktree:
/// the staged files are installed, the recovery is reported, and the scanner then finds its rules.
#[test]
fn an_interrupted_copy_is_recovered_by_the_next_command() {
    let fixture: Fixture = fixture("copy-recovery");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let opted_out: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(opted_out.code, Some(0));
    let one: PathBuf = std::fs::canonicalize(fixture.root.join("one")).expect("canonical worktree");
    assert!(!one.join(RULES_FILE).exists());
    // The state an interrupted copy leaves: a staged journal for the registered worktree, nothing installed.
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let context: GitContext = GitContext {
        real_git: PathBuf::from(super::support::REAL_GIT),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
    };
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &context,
        std::fs::canonicalize(repo.as_path()).expect("canonical source").as_path(),
        new_stage_container(one.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Paths(vec![String::from(RULES_FILE)]),
    )
    .expect("staged");
    create_journal(common.as_path(), one.as_path(), &snapshot).expect("journal");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1);
    // A read-only command from the linked worktree recovers it before Git runs.
    let status: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(status.code, Some(0), "{}", stderr_of(&status));
    assert_eq!(status.stdout, b"true\n");
    assert_eq!(
        stderr_of(&status),
        "cli-git: recovered ignored-state copies for 1 worktree transaction.\n"
    );
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    assert!(one.join(RULES_FILE).exists());
    assert_scanner_has_rules(&check_all(&fixture, one.as_path()));
    // Recovery is not repeated.
    let again: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert!(again.stderr.is_empty(), "{}", stderr_of(&again));
    remove(&fixture);
}

/// A copy killed while it was still staging, before any journal existed, leaves a stage that the next command removes:
/// the stage is reported once, its claim goes, and the command then runs as usual.
#[test]
fn a_stage_abandoned_before_its_journal_is_removed_by_the_next_command() {
    let fixture: Fixture = fixture("copy-abandoned-stage");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let opted_out: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(opted_out.code, Some(0));
    let one: PathBuf = std::fs::canonicalize(fixture.root.join("one")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    // The state a killed copy leaves: the claim, and a half-filled private stage that no journal names.
    let claim = claim_stage(common.as_path(), one.as_path()).expect("claim");
    let stage: PathBuf = claim.container().to_path_buf();
    std::fs::create_dir(stage.as_path()).expect("stage");
    std::fs::set_permissions(stage.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private");
    std::fs::create_dir(stage.join("payload")).expect("payload");
    std::fs::write(stage.join("payload/partial"), b"copied so far").expect("partial copy");
    let status: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(status.code, Some(0), "{}", stderr_of(&status));
    assert_eq!(status.stdout, b"true\n");
    assert!(stderr_of(&status).starts_with("cli-git: removed the stage "), "{}", stderr_of(&status));
    assert!(!stage.exists());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"));
    // Removal is not repeated.
    let again: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert!(again.stderr.is_empty(), "{}", stderr_of(&again));
    remove(&fixture);
}

/// A nested Git command that inherits a live settlement lease is forwarded without settling again,
/// while a token that does not belong to the lock is not trusted.
#[test]
fn a_nested_command_with_a_foreign_lease_still_runs() {
    let fixture: Fixture = fixture("copy-lease");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(0));
    let one: PathBuf = fixture.root.join("one");
    let observed: Observed = observe(
        wrapped(&fixture)
            .current_dir(&one)
            .env("CLI_GIT_WORKTREE_COPY_LEASE", "not-the-lock-token")
            .args(["rev-parse", "--is-inside-work-tree"]),
        b"",
    );
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(observed.stdout, b"true\n");
    remove(&fixture);
}

/// Git runs a hook of the new worktree while the settlement lock is held;
///  a worktree that hook creates through the wrapper inherits the lease instead of waiting for the lock it is inside.
#[test]
fn a_hook_that_creates_a_worktree_runs_under_the_inherited_lease() {
    let fixture: Fixture = fixture("copy-hook-lease");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    // A configuration hook: traditional hooks never run through the wrapper's real-Git starts.
    let hook: PathBuf = fixture.root.join("post-checkout-hook");
    super::support::executable(
        hook.as_path(),
        b"#!/bin/sh\nif [ -n \"$CLI_GIT_WORKTREE_COPY_LEASE\" ] && [ ! -e ../nested ]; then\n  printf 'leased' > ../hook-saw-lease\n  \"$CLI_GIT_TEST_WRAPPER\" worktree add --quiet -b nested ../nested || exit 1\nfi\n",
    );
    super::support::git(&fixture, repo.as_path(), &["config", "hook.control.event", "post-checkout"]);
    super::support::git(&fixture, repo.as_path(), &["config", "hook.control.command", hook.to_str().expect("UTF-8")]);
    // Git puts its own directory first on a hook's PATH, so the hook names the wrapper explicitly.
    let created: Observed = observe(
        wrapped(&fixture)
            .current_dir(&repo)
            .env("CLI_GIT_TEST_WRAPPER", fixture.root.join("bin/git"))
            .args(["worktree", "add", "--quiet", "-b", "one", "../one"]),
        b"",
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(
        std::fs::read(fixture.root.join("hook-saw-lease")).expect("the hook saw the lease"),
        b"leased"
    );
    assert!(fixture.root.join("nested/.git").exists(), "the nested creation ran");
    assert!(fixture.root.join("one").join(RULES_FILE).exists(), "the outer copy finished");
    // The outer settlement sees both registrations, so it copies into both worktrees.
    assert!(fixture.root.join("nested").join(RULES_FILE).exists());
    assert!(stderr_of(&created).ends_with(" into 2 new worktrees.\n"), "{}", stderr_of(&created));
    remove(&fixture);
}

/// A copy interrupted from the main checkout is recovered by the next worktree creation from the main checkout,
/// before Git creates the next worktree.
#[test]
fn an_interrupted_main_checkout_copy_is_recovered_by_the_next_creation() {
    let fixture: Fixture = fixture("copy-recovery-main");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "one", "../one"],
        )
        .code,
        Some(0)
    );
    let one: PathBuf = std::fs::canonicalize(fixture.root.join("one")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let context: GitContext = GitContext {
        real_git: PathBuf::from(super::support::REAL_GIT),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
    };
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &context,
        std::fs::canonicalize(repo.as_path()).expect("canonical source").as_path(),
        new_stage_container(one.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Paths(vec![String::from(RULES_FILE)]),
    )
    .expect("staged");
    create_journal(common.as_path(), one.as_path(), &snapshot).expect("journal");
    let next: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(next.code, Some(0), "{}", stderr_of(&next));
    assert!(
        stderr_of(&next).starts_with("cli-git: recovered ignored-state copies for 1 worktree transaction.\n"),
        "{}",
        stderr_of(&next)
    );
    assert!(one.join(RULES_FILE).exists());
    assert!(fixture.root.join("two").join(RULES_FILE).exists());
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    remove(&fixture);
}

/// Moving a worktree registers nothing new, so it succeeds from either kind of source and prints no summary.
#[test]
fn moving_a_worktree_is_forwarded_and_prints_no_summary() {
    let fixture: Fixture = fixture("copy-move");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    assert_eq!(
        run_wrapped(
            &fixture,
            repo.as_path(),
            &["worktree", "add", "--quiet", "-b", "one", "../one"],
        )
        .code,
        Some(0)
    );
    let moved: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "move", "../one", "../moved"]);
    assert_eq!(moved.code, Some(0), "{}", stderr_of(&moved));
    assert!(moved.stderr.is_empty(), "{}", stderr_of(&moved));
    assert!(fixture.root.join("moved").join(RULES_FILE).exists());
    let moved_again: Observed = run_wrapped(
        &fixture,
        fixture.root.join("moved").as_path(),
        &["worktree", "move", "../moved", "../final"],
    );
    assert_eq!(moved_again.code, Some(0), "{}", stderr_of(&moved_again));
    assert!(moved_again.stderr.is_empty(), "{}", stderr_of(&moved_again));
    remove(&fixture);
}

/// A copy that fails after Git created the worktree ends with status 2 and the copy's own message,
///  leaves the created worktree in place, and leaves no lock or journal behind.
#[test]
fn a_failed_copy_ends_with_status_two_and_its_message() {
    let fixture: Fixture = fixture("copy-failure");
    let repo: PathBuf = repository_with_rules(&fixture, false);
    // A file the caller cannot read cannot be copied, and Git still lists it as an ignored file of the checkout.
    std::fs::create_dir_all(repo.join(".cache")).expect("cache directory");
    std::fs::write(repo.join(RULES_FILE), b"rules\n").expect("rules");
    std::fs::set_permissions(repo.join(RULES_FILE), std::fs::Permissions::from_mode(0o000))
        .expect("unreadable");
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(2), "{}", stderr_of(&created));
    assert!(
        stderr_of(&created).starts_with("cli-git: could not stage ignored state from "),
        "{}",
        stderr_of(&created)
    );
    assert!(!stderr_of(&created).contains("cli-git: copied"), "{}", stderr_of(&created));
    assert!(fixture.root.join("one/.git").exists(), "Git created the worktree");
    assert!(!fixture.root.join("one").join(RULES_FILE).exists());
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    std::fs::set_permissions(repo.join(RULES_FILE), std::fs::Permissions::from_mode(0o600))
        .expect("readable again");
    // The failed copy released the lock, so the next creation is not held up.
    let next: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(next.code, Some(0), "{}", stderr_of(&next));
    remove(&fixture);
}

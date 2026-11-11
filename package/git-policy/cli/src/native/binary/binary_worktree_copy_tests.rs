//! What:
//!  The linked-worktree ignored-state copy through the built executable:
//!  `git worktree add` from the main checkout and from a linked worktree brings the generated forbidden-strings
//!  rules file into the new worktree,
//! and the scanner there finds its rules.
//! Why:
//!  The user decision of 2026-10-12 is that a fresh worktree must get `.cache/forbidden-strings.rules.txt` when the
//!  wrapper creates it;
//! a source without the file copies nothing,
//! so the scanner fails closed instead of scanning
//!  with fewer rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // run(wrappedGit, ['worktree', 'add', ...]); expect(run(wrappedGit, ['monochromatic-git-policy-cli', 'check', '--all']).status).toBe(1);
//! ```

/// Import the shared fixtures and the bounded process helpers.
use super::support::{
    Fixture, Observed, dead_copy_owner, fixture, git, observe, remove, repository, run_wrapped, stderr_of, wrapped,
};
use git_policy_cli::worktree_copy_journal::{create_journal_for, read_pending_journals};
use git_policy_cli::worktree_copy_selection::CopySelection;
use git_policy_cli::worktree_copy_stage::{StagedSnapshot, new_stage_container, stage_ignored_snapshot};
use git_policy_cli::worktree_copy_stage_claim::{claim_stage_for, stage_claims_exist};
use git_policy_cli::transaction_git::GitContext;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Where the generated rules file lives,
/// as the configuration's `rulesFile` names it.
const RULES_FILE: &str = ".cache/forbidden-strings.rules.txt";

/// The text the rules file forbids and the tracked file `a.txt` contains.
pub(super) fn needle() -> String {
    return ["PLANTED", "COPY", "NEEDLE"].join("_");
}

/// A repository whose committed configuration names the rules file,
/// with `a.txt` holding the forbidden text.
/// The rules file and a second ignored directory exist only when `with_rules` is set.
pub(super) fn repository_with_rules(fixture: &Fixture, with_rules: bool) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(
        repo.join("monochromatic-git-policy-cli.config.jsonc"),
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
        &["add", "--", "monochromatic-git-policy-cli.config.jsonc", ".gitignore", "a.txt"],
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
        &["monochromatic-git-policy-cli", "check", "--policy", "security/forbidden-strings", "--all"],
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

/// The check could not load rules and scanned nothing;
/// the message names the missing path.
fn assert_scanner_failed_closed(observed: &Observed, worktree: &Path) {
    let stdout: String = String::from_utf8_lossy(&observed.stdout).into_owned();
    assert_eq!(observed.code, Some(2), "{stdout}");
    assert!(stdout.contains("\"code\":\"policy-incomplete\""), "{stdout}");
    assert!(
        stdout.contains(worktree.join(RULES_FILE).to_string_lossy().as_ref()),
        "{stdout}"
    );
}

/// A worktree added from the main checkout receives everything the checkout ignores (the rules file,
/// its sibling and the
/// heavy tree),
/// and the scanner there finds its rules;
///  the same add with the opt-out leaves the scanner without rules.
#[test]
fn worktree_added_from_the_main_checkout_receives_its_ignored_state() {
    let fixture: Fixture = fixture("copy-main");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let summary: String = stderr_of(&created);
    assert!(
        summary.starts_with("monochromatic-git-policy-cli: copied ")
            && summary.contains(format!(" ignored filesystem entries from \"{}\" into 1 new worktree.\n", repo.display()).as_str()),
        "{summary}"
    );
    let one: PathBuf = fixture.root.join("one");
    assert_eq!(
        std::fs::read(one.join(RULES_FILE)).expect("copied rules"),
        format!("{}\n", needle()).into_bytes()
    );
    assert_eq!(std::fs::read(one.join(".cache/other.txt")).expect("sibling"), b"other\n", "the whole ignored state arrives");
    assert_eq!(std::fs::read(one.join("heavy/data")).expect("heavy tree"), b"payload\n");
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

/// A configuration naming several rules files,
/// one optional and present,
/// one optional and absent,
///  copies every ignored file the main checkout has,
///  and the scanner in the new worktree reads both present files.
#[test]
fn worktree_added_from_the_main_checkout_receives_every_listed_rules_file() {
    let fixture: Fixture = fixture("copy-listed");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(
        repo.join("monochromatic-git-policy-cli.config.jsonc"),
        format!(
            "{{ \"policies\": {{ \"security/forbidden-strings\": [\"error\", {{ \"builtinRules\": false, \"rulesFile\": [\"{RULES_FILE}\", {{ \"path\": \".cache/local-rules.txt\", \"optional\": true }}, {{ \"path\": \".cache/never.txt\", \"optional\": true }}] }}] }} }}\n"
        ),
    )
    .expect("configuration");
    std::fs::write(repo.join(".gitignore"), b".cache/\n").expect("gitignore");
    std::fs::write(repo.join("a.txt"), format!("first\n{}\n", needle())).expect("shared needle file");
    std::fs::write(repo.join("b.txt"), b"first\nLISTED_LOCAL_COPY_NEEDLE\n").expect("local needle file");
    git(
        &fixture,
        repo.as_path(),
        &["add", "--", "monochromatic-git-policy-cli.config.jsonc", ".gitignore", "a.txt", "b.txt"],
    );
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=configuration"]);
    std::fs::create_dir_all(repo.join(".cache")).expect("cache directory");
    std::fs::write(repo.join(RULES_FILE), format!("{}\n", needle())).expect("shared rules");
    std::fs::write(repo.join(".cache/local-rules.txt"), b"LISTED_LOCAL_COPY_NEEDLE\n").expect("local rules");
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let one: PathBuf = fixture.root.join("one");
    assert!(one.join(RULES_FILE).exists(), "the required file is copied");
    assert!(one.join(".cache/local-rules.txt").exists(), "the present optional file is copied");
    assert!(!one.join(".cache/never.txt").exists(), "an absent optional file is not invented");
    let checked: Observed = check_all(&fixture, one.as_path());
    let stdout: String = String::from_utf8_lossy(&checked.stdout).into_owned();
    assert_eq!(checked.code, Some(1), "{stdout} {}", stderr_of(&checked));
    assert!(stdout.contains("a.txt") && stdout.contains("b.txt"), "both rules files apply: {stdout}");
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
    // The main checkout's `.cache` (directory, rules file, sibling) and `heavy` (directory, data) arrived in `one`, which
    // `two` copies in turn.
    assert_eq!(
        stderr_of(&created),
        format!(
            "monochromatic-git-policy-cli: copied 5 ignored filesystem entries from \"{}\" into 1 new worktree.\n",
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

/// A main checkout without a rules file copies nothing,
/// and the scanner in the new worktree fails closed.
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

/// A main checkout whose configuration names no rules file still hands over its ignored state:
/// the copy reads no configuration.
#[test]
fn a_main_checkout_without_a_configured_rules_file_still_copies_its_ignored_state() {
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
    assert!(stderr_of(&created).starts_with("monochromatic-git-policy-cli: copied "), "{}", stderr_of(&created));
    assert_eq!(std::fs::read(fixture.root.join("one").join(RULES_FILE)).expect("copied"), b"rules\n");
    remove(&fixture);
}

/// A worktree Git registers with relative paths (the option,
/// or the user's `worktree.useRelativePaths`) is copied into like
/// any other.
#[test]
fn a_worktree_registered_with_relative_paths_is_copied_into() {
    let fixture: Fixture = fixture("copy-relative-paths");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let option: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "--relative-paths", "-b", "one", "../one"],
    );
    assert_eq!(option.code, Some(0), "{}", stderr_of(&option));
    assert!(stderr_of(&option).starts_with("monochromatic-git-policy-cli: copied "), "{}", stderr_of(&option));
    assert_eq!(std::fs::read(fixture.root.join("one").join(RULES_FILE)).expect("copied"), format!("{}\n", needle()).into_bytes());
    let configured: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["-c", "worktree.useRelativePaths=true", "worktree", "add", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(configured.code, Some(0), "{}", stderr_of(&configured));
    assert!(stderr_of(&configured).starts_with("monochromatic-git-policy-cli: copied "), "{}", stderr_of(&configured));
    assert_eq!(std::fs::read(fixture.root.join("two").join(RULES_FILE)).expect("copied"), format!("{}\n", needle()).into_bytes());
    remove(&fixture);
}

/// A command Git expands from an alias into `worktree add` is copied too,
/// and a failed Git keeps its own status.
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
    assert!(!stderr_of(&refused).contains("monochromatic-git-policy-cli: copied"));
    // Nothing was copied into the destination Git refused. What may stand there is the wrapper's own reservation of it:
    //  an empty subvolume Git's failure leaves to the ladder to remove on btrfs, and an empty directory elsewhere, which
    //  Git fills when the same add is asked for again.
    let again: PathBuf = fixture.root.join("again");
    match std::fs::read_dir(&again) {
        Ok(listing) => assert_eq!(listing.count(), 0, "the destination holds nothing but the wrapper's empty reservation"),
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "the destination is absent or empty: {error}"),
    }
    remove(&fixture);
}

/// A copy interrupted before it finished is recovered by the next command run from a linked worktree:
/// the staged files are installed,
/// the recovery is reported,
/// and the scanner then finds its rules.
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
        common_dir: None,
    };
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &context,
        std::fs::canonicalize(repo.as_path()).expect("canonical source").as_path(),
        new_stage_container(one.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    create_journal_for(common.as_path(), one.as_path(), &snapshot, &dead_copy_owner()).expect("journal");
    assert_eq!(read_pending_journals(common.as_path()).expect("pending").len(), 1);
    // A read-only command from the linked worktree recovers it before Git runs.
    let status: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(status.code, Some(0), "{}", stderr_of(&status));
    assert_eq!(status.stdout, b"true\n");
    assert_eq!(
        stderr_of(&status),
        "monochromatic-git-policy-cli: recovered ignored-state copies for 1 worktree transaction.\n"
    );
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    assert!(one.join(RULES_FILE).exists());
    assert_scanner_has_rules(&check_all(&fixture, one.as_path()));
    // Recovery is not repeated.
    let again: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert!(again.stderr.is_empty(), "{}", stderr_of(&again));
    remove(&fixture);
}

/// A copy killed while it was still staging,
/// before any journal existed,
/// leaves a stage that the next command removes:
/// the stage is reported once,
/// its claim goes,
/// and the command then runs as usual.
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
    let claim = claim_stage_for(common.as_path(), one.as_path(), &dead_copy_owner()).expect("claim");
    let stage: PathBuf = claim.container().to_path_buf();
    std::fs::create_dir(stage.as_path()).expect("stage");
    std::fs::set_permissions(stage.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private");
    std::fs::create_dir(stage.join("payload")).expect("payload");
    std::fs::write(stage.join("payload/partial"), b"copied so far").expect("partial copy");
    let status: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(status.code, Some(0), "{}", stderr_of(&status));
    assert_eq!(status.stdout, b"true\n");
    assert!(stderr_of(&status).starts_with("monochromatic-git-policy-cli: removed the stage "), "{}", stderr_of(&status));
    assert!(!stage.exists());
    assert!(!stage_claims_exist(common.as_path()).expect("claims"));
    // Removal is not repeated.
    let again: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert!(again.stderr.is_empty(), "{}", stderr_of(&again));
    remove(&fixture);
}

/// Git runs a smudge filter while it checks out the new worktree;
/// a worktree that filter creates through the wrapper is an
///  add of its own:
/// it settles itself from its own source,
/// with its own summary,
/// and the outer add copies into its own
///  worktree only.
/// No lock or lease is passed to the filter,
/// so nothing waits.
/// (A hook used to play this part;
/// the wrapper
///  supports no external hook program,
/// and a filter is a program Git starts with the same environment.)
#[test]
fn a_filter_that_creates_a_worktree_through_the_wrapper_settles_it_itself() {
    let fixture: Fixture = fixture("copy-filter-nested");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    let filter: PathBuf = fixture.root.join("smudge-filter");
    super::support::executable(
        filter.as_path(),
        b"#!/bin/sh\ncat\nif [ ! -e ../nested ]; then\n  printf 'ran' > ../filter-ran\n  \"$CLI_GIT_TEST_WRAPPER\" worktree add --quiet -b nested ../nested >&2 || exit 1\nfi\n",
    );
    std::fs::write(repo.join(".gitattributes"), b"*.probe filter=probe\n").expect("attributes");
    std::fs::write(repo.join("x.probe"), b"probe\n").expect("probe file");
    super::support::git(&fixture, repo.as_path(), &["add", "--", ".gitattributes", "x.probe"]);
    super::support::git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=probe"]);
    super::support::git(&fixture, repo.as_path(), &["config", "filter.probe.smudge", filter.to_str().expect("UTF-8")]);
    // A source whose tracked files are all older than its index is cloned or snapshotted by the worktree ladder, and then
    //  Git checks nothing out and runs no filter. Whether the files are older depends on whether a second boundary fell
    //  between writing them and committing, so an untracked file that no ignore rule covers makes the source dirty on
    //  every host and leaves the checkout, which is what this control is about, to Git.
    std::fs::write(repo.join("untracked-so-git-checks-out.txt"), b"u\n").expect("untracked file");
    // Git puts its own directory first on a child's PATH, so the filter names the wrapper explicitly.
    let created: Observed = observe(
        wrapped(&fixture)
            .current_dir(&repo)
            .env("CLI_GIT_TEST_WRAPPER", fixture.root.join("bin/git"))
            .args(["worktree", "add", "--quiet", "-b", "one", "../one"]),
        b"",
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(std::fs::read(fixture.root.join("filter-ran")).expect("the filter ran"), b"ran");
    assert!(fixture.root.join("nested/.git").exists(), "the nested creation ran");
    assert!(fixture.root.join("one").join(RULES_FILE).exists(), "the outer copy finished");
    // The outer add copies into its own worktree only; the nested add's source is the half-made `one`, which the outer add
    //  is still filling, so the nested add copies nothing, says why, and writes no summary, while the outer add wrote one.
    assert!(!fixture.root.join("nested").join(RULES_FILE).exists());
    let said: String = stderr_of(&created);
    assert!(said.contains("was made, but it received no ignored-state copy") && said.contains("is itself being filled by another `git worktree add`"), "the nested add says why it copied nothing: {said}");
    assert_eq!(said.matches(" into 1 new worktree.\n").count(), 1, "{said}");
    remove(&fixture);
}

/// What:
///  A copy interrupted from the main checkout is made by the first command run in the worktree whose files it fills,
///  and a command in another worktree leaves it there and says which worktree makes it.
/// Why:
///  The copy is that worktree's own work:
/// staging and installing the main checkout's ignored state costs what that tree costs,
/// and a command in another worktree gains nothing from it
/// (`doc/planning/cli-git-owed-copy-off-other-commands-path.md`).
/// The copy must still be made,
/// and the first command run in the worktree is what makes it.
#[test]
fn an_interrupted_main_checkout_copy_is_made_by_the_first_command_in_its_worktree() {
    let fixture: Fixture = fixture("copy-recovery-main");
    let repo: PathBuf = repository_with_rules(&fixture, true);
    // `one` stands as a kill between Git's checkout and the copy leaves it: registered, with a whole checkout and no
    // ignored state of its own.
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
        common_dir: None,
    };
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &context,
        std::fs::canonicalize(repo.as_path()).expect("canonical source").as_path(),
        new_stage_container(one.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    create_journal_for(common.as_path(), one.as_path(), &snapshot, &dead_copy_owner()).expect("journal");
    // A creation in the main checkout is a command in another worktree: it names the worktree whose files are missing,
    // writes nothing there, and copies the main checkout's state into its own new worktree only.
    let elsewhere: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(elsewhere.code, Some(0), "{}", stderr_of(&elsewhere));
    assert!(!one.join(RULES_FILE).exists(), "no command in another worktree fills it: {}", stderr_of(&elsewhere));
    assert!(!one.join("heavy/data").exists(), "{}", stderr_of(&elsewhere));
    assert!(
        stderr_of(&elsewhere).contains("the first command run in that worktree makes it, and this command does not wait for it"),
        "the line says which worktree makes it and that nothing waits here: {}",
        stderr_of(&elsewhere)
    );
    assert_eq!(
        read_pending_journals(common.as_path()).expect("pending").len(),
        1,
        "the journal stays for the worktree whose command makes the copy"
    );
    assert!(fixture.root.join("two").join(RULES_FILE).exists(), "the creating command still copies into its own worktree");
    assert!(fixture.root.join("two").join("heavy/data").exists());
    // The first command run in `one` makes the copy that worktree owes, before it does anything of its own.
    let made: Observed = run_wrapped(&fixture, one.as_path(), &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(made.code, Some(0), "{}", stderr_of(&made));
    assert!(
        stderr_of(&made).starts_with("monochromatic-git-policy-cli: recovered ignored-state copies for 1 worktree transaction.\n"),
        "{}",
        stderr_of(&made)
    );
    assert_eq!(
        std::fs::read(one.join(RULES_FILE)).expect("the recovered copy carries the rules file"),
        format!("{}\n", needle()).into_bytes()
    );
    assert!(one.join("heavy/data").exists(), "the recovered copy carries the whole ignored state of the main checkout");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty(), "a finished copy takes its journal with it");
    remove(&fixture);
}

/// Moving a worktree registers nothing new,
/// so it succeeds from either kind of source and prints no summary.
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
///  leaves the created worktree in place,
/// and leaves no lock or journal behind.
#[test]
fn a_failed_copy_ends_with_status_two_and_its_message() {
    let fixture: Fixture = fixture("copy-failure");
    let repo: PathBuf = repository_with_rules(&fixture, false);
    // A named pipe cannot be copied, and its ignored directory is a root of the copy; an unreadable file is left out instead.
    std::fs::create_dir_all(repo.join(".cache")).expect("cache directory");
    let pipe = std::ffi::CString::new(repo.join(RULES_FILE).as_os_str().as_encoded_bytes()).expect("pipe path");
    // SAFETY: `pipe` is a NUL-terminated path that outlives the call.
    assert_eq!(unsafe { libc::mkfifo(pipe.as_ptr(), 0o600) }, 0, "mkfifo");
    let created: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(created.code, Some(2), "{}", stderr_of(&created));
    assert!(
        stderr_of(&created).starts_with("monochromatic-git-policy-cli: ignored path ")
            && stderr_of(&created).contains("has unsupported filesystem type."),
        "{}",
        stderr_of(&created)
    );
    assert!(!stderr_of(&created).contains("monochromatic-git-policy-cli: copied"), "{}", stderr_of(&created));
    assert!(fixture.root.join("one/.git").exists(), "Git created the worktree");
    assert!(!fixture.root.join("one").join(RULES_FILE).exists());
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    assert!(read_pending_journals(common.as_path()).expect("pending").is_empty());
    // The failed copy released the lock, so the next creation is not held up.
    let next: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "two", "../two"],
    );
    assert_eq!(next.code, Some(0), "{}", stderr_of(&next));
    remove(&fixture);
}

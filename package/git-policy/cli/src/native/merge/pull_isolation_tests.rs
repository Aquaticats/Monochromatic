//! Controls for how a pull is read:
//! which command lines and settings make it a merge the wrapper completes.

use super::*;
use crate::forwarded_command::Route;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_output, isolated_overlay, remove, repository};
use std::path::PathBuf;

/// Command-line words as the owned text a command holds.
fn words(list: &[&str]) -> Vec<OsString> {
    return list.iter().map(|word| { return OsString::from(*word); }).collect();
}

/// The settings that add steps the split pull does not mirror,
/// as the keys present.
fn settings(list: &[&str]) -> Vec<String> {
    return list.iter().map(|key| { return String::from(*key); }).collect();
}

/// What the rebase settings of a repository holding these `git config --add` entries say about a pull,
/// with what Git itself reads there.
fn statement_of(name: &str, entries: &[(&str, &str)]) -> (RebaseStatement, Option<String>) {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for (key, value) in entries {
        git(repo.as_path(), &["config", "--add", key, value]);
    }
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let statement: RebaseStatement = rebase_statement(&context, "main");
    // The key `config_get_rebase` reads first when it is present, as Git judges this repository.
    let named: bool = entries.iter().any(|(key, _value)| { return *key == "branch.main.rebase"; });
    let key: &str = if named { "branch.main.rebase" } else { "pull.rebase" };
    let asked = git_output(repo.as_path(), &["config", "--bool", "--get", key]);
    let read: Option<String> = if asked.status.success() {
        Some(String::from_utf8_lossy(asked.stdout.as_slice()).trim().to_owned())
    } else {
        None
    };
    remove(parent.as_path());
    return (statement, read);
}

/// Each option goes to the half of the pull that takes it,
/// the shared ones to both,
/// and the operands stay in order.
#[test]
fn options_go_to_the_half_that_takes_them() {
    let plan: PullPlan = plan_pull(words(&["--no-rebase", "--no-edit", "--prune", "-q", "origin", "main"]).as_slice()).expect("read exactly");
    assert_eq!(plan.merge, words(&["--no-edit", "-q"]));
    assert_eq!(plan.fetch, words(&["--prune", "-q"]));
    assert_eq!(plan.places, words(&["origin", "main"]));
    assert!(plan.merge_stated);
    assert!(!plan.fast_forward_stated);
    assert!(plan_pull(words(&["--no-ff"]).as_slice()).expect("read exactly").fast_forward_stated);
}

/// An option this does not read exactly,
/// an abbreviation,
/// `--`,
/// or a refspec with a destination leaves the pull to native Git.
#[test]
fn what_is_not_read_exactly_is_native_gits() {
    for rest in [
        vec!["--rebase"],
        vec!["--autostash"],
        vec!["--dry-run"],
        vec!["-s", "ours"],
        vec!["--no-edi"],
        vec!["--"],
        vec!["origin", "main:main"],
    ] {
        assert_eq!(plan_pull(words(rest.as_slice()).as_slice()), None, "{rest:?}");
    }
    assert!(plan_pull(words(&["origin"]).as_slice()).is_some(), "a repository operand alone is read");
    assert!(plan_pull(words(&["ssh://host:22/repo"]).as_slice()).is_some(), "a colon in the repository is not a refspec");
}

/// The command line and the settings decide whether the pull merges,
/// as `cmd_pull` decides it.
#[test]
fn merging_is_stated_by_the_line_or_the_settings() {
    let typed: PullPlan = plan_pull(words(&["--no-rebase"]).as_slice()).expect("read exactly");
    let plain: PullPlan = plan_pull(words(&[]).as_slice()).expect("read exactly");
    let ff: PullPlan = plan_pull(words(&["--no-ff"]).as_slice()).expect("read exactly");
    assert!(merge_is_stated(&typed, settings(&[]).as_slice(), RebaseStatement::NotMerge), "the line overrides the setting");
    assert!(merge_is_stated(&plain, settings(&[]).as_slice(), RebaseStatement::Merge));
    assert!(!merge_is_stated(&plain, settings(&[]).as_slice(), RebaseStatement::NotMerge));
    assert!(!merge_is_stated(&plain, settings(&[]).as_slice(), RebaseStatement::Unstated), "nothing stated: Git refuses divergent branches itself");
    assert!(merge_is_stated(&ff, settings(&[]).as_slice(), RebaseStatement::Unstated), "a typed way of reconciling is a stated merge");
    assert!(!merge_is_stated(&typed, settings(&["pull.autostash"]).as_slice(), RebaseStatement::Merge), "a stash around the pull is native Git's");
    assert!(!merge_is_stated(&typed, settings(&["pull.ff"]).as_slice(), RebaseStatement::Merge), "pull.ff is Git's to combine");
    assert!(merge_is_stated(&ff, settings(&["pull.ff"]).as_slice(), RebaseStatement::Unstated), "the typed choice replaces pull.ff");
    assert!(!merge_is_stated(&typed, settings(&["submodule.recurse"]).as_slice(), RebaseStatement::Merge));
}

/// The rebase setting is read the way Git's own pull reads it:
/// the last of several values,
/// and every spelling of a boolean.
/// Why:
///   The wrapper read the first value and a short list of spellings,
/// so a valid setting Git reads as false kept the pull native,
/// where native Git's merge failed over another session's staged work.
#[test]
fn the_rebase_setting_is_read_the_way_git_reads_it() {
    let cases: [(&str, &[(&str, &str)], RebaseStatement); 21] = [
        ("none", &[], RebaseStatement::Unstated),
        ("false", &[("pull.rebase", "false")], RebaseStatement::Merge),
        ("no", &[("pull.rebase", "no")], RebaseStatement::Merge),
        ("off-shouted", &[("pull.rebase", "OFF")], RebaseStatement::Merge),
        ("zero", &[("pull.rebase", "0")], RebaseStatement::Merge),
        ("double-zero", &[("pull.rebase", "00")], RebaseStatement::Merge),
        ("hex-zero", &[("pull.rebase", "0x0")], RebaseStatement::Merge),
        ("plus-zero", &[("pull.rebase", "+0")], RebaseStatement::Merge),
        ("zero-with-unit", &[("pull.rebase", "0k")], RebaseStatement::Merge),
        ("empty", &[("pull.rebase", "")], RebaseStatement::Merge),
        ("true", &[("pull.rebase", "true")], RebaseStatement::NotMerge),
        ("one", &[("pull.rebase", "1")], RebaseStatement::NotMerge),
        ("merges", &[("pull.rebase", "merges")], RebaseStatement::NotMerge),
        ("interactive", &[("pull.rebase", "interactive")], RebaseStatement::NotMerge),
        ("unreadable", &[("pull.rebase", "nonsense")], RebaseStatement::NotMerge),
        ("duplicate-true-then-false", &[("pull.rebase", "true"), ("pull.rebase", "false")], RebaseStatement::Merge),
        ("duplicate-false-then-true", &[("pull.rebase", "false"), ("pull.rebase", "true")], RebaseStatement::NotMerge),
        ("branch-duplicate-true-then-false", &[("branch.main.rebase", "true"), ("branch.main.rebase", "false")], RebaseStatement::Merge),
        ("branch-zero", &[("branch.main.rebase", "00")], RebaseStatement::Merge),
        ("branch-false-beats-pull-true", &[("branch.main.rebase", "false"), ("pull.rebase", "true")], RebaseStatement::Merge),
        ("branch-true-beats-pull-false", &[("branch.main.rebase", "true"), ("pull.rebase", "false")], RebaseStatement::NotMerge),
    ];
    for (name, entries, expected) in cases {
        let (statement, git_reads) = statement_of(format!("pull-rebase-{name}").as_str(), entries);
        assert_eq!(statement, expected, "{name}: Git reads {git_reads:?}");
        // A case whose expectation Git reads as a boolean must expect exactly what Git answers, so the table cannot
        // drift from Git's own reading of the value.
        if let Some(read) = git_reads {
            let expected_read: &str = if expected == RebaseStatement::Merge { "false" } else { "true" };
            assert_eq!(read, expected_read, "{name}: the expectation is Git's own reading");
        }
    }
}

/// What:
///  A pull whose merge fast-forwards is split into its fetch and its merge,
///  and the merge goes through the isolated merge only when something of the person's stands at the
///  paths it writes.
/// Why:
///   A fast-forward is native Git's whole while nothing is in its way,
///  so the split pull keeps that ordinary case out of
///  the carry;
///  an ignored file at a path the incoming commit adds,
///  or an unsaved edit at one it changes,
///  is the work the carry
///  keeps, and the pull owed it the same answer a typed `git merge` gives (`SPEC.md`, "Carry edits forward").
///  `--ff-only` keeps Git's own refusal when the merge cannot fast-forward,
///  so that pull stays native Git's
///  whole after its idempotent fetch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(isolateOf(cleanClone)).toBe(false); expect(isolateOf(cloneWithAnIgnoredFileInTheWay)).toBe(true); expect(ffOnlyDiverged).toEqual({ kind: 'native' });
/// ```
#[test]
fn a_fast_forward_pull_is_isolated_only_when_something_stands_in_its_way() {
    let clean: (PathBuf, PathBuf) = a_clone_with_an_upstream_commit("pull-ff-clean", None);
    assert_eq!(isolate_of(clean.1.as_path(), &[]), Some(false), "nothing stands in the way, so the child merge is the whole answer");
    remove(clean.0.as_path());
    let ignored: (PathBuf, PathBuf) = a_clone_with_an_upstream_commit("pull-ff-ignored", Some("ign.txt"));
    assert_eq!(isolate_of(ignored.1.as_path(), &[]), Some(true), "an ignored file stands at the added path, so the merge is carried");
    remove(ignored.0.as_path());
    let edited: (PathBuf, PathBuf) = a_clone_with_an_upstream_commit("pull-ff-edited", None);
    std::fs::write(edited.1.join("a.txt"), b"the person's own line\n").expect("file");
    assert_eq!(isolate_of(edited.1.as_path(), &[]), Some(true), "an unsaved edit stands at a changed path, so the merge is carried");
    remove(edited.0.as_path());
    let diverged: (PathBuf, PathBuf) = a_clone_with_an_upstream_commit("pull-ff-only-diverged", None);
    std::fs::write(diverged.1.join("m.txt"), b"a commit of the person's own\n").expect("file");
    git(diverged.1.as_path(), &["add", "--all"]);
    git(diverged.1.as_path(), &["commit", "--quiet", "--message=main"]);
    // The local branch has moved, so the fetched head no longer fast-forwards; `--ff-only` states that Git's refusal is
    //  the answer, and the pull runs again as a whole so Git words it.
    assert_eq!(isolate_of(diverged.1.as_path(), &["--ff-only"]), None, "--ff-only over a branch that cannot fast-forward");
    remove(diverged.0.as_path());
}

/// What:
///  A clone of a bare origin whose `main` the origin has since moved ahead by one commit,
///  with the person's state at the path that commit adds when one is named.
/// Why:
///   A pull's merge half reads a real `FETCH_HEAD`,
///  so the control needs the fetch a pull makes,
///  and a local bare
///  origin needs no network.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aCloneWithAnUpstreamCommit(name, ignored) { return { parent, local }; }
/// ```
fn a_clone_with_an_upstream_commit(name: &str, ignored: Option<&str>) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let origin: PathBuf = parent.join("origin.git");
    git(parent.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", origin.to_string_lossy().as_ref()]);
    let seed: PathBuf = parent.join("seed");
    git(parent.as_path(), &["clone", "--quiet", origin.to_string_lossy().as_ref(), seed.to_string_lossy().as_ref()]);
    std::fs::write(seed.join("a.txt"), b"one\n").expect("file");
    std::fs::write(seed.join("keep.txt"), b"keep\n").expect("file");
    git(seed.as_path(), &["add", "--all"]);
    git(seed.as_path(), &["commit", "--quiet", "--message=base"]);
    git(seed.as_path(), &["push", "--quiet", "origin", "main"]);
    let local: PathBuf = parent.join("local");
    git(parent.as_path(), &["clone", "--quiet", origin.to_string_lossy().as_ref(), local.to_string_lossy().as_ref()]);
    for ignored_path in ignored.iter() {
        std::fs::write(local.join(".gitignore"), format!("{ignored_path}\n")).expect("file");
        git(local.as_path(), &["add", "--", ".gitignore"]);
        git(local.as_path(), &["commit", "--quiet", "--message=ignore"]);
        git(local.as_path(), &["push", "--quiet", "origin", "main"]);
        git(seed.as_path(), &["pull", "--quiet", "--no-rebase"]);
        git(local.as_path(), &["reset", "--quiet", "--hard", "origin/main"]);
        std::fs::write(local.join(ignored_path), b"PERSON-MARKER\n").expect("file");
    }
    // The origin moves ahead by one commit that changes `a.txt` and adds whatever the control ignores.
    std::fs::write(seed.join("a.txt"), b"one\ntwo\n").expect("file");
    for ignored_path in ignored.iter() {
        std::fs::write(seed.join(ignored_path), b"the branch's own line\n").expect("file");
        git(seed.as_path(), &["add", "--force", "--", ignored_path]);
    }
    git(seed.as_path(), &["add", "--all"]);
    git(seed.as_path(), &["commit", "--quiet", "--message=upstream"]);
    git(seed.as_path(), &["push", "--quiet", "origin", "main"]);
    return (parent, local);
}

/// What:
///  What one split pull makes of its merge half:
///  nothing when the whole pull is native Git's,
///  and otherwise whether that merge goes through the isolated merge.
/// Why:
///   The control reads the one decision the pull adds to a typed merge,
///  and the fetch it makes is the real one, from a bare
///  origin in the fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isolateOf(local, extra) { const step = pullStep(forwardFor(local, ['pull', '--no-rebase', '--no-edit', ...extra]), index); return step.kind === 'merge' ? step.isolate : undefined; }
/// ```
fn isolate_of(local: &Path, extra: &[&str]) -> Option<bool> {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), local.as_os_str().to_owned()];
    let identity: WorktreeIdentity = crate::worktree_identity::resolve_worktree_identity(
        Path::new(REAL_GIT),
        prefix.as_slice(),
        isolated_overlay().as_slice(),
    )
    .expect("identity");
    let mut words: Vec<OsString> = prefix.clone();
    for token in ["pull", "--no-rebase", "--no-edit"] {
        words.push(OsString::from(token));
    }
    for token in extra {
        words.push(OsString::from(*token));
    }
    let command: ResolvedCommand = ResolvedCommand {
        arguments: words.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("pull")),
        route: Route::Direct,
        // A pull reached here runs no shell alias.
        shell_command: None,
    };
    let forward: CoordinatedForward = CoordinatedForward {
        real_git: PathBuf::from(REAL_GIT),
        arguments: words,
        overlay: isolated_overlay(),
        identity,
        command,
        environment: Vec::new(),
        copy: None,
    };
    return match pull_step(&forward, local.join(".git").join("index").as_path()) {
        PullStep::Merge { isolate, .. } => Some(isolate),
        PullStep::Native => None,
        PullStep::Ended(run) => panic!("the pull's fetch ended the command: {run:?}"),
    };
}

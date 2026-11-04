//! Controls for the native check that decides whether a command left the branch ahead of its upstream: the walk over the
//! commit parents against real Git, and the tip decision on disposable repositories.

use super::*;
use crate::repo_read_layout::read_layout_in;
use crate::test_support::{fixture, git, remove};

/// A repository with a bare remote, `main` pushed and tracking `origin/main`.
fn repository(name: &str) -> (PathBuf, PathBuf) {
    let root = fixture(name);
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    git(&root, &["init", "--bare", "--quiet", "--initial-branch=main", "remote.git"]);
    git(&root, &["init", "--quiet", "--initial-branch=main", "repo"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=base"]);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    return (root, repo);
}

/// The commit a revision names.
fn oid(repo: &Path, revision: &str) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", revision]).stdout).trim().to_owned();
}

/// Whether real Git says `ancestor` is an ancestor of `descendant`.
fn git_says_ancestor(repo: &Path, ancestor: &str, descendant: &str) -> bool {
    return crate::test_support::git_output(repo, &["merge-base", "--is-ancestor", ancestor, descendant]).status.success();
}

/// A watch of the repository as the wrapper would start it, with the fixture's environment.
fn watch(repo: &Path) -> TipWatch {
    let overlay: Vec<(OsString, OsString)> = crate::test_support::isolated_overlay();
    return watch_tip(repo, Path::new("/usr/bin/git"), overlay.as_slice(), &[]).expect("a branch with a tip");
}

/// The walk answers like `git merge-base --is-ancestor` for ancestors, non-ancestors and the commit itself.
#[test]
fn the_walk_agrees_with_git_about_ancestry() {
    let (root, repo) = repository("atr-walk");
    let base = oid(&repo, "HEAD");
    git(&repo, &["switch", "--quiet", "--create", "side"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=side"]);
    let side = oid(&repo, "HEAD");
    git(&repo, &["switch", "--quiet", "main"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=main"]);
    git(&repo, &["merge", "--quiet", "--no-ff", "--no-edit", "side"]);
    let merged = oid(&repo, "HEAD");
    let layout = read_layout_in(&repo, crate::test_support::isolated_overlay().as_slice()).expect("layout");
    let database = ObjectDatabase::open(layout.common_dir.as_path(), None, layout.object_format).expect("database");
    for (from, target) in [(&merged, &base), (&merged, &side), (&side, &base), (&base, &side), (&side, &merged), (&base, &base)] {
        assert_eq!(
            reaches(&database, &layout, from, target, WALK_BUDGET),
            Some(git_says_ancestor(&repo, target, from)),
            "{target} from {from}"
        );
    }
    assert_eq!(reaches(&database, &layout, &merged, &base, 0), None, "a walk past its budget does not guess");
    remove(&root);
}

/// A commit on top of the upstream is handed over; the same tip is not handed over twice, and an unmoved branch is not.
#[test]
fn a_branch_that_grew_ahead_of_its_upstream_is_handed_over() {
    let (root, repo) = repository("atr-grew");
    let watched = watch(&repo);
    assert_eq!(tip_to_push(&watched), None, "nothing moved");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    assert_eq!(tip_to_push(&watched), Some(oid(&repo, "HEAD")));
    remove(&root);
}

/// A branch reset to an older commit moved backwards, which is not growth.
#[test]
fn a_branch_that_moved_backwards_is_not_handed_over() {
    let (root, repo) = repository("atr-back");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    git(&repo, &["push", "--quiet", "origin", "main"]);
    let watched = watch(&repo);
    git(&repo, &["reset", "--quiet", "--hard", "HEAD~1"]);
    assert_eq!(tip_to_push(&watched), None);
    remove(&root);
}

/// A rewritten branch (the same commits replayed) has grown past the upstream but not from its old tip: a rebase.
#[test]
fn a_rewritten_branch_is_not_handed_over() {
    let (root, repo) = repository("atr-rewrite");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=local"]);
    let watched = watch(&repo);
    git(&repo, &["commit", "--quiet", "--amend", "--allow-empty", "--message=local, rewritten"]);
    assert_eq!(tip_to_push(&watched), None);
    remove(&root);
}

/// A branch that has diverged from the tracking ref is not handed over, and neither is one without an upstream.
#[test]
fn a_diverged_branch_and_a_branch_without_an_upstream_are_not_handed_over() {
    let (root, repo) = repository("atr-diverged");
    git(&repo, &["switch", "--quiet", "--create", "loose"]);
    let loose = watch(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    assert_eq!(tip_to_push(&loose), None, "no upstream");
    git(&repo, &["switch", "--quiet", "main"]);
    let watched = watch(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=mine"]);
    // The tracking ref moves to a commit the branch does not contain, as a fetch would move it.
    git(&repo, &["switch", "--quiet", "--detach", "HEAD~1"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=theirs"]);
    let theirs = oid(&repo, "HEAD");
    git(&repo, &["update-ref", "refs/remotes/origin/main", &theirs]);
    git(&repo, &["switch", "--quiet", "main"]);
    assert_eq!(tip_to_push(&watched), None);
    remove(&root);
}

/// The overlay of a user whose own configuration file holds `settings`,
/// written to `file`.
fn overlay_with(file: &Path, settings: &str) -> Vec<(OsString, OsString)> {
    std::fs::write(file, format!("{settings}[user]\n\tname = Someone\n")).expect("global configuration");
    let mut overlay: Vec<(OsString, OsString)> = crate::test_support::isolated_overlay()
        .into_iter()
        .filter(|(name, _)| { return name != "GIT_CONFIG_GLOBAL"; })
        .collect();
    overlay.push((OsString::from("GIT_CONFIG_GLOBAL"), file.as_os_str().to_os_string()));
    return overlay;
}

/// The repository of the live report:
/// format version 0 with `extensions.worktreeConfig`,
/// `push.default = matching`,
/// a tracking branch,
/// a linked worktree,
/// and a user whose own configuration file holds `settings`.
/// A merge in it must be handed over:
/// the watch starts,
///  and after the merge the tip is the one to push.
/// `reader_checks_values` says whether the reader that checks every value answers for the user.
fn a_merge_is_handed_over_for_a_user_with(name: &str, settings: &str, reader_checks_values: bool) {
    let (root, repo) = repository(name);
    git(&repo, &["config", "extensions.worktreeConfig", "true"]);
    git(&repo, &["config", "push.default", "matching"]);
    git(&repo, &["worktree", "add", "--quiet", "--detach", "../linked"]);
    git(&repo, &["switch", "--quiet", "--create", "topic"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=topic"]);
    git(&repo, &["switch", "--quiet", "main"]);
    let overlay: Vec<(OsString, OsString)> = overlay_with(root.join("global.gitconfig").as_path(), settings);
    assert_eq!(
        read_layout_in(&repo, overlay.as_slice()).is_some(),
        reader_checks_values,
        "the premise: whether the reader that checks every value answers for this user's configuration"
    );
    let watched: TipWatch = try_watch_tip(&repo, Path::new("/usr/bin/git"), overlay.as_slice(), &[]).expect("the watch starts");
    git(&repo, &["merge", "--quiet", "--no-ff", "--no-edit", "topic"]);
    assert_eq!(tip_to_push_or_why(&watched), Ok(oid(&repo, "HEAD")));
    remove(&root);
}

/// A user-level value the typed proof still refuses (`diff.dirstat` stands in:
/// Git accepts it and the proof cannot vouch for
/// it) must not stop the trigger.
/// Before,
/// the reader declined the repository for such a value,
/// the watch never started,
/// and the merge stayed local without a word.
#[test]
fn a_user_level_setting_the_reader_cannot_prove_does_not_stop_the_trigger() {
    a_merge_is_handed_over_for_a_user_with("atr-unprovable", "[diff]\n\tdirstat = changes\n", false);
}

/// A user-level `core.fsmonitor = true`,
/// the setting of the live report,
/// is proven now,
/// so the reader that checks every
/// value answers for it,
/// and the merge is handed over too.
#[test]
fn a_user_level_fsmonitor_true_lets_the_reader_answer_and_the_trigger_start() {
    a_merge_is_handed_over_for_a_user_with("atr-fsmonitor", "[core]\n\tfsmonitor = true\n", true);
}

/// Each step that declines names itself:
/// a branch that did not move,
/// an upstream that is the tip,
/// a rewritten branch,
/// a diverged one,
/// an upstream that is not a single plain one,
/// a detached head and a repository the reader does not take.
#[test]
fn every_decline_names_its_reason() {
    let (root, repo) = repository("atr-reasons");
    let reason = |watched: &TipWatch| -> &'static str { return tip_to_push_or_why(watched).expect_err("declined").reason; };
    let at_base: TipWatch = watch(&repo);
    assert_eq!(reason(&at_base), "tip-unchanged");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    git(&repo, &["push", "--quiet", "origin", "main"]);
    assert_eq!(reason(&at_base), "upstream-equals-tip");
    let published: TipWatch = watch(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--amend", "--message=rewritten"]);
    assert_eq!(reason(&published), "not-grown");
    let rewritten: TipWatch = watch(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=on the rewrite"]);
    assert_eq!(reason(&rewritten), "not-ahead");
    git(&repo, &["config", "--unset", "branch.main.merge"]);
    assert_eq!(reason(&rewritten), "upstream-not-single");
    git(&repo, &["switch", "--quiet", "--detach"]);
    assert_eq!(reason(&rewritten), "head-detached");
    let empty = root.join("nowhere");
    std::fs::create_dir(&empty).expect("directory");
    let declined = try_watch_tip(&empty, Path::new("/usr/bin/git"), crate::test_support::isolated_overlay().as_slice(), &[]).expect_err("no repository");
    assert_eq!(declined.reason, "layout-declined");
    remove(&root);
}

/// The record line is one JSON object with the fields a person diagnoses from,
/// the reason as a word that does not change,
/// and text that needs escaping escaped.
#[test]
fn a_trigger_record_is_one_line_that_names_the_decision() {
    use crate::push_trigger_log::{Decision, Outcome as Ending, trigger_line};
    let decision = |ending: Ending, reason: &'static str, detail: &'static str, branch: Option<&'static str>| {
        return Decision { common_dir: None, origin: None, command: "merge", outcome: ending, reason, detail, branch };
    };
    let line: String = trigger_line(&decision(Ending::Declined, "upstream-equals-tip", "a \"quoted\"\nname", Some("refs/heads/main")));
    assert_eq!(line.matches('\n').count(), 1, "one line: {line:?}");
    assert!(line.ends_with("}\n"));
    for part in ["\"schemaVersion\":1", "\"command\":\"merge\"", "\"outcome\":\"declined\"", "\"reason\":\"upstream-equals-tip\"", "\"branch\":\"refs/heads/main\"", "\\\"quoted\\\"\\n"] {
        assert!(line.contains(part), "{part} in {line}");
    }
    assert!(!trigger_line(&decision(Ending::HandedOver, "ahead-of-upstream", "", None)).contains("\"branch\""));
    assert!(trigger_line(&decision(Ending::HandedOver, "ahead-of-upstream", "", None)).contains("\"outcome\":\"handed-over\""));
}

//! What:
//!  What a commit does when the finished shadow it was handed goes between its hand-out and its last write,
//!  through the built executable and the real service:
//!  a tree that goes after the commit's own question of it and before its first write,
//!  and one that goes after that write.
//! Why:
//!  The tree is the wrapper's own cache,
//!  so nothing another process of the repository does to it may end a commit with a nonzero status
//!  (`SPEC.md`,
//!  "Merges and commits never fail").
//!  A tree that goes before the first write is a tree the commit has written nothing of its own into,
//!  so it takes the pool's member or makes its own tree and lands;
//!  a tree that goes after that write is one the commit is already done with,
//!  because the request carries the change and the landing reads the commit's own scratch tree.
//!  A tree that goes while Git builds in it is the wrapper's own cache too,
//!  and so is one that goes after Git has built the commit and before it is read back out of the tree,
//!  so the commit builds once more in another tree and lands, and its record says so.
//!  The first two instants are the wrapper's own phase markers (`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`),
//!  and the third is a `pre-commit` hook of the fixture's own repository,
//!  which is linked into the shadow and holds Git inside the build while the tree goes.
//!  Nothing here is a stand-in for the wrapper.
//!
//!  A removal is not one step: a process killed between its steps leaves the tree's root with some of its entries gone,
//!  and a commit that built in it would fail. Each of the three points where the build reads the tree is controlled with
//!  that state too, so the recovery question is "does the tree still hold what the build was given" and not "is there a
//!  directory at that path".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pauseAt('shadow-handed'); await rm(handed, { recursive: true }); release(); expect(await commit()).toLand();
//! pauseAt('shadow-readable'); await rm(handed, { recursive: true }); release(); expect(await commit()).toLand();
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, Observed, executable, fixture, git, observe, repository, stderr_of, start_fixture_service};
use git_policy_cli::commit_timing::TIMING_FILE;
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use git_policy_cli::wrapper_names_old_and_new::SHADOW_DRAWN_FOLDER;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The longest a control waits for the paused wrapper to arrive,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with the service that serves it,
/// an edit the next commit names,
/// and where the paused wrapper leaves its markers.
struct Case {
    fixture: Fixture,
    repo: PathBuf,
    service: FixtureService,
    markers: PathBuf,
}

/// A repository with one commit on `main`,
/// and a file the next commit changes.
fn case(name: &str) -> Case {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["config", "commit.verbose", "false"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("markers");
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Case { fixture, repo, service, markers };
}

/// The drawn trees the worktree's drawn folder holds now.
fn drawn_trees(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else { return Vec::new() };
    return entries
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("drawn-"); }); })
        .collect();
}

/// The command that commits the next edit,
/// under the service's runtime and with the marker variable armed for `phase`.
fn a_commit_paused_at(case: &Case, phase: &str, message: &str) -> Command {
    let mut command: Command = wrapped_commit(&case.fixture, case.repo.as_path(), &["commit", "--quiet", &format!("--message={message}"), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &case.service.runtime);
    command.env(PHASE_SIGNAL_VARIABLE, format!("{phase}:pause:{}", case.markers.display()));
    return command;
}

/// What:
///  A repository whose service drew the next commit's tree,
///  with the edit that commit names,
///  and where the drawn tree is.
/// Why:
///   The draw happens beside the landing that answers a commit,
///  so a control needs one commit made before the one it means to hold.
fn a_repository_with_a_drawn_tree(name: &str) -> (Case, PathBuf) {
    let case: Case = case(name);
    std::fs::write(case.repo.join("a.txt"), "base\none\n").expect("the first edit");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "a.txt"]);
    let mut first: Command = wrapped_commit(&case.fixture, case.repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    first.env("XDG_RUNTIME_DIR", &case.service.runtime);
    let landed: Observed = observe(&mut first, b"");
    assert_eq!(landed.code, Some(0), "the commit that makes the service draw: {}", stderr_of(&landed));
    let folder: PathBuf = case.repo.join(".git").join(SHADOW_DRAWN_FOLDER).join("main");
    let began: Instant = Instant::now();
    let mut drawn: Vec<PathBuf> = drawn_trees(folder.as_path());
    while drawn.is_empty() {
        assert!(began.elapsed() < PATIENCE, "the service drew the next commit's tree ahead of it");
        std::thread::sleep(Duration::from_millis(10));
        drawn = drawn_trees(folder.as_path());
    }
    assert_eq!(drawn.len(), 1, "a stock of one is what the draw keeps: {drawn:?}");
    assert!(
        the_service_can_hand_a_tree_out(&case),
        "this control did not run: the fixture's service listens on an abstract socket (its runtime directory is too long for a \
         socket path), and a commit asks an abstract service for no drawn tree (`service_client_shadow_hand_out.rs`), so no \
         tree is handed and this control's scenario cannot happen here"
    );
    let handed: PathBuf = drawn.remove(0);
    std::fs::write(case.repo.join("a.txt"), "base\none\ntwo\n").expect("the second edit");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "a.txt"]);
    return (case, handed);
}

/// What:
///  Whether this fixture's service listens at a named socket,
///  which is the only shape in which a commit is handed a drawn tree.
/// Why:
///   A service on an abstract socket hands nothing (`service_client_shadow_hand_out.rs`),
///  so a control of the handed-tree loss would fail for want of a tree or,
///  worse, pass without the loss it names.
///  The fixture's runtime directory is an alias short enough for a socket path
///  (`short_runtime_directory`), so this is a premise a run has and not a hope,
///  and a run that cannot meet it says so instead of proving nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const canHandOut = (c) => socketAddressOf(c.service.runtime, instanceOf(c.repo + '/.git')) instanceof Path
/// ```
fn the_service_can_hand_a_tree_out(case: &Case) -> bool {
    let git_dir: PathBuf = case.repo.join(".git");
    let instance: String = git_policy_cli::service_instance::instance_of(git_dir.as_path());
    return matches!(
        git_policy_cli::service_address_of_a_worktree::socket_address_of(case.service.runtime.as_path(), instance.as_str()),
        git_policy_cli::service_server_socket_binding::Bind::Path(_named)
    );
}

/// What:
///  Whether the commit wrote its own ref store into `tree`,
///  which is what a commit that builds in a handed tree does and a commit that left the tree stocked never does.
/// Why:
///   A control that removes "the tree the commit is building in" must know that this is that tree,
///  and a tree in the drawn folder is only a candidate until the commit's ref store is in it
///  (`shadow_ref_write.rs`, `write_shadow_refs`).
///  This is what makes the difference between a control that exercises the hand-out and one that passes for nothing
///  when no tree was handed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const builtIn = (tree) => existsSync(join(tree, 'packed-refs'))
/// ```
fn the_commit_built_in(tree: &Path) -> bool {
    return tree.join("packed-refs").is_file();
}

/// What:
///  A commit that is handed a drawn tree and loses it after its own question of it,
///  and before its first write into it,
///  lands, in a tree of its own or one from the pool.
/// Why:
///   The tree the service answered with is the wrapper's own cache,
///  and a tree that goes at that instant is not something the person did or can fix.
///  Nothing has been sent to the service yet and what was written went into the tree the command abandons,
///  so the commit takes the pool's member or makes its own tree and lands as though it had never been handed one.
#[test]
fn a_commit_that_loses_its_handed_tree_after_its_question_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-taken-away");
    let mut command: Command = a_commit_paused_at(&case, "shadow-handed", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-handed-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(
            began.elapsed() < PATIENCE,
            "the wrapper reached the instant between its question of the handed tree and its first write into it"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    // The `shadow-handed` marker is reached only in the arm that holds a tree the service handed over, so the wait above is
    //  this control's proof that a tree was handed, and no assertion about the tree's own content is needed here.
    // The tree goes, as another program of the repository's removal would take it.
    std::fs::remove_dir_all(handed.as_path()).expect("the handed tree goes");
    std::fs::write(markers.join("shadow-handed-1.release"), b"").expect("release the paused command");
    let seen: Observed = waiting.join().expect("the paused command");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree went must still land: {words}");
    assert!(!words.contains("packed-refs"), "nothing about the wrapper's own cache reaches the person: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(
        subjects.lines().collect::<Vec<&str>>(),
        vec!["two", "one", "base", "initial"],
        "the commit that lost its tree is on the branch, above the one the draw followed"
    );
    remove_settled(&case.fixture);
}

/// What:
///  A commit whose handed tree goes after the last write into it still lands.
/// Why:
///   After that write the commit is done with the tree:
///  its request carries the change and the landing reads the commit's own scratch tree,
///  so the tree going is a removal the commit never notices,
///  and the service's own removal of that tree then finds nothing to remove.
#[test]
fn a_commit_whose_handed_tree_goes_after_its_last_write_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-taken-away-after");
    let mut command: Command = a_commit_paused_at(&case, "service-prepared", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("service-prepared-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(
            began.elapsed() < PATIENCE,
            "the wrapper reached the instant after its last write into the handed tree and before it sent the commit"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        the_commit_built_in(handed.as_path()),
        "the commit wrote its ref store into the tree it was handed, so this control is exercising a handed tree"
    );
    std::fs::remove_dir_all(handed.as_path()).expect("the handed tree goes");
    std::fs::write(markers.join("service-prepared-1.release"), b"").expect("release the paused command");
    let seen: Observed = waiting.join().expect("the paused command");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree went after its last write still lands: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(
        subjects.lines().collect::<Vec<&str>>(),
        vec!["two", "one", "base", "initial"],
        "the commit that lost its tree is on the branch, above the one the draw followed"
    );
    remove_settled(&case.fixture);
}

/// The last timing record the repository wrote,
/// which a commit that built again says what it did in.
fn last_record(repo: &Path) -> serde_json::Value {
    let path: PathBuf = repo.join(".git").join(TIMING_FILE);
    let text: String = std::fs::read_to_string(path.as_path()).expect("the commit's timing record");
    let last: &str = text.lines().filter(|line| { return !line.trim().is_empty(); }).next_back().expect("one record");
    return serde_json::from_str(last).expect("a record of one commit");
}

/// What:
///  A commit whose handed tree goes while Git builds in it lands, in a tree of its own or one from the pool,
///  and its record says what it did.
/// Why:
///  The tree is the wrapper's own cache and a drive can fill or a person can remove one,
///  so nothing of that may end a commit with a nonzero status (`SPEC.md`, "Merges and commits never fail").
///  The commit is held at the wrapper's own marker for the instant its shadow is built and Git has not yet run in it,
///  the tree goes there, and the build that follows fails on a tree that is gone;
///  that failure is answered with one more build in another tree (`service_native.rs`, `build_with_git`),
///  and the commit's timing record names the stretch rather than the terminal.
#[test]
fn a_commit_whose_handed_tree_goes_while_git_builds_in_it_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-taken-during-the-build");
    // The marker of the second build is released before it is reached, so only the first build is held.
    std::fs::write(case.markers.join("shadow-built-2.release"), b"").expect("the release of a second build");
    let mut command: Command = a_commit_paused_at(&case, "shadow-built", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-built-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the instant its shadow is built and Git is about to run in it");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(handed.exists(), "the tree the commit was handed is the tree it is about to build in");
    // The tree goes, as a person's removal or a full drive takes one: no sweep and no claim is involved.
    assert!(
        the_commit_built_in(handed.as_path()),
        "the commit wrote its ref store into the tree it was handed, so this control is exercising a handed tree"
    );
    std::fs::remove_dir_all(handed.as_path()).expect("the handed tree goes");
    std::fs::write(markers.join("shadow-built-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose shadow went under its build must still land: {words}");
    assert!(!words.contains("not a git repository"), "nothing of the wrapper's own cache reaches the person: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(subjects.lines().collect::<Vec<&str>>(), vec!["two", "one", "base", "initial"], "the commit is on the branch");
    let record: serde_json::Value = last_record(case.repo.as_path());
    let parts: &serde_json::Map<String, serde_json::Value> =
        record["buildParts"].as_object().unwrap_or_else(|| { panic!("the record has no build parts: {record}") });
    assert!(parts.contains_key("shadow went"), "the record says the shadow went and the commit built again: {record}");
    remove_settled(&case.fixture);
}

/// What:
///  A commit whose handed tree goes after Git has built its commit and before that commit is read back out of the tree
///  lands, in a tree of its own or one from the pool,
///  and its record says what it did.
/// Why:
///  Git wrote the commit and the ref that names it into the tree,
///  so a tree that goes there takes the only name of the commit Git made:
///  the object is in the real repository and nothing points at it,
///  and a read-back of the gone tree can only answer with its failure (`service_native.rs`, `build_with_git`).
///  The wrapper makes the commit once more in another tree, which asks the same command line again, and lands.
///  The commit is held at the wrapper's own marker for that instant, so the loss is placed exactly there.
#[test]
fn a_commit_whose_handed_tree_goes_after_git_built_it_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-taken-after-git-built-it");
    // The marker of the second attempt is released before it is reached, so only the first attempt is held.
    std::fs::write(case.markers.join("shadow-readable-2.release"), b"").expect("the release of a second attempt");
    let mut command: Command = a_commit_paused_at(&case, "shadow-readable", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-readable-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(
            began.elapsed() < PATIENCE,
            "the wrapper reached the instant Git had built its commit in the handed tree and the commit was not yet read back"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(the_commit_built_in(handed.as_path()), "the commit wrote its ref store into the tree it was handed, so this control is exercising a handed tree");
    // The tree goes, as a person's removal or a full drive takes one: no sweep and no claim is involved.
    std::fs::remove_dir_all(handed.as_path()).expect("the handed tree goes");
    std::fs::write(markers.join("shadow-readable-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose shadow went before its read-back must still land: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(subjects.lines().collect::<Vec<&str>>(), vec!["two", "one", "base", "initial"], "the commit is on the branch");
    let record: serde_json::Value = last_record(case.repo.as_path());
    let parts: &serde_json::Map<String, serde_json::Value> =
        record["buildParts"].as_object().unwrap_or_else(|| { panic!("the record has no build parts: {record}") });
    assert!(parts.contains_key("shadow went"), "the record says the shadow went and the commit built again: {record}");
    assert_eq!(record["attempts"], serde_json::Value::from(1), "the attempt count is the commit's own and the rebuild is inside it: {record}");
    remove_settled(&case.fixture);
}

/// What:
///  A `GIT_EDITOR` that takes `HEAD` out of the tree Git is building in, once, and lets the commit go on.
/// Why:
///  The build's own Git is the only thing running while the tree is read and written,
///  so an editor that removes one entry of it leaves a tree that is partly there *after* the recovery's first question
///  had seen it whole:
///  the question the recovery asks is the one this control is about, and Git's own failure follows from the tree
///  (`fatal: not a git repository`).
///  It removes once and never again, so the commit's second build is not spoiled the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const editor = (tree, markers) => ifExists(join(markers, 'editor-ran')) ? exit(0) : (unlink(join(tree, 'HEAD')), exit(0))
/// ```
fn an_editor_that_removes_head_once(case: &Case, tree: &Path) -> PathBuf {
    let ran: PathBuf = case.markers.join("editor-ran");
    let editor: PathBuf = case.markers.join("an-editor-that-removes-head");
    executable(
        editor.as_path(),
        format!(
            "#!/bin/sh\n\
             if [ -e '{ran}' ]; then exit 0; fi\n\
             printf ran > '{ran}'\n\
             rm --force '{head}'\n\
             exit 0\n",
            ran = ran.display(),
            head = tree.join("HEAD").display()
        )
        .as_bytes(),
    );
    return editor;
}

/// What:
///  A commit whose handed tree lost `HEAD` before Git was asked to build in it lands in another tree,
///  and its record says so.
/// Why:
///  A removal killed after unlinking `HEAD` leaves the root directory and the rest of the tree,
///  so a question that asks only whether the root is a directory answers yes about a tree Git cannot use.
///  The build asks whether the tree still holds what it was given, and a tree that has lost an entry is one the commit
///  builds in another tree rather than fails in (`service_native.rs`, `build_with_git`;
///  `shadow_create.rs`, `the_shadow_holds_what_a_build_needs`).
#[test]
fn a_commit_whose_handed_tree_lost_head_before_git_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-lost-head-before-git");
    std::fs::write(case.markers.join("shadow-built-2.release"), b"").expect("the release of a second build");
    let mut command: Command = a_commit_paused_at(&case, "shadow-built", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-built-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the instant its shadow is built and Git is about to run in it");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(the_commit_built_in(handed.as_path()), "the commit wrote its ref store into the tree it was handed, so this control is exercising a handed tree");
    // A removal killed after unlinking HEAD and before its last rmdir leaves exactly this.
    std::fs::remove_file(handed.join("HEAD")).expect("HEAD goes");
    assert!(handed.is_dir(), "the root of the tree is still a directory, which is all a root-only question sees");
    std::fs::write(markers.join("shadow-built-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree lost HEAD before Git must still land: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(subjects.lines().collect::<Vec<&str>>(), vec!["two", "one", "base", "initial"], "the commit is on the branch");
    let record: serde_json::Value = last_record(case.repo.as_path());
    let parts: &serde_json::Map<String, serde_json::Value> =
        record["buildParts"].as_object().unwrap_or_else(|| { panic!("the record has no build parts: {record}") });
    assert!(parts.contains_key("shadow went"), "the record says the shadow went and the commit built again: {record}");
    remove_settled(&case.fixture);
}

/// What:
///  A commit whose handed tree loses `HEAD` while Git builds in it lands in another tree,
///  and its record says so.
/// Why:
///  Git fails on a tree that has lost an entry, and the recovery after a failed build asks whether the tree still holds
///  what the build was given: a tree that has lost one is one the commit builds in again
///  (`service_native.rs`, `build_with_git`).
#[test]
fn a_commit_whose_handed_tree_loses_head_while_git_builds_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-loses-head-while-git-builds");
    let editor: PathBuf = an_editor_that_removes_head_once(&case, handed.as_path());
    let mut command: Command = wrapped_commit(&case.fixture, case.repo.as_path(), &["commit", "--edit", "--message=two", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &case.service.runtime);
    command.env("GIT_EDITOR", &editor);
    let seen: Observed = observe(&mut command, b"");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree lost HEAD while Git built must still land: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(subjects.lines().collect::<Vec<&str>>(), vec!["two", "one", "base", "initial"], "the commit is on the branch");
    let record: serde_json::Value = last_record(case.repo.as_path());
    let parts: &serde_json::Map<String, serde_json::Value> =
        record["buildParts"].as_object().unwrap_or_else(|| { panic!("the record has no build parts: {record}") });
    assert!(parts.contains_key("shadow went"), "the record says the shadow went and the commit built again: {record}");
    remove_settled(&case.fixture);
}

/// What:
///  A commit whose handed tree lost `HEAD` after Git succeeded and before the read-back lands in another tree.
/// Why:
///  The ref that names Git's commit lives in the tree, so a tree that lost an entry cannot be read back;
///  the recovery asks whether the tree still holds what the build was given and builds again
///  (`service_native.rs`, `build_with_git`).
#[test]
fn a_commit_whose_handed_tree_lost_head_before_the_read_back_still_lands() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-lost-head-before-read-back");
    std::fs::write(case.markers.join("shadow-readable-2.release"), b"").expect("the release of a second attempt");
    let mut command: Command = a_commit_paused_at(&case, "shadow-readable", "two");
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-readable-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the instant Git had built its commit in the handed tree");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(the_commit_built_in(handed.as_path()), "the commit wrote its ref store into the tree it was handed, so this control is exercising a handed tree");
    std::fs::remove_file(handed.join("HEAD")).expect("HEAD goes");
    assert!(handed.is_dir(), "the root of the tree is still a directory, which is all a root-only question sees");
    std::fs::write(markers.join("shadow-readable-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "a commit whose handed tree lost HEAD before its read-back must still land: {words}");
    let subjects: String = git_text(&case.fixture, case.repo.as_path(), &["log", "--format=%s"]);
    assert_eq!(subjects.lines().collect::<Vec<&str>>(), vec!["two", "one", "base", "initial"], "the commit is on the branch");
    let record: serde_json::Value = last_record(case.repo.as_path());
    let parts: &serde_json::Map<String, serde_json::Value> =
        record["buildParts"].as_object().unwrap_or_else(|| { panic!("the record has no build parts: {record}") });
    assert!(parts.contains_key("shadow went"), "the record says the shadow went and the commit built again: {record}");
    remove_settled(&case.fixture);
}

/// What:
///  A client killed after its hand-out and before its request leaves no tree the service keeps.
/// Why:
///  The service's claim on a tree it handed out is the half of the hand-over the client cannot cover:
///  it must last until the commit's own claim exists, and no longer.
///  A client killed before it asks for a landing never sends the request that would let the claim go at the landing,
///  so the claim's life is tied to the connection the answer came on, which the kernel closes when the client dies
///  (`service_server_connection_handling.rs`, `handle_connection`).
///  Nothing else ends it: the constant `A_DRAWN_TREE_IS_KEPT_FOR` is the owner's unanswered question.
#[test]
fn a_client_killed_before_its_request_leaves_no_claimed_tree() {
    let (case, handed): (Case, PathBuf) = a_repository_with_a_drawn_tree("handed-tree-of-a-killed-client");
    let mut command: Command = a_commit_paused_at(&case, "shadow-built", "two");
    let mut client: std::process::Child = command.spawn().expect("the wrapped commit");
    let reached: PathBuf = case.markers.join("shadow-built-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper built in the tree the service handed it");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(the_commit_built_in(handed.as_path()), "the commit wrote its ref store into the tree it was handed, so this control is exercising a handed tree");
    // The client dies at this instruction: no request is sent, and its connection to the service closes with it.
    client.kill().expect("kill the paused client");
    let _ended: std::io::Result<std::process::ExitStatus> = client.wait();
    let waited: Instant = Instant::now();
    while git_policy_cli::tree_held_by_a_live_process::whether_held(handed.as_path()) != git_policy_cli::tree_held_by_a_live_process::WhetherHeld::ByNobody {
        assert!(
            waited.elapsed() < PATIENCE,
            "the service let go of the tree whose client died before its request: {:?}",
            git_policy_cli::tree_held_by_a_live_process::whether_held(handed.as_path())
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // Nobody holds it now, so a sweep takes it once it is past the bound: the tree is not kept for the service's life.
    let old: std::time::SystemTime = std::time::SystemTime::now() - (git_policy_cli::service_scratch_abandonment::SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    std::fs::File::open(handed.as_path()).expect("the handed tree").set_modified(old).expect("age it past the bound");
    let common_dir: PathBuf = case.repo.join(".git");
    let swept: Vec<PathBuf> = git_policy_cli::shadow_drawn::sweep_drawn_of(common_dir.as_path(), common_dir.as_path(), std::time::SystemTime::now());
    assert_eq!(swept, vec![handed.clone()], "a sweep takes the tree of a client that died before its request");
    assert!(!handed.exists(), "and the tree is gone");
    remove_settled(&case.fixture);
}

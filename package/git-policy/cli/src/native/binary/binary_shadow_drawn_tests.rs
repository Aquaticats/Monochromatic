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

/// The one scratch tree a paused commit made,
/// or a panic naming what is there instead.
/// Why:
///  A commit's scratch tree carries the name of the attempt that made it, so a control finds it by its prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scratchTree = (repo) => readdirSync(join(repo, '.git/monochromatic-git-policy-cli-scratch')).filter(n => n.startsWith('monochromatic-git-policy-cli-service-'))
/// ```
fn the_scratch_tree_of_the_commit(repo: &Path) -> PathBuf {
    let folder: PathBuf = repo.join(".git").join("monochromatic-git-policy-cli-scratch");
    let entries: Vec<PathBuf> = std::fs::read_dir(folder.as_path())
        .expect("the repository scratch folder")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("monochromatic-git-policy-cli-service-"); }); })
        .collect();
    assert_eq!(entries.len(), 1, "one commit attempt has one scratch tree: {entries:?}");
    return entries.into_iter().next().expect("the tree");
}

/// What:
///  The subject and parents of `HEAD`,
///  as `git log` prints them.
/// Why:
///  What a merge became is read off the commit itself: a one-parent commit is a merge that lost its other side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = (repo) => git(repo, ['rev-list', '--parents', '-n', '1', 'HEAD']).trim().split(' ')
/// ```
fn head_and_its_parents(case: &Case) -> Vec<String> {
    let line: String = super::commit::git_text(&case.fixture, case.repo.as_path(), &["rev-list", "--parents", "-n", "1", "HEAD"]);
    return line.split_whitespace().map(String::from).skip(1).collect();
}

/// What:
///  A merge conclusion prepared in `repo` that loses only its shadow's `MERGE_HEAD` at the build pause
///  lands with both of its parents.
/// Why:
///  Git's commit names its parents from the conclusion state copied into the shadow, so a shadow without
///  `MERGE_HEAD` makes a one-parent commit that says nothing;
///  the conclusion cleanup then reads the shadow's absent file as proof that Git concluded the merge and removes
///  the real merge state.
///  The commit is held against the parents the command began with (`service_native.rs`, `build_with_git`).
#[test]
fn a_merge_whose_shadow_loses_merge_head_lands_with_every_parent() {
    let case: Case = case("merge-loses-shadow-merge-head");
    git(&case.fixture, case.repo.as_path(), &["config", "commit.verbose", "false"]);
    // A side branch whose commit the merge must name as its second parent.
    git(&case.fixture, case.repo.as_path(), &["checkout", "--quiet", "-b", "side-control"]);
    std::fs::write(case.repo.join("side.txt"), b"side\n").expect("side");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "side.txt"]);
    git(&case.fixture, case.repo.as_path(), &["commit", "--quiet", "--message=side"]);
    let side: String = super::commit::git_text(&case.fixture, case.repo.as_path(), &["rev-parse", "side-control"]).trim().to_string();
    git(&case.fixture, case.repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(case.repo.join("main.txt"), b"main\n").expect("main");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "main.txt"]);
    git(&case.fixture, case.repo.as_path(), &["commit", "--quiet", "--message=main"]);
    git(&case.fixture, case.repo.as_path(), &["merge", "--no-commit", "--no-ff", "side-control"]);
    assert!(case.repo.join(".git").join("MERGE_HEAD").is_file(), "the merge is in progress");
    // The build that answers the loss reaches the pause again, and this control holds only the first attempt.
    std::fs::write(case.markers.join("shadow-built-2.release"), b"").expect("a second attempt may go on");
    let mut command: Command = a_commit_built_by_git_paused_at(&case, "shadow-built", &["commit", "--quiet", "--message=merge"]);
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-built-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the instant its shadow is built");
        std::thread::sleep(Duration::from_millis(5));
    }
    let shadow_merge_head: PathBuf = the_scratch_tree_of_the_commit(case.repo.as_path()).join("shadow").join("MERGE_HEAD");
    assert!(shadow_merge_head.is_file(), "the shadow holds the conclusion state the merge needs");
    std::fs::remove_file(shadow_merge_head.as_path()).expect("the shadow's MERGE_HEAD goes");
    std::fs::write(markers.join("shadow-built-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "the merge's commit must land: {words}");
    let parents: Vec<String> = head_and_its_parents(&case);
    assert_eq!(parents.len(), 2, "the commit names both of a merge's parents: {parents:?}");
    assert!(parents.contains(&side), "the side commit is one of them: {parents:?} side={side}");
    assert!(!case.repo.join(".git").join("MERGE_HEAD").exists(), "and the real merge state went with the conclusion it was kept for");
    remove_settled(&case.fixture);
}

/// The commit `arguments` name, held at `phase`.
/// Why:
///  A commit that names its paths is the one Git builds in the shadow,
///  so the build phase and every file it reads are reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const command = wrappedCommit(fixture, repo, ['commit', '--quiet', '--file=message.txt']); command.env('CLI_GIT_TEST_ONLY_PHASE_SIGNAL', `${phase}:pause:${markers}`)
/// ```
fn a_commit_built_by_git_paused_at(case: &Case, phase: &str, arguments: &[&str]) -> Command {
    let mut command: Command = wrapped_commit(&case.fixture, case.repo.as_path(), arguments);
    command.env("XDG_RUNTIME_DIR", &case.service.runtime);
    command.env(PHASE_SIGNAL_VARIABLE, format!("{phase}:pause:{}", case.markers.display()));
    return command;
}

/// What:
///  A commit whose private index goes before Git reads it is still the commit that was asked for.
/// Why:
///  Git is given the private index as `GIT_INDEX_FILE`,
///  and an index that is not there reads to Git as an empty one,
///  so the commit becomes the empty tree and deletes every tracked file from history while the worktree keeps them.
///  The index this command was given is kept apart from the one Git reads and the commit is held against it
///  (`service_native.rs`, `build_with_git`).
#[test]
fn a_commit_whose_private_index_goes_is_still_the_commit_that_was_asked_for() {
    let case: Case = case("index-goes-before-the-build");
    git(&case.fixture, case.repo.as_path(), &["config", "commit.verbose", "false"]);
    std::fs::write(case.repo.join("base.txt"), b"base\n").expect("base");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "base.txt"]);
    git(&case.fixture, case.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(case.repo.join("base.txt"), b"base\nmore\n").expect("the edit");
    std::fs::write(case.repo.join("added.txt"), b"added\n").expect("the addition");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "base.txt", "added.txt"]);
    // The build that answers the loss reaches the pause again, and this control holds only the first attempt.
    std::fs::write(case.markers.join("shadow-built-2.release"), b"").expect("a second attempt may go on");
    let mut command: Command = a_commit_built_by_git_paused_at(&case, "shadow-built", &["commit", "--quiet", "--message=two", "--", "base.txt", "added.txt"]);
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-built-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the instant its shadow is built");
        std::thread::sleep(Duration::from_millis(5));
    }
    let index: PathBuf = the_scratch_tree_of_the_commit(case.repo.as_path()).join("commit.index");
    assert!(index.is_file(), "the private index is there before the loss");
    std::fs::remove_file(index.as_path()).expect("the private index goes");
    std::fs::write(markers.join("shadow-built-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "the commit must land: {words}");
    let files: String = super::commit::git_text(&case.fixture, case.repo.as_path(), &["ls-tree", "-r", "--name-only", "HEAD"]);
    let listed: Vec<&str> = files.lines().map(str::trim).filter(|line| { return !line.is_empty(); }).collect();
    assert!(listed.contains(&"base.txt"), "the commit keeps the file the person had tracked: {listed:?}");
    assert!(listed.contains(&"added.txt"), "and the one this commit added: {listed:?}");
    remove_settled(&case.fixture);
}

/// What:
///  A reftable shadow whose tables go is built in another tree, and the commit lands.
/// Why:
///  A reftable shadow keeps its refs in Git's own tables,
///  and a whole-shadow question that looks at no table answers for a tree Git refuses every transaction in
///  (`fatal: reftable: transaction prepare: I/O error`);
///  the list of those tables is what the question asks for (`shadow_create.rs`, `the_shadow_holds_what_a_creation_makes`).
#[test]
fn a_reftable_shadow_that_loses_its_tables_is_built_again() {
    let mut case: Case = case("reftable-loses-its-tables");
    let reftable: PathBuf = case.fixture.root.join("reftable");
    std::fs::create_dir(reftable.as_path()).expect("the repository directory");
    git(&case.fixture, reftable.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(&case.fixture, reftable.as_path(), &["config", "commit.verbose", "false"]);
    git(&case.fixture, reftable.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    std::fs::write(reftable.join("a.txt"), b"one\n").expect("a");
    git(&case.fixture, reftable.as_path(), &["add", "--", "a.txt"]);
    let base: String = super::commit::git_text(&case.fixture, reftable.as_path(), &["rev-parse", "HEAD"]);
    case.service.serve(&case.fixture, reftable.join(".git").as_path());
    case.repo = reftable;
    // The build that answers the loss reaches the pause again, and this control holds only the first attempt.
    std::fs::write(case.markers.join("shadow-built-2.release"), b"").expect("a second attempt may go on");
    let mut command: Command = a_commit_built_by_git_paused_at(&case, "shadow-built", &["commit", "--quiet", "--message=two", "--", "a.txt"]);
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join("shadow-built-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the instant its reftable shadow is built");
        std::thread::sleep(Duration::from_millis(5));
    }
    let tables: PathBuf = the_scratch_tree_of_the_commit(case.repo.as_path()).join("shadow").join("reftable");
    assert!(tables.is_dir(), "the shadow holds Git's tables before the loss");
    std::fs::remove_dir_all(tables.as_path()).expect("the tables go");
    std::fs::write(markers.join("shadow-built-1.release"), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let words: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "the commit must land in another tree: {words}");
    assert_ne!(super::commit::git_text(&case.fixture, case.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch moved");
    remove_settled(&case.fixture);
}

/// One row of the loss table:
/// the entry taken away under the commit's scratch tree,
/// and what the command did.
/// Why:
///  A table is what a reader needs to see that no loss of one file or folder of the scratch tree,
///  at either pause, ends in a commit the person did not write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LossRow = { entry: string; exit: number | null; files: string[]; rebuilt: string | null };
/// ```
#[derive(Debug)]
struct LossRow {
    /// The entry's name under the scratch tree.
    entry: String,
    /// What the command exited with.
    exit: Option<i32>,
    /// The files the commit it made holds,
    /// or nothing when it made none.
    files: Vec<String>,
    /// The stretch its record names when it built again,
    /// or nothing.
    rebuilt: Option<String>,
}

/// What:
///  Takes one entry of the commit's scratch tree away at `phase` and answers what the command did.
/// Why:
///  It is one row of the loss table,
///  and every row starts from a fixture of its own so no row depends on another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneLoss = async (name, phase, entry) => { ...; unlink(join(scratch, entry)); release(); return row }
/// ```
fn one_loss(name: &str, phase: &str, entry: &str) -> LossRow {
    let case: Case = case(name);
    git(&case.fixture, case.repo.as_path(), &["config", "commit.verbose", "false"]);
    std::fs::write(case.repo.join("base.txt"), b"base\n").expect("base");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "base.txt"]);
    git(&case.fixture, case.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(case.repo.join("base.txt"), b"base\nmore\n").expect("the edit");
    std::fs::write(case.repo.join("added.txt"), b"added\n").expect("the addition");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "base.txt", "added.txt"]);
    std::fs::write(case.markers.join(format!("{phase}-2.release")), b"").expect("a second attempt may go on");
    let mut command: Command = a_commit_built_by_git_paused_at(&case, phase, &["commit", "--quiet", "--message=two", "--", "base.txt", "added.txt"]);
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join(format!("{phase}-1.reached"));
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the {phase} pause for the row {entry}");
        std::thread::sleep(Duration::from_millis(5));
    }
    let scratch: PathBuf = the_scratch_tree_of_the_commit(case.repo.as_path());
    let target: PathBuf = scratch.join(entry);
    let present: Vec<String> = std::fs::read_dir(scratch.as_path())
        .expect("the scratch tree")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    println!("reached={phase}:{entry} entries={present:?}");
    let removed: std::io::Result<()> = match std::fs::symlink_metadata(target.as_path()) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(target.as_path()),
        Ok(_) => std::fs::remove_file(target.as_path()),
        Err(error) => Err(error),
    };
    assert!(removed.is_ok(), "the row {entry} takes its entry away: {removed:?}");
    std::fs::write(markers.join(format!("{phase}-1.release")), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    let files: Vec<String> = match seen.code {
        Some(0) => super::commit::git_text(&case.fixture, case.repo.as_path(), &["ls-tree", "-r", "--name-only", "HEAD"])
            .lines()
            .map(str::trim)
            .filter(|line| { return !line.is_empty(); })
            .map(String::from)
            .collect(),
        _other => Vec::new(),
    };
    let rebuilt: Option<String> = std::fs::read_to_string(case.repo.join(".git").join(TIMING_FILE))
        .ok()
        .and_then(|text| { return text.lines().filter(|line| { return !line.trim().is_empty(); }).next_back().map(String::from); })
        .and_then(|line| { return serde_json::from_str::<serde_json::Value>(line.as_str()).ok(); })
        .and_then(|record| {
            let parts: serde_json::Map<String, serde_json::Value> = record["buildParts"].as_object().cloned().unwrap_or_default();
            for part in ["shadow went", "commit differed"] {
                if parts.contains_key(part) {
                    return Some(String::from(part));
                }
            }
            return None;
        });
    remove_settled(&case.fixture);
    return LossRow { entry: String::from(entry), exit: seen.code, files, rebuilt };
}

/// What:
///  Every entry of the commit's scratch tree is taken away in turn at `phase`,
///  and no row lands a commit the person did not write.
/// Why:
///  A commit's scratch tree is files the wrapper made,
///  and any of them can go at any instant for a cause no sweep explains.
///  A list of files to look at before Git runs can only be longer,
///  so what the control holds is the outcome:
///  a row either lands exactly the commit the command asked for,
///  or it lands again in another tree,
///  or it fails with the person's worktree untouched,
///  and never a commit of other content and never a real merge state removed by a commit that did not conclude it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of ['commit.index', 'captured.index', 'shadow', ...]) { row = oneLoss(phase, entry); expect(row.files == expected || row.rebuilt) }
/// ```
fn the_loss_table(name: &str, phase: &str) -> Vec<LossRow> {
    let probe: Case = case(&format!("{name}-entries"));
    git(&probe.fixture, probe.repo.as_path(), &["config", "commit.verbose", "false"]);
    std::fs::write(probe.repo.join("base.txt"), b"base\n").expect("base");
    git(&probe.fixture, probe.repo.as_path(), &["add", "--", "base.txt"]);
    git(&probe.fixture, probe.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(probe.repo.join("added.txt"), b"added\n").expect("the addition");
    git(&probe.fixture, probe.repo.as_path(), &["add", "--", "added.txt"]);
    std::fs::write(probe.markers.join(format!("{phase}-2.release")), b"").expect("a second attempt may go on");
    let mut command: Command = a_commit_built_by_git_paused_at(&probe, phase, &["commit", "--quiet", "--message=probe", "--", "added.txt"]);
    let markers: PathBuf = probe.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join(format!("{phase}-1.reached"));
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the {phase} pause of the probe");
        std::thread::sleep(Duration::from_millis(5));
    }
    let scratch: PathBuf = the_scratch_tree_of_the_commit(probe.repo.as_path());
    let mut entries: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(scratch.as_path()).expect("the scratch tree").flatten() {
        entries.push(entry.file_name().to_string_lossy().into_owned());
    }
    for entry in std::fs::read_dir(scratch.join("shadow").as_path()).expect("the shadow").flatten() {
        entries.push(format!("shadow/{}", entry.file_name().to_string_lossy()));
    }
    entries.sort();
    std::fs::write(markers.join(format!("{phase}-1.release")), b"").expect("release the probe");
    let _seen: Observed = waiting.join().expect("the probe");
    remove_settled(&probe.fixture);
    let mut rows: Vec<LossRow> = Vec::new();
    for entry in entries {
        rows.push(one_loss(&format!("{name}-{}", entry.replace('/', "-")), phase, entry.as_str()));
    }
    return rows;
}

/// What:
///  Every entry of a plain commit's scratch tree can go at the build pause,
///  and no row lands a commit the person did not write.
#[test]
fn every_entry_of_the_scratch_tree_can_go_before_git_and_the_commit_is_the_one_asked_for() {
    let rows: Vec<LossRow> = the_loss_table("loss-table-before-git", "shadow-built");
    for row in &rows {
        println!("row={} exit={:?} files={:?} rebuilt={:?}", row.entry, row.exit, row.files, row.rebuilt);
    }
    assert!(!rows.is_empty(), "the table has rows");
    // The tree the command asked for: the fixture's own file, the one it changed and the one it added.
    let asked: Vec<String> = vec![String::from("a.txt"), String::from("added.txt"), String::from("base.txt")];
    for row in &rows {
        if row.exit == Some(0) {
            assert_eq!(row.files, asked, "the row {} landed the commit that was asked for", row.entry);
        }
    }
}

/// What:
///  Every entry of a plain commit's scratch tree can go after Git made the commit and before it lands,
///  and no row lands a commit the person did not write.
#[test]
fn every_entry_of_the_scratch_tree_can_go_after_git_and_the_commit_is_the_one_asked_for() {
    let rows: Vec<LossRow> = the_loss_table("loss-table-after-git", "service-prepared");
    for row in &rows {
        println!("row={} exit={:?} files={:?} rebuilt={:?}", row.entry, row.exit, row.files, row.rebuilt);
    }
    assert!(!rows.is_empty(), "the table has rows");
    // The tree the command asked for: the fixture's own file, the one it changed and the one it added.
    let asked: Vec<String> = vec![String::from("a.txt"), String::from("added.txt"), String::from("base.txt")];
    for row in &rows {
        if row.exit == Some(0) {
            assert_eq!(row.files, asked, "the row {} landed the commit that was asked for", row.entry);
        }
    }
}

/// One row of the property table:
/// an entry of a commit's scratch tree taken away at one instant of a build,
/// and what came back.
/// Why:
///  The property is one sentence,
///  and a reader needs the row that broke it: what the command exited with, the files of the commit it landed,
///  that commit's parents, and whether the branch moved at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropertyRow = { row: string; exit: number | null; files: string[]; parents: string[]; branchMoved: boolean };
/// ```
#[derive(Debug)]
struct PropertyRow {
    /// The phase and the entry, as one name a reader can find.
    row: String,
    /// What the command exited with.
    exit: Option<i32>,
    /// The files the landed commit holds,
    /// or nothing when it landed none.
    files: Vec<String>,
    /// The parents of the commit `HEAD` names after the command.
    parents: Vec<String>,
    /// Whether `HEAD` is a commit the command made.
    branch_moved: bool,
}

/// Arms the second and third arrival at every pause of a build,
/// so a command that takes its capture again or builds again goes on without a control holding it.
/// Why:
///  A capture taken again and a build made again reach their pauses again,
///  numbered by occurrence,
///  and a control that held only the first arrival would stand at a pause nothing releases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const phase of PAD) for (const n of [2, 3]) writeFileSync(join(markers, `${phase}-${n}.release`), '')
/// ```
fn arm_the_later_arrivals(markers: &Path, phase: &str) {
    for later in ["2", "3"] {
        let _armed: std::io::Result<()> = std::fs::write(markers.join(format!("{phase}-{later}.release")), b"");
        for other in ["commit-capture-taken", "commit-captured", "shadow-built", "shadow-readable"] {
            let _also: std::io::Result<()> = std::fs::write(markers.join(format!("{other}-{later}.release")), b"");
        }
    }
}

/// Where the release file of `phase`'s first pause is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const releaseOf = (markers, phase) => join(markers, `${phase}-1.release`)
/// ```
fn markers_join(markers: &Path, phase: &str) -> PathBuf {
    return markers.join(format!("{phase}-1.release"));
}

/// The commit one fixture is set up for:
/// the base commit its history starts from,
/// and the message the commit under test carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagedCase = (name) => { const c = case(name); ...; return { case: c, base } }
/// ```
fn staged_case(name: &str) -> (Case, String) {
    let case: Case = case(name);
    std::fs::write(case.repo.join("a.txt"), b"base\nmore\n").expect("the edit");
    std::fs::write(case.repo.join("added.txt"), b"added\n").expect("the addition");
    git(&case.fixture, case.repo.as_path(), &["add", "--", "a.txt", "added.txt"]);
    let base: String = super::commit::git_text(&case.fixture, case.repo.as_path(), &["rev-parse", "HEAD"]);
    return (case, base);
}

/// The entries of a commit's scratch tree that a loss can take at `phase`,
/// with the tree itself first.
/// Why:
///  Which files exist depends on the instant:
///  at the capture the tree holds the two indexes and the kept copy and nothing else,
/// and after Git has built the commit it holds the shadow, its ref store and the commit's own configuration.
///  The table is every one of them at the instant it is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entriesAt = (phase) => ['.', ...readdir(scratch), ...readdir(scratch/shadow).map(name => 'shadow/' + name)]
/// ```
fn entries_at(phase: &str) -> Vec<String> {
    let (case, _base): (Case, String) = staged_case(&format!("property-entries-{phase}"));
    arm_the_later_arrivals(case.markers.as_path(), phase);
    let mut command: Command = a_commit_built_by_git_paused_at(&case, phase, &["commit", "--quiet", "--message=two", "--", "a.txt", "added.txt"]);
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join(format!("{phase}-1.reached"));
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the {phase} pause of the probe");
        std::thread::sleep(Duration::from_millis(5));
    }
    let scratch: PathBuf = the_scratch_tree_of_the_commit(case.repo.as_path());
    let mut entries: Vec<String> = vec![String::from(".")];
    for entry in std::fs::read_dir(scratch.as_path()).expect("the scratch tree").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let path: PathBuf = entry.path();
        entries.push(name.clone());
        if name == "shadow" && path.is_dir() {
            for inner in std::fs::read_dir(path.as_path()).expect("the shadow").flatten() {
                entries.push(format!("shadow/{}", inner.file_name().to_string_lossy()));
            }
        }
    }
    std::fs::write(markers_join(&case.markers, phase), b"").expect("release the probe");
    let _seen: Observed = waiting.join().expect("the probe");
    remove_settled(&case.fixture);
    entries.sort();
    entries.dedup();
    return entries;
}

/// What:
///  Takes `entry` away at `phase` of one command's build and answers what came back.
/// Why:
///   It is one row of the property table,
///   and every row starts from a fixture of its own so no row depends on another.
///   The entry `.` is the commit's scratch tree itself:
///   every file of it goes at one instant, which is the case where the tree the commit was asked for cannot be read at all.
///   The release is written before anything is asserted, so a row that fails leaves no commit held at its pause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneLost = (name, phase, entry) => { ...; remove(join(scratch, entry)); release(); return row }
/// ```
fn one_lost_at(name: &str, phase: &str, entry: &str) -> PropertyRow {
    let (case, base): (Case, String) = staged_case(name);
    arm_the_later_arrivals(case.markers.as_path(), phase);
    let mut command: Command = a_commit_built_by_git_paused_at(&case, phase, &["commit", "--quiet", "--message=two", "--", "a.txt", "added.txt"]);
    let markers: PathBuf = case.markers.clone();
    let waiting: std::thread::JoinHandle<Observed> = std::thread::spawn(move || { return observe(&mut command, b""); });
    let reached: PathBuf = markers.join(format!("{phase}-1.reached"));
    let began: Instant = Instant::now();
    while !reached.exists() {
        assert!(began.elapsed() < PATIENCE, "the wrapper reached the {phase} pause for the row {entry}");
        std::thread::sleep(Duration::from_millis(5));
    }
    let scratch: PathBuf = the_scratch_tree_of_the_commit(case.repo.as_path());
    let target: PathBuf = if entry == "." { scratch.clone() } else { scratch.join(entry) };
    let removed: std::io::Result<()> = match std::fs::symlink_metadata(target.as_path()) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(target.as_path()),
        Ok(_) => std::fs::remove_file(target.as_path()),
        Err(error) => Err(error),
    };
    std::fs::write(markers.join(format!("{phase}-1.release")), b"").expect("release the held commit");
    let seen: Observed = waiting.join().expect("the commit");
    assert!(removed.is_ok(), "the row {row} takes its entry away: {removed:?}", row = format!("{phase}:{entry}"));
    let moved: Option<String> = match seen.code {
        Some(0) => Some(super::commit::git_text(&case.fixture, case.repo.as_path(), &["rev-parse", "HEAD"])),
        _other => None,
    };
    let files: Vec<String> = match moved.as_ref() {
        Some(_) => super::commit::git_text(&case.fixture, case.repo.as_path(), &["ls-tree", "-r", "--name-only", "HEAD"])
            .lines()
            .map(str::trim)
            .filter(|line| { return !line.is_empty(); })
            .map(String::from)
            .collect(),
        None => Vec::new(),
    };
    let parents: Vec<String> = match seen.code {
        Some(0) => head_and_its_parents(&case),
        _other => Vec::new(),
    };
    let branch_moved: bool = super::commit::git_text(&case.fixture, case.repo.as_path(), &["rev-parse", "HEAD"]) != base;
    remove_settled(&case.fixture);
    return PropertyRow { row: format!("{phase}:{entry}"), exit: seen.code, files, parents, branch_moved };
}

/// What:
///  No instant of a build at which a commit's scratch tree can go lets a commit the person did not write land.
/// Why:
///  The scratch tree is the wrapper's own,
///  and every file of it can go at any instruction for a cause no sweep explains.
///  What the person is owed is one of two outcomes:
///  the commit the command asked for,
///  or no commit at all,
///  and never a commit of other content and never a branch moved by one.
///  The instants are the ones a Git start reads the tree at: the capture's own read of the kept index,
///  the moment after it, the shadow built and not yet used, the commit Git built and not yet read back,
///  and the preparation that follows the read-back.
#[test]
fn no_instant_of_a_build_hands_back_a_commit_the_command_did_not_ask_for() {
    let phases: [&str; 5] = ["commit-capture-taken", "commit-captured", "shadow-built", "shadow-readable", "service-prepared"];
    let asked: Vec<String> = vec![String::from("a.txt"), String::from("added.txt")];
    let mut rows: Vec<PropertyRow> = Vec::new();
    for phase in phases {
        for entry in entries_at(phase) {
            let name: String = format!("property-{}-{}", phase, entry.replace('/', "-").replace('.', "tree"));
            let row: PropertyRow = one_lost_at(name.as_str(), phase, entry.as_str());
            println!("row={} exit={:?} files={:?} parents={:?} branch_moved={}", row.row, row.exit, row.files, row.parents, row.branch_moved);
            rows.push(row);
        }
    }
    assert!(!rows.is_empty(), "every instant has its rows");
    for row in &rows {
        match row.exit {
            Some(0) => {
                assert_eq!(row.files, asked, "the row {} landed the commit that was asked for", row.row);
                assert_eq!(row.parents.len(), 1, "the row {} landed a commit of one parent", row.row);
                assert!(row.branch_moved, "the row {} landed a commit, so the branch moved", row.row);
            }
            _other => {
                assert!(!row.branch_moved, "the row {} moved the branch without landing the commit that was asked for", row.row);
            }
        }
    }
}

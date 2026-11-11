//! What:
//!  A wrapped commit whose command line names the repository it works on:
//! `--git-dir` with `--work-tree`,
//! `GIT_DIR` with `GIT_WORK_TREE`,
//! and a relative `-C`.
//! Why:
//!  "Commits should never fail":
//! native Git commits in every layout these name,
//! so the wrapper must commit there too,
//! and it must read the repository the caller named,
//! never the one the working directory happens to be in.
//!  Real Git is the oracle:
//! each case runs the same command line in an identical fixture and compares what landed.
//!  `require-root` still refuses a commit whose effective directory is not the repository's root,
//! which is the documented policy
//! (`doc/decision/cli-git-policies-platform.md`),
//! not a reading of the command line.
//!  Every fixture repository sits inside a repository of the fixture's own,
//! so a wrapper that reads the wrong repository reads that one,
//! never the checkout the fixture lives in.
//!  A hand bump's dependents belong to the same commit
//! (`doc/decision/private-npm-registry.md:158-160`),
//! so one case commits a workspace in a named repository and reads what landed there.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::dependent_version::{CONFIGURATION, manifest, publish_set, read, write};
use super::support::{Fixture, direct, fixture, git, repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A repository whose Git directory sits outside its work tree,
/// as the dotfiles pattern has it,
/// with one tracked file at the top of the work tree,
/// one in `sub/`,
/// and an edit in the top one ready to commit.
struct Named {
    /// The bare Git directory,
    /// which Git finds only when it is named.
    git_dir: PathBuf,
    /// The work tree the Git directory is paired with.
    work_tree: PathBuf,
}

/// The two global options that name `named`,
/// in the order Git reads them.
fn options_of(named: &Named) -> Vec<String> {
    return vec![
        format!("--git-dir={}", named.git_dir.display()),
        format!("--work-tree={}", named.work_tree.display()),
    ];
}

/// What:
///  Make the repository `name` of the dotfiles pattern inside `fixture`,
/// nested in a repository of the fixture's own.
/// Why:
///  The pair names a Git directory outside any work tree,
/// which is exactly the layout a wrapper that reads the wrong repository mistakes for the one of the directory it
///  runs in:
/// nesting it in a known repository makes that mistake land in the fixture,
/// not in the checkout the fixture sits in.
fn dotfiles(fixture: &Fixture, name: &str) -> Named {
    let holder: PathBuf = repository(fixture, OsStr::new(name));
    let named: Named = Named { git_dir: holder.join("cfg.git"), work_tree: holder.join("wt") };
    std::fs::create_dir_all(named.work_tree.join("sub").as_path()).expect("the work tree");
    git(fixture, holder.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", named.git_dir.to_str().expect("a text path")]);
    std::fs::write(named.work_tree.join("a.txt"), "one\n").expect("a");
    std::fs::write(named.work_tree.join("sub/b.txt"), "other\n").expect("b");
    let mut staged: Vec<String> = options_of(&named);
    staged.extend(["add", "a.txt", "sub/b.txt"].map(str::to_string));
    git(fixture, named.work_tree.as_path(), staged.as_slice());
    let mut committed: Vec<String> = options_of(&named);
    committed.extend(["commit", "--quiet", "--message=base"].map(str::to_string));
    git(fixture, named.work_tree.as_path(), committed.as_slice());
    let mut detached: Vec<String> = options_of(&named);
    detached.extend(["config", "maintenance.autoDetach", "false"].map(str::to_string));
    git(fixture, named.work_tree.as_path(), detached.as_slice());
    std::fs::write(named.work_tree.join("a.txt"), "one\ntwo\n").expect("edit a");
    return named;
}

/// What:
///  The wrapped line for a commit in a dotfiles repository,
/// with its service running.
/// Why:
///   The fixture's service starts the instance of every worktree it finds under the fixture root,
/// and a bare Git directory is no worktree,
/// so the instance of the Git directory the line names is started here.
fn wrapped_named(fixture: &Fixture, named: &Named, words: &[&str]) -> Command {
    return wrapped_named_in(fixture, named, named.work_tree.as_path(), words);
}

/// What:
///  The wrapped line for a commit in a dotfiles repository,
///  run from `directory`.
/// Why:
///   A line that moves Git with `-C` starts somewhere else than the work tree it names,
///  and the service that lands its commit is the one of the Git directory the line names.
fn wrapped_named_in(fixture: &Fixture, named: &Named, directory: &Path, words: &[&str]) -> Command {
    let command: Command = wrapped_commit(fixture, directory, words);
    let service = fixture.service.lock().expect("the fixture's service lock");
    service.as_ref().expect("a wrapped command starts the fixture's service").serve(fixture, named.git_dir.as_path());
    return command;
}

/// What:
///  A Git query against `named` in the text of its answer.
/// Why:
///   Every read of a repository whose Git directory sits outside its work tree names it,
/// and the assertions below read the commit that landed there.
fn named_text(fixture: &Fixture, named: &Named, arguments: &[&str]) -> String {
    let mut line: Vec<String> = options_of(named);
    line.extend(arguments.iter().map(|argument| { return (*argument).to_string(); }));
    let words: Vec<&str> = line.iter().map(String::as_str).collect();
    return git_text(fixture, named.work_tree.as_path(), words.as_slice());
}

/// Make a plain repository at `directory` with one tracked file in a subdirectory,
/// and an edit in `tracked.txt` at its root.
fn plain_with_a_subdirectory(fixture: &Fixture, directory: &Path) -> PathBuf {
    let repo: PathBuf = directory.to_path_buf();
    std::fs::create_dir_all(repo.as_path()).expect("the repository's directory");
    git(fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    std::fs::create_dir(repo.join("sub").as_path()).expect("a directory of the repository");
    std::fs::write(repo.join("tracked.txt"), "base\n").expect("the tracked file");
    std::fs::write(repo.join("sub/b.txt"), "other\n").expect("a file in the subdirectory");
    git(fixture, repo.as_path(), &["add", "--", "tracked.txt", "sub/b.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("tracked.txt"), "base\nchanged\n").expect("edit the tracked file");
    return repo;
}

/// What a finished commit left on the branch:
/// its tree,
/// its whole message,
/// and how many commits the branch holds.
fn landed(fixture: &Fixture, directory: &Path, options: &[String]) -> (String, String, String) {
    let query = |arguments: &[&str]| -> String {
        let mut all: Vec<String> = options.to_vec();
        all.extend(arguments.iter().map(|argument| { return (*argument).to_string(); }));
        let output: Output = git(fixture, directory, all.as_slice());
        return String::from_utf8_lossy(output.stdout.as_slice()).trim_end().to_string();
    };
    return (query(&["rev-parse", "HEAD^{tree}"]), query(&["log", "-1", "--format=%B"]), query(&["rev-list", "--count", "HEAD"]));
}

/// What a repository holds,
/// so a commit that read the wrong one shows in it:
/// the object store's own count and the entries of its Git directory.
fn stores_of(fixture: &Fixture, repo: &Path) -> (String, Vec<String>) {
    let counted: String = git_text(fixture, repo, &["count-objects", "-v"]).trim_end().to_string();
    let mut entries: Vec<String> = std::fs::read_dir(repo.join(".git"))
        .expect("the Git directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .collect();
    entries.sort();
    return (counted, entries);
}

/// Whether the fixture's own holder repository,
/// which encloses `directory`,
/// still has its one commit.
fn holder_unchanged(fixture: &Fixture, directory: &Path) -> bool {
    return git_text(fixture, directory, &["rev-list", "--count", "HEAD"]).trim_end() == "1";
}

/// What:
///  `--git-dir` and `--work-tree` name the repository a commit works on,
/// from inside its work tree,
/// and it lands there.
/// Why:
///  "Commits should never fail":
/// native Git commits in this layout,
/// and the wrapper must make the same commit.
#[test]
fn a_commit_named_by_the_git_directory_and_work_tree_options_lands() {
    let fixture: Fixture = fixture("naming-git-dir-options");
    let named: Named = dotfiles(&fixture, "named");
    let oracle: Named = dotfiles(&fixture, "oracle");
    let mut line: Vec<String> = options_of(&named);
    line.extend(["commit", "--quiet", "--message=named repository", "--", "a.txt"].map(str::to_string));
    let words: Vec<&str> = line.iter().map(String::as_str).collect();
    let served: Output = wrapped_named(&fixture, &named, words.as_slice()).output().expect("the wrapped commit runs");
    assert!(served.status.success(), "the commit landed: {}", String::from_utf8_lossy(served.stderr.as_slice()));
    // Native Git, of the same input from the same start, is the oracle.
    let mut oracle_line: Vec<String> = options_of(&oracle);
    oracle_line.extend(["commit", "--quiet", "--message=named repository", "--", "a.txt"].map(str::to_string));
    let native: Output = direct(&fixture).current_dir(oracle.work_tree.as_path()).args(oracle_line.as_slice()).output().expect("native Git commits");
    assert!(native.status.success(), "the oracle commits: {}", String::from_utf8_lossy(native.stderr.as_slice()));
    assert_eq!(
        landed(&fixture, named.work_tree.as_path(), options_of(&named).as_slice()),
        landed(&fixture, oracle.work_tree.as_path(), options_of(&oracle).as_slice()),
        "the tree, the message and the branch's length agree with native Git"
    );
    assert!(holder_unchanged(&fixture, named.work_tree.parent().expect("the holder")), "the repository the fixture is nested in gained no commit");
    remove_settled(&fixture);
}

/// What:
///  A relative `--git-dir` and `--work-tree` are read from the directory the command runs in,
/// as Git reads them.
/// Why:
///  The wrapper resolves the pair itself to name the repository in the other processes of a commit,
/// so a relative pair is where that resolution can pick another repository.
#[test]
fn a_commit_named_by_a_relative_git_directory_and_work_tree_options_lands() {
    let fixture: Fixture = fixture("naming-git-dir-relative");
    let named: Named = dotfiles(&fixture, "named");
    let oracle: Named = dotfiles(&fixture, "oracle");
    let relative: [&str; 2] = ["--git-dir=../cfg.git", "--work-tree=."];
    let mut line: Vec<String> = relative.map(str::to_string).to_vec();
    line.extend(["commit", "--quiet", "--message=relative name", "--", "a.txt"].map(str::to_string));
    let words: Vec<&str> = line.iter().map(String::as_str).collect();
    let served: Output = wrapped_named(&fixture, &named, words.as_slice()).output().expect("the wrapped commit runs");
    assert!(served.status.success(), "the commit landed: {}", String::from_utf8_lossy(served.stderr.as_slice()));
    let mut oracle_line: Vec<String> = relative.map(str::to_string).to_vec();
    oracle_line.extend(["commit", "--quiet", "--message=relative name", "--", "a.txt"].map(str::to_string));
    let native: Output = direct(&fixture).current_dir(oracle.work_tree.as_path()).args(oracle_line.as_slice()).output().expect("native Git commits");
    assert!(native.status.success(), "the oracle commits: {}", String::from_utf8_lossy(native.stderr.as_slice()));
    let pair: Vec<String> = relative.map(str::to_string).to_vec();
    assert_eq!(
        landed(&fixture, named.work_tree.as_path(), pair.as_slice()),
        landed(&fixture, oracle.work_tree.as_path(), pair.as_slice()),
        "the tree, the message and the branch's length agree with native Git"
    );
    remove_settled(&fixture);
}

/// What:
///  `GIT_DIR` and `GIT_WORK_TREE` name the repository a commit works on,
/// and it lands there.
/// Why:
///   The resident service is another process,
/// which inherits none of the caller's variables:
/// the repository the variables name must travel with the request.
#[test]
fn a_commit_named_by_the_git_directory_and_work_tree_variables_lands() {
    let fixture: Fixture = fixture("naming-git-dir-variables");
    let named: Named = dotfiles(&fixture, "named");
    let oracle: Named = dotfiles(&fixture, "oracle");
    let served: Output = wrapped_named(&fixture, &named, &["commit", "--quiet", "--message=named by variables", "--", "a.txt"])
        .env("GIT_DIR", named.git_dir.as_path())
        .env("GIT_WORK_TREE", named.work_tree.as_path())
        .output()
        .expect("the wrapped commit runs");
    assert!(served.status.success(), "the commit landed: {}", String::from_utf8_lossy(served.stderr.as_slice()));
    let native: Output = direct(&fixture)
        .current_dir(oracle.work_tree.as_path())
        .env("GIT_DIR", oracle.git_dir.as_path())
        .env("GIT_WORK_TREE", oracle.work_tree.as_path())
        .args(["commit", "--quiet", "--message=named by variables", "--", "a.txt"])
        .output()
        .expect("native Git commits");
    assert!(native.status.success(), "the oracle commits: {}", String::from_utf8_lossy(native.stderr.as_slice()));
    assert_eq!(
        landed(&fixture, named.work_tree.as_path(), options_of(&named).as_slice()),
        landed(&fixture, oracle.work_tree.as_path(), options_of(&oracle).as_slice()),
        "the tree, the message and the branch's length agree with native Git"
    );
    assert!(holder_unchanged(&fixture, named.work_tree.parent().expect("the holder")), "the repository the fixture is nested in gained no commit");
    remove_settled(&fixture);
}

/// What:
///  A relative `-C` moves the command once,
/// and the commit lands from where it moved to.
/// Why:
///  The wrapper already runs each Git it starts in the directory the `-C` chain reached,
/// and forwarding `-C` to that Git applied it a second time,
/// so `git -C .. commit` from a subdirectory read the repository one level above the one the command was in.
///  The control keeps the in-process layout reader off,
///  which is the answer Git gives for a repository the reader cannot prove it reads anyway
///  (a configuration value it cannot type,
///  a link,
///  a Git directory an environment variable names),
///  so every Git of the command really starts and the prefix each one receives is measured.
#[test]
fn a_commit_moved_by_a_relative_directory_change_lands() {
    let fixture: Fixture = fixture("naming-relative-C");
    let holder: PathBuf = repository(&fixture, OsStr::new("held"));
    let repo: PathBuf = plain_with_a_subdirectory(&fixture, holder.join("repo").as_path());
    let oracle_holder: PathBuf = repository(&fixture, OsStr::new("held-oracle"));
    let oracle: PathBuf = plain_with_a_subdirectory(&fixture, oracle_holder.join("repo").as_path());
    let served: Output = wrapped_commit(&fixture, repo.join("sub").as_path(), &["-C", "..", "commit", "--quiet", "--message=moved once", "--", "tracked.txt"])
        .env(git_policy_cli::repo_read_hook::SWITCH_VARIABLE, "off")
        .output()
        .expect("the wrapped commit runs");
    assert!(served.status.success(), "the commit landed: {}", String::from_utf8_lossy(served.stderr.as_slice()));
    let native: Output = direct(&fixture)
        .current_dir(oracle.join("sub").as_path())
        .args(["-C", "..", "commit", "--quiet", "--message=moved once", "--", "tracked.txt"])
        .output()
        .expect("native Git commits");
    assert!(native.status.success(), "the oracle commits: {}", String::from_utf8_lossy(native.stderr.as_slice()));
    assert_eq!(landed(&fixture, repo.as_path(), &[]), landed(&fixture, oracle.as_path(), &[]), "the commit agrees with native Git");
    assert!(holder_unchanged(&fixture, holder.as_path()), "the repository one level above the command gained no commit");
    remove_settled(&fixture);
}

/// What:
///  A commit that names another repository than the one its working directory is in lands in the named one,
/// and leaves the one it runs in untouched.
/// Why:
///  The wrapper once read the repository of the working directory:
/// it prepared the commit against that repository's branch,
/// refused it for a name it could not commit,
/// and left a scratch directory and an object in that repository.
///  Native Git commits the repository the line names,
/// and a person who names one means it.
#[test]
fn a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone() {
    let fixture: Fixture = fixture("naming-another-repository");
    let running_in: PathBuf = plain_with_a_subdirectory(&fixture, fixture.root.join("running-in").as_path());
    let named: PathBuf = plain_with_a_subdirectory(&fixture, fixture.root.join("named").as_path());
    let oracle_in: PathBuf = plain_with_a_subdirectory(&fixture, fixture.root.join("oracle-in").as_path());
    let oracle_named: PathBuf = plain_with_a_subdirectory(&fixture, fixture.root.join("oracle-named").as_path());
    let before: (String, Vec<String>) = stores_of(&fixture, running_in.as_path());
    let running_in_head: String = git_text(&fixture, running_in.as_path(), &["rev-parse", "HEAD"]);
    let running_in_status: String = git_text(&fixture, running_in.as_path(), &["status", "--short"]);
    let git_dir: String = format!("--git-dir={}", named.join(".git").display());
    let work_tree: String = format!("--work-tree={}", named.display());
    let served: Output = wrapped_commit(&fixture, running_in.as_path(), &[git_dir.as_str(), work_tree.as_str(), "commit", "--quiet", "--message=named elsewhere", "--", "tracked.txt"])
        .output()
        .expect("the wrapped commit runs");
    assert!(served.status.success(), "the commit lands in the repository the line names: {}", String::from_utf8_lossy(served.stderr.as_slice()));
    // Native Git, of the same line in an identical pair of repositories, is the oracle.
    let oracle_git_dir: String = format!("--git-dir={}", oracle_named.join(".git").display());
    let oracle_work_tree: String = format!("--work-tree={}", oracle_named.display());
    let native: Output = direct(&fixture)
        .current_dir(oracle_in.as_path())
        .args([oracle_git_dir.as_str(), oracle_work_tree.as_str(), "commit", "--quiet", "--message=named elsewhere", "--", "tracked.txt"])
        .output()
        .expect("native Git commits");
    assert!(native.status.success(), "the oracle commits: {}", String::from_utf8_lossy(native.stderr.as_slice()));
    assert_eq!(landed(&fixture, named.as_path(), &[]), landed(&fixture, oracle_named.as_path(), &[]), "the commit agrees with native Git");
    assert_eq!(git_text(&fixture, running_in.as_path(), &["rev-parse", "HEAD"]), running_in_head, "the branch the command ran in did not move");
    assert_eq!(git_text(&fixture, running_in.as_path(), &["status", "--short"]), running_in_status, "its uncommitted edit is still there");
    assert_eq!(stores_of(&fixture, running_in.as_path()), before, "the repository the command ran in gained nothing");
    assert_eq!(git_text(&fixture, oracle_in.as_path(), &["status", "--short"]), running_in_status, "the oracle's first repository was left alone too");
    remove_settled(&fixture);
}

/// What:
///  A hand bump of one manifest in a named repository carries every clean dependent's patch bump
///  in the same commit,
///  and brings their worktree copies to the committed bytes.
/// Why:
///  The owner's decision is that a hand bump commits its dependents in the same commit
/// (`doc/decision/private-npm-registry.md:158-160`),
/// and the wrapper reads the real index to prove each unselected copy still matches `HEAD`.
///  A listing that names neither `--git-dir` nor `--work-tree` starts in a work tree with no `.git`
/// of its own,
/// Git answers "not a git repository",
/// and the commit lands the hand bump while every clean dependent keeps its old version.
///  The fixture's own root sits inside this checkout,
/// so the service's Git would find that checkout's repository one level above the work tree and read
/// the staged index through that discovery;
/// `GIT_CEILING_DIRECTORIES` at the work tree's directory says what a person's home directory says,
/// that no repository encloses this work tree.
#[test]
fn a_named_commit_carries_the_dependents_of_a_hand_bump() {
    let fixture: Fixture = fixture("naming-dependent-version");
    let named: Named = dotfiles(&fixture, "named");
    let work_tree: &Path = named.work_tree.as_path();
    write(work_tree, "package/module/a/package.json", &manifest("@s/a", "1.0.0", None));
    write(work_tree, "package/module/b/package.json", &manifest("@s/b", "2.0.0", Some("@s/a")));
    write(work_tree, "package/module/c/package.json", &manifest("@s/c", "3.0.0", Some("@s/a")));
    write(work_tree, "package/config/pnpr/config.yaml", &publish_set(&["@s/a", "@s/b", "@s/c"]));
    write(work_tree, "monochromatic-git-policy-cli.config.jsonc", CONFIGURATION);
    let mut staged: Vec<String> = options_of(&named);
    staged.extend(["add", "--all"].map(str::to_string));
    git(&fixture, work_tree, staged.as_slice());
    let mut based: Vec<String> = options_of(&named);
    based.extend(["commit", "--quiet", "--message=workspace"].map(str::to_string));
    git(&fixture, work_tree, based.as_slice());
    write(work_tree, "package/module/a/package.json", &manifest("@s/a", "1.1.0", None));
    let mut raised: Vec<String> = options_of(&named);
    raised.extend(["add", "--", "package/module/a/package.json"].map(str::to_string));
    git(&fixture, work_tree, raised.as_slice());
    let ceiling: PathBuf = std::fs::canonicalize(work_tree.parent().expect("the work tree's directory"))
        .expect("the work tree's directory resolves");
    let mut line: Vec<String> = options_of(&named);
    line.extend(["commit", "--message=bump", "--", "package/module/a/package.json"].map(str::to_string));
    let words: Vec<&str> = line.iter().map(String::as_str).collect();
    let served: Output = wrapped_named(&fixture, &named, words.as_slice())
        .env("GIT_CEILING_DIRECTORIES", ceiling.as_os_str())
        .output()
        .expect("the wrapped commit runs");
    let report: String = String::from_utf8_lossy(served.stderr.as_slice()).into_owned();
    assert!(served.status.success(), "the commit lands: {report}");
    assert_eq!(
        named_text(&fixture, &named, &["show", "HEAD:package/module/a/package.json"]),
        manifest("@s/a", "1.1.0", None),
        "the hand bump lands: {report}"
    );
    assert_eq!(
        named_text(&fixture, &named, &["show", "HEAD:package/module/b/package.json"]),
        manifest("@s/b", "2.0.1", Some("@s/a")),
        "the clean dependent's bump lands in the same commit: {report}"
    );
    assert_eq!(
        named_text(&fixture, &named, &["show", "HEAD:package/module/c/package.json"]),
        manifest("@s/c", "3.0.1", Some("@s/a")),
        "every clean dependent's bump lands in the same commit: {report}"
    );
    assert_eq!(read(work_tree, "b"), manifest("@s/b", "2.0.1", Some("@s/a")), "the worktree copy takes the committed bytes");
    assert_eq!(read(work_tree, "c"), manifest("@s/c", "3.0.1", Some("@s/a")), "the worktree copy takes the committed bytes");
    assert_eq!(
        named_text(&fixture, &named, &["status", "--porcelain"]),
        "",
        "the commit left no version change behind in the worktree"
    );
    remove_settled(&fixture);
}

/// What:
///  A hand bump committed in a repository named by relative options,
///  reached by a relative `-C` from the directory beside its work tree,
///  carries every clean dependent's patch bump in the same commit.
/// Why:
///  Git resolves a relative `--git-dir` after the caller's last `-C`,
///  so a real-index listing that reads those options again from the worktree top
///  asks for `<work tree>/wt/../cfg.git`,
///  which is no repository:
///  the listing fails,
///  and the commit lands the hand bump while every clean dependent keeps its old version.
///  The owner's decision is that a hand bump commits its dependents in the same commit
/// (`doc/decision/private-npm-registry.md:158-160`).
///  The command runs from the directory Git moves out of,
///  which is where a person writes it,
///  and its relative options are the ones the same person would write there.
#[test]
fn a_moved_named_commit_carries_the_dependents_of_a_hand_bump() {
    let fixture: Fixture = fixture("naming-moved-dependent-version");
    let named: Named = dotfiles(&fixture, "named");
    let work_tree: &Path = named.work_tree.as_path();
    let holder: PathBuf = named.git_dir.parent().expect("the Git directory's directory").to_path_buf();
    write(work_tree, "package/module/a/package.json", &manifest("@s/a", "1.0.0", None));
    write(work_tree, "package/module/b/package.json", &manifest("@s/b", "2.0.0", Some("@s/a")));
    write(work_tree, "package/module/c/package.json", &manifest("@s/c", "3.0.0", Some("@s/a")));
    write(work_tree, "package/config/pnpr/config.yaml", &publish_set(&["@s/a", "@s/b", "@s/c"]));
    write(work_tree, "monochromatic-git-policy-cli.config.jsonc", CONFIGURATION);
    let mut staged: Vec<String> = options_of(&named);
    staged.extend(["add", "--all"].map(str::to_string));
    git(&fixture, work_tree, staged.as_slice());
    let mut based: Vec<String> = options_of(&named);
    based.extend(["commit", "--quiet", "--message=workspace"].map(str::to_string));
    git(&fixture, work_tree, based.as_slice());
    write(work_tree, "package/module/a/package.json", &manifest("@s/a", "1.1.0", None));
    let mut raised: Vec<String> = options_of(&named);
    raised.extend(["add", "--", "package/module/a/package.json"].map(str::to_string));
    git(&fixture, work_tree, raised.as_slice());
    let ceiling: PathBuf = std::fs::canonicalize(holder.as_path()).expect("the holder resolves");
    let served: Output = wrapped_named_in(
        &fixture,
        &named,
        holder.as_path(),
        &[
            "-C",
            "wt",
            "--git-dir=../cfg.git",
            "--work-tree=.",
            "commit",
            "--message=bump",
            "--",
            "package/module/a/package.json",
        ],
    )
    .env("GIT_CEILING_DIRECTORIES", ceiling.as_os_str())
    .output()
    .expect("the wrapped commit runs");
    let report: String = String::from_utf8_lossy(served.stderr.as_slice()).into_owned();
    assert!(served.status.success(), "the commit lands: {report}");
    assert_eq!(
        named_text(&fixture, &named, &["show", "HEAD:package/module/a/package.json"]),
        manifest("@s/a", "1.1.0", None),
        "the hand bump lands: {report}"
    );
    assert_eq!(
        named_text(&fixture, &named, &["show", "HEAD:package/module/b/package.json"]),
        manifest("@s/b", "2.0.1", Some("@s/a")),
        "the clean dependent's bump lands in the same commit: {report}"
    );
    assert_eq!(
        named_text(&fixture, &named, &["show", "HEAD:package/module/c/package.json"]),
        manifest("@s/c", "3.0.1", Some("@s/a")),
        "every clean dependent's bump lands in the same commit: {report}"
    );
    assert_eq!(read(work_tree, "b"), manifest("@s/b", "2.0.1", Some("@s/a")), "the worktree copy takes the committed bytes: {report}");
    assert_eq!(read(work_tree, "c"), manifest("@s/c", "3.0.1", Some("@s/a")), "the worktree copy takes the committed bytes: {report}");
    assert_eq!(
        named_text(&fixture, &named, &["status", "--porcelain"]),
        "",
        "the commit left no version change behind in the worktree: {report}"
    );
    remove_settled(&fixture);
}

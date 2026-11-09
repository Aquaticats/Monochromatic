//! What:
//!  Fixtures for the working-tree discovery:
//!  the parts of a `.git` file and a linked worktree's layout that decide
//!  where a scan reads its index from.
//! Why:
//!  The real shapes are only reachable through real Git repositories,
//!  so the tests that can use real Git do,
//!  while the ones Git itself refuses to create (a `gitdir:` line naming a
//!  path with a newline, a file without the prefix, an oversized file) are
//!  written directly and compared against Git's own reading rules.

/// Imports the discovery entry points under test.
use super::{discover, gitdir_from_git_file, index_hash_kind, lexically_normalized};
/// Imports the filesystem and path types the fixtures use.
use std::fs;
use std::path::{Path, PathBuf};
/// Imports the real Git binary for the fixtures that need a repository.
use std::process::Command;

/// What:     `fn unique_tmp(label)` returns a fresh empty directory under
///           the test scratch root.
/// Why:      Cargo test runs tests in parallel, so two fixtures cannot share
///           a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uniqueTmp(label: string) { return path.join(scratchRoot(), `${label}-${process.pid}`); }
/// ```
fn unique_tmp(label: &str) -> PathBuf {
    let dir: PathBuf = crate::test_scratch::scratch_root()
        .join(format!("git-working-tree-test-{}-{}", label, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create tmp dir");
    return dir
}

/// What:     `fn run_git(dir, args)` runs the real Git binary in `dir`.
/// Why:      The developer's `git` on PATH is the policy wrapper, which adds
///           hooks the fixtures must not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runGit(dir: string, args: string[]) { execFileSync('/usr/bin/git', args, { cwd: dir }); }
/// ```
fn run_git(dir: &Path, args: &[&str]) {
    let git_bin: &str = if Path::new("/usr/bin/git").exists() { "/usr/bin/git" } else { "git" };
    let status = Command::new(git_bin)
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git command failed to spawn");
    assert!(status.success(), "git {:?} failed in {:?}", args, dir);
}

/// What:     `fn seed_repository(repository)` creates a repository with one
///           committed file and a `.gitignore` that hides `*.ignored`.
/// Why:      A linked worktree can only be added on top of a commit, and the
///           fixture needs a file for the index to record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function seedRepository(repository: string) { ... }
/// ```
fn seed_repository(repository: &Path) {
    fs::create_dir_all(repository).expect("create repository dir");
    run_git(repository, &["init", "-q"]);
    run_git(repository, &["config", "user.email", "t@t"]);
    run_git(repository, &["config", "user.name", "t"]);
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(repository.join("plain.txt"), "ordinary content").expect("write plain.txt");
    run_git(repository, &["add", "-f", ".gitignore", "plain.txt"]);
    run_git(repository, &["commit", "-q", "-m", "initial", ".gitignore", "plain.txt"]);
}

/// What:     `fn write_git_file(root, contents)` writes `root/.git` as a file
///           holding the given bytes.
/// Why:      A `.git` file's parsing rules are Git's, not a shape real Git has
///           to be asked to produce; every case here is one Git itself reads
///           (or refuses).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeGitFile(root: string, contents: string) { writeFileSync(path.join(root, '.git'), contents); }
/// ```
fn write_git_file(root: &Path, contents: &[u8]) {
    fs::create_dir_all(root).expect("create fixture dir");
    fs::write(root.join(".git"), contents).expect("write .git file");
}

#[test]
fn discover_finds_the_index_of_a_linked_worktree() {
    let base = unique_tmp("linked-worktree");
    let main: PathBuf = base.join("main");
    seed_repository(&main);
    let worktree: PathBuf = base.join("wt");
    run_git(
        &main,
        &["worktree", "add", "-q", worktree.to_str().expect("worktree utf8"), "-b", "wt-branch"],
    );

    let tree = discover(&worktree).expect("a linked worktree is a working tree");
    let expected: PathBuf = main.join(".git/worktrees/wt");
    assert_eq!(
        tree.index_path(),
        expected.join("index"),
        "the index must be the one Git uses for the worktree"
    );
    assert!(tree.index_path().is_file(), "the resolved index must exist");
    assert_eq!(
        tree.config_path(),
        main.join(".git/config"),
        "a worktree's shared configuration lives in the common directory"
    );
    assert_eq!(tree.below_walk_root(b"plain.txt"), Some(&b"plain.txt"[..]));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_reads_the_scope_of_a_subfolder_walk_root() {
    let base = unique_tmp("subfolder-scope");
    let main: PathBuf = base.join("main");
    seed_repository(&main);
    let subfolder: PathBuf = main.join("sub");
    fs::create_dir_all(&subfolder).expect("create subfolder");
    fs::write(subfolder.join("inner.txt"), "ordinary content").expect("write inner.txt");
    run_git(&main, &["add", "-f", "sub/inner.txt"]);
    run_git(&main, &["commit", "-q", "-m", "inner", "sub/inner.txt"]);

    let tree = discover(&subfolder).expect("a subfolder of a repository is inside one");
    assert_eq!(tree.index_path(), main.join(".git/index"));
    assert_eq!(
        tree.below_walk_root(b"sub/inner.txt"),
        Some(&b"inner.txt"[..]),
        "an entry below the walked directory loses the prefix it carries"
    );
    assert_eq!(
        tree.below_walk_root(b"plain.txt"),
        None,
        "an entry outside the walked directory must not be reported under it"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_reads_a_relative_gitdir_line_of_a_submodule() {
    let base = unique_tmp("submodule-gitdir");
    let other: PathBuf = base.join("other");
    seed_repository(&other);
    let host: PathBuf = base.join("host");
    fs::create_dir_all(&host).expect("create host dir");
    run_git(&host, &["init", "-q"]);
    run_git(&host, &["config", "user.email", "t@t"]);
    run_git(&host, &["config", "user.name", "t"]);
    run_git(
        &host,
        &["-c", "protocol.file.allow=always", "submodule", "add", "-q", "../other", "vendor/sub"],
    );

    let tree = discover(&host.join("vendor/sub")).expect("a submodule is a working tree");
    assert_eq!(
        tree.index_path(),
        host.join(".git/modules/vendor/sub/index"),
        "a relative gitdir line resolves against the directory holding the .git file"
    );
    assert!(tree.index_path().is_file(), "the resolved index must exist");

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_walks_past_a_directory_without_a_git_entry() {
    let repository = unique_tmp("no-git-entry");
    seed_repository(&repository);
    let subfolder: PathBuf = repository.join("sub");
    fs::create_dir_all(&subfolder).expect("create subfolder");
    assert!(
        !subfolder.join(".git").exists(),
        "fixture precondition: the subfolder holds no .git entry"
    );

    let tree = discover(&subfolder).expect("a directory below a working tree is inside one");
    assert_eq!(
        tree.index_path(),
        repository.join(".git/index"),
        "the nearest .git above the walked directory is the one read"
    );

    let _ = fs::remove_dir_all(&repository);
}

#[test]
fn gitdir_from_git_file_reads_an_absolute_path() {
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/repo/.git/worktrees/wt\n"),
        Some(PathBuf::from("/srv/repo/.git/worktrees/wt"))
    );
}

#[test]
fn gitdir_from_git_file_keeps_a_newline_inside_the_named_path() {
    // Git strips exactly one trailing newline, so a newline inside the path
    // stays part of it rather than ending the line.
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/a\nb/.git\n"),
        Some(PathBuf::from("/srv/a\nb/.git")),
        "only the final newline is a terminator"
    );
}

#[test]
fn gitdir_from_git_file_keeps_a_path_with_spaces_whole() {
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/my repo/.git/worktrees/wt\n"),
        Some(PathBuf::from("/srv/my repo/.git/worktrees/wt"))
    );
}

#[test]
fn gitdir_from_git_file_rejects_a_file_without_the_prefix() {
    assert_eq!(gitdir_from_git_file(b"/srv/repo/.git\n"), None);
    assert_eq!(gitdir_from_git_file(b"gitdir:/srv/repo/.git\n"), None);
    assert_eq!(gitdir_from_git_file(b""), None);
    assert_eq!(gitdir_from_git_file(b"gitdir: \n"), None);
}

#[test]
fn discover_resolves_a_gitdir_line_naming_an_absolute_path_outside_the_tree() {
    let base = unique_tmp("absolute-gitdir-line");
    let git_dir: PathBuf = base.join("admin dir");
    fs::create_dir_all(&git_dir).expect("create the git directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("write HEAD");
    let worktree: PathBuf = base.join("wt");
    write_git_file(
        &worktree,
        format!("gitdir: {}\n", git_dir.to_str().expect("git dir utf8")).as_bytes(),
    );

    let tree = discover(&worktree).expect("a .git file names a working tree");
    assert_eq!(tree.index_path(), git_dir.join("index"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_resolves_a_relative_gitdir_line_against_the_working_tree() {
    let base = unique_tmp("relative-gitdir-line");
    let git_dir: PathBuf = base.join("shared/admin");
    fs::create_dir_all(&git_dir).expect("create the git directory");
    let worktree: PathBuf = base.join("wt");
    write_git_file(&worktree, b"gitdir: ../shared/admin\n");

    let tree = discover(&worktree).expect("a relative gitdir line resolves");
    assert_eq!(
        tree.index_path(),
        git_dir.join("index"),
        "a relative gitdir line is resolved against the directory holding the .git file"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_walks_past_a_malformed_git_file() {
    let repository = unique_tmp("malformed-git-file");
    seed_repository(&repository);
    let nested: PathBuf = repository.join("nested");
    write_git_file(&nested, b"not a git file\n");

    let tree = discover(&nested).expect("the repository above the malformed file is found");
    assert_eq!(
        tree.index_path(),
        repository.join(".git/index"),
        "a .git file without the gitdir: prefix names no working tree, so discovery continues upward"
    );

    let _ = fs::remove_dir_all(&repository);
}

#[test]
fn lexically_normalized_drops_navigation_components() {
    assert_eq!(
        lexically_normalized(Path::new("/srv/a/./b/../c")),
        PathBuf::from("/srv/a/c")
    );
    assert_eq!(lexically_normalized(Path::new("/srv/a/../../..")), PathBuf::from("/"));
}

/// What:     `fn write_config(dir, contents)` materialises a Git
///           configuration at `dir/config`.
/// Why:      Each hash-kind case is one section shape; a helper keeps the
///           test bodies on the shape they test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeConfig(dir: string, contents: string) { writeFileSync(path.join(dir, 'config'), contents); }
/// ```
fn write_config(dir: &Path, contents: &str) {
    fs::create_dir_all(dir).expect("create config dir");
    fs::write(dir.join("config"), contents).expect("write config");
}

#[test]
fn index_hash_kind_reads_sha256_from_the_standard_shape() {
    let dir = unique_tmp("detect-sha256-standard");
    write_config(&dir, "[extensions]\n\tobjectformat = sha256\n[core]\n\tbare = false\n");
    assert_eq!(index_hash_kind(&dir.join("config")), gix_hash::Kind::Sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn index_hash_kind_defaults_to_sha1_without_the_extensions_section() {
    let dir = unique_tmp("detect-sha1-default");
    write_config(&dir, "[core]\n\tbare = false\n[user]\n\temail = t@t\n");
    assert_eq!(index_hash_kind(&dir.join("config")), gix_hash::Kind::Sha1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn index_hash_kind_defaults_to_sha1_when_the_config_is_missing() {
    let dir = unique_tmp("detect-sha1-noconfig");
    assert_eq!(index_hash_kind(&dir.join("config")), gix_hash::Kind::Sha1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn index_hash_kind_reads_sha256_with_comments_and_irregular_whitespace() {
    let dir = unique_tmp("detect-sha256-comments");
    write_config(
        &dir,
        "# comment line\n[Extensions]\n  ObjectFormat=  sha256  ; trailing\n[core]\n",
    );
    assert_eq!(index_hash_kind(&dir.join("config")), gix_hash::Kind::Sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn index_hash_kind_ignores_objectformat_outside_the_extensions_section() {
    let dir = unique_tmp("detect-sha1-wrong-section");
    write_config(&dir, "[core]\n\tobjectformat = sha256\n");
    assert_eq!(
        index_hash_kind(&dir.join("config")),
        gix_hash::Kind::Sha1,
        "a bogus shape must default to Sha1 rather than corrupt real Sha1 indexes"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn index_hash_kind_reads_sha256_from_an_extensions_subsection() {
    let dir = unique_tmp("detect-sha256-subsection");
    write_config(&dir, "[extensions \"weird\"]\n\tobjectformat = sha256\n");
    assert_eq!(index_hash_kind(&dir.join("config")), gix_hash::Kind::Sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn discover_returns_none_when_no_ancestor_holds_a_git_entry() {
    // Only the filesystem root sits above this path, so nothing names a
    // working tree; the precondition assert fails loudly on a machine that
    // does hold a repository there rather than testing something else.
    let unreachable: PathBuf = PathBuf::from("/").join("forbidden-strings-no-such-directory");
    for ancestor in unreachable.ancestors() {
        assert!(
            !ancestor.join(".git").exists(),
            "fixture precondition: {} holds a .git entry",
            ancestor.display()
        );
    }
    assert!(
        discover(&unreachable).is_none(),
        "a path with no .git entry above it is not inside a working tree"
    );
}

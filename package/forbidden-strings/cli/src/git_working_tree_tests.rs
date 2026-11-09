//! What:
//!  Fixtures for the working-tree discovery:
//!  the parts of a `.git` file and a working tree layout that decide where a
//!  scan reads its index from.
//! Why:
//!  The real shapes are only reachable through real Git repositories,
//!  so the tests that can use real Git do,
//!  and the ones Git itself refuses to create are written directly and then
//!  compared against what the installed Git binary accepts,
//!  because a parser asserting its own expectations proves nothing.

/// Imports the discovery entry points under test.
use super::{
    discover, gitdir_from_git_file, index_hash_kind, lexically_normalized, Discovery, GitWorkingTree,
};
/// Imports the filesystem and path types the fixtures use.
use std::fs;
use std::path::{Path, PathBuf};
/// Imports the real Git binary for the fixtures that need a repository.
use std::process::Command;

/// What:
///     `fn unique_tmp(label)` returns a fresh empty directory under
///           the test scratch root.
/// Why:
///      Cargo test runs tests in parallel,
/// so two fixtures cannot share
///           a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uniqueTmp(label: string) { return path.join(scratchRoot(), `${label}-${process.pid}`); }
/// ```
fn unique_tmp(label: &str) -> PathBuf {
    let dir: PathBuf = crate::test_scratch::scratch_root().join(format!(
        "git-working-tree-test-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create tmp dir");
    return dir
}

/// What:
///     `const GIT` names the system Git binary,
/// falling back to `PATH`.
/// Why:
///      The developer's `git` on `PATH` is this repository's policy
///           wrapper,
/// whose hooks and copies a fixture must not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GIT = fs.existsSync('/usr/bin/git') ? '/usr/bin/git' : 'git';
/// ```
fn git_binary() -> &'static str {
    if Path::new("/usr/bin/git").exists() {
        return "/usr/bin/git"
    }
    return "git"
}

/// What:
///     `fn run_git(dir, args)` runs the real Git binary in `dir`.
/// Why:
///      Fixture state must be Git's own serialization,
/// not a
///           hand-written imitation of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runGit(dir: string, args: string[]) { execFileSync(GIT, args, { cwd: dir }); }
/// ```
fn run_git(dir: &Path, args: &[&str]) {
    let status = Command::new(git_binary())
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git command failed to spawn");
    assert!(status.success(), "git {:?} failed in {:?}", args, dir);
}

/// What:
///     `fn seed_repository(repository)` creates a repository with one
///           committed file.
/// Why:
///      A linked worktree needs a commit to check out,
/// and the index
///           needs a file to record.
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
    fs::write(repository.join("plain.txt"), "ordinary content").expect("write plain.txt");
    run_git(repository, &["add", "-f", "plain.txt"]);
    run_git(repository, &["commit", "-q", "-m", "initial", "plain.txt"]);
}

/// What:
///     `fn write_git_file(root, contents)` writes `root/.git` as a file
///           holding the given bytes.
/// Why:
///      A `.git` file's parsing rules belong to Git,
/// and the bytes that
///           test them are cheap to write while being awkward to make Git
///           emit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeGitFile(root: string, contents: Buffer) { writeFileSync(path.join(root, '.git'), contents); }
/// ```
fn write_git_file(root: &Path, contents: &[u8]) {
    fs::create_dir_all(root).expect("create fixture dir");
    fs::write(root.join(".git"), contents).expect("write .git file");
}

/// What:
///     `fn expect_tree(discovery)` unwraps a resolved working tree and
///           fails with the discovery's own words otherwise.
/// Why:
///      Every assertion below needs the tree,
/// and a failure must say
///           which of "no repository" or "unusable entry" came back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expectTree(discovery: Discovery) { if (discovery.kind !== 'working-tree') throw new Error(JSON.stringify(discovery)); return discovery.tree; }
/// ```
fn expect_tree(discovery: Discovery) -> GitWorkingTree {
    return match discovery {
        Discovery::WorkingTree(tree) => tree,
        Discovery::NoRepository => panic!("discovery found no repository"),
        Discovery::Unusable { entry, cause } => panic!("discovery rejected {:?}: {}", entry, cause),
    }
}

/// What:
///     `fn resolved(path)` returns `path` with its parent directory
///           canonicalized,
/// so a fixture path and a path discovery built from
///           the same directory compare equal even when the scratch root sits
///           behind a symlink.
/// Why:
///      Discovery follows the filesystem's own resolution,
/// as Git does,
///           so comparing raw spellings would test the scratch root's mounting
///           rather than the resolution under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolved(path: string): string { return join(realpathSync(dirname(path)), basename(path)); }
/// ```
fn resolved(path: &Path) -> PathBuf {
    let Some(parent) = path.parent().and_then(|parent| return std::fs::canonicalize(parent).ok()) else {
        return path.to_path_buf()
    };
    return match path.file_name() {
        Some(name) => parent.join(name),
        None => parent,
    }
}

/// What:
///     `fn git_git_dir(root)` asks the installed Git which Git directory
///           it uses for `root`,
/// or `None` when Git refuses the directory.
/// Why:
///      Every claim about stepping over an entry or stopping on one is a
///           claim about what Git does;
/// asking Git settles it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitGitDir(root: string): string | null { ... }
/// ```
fn git_git_dir(root: &Path) -> Option<PathBuf> {
    let output = Command::new(git_binary())
        .args(["rev-parse", "--absolute-git-dir"])
        .current_dir(root)
        .output()
        .expect("git rev-parse failed to spawn");
    if !output.status.success() {
        return None
    }
    let text: String = String::from_utf8(output.stdout).expect("git path utf8");
    return Some(PathBuf::from(text.trim_end_matches(['\n', '\r'])))
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

    let tree = expect_tree(discover(&worktree));
    let expected: PathBuf = main.join(".git/worktrees/wt");
    assert_eq!(
        resolved(&tree.index_path()),
        resolved(&expected.join("index")),
        "the index must be the one Git uses for the worktree"
    );
    assert!(tree.index_path().is_file(), "the resolved index must exist");
    assert_eq!(
        resolved(&tree.config_path()),
        resolved(&main.join(".git/config")),
        "a worktree's shared configuration lives in the common directory"
    );
    assert_eq!(tree.below_walk_root(b"plain.txt"), Some(&b"plain.txt"[..]));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_reads_the_scope_of_a_subfolder_walk_root() {
    let base = unique_tmp("subfolder-scope");
    let repository: PathBuf = base.join("repository");
    seed_repository(&repository);
    let subfolder: PathBuf = repository.join("sub");
    fs::create_dir_all(&subfolder).expect("create subfolder");
    fs::write(subfolder.join("inner.txt"), "ordinary content").expect("write inner.txt");
    run_git(&repository, &["add", "-f", "sub/inner.txt"]);
    run_git(&repository, &["commit", "-q", "-m", "inner", "sub/inner.txt"]);

    let tree = expect_tree(discover(&subfolder));
    assert_eq!(resolved(&tree.index_path()), resolved(&repository.join(".git/index")));
    assert_eq!(
        tree.below_walk_root(b"sub/inner.txt"),
        Some(&b"inner.txt"[..]),
        "an entry below the walked directory loses the prefix it carries"
    );
    assert_eq!(
        tree.below_walk_root(b"submarine.txt"),
        None,
        "a sibling whose name starts with the scope must not be treated as below it"
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

    let tree = expect_tree(discover(&host.join("vendor/sub")));
    assert_eq!(
        resolved(&tree.index_path()),
        resolved(&host.join(".git/modules/vendor/sub/index")),
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

    let tree = expect_tree(discover(&subfolder));
    assert_eq!(
        resolved(&tree.index_path()),
        resolved(&repository.join(".git/index")),
        "the nearest .git above the walked directory is the one read"
    );

    let _ = fs::remove_dir_all(&repository);
}

#[test]
fn discover_walks_past_a_directory_named_git_that_holds_no_repository() {
    let repository = unique_tmp("stray-git-directory");
    seed_repository(&repository);
    let nested: PathBuf = repository.join("nested");
    fs::create_dir_all(nested.join(".git")).expect("create a stray .git directory");
    // Git steps over a `.git` directory that holds no repository, so a scan
    // must not report an unusable repository nor invent an index for it.
    assert_eq!(
        git_git_dir(&nested).map(|dir| return resolved(&dir)),
        Some(resolved(&repository.join(".git"))),
        "fixture precondition: Git steps over the stray directory"
    );

    let tree = expect_tree(discover(&nested));
    assert_eq!(
        resolved(&tree.index_path()),
        resolved(&repository.join(".git/index")),
        "discovery must continue to the repository that does hold one"
    );

    let _ = fs::remove_dir_all(&repository);
}

#[test]
fn discover_reports_a_malformed_git_file_instead_of_stepping_over_it() {
    let repository = unique_tmp("malformed-git-file");
    seed_repository(&repository);
    let nested: PathBuf = repository.join("nested");
    write_git_file(&nested, b"not a git file\n");
    assert!(
        git_git_dir(&nested).is_none(),
        "fixture precondition: Git refuses the malformed file"
    );

    // Git stops discovery with a fatal error on a `.git` file it cannot read
    // rather than using the repository above it; the same stop here is the
    // value the caller turns into a diagnostic.
    match discover(&nested) {
        Discovery::Unusable { entry, cause } => {
            assert_eq!(entry, nested.join(".git"));
            assert!(
                cause.contains("gitdir: "),
                "the cause must name what the file lacks; got {:?}",
                cause
            );
        }
        Discovery::WorkingTree(tree) => panic!(
            "a malformed .git file must not fall back to the index at {:?}",
            tree.index_path()
        ),
        Discovery::NoRepository => panic!("a malformed .git file must be reported, not ignored"),
    }

    let _ = fs::remove_dir_all(&repository);
}

#[test]
fn discover_reports_a_git_file_naming_something_that_is_not_a_repository() {
    let build = unique_tmp("git-file-not-a-repository");
    let repository: PathBuf = build.join("repository");
    seed_repository(&repository);
    let not_a_repository: PathBuf = build.join("plain");
    fs::create_dir_all(&not_a_repository).expect("create plain dir");
    let nested: PathBuf = repository.join("nested");
    write_git_file(
        &nested,
        format!("gitdir: {}\n", not_a_repository.to_str().expect("plain utf8")).as_bytes(),
    );
    assert!(
        git_git_dir(&nested).is_none(),
        "fixture precondition: Git refuses a gitfile that names no repository"
    );

    match discover(&nested) {
        Discovery::Unusable { cause, .. } => assert!(
            cause.contains("not a Git directory"),
            "the cause must say the named directory is not a repository; got {:?}",
            cause
        ),
        other => panic!("expected an unusable .git file, got {}", describe(&other)),
    }

    let _ = fs::remove_dir_all(&build);
}

/// What:
///     `fn describe(discovery)` names what discovery returned,
/// for a
///           failure message.
/// Why:
///      `Discovery` carries no `Debug` implementation because a working
///           tree is not printable state a caller needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describe(discovery: Discovery): string { return discovery.kind; }
/// ```
fn describe(discovery: &Discovery) -> &'static str {
    return match discovery {
        Discovery::WorkingTree(_) => "a working tree",
        Discovery::NoRepository => "no repository",
        Discovery::Unusable { .. } => "an unusable .git entry",
    }
}

#[test]
fn discover_returns_no_repository_when_no_ancestor_holds_a_git_entry() {
    // Only the filesystem root sits above this path, so nothing names a
    // working tree; the precondition fails loudly on a machine that does hold
    // a repository there rather than testing something else.
    let unreachable: PathBuf = PathBuf::from("/").join("forbidden-strings-no-such-directory");
    for ancestor in unreachable.ancestors() {
        assert!(
            !ancestor.join(".git").exists(),
            "fixture precondition: {} holds a .git entry",
            ancestor.display()
        );
    }
    assert!(
        matches!(discover(&unreachable), Discovery::NoRepository),
        "a path with no .git entry above it is not inside a working tree"
    );
}

#[test]
fn gitdir_from_git_file_reads_an_absolute_path() {
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/repo/.git/worktrees/wt\n"),
        Ok(PathBuf::from("/srv/repo/.git/worktrees/wt"))
    );
}

#[test]
fn gitdir_from_git_file_strips_every_trailing_line_break() {
    // Git strips `\n` and `\r` in a loop, so CRLF, a missing final newline, and
    // several trailing line breaks all name the same path.
    for contents in [
        &b"gitdir: /srv/repo/.git"[..],
        &b"gitdir: /srv/repo/.git\r\n"[..],
        &b"gitdir: /srv/repo/.git\r"[..],
        &b"gitdir: /srv/repo/.git\n\n\n"[..],
    ] {
        assert_eq!(
            gitdir_from_git_file(contents),
            Ok(PathBuf::from("/srv/repo/.git")),
            "got {:?}",
            String::from_utf8_lossy(contents)
        );
    }
}

#[test]
fn gitdir_from_git_file_keeps_other_trailing_bytes() {
    // Git strips line breaks only: a path Git would then fail to resolve keeps
    // its spaces and tabs here too, so the two agree about what was named.
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/repo/.git   \n"),
        Ok(PathBuf::from("/srv/repo/.git   "))
    );
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/repo/.git\t\n"),
        Ok(PathBuf::from("/srv/repo/.git\t"))
    );
}

#[test]
fn gitdir_from_git_file_keeps_a_newline_inside_the_named_path() {
    // Only trailing line breaks are terminators, so a newline Git cannot strip
    // from the end stays inside the path.
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/a\nb/.git\n"),
        Ok(PathBuf::from("/srv/a\nb/.git"))
    );
}

#[test]
fn gitdir_from_git_file_keeps_a_path_with_spaces_whole() {
    assert_eq!(
        gitdir_from_git_file(b"gitdir: /srv/my repo/.git/worktrees/wt\n"),
        Ok(PathBuf::from("/srv/my repo/.git/worktrees/wt"))
    );
}

#[test]
fn gitdir_from_git_file_rejects_a_file_without_the_prefix_or_a_path() {
    for contents in [&b"/srv/repo/.git\n"[..], &b"gitdir:/srv/repo/.git\n"[..], &b""[..], &b"gitdir: \n"[..]] {
        assert!(
            gitdir_from_git_file(contents).is_err(),
            "Git refuses {:?}",
            String::from_utf8_lossy(contents)
        );
    }
}

#[test]
fn discover_resolves_a_gitdir_line_naming_an_absolute_path_outside_the_tree() {
    let base = unique_tmp("absolute-gitdir-line");
    let repository: PathBuf = base.join("repository");
    seed_repository(&repository);
    let git_dir: PathBuf = repository.join(".git");
    let worktree: PathBuf = base.join("wt");
    write_git_file(
        &worktree,
        format!("gitdir: {}\n", git_dir.to_str().expect("git dir utf8")).as_bytes(),
    );

    let tree = expect_tree(discover(&worktree));
    assert_eq!(resolved(&tree.index_path()), resolved(&git_dir.join("index")));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_resolves_a_relative_gitdir_line_against_the_working_tree() {
    let base = unique_tmp("relative-gitdir-line");
    let repository: PathBuf = base.join("repository");
    seed_repository(&repository);
    let worktree: PathBuf = base.join("repository/nested/wt");
    write_git_file(&worktree, b"gitdir: ../../.git\n");

    let tree = expect_tree(discover(&worktree));
    assert_eq!(
        resolved(&tree.index_path()),
        resolved(&repository.join(".git/index")),
        "a relative gitdir line is resolved against the directory holding the .git file"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn discover_follows_a_commondir_of_a_linked_worktree() {
    let base = unique_tmp("commondir");
    let repository: PathBuf = base.join("repository");
    seed_repository(&repository);
    let git_dir: PathBuf = repository.join(".git");
    let worktree_git_dir: PathBuf = git_dir.join("worktrees/wt");
    fs::create_dir_all(&worktree_git_dir).expect("create the worktree git directory");
    fs::write(worktree_git_dir.join("HEAD"), "ref: refs/heads/wt-branch\n").expect("write HEAD");
    fs::write(worktree_git_dir.join("commondir"), "../..\r\n").expect("write commondir");
    let worktree: PathBuf = base.join("wt");
    write_git_file(
        &worktree,
        format!("gitdir: {}\n", worktree_git_dir.to_str().expect("git dir utf8")).as_bytes(),
    );

    let tree = expect_tree(discover(&worktree));
    assert_eq!(resolved(&tree.index_path()), resolved(&worktree_git_dir.join("index")));
    assert_eq!(
        resolved(&tree.config_path()),
        resolved(&git_dir.join("config")),
        "commondir names the directory holding the shared configuration, line breaks stripped"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn lexically_normalized_drops_navigation_components() {
    assert_eq!(
        lexically_normalized(Path::new("/srv/a/./b/../c")),
        PathBuf::from("/srv/a/c")
    );
    assert_eq!(lexically_normalized(Path::new("/srv/a/../../..")), PathBuf::from("/"));
}

/// What:
///     `fn write_config(dir, contents)` materialises a Git
///           configuration at `dir/config`.
/// Why:
///      Each hash-kind case is one section shape;
/// a helper keeps the
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

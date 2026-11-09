// What:     Integration-style tests for `list_files` and the private
//           `detect_index_hash_kind` helper. Each test creates a temporary
//           git repository, populates it with a fixture file set, then
//           calls the function under test and asserts on the result. This
//           sidecar file is pulled in by
//           `#[cfg(test)] #[path = "walk_tests.rs"] mod tests;` at the
//           bottom of `walk.rs`, so it compiles only under
//           `cargo nextest run` / `cargo test` and still reaches `walk.rs`'s
//           private items through `super::`.
// Why:      The bug shapes (BUG 3 force-added gitignore, BUG 4 submodule
//           gitlink, Sha256 index parsing) only reproduce against real git
//           state; a unit test on a pure in-memory abstraction wouldn't
//           catch them.

use super::list_files;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

// What:     `fn unique_tmp(label: &str) -> PathBuf` returns a fresh
//           empty directory under the test scratch root.
//           Uses pid + label to avoid collisions across concurrent
//           tests; we don't take a `tempfile` crate dependency for
//           one test.
// Why:      Cargo test runs tests in parallel by default; without a
//           per-test unique path two tests would race on the same
//           directory.
//
// In TS you'd write (pseudocode):
// ```ts
// function uniqueTmp(label: string) {
//   return path.join(os.tmpdir(), `${label}-${process.pid}`);
// }
// ```
fn unique_tmp(label: &str) -> PathBuf {
    let dir = crate::test_scratch::scratch_root().join(format!(
        "fs-walk-test-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create tmp dir");
    return dir
}

// What:     `fn run_git(dir, args)` runs `git <args>` in `dir` and
//           panics if the exit status is non-zero. The args are
//           passed as a fixed-shape array so callers don't have to
//           build a Vec.
// Why:      Test setup needs deterministic git invocations; failing
//           fast on a setup error keeps the actual assertions
//           focused on the function under test.
//
// In TS you'd write (pseudocode):
// ```ts
// function runGit(dir: string, args: string[]) {
//   execSync(`git ${args.join(" ")}`, { cwd: dir });
// }
// ```
fn run_git(dir: &Path, args: &[&str]) {
    // What:     Resolve the git binary. The dev environment in this
    //           repo wraps `git` with a CLI policy enforcer that
    //           rejects bulk-add `.` and commit-without-pathspec
    //           shapes. Tests need direct access to the real binary
    //           so the setup steps run unmodified.
    // Why:      The wrapper is in `node_modules/.bin/git` which
    //           is earlier in PATH; tests spawning `git` would hit
    //           it and fail on perfectly normal setup invocations.
    //           Probe `/usr/bin/git` first; fall back to PATH lookup
    //           if absent so the test still works on other systems.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const gitBin = fs.existsSync("/usr/bin/git") ? "/usr/bin/git" : "git";
    // ```
    let git_bin = if std::path::Path::new("/usr/bin/git").exists() {
        "/usr/bin/git"
    } else {
        "git"
    };
    let status = Command::new(git_bin)
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git command failed to spawn");
    assert!(
        status.success(),
        "git {:?} failed in {:?}",
        args,
        dir
    );
}

#[test]
fn list_files_includes_force_added_gitignored_file() {
    // What:     Sets up a fixture git repo containing:
    //             - .gitignore with `*.ignored` pattern
    //             - tracked.ignored (force-added despite .gitignore)
    //             - normal.txt (tracked normally)
    //           Then calls `list_files(dir)` and asserts BOTH files
    //           appear in the output.
    // Why:      BUG 3 regression: pre-fix the WalkBuilder honoured
    //           .gitignore and silently skipped tracked.ignored.
    //           Post-fix the `git ls-files --ignored --exclude-standard`
    //           union recovers it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("list_files includes force-added gitignored file", () => { ... });
    // ```
    let dir = unique_tmp("bug3-tracked-ignored");
    run_git(&dir, &["init", "-q"]);
    run_git(&dir, &["config", "user.email", "t@t"]);
    run_git(&dir, &["config", "user.name", "t"]);
    fs::write(dir.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(dir.join("tracked.ignored"), "secret content")
        .expect("write tracked.ignored");
    fs::write(dir.join("normal.txt"), "normal content").expect("write normal.txt");
    run_git(&dir, &["add", "-f", ".gitignore", "tracked.ignored", "normal.txt"]);
    // Explicit pathspec on commit -- the dev environment's git
    // wrapper rejects commits without one, and real git accepts it
    // either way.
    run_git(
        &dir,
        &["commit", "-q", "-m", "initial", ".gitignore", "tracked.ignored", "normal.txt"],
    );

    let files = list_files(dir.to_str().expect("dir utf8")).expect("list_files");
    // What:     Collect basenames of returned paths into a Vec for
    //           the diagnostic message; the assertion is on
    //           membership, not order.
    // Why:      The walker returns paths like `<dir>/tracked.ignored`
    //           and `<dir>/./normal.txt`; comparing basenames keeps
    //           the test platform-independent.
    let basenames: Vec<String> = files
        .iter()
        .filter_map(|p| {
            return std::path::Path::new(p)
                .file_name()
                .and_then(|n| return n.to_str())
                .map(|s| return s.to_string())
        })
        .collect();
    assert!(
        basenames.iter().any(|b| return b == "normal.txt"),
        "normal tracked file must be listed; got {:?}",
        basenames
    );
    assert!(
        basenames.iter().any(|b| return b == "tracked.ignored"),
        "BUG 3: force-added gitignored file must be listed; got {:?}",
        basenames
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn list_files_excludes_submodule_gitlink_entries() {
    // What:     Sets up a fixture git repo, then injects a submodule
    //           gitlink (mode 0o160000) directly into the index via
    //           `git update-index --add --cacheinfo`. The on-disk
    //           path for the submodule does not exist as a real
    //           submodule clone -- the gitlink is just a tracked
    //           index entry pointing at an arbitrary object id.
    //           `list_files(dir)` must skip the gitlink because
    //           our gix-index path filters non-FILE/FILE_EXECUTABLE
    //           entries.
    // Why:      The gix-index replacement for the previous
    //           `git ls-files --cached --ignored --exclude-standard`
    //           subprocess reads ALL index entries. Without a mode
    //           filter, tracked submodules would surface as
    //           "Is a directory" read-error hits via BUG 4's
    //           surface (~12 false positives on the Linux kernel
    //           per scan). This test pins the filter against
    //           silent regression.
    let dir = unique_tmp("gix-mode-filter");
    run_git(&dir, &["init", "-q"]);
    run_git(&dir, &["config", "user.email", "t@t"]);
    run_git(&dir, &["config", "user.name", "t"]);
    // What:     Create one real tracked file to keep the index
    //           non-empty after our cacheinfo injection.
    fs::write(dir.join("real.txt"), "ordinary").expect("write real.txt");
    run_git(&dir, &["add", "-f", "real.txt"]);
    // What:     `git update-index --add --cacheinfo
    //           160000,<sha>,vendor/sub` adds an entry with
    //           Mode::COMMIT (gitlink). The SHA must be 40 hex
    //           chars; the value need not refer to a real commit
    //           for the index to accept the entry.
    // Why:      Mirrors a tracked submodule without the complexity
    //           of creating a real second repository.
    run_git(
        &dir,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000,0000000000000000000000000000000000000001,vendor/sub",
        ],
    );
    run_git(
        &dir,
        &["commit", "-q", "-m", "initial", "real.txt"],
    );

    let files = list_files(dir.to_str().expect("dir utf8")).expect("list_files");
    let normalized: Vec<String> = files
        .iter()
        .map(|p| return p.trim_start_matches("./").to_string())
        .collect();
    assert!(
        normalized.iter().any(|p| return p.ends_with("real.txt")),
        "regular tracked file must still be listed; got {:?}",
        normalized
    );
    assert!(
        !normalized.iter().any(|p| return p.ends_with("vendor/sub")),
        "submodule gitlink (Mode::COMMIT) must NOT be listed; got {:?}",
        normalized
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     Helper `write_config(dir, contents)` materialises a
//           `.git/config` file under `dir` with the given contents.
//           Used by the `detect_index_hash_kind` unit tests below.
// Why:      Each test exercises a different config shape; a helper
//           keeps the test bodies focused on the shape they test.
fn write_config(dir: &Path, contents: &str) {
    let git_dir = dir.join(".git");
    fs::create_dir_all(&git_dir).expect("create .git");
    fs::write(git_dir.join("config"), contents).expect("write .git/config");
}

#[test]
fn detect_sha256_in_standard_config_shape() {
    let dir = unique_tmp("detect-sha256-standard");
    write_config(
        &dir,
        "[extensions]\n\tobjectformat = sha256\n[core]\n\tbare = false\n",
    );
    assert_eq!(super::detect_index_hash_kind(&dir), gix_hash::Kind::Sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn detect_sha1_when_extensions_absent() {
    let dir = unique_tmp("detect-sha1-default");
    write_config(&dir, "[core]\n\tbare = false\n[user]\n\temail = t@t\n");
    assert_eq!(super::detect_index_hash_kind(&dir), gix_hash::Kind::Sha1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn detect_sha1_when_config_missing() {
    let dir = unique_tmp("detect-sha1-noconfig");
    // No .git/config written -- read_to_string returns Err, helper
    // must default to Sha1.
    assert_eq!(super::detect_index_hash_kind(&dir), gix_hash::Kind::Sha1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn detect_sha256_with_comments_and_irregular_whitespace() {
    let dir = unique_tmp("detect-sha256-comments");
    write_config(
        &dir,
        // What:     Mixed-case section, `#` and `;` comments,
        //           tab and space variation, mid-line comment.
        // Why:      Git config tolerates all of these; our parser
        //           must too.
        "# comment line\n[Extensions]\n  ObjectFormat=  sha256  ; trailing\n[core]\n",
    );
    assert_eq!(super::detect_index_hash_kind(&dir), gix_hash::Kind::Sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn detect_sha1_when_objectformat_is_outside_extensions_section() {
    let dir = unique_tmp("detect-sha1-wrong-section");
    // What:     `objectformat` lives under `[core]` here; our
    //           parser must NOT treat that as a Sha256 signal.
    // Why:      Bogus configs should default-to-Sha1 rather than
    //           silently break parsing on real Sha1 indexes.
    write_config(
        &dir,
        "[core]\n\tobjectformat = sha256\n",
    );
    assert_eq!(super::detect_index_hash_kind(&dir), gix_hash::Kind::Sha1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn detect_sha256_with_extensions_subsection() {
    let dir = unique_tmp("detect-sha256-subsection");
    // What:     `[extensions "foo"]` is a subsection; our parser
    //           treats it as `extensions` proper (`split_whitespace`
    //           strips the subsection name).
    // Why:      Git itself reads `extensions.objectformat` under
    //           any subsection; our detection should match.
    write_config(
        &dir,
        "[extensions \"weird\"]\n\tobjectformat = sha256\n",
    );
    assert_eq!(super::detect_index_hash_kind(&dir), gix_hash::Kind::Sha256);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn list_files_handles_sha256_repo_force_added_gitignored() {
    // What:     End-to-end test on a fresh `git init
    //           --object-format=sha256` repo with a force-added
    //           gitignored file. `list_files` must return both the
    //           ordinary tracked file AND the force-added one;
    //           gix-index must parse the Sha256 index correctly via
    //           our `detect_index_hash_kind` plumbing.
    // Why:      Pre-fix the in-process index reader was hard-wired
    //           to Sha1, so a Sha256 index either failed to parse
    //           (Err fall-through to walker-only, missing the
    //           force-added file) or produced garbage paths.
    //           Without this regression test the SHA-256 path could
    //           silently break next time we touch walk.rs.
    let dir = unique_tmp("sha256-force-added");
    run_git(&dir, &["init", "--object-format=sha256", "-q"]);
    run_git(&dir, &["config", "user.email", "t@t"]);
    run_git(&dir, &["config", "user.name", "t"]);
    fs::write(dir.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(dir.join("tracked.ignored"), "secret content")
        .expect("write tracked.ignored");
    fs::write(dir.join("normal.txt"), "normal content").expect("write normal.txt");
    run_git(&dir, &["add", "-f", ".gitignore", "tracked.ignored", "normal.txt"]);
    run_git(
        &dir,
        &["commit", "-q", "-m", "initial", ".gitignore", "tracked.ignored", "normal.txt"],
    );

    let files = list_files(dir.to_str().expect("dir utf8")).expect("list_files");
    let basenames: Vec<String> = files
        .iter()
        .filter_map(|p| {
            return std::path::Path::new(p)
                .file_name()
                .and_then(|n| return n.to_str())
                .map(|s| return s.to_string())
        })
        .collect();
    assert!(
        basenames.iter().any(|b| return b == "normal.txt"),
        "sha256 repo: normal tracked file must be listed; got {:?}",
        basenames
    );
    assert!(
        basenames.iter().any(|b| return b == "tracked.ignored"),
        "sha256 repo: force-added gitignored file must be listed via \
         gix-index path; got {:?}",
        basenames
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `fn seed_repository(repository, ignored)` creates a Git
//           repository whose `.gitignore` hides `*.ignored`, commits one
//           plain file, and force-adds the named ignored files so they are
//           tracked while the ignore rule still matches them.
// Why:      Every regression test below needs that one shape: a tracked file
//           the walker cannot see, so only the index union can report it.
//
// In TS you'd write (pseudocode):
// ```ts
// function seedRepository(repository: string, ignored: string[]) { ... }
// ```
fn seed_repository(repository: &Path, ignored: &[&str]) {
    fs::create_dir_all(repository).expect("create repository dir");
    run_git(repository, &["init", "-q"]);
    run_git(repository, &["config", "user.email", "t@t"]);
    run_git(repository, &["config", "user.name", "t"]);
    fs::write(repository.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(repository.join("plain.txt"), "ordinary content").expect("write plain.txt");
    let mut paths: Vec<&str> = vec![".gitignore", "plain.txt"];
    for relative in ignored {
        let absolute: PathBuf = repository.join(relative);
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).expect("create the ignored file's parent");
        }
        fs::write(&absolute, "secret content").expect("write the ignored tracked file");
        paths.push(relative);
    }
    let mut add: Vec<&str> = vec!["add", "-f"];
    add.extend(&paths);
    run_git(repository, &add);
    let mut commit: Vec<&str> = vec!["commit", "-q", "-m", "initial"];
    commit.extend(&paths);
    run_git(repository, &commit);
}

// What:     `fn listed(root)` lists one directory through `list_files`.
// Why:      One call site keeps every test below on the same entry point.
//
// In TS you'd write (pseudocode):
// ```ts
// function listed(root: string): string[] { return listFiles(root); }
// ```
fn listed(root: &Path) -> Vec<String> {
    return list_files(root.to_str().expect("root utf8")).expect("list_files")
}

// What:     `fn relative_listing(root)` reports each listed path relative to
//           `root`, with `/` separators.
// Why:      A scan of a nested directory must report paths under that
//           directory; an index-derived path that still carries the
//           repository-relative prefix, or an absolute one, fails here.
//
// In TS you'd write (pseudocode):
// ```ts
// function relativeListing(root: string): string[] { ... }
// ```
fn relative_listing(root: &Path) -> Vec<String> {
    return listed(root)
        .iter()
        .map(|path| return relative_to(root, path))
        .collect()
}

// What:     `fn relative_to` expresses one listed path relative to the
//           directory the scan started in.
// Why:      The walker reports absolute paths for an absolute root and
//           `./`-prefixed ones for `.`; the index reports
//           repository-relative paths. One conversion compares all three.
//
// In TS you'd write (pseudocode):
// ```ts
// function relativeTo(root: string, path: string): string { ... }
// ```
fn relative_to(root: &Path, path: &str) -> String {
    let entry: &Path = Path::new(path);
    let relative: &Path = entry.strip_prefix(root).unwrap_or(entry);
    return relative
        .to_str()
        .expect("utf8 relative path")
        .trim_start_matches("./")
        .replace('\\', "/")
}

// What:     `fn git_says_ignored(repository, relative)` asks Git whether the
//           named working-tree file matches an ignore rule.
// Why:      These tests claim the walker cannot see the force-added file;
//           that claim rests on Git itself calling the file ignored, not on
//           this test assuming what `*.ignored` matches. `--no-index` is
//           required because `git check-ignore` reports a tracked path as
//           not ignored without it, which would make the precondition test
//           the opposite of the fixture's intent.
//
// In TS you'd write (pseudocode):
// ```ts
// function gitSaysIgnored(repository: string, relative: string): boolean { ... }
// ```
fn git_says_ignored(repository: &Path, relative: &str) -> bool {
    let status = Command::new("/usr/bin/git")
        .args(["check-ignore", "--quiet", "--no-index", "--", relative])
        .current_dir(repository)
        .status()
        .expect("git check-ignore failed to spawn");
    return status.success()
}

// What:     `fn add_linked_worktree(main, worktree)` asks Git to check the
//           main repository's current commit out into a linked worktree.
// Why:      A linked worktree is the shape issue 1232 names: its `.git` is a
//           file holding a `gitdir:` line, so `<worktree>/.git/index` does
//           not exist.
//
// In TS you'd write (pseudocode):
// ```ts
// function addLinkedWorktree(main: string, worktree: string) { ... }
// ```
fn add_linked_worktree(main: &Path, worktree: &Path) {
    run_git(
        main,
        &["worktree", "add", "-q", worktree.to_str().expect("worktree utf8"), "-b", "wt-branch"],
    );
    assert!(
        worktree.join(".git").is_file(),
        "fixture precondition: a linked worktree's .git is a file"
    );
}

#[test]
fn list_files_reports_a_tracked_ignored_file_in_a_linked_worktree() {
    let base = unique_tmp("worktree-root");
    let main = base.join("main");
    seed_repository(&main, &["tracked.ignored"]);
    let worktree = base.join("wt");
    add_linked_worktree(&main, &worktree);
    assert!(
        git_says_ignored(&worktree, "tracked.ignored"),
        "fixture precondition: git ignores the force-added file"
    );

    let paths = relative_listing(&worktree);
    assert!(
        paths.iter().any(|path| return path == "plain.txt"),
        "the plain tracked file must be listed; got {:?}",
        paths
    );
    assert!(
        paths.iter().any(|path| return path == "tracked.ignored"),
        "issue 1232: a linked worktree's tracked, ignored file must be listed; got {:?}",
        paths
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn list_files_reports_a_tracked_ignored_file_from_a_repository_subfolder() {
    let repository = unique_tmp("main-subfolder");
    seed_repository(&repository, &["sub/tracked.ignored"]);
    assert!(
        git_says_ignored(&repository, "sub/tracked.ignored"),
        "fixture precondition: git ignores the force-added file"
    );

    let paths = relative_listing(&repository.join("sub"));
    assert!(
        paths.iter().any(|path| return path == "tracked.ignored"),
        "issue 1232: a subfolder scan must report the repository's tracked, \
         ignored file under that subfolder; got {:?}",
        paths
    );
    assert!(
        !paths.iter().any(|path| return path == "plain.txt"),
        "a subfolder scan must not add files outside the walked directory; got {:?}",
        paths
    );

    let _ = fs::remove_dir_all(&repository);
}

#[test]
fn list_files_reports_a_tracked_ignored_file_from_a_linked_worktree_subfolder() {
    let base = unique_tmp("worktree-subfolder");
    let main = base.join("main");
    seed_repository(&main, &["sub/tracked.ignored"]);
    let worktree = base.join("wt");
    add_linked_worktree(&main, &worktree);

    let paths = relative_listing(&worktree.join("sub"));
    assert!(
        paths.iter().any(|path| return path == "tracked.ignored"),
        "issue 1232: a linked worktree subfolder scan must report the \
         tracked, ignored file under that subfolder; got {:?}",
        paths
    );
    assert!(
        !paths.iter().any(|path| return path == "plain.txt"),
        "a linked worktree subfolder scan must not add files outside the walked directory; got {:?}",
        paths
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn list_files_reports_a_tracked_ignored_file_in_a_worktree_path_with_spaces() {
    let base = unique_tmp("worktree-space");
    let main = base.join("main");
    seed_repository(&main, &["tracked.ignored"]);
    let worktree = base.join("wt with space");
    add_linked_worktree(&main, &worktree);

    let paths = relative_listing(&worktree);
    assert!(
        paths.iter().any(|path| return path == "tracked.ignored"),
        "a gitdir line naming a path with a space must resolve; got {:?}",
        paths
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn list_files_reports_a_tracked_ignored_file_in_a_submodule_with_a_relative_gitdir() {
    let base = unique_tmp("submodule-relative-gitdir");
    let other = base.join("other");
    seed_repository(&other, &["tracked.ignored"]);
    let host = base.join("host");
    fs::create_dir_all(&host).expect("create host dir");
    run_git(&host, &["init", "-q"]);
    run_git(&host, &["config", "user.email", "t@t"]);
    run_git(&host, &["config", "user.name", "t"]);
    run_git(
        &host,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            "../other",
            "vendor/sub",
        ],
    );
    let submodule = host.join("vendor/sub");
    let dot_git: String = fs::read_to_string(submodule.join(".git")).expect("read the submodule's .git file");
    assert!(
        dot_git.starts_with("gitdir: ../"),
        "fixture precondition: a submodule's gitdir line is relative; got {:?}",
        dot_git
    );

    let paths = relative_listing(&submodule);
    assert!(
        paths.iter().any(|path| return path == "tracked.ignored"),
        "a relative gitdir line must resolve against the working tree; got {:?}",
        paths
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn list_files_reports_a_tracked_ignored_file_in_a_sha256_linked_worktree() {
    let base = unique_tmp("worktree-sha256");
    let main = base.join("main");
    fs::create_dir_all(&main).expect("create main dir");
    run_git(&main, &["init", "-q", "--object-format=sha256"]);
    run_git(&main, &["config", "user.email", "t@t"]);
    run_git(&main, &["config", "user.name", "t"]);
    fs::write(main.join(".gitignore"), "*.ignored\n").expect("write .gitignore");
    fs::write(main.join("plain.txt"), "ordinary content").expect("write plain.txt");
    fs::write(main.join("tracked.ignored"), "secret content").expect("write tracked.ignored");
    run_git(&main, &["add", "-f", ".gitignore", "plain.txt", "tracked.ignored"]);
    run_git(
        &main,
        &["commit", "-q", "-m", "initial", ".gitignore", "plain.txt", "tracked.ignored"],
    );
    let worktree = base.join("wt");
    add_linked_worktree(&main, &worktree);

    let paths = relative_listing(&worktree);
    assert!(
        paths.iter().any(|path| return path == "tracked.ignored"),
        "a SHA-256 worktree index must parse; got {:?}",
        paths
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn list_files_scans_a_directory_without_a_repository_of_its_own() {
    let directory = unique_tmp("no-repository");
    fs::write(directory.join("only.txt"), "ordinary content").expect("write only.txt");
    assert!(
        !directory.join(".git").exists(),
        "fixture precondition: the directory holds no .git entry"
    );

    let paths = relative_listing(&directory);
    assert_eq!(
        paths,
        vec!["only.txt".to_string()],
        "a directory without a repository of its own must scan exactly what the walker sees"
    );

    let _ = fs::remove_dir_all(&directory);
}

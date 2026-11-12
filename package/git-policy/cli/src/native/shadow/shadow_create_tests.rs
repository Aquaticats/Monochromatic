//! Controls for the shadow repository as a whole:
//!  what it answers for `HEAD`,
//! the upstream,
//! conditional includes and the real refs,
//!  and that creating it leaves the real refs and index unchanged.

use super::*;
use crate::capture_invocation::{capture_invocation_base, capture_invocation_layout};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::shadow_git::shadow_request;
use crate::transaction_git::run_git_checked;
use crate::transaction_journal::RefFormat;

/// Captures the invocation of a repository's working directory,
/// as the transaction does.
fn capture_of(repo: &Path) -> InvocationCapture {
    let layout = capture_invocation_layout(&git_context(), repo, false).expect("layout");
    return capture_invocation_base(&git_context(), repo, layout).expect("capture");
}

/// Reads one shadow answer,
/// through the shadow request,
/// with the variables stripped.
fn shadow_text(shadow: &Path, arguments: &[&str]) -> String {
    let output = run_git_checked(&git_context(), &shadow_request(shadow, shadow, arguments))
        .expect("shadow answer");
    return String::from_utf8(output.stdout).expect("UTF-8").trim().to_string();
}

/// Creates the shadow of a fixture transaction and returns the shadow path and the transaction directory.
fn create(repo: &Path, root: &Path, transaction_id: &str) -> (PathBuf, PathBuf, InvocationCapture) {
    let capture: InvocationCapture = capture_of(repo);
    let transaction: PathBuf = root.join(format!("transaction-{transaction_id}"));
    std::fs::create_dir_all(&transaction).expect("transaction directory");
    let shadow: PathBuf = create_shadow_repository(
        &git_context(),
        repo,
        &capture,
        transaction_id,
        transaction.as_path(),
    )
    .expect("shadow");
    return (shadow, transaction, capture);
}

/// A fixed transaction ID of the shape the registry accepts.
const TRANSACTION: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// The shadow answers `HEAD`,
/// the base and the upstream the way the real repository does,
///  and sits under the transaction's path in the common directory.
#[test]
fn shadow_answers_head_base_and_upstream_like_the_real_repository() {
    let root: PathBuf = fixture("shadow-create-answers");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["remote", "add", "origin", "https://example.invalid/repo.git"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", head.as_str()]);
    git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(shadow, repo.join(".git").join("monochromatic-git-policy-cli").join("shadow").join(TRANSACTION));
    assert_eq!(shadow_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(
        shadow_text(shadow.as_path(), &["rev-parse", "--abbrev-ref", "@{upstream}"]),
        "origin/main"
    );
    assert_eq!(
        shadow_text(shadow.as_path(), &["config", "--get", "core.worktree"]),
        repo.display().to_string()
    );
    assert_eq!(shadow_text(shadow.as_path(), &["config", "--get", "gc.auto"]), "0");
    remove(root.as_path());
}

/// An `includeIf "onbranch:"` in the real config sees the shadow's `HEAD` and so matches the incumbent.
#[test]
fn include_if_onbranch_reads_the_shadow_head() {
    let root: PathBuf = fixture("shadow-create-onbranch");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".git").join("branch.config"), "[x]\n\tv = onbranch\n").expect("include");
    std::fs::OpenOptions::new()
        .append(true)
        .open(repo.join(".git").join("config"))
        .and_then(|mut file| {
            use std::io::Write;
            return file.write_all(b"[includeIf \"onbranch:main\"]\n\tpath = branch.config\n");
        })
        .expect("append include");
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(shadow_text(shadow.as_path(), &["config", "--get", "x.v"]), "onbranch");
    remove(root.as_path());
}

/// A detached real `HEAD` leaves the shadow detached at the same commit.
#[test]
fn detached_repository_gives_a_detached_shadow() {
    let root: PathBuf = fixture("shadow-create-detached");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]), head);
    assert!(run_git_checked(
        &git_context(),
        &shadow_request(shadow.as_path(), shadow.as_path(), &["symbolic-ref", "-q", "HEAD"]),
    )
    .is_err());
    remove(root.as_path());
}

/// The shadow links shared entries and never carries the real index,
/// `HEAD`'s private copies or objects.
#[test]
fn shadow_links_shared_entries_and_keeps_private_ones_out() {
    let root: PathBuf = fixture("shadow-create-links");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(
        std::fs::read_link(shadow.join("hooks")).expect("hooks link"),
        repo.join(".git").join("hooks")
    );
    assert!(std::fs::symlink_metadata(shadow.join("index")).is_err());
    assert_eq!(
        std::fs::read_to_string(shadow.join("objects").join("info").join("alternates")).expect("alternates"),
        format!("{}\n", repo.join(".git").join("objects").display())
    );
    remove(root.as_path());
}

/// What:
///  A repository whose directory name is not UTF-8 writes its real object directory into the shadow's alternates
///  as its own bytes,
///  and the shadow's Git reads the real store through it.
/// Why:
///   A spelling of the path replaced the byte with U+FFFD,
///  so the alternate named a directory that does not exist and the shadow's Git called the shadow no repository.
#[test]
fn the_alternates_keep_a_path_that_is_not_text() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("shadow-create-not-utf8");
    let repo: PathBuf = root.join(std::ffi::OsString::from_vec(b"rep\xefo".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    let mut expected: Vec<u8> = path_bytes_of(repo.join(".git").join("objects").as_path()).to_vec();
    expected.push(b'\n');
    assert_eq!(
        std::fs::read(shadow.join("objects").join("info").join("alternates")).expect("alternates"),
        expected,
        "the alternate is the object directory's own bytes"
    );
    assert_eq!(
        shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]),
        git_text(repo.as_path(), &["rev-parse", "HEAD"]),
        "the shadow's Git reads the real store through the alternate"
    );
    remove(root.as_path());
}

/// What:
///  A repository whose directory name holds a line feed writes its real object directory into the shadow's alternates
///  as a C-quoted line,
///  and the shadow's Git reads the real store through it.
/// Why:
///   The alternates file holds one pathname per line and an unquoted line ends at its first line feed,
///  so the entry named a directory that does not exist and the shadow's Git called the shadow no repository.
///  Git reads a line that begins with a double quote as a C-quoted pathname (`parse_alternates` in `odb.c`,
///  `unquote_c_style` in `quote.c`).
#[test]
fn the_alternates_quote_a_path_a_line_feed_would_split() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("shadow-create-line-feed");
    let repo: PathBuf = root.join(OsString::from_vec(b"a\nb".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    let mut expected: Vec<u8> = vec![b'"'];
    expected.extend_from_slice(path_bytes_of(root.as_path()));
    expected.extend_from_slice(b"/a\\nb/.git/objects\"\n");
    assert_eq!(
        std::fs::read(shadow.join("objects").join("info").join("alternates")).expect("alternates"),
        expected,
        "the line feed is escaped in a quoted line"
    );
    assert_eq!(
        shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]),
        git_text(repo.as_path(), &["rev-parse", "HEAD"]),
        "the shadow's Git reads the real store through the quoted alternate"
    );
    remove(root.as_path());
}

/// What:
///  A quoted alternate keeps a double quote and a backslash inside the one pathname it names.
/// Why:
///   A quote inside a quoted line would end the pathname early and a backslash would start an escape,
///  so both are written as their C escapes and the shadow's Git still reads the real store.
#[test]
fn a_quoted_alternate_keeps_a_quote_and_a_backslash_in_one_pathname() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("shadow-create-quoted-name");
    let repo: PathBuf = root.join(OsString::from_vec(b"q\"b\\s\nf".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    let mut expected: Vec<u8> = vec![b'"'];
    expected.extend_from_slice(path_bytes_of(root.as_path()));
    expected.extend_from_slice(b"/q\\\"b\\\\s\\nf/.git/objects\"\n");
    assert_eq!(
        std::fs::read(shadow.join("objects").join("info").join("alternates")).expect("alternates"),
        expected,
        "the quote and the backslash are escaped beside the line feed"
    );
    assert_eq!(
        shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]),
        git_text(repo.as_path(), &["rev-parse", "HEAD"]),
        "one pathname still names the real store"
    );
    remove(root.as_path());
}

/// Creating the shadow leaves the real refs,
/// the real index and the worktree bytes unchanged.
#[test]
fn creating_the_shadow_leaves_real_state_unchanged() {
    let root: PathBuf = fixture("shadow-create-unchanged");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    let refs_before: String = git_text(repo.as_path(), &["for-each-ref"]);
    let index_before: Vec<u8> = std::fs::read(repo.join(".git").join("index")).expect("index");
    let (_shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(git_text(repo.as_path(), &["for-each-ref"]), refs_before);
    assert_eq!(std::fs::read(repo.join(".git").join("index")).expect("index"), index_before);
    assert_eq!(std::fs::read_to_string(repo.join("a.txt")).expect("worktree"), "a\n");
    remove(root.as_path());
}

/// A merge in progress is copied into the shadow and the comparison copies,
/// as its own bytes.
#[test]
fn a_merge_in_progress_is_copied_to_the_shadow() {
    let root: PathBuf = fixture("shadow-create-merge");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(".git").join("MERGE_HEAD"), format!("{head}\n")).expect("merge head");
    std::fs::write(repo.join(".git").join("MERGE_MSG"), "Merge\n").expect("merge msg");
    let (shadow, transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(std::fs::read_to_string(shadow.join("MERGE_MSG")).expect("shadow msg"), "Merge\n");
    assert_eq!(
        std::fs::read_to_string(transaction.join("conclusion").join("MERGE_HEAD")).expect("copy"),
        format!("{head}\n")
    );
    // A files-backend ref store holds no conclusion name, so the state names nothing and no record is written.
    assert!(
        std::fs::symlink_metadata(transaction.join("conclusion").join("store-held.json")).is_err(),
        "a record that names nothing is not written"
    );
    remove(root.as_path());
}

/// A sha256 repository and a reftable repository both get a shadow that answers the same `HEAD`.
#[test]
fn sha256_and_reftable_repositories_get_matching_shadows() {
    let root: PathBuf = fixture("shadow-create-formats");
    let sha256: PathBuf = root.join("sha256");
    std::fs::create_dir(&sha256).expect("sha256 directory");
    git(sha256.as_path(), &["init", "--quiet", "--object-format=sha256", "--initial-branch=main"]);
    git(sha256.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let sha256_head: String = git_text(sha256.as_path(), &["rev-parse", "HEAD"]);
    let (shadow, _transaction, capture) = create(sha256.as_path(), sha256.parent().expect("fixture"), "11111111-1111-4111-8111-111111111111");
    assert_eq!(capture.layout.object_format, "sha256");
    assert_eq!(shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]), sha256_head);

    let reftable: PathBuf = root.join("reftable");
    std::fs::create_dir(&reftable).expect("reftable directory");
    git(reftable.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(reftable.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let reftable_head: String = git_text(reftable.as_path(), &["rev-parse", "HEAD"]);
    let (reftable_shadow, _reftable_transaction, reftable_capture) = create(reftable.as_path(), reftable.parent().expect("fixture"), "22222222-2222-4222-8222-222222222222");
    assert_eq!(reftable_capture.layout.ref_format, RefFormat::Reftable);
    assert_eq!(shadow_text(reftable_shadow.as_path(), &["rev-parse", "HEAD"]), reftable_head);
    remove(root.as_path());
}

/// What:
///  A reftable repository whose own directory name is not UTF-8 gets its shadow at its own path,
///  and that shadow answers the real `HEAD`.
/// Why:
///   The reftable backend's tables are Git's to create,
///  so the shadow is made by `git init`,
///  which was handed a spelling of the shadow's path:
///  `display()` wrote the bytes that are not UTF-8 as U+FFFD,
///  Git initialized that other directory,
///  and the creation then protected a path that did not exist,
///  so the commit could not be prepared and a stray shadow stayed under the replacement-character name.
#[test]
fn a_reftable_shadow_is_made_at_a_path_that_is_not_text() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("shadow-create-reftable-not-utf8");
    let repo: PathBuf = root.join(OsString::from_vec(b"reftable-\xef".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let (shadow, _transaction, capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    assert_eq!(capture.layout.ref_format, RefFormat::Reftable);
    assert_eq!(
        shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]),
        head,
        "the shadow's Git reads the real reftable refs through the shadow's own path"
    );
    let stray: PathBuf = root.join(OsString::from_vec("reftable-\u{fffd}".as_bytes().to_vec()));
    assert!(!stray.exists(), "no shadow is made at a spelling of the repository's path: {}", stray.display());
    remove(root.as_path());
}

/// The shadow and the directories above it are private,
/// as the incumbent's recursive private mkdir makes them.
#[cfg(unix)]
#[test]
fn shadow_and_its_parents_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("shadow-create-modes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    for directory in [
        shadow.clone(),
        shadow.parent().expect("parent").to_path_buf(),
        shadow.parent().expect("parent").parent().expect("grandparent").to_path_buf(),
    ] {
        let mode: u32 = std::fs::metadata(&directory).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "{}", directory.display());
    }
    remove(root.as_path());
}

/// The config and alternates that creation overwrites in the shadow are private files.
#[cfg(unix)]
#[test]
fn overwritten_shadow_files_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("shadow-create-file-modes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    for file in [shadow.join("config"), shadow.join("objects").join("info").join("alternates")] {
        let mode: u32 = std::fs::metadata(&file).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "{}", file.display());
    }
    remove(root.as_path());
}

/// The `config` starts Git logged to a trace file,
/// as their argument lists.
fn config_starts(trace: &Path) -> Vec<String> {
    return std::fs::read_to_string(trace)
        .unwrap_or_default()
        .lines()
        .filter(|line| { return line.contains("| start "); })
        .filter_map(|line| { return line.rsplit_once("| ").map(|(_head, command)| { return command.to_string(); }); })
        .filter(|command| { return command.split(' ').nth(1) == Some("config"); })
        .collect();
}

/// Creating a shadow reads the repository format from the snapshot when it can,
/// and from Git's `--file` listing
///  when the configuration includes another file.
#[test]
fn the_repository_format_comes_from_the_snapshot_unless_the_configuration_includes_a_file() {
    let root: PathBuf = fixture("shadow-create-format-source");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let trace: PathBuf = root.join("trace2.txt");
    let mut overlay = crate::test_support::isolated_overlay();
    overlay.push((std::ffi::OsString::from("GIT_TRACE2_PERF"), trace.clone().into_os_string()));
    let context: GitContext = GitContext { overlay, ..git_context() };
    for (name, include, transaction_id) in [
        ("plain", false, TRANSACTION),
        ("included", true, "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a11"),
    ] {
        if include {
            std::fs::write(repo.join(".git/extra"), "[user]\n\tname = Someone\n").expect("extra");
            git(repo.as_path(), &["config", "include.path", "extra"]);
        }
        let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
        let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
        let transaction: PathBuf = root.join(format!("transaction-{name}"));
        std::fs::create_dir_all(&transaction).expect("transaction directory");
        let _ = std::fs::remove_file(trace.as_path());
        create_shadow_repository(&context, repo.as_path(), &capture, transaction_id, transaction.as_path())
            .expect("shadow");
        let started: Vec<String> = config_starts(trace.as_path());
        if include {
            assert_eq!(started.len(), 1, "{started:?}");
            assert!(started[0].ends_with("config --file config --null --list"), "{started:?}");
        } else {
            assert!(started.is_empty(), "{started:?}");
        }
    }
    remove(root.as_path());
}

/// A directory that exists already is accepted,
/// a file in its place is not,
/// and a missing parent fails.
#[test]
fn an_existing_private_directory_is_accepted_and_a_file_is_not() {
    let root: PathBuf = fixture("shadow-ensure");
    let directory: PathBuf = root.join("shadow");
    ensure_private_directory(directory.as_path()).expect("created");
    ensure_private_directory(directory.as_path()).expect("already there");
    std::fs::write(root.join("file"), b"").expect("file");
    assert!(ensure_private_directory(root.join("file").as_path()).is_err());
    assert!(ensure_private_directory(root.join("missing/child").as_path()).is_err());
    remove(root.as_path());
}

/// Many commits creating the shared parents at once all succeed.
#[test]
fn concurrent_creators_of_the_shared_parents_all_succeed() {
    let root: PathBuf = fixture("shadow-parents-race");
    let shadow: PathBuf = root.join("monochromatic-git-policy-cli/shadow/one");
    let workers: Vec<std::thread::JoinHandle<std::io::Result<()>>> = (0..16)
        .map(|_| {
            let target: PathBuf = shadow.clone();
            return std::thread::spawn(move || { return create_parent(target.as_path()); });
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker").expect("every creator succeeds");
    }
    assert!(root.join("monochromatic-git-policy-cli/shadow").is_dir());
    remove(root.as_path());
}

/// What:
///  The `git init` that makes a reftable shadow,
/// as the wrapper starts it.
/// Why:
///   The owner's rule is that no Git start of the wrapper or the service gets `--quiet` or `-q`;
/// the reftable shadow is the
///  one whose tables only Git writes,
/// so it is made by a start of `init`,
/// which is looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepository(recordingGit, reftableRepo, ...); expect(startsOf('init')).toHaveLength(1); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_reftable_shadow_is_made_by_an_init_without_quiet() {
    let root: PathBuf = fixture("shadow-create-init-no-quiet");
    let reftable: PathBuf = root.join("reftable");
    std::fs::create_dir(&reftable).expect("reftable directory");
    git(reftable.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(reftable.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let capture: InvocationCapture = capture_of(reftable.as_path());
    assert_eq!(capture.layout.ref_format, RefFormat::Reftable);
    let (mut context, log) = crate::recording_git::recording_context(root.as_path());
    // The shadow's ref writes hand their paths on standard input, which a request makes in the common directory.
    context.common_dir = Some(reftable.join(".git"));
    let transaction: PathBuf = root.join("transaction-init-no-quiet");
    std::fs::create_dir_all(&transaction).expect("transaction directory");
    create_shadow_repository(&context, reftable.as_path(), &capture, TRANSACTION, transaction.as_path()).expect("shadow");
    let inits: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.starts_with("init "); }).collect();
    assert_eq!(inits.len(), 1, "the shadow is made by one start of init: {inits:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(root.as_path());
}

/// A reftable repository,
/// and a Git shim that hangs in the first `hang` starts of `init` after it made a partial shadow directory
///  (the way a cut `init` leaves one),
/// and passes everything else to real Git.
/// Returns the context,
/// the repository,
/// its capture,
/// the
///  file that counts the starts of `init`,
/// and the transaction directory.
fn reftable_with_hanging_init(root: &Path, hang: u32) -> (GitContext, PathBuf, InvocationCapture, PathBuf, PathBuf) {
    let reftable: PathBuf = root.join("reftable");
    std::fs::create_dir(&reftable).expect("reftable directory");
    git(reftable.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(reftable.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let capture: InvocationCapture = capture_of(reftable.as_path());
    let count: PathBuf = root.join("init-starts");
    let shim: PathBuf = root.join("hanging-init-git");
    crate::test_support::executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\nfor argument in \"$@\"; do last=\"$argument\"; if [ \"$argument\" = init ]; then is_init=1; fi; done\nif [ -n \"$is_init\" ]; then\n  n=$(cat '{count}' 2>/dev/null || echo 0); n=$((n + 1)); echo $n > '{count}'\n  if [ $n -le {hang} ]; then mkdir -p \"$last\"; : > \"$last/PARTIAL\"; exec sleep 600; fi\nfi\nexec /usr/bin/git \"$@\"\n",
            count = count.display(),
        )
        .as_bytes(),
    );
    let mut context: GitContext = GitContext { real_git: shim, ..git_context() };
    context.common_dir = Some(reftable.join(".git"));
    let transaction: PathBuf = root.join("transaction-hanging-init");
    std::fs::create_dir_all(&transaction).expect("transaction directory");
    return (context, reftable, capture, count, transaction);
}

/// A shadow whose `init` was ended at its ceiling is thrown away and made again in a fresh directory,
/// never run again in the partial
/// one:
/// the experiment found a cut `init` does not run twice to the same state in place (50 of 104 second runs ended with
/// different reftable tables),
/// and after the partial directory is discarded the second run equals an uncut one (104 of 104;
/// the owner's
/// "Never cut a branch write (Recommended)":
/// a step that cannot run twice has no ceiling unless it is restartable like this).
#[test]
fn a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory() {
    let root: PathBuf = fixture("shadow-create-init-cut");
    let (context, reftable, capture, count, transaction) = reftable_with_hanging_init(root.as_path(), 1);
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(std::time::Duration::from_millis(700))); });
    let made = create_shadow_repository(&context, reftable.as_path(), &capture, TRANSACTION, transaction.as_path());
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    let shadow: PathBuf = made.expect("the second start made the shadow");
    assert_eq!(std::fs::read_to_string(&count).expect("count").trim(), "2", "init was started twice");
    assert!(!shadow.join("PARTIAL").exists(), "nothing of the cut start is left in the shadow");
    assert_eq!(shadow_text(shadow.as_path(), &["rev-parse", "HEAD"]), git_text(reftable.as_path(), &["rev-parse", "HEAD"]));
    remove(root.as_path());
}

/// The fresh start is made a bounded number of times:
/// a shadow whose `init` is cut every time fails the transaction with a message
/// that names the step,
/// and leaves no partial directory.
#[test]
fn a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing() {
    let root: PathBuf = fixture("shadow-create-init-cut-always");
    let (context, reftable, capture, count, transaction) = reftable_with_hanging_init(root.as_path(), 1000);
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(std::time::Duration::from_millis(500))); });
    let began: std::time::Instant = std::time::Instant::now();
    let made = create_shadow_repository(&context, reftable.as_path(), &capture, TRANSACTION, transaction.as_path());
    let waited: std::time::Duration = began.elapsed();
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    let failure: GitFailure = made.expect_err("every start was cut");
    assert!(failure.0.contains("init"), "{failure:?}");
    assert_eq!(std::fs::read_to_string(&count).expect("count").trim(), SHADOW_CREATION_ATTEMPTS.to_string(), "{failure:?}");
    assert!(waited < std::time::Duration::from_secs(60), "{waited:?}");
    let shadow: PathBuf = crate::shadow_git::shadow_repository_path(reftable.join(".git").as_path(), TRANSACTION);
    assert!(!shadow.exists(), "the partial directory is discarded");
    remove(root.as_path());
}

/// Which outcomes of a start of `init` are retried in a fresh directory (sol-1 round 4):
/// a start ended at its ceiling,
/// a start the system
/// refused as a finished command with Git's words,
/// and a start the system refused before the command ran (`EAGAIN`,
/// which reaches here
/// as an I/O error of kind `WouldBlock`).
/// Another I/O error is not retried,
/// and neither is a finished command.
#[test]
fn only_a_cut_or_refused_start_of_init_is_retried() {
    let refused_text = b"fatal: unable to create thread: Resource temporarily unavailable\n".to_vec();
    let finished = |code: i32, stderr: Vec<u8>| -> std::io::Result<GitOutput> { return Ok(GitOutput { code: Some(code), stdout: Vec::new(), stderr }); };
    assert!(start_was_cut_or_refused(&Err(std::io::Error::from(std::io::ErrorKind::TimedOut))), "ended at the ceiling");
    assert!(start_was_cut_or_refused(&Err(std::io::Error::from_raw_os_error(libc::EAGAIN))), "the system refused the process before it ran");
    assert!(start_was_cut_or_refused(&finished(128, refused_text)), "Git reported the refusal");
    assert!(!start_was_cut_or_refused(&Err(std::io::Error::from(std::io::ErrorKind::NotFound))), "no such program");
    assert!(!start_was_cut_or_refused(&Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))), "not an executable");
    assert!(!start_was_cut_or_refused(&finished(128, b"fatal: not a repository\n".to_vec())), "a failure of another kind");
    assert!(!start_was_cut_or_refused(&finished(0, Vec::new())), "a start that worked");
}

/// A valid reference whose name is not text,
/// outside the target branch,
/// does not stop the shadow:
/// real Git commits beside it,
/// and the commit the wrapper builds needs the same repository.
#[test]
fn a_ref_name_that_is_not_text_does_not_stop_the_shadow() {
    let root: PathBuf = fixture("shadow-create-non-utf8-ref");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tag: std::ffi::OsString = crate::path_bytes::os_from_path_bytes(b"refs/tags/nonutf-\xff");
    git(repo.as_path(), &[std::ffi::OsStr::new("update-ref"), tag.as_os_str(), std::ffi::OsStr::new(head.as_str())]);
    // The positive control: real Git accepts this repository and its refs and commits in it.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=native control"]);
    let (shadow, _transaction, _capture) = create(repo.as_path(), root.as_path(), TRANSACTION);
    // The snapshot carries the name as the bytes it is, so the shadow's Git sees the same ref.
    let listed = run_git_checked(
        &git_context(),
        &shadow_request(shadow.as_path(), shadow.as_path(), &["for-each-ref", "--format=%(refname)"]),
    )
    .expect("the shadow lists its refs");
    let names: Vec<&[u8]> = listed.stdout.split(|byte| { return *byte == b'\n'; }).collect();
    assert!(names.contains(&b"refs/tags/nonutf-\xff".as_slice()), "{}", String::from_utf8_lossy(listed.stdout.as_slice()));
    remove(root.as_path());
}

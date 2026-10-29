//! Controls for the shadow repository as a whole:
//!  what it answers for `HEAD`, the upstream, conditional includes and the real refs,
//!  and that creating it leaves the real refs and index unchanged.

use super::*;
use crate::capture_invocation::{capture_invocation_base, capture_invocation_layout};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::shadow_git::shadow_request;
use crate::transaction_journal::RefFormat;

/// Captures the invocation of a repository's working directory, as the transaction does.
fn capture_of(repo: &Path) -> InvocationCapture {
    let layout = capture_invocation_layout(&git_context(), repo, false).expect("layout");
    return capture_invocation_base(&git_context(), repo, layout).expect("capture");
}

/// Reads one shadow answer, through the shadow request, with the variables stripped.
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

/// The shadow answers `HEAD`, the base and the upstream the way the real repository does,
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
    assert_eq!(shadow, repo.join(".git").join("cli-git").join("shadow").join(TRANSACTION));
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

/// The shadow links shared entries and never carries the real index, `HEAD`'s private copies or objects.
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

/// Creating the shadow leaves the real refs, the real index and the worktree bytes unchanged.
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

/// A merge in progress is copied into the shadow and the comparison copies, as its own bytes.
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
    assert_eq!(
        std::fs::read_to_string(transaction.join("conclusion").join("store-held.json")).expect("record"),
        "{}\n"
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

/// The shadow and the directories above it are private, as the incumbent's recursive private mkdir makes them.
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

/// The `config` starts Git logged to a trace file, as their argument lists.
fn config_starts(trace: &Path) -> Vec<String> {
    return std::fs::read_to_string(trace)
        .unwrap_or_default()
        .lines()
        .filter(|line| { return line.contains("| start "); })
        .filter_map(|line| { return line.rsplit_once("| ").map(|(_head, command)| { return command.to_string(); }); })
        .filter(|command| { return command.split(' ').nth(1) == Some("config"); })
        .collect();
}

/// Creating a shadow reads the repository format from the snapshot when it can, and from Git's `--file` listing
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

/// A directory that exists already is accepted, a file in its place is not, and a missing parent fails.
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
    let shadow: PathBuf = root.join("cli-git/shadow/one");
    let workers: Vec<std::thread::JoinHandle<std::io::Result<()>>> = (0..16)
        .map(|_| {
            let target: PathBuf = shadow.clone();
            return std::thread::spawn(move || { return create_parent(target.as_path()); });
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker").expect("every creator succeeds");
    }
    assert!(root.join("cli-git/shadow").is_dir());
    remove(root.as_path());
}

//! What:
//!  A commit made through the service where the runtime directory's path is long:
//!  as long as a socket path may be,
//!  one byte past that,
//!  and short.
//! Why:
//!  A Unix socket address carries 107 bytes of path,
//! and the runtime directory is the person's to choose:
//! a scratch folder under an agent's home,
//! a container's private directory and a nested session's directory all reach past the forty bytes a path form leaves.
//! Before the abstract form existed the commit failed there while real Git made the commit
//! (owner's rule, "Commits should never fail").
//! Each control runs the built wrapper through `git` on a fixture PATH,
//!  with a service the fixture starts in a runtime directory the control chose,
//!  so what is checked is the program a person runs and the address it really reaches.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, repository, start_fixture_service, start_fixture_service_in};
use git_policy_cli::service_address_of_a_worktree::socket_address_of;
use git_policy_cli::service_instance::{instance_of, longest_runtime_directory, socket_path};
use git_policy_cli::service_server::Bind;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};

/// The subject of the newest commit,
/// which is what says the commit was made.
const SUBJECT: &str = "through a long runtime directory";

/// The bytes a digest of a tag takes in a runtime directory's name,
/// which is the room this control asks for where the tag itself does not fit.
const DIGEST_NAME_BYTES: usize = 8;

/// What:
///  A runtime directory with a path of exactly `length` bytes,
///  made under the longest folder this account offers that leaves room for its name.
/// Why:
///   The boundary the kernel has is a count of bytes,
/// so a control that guessed the length would prove nothing about it,
/// and the fixture root under the build directory is far too long to build one under.
/// The room below the folder is what is left once its own path and the name are taken,
/// and it is shorter where the home directory's path is longer or the process number has more digits:
/// the gate's container leaves seven bytes under `$HOME/temp/agent`,
/// where a name of `at-the-longest` plus padding needs thirteen.
/// So the folder is asked for by the room the name needs,
/// and a control here never depends on how long this account's home directory is.
fn runtime_directory_of_length(length: usize, tag: &str) -> PathBuf {
    let name_length: usize = tag.len().min(DIGEST_NAME_BYTES);
    let longest_parent: usize = length.saturating_sub(1 + name_length);
    let parent: PathBuf = runtime_directory_parent("lr", longest_parent);
    let room: usize = length.checked_sub(parent.as_os_str().len() + 1).expect("a runtime directory longer than the folder it stands in");
    // The name holds the tag where there is room for it and padding to the byte the control asks for. A tag that does not
    //  fit is replaced by a digest of itself: two controls still get two directories, whichever number this process runs
    //  as, and the room the digest needs was what the folder above was chosen for.
    let name: String = if room >= tag.len() {
        format!("{tag}{}", "x".repeat(room - tag.len()))
    } else {
        let mut digest: String = git_policy_cli::service_instance::digest_of_bytes(tag.as_bytes());
        digest.truncate(room);
        digest
    };
    let runtime: PathBuf = parent.join(name);
    std::fs::create_dir_all(&runtime).expect("the runtime directory");
    assert_eq!(runtime.as_os_str().len(), length, "the directory is exactly as long as the control asks");
    return runtime;
}

/// What:
///  The folder this process's runtime directories are made in:
///  the longest place this account offers that is at most `longest` bytes long.
/// Why:
///   A socket address carries 107 bytes of path,
/// and the room a runtime directory of a fixed length leaves for its own name is what stays past the folder it stands in,
/// which depends on this account and on this process.
/// The binary controls cannot use the library's test support (`#[cfg(test)]` is not compiled into this target),
/// so this is the same choice the unit controls make,
/// named here for the one thing they share.
fn runtime_directory_parent(prefix: &str, longest: usize) -> PathBuf {
    let name: String = format!("{prefix}{}", std::process::id());
    let mut places: Vec<PathBuf> = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        // The folder this package's fixtures keep their sockets in.
        places.push(home.join("temp/agent").join(name.as_str()));
        // The same folder without the two segments the package adds, which a short home directory cannot spare.
        places.push(home.join("temp").join(name.as_str()));
    }
    if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from) {
        // The directory the product itself serves a session's sockets in, which is the shortest place an account has.
        places.push(runtime.join(name.as_str()));
    }
    for place in &places {
        if place.as_os_str().len() > longest {
            continue;
        }
        std::fs::create_dir_all(place).expect("the folder of this control's runtime directories");
        return place.clone();
    }
    let tried: String = if places.is_empty() {
        String::from("this account names no home directory and no runtime directory")
    } else {
        let described: Vec<String> = places.iter().map(|place| { return format!("{} ({} bytes)", place.display(), place.as_os_str().len()); }).collect();
        described.join(", ")
    };
    panic!("no folder this account offers leaves room for a runtime directory's name: the control needs a folder of at most {longest} bytes, and the folders are {tried}");
}

/// A repository with one committed file and an edit of it waiting for a commit.
fn edited(name: &str) -> (Fixture, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    return (fixture, repo);
}

/// What:
///  Commits `a.txt` through a service the fixture starts in `runtime`.
/// Why:
///   The runtime directory is the control's to place,
/// and the service is started at whatever address the wrapper's own function gives that directory,
/// which is the address the commit computes for itself.
/// The service is left in the fixture, so it runs until the control ends and the assertion that its address really
///  answers is made while it does.
fn commit_through(fixture: &Fixture, repo: &Path, runtime: PathBuf) -> Output {
    let service = start_fixture_service_in(runtime.clone());
    service.serve(fixture, repo.join(".git").as_path());
    *fixture.service.lock().expect("the fixture's service lock") = Some(service);
    let mut command = wrapped_commit(fixture, repo, &["commit", "--message", SUBJECT, "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime.as_path());
    return command.output().expect("the wrapped commit runs");
}

/// What:
///  A commit is made through the service where the runtime directory is as long as a socket path may be,
///  and the socket is the file it has always been.
/// Why:
///   This is the longest runtime directory that keeps the path form,
/// and the commit must be one no person can tell from any other.
/// The path form is what the sentence is about,
/// so it is said before the commit runs:
/// one byte longer would be waited at by a name,
/// and the socket file beside it would answer nothing.
#[test]
fn a_commit_is_made_where_the_runtime_directory_is_as_long_as_a_socket_path_allows() {
    let (fixture, repo) = edited("long-runtime-at-the-longest");
    let instance: String = instance_of(repo.join(".git").as_path());
    let runtime: PathBuf = runtime_directory_of_length(longest_runtime_directory(instance.as_str()), "at-the-longest");
    let socket: PathBuf = socket_path(runtime.as_path(), instance.as_str());
    assert!(
        matches!(socket_address_of(runtime.as_path(), instance.as_str()), Bind::Path(_)),
        "a runtime directory at the longest keeps the path form, whose socket path is {} bytes",
        socket.as_os_str().len()
    );
    let output: Output = commit_through(&fixture, &repo, runtime.clone());
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), SUBJECT, "the commit was made");
    assert!(socket.exists(), "the service listens at the socket file the path form names: {}", socket.display());
    remove_settled(&fixture);
}

/// What:
///  A commit is made through the service where the runtime directory is one byte too long for a socket path,
///  and the service answers at the abstract name that directory takes.
/// Why:
///   This is the failure the change removes:
/// the kernel refuses a socket path of 108 bytes,
/// so before the abstract form the manager bound nothing and the commit ended,
/// while real Git made the commit in the same directory.
#[test]
#[cfg(target_os = "linux")]
fn a_commit_is_made_where_the_runtime_directory_is_past_what_a_socket_path_carries() {
    let (fixture, repo) = edited("long-runtime-past-the-longest");
    let instance: String = instance_of(repo.join(".git").as_path());
    let runtime: PathBuf = runtime_directory_of_length(longest_runtime_directory(instance.as_str()) + 1, "past-the-longest");
    let output: Output = commit_through(&fixture, &repo, runtime.clone());
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), SUBJECT, "the commit was made");
    let Bind::Abstract(name) = socket_address_of(runtime.as_path(), instance.as_str()) else {
        panic!("a runtime directory this long is served at an abstract name")
    };
    assert!(
        git_policy_cli::service_address_of_a_worktree::the_address_is_there(&Bind::Abstract(name.clone())),
        "the service answers at the abstract name {name}"
    );
    assert!(
        !socket_path(runtime.as_path(), instance.as_str()).exists(),
        "no socket file can stand at a path this long, so the file form is not what answered"
    );
    remove_settled(&fixture);
}

/// What:
///  The same commit with a runtime directory that a socket path carries with room to spare is made exactly as it is today.
/// Why:
///   Every ordinary session has such a runtime directory,
/// and the control of the two long cases means nothing unless this one is green beside them.
#[test]
fn a_commit_with_a_short_runtime_directory_is_made_as_it_is_today() {
    let (fixture, repo) = edited("long-runtime-short");
    let service = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    let mut command = wrapped_commit(&fixture, &repo, &["commit", "--message", SUBJECT, "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", service.runtime.as_path());
    let output: Output = command.output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), SUBJECT, "the commit was made");
    assert!(
        socket_path(service.runtime.as_path(), instance_of(repo.join(".git").as_path()).as_str()).exists(),
        "the short runtime directory keeps the socket file"
    );
    remove_settled(&fixture);
}

/// What:
///  A first commit is made where the long runtime directory is spelled with a `..` part and is not there yet.
/// Why:
///   The client's abstract name is made before anything is created, and the socket unit's a moment later,
/// after provisioning has made the directory.
/// A `..` part over a directory that was not there gave those two names different places,
/// so the stand-in manager started the service where the unit names it and the commit waited at an address nothing
/// held, which is a commit real Git makes in the same directory.
/// The stand-in for the manager is the package's own: a bus of the fixture's, and the service started at whatever
/// address the wrapper's own function gives the runtime directory when the unit is started.
#[test]
#[cfg(target_os = "linux")]
fn a_first_commit_is_made_where_the_long_runtime_directory_has_a_relative_part_and_is_not_there_yet() {
    let (fixture, repo) = edited("long-runtime-relative-part");
    let instance: String = instance_of(repo.join(".git").as_path());
    let (holder, runtime): (PathBuf, PathBuf) = runtime_directory_spelled_through_a_missing_directory(&instance, "relative-part");
    assert!(!runtime.exists(), "the spelling's directory is not there when the first commit starts");
    // The stand-in manager's bus stands in a directory of its own, because a bus inside the spelling would make the
    // directory the commit has not made yet.
    let bus_directory: PathBuf = super::support::short_runtime_directory(&fixture);
    let services: Arc<Mutex<Vec<Child>>> = Arc::new(Mutex::new(Vec::new()));
    let start: Box<dyn Fn() + Send> = {
        // What the stand-in manager's start needs of its own: the spelling it starts a service at, and the instance.
        let started_at: PathBuf = runtime.clone();
        let started_for: String = instance.clone();
        let journal: PathBuf = fixture.root.join("journal");
        let home: PathBuf = fixture.root.join("home");
        let started: Arc<Mutex<Vec<Child>>> = Arc::clone(&services);
        Box::new(move || {
            let address: Bind = socket_address_of(started_at.as_path(), started_for.as_str());
            // A manager asked to start a unit that already runs starts nothing: the stand-in asks for a service only where
            // one is not answering.
            if git_policy_cli::service_address_of_a_worktree::the_address_is_there(&address) {
                return;
            }
            let mut command: Command = super::support::service_command();
            command.args(["--journal"]).arg(journal.to_str().expect("a UTF-8 fixture path"));
            match &address {
                Bind::Path(path) => {
                    command.arg("--path").arg(path);
                }
                Bind::Abstract(name) => {
                    command.arg("--abstract").arg(name);
                }
                Bind::Inherited => panic!("the stand-in manager names the address it starts a service at"),
            }
            command
                .args(["--instance", started_for.as_str()])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", home.as_path())
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit());
            let child: Child = command.spawn().expect("the service the stand-in manager starts");
            started.lock().expect("the started services").push(child);
        })
    };
    let bus = super::service_provision::fake_bus(bus_directory.as_path(), false, start);
    let mut command = wrapped_commit(&fixture, &repo, &["commit", "--message", SUBJECT, "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime.as_path());
    command.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", bus_directory.join("bus").display()));
    let output: Output = command.output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    let asked: Vec<String> = bus.join().expect("the stand-in bus answers");
    assert_eq!(output.status.code(), Some(0), "the commit was made: {stderr}\nthe manager was asked {asked:?}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), SUBJECT, "the commit was made");
    // A listener holds the address this spelling names: the place it resolves to, which is also the name the client made
    // from the spelling before the directory existed.
    assert!(
        git_policy_cli::service_address_of_a_worktree::the_address_is_there(&socket_address_of(runtime.as_path(), instance.as_str())),
        "the service answers at the address this spelling names: {}",
        runtime.display()
    );
    for service in services.lock().expect("the started services").iter_mut() {
        let _killed = service.kill();
        let _waited = service.wait();
    }
    let _removed = std::fs::remove_dir_all(&holder);
    remove_settled(&fixture);
}

/// What:
///  A runtime directory spelled through a directory that is not there yet,
/// and the holder of that spelling,
/// long enough that no socket path carries its socket.
/// Why:
///   A runtime directory of forty bytes leaves no room for the socket's name,
/// so the address is an abstract name;
/// the plain spelling is what the other controls use, and this one is the spelling the file system has to fold.
fn runtime_directory_spelled_through_a_missing_directory(instance: &str, tag: &str) -> (PathBuf, PathBuf) {
    let base: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    std::fs::create_dir_all(&base).expect("the scratch directory");
    // `<holder>` exists; `<holder>/<name>/missing` does not, and `..` steps back over it.
    let holder: PathBuf = base.join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&holder).expect("the holder of the spelling");
    let spelled: PathBuf = holder.join("name").join("missing").join("..").join("run");
    assert!(
        spelled.as_os_str().len() > longest_runtime_directory(instance),
        "the spelling is long enough for a name: {} bytes",
        spelled.as_os_str().len()
    );
    return (holder, spelled);
}

//! Controls for the route decision:
//! a runtime directory, the worktree's Git directory and the wrapper's own executable
//!  are the whole of it,
//! and nothing here needs a session bus or a service manager.

use super::*;
use crate::service_instance::socket_for;
use crate::test_support::{fixture, remove};

const GIT: &str = "/usr/bin/git";

struct Machine {
    root: PathBuf,
    runtime: PathBuf,
    wrapper: PathBuf,
    /// The worktree's own Git directory.
    git_dir: PathBuf,
}

fn machine(name: &str) -> Machine {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = root.join("run");
    let programs: PathBuf = root.join("programs");
    std::fs::create_dir_all(&runtime).expect("runtime");
    std::fs::create_dir_all(&programs).expect("programs");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    return Machine { wrapper: programs.join("git"), runtime, root, git_dir };
}

fn inputs<'a>(machine: &'a Machine) -> RouteInputs<'a> {
    return RouteInputs {
        runtime_dir: Some(machine.runtime.as_path()),
        state_home: Some(machine.root.as_path()),
        named_runtime_dir: Some(machine.runtime.as_path()),
        bus_address: None,
        home: Some(machine.root.as_path()),
        own_executable: Some(machine.wrapper.as_path()),
        real_git: Path::new(GIT),
        git_dir: machine.git_dir.as_path(),
    };
}

fn add_program(machine: &Machine) -> PathBuf {
    // The service is a mode of the wrapper's own executable, so the wrapper is the program the unit names.
    std::fs::write(&machine.wrapper, b"program").expect("the wrapper's own executable");
    return machine.wrapper.clone();
}

/// What: A runtime directory, a worktree and the wrapper's own executable are a route, with what starting needs.
/// Why:  The wrapper starts the server itself, so no bus and no unit take part in the decision.
#[test]
fn the_program_and_the_runtime_directory_mean_a_route() {
    let machine: Machine = machine("route-program");
    add_program(&machine);
    let route: Option<Route> = choose_route(&inputs(&machine));
    let Some(Route { address, host }) = route else { panic!("a route expected") };
    assert_eq!(address, Bind::Path(socket_for(machine.runtime.as_path(), machine.git_dir.as_path())));
    let named = host.expect("what starting needs comes with the route");
    assert_eq!(named.instance, crate::service_instance::instance_of(machine.git_dir.as_path()));
    assert_eq!(named.journal(), crate::service_journal_folder_of_an_instance::journal_of(&machine.root.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current), named.instance.as_str()));
    remove(machine.root.as_path());
}

/// What: Without an executable, an existing socket is still used as it is, and otherwise nothing.
/// Why:  A socket another program made is reachable; with neither the socket nor a program there is nothing to reach.
#[test]
fn an_existing_socket_is_used_without_a_program() {
    let machine: Machine = machine("route-socket");
    let socket: PathBuf = socket_for(machine.runtime.as_path(), machine.git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("the socket's folder")).expect("socket folder");
    std::fs::write(socket.as_path(), b"").expect("a socket file");
    assert_eq!(choose_route(&inputs(&machine)), Some(Route { address: Bind::Path(socket.clone()), host: None }));
    std::fs::remove_file(socket.as_path()).expect("the socket file goes");
    assert_eq!(choose_route(&inputs(&machine)), None, "neither a socket nor a program is no route");
    remove(machine.root.as_path());
}

/// What: No runtime directory is no route at all.
/// Why:  The address, the log and the notes live there, and a relative or absent folder names no place.
#[test]
fn no_runtime_directory_is_no_route() {
    let machine: Machine = machine("route-no-runtime");
    add_program(&machine);
    let mut route_inputs: RouteInputs<'_> = inputs(&machine);
    route_inputs.runtime_dir = None;
    assert_eq!(choose_route(&route_inputs), None);
    remove(machine.root.as_path());
}

/// Each worktree has its own socket and its own instance,
/// so two worktrees of one repository are never routed to one service,
///  and one worktree is routed to the same one however its commit is started.
#[test]
fn two_worktrees_are_routed_to_two_services() {
    let machine: Machine = machine("route-two-worktrees");
    add_program(&machine);
    let linked: PathBuf = machine.git_dir.join("worktrees/linked");
    std::fs::create_dir_all(&linked).expect("linked worktree's git directory");
    let first = choose_route(&inputs(&machine));
    let second = choose_route(&RouteInputs { git_dir: linked.as_path(), ..inputs(&machine) });
    let Some(Route { address: first_socket, host: Some(first_host) }) = first.clone() else { panic!("a route expected") };
    let Some(Route { address: second_socket, host: Some(second_host) }) = second else { panic!("a route expected") };
    assert_ne!(first_socket, second_socket, "two worktrees, two sockets");
    assert_ne!(first_host.instance, second_host.instance, "two worktrees, two unit instances");
    assert_eq!(first, choose_route(&inputs(&machine)), "one worktree, one route");
    remove(machine.root.as_path());
}

/// A Git directory that cannot be resolved has no route even when the runtime directory
/// and the program are both there:
/// no socket path or unit instance is named for it,
/// and nothing is created under the runtime directory.
#[test]
fn an_unreachable_git_directory_has_no_route_and_names_nothing() {
    let machine: Machine = machine("route-unreachable");
    add_program(&machine);
    let link: PathBuf = machine.root.join("dangling");
    std::os::unix::fs::symlink(machine.root.join("no-such-target"), &link).expect("dangling link");
    let git_dir: PathBuf = link.join(".git");
    let before: Vec<PathBuf> = entries_of(machine.runtime.as_path());
    let mut route_inputs: RouteInputs<'_> = inputs(&machine);
    route_inputs.git_dir = git_dir.as_path();
    assert_eq!(choose_route(&route_inputs), None);
    assert_eq!(entries_of(machine.runtime.as_path()), before, "nothing was created");
    remove(machine.root.as_path());
}

/// What:
///  The worktree's Git directory asked of Git is the bytes Git printed,
///  so the service instance has a name for a repository whose directory is not UTF-8.
/// Why:
///   The instance is named from that path,
///  and a spelling of it named a directory that does not exist,
///  so the commit was refused as "the Git directory cannot be resolved".
#[test]
fn the_worktree_git_directory_keeps_a_path_that_is_not_text() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("route-git-dir-not-utf8");
    let repo: PathBuf = root.join(std::ffi::OsString::from_vec(b"rep\xefo".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    crate::test_support::git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    // Global options name the repository, so the in-process reader leaves the answer to Git.
    let context: GitContext = GitContext {
        real_git: PathBuf::from(GIT),
        overlay: crate::test_support::isolated_overlay(),
        global_prefix: vec![std::ffi::OsString::from("--git-dir"), repo.join(".git").into_os_string()],
        common_dir: None,
    };
    assert_eq!(
        worktree_git_dir(&context, root.as_path()),
        Some(repo.join(".git")),
        "the Git directory keeps the bytes Git printed"
    );
    remove(root.as_path());
}

/// What:
///  The worktree's Git directory asked of Git keeps a trailing space or line feed that is part of the directory's name.
/// Why:
///   An answer that is a pathname loses only Git's one terminator,
///  so taking every ASCII whitespace byte after it read a directory named `metadata `,
///  or one whose name ends in a line feed,
///  as `metadata`,
///  the route then named a directory that does not exist,
/// and the commit had no instance to hand its work to.
#[test]
fn the_worktree_git_directory_keeps_a_trailing_space_or_line_feed() {
    let root: PathBuf = fixture("route-git-dir-trailing-whitespace");
    for (label, name) in [("space", "metadata "), ("line-feed", "metadata\n")] {
        let worktree: PathBuf = root.join(format!("worktree-{label}"));
        std::fs::create_dir(worktree.as_path()).expect("worktree directory");
        let metadata: PathBuf = root.join(name);
        crate::test_support::git(
            root.as_path(),
            &[
                "init",
                "--quiet",
                "--initial-branch=main",
                "--separate-git-dir",
                metadata.to_str().expect("UTF-8 fixture name"),
                worktree.to_str().expect("UTF-8 fixture path"),
            ],
        );
        let context: GitContext = GitContext {
            real_git: PathBuf::from(GIT),
            overlay: crate::test_support::isolated_overlay(),
            global_prefix: vec![std::ffi::OsString::from("--git-dir"), metadata.clone().into_os_string()],
            common_dir: None,
        };
        // Git answers the real path of a directory that `--separate-git-dir` named.
        assert_eq!(
            worktree_git_dir(&context, root.as_path()),
            Some(std::fs::canonicalize(metadata.as_path()).expect("the directory Git made")),
            "{label}: the Git directory keeps every byte of its name"
        );
    }
    remove(root.as_path());
}

/// The status of a worktree whose Git directory cannot be resolved says the cause,
/// and does not connect to any socket.
#[test]
fn the_status_of_an_unreachable_git_directory_says_the_cause() {
    let machine: Machine = machine("route-status-unreachable");
    let git_dir: PathBuf = machine.root.join("absent/.git");
    let report: StatusReport = service_status(Some(machine.runtime.as_path()), Some(git_dir.as_path()));
    assert_eq!(report.code, 1);
    assert!(report.stderr.contains(git_dir.to_string_lossy().as_ref()), "{}", report.stderr);
    assert!(report.stderr.contains("cannot be resolved"), "{}", report.stderr);
    assert!(!report.stderr.contains("sets itself up"), "{}", report.stderr);
    remove(machine.root.as_path());
}

/// The sorted entries of a folder.
fn entries_of(folder: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(folder).expect("folder").flatten().map(|entry| return entry.path()).collect();
    found.sort();
    return found;
}

/// What:
///  `service status` says a service that takes a connection and greets nothing does not answer.
/// Why:
///  A connection that the kernel queues is not an answer; the service would never greet it. This is the review's related
///  measurement (`debug-stop-fill-log.txt`), where a stopped service was reported "running and answers".
#[test]
fn the_status_of_a_service_that_queues_but_never_greets_says_it_does_not_answer() {
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("stqg");
    let git_dir: std::path::PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    let socket: std::path::PathBuf = socket_for(root.as_path(), git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("the folder of the address")).expect("socket folder");
    // A processor: it accepts and says nothing, which is a service stopped at an instruction.
    let listening = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a listening socket");
    let holding = std::thread::spawn(move || {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listening.accept() {
            held.push(stream);
            if held.len() > 4 {
                break;
            }
        }
    });
    let report: StatusReport = service_status(Some(root.as_path()), Some(git_dir.as_path()));
    assert_eq!(report.code, 1, "{}", report.stderr);
    assert!(!report.stdout.contains("answers"), "a service that greets nothing does not answer: {}", report.stdout);
    std::mem::forget(holding);
    crate::test_support::remove(root.as_path());
}

/// What:
///  `service status` says a service whose queue is full does not answer,
///  and says it within its bound.
/// Why:
///  A queued connection was read as an answer, so a service stopped at an instruction was reported "running and answers"
/// (the review of `3d5c1a0d8c`, reproduced). The answer is a greeting, not a connection.
#[test]
fn the_status_of_a_service_whose_queue_is_full_says_it_does_not_answer() {
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("stfq");
    let git_dir: std::path::PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    let socket: std::path::PathBuf = socket_for(root.as_path(), git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("the folder of the address")).expect("socket folder");
    let listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a listening socket");
    let queued = crate::test_support::fill_a_socket_backlog(socket.as_path());
    assert!(!queued.is_empty(), "the kernel queued at least one connection");
    let began = std::time::Instant::now();
    let report: StatusReport = service_status(Some(root.as_path()), Some(git_dir.as_path()));
    let took = began.elapsed();
    assert_eq!(report.code, 1, "{}", report.stderr);
    assert!(!report.stdout.contains("answers"), "a service that takes no connection does not answer: {}", report.stdout);
    assert!(took < crate::wait_ceilings::CONNECT_CEILING + std::time::Duration::from_secs(2), "and the answer comes at its bound: {took:?}");
    drop(queued);
    drop(listener);
    crate::test_support::remove(root.as_path());
}

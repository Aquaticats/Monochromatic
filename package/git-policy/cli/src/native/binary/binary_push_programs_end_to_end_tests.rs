//! What:
//!  The carry of a push's programs,
//! end to end in a real commit service:
//!  a stand-in `git-lfs` in a folder only the committing shell's `PATH` names is run by the service's own push,
//!  a push with no stand-in on that list is refused and the refusal names the shell's list,
//!  a push whose settings a restart took away stops and the command it prints lands the commit,
//!  and the five names the service's own environment held are unset for a push that carries a list.
//! Why:
//!   The unit controls prove the list reaches a child's environment;
//! only a real service,
//! a real Git and a push that carries an object prove that the service's push,
//!  which nothing of the committing command survives,
//! searches that list
//! (`service_push_programs_held_in_memory`).
//! The stand-in is the instrument:
//! it writes whether the push's environment held the request marker,
//! the service marker
//! and the fake values,
//! as classifications only,
//! so the sweep for a written value cannot find it in the instrument itself.
//! A folder the stand-in lives in,
//! or the shell-only folder the refusal's words name,
//! is outside the fixture root for that
//! reason.
//!  Each control needs an image whose own program folders hold no `git-lfs`,
//! because a real one answers the push before the
//!  stand-in and the control would prove nothing;
//! such an image is acknowledged with an environment variable (`CLI_GIT_END_TO_END_LFS_OCCUPIED_ACKNOWLEDGED`),
//! and the line that says the control did not run names it.
//! Every carried value is plainly fake.

use super::commit::{keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, direct, executable, fixture, git, give_service_environment, observe_raw, repository, start_fixture_service, stderr_of, FixtureService,
};
use git_policy_cli::service_fault::{AFTER_PROMOTE, FAULT_VARIABLE};
use std::ffi::{OsStr, OsString};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PATIENCE: Duration = Duration::from_secs(120);
const FAKE_SSH_COMMAND: &str = "ssh -i /nowhere/FAKE_SSH_COMMAND_KEY_71c3";
const FAKE_PROXY: &str = "nc -x fakeuser:FAKE_PROXY_PASSWORD_2a6f proxy.invalid 1080";
const FAKE_GIT_SSH: &str = "/nowhere/FAKE_GIT_SSH_PROGRAM_c4f1";
const SHELL_FOLDER_NAME: &str = "FAKE_SHELL_ONLY_FOLDER_5e2a";
const EXEC_ALIAS_NAME: &str = "FAKE_EXEC_ALIAS_8b21";
const RED_FOLDER_NAME: &str = "FAKE_RED_ARM_FOLDER_9d41";

struct World {
    fixture: Fixture,
    repo: PathBuf,
    origin: PathBuf,
    service: FixtureService,
    shell_folder: PathBuf,
    exec_alias: PathBuf,
    red_folder: PathBuf,
    record: PathBuf,
    journal_name: String,
    journal: Arc<Mutex<Vec<u8>>>,
}

fn capture_journal(name: &str) -> Arc<Mutex<Vec<u8>>> {
    let address: SocketAddr = SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract name");
    let socket: UnixDatagram = UnixDatagram::bind_addr(&address).expect("the journal name binds");
    socket.set_read_timeout(Some(Duration::from_millis(200))).expect("a read timeout");
    let received: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let sink: Arc<Mutex<Vec<u8>>> = Arc::clone(&received);
    std::thread::spawn(move || {
        let mut buffer: Vec<u8> = vec![0; 1 << 16];
        loop {
            match socket.recv(buffer.as_mut_slice()) {
                Ok(length) => sink.lock().expect("the journal sink").extend_from_slice(&buffer[..length]),
                Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                    if Arc::strong_count(&sink) == 1 {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    });
    return received;
}

fn stand_in_source(record: &Path) -> String {
    return format!(
        "#!/bin/sh\n\
         {{\n\
         printf 'ran first=%s\\n' \"$1\"\n\
         printf 'request-mark=%s\\n' \"${{MONOCHROMATIC_PUSH_PROGRAMS_OF_THE_REQUEST:+set}}\"\n\
         printf 'service-mark=%s\\n' \"${{MONOCHROMATIC_PUSH_RUNS_IN_THE_COMMIT_SERVICE:+set}}\"\n\
         printf 'service-list-mark=%s\\n' \"${{MONOCHROMATIC_PUSH_PROGRAMS_OF_THE_SERVICE:+set}}\"\n\
         case \"$GIT_SSH_COMMAND\" in\n\
         '{FAKE_SSH_COMMAND}') printf 'ssh-command=the-fake\\n' ;;\n\
         '') printf 'ssh-command=unset\\n' ;;\n\
         *) printf 'ssh-command=another\\n' ;;\n\
         esac\n\
         case \"$GIT_PROXY_COMMAND\" in\n\
         '{FAKE_PROXY}') printf 'proxy-command=the-fake\\n' ;;\n\
         '') printf 'proxy-command=unset\\n' ;;\n\
         *) printf 'proxy-command=another\\n' ;;\n\
         esac\n\
         case \"$GIT_SSH\" in\n\
         '{FAKE_GIT_SSH}') printf 'git-ssh=the-fake\\n' ;;\n\
         '') printf 'git-ssh=unset\\n' ;;\n\
         *) printf 'git-ssh=another\\n' ;;\n\
         esac\n\
         case \":$PATH:\" in\n\
         *\":${{0%/*}}:\"*) printf 'path-has-the-shell-folder=yes\\n' ;;\n\
         *) printf 'path-has-the-shell-folder=no\\n' ;;\n\
         esac\n\
         }} >> '{record}'\n\
         exit 0\n",
        record = record.display(),
    );
}

/// The variable that acknowledges an image whose own program folders hold a real `git-lfs`,
/// where these controls cannot run and say so.
const OCCUPIED_ACKNOWLEDGED: &str = "CLI_GIT_END_TO_END_LFS_OCCUPIED_ACKNOWLEDGED";

/// What:
///  Whether a real `git-lfs` sits in a folder the service's own list searches,
///  and which one.
/// Why:
///   The controls prove the carry with a stand-in the service's own list cannot find;
/// a real program there answers the push
///  first,
/// so the stand-in never runs and a green control would prove nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shadowedBy = () => ['/usr/bin', '/bin', '/usr/local/bin'].find((folder) => existsSync(join(folder, 'git-lfs')));
/// ```
fn shadowed_by() -> Option<&'static str> {
    for folder in ["/usr/bin", "/bin", "/usr/local/bin"] {
        if Path::new(folder).join("git-lfs").exists() {
            return Some(folder);
        }
    }
    return None;
}

fn world(name: &str) -> Option<World> {
    return world_with(name, false, &[]);
}

/// A world whose service may hold a list of its own that reaches the stand-in too,
/// and variables of its own,
/// or nothing where a real `git-lfs` would answer the push before the stand-in.
fn world_with(name: &str, stand_in_on_the_services_path: bool, service_variables: &[(&str, &str)]) -> Option<World> {
    if let Some(folder) = shadowed_by() {
        let acknowledged: bool = std::env::var_os(OCCUPIED_ACKNOWLEDGED).is_some_and(|value| { return !value.is_empty(); });
        assert!(
            acknowledged,
            "{name} needs an image whose own program folders hold no git-lfs, and {folder} holds one; run it in the gate image, or set {OCCUPIED_ACKNOWLEDGED}=1"
        );
        eprintln!("END TO END CARRY CONTROL NOT RUN: {name} ({folder} holds a real git-lfs, which answers before the stand-in)");
        return None;
    }
    let fixture: Fixture = fixture(name);
    // Outside the fixture root and the runtime directory: the stand-in's own text holds the fake values it classifies, so a
    //  sweep that reached it would find them there.
    let shell_folder: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent/git-policy-push-programs-end-to-end-stand-in-git-lfs").join(name).join(SHELL_FOLDER_NAME);
    if stand_in_on_the_services_path {
        give_service_environment(&fixture, "PATH", format!("/usr/bin:/bin:{}", shell_folder.display()).as_str());
    } else {
        give_service_environment(&fixture, "PATH", "/usr/bin:/bin");
    }
    for (variable, value) in service_variables {
        give_service_environment(&fixture, variable, value);
    }
    let journal_name: String = format!("end-to-end-journal-{name}-{}", std::process::id());
    let journal: Arc<Mutex<Vec<u8>>> = capture_journal(journal_name.as_str());
    give_service_environment(&fixture, git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{journal_name}").as_str());
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", origin.to_str().expect("text")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    git(&fixture, &repo, &["remote", "add", "origin", origin.to_str().expect("text")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    // The wrapper uploads before a push only for a repository whose own configuration holds an LFS key.
    git(&fixture, &repo, &["config", "lfs.repositoryformatversion", "0"]);
    let record: PathBuf = fixture.root.join("lfs-saw.txt");
    std::fs::create_dir_all(shell_folder.as_path()).expect("the shell-only folder");
    // A folder on the shell's list of the arm that has no stand-in: it holds no `git-lfs`, and its name is the marker a sweep
    //  looks for, so the list can be followed to every place a refusal that names it writes it.
    let red_folder: PathBuf = shell_folder.parent().expect("the stand-in's parent").join(RED_FOLDER_NAME);
    std::fs::create_dir_all(red_folder.as_path()).expect("the red arm's folder");
    executable(shell_folder.join("git-lfs").as_path(), stand_in_source(record.as_path()).as_bytes());
    let exec_path: String = String::from_utf8_lossy(direct(&fixture).arg("--exec-path").output().expect("real Git answers").stdout.as_slice()).trim().to_owned();
    let exec_alias: PathBuf = fixture.root.join(EXEC_ALIAS_NAME);
    std::os::unix::fs::symlink(exec_path.as_str(), exec_alias.as_path()).expect("the exec path alias");
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Some(World { fixture, repo, origin, service, shell_folder, exec_alias, red_folder, record, journal_name, journal });
}

fn shell_path(world: &World, with_stand_in: bool) -> OsString {
    let mut path: OsString = world.fixture.root.join("bin").into_os_string();
    if with_stand_in {
        path.push(":");
        path.push(world.shell_folder.as_path());
    } else {
        path.push(":");
        path.push(world.red_folder.as_path());
    }
    path.push(":/usr/bin:/bin");
    return path;
}

fn shell(world: &World, arguments: &[&str], with_stand_in: bool, transport: bool) -> Command {
    let mut command: Command = wrapped_commit(&world.fixture, world.repo.as_path(), arguments);
    command.env("PATH", shell_path(world, with_stand_in));
    command.env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{}", world.journal_name));
    command.env("GIT_EXEC_PATH", world.exec_alias.as_path());
    if transport {
        command.env("GIT_SSH_COMMAND", FAKE_SSH_COMMAND);
        command.env("GIT_PROXY_COMMAND", FAKE_PROXY);
        command.env("GIT_SSH", FAKE_GIT_SSH);
    }
    return command;
}

fn commit_a_pointer(world: &World, name: &str, with_stand_in: bool, transport: bool, fault: Option<&str>) -> std::process::Output {
    let content: Vec<u8> = format!("{name} LFS content\n").into_bytes();
    let digest: [u8; 32] = git_policy_cli::subvolume_sha::sha256(content.as_slice());
    let oid: String = git_policy_cli::hexadecimal_of_bytes::hexadecimal_of_bytes(digest.as_slice());
    let pointer: String = format!("version https://git-lfs.github.com/spec/v1\noid sha256:{oid}\nsize {}\n", content.len());
    std::fs::write(world.repo.join(name), pointer.as_bytes()).expect("pointer file");
    let storage: PathBuf = world.repo.join(".git/lfs/objects").join(&oid[..2]).join(&oid[2..4]);
    std::fs::create_dir_all(&storage).expect("LFS object directory");
    std::fs::write(storage.join(oid.as_str()), content.as_slice()).expect("LFS object");
    let added: Observed = observe_raw(&mut shell(world, &["add", "--", name], with_stand_in, transport), b"");
    assert_eq!(added.code, Some(0), "the add: {}", stderr_of(&added));
    let message: String = format!("--message={name}");
    let mut committing: Command = shell(world, &["commit", "--quiet", message.as_str(), "--", name], with_stand_in, transport);
    if let Some(point) = fault {
        committing.env(FAULT_VARIABLE, point);
    }
    return committing.output().expect("the wrapped commit runs");
}

fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn files_under(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if !kind.is_file() {
                continue;
            }
            found.push((path.clone(), std::fs::read(path.as_path()).unwrap_or_default()));
        }
    }
    return found;
}

/// Every file under the fixture root and the runtime directory,
/// and the journal bytes,
/// holding `value`.
fn holding(world: &World, value: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for root in [world.fixture.root.clone(), world.service.runtime.clone()] {
        for (path, bytes) in files_under(root.as_path()) {
            if bytes.windows(value.len()).any(|window| { return window == value.as_bytes(); }) {
                found.push(path.display().to_string());
            }
        }
    }
    if world.journal.lock().expect("the journal").windows(value.len()).any(|window| { return window == value.as_bytes(); }) {
        found.push(String::from("<the journal the service and the commands sent>"));
    }
    return found;
}

fn origin_main(world: &World) -> String {
    let output: std::process::Output = direct(&world.fixture).current_dir(world.origin.as_path()).args(["rev-parse", "--verify", "--quiet", "refs/heads/main"]).output().expect("real Git answers");
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
}

fn head_of(world: &World) -> String {
    return String::from_utf8_lossy(git(&world.fixture, world.repo.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice()).trim().to_owned();
}

fn saw(world: &World) -> String {
    return std::fs::read_to_string(world.record.as_path()).unwrap_or_default();
}

/// What the stand-in wrote,
/// as runs:
/// one string per run,
/// its lines joined by a space.
fn runs(world: &World) -> Vec<String> {
    let mut runs: Vec<String> = Vec::new();
    for line in saw(world).lines() {
        if line.starts_with("ran first=") {
            runs.push(String::from(line));
        } else if let Some(run) = runs.last_mut() {
            run.push(' ');
            run.push_str(line);
        }
    }
    return runs;
}

/// Every value the committing shell set,
/// on any disk or in the journal.
fn assert_nothing_written(world: &World) {
    // The positive control first: the same search must find a value that is on a disk.
    let planted: PathBuf = world.fixture.root.join("planted-value.txt");
    std::fs::write(planted.as_path(), format!("{FAKE_SSH_COMMAND}\n")).expect("a planted value");
    assert_eq!(holding(world, FAKE_SSH_COMMAND), vec![planted.display().to_string()], "the search finds a value that is on the disk");
    std::fs::remove_file(planted.as_path()).expect("the planted value goes");
    let planted_in_the_journal: bool = {
        let socket: UnixDatagram = UnixDatagram::unbound().expect("a datagram socket");
        let address: SocketAddr = SocketAddr::from_abstract_name(world.journal_name.as_bytes()).expect("the journal name");
        socket.send_to_addr(format!("PLANTED={FAKE_PROXY}\n").as_bytes(), &address).expect("the plant is sent");
        wait_until("the planted journal line arrives", &|| { return holding(world, FAKE_PROXY).iter().any(|place| { return place.starts_with("<the journal"); }); });
        true
    };
    assert!(planted_in_the_journal);
    for (name, value) in [
        ("GIT_SSH_COMMAND", FAKE_SSH_COMMAND),
        ("GIT_SSH", FAKE_GIT_SSH),
        ("PATH (its shell-only folder)", SHELL_FOLDER_NAME),
        ("GIT_EXEC_PATH (its alias)", EXEC_ALIAS_NAME),
    ] {
        assert!(holding(world, value).is_empty(), "{name} reaches no file or journal line: {:?}", holding(world, value));
    }
    // The proxy command is searched after the plant, whose journal line is the one place it may be.
    let proxy_places: Vec<String> = holding(world, FAKE_PROXY);
    assert_eq!(proxy_places, vec![String::from("<the journal the service and the commands sent>")], "GIT_PROXY_COMMAND is only where the plant put it: {proxy_places:?}");
}

#[test]
fn green_the_services_push_searches_the_committing_shells_list_and_runs_its_stand_in() {
    let Some(world): Option<World> = world("push-programs-end-to-end-green") else { return };
    let committed: std::process::Output = commit_a_pointer(&world, "one.bin", true, true, None);
    assert!(committed.status.success(), "the commit lands: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let commit: String = head_of(&world);
    wait_until("the service's push publishes the pointer commit", &|| { return origin_main(&world) == commit; });
    let runs: Vec<String> = runs(&world);
    assert!(!runs.is_empty(), "the stand-in ran: the carried list reached the service's push");
    assert!(
        runs.iter().any(|run| { return run.contains("request-mark=set") && run.contains("service-mark=set") && run.contains("path-has-the-shell-folder=yes"); }),
        "a run made by the service's push carried the request mark, the service mark and the shell's folder: {runs:?}"
    );
    assert!(
        runs.iter().any(|run| { return run.contains("ssh-command=the-fake") && run.contains("proxy-command=the-fake") && run.contains("git-ssh=the-fake"); }),
        "the three transport values reached the stand-in, as classifications: {runs:?}"
    );
    // The service's own entries go to its standard error where it is not under the journal (the fixture's), and that stream is a
    //  file under the fixture root: the sweep reaches it only if it holds what the service said, so the line the service says
    //  for this push is the positive control that the log is in the sweep.
    wait_until("the service's log holds the line it says for the push", &|| { return !holding(&world, "pushed branch=").is_empty(); });
    assert_nothing_written(&world);
    world.service.stop_all();
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.shell_folder.parent().expect("the stand-in's parent")).expect("the stand-in's own folder goes");
}

#[test]
fn red_with_no_stand_in_on_the_shells_list_the_services_push_is_refused_naming_the_shells_list() {
    let Some(world): Option<World> = world("push-programs-end-to-end-red") else { return };
    let committed: std::process::Output = commit_a_pointer(&world, "one.bin", false, true, None);
    assert!(committed.status.success(), "the commit lands: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let commit: String = head_of(&world);
    let notice: PathBuf = git_policy_cli::push_failure_notice::notice_directory(world.repo.join(".git").as_path());
    wait_until("the push records what stopped it", &|| { return std::fs::read_dir(notice.as_path()).is_ok_and(|entries| { return entries.flatten().next().is_some(); }); });
    // Before the next command takes the notice: the failure notice, the retry plan and the service's log are all on disk now.
    // What: The one value a refusal writes is the folder list it searched, and it reaches the person's notice and the service's
    //  own log alone.
    // Why:  The note a person reads must name the list whose program is missing, and the documentation states that this is the
    //  only value that reaches a file (`doc/troubleshooting/git-policy-cli-commit-service-cannot-find-the-shells-programs.md`).
    let marker_places: Vec<String> = holding(&world, RED_FOLDER_NAME);
    assert!(
        marker_places.iter().any(|place| { return place.contains("notices"); }),
        "the refusal's own record names the shell's folder: {marker_places:?}"
    );
    assert!(
        marker_places.iter().all(|place| { return !place.ends_with("config") && !place.contains("work-items") && !place.contains("/lfs/"); }),
        "the folder list reaches no configuration, work record or object store: {marker_places:?}"
    );
    for (name, value) in [("GIT_SSH_COMMAND", FAKE_SSH_COMMAND), ("GIT_SSH", FAKE_GIT_SSH), ("GIT_EXEC_PATH (its alias)", EXEC_ALIAS_NAME)] {
        assert!(holding(&world, value).is_empty(), "after a refusal, {name} reaches no file or journal line: {:?}", holding(&world, value));
    }
    let proxy_places: Vec<String> = holding(&world, "FAKE_PROXY_PASSWORD_2a6f");
    assert!(proxy_places.is_empty(), "after a refusal, GIT_PROXY_COMMAND reaches no file or journal line: {proxy_places:?}");
    let shown: Observed = observe_raw(&mut shell(&world, &["status"], false, true), b"");
    let words: String = stderr_of(&shown);
    assert_ne!(origin_main(&world), commit, "nothing reached the remote");
    assert!(runs(&world).is_empty(), "the stand-in never ran");
    assert!(words.contains("was not found"), "the refusal says the program was not found: {words}");
    assert!(words.contains("`PATH` of the shell that asked for this push"), "and names the shell's list, which is what the service searched: {words}");
    world.service.stop_all();
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.shell_folder.parent().expect("the stand-in's parent")).expect("the stand-in's own folder goes");
}

#[test]
fn restart_a_push_of_a_pointer_with_no_transport_names_the_services_list_and_the_printed_command_runs() {
    let Some(world): Option<World> = world("push-programs-end-to-end-restart") else { return };
    let killed: std::process::Output = commit_a_pointer(&world, "one.bin", true, false, Some(AFTER_PROMOTE));
    assert!(!killed.status.success(), "the wrapper was killed at the fault point");
    let commit: String = head_of(&world);
    assert_ne!(origin_main(&world), commit, "the push has not been made");
    world.service.stop_all();
    world.service.serve(&world.fixture, world.repo.join(".git").as_path());
    let notice: PathBuf = git_policy_cli::push_failure_notice::notice_directory(world.repo.join(".git").as_path());
    wait_until("the push records what stopped it", &|| { return std::fs::read_dir(notice.as_path()).is_ok_and(|entries| { return entries.flatten().next().is_some(); }); });
    let shown: Observed = observe_raw(&mut shell(&world, &["status"], true, false), b"");
    let words: String = stderr_of(&shown);
    assert_ne!(origin_main(&world), commit, "nothing reached the remote");
    assert!(words.contains("searched the commit service's own `PATH`"), "the refusal names the service's own list: {words}");
    assert!(words.contains("push by hand once from that shell (`git push`)"), "and the remedy is the shell's own push: {words}");
    assert!(runs(&world).is_empty(), "the stand-in never ran in the service");
    // The remedy the words name, run in the shell that holds the stand-in.
    let pushed: Observed = observe_raw(&mut shell(&world, &["push", "--quiet", "origin", "main"], true, false), b"");
    assert_eq!(pushed.code, Some(0), "the shell's own push goes through: {}", stderr_of(&pushed));
    assert_eq!(origin_main(&world), commit, "the commit is on the remote");
    let runs: Vec<String> = runs(&world);
    assert!(runs.iter().any(|run| { return run.contains("service-mark=") && !run.contains("service-mark=set") && run.contains("path-has-the-shell-folder=yes"); }), "the stand-in ran from the shell, not the service: {runs:?}");
    world.service.stop_all();
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.shell_folder.parent().expect("the stand-in's parent")).expect("the stand-in's own folder goes");
}

#[test]
fn a7_a_value_only_the_service_holds_is_unset_for_a_push_that_carries_a_list_and_is_seen_when_none_came() {
    const SERVICE_SSH_COMMAND: &str = "ssh -i /nowhere/FAKE_SERVICE_OWN_SSH_COMMAND_33d0";
    // The service holds its own transport, and its own list reaches the stand-in too, so the stand-in runs in both arms.
    let Some(world): Option<World> = world_with("push-programs-end-to-end-a7", true, &[("GIT_SSH_COMMAND", SERVICE_SSH_COMMAND)]) else { return };
    // Arm one: the command's list came, the command chose no transport, so the service's own transport is unset for the push.
    let committed: std::process::Output = commit_a_pointer(&world, "one.bin", true, false, None);
    assert!(committed.status.success(), "the commit lands: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let first: String = head_of(&world);
    wait_until("the service's push publishes the first pointer commit", &|| { return origin_main(&world) == first; });
    let carried: Vec<String> = runs(&world);
    assert!(carried.iter().any(|run| { return run.contains("request-mark=set") && run.contains("ssh-command=unset"); }), "the service's own GIT_SSH_COMMAND is not the push's: {carried:?}");
    // Arm two, the positive control: no list reaches the service (a restart), so its own transport is what the push has,
    //  and the same stand-in reports it as another command line than the shell's.
    let killed: std::process::Output = commit_a_pointer(&world, "two.bin", true, false, Some(AFTER_PROMOTE));
    assert!(!killed.status.success(), "the wrapper was killed at the fault point");
    let second: String = head_of(&world);
    world.service.stop_all();
    world.service.serve(&world.fixture, world.repo.join(".git").as_path());
    wait_until("the restarted service's push publishes the second pointer commit", &|| { return origin_main(&world) == second; });
    let all: Vec<String> = runs(&world);
    assert!(all.iter().any(|run| { return run.contains("request-mark=") && !run.contains("request-mark=set") && run.contains("service-list-mark=set") && run.contains("ssh-command=another"); }), "with no list the push has the service's own transport, so the instrument can see one: {all:?}");
    world.service.stop_all();
    remove_settled(&world.fixture);
    std::fs::remove_dir_all(world.shell_folder.parent().expect("the stand-in's parent")).expect("the stand-in's own folder goes");
}

//! Binary-level controls for the Git LFS upload against the real `git-lfs` and an LFS server written inline.
//! The server speaks the batch API and the basic transfer over a local socket and records,
//! for every uploaded object,
//!  what the origin's `main` was at that moment,
//! so the order "object first,
//! ref second" is observed and not assumed.
//! Hosts without `git-lfs` on PATH report that and check nothing;
//! the stand-in controls in
//!  `binary_lfs_pre_push_tests.rs` cover the same paths there.

use super::commit::remove_settled;
use super::support::{Fixture, Observed, REAL_GIT, decline_in_process_lfs_scan, direct, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// What the server saw of one uploaded object.
#[derive(Clone, Debug)]
struct Upload {
    /// The object name the client sent.
    oid: String,
    /// The bytes it sent.
    bytes: Vec<u8>,
    /// The origin's `refs/heads/main` when the object arrived,
    /// or `absent`.
    origin_main: String,
}

/// The server's shared record.
#[derive(Default)]
struct ServerState {
    /// Every stored object.
    uploads: Vec<Upload>,
    /// Whether the batch endpoint refuses,
    /// as a server that rejects the credentials would.
    refuse_batch: bool,
    /// Whether the batch endpoint asks for a login,
    /// as the Worker of the 2026-11-10 incident did:
    /// an HTTP 401 with the realm it wants.
    login_required: bool,
    /// How many of the next batch requests are answered with HTTP 503,
    /// as the incident's server did.
    fail_batches: usize,
    /// Every batch request received,
    /// answered or not.
    batches: usize,
}

/// A running LFS server on a loopback port.
struct LfsServer {
    /// The port it listens on.
    port: u16,
    /// What it has seen.
    state: Arc<Mutex<ServerState>>,
    /// Set to end the accept loop.
    stop: Arc<AtomicBool>,
    /// The accept loop.
    thread: Option<JoinHandle<()>>,
    /// The repository's hooks directory,
    /// made read-only so `git-lfs` cannot write its shims into it again;
    ///  restored when the server ends so the fixture can be removed.
    hooks: Option<PathBuf>,
}

/// The origin's `main` for the server's record.
fn origin_main(origin: &Path) -> String {
    let output = std::process::Command::new(REAL_GIT)
        .arg(format!("--git-dir={}", origin.display()))
        .args(["rev-parse", "--verify", "--quiet", "refs/heads/main"])
        .output()
        .expect("rev-parse starts");
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return if text.is_empty() { String::from("absent") } else { text };
}

/// Read one request:
/// its request line,
/// its headers' content length,
/// and its body.
fn read_request(stream: &TcpStream) -> Option<(String, Vec<u8>)> {
    let mut reader: BufReader<&TcpStream> = BufReader::new(stream);
    let mut request_line: String = String::new();
    reader.read_line(&mut request_line).ok()?;
    let mut length: usize = 0;
    loop {
        let mut line: String = String::new();
        reader.read_line(&mut line).ok()?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse().ok()?;
        }
    }
    let mut body: Vec<u8> = vec![0; length];
    reader.read_exact(body.as_mut_slice()).ok()?;
    return Some((request_line.trim_end().to_owned(), body));
}

/// Write one response with a body and close the connection.
fn respond(stream: &TcpStream, status: &str, body: &str) {
    return respond_with(stream, status, "", body);
}

/// Write one response with a body,
/// the headers `extra` adds
/// (each line terminated by `\r\n`)
/// and a closed connection.
fn respond_with(mut stream: &TcpStream, status: &str, extra: &str, body: &str) {
    let text: String = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/vnd.git-lfs+json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    // A client that went away is not a server failure.
    let _written = stream.write_all(text.as_bytes());
}

/// Answer one request of the batch API,
/// the lock-verification endpoint or the basic upload.
fn serve(stream: &TcpStream, port: u16, origin: &Path, state: &Mutex<ServerState>) {
    let Some((request_line, body)) = read_request(stream) else {
        return;
    };
    if request_line.starts_with("POST /lfs/objects/batch ") {
        {
            let mut seen = state.lock().expect("state");
            seen.batches += 1;
            if seen.fail_batches > 0 {
                seen.fail_batches -= 1;
                drop(seen);
                respond(stream, "503 Service Unavailable", "{}");
                return;
            }
        }
        if state.lock().expect("state").login_required {
            // The header a client reads as "send a login this way": the Worker's own answer (401 plus the realm).
            respond_with(
                stream,
                "401 Unauthorized",
                "LFS-Authenticate: Basic realm=\"monochromatic-lfs\"\r\n",
                "{\"message\":\"a login is needed\"}",
            );
            return;
        }
        if state.lock().expect("state").refuse_batch {
            respond(stream, "403 Forbidden", "{\"message\":\"the fixture server refuses every batch\"}");
            return;
        }
        let asked: serde_json::Value = serde_json::from_slice(body.as_slice()).unwrap_or_default();
        let known: Vec<String> = state.lock().expect("state").uploads.iter().map(|upload| { return upload.oid.clone(); }).collect();
        // Built as text: the objects only carry hexadecimal names and whole numbers.
        let objects: Vec<String> = asked["objects"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|object| {
                let oid: String = object["oid"].as_str().unwrap_or_default().to_owned();
                let size: u64 = object["size"].as_u64().unwrap_or_default();
                if known.contains(&oid) {
                    return format!("{{\"oid\":\"{oid}\",\"size\":{size}}}");
                }
                return format!(
                    "{{\"oid\":\"{oid}\",\"size\":{size},\"actions\":{{\"upload\":{{\"href\":\"http://127.0.0.1:{port}/lfs/objects/{oid}\"}}}}}}"
                );
            })
            .collect();
        respond(stream, "200 OK", format!("{{\"transfer\":\"basic\",\"objects\":[{}]}}", objects.join(",")).as_str());
    } else if request_line.starts_with("POST /lfs/locks/verify ") {
        respond(stream, "200 OK", "{\"ours\":[],\"theirs\":[]}");
    } else if let Some(rest) = request_line.strip_prefix("PUT /lfs/objects/") {
        let oid: String = rest.split(' ').next().unwrap_or_default().to_owned();
        let seen: String = origin_main(origin);
        state.lock().expect("state").uploads.push(Upload { oid, bytes: body, origin_main: seen });
        respond(stream, "200 OK", "{}");
    } else {
        respond(stream, "404 Not Found", "{\"message\":\"unknown endpoint\"}");
    }
}

impl LfsServer {
    /// Start the accept loop on an unused loopback port.
    fn start(origin: PathBuf) -> Self {
        let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port: u16 = listener.local_addr().expect("address").port();
        let state: Arc<Mutex<ServerState>> = Arc::new(Mutex::new(ServerState::default()));
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let thread_state: Arc<Mutex<ServerState>> = Arc::clone(&state);
        let thread_stop: Arc<AtomicBool> = Arc::clone(&stop);
        let thread: JoinHandle<()> = std::thread::spawn(move || {
            for connection in listener.incoming() {
                if thread_stop.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(stream) = connection else {
                    continue;
                };
                // A client that stalls must not hold the loop for good.
                let _timeout = stream.set_read_timeout(Some(Duration::from_secs(20)));
                serve(&stream, port, origin.as_path(), &thread_state);
            }
        });
        return Self { port, state, stop, thread: Some(thread), hooks: None };
    }

    /// The uploads so far.
    fn uploads(&self) -> Vec<Upload> {
        return self.state.lock().expect("state").uploads.clone();
    }
}

/// The server ends with its test:
/// the flag is set and a connection wakes the accept loop.
impl Drop for LfsServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(hooks) = self.hooks.as_deref() {
            // A directory that is already gone needs no restoring.
            let _restored = std::fs::set_permissions(hooks, std::fs::Permissions::from_mode(0o755));
        }
        // The wake-up connection only has to arrive; whether it is accepted does not matter.
        let _connected = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(thread) = self.thread.take() {
            // A panic in the loop already failed the test that owned the server.
            let _joined = thread.join();
        }
    }
}

/// The real `git-lfs` on the test process's PATH,
/// if any.
fn real_lfs_program() -> Option<PathBuf> {
    let path: std::ffi::OsString = std::env::var_os("PATH")?;
    return std::env::split_paths(&path)
        .map(|directory| { return directory.join("git-lfs"); })
        .find(|candidate| { return candidate.is_file(); });
}

/// A repository wired to the real `git-lfs` and the server,
/// with one committed LFS file;
/// `None` without `git-lfs`.
fn lfs_world(fixture: &Fixture) -> Option<(PathBuf, PathBuf, LfsServer, Vec<u8>)> {
    let Some(program) = real_lfs_program() else {
        eprintln!("skipped: git-lfs is not on PATH, so the real upload was not exercised");
        return None;
    };
    std::os::unix::fs::symlink(program, fixture.root.join("bin/git-lfs")).expect("git-lfs link");
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("UTF-8")]);
    let mut server: LfsServer = LfsServer::start(origin.clone());
    let settings: [(&str, String); 7] = [
        ("filter.lfs.clean", String::from("git-lfs clean -- %f")),
        ("filter.lfs.smudge", String::from("git-lfs smudge -- %f")),
        ("filter.lfs.process", String::from("git-lfs filter-process")),
        ("filter.lfs.required", String::from("true")),
        ("lfs.url", format!("http://127.0.0.1:{}/lfs", server.port)),
        ("lfs.transfer.maxretries", String::from("1")),
        ("maintenance.autoDetach", String::from("false")),
    ];
    for (key, value) in settings {
        git(fixture, repo.as_path(), &["config", key, value.as_str()]);
    }
    std::fs::write(repo.join(".gitattributes"), b"*.bin filter=lfs diff=lfs merge=lfs -text\n").expect("attributes");
    let content: Vec<u8> = (0..4096_u32).map(|index| { return u8::try_from(index % 251).expect("below 256"); }).collect();
    std::fs::write(repo.join("big.bin"), content.as_slice()).expect("big.bin");
    // `git add` runs the clean filter, which needs `git-lfs` on PATH; the direct helper's PATH has none.
    let added: Observed = observe(
        &mut super::commit::wrapped_commit(fixture, repo.as_path(), &["add", "--", ".gitattributes", "big.bin"]),
        b"",
    );
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    // Real Git commits, so no auto-push runs; its PATH names the fixture's `bin` so the filter finds `git-lfs`.
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    let committed = direct(fixture)
        .current_dir(repo.as_path())
        .env("PATH", path)
        .args(["commit", "--quiet", "--message=lfs"])
        .output()
        .expect("commit starts");
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(committed.stderr.as_slice()));
    // `git-lfs` writes its four hook shims into the repository whenever its filter runs, which would make the real
    //  program upload through a traditional hook and hide a missing wrapper hook. The shims go, and the directory
    //  stays read-only until the server is dropped, so the upload under test can only come from the wrapper.
    let hooks: PathBuf = repo.join(".git/hooks");
    for shim in ["pre-push", "post-checkout", "post-commit", "post-merge"] {
        // A shim that was never written needs no removal.
        let _removed = std::fs::remove_file(hooks.join(shim));
    }
    std::fs::set_permissions(&hooks, std::fs::Permissions::from_mode(0o555)).expect("read-only hooks directory");
    server.hooks = Some(hooks);
    return Some((repo, origin, server, content));
}

/// The object name a pointer file at `rev:path` records.
fn pointer_oid_at(fixture: &Fixture, repo: &Path, rev_path: &str) -> String {
    let pointer: String = super::commit::git_text(fixture, repo, &["cat-file", "blob", rev_path]);
    return pointer
        .lines()
        .find_map(|line| { return line.strip_prefix("oid sha256:"); })
        .expect("a pointer file has an oid line")
        .to_owned();
}

/// The object name the pointer file at `HEAD:big.bin` records.
fn pointer_oid(fixture: &Fixture, repo: &Path) -> String {
    return pointer_oid_at(fixture, repo, "HEAD:big.bin");
}

/// The origin's `main` seen from outside.
fn published_main(fixture: &Fixture, origin: &Path) -> String {
    let output = direct(fixture)
        .current_dir(origin)
        .args(["rev-parse", "--verify", "--quiet", "refs/heads/main"])
        .output()
        .expect("rev-parse starts");
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return if text.is_empty() { String::from("absent") } else { text };
}

/// The object reaches the server before the ref reaches the origin,
/// and the origin then holds the pointer commit.
#[test]
fn the_real_upload_delivers_the_object_before_the_ref() {
    let fixture: Fixture = fixture("lfs-real");
    let Some((repo, origin, server, content)) = lfs_world(&fixture) else {
        return;
    };
    let pushed: Observed = observe(
        &mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["push", "origin", "main"]),
        b"",
    );
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    let uploads: Vec<Upload> = server.uploads();
    assert_eq!(uploads.len(), 1, "one object was uploaded: {stderr}");
    assert_eq!(uploads[0].oid, pointer_oid(&fixture, repo.as_path()));
    assert_eq!(uploads[0].bytes, content, "the server holds the file's bytes");
    assert_eq!(uploads[0].origin_main, "absent", "the object arrived before the ref moved");
    assert_ne!(published_main(&fixture, origin.as_path()), "absent", "the ref moved after the upload: {stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// A server that refuses the upload stops the push:
/// the origin gets no ref and the caller gets the explanation.
#[test]
fn a_refused_upload_publishes_no_pointer() {
    let fixture: Fixture = fixture("lfs-real-refused");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    server.state.lock().expect("state").refuse_batch = true;
    let pushed: Observed = observe(
        &mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["push", "origin", "main"]),
        b"",
    );
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert!(stderr.contains("`git-lfs push` exited with status"), "{stderr}");
    assert!(stderr.contains("nothing was published"), "{stderr}");
    assert_eq!(published_main(&fixture, origin.as_path()), "absent", "{stderr}");
    assert!(server.uploads().is_empty());
    assert_eq!(batches(&server), 1, "a refused credential is asked once, never retried: {stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// The auto-push of a commit that adds an LFS file uploads the object first,
/// with the real program.
#[test]
fn the_real_upload_precedes_the_auto_push_of_a_commit() {
    let fixture: Fixture = fixture("lfs-real-auto");
    let Some((repo, origin, server, content)) = lfs_world(&fixture) else {
        return;
    };
    std::fs::write(repo.join("second.bin"), [content.as_slice(), b"second"].concat()).expect("second.bin");
    let added: Observed = observe(
        &mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["add", "--", "second.bin"]),
        b"",
    );
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let landed: Observed = observe(
        &mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "second.bin"]),
        b"",
    );
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("auto-push to origin failed"), "{stderr}");
    let uploads: Vec<Upload> = server.uploads();
    assert_eq!(uploads.len(), 2, "both LFS objects of the branch were uploaded: {stderr}");
    assert!(uploads.iter().all(|upload| { return upload.origin_main == "absent"; }), "{uploads:?}");
    assert_ne!(published_main(&fixture, origin.as_path()), "absent", "{stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// How many batch requests the server has seen.
fn batches(server: &LfsServer) -> usize {
    return server.state.lock().expect("state").batches;
}

/// A wrapped `git push origin main` with extra environment variables.
fn push_main(fixture: &Fixture, repo: &Path, variables: &[(&str, &str)]) -> Observed {
    let mut command = super::commit::wrapped_commit(fixture, repo, &["push", "origin", "main"]);
    command.envs(variables.iter().copied());
    return observe(&mut command, b"");
}

/// A commit that adds no file,
/// made by real Git so that no auto-push runs.
fn empty_commit(fixture: &Fixture, repo: &Path) {
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    let committed = direct(fixture)
        .current_dir(repo)
        .env("PATH", path)
        .args(["commit", "--quiet", "--allow-empty", "--message=no new object"])
        .output()
        .expect("commit starts");
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(committed.stderr.as_slice()));
}

/// Point the clone at the server through a URL that carries a user and a token,
/// as the incident's `lfs.url` did.
fn use_credentialed_url(fixture: &Fixture, repo: &Path, server: &LfsServer) {
    let url: String = format!("http://user:secret@127.0.0.1:{}/lfs", server.port);
    git(fixture, repo, &["config", "lfs.url", url.as_str()]);
}

/// A push of a commit that adds no LFS object sends no batch request:
/// git-lfs skips what the tracking refs reach.
#[test]
fn a_push_that_adds_no_object_sends_no_batch_request() {
    let fixture: Fixture = fixture("lfs-real-bounded");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let after_first: usize = batches(&server);
    assert!(after_first >= 1, "the first push uploaded its object through the batch API");
    empty_commit(&fixture, repo.as_path());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&second);
    assert_eq!(second.code, Some(0), "{stderr}");
    assert_eq!(batches(&server), after_first, "the second push added no object, so no batch request: {stderr}");
    assert_eq!(server.uploads().len(), 1);
    assert_eq!(published_main(&fixture, origin.as_path()), super::commit::git_text(&fixture, repo.as_path(), &["rev-parse", "main"]).trim());
    drop(server);
    remove_settled(&fixture);
}

/// Positive control for the bound:
/// a remote with a push URL keeps the full scan,
/// which asks the server about every
///  object again,
/// so the same push that sent nothing above sends one batch request here.
#[test]
fn a_remote_with_a_push_url_keeps_the_full_scan() {
    let fixture: Fixture = fixture("lfs-real-full-scan");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    git(&fixture, repo.as_path(), &["config", "remote.origin.pushurl", origin.to_str().expect("UTF-8")]);
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let after_first: usize = batches(&server);
    empty_commit(&fixture, repo.as_path());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&second);
    assert_eq!(second.code, Some(0), "{stderr}");
    assert_eq!(batches(&server), after_first + 1, "the full scan asked about the known object again: {stderr}");
    assert_eq!(server.uploads().len(), 1, "an object the server has is not sent twice");
    drop(server);
    remove_settled(&fixture);
}

/// What:
///  A second bare origin and the server behind it,
/// with the remote name and `lfs.url` moved onto them.
/// Why:
///   `git remote set-url` leaves this clone's tracking refs describing the server it fetched from,
/// and a push that
///  takes them as what the new server holds publishes pointers whose objects never travelled.
fn move_origin(fixture: &Fixture, repo: &Path) -> (PathBuf, LfsServer) {
    let origin: PathBuf = fixture.root.join("second.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "second.git"]);
    let server: LfsServer = LfsServer::start(origin.clone());
    git(fixture, repo, &["config", "lfs.url", format!("http://127.0.0.1:{}/lfs", server.port).as_str()]);
    git(fixture, repo, &["remote", "set-url", "origin", origin.to_str().expect("UTF-8")]);
    return (origin, server);
}

/// A commit that adds one LFS file on a branch of its own,
/// made by real Git so that no auto-push runs,
/// leaving the
///  worktree on `main`.
fn side_branch_with_a_pointer(fixture: &Fixture, repo: &Path, content: &[u8]) -> String {
    git(fixture, repo, &["switch", "--quiet", "--create", "side"]);
    std::fs::write(repo.join("side.bin"), content).expect("side.bin");
    let added: Observed = observe(
        &mut super::commit::wrapped_commit(fixture, repo, &["add", "--", "side.bin"]),
        b"",
    );
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    let committed = direct(fixture)
        .current_dir(repo)
        .env("PATH", path)
        .args(["commit", "--quiet", "--message=side"])
        .output()
        .expect("commit starts");
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let oid: String = pointer_oid_at(fixture, repo, "side:side.bin");
    git(fixture, repo, &["switch", "--quiet", "main"]);
    return oid;
}

/// What:
///  A remote whose URL was moved to another server uploads the objects again,
/// to the server the push goes to.
/// Why:
///   The tracking refs describe the server the last fetch saw;
/// taking them as what the new server holds
///  sends nothing and publishes a pointer whose content is nowhere.
#[test]
fn a_remote_whose_url_moved_uploads_its_objects_to_the_new_server() {
    let fixture: Fixture = fixture("lfs-real-moved-url");
    let Some((repo, first_origin, first, content)) = lfs_world(&fixture) else {
        return;
    };
    let pushed: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(pushed.code, Some(0), "{}", stderr_of(&pushed));
    assert_eq!(first.uploads().len(), 1, "the first server holds the object: {}", stderr_of(&pushed));
    let (second_origin, second): (PathBuf, LfsServer) = move_origin(&fixture, repo.as_path());
    let moved: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&moved);
    assert_eq!(moved.code, Some(0), "{stderr}");
    assert_ne!(published_main(&fixture, second_origin.as_path()), "absent", "the ref moved on the new server: {stderr}");
    let uploads: Vec<Upload> = second.uploads();
    assert_eq!(uploads.len(), 1, "the object reached the server the push went to: {stderr}");
    assert_eq!(uploads[0].bytes, content, "and it is the object's content: {stderr}");
    drop(second);
    drop(first);
    remove_settled(&fixture);
    let _first_origin: PathBuf = first_origin;
}

/// What:
///  A mirror Git reads from a numeric true value publishes every branch and uploads every object.
/// Why:
///   `remote.<name>.mirror=2` is true to Git and was false to the plan,
/// so the other branches were published
///  with pointers whose objects never travelled.
#[test]
fn a_numeric_mirror_uploads_the_objects_of_every_branch_it_publishes() {
    let fixture: Fixture = fixture("lfs-real-numeric-mirror");
    let Some((repo, origin, server, content)) = lfs_world(&fixture) else {
        return;
    };
    let side_oid: String = side_branch_with_a_pointer(&fixture, repo.as_path(), [content.as_slice(), b"side"].concat().as_slice());
    git(&fixture, repo.as_path(), &["config", "remote.origin.mirror", "2"]);
    let read: String = super::commit::git_text(&fixture, repo.as_path(), &["config", "--type=bool", "--get", "remote.origin.mirror"]);
    assert_eq!(read.trim(), "true", "Git reads the fixture's value as true");
    let pushed: Observed = observe(&mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["push", "origin"]), b"");
    let stderr: String = stderr_of(&pushed);
    assert_eq!(pushed.code, Some(0), "{stderr}");
    let side_published = direct(&fixture)
        .current_dir(origin.as_path())
        .args(["rev-parse", "--verify", "--quiet", "refs/heads/side"])
        .output()
        .expect("rev-parse starts");
    assert!(side_published.status.success(), "the mirror published the side branch: {stderr}");
    let uploads: Vec<Upload> = server.uploads();
    let names: Vec<&str> = uploads.iter().map(|upload| { return upload.oid.as_str(); }).collect();
    assert!(names.contains(&side_oid.as_str()), "the side branch's object was uploaded: {names:?} {stderr}");
    assert_eq!(uploads.len(), 2, "both objects travelled once: {names:?} {stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// How long the service waits before its first new try of a push in the control that must see no new try:
/// far past the control.
const NO_NEW_TRY_SECONDS: &str = "600";

/// A typed push whose server answers HTTP 503 fails on the first answer:
/// the wrapper does not run `git-lfs push` again (pick 12
///  of the shipped-limits audit,
/// the coordinator's under the owner's delegation),
/// the program's words are relayed with the
///  credentials hidden,
/// the explanation follows,
/// and nothing is published.
#[test]
fn a_server_error_ends_a_typed_push_at_once() {
    let fixture: Fixture = fixture("lfs-real-server-error-once");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    use_credentialed_url(&fixture, repo.as_path(), &server);
    server.state.lock().expect("state").fail_batches = 2;
    let pushed: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&pushed);
    assert_ne!(pushed.code, Some(0), "{stderr}");
    assert_eq!(batches(&server), 1, "one refused batch request and no second run: {stderr}");
    assert!(!stderr.contains("retry 1"), "{stderr}");
    assert!(stderr.contains("from HTTP 503") && stderr.contains("nothing was published"), "{stderr}");
    assert!(!stderr.contains("secret"), "{stderr}");
    assert_eq!(published_main(&fixture, origin.as_path()), "absent", "{stderr}");
    assert!(server.uploads().is_empty());
    drop(server);
    remove_settled(&fixture);
}

/// The auto-push of a commit whose upload meets HTTP 503 is left to the service's push retry plan alone:
/// the push fails once,
///  the person's record says the service tries again by itself,
/// and no second run happens in place.
#[test]
fn an_auto_push_that_meets_a_server_error_is_left_to_the_services_retry_plan() {
    let fixture: Fixture = fixture("lfs-real-auto-plan");
    super::support::give_service_environment(&fixture, "CLI_GIT_PUSH_RETRY_SECONDS", NO_NEW_TRY_SECONDS);
    let Some((repo, origin, server, content)) = lfs_world(&fixture) else {
        return;
    };
    use_credentialed_url(&fixture, repo.as_path(), &server);
    std::fs::write(repo.join("second.bin"), [content.as_slice(), b"second"].concat()).expect("second.bin");
    let added: Observed = observe(
        &mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["add", "--", "second.bin"]),
        b"",
    );
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    server.state.lock().expect("state").fail_batches = 1;
    let mut commit = super::commit::wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=second", "--", "second.bin"]);
    let landed: Observed = observe(&mut commit, b"");
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "the commit stays saved: {stderr}");
    let report: String = super::support::push_report(repo.as_path(), &stderr);
    assert_eq!(batches(&server), 1, "one refused batch request, and the new try is the plan's, far off: {report}");
    assert!(report.contains("tries this push again by itself"), "{report}");
    assert!(report.contains("http://<credentials>@127.0.0.1"), "{report}");
    assert!(!report.contains("secret"), "{report}");
    assert_eq!(published_main(&fixture, origin.as_path()), "absent", "{report}");
    drop(server);
    remove_settled(&fixture);
}

/// Put a recording script in place of the `git-lfs` link:
/// it appends its arguments to the returned file,
/// then runs the
///  real program.
/// The file does not exist until the wrapper starts `git-lfs`.
fn record_git_lfs(fixture: &Fixture) -> PathBuf {
    let link: PathBuf = fixture.root.join("bin/git-lfs");
    let real: PathBuf = std::fs::read_link(&link).expect("the link to the real program");
    std::fs::remove_file(&link).expect("link removed");
    let log: PathBuf = fixture.root.join("git-lfs-runs.txt");
    let script: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec '{}' \"$@\"\n", log.display(), real.display());
    super::support::executable(link.as_path(), script.as_bytes());
    return log;
}

/// The arguments of every `git-lfs push` run,
/// one run per line.
/// Git's own `filter-process` runs of the same program are
///  not the upload and are left out.
fn recorded_runs(log: &Path) -> Vec<String> {
    return std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .filter(|line| { return line.starts_with("push"); })
        .map(str::to_owned)
        .collect();
}

/// Add `name` with `content` as an LFS file through the wrapper (so the clean filter stores the object) and commit it with
///  real Git,
/// so that no auto-push runs;
/// the object name its pointer holds.
fn commit_lfs_file(fixture: &Fixture, repo: &Path, name: &str, content: &[u8]) -> String {
    std::fs::write(repo.join(name), content).expect("lfs file");
    let added: Observed = observe(&mut super::commit::wrapped_commit(fixture, repo, &["add", "--", name]), b"");
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    let committed = direct(fixture)
        .current_dir(repo)
        .env("PATH", path)
        .args(["commit", "--quiet", &format!("--message={name}"), "--", name])
        .output()
        .expect("commit starts");
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let pointer: String = super::commit::git_text(fixture, repo, &["cat-file", "blob", &format!("HEAD:{name}")]);
    return pointer.lines().find_map(|line| { return line.strip_prefix("oid sha256:"); }).expect("an oid line").to_owned();
}

/// A push of commits that add no pointer starts no `git-lfs` at all and makes no request to the LFS server.
#[test]
fn a_push_without_a_new_pointer_starts_no_git_lfs() {
    let fixture: Fixture = fixture("lfs-real-none");
    let Some((repo, _origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    let after_first: usize = batches(&server);
    empty_commit(&fixture, repo.as_path());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(second.code, Some(0), "{}", stderr_of(&second));
    assert_eq!(recorded_runs(log.as_path()), Vec::<String>::new(), "git-lfs was not started");
    assert_eq!(batches(&server), after_first, "and no request reached the server");
    drop(server);
    remove_settled(&fixture);
}

/// A repository whose configuration holds a key the typed-value proof cannot vouch for (the stand-in for a user-level
///  `core.fsmonitor = true`,
/// which would start a daemon in a test) is still read in process once Git has read the plan:
/// the
///  push adds no pointer,
/// so it starts no `git-lfs`.
/// Before,
/// the reader declined it and `git-lfs` scanned the whole push.
#[test]
fn a_value_the_typed_proof_refuses_does_not_send_the_scan_to_git_lfs() {
    let fixture: Fixture = fixture("lfs-real-unprovable");
    let Some((repo, _origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    let after_first: usize = batches(&server);
    git(&fixture, repo.as_path(), &["config", "diff.dirstat", "changes"]);
    empty_commit(&fixture, repo.as_path());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(second.code, Some(0), "{}", stderr_of(&second));
    assert_eq!(recorded_runs(log.as_path()), Vec::<String>::new(), "git-lfs was not started");
    assert_eq!(batches(&server), after_first, "and no request reached the server");
    drop(server);
    remove_settled(&fixture);
}

/// The same for the auto-push,
/// which runs the push as a bounded child:
/// a commit that adds no pointer,
/// in a repository whose
///  configuration holds a key the typed-value proof refuses,
/// reaches the remote without a `git-lfs` run.
#[test]
fn an_auto_push_with_a_value_the_typed_proof_refuses_starts_no_git_lfs() {
    let fixture: Fixture = fixture("lfs-real-auto-unprovable");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    git(&fixture, repo.as_path(), &["config", "diff.dirstat", "changes"]);
    let landed: Observed = observe(&mut super::commit::wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]), b"");
    let stderr: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "{stderr}");
    assert!(!stderr.contains("auto-push to origin failed"), "{stderr}");
    assert_eq!(published_main(&fixture, origin.as_path()), super::commit::git_text(&fixture, repo.as_path(), &["rev-parse", "main"]).trim(), "{stderr}");
    assert_eq!(recorded_runs(log.as_path()), Vec::<String>::new(), "git-lfs was not started: {stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// A push that adds one pointer uploads exactly that object by name,
/// with one `git-lfs` run and one batch request.
#[test]
fn a_push_that_adds_one_pointer_uploads_exactly_that_object() {
    let fixture: Fixture = fixture("lfs-real-one");
    let Some((repo, origin, server, content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    let after_first: usize = batches(&server);
    let added: String = commit_lfs_file(&fixture, repo.as_path(), "second.bin", [content.as_slice(), b"second"].concat().as_slice());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&second);
    assert_eq!(second.code, Some(0), "{stderr}");
    assert_eq!(recorded_runs(log.as_path()), vec![format!("push --object-id origin {added}")], "{stderr}");
    assert_eq!(batches(&server), after_first + 1, "{stderr}");
    let uploads: Vec<Upload> = server.uploads();
    assert_eq!(uploads.len(), 2, "{stderr}");
    assert_eq!(uploads[1].oid, added);
    assert_ne!(published_main(&fixture, origin.as_path()), "absent");
    drop(server);
    remove_settled(&fixture);
}

/// A pointer that one commit adds and the next removes is still uploaded,
/// because the commit that holds it is pushed.
#[test]
fn a_pointer_added_and_removed_between_pushes_is_still_uploaded() {
    let fixture: Fixture = fixture("lfs-real-transient");
    let Some((repo, _origin, server, content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    let transient: String = commit_lfs_file(&fixture, repo.as_path(), "transient.bin", [content.as_slice(), b"transient"].concat().as_slice());
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    for arguments in [["rm", "--quiet", "--", "transient.bin"], ["commit", "--quiet", "--message=remove it again", ""]] {
        let kept: Vec<&str> = arguments.iter().copied().filter(|word| { return !word.is_empty(); }).collect();
        let ran = direct(&fixture).current_dir(repo.as_path()).env("PATH", &path).args(kept).output().expect("starts");
        assert!(ran.status.success(), "{}", String::from_utf8_lossy(ran.stderr.as_slice()));
    }
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&second);
    assert_eq!(second.code, Some(0), "{stderr}");
    assert_eq!(recorded_runs(log.as_path()), vec![format!("push --object-id origin {transient}")], "{stderr}");
    assert!(server.uploads().iter().any(|upload| { return upload.oid == transient; }), "{stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// The destination the tracking refs may not describe is scanned from the start:
/// the object the first push sent is named
///  again (the server answers that it has it),
/// and only that,
/// by name.
#[test]
fn an_untrusted_destination_names_every_pointer_of_the_history() {
    let fixture: Fixture = fixture("lfs-real-untrusted");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    git(&fixture, repo.as_path(), &["config", "remote.origin.pushurl", origin.to_str().expect("UTF-8")]);
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    empty_commit(&fixture, repo.as_path());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    let stderr: String = stderr_of(&second);
    assert_eq!(second.code, Some(0), "{stderr}");
    let runs: Vec<String> = recorded_runs(log.as_path());
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert!(runs[0].starts_with("push --object-id origin ") && runs[0].len() == "push --object-id origin ".len() + 64, "{runs:?}");
    assert_eq!(server.uploads().len(), 1, "an object the server has is not sent twice");
    drop(server);
    remove_settled(&fixture);
}

/// When the readers decline the repository,
/// `git-lfs` scans with the bounded form,
/// as before.
#[test]
fn a_repository_the_readers_decline_is_scanned_by_git_lfs() {
    let fixture: Fixture = fixture("lfs-real-declined");
    let Some((repo, _origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    decline_in_process_lfs_scan(&fixture, repo.as_path());
    empty_commit(&fixture, repo.as_path());
    let second: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(second.code, Some(0), "{}", stderr_of(&second));
    assert_eq!(recorded_runs(log.as_path()), vec![String::from("push origin main")]);
    drop(server);
    remove_settled(&fixture);
}

/// A push that recurses into submodules and adds no pointer of its own starts no `git-lfs`,
/// and still prints the notice
///  that a submodule's LFS objects are not covered.
#[test]
fn a_submodule_push_without_a_new_pointer_still_gets_the_notice() {
    let fixture: Fixture = fixture("lfs-real-submodule-notice");
    let Some((repo, _origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    let first: Observed = push_main(&fixture, repo.as_path(), &[]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let log: PathBuf = record_git_lfs(&fixture);
    empty_commit(&fixture, repo.as_path());
    let mut command = super::commit::wrapped_commit(&fixture, repo.as_path(), &["push", "--recurse-submodules=on-demand", "origin", "main"]);
    let second: Observed = observe(&mut command, b"");
    let stderr: String = stderr_of(&second);
    assert_eq!(second.code, Some(0), "{stderr}");
    assert!(stderr.contains("this push also pushes submodules"), "{stderr}");
    assert_eq!(recorded_runs(log.as_path()), Vec::<String>::new(), "git-lfs was not started");
    drop(server);
    remove_settled(&fixture);
}

//region A push nobody typed never opens the password dialog

/// A recording stand-in for the desktop password program a host names in `SSH_ASKPASS`,
/// as this machine names `/usr/bin/ksshaskpass`.
/// It records every question and answers one,
/// so a route that asks a person is seen at once and never waits for an answer that will not come.
fn recording_askpass(fixture: &Fixture) -> PathBuf {
    let log: PathBuf = fixture.root.join("askpass-questions.txt");
    let script: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nprintf 'someone\\n'\n", log = log.display());
    super::support::executable(fixture.root.join("askpass").as_path(), script.as_bytes());
    return log;
}

/// Every question the stand-in password program was asked,
/// in order.
fn questions(log: &Path) -> Vec<String> {
    return std::fs::read_to_string(log).unwrap_or_default().lines().map(str::to_owned).collect();
}

/// A pseudo-terminal the test keeps open:
/// the master end,
/// which nothing writes into,
/// and the slave end,
/// which a child's standard input is made from.
struct Terminal {
    /// Kept open for the child's whole run.
    _master: std::fs::File,
    /// The end the child's standard input is made from.
    slave: std::fs::File,
}

/// Open a new pseudo-terminal.
fn terminal() -> Terminal {
    use std::os::fd::FromRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    // SAFETY: `posix_openpt` returns a descriptor nothing else owns, which `File` takes over at once; the two calls after it
    //  only read or change that descriptor. `O_NOCTTY` keeps the terminal from becoming this test process's own.
    let (master, descriptor): (std::fs::File, libc::c_int) = unsafe {
        let descriptor: libc::c_int = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(descriptor >= 0, "a pseudo-terminal is opened");
        let master: std::fs::File = std::fs::File::from_raw_fd(descriptor);
        assert_eq!(libc::grantpt(descriptor), 0, "the slave is granted");
        assert_eq!(libc::unlockpt(descriptor), 0, "the slave is unlocked");
        (master, descriptor)
    };
    let name: std::ffi::CString = super::support::pseudo_terminal_slave_name(descriptor);
    let slave: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open(name.to_str().expect("a UTF-8 terminal name"))
        .expect("the slave opens");
    return Terminal { _master: master, slave };
}

/// What:
///  Run one wrapped push whose standard input is `/dev/null`
///  (or a terminal,
/// when `at` is given)
///  and whose process is in a session of its own.
/// Why:
///   The incident's push came from a session with no controlling terminal,
/// and a control that inherited this host's terminal would exercise the other route;
/// the new session makes the control's meaning the same wherever the suite runs.
fn observe_push(fixture: &Fixture, repo: &Path, arguments: &[&str], variables: &[(&str, &str)], at: Option<&std::fs::File>) -> Observed {
    use std::os::unix::process::CommandExt;
    let mut command: std::process::Command = super::commit::wrapped_commit(fixture, repo, arguments);
    command.envs(variables.iter().copied()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    match at {
        Some(slave) => {
            command.stdin(std::process::Stdio::from(slave.try_clone().expect("duplicate the terminal")));
        }
        None => {
            command.stdin(std::process::Stdio::null());
        }
    }
    // SAFETY: between fork and exec only async-signal-safe calls run: `setsid`, and on the terminal route one `ioctl` on
    //  descriptor 0, which is the terminal the command was given.
    let with_terminal: bool = at.is_some();
    unsafe {
        command.pre_exec(move || {
            libc::setsid();
            if with_terminal {
                libc::ioctl(0, libc::TIOCSCTTY, 0);
            }
            return Ok(());
        });
    }
    let child: std::process::Child = command.spawn().expect("the wrapped push starts");
    let output: std::process::Output = child.wait_with_output().expect("the wrapped push finishes");
    assert_ne!(
        std::os::unix::process::ExitStatusExt::signal(&output.status),
        Some(super::support::TIMED_OUT_SIGNAL),
        "the push hit the fixture's wall-clock bound: {arguments:?}"
    );
    return Observed { code: output.status.code(), stdout: output.stdout, stderr: output.stderr };
}

/// The programs still running whose working directory is inside the fixture,
/// as `"pid:command"`,
/// after waiting a moment for the ones a finished run is ending.
fn processes_under(root: &Path) -> Vec<String> {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let mut found: Vec<String> = Vec::new();
        // What: A system with no `/proc` makes this check fail instead of passing with nothing looked at.
        // Why:  The control's point is that the run left no program behind, and a walk that cannot look would assert exactly
        //  the sentence it was meant to test.
        let entries: std::fs::ReadDir = std::fs::read_dir("/proc").expect("this control needs /proc to look for leftover programs");
        for entry in entries.flatten() {
            let process: PathBuf = entry.path();
            let Ok(cwd) = std::fs::read_link(process.join("cwd")) else {
                continue;
            };
            if !cwd.starts_with(root) {
                continue;
            }
            let command: String = std::fs::read(process.join("cmdline"))
                .unwrap_or_default()
                .split(|byte| { return *byte == 0; })
                .map(|word| { return String::from_utf8_lossy(word).into_owned(); })
                .collect::<Vec<String>>()
                .join(" ");
            found.push(format!("{}:{command}", entry.file_name().to_string_lossy()));
        }
        if found.is_empty() || std::time::Instant::now() > deadline {
            return found;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The second words of a push the incident's form names:
/// an explicit URL destination and a bare refspec.
const URL_FORM_DESTINATION: &str = "refs/heads/fixture-url-form";

/// The ref a push in the incident's form publishes.
fn published_url_form(fixture: &Fixture, origin: &Path) -> String {
    let output = direct(fixture)
        .current_dir(origin)
        .args(["rev-parse", "--verify", "--quiet", URL_FORM_DESTINATION])
        .output()
        .expect("rev-parse starts");
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return if text.is_empty() { String::from("absent") } else { text };
}

/// A push nobody typed does not open the desktop password dialog:
/// the LFS server asks for a login,
/// `SSH_ASKPASS` names a recorder,
/// the command's session has no controlling terminal,
/// and the upload fails at once with the login the caller can act on.
#[test]
fn an_unattended_push_without_a_controlling_terminal_never_opens_the_password_dialog() {
    let fixture: Fixture = fixture("lfs-real-no-terminal");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    server.state.lock().expect("state").login_required = true;
    let log: PathBuf = recording_askpass(&fixture);
    let askpass: String = fixture.root.join("askpass").to_string_lossy().into_owned();
    let destination: String = format!("file://{}", origin.display());
    let refspec: String = format!("HEAD:{URL_FORM_DESTINATION}");
    let pushed: Observed = observe_push(
        &fixture,
        repo.as_path(),
        &["push", destination.as_str(), refspec.as_str()],
        &[("SSH_ASKPASS", askpass.as_str())],
        None,
    );
    let stderr: String = stderr_of(&pushed);
    assert!(!log.exists(), "the desktop password program was started: {:?}", questions(log.as_path()));
    assert_ne!(pushed.code, Some(0), "a login the caller does not have is a failure: {stderr}");
    assert_eq!(published_url_form(&fixture, origin.as_path()), "absent", "nothing was published: {stderr}");
    assert!(server.uploads().is_empty(), "{stderr}");
    assert_eq!(processes_under(fixture.root.as_path()), Vec::<String>::new(), "the run left a program behind");
    // The refusal names the server that asked, from the program's own words, and says how to give it a login.
    let asked: String = format!("the server `127.0.0.1:{}`", server.port);
    assert!(stderr.contains(asked.as_str()), "the server that asked is named: {stderr}");
    assert!(stderr.contains("no controlling terminal") && stderr.contains("credential.helper"), "{stderr}");
    assert!(stderr.contains("doc/troubleshooting/lfs-credential-in-url.md"), "{stderr}");
    assert!(stderr.contains("`git-lfs push` exited with status"), "the program's own status is kept: {stderr}");
    drop(server);
    remove_settled(&fixture);
}

/// A push a person typed keeps Git's and git-lfs's own prompting:
/// with a controlling terminal in its session,
/// the same login-requiring server reaches the password program.
#[test]
fn a_push_with_a_controlling_terminal_keeps_its_prompting() {
    let fixture: Fixture = fixture("lfs-real-terminal");
    let Some((repo, origin, server, _content)) = lfs_world(&fixture) else {
        return;
    };
    server.state.lock().expect("state").login_required = true;
    let log: PathBuf = recording_askpass(&fixture);
    let at: Terminal = terminal();
    let askpass: String = fixture.root.join("askpass").to_string_lossy().into_owned();
    let destination: String = format!("file://{}", origin.display());
    let refspec: String = format!("HEAD:{URL_FORM_DESTINATION}");
    let pushed: Observed = observe_push(
        &fixture,
        repo.as_path(),
        &["push", destination.as_str(), refspec.as_str()],
        &[("SSH_ASKPASS", askpass.as_str())],
        Some(&at.slave),
    );
    let stderr: String = stderr_of(&pushed);
    assert!(!questions(log.as_path()).is_empty(), "the route at a terminal asked nobody: {stderr}");
    assert_ne!(pushed.code, Some(0), "the server refuses the login the stand-in gives: {stderr}");
    drop(server);
    remove_settled(&fixture);
}

//endregion

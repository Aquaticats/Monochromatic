//! Binary-level controls for the Git LFS upload against the real `git-lfs` and an LFS server written inline.
//! The server speaks the batch API and the basic transfer over a local socket and records, for every uploaded object,
//!  what the origin's `main` was at that moment, so the order "object first, ref second" is observed and not assumed.
//! Hosts without `git-lfs` on PATH report that and check nothing; the stand-in controls in
//!  `binary_lfs_pre_push_tests.rs` cover the same paths there.

use super::commit::remove_settled;
use super::support::{Fixture, Observed, REAL_GIT, direct, fixture, git, observe, repository, stderr_of};
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
    /// The origin's `refs/heads/main` when the object arrived, or `absent`.
    origin_main: String,
}

/// The server's shared record.
#[derive(Default)]
struct ServerState {
    /// Every stored object.
    uploads: Vec<Upload>,
    /// Whether the batch endpoint refuses, as a server that rejects the credentials would.
    refuse_batch: bool,
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
    /// The repository's hooks directory, made read-only so `git-lfs` cannot write its shims into it again;
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

/// Read one request: its request line, its headers' content length, and its body.
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
fn respond(mut stream: &TcpStream, status: &str, body: &str) {
    let text: String = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/vnd.git-lfs+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    // A client that went away is not a server failure.
    let _written = stream.write_all(text.as_bytes());
}

/// Answer one request of the batch API, the lock-verification endpoint or the basic upload.
fn serve(stream: &TcpStream, port: u16, origin: &Path, state: &Mutex<ServerState>) {
    let Some((request_line, body)) = read_request(stream) else {
        return;
    };
    if request_line.starts_with("POST /lfs/objects/batch ") {
        if state.lock().expect("state").refuse_batch {
            respond(stream, "403 Forbidden", "{\"message\":\"the fixture server refuses every batch\"}");
            return;
        }
        let asked: serde_json::Value = serde_json::from_slice(body.as_slice()).unwrap_or_default();
        let known: Vec<String> = state.lock().expect("state").uploads.iter().map(|upload| { return upload.oid.clone(); }).collect();
        let objects: Vec<serde_json::Value> = asked["objects"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|object| {
                let oid: String = object["oid"].as_str().unwrap_or_default().to_owned();
                if known.contains(&oid) {
                    return serde_json::json!({ "oid": oid, "size": object["size"] });
                }
                return serde_json::json!({
                    "oid": oid,
                    "size": object["size"],
                    "actions": { "upload": { "href": format!("http://127.0.0.1:{port}/lfs/objects/{oid}") } }
                });
            })
            .collect();
        respond(stream, "200 OK", serde_json::json!({ "transfer": "basic", "objects": objects }).to_string().as_str());
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

/// The server ends with its test: the flag is set and a connection wakes the accept loop.
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

/// The real `git-lfs` on the test process's PATH, if any.
fn real_lfs_program() -> Option<PathBuf> {
    let path: std::ffi::OsString = std::env::var_os("PATH")?;
    return std::env::split_paths(&path)
        .map(|directory| { return directory.join("git-lfs"); })
        .find(|candidate| { return candidate.is_file(); });
}

/// A repository wired to the real `git-lfs` and the server, with one committed LFS file; `None` without `git-lfs`.
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

/// The object name a pointer file records.
fn pointer_oid(fixture: &Fixture, repo: &Path) -> String {
    let pointer: String = super::commit::git_text(fixture, repo, &["cat-file", "blob", "HEAD:big.bin"]);
    return pointer
        .lines()
        .find_map(|line| { return line.strip_prefix("oid sha256:"); })
        .expect("a pointer file has an oid line")
        .to_owned();
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

/// The object reaches the server before the ref reaches the origin, and the origin then holds the pointer commit.
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

/// A server that refuses the upload stops the push: the origin gets no ref and the caller gets the explanation.
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
    assert!(stderr.contains("`git-lfs pre-push` exited with status"), "{stderr}");
    assert!(stderr.contains("nothing was published"), "{stderr}");
    assert_eq!(published_main(&fixture, origin.as_path()), "absent", "{stderr}");
    assert!(server.uploads().is_empty());
    drop(server);
    remove_settled(&fixture);
}

/// The auto-push of a commit that adds an LFS file uploads the object first, with the real program.
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

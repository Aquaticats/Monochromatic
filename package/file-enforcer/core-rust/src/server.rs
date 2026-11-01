//! What:
//!  The per-worktree server:
//! listens on an owner-only Unix socket,
//! serializes every run,
//! and exits when idle.
//! Why:
//!  One process owns all writes to a worktree,
//! so concurrent `write`,
//! `check` and `watch` commands never clobber each
//!  other.
//! It exits after an idle period with no clients and no watch subscriber,
//! when its worktree root disappears
//!  or is replaced,
//! and when a client's handshake shows a different build.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export async function serve(options: ServerOptions, registry: RuleRegistry): Promise<void>
//! ```

/// Imports `ConfigScope` from `crate::config_decode`.
use crate::config_decode::ConfigScope;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `build_identity`,
/// `worktree_identity` from `crate::identity`.
use crate::identity::{build_identity, worktree_identity};
/// Imports `RuleRegistry` from `crate::registry`.
use crate::registry::RuleRegistry;
/// Imports `handle_connection` from `crate::server_connection`.
use crate::server_connection::handle_connection;
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `PermissionsExt` from `std::os::unix::fs`.
use std::os::unix::fs::PermissionsExt;
/// Imports `UnixListener` from `std::os::unix::net`.
use std::os::unix::net::UnixListener;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;
/// Imports `AtomicBool`,
/// `AtomicUsize`,
/// `Ordering` from `std::sync::atomic`.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
/// Imports `Arc`,
/// `Mutex` from `std::sync`.
use std::sync::{Arc, Mutex};
/// Imports `Duration`,
/// `Instant` from `std::time`.
use std::time::{Duration, Instant};

/// What:
///  What a server serves and how it behaves.
/// Why:
///  The command line passes these to the hidden `serve` command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerOptions = { root: string; scope: ConfigScope; socket: string; idle: number };
/// ```
#[derive(Clone, Debug)]
pub struct ServerOptions {
    /// Worktree root,
    /// or the home directory for the per-user server.
    pub root: PathBuf,
    /// Which config scope this server runs.
    pub scope: ConfigScope,
    /// Socket path to bind.
    pub socket: PathBuf,
    /// How long the server may sit without clients before it exits.
    pub idle: Duration,
}

/// What:
///  State shared by every connection thread.
///  `Arc` is a shared,
/// reference-counted pointer (TS:
/// an object several closures keep alive);
///  `Mutex` is a lock around a value (TS has no equivalent:
/// threads share memory).
/// Why:
///  The run lock is what serializes writes;
/// the counters decide idle exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Shared { runLock: Mutex; shutdown: boolean; connections: number; lastActivity: number }
/// ```
pub struct Shared {
    /// Options the server started with.
    pub options: ServerOptions,
    /// Rule builders for every run.
    pub registry: RuleRegistry,
    /// Held while a run executes,
    /// so runs never overlap.
    pub run_lock: Mutex<()>,
    /// Set to ask the accept loop to stop.
    pub shutdown: AtomicBool,
    /// Open client connections,
    /// watch subscribers included.
    pub connections: AtomicUsize,
    /// When a connection last opened or closed.
    pub last_activity: Mutex<Instant>,
    /// The worktree identity captured at startup.
    pub worktree: Value,
    /// This build's identity.
    pub build: String,
}

/// What:
///   Operations on the state every connection thread shares.
/// Why:
///   `touch` records activity so the idle timer only counts time with no connection and no request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Shared { touch(): void }
/// ```
impl Shared {
    /// What:
    ///  Note that something happened,
    /// restarting the idle clock.
    /// Why:
    ///  Idle means "no connection opened or closed for a while".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// touch(): void
    /// ```
    pub fn touch(&self) {
        if let Ok(mut last) = self.last_activity.lock() {
            *last = Instant::now();
        }
    }
}

/// What:
///  Whether the server should stop:
/// asked to,
/// idle too long,
/// or its worktree is gone or replaced.
/// Why:
///  A server must not outlive the directory it serves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shouldStop(shared: Shared): boolean
/// ```
fn should_stop(shared: &Shared) -> bool {
    if shared.shutdown.load(Ordering::SeqCst) {
        return true;
    }
    if worktree_identity(&shared.options.root).ok().as_ref() != Some(&shared.worktree) {
        return true;
    }
    if shared.connections.load(Ordering::SeqCst) > 0 {
        return false;
    }
    let idle_for = match shared.last_activity.lock() {
        Ok(last) => last.elapsed(),
        Err(_) => Duration::default(),
    };
    return idle_for >= shared.options.idle;
}

/// What:
///  Bind the socket and serve until `should_stop`;
/// remove the socket on the way out.
/// Why:
///  The accept loop is non-blocking so the idle and worktree checks run every few milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function serve(options: ServerOptions, registry: RuleRegistry): Promise<void>
/// ```
pub fn serve(options: &ServerOptions, registry: RuleRegistry) -> Result<(), Diagnostic> {
    let worktree = worktree_identity(&options.root)?;
    let listener = match UnixListener::bind(&options.socket) {
        Ok(listener) => listener,
        Err(error) => {
            let message = format!("cannot bind the socket {}: {error}", options.socket.display());
            return Err(Diagnostic::new("socket-bind-failed", &message));
        }
    };
    if let Err(error) = fs::set_permissions(&options.socket, fs::Permissions::from_mode(0o600)) {
        let _ = fs::remove_file(&options.socket);
        let message = format!("cannot restrict the socket {}: {error}", options.socket.display());
        return Err(Diagnostic::new("socket-bind-failed", &message));
    }
    if let Err(error) = listener.set_nonblocking(true) {
        let _ = fs::remove_file(&options.socket);
        let message = format!("cannot make the socket non-blocking: {error}");
        return Err(Diagnostic::new("socket-bind-failed", &message));
    }
    let shared = Arc::new(Shared {
        options: options.clone(),
        registry,
        run_lock: Mutex::new(()),
        shutdown: AtomicBool::new(false),
        connections: AtomicUsize::new(0),
        last_activity: Mutex::new(Instant::now()),
        worktree,
        build: build_identity(),
    });
    while !should_stop(&shared) {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                shared.connections.fetch_add(1, Ordering::SeqCst);
                shared.touch();
                let connection_state = Arc::clone(&shared);
                std::thread::spawn(move || {
                    handle_connection(&connection_state, stream);
                    connection_state.connections.fetch_sub(1, Ordering::SeqCst);
                    connection_state.touch();
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
    let _ = fs::remove_file(&options.socket);
    // Let an in-flight run finish before the process ends: it holds the run lock.
    let _guard = shared.run_lock.lock();
    return Ok(());
}

//! The files each language server asked to hear about,
//!  through `client/registerCapability` for
//! `workspace/didChangeWatchedFiles`,
//!  and the changes waiting to be sent to them.
//!
//! helix-lsp's own handler (`helix-lsp/src/file_event.rs` at the pinned revision) keeps only string glob
//! patterns,
//!  ignores the kinds a watcher asks for,
//!  and always sends "changed".
//!  Helix declares support for
//! relative patterns,
//!  and both rust-analyzer and the TypeScript 7 server register relative patterns,
//!  so
//! this module keeps the registrations instead,
//!  per the Language Server Protocol 3.17:
//! - a watcher is a glob pattern,
//!    either a string or a pattern relative to a base address;
//! - `*` and `?` stay inside one path segment,
//!    `**` spans any number of segments,
//!    `{a,b}` and `[...]` group;
//! - a watcher's kind says which of created,
//!    changed,
//!    and deleted it wants;
//!    it defaults to all three.
//!
//! Changes are gathered per path,
//!  so a burst (a build writing many files) becomes one notification per
//! server in which each path appears once,
//!  with the kind that matches its final state on disk.

/// The changes the change watcher forwards.
use crate::change_watch::{ServerChange, ServerChangeKind};
/// What:
///  `GlobBuilder` compiles one glob pattern;
///  `GlobMatcher` tests paths against it.
/// Why:
///  The protocol's glob syntax is what globset implements,
///  with `literal_separator` keeping `*` and
///      `?` inside one path segment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { minimatch } from 'minimatch';
/// ```
use globset::{GlobBuilder, GlobMatcher};
/// The protocol's data types.
use helix_lsp::lsp;
/// What:
///  `BTreeMap` is an ordered map (sibling:
///  `HashMap`,
///  unordered);
///  `Duration`/`Instant` are a time span
///       and a monotonic time point.
/// Why:
///  Pending changes are sent in path order,
///  which keeps notifications reproducible in tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending = new Map<string, Kind>();
/// ```
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// A burst is sent once no change arrived for this long.
pub(super) const FORWARD_QUIET: Duration = Duration::from_millis(50);

/// A burst that never pauses is still sent this long after its first change.
pub(super) const FORWARD_LIMIT: Duration = Duration::from_millis(500);

/// What:
///  Combine the kind already pending for a path with a later one,
///  so the result describes the path's
///       final state.
///  `Option<ServerChangeKind>` is the pending kind,
///  or `None` for the first change.
/// Why:
///  A deletion after a creation must still arrive as a deletion;
///  a file replaced by a deletion and a
///      creation still exists,
///  so it changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function merge(previous: Kind | undefined, next: Kind): Kind
/// ```
pub fn merge(previous: Option<ServerChangeKind>, next: ServerChangeKind) -> ServerChangeKind {
    // What: `match` on the pair; `Some(ServerChangeKind::Deleted | ServerChangeKind::Changed)` matches either.
    // Why: The latest event decides whether the path exists; the earlier one decides whether it is new.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (next === 'deleted') return 'deleted';
    // if (next === 'created') return previous === 'deleted' || previous === 'changed' ? 'changed' : 'created';
    // return previous === 'created' ? 'created' : 'changed';
    // ```
    match (previous, next) {
        (_, ServerChangeKind::Deleted) => {
            return ServerChangeKind::Deleted;
        }
        (
            Some(ServerChangeKind::Deleted | ServerChangeKind::Changed),
            ServerChangeKind::Created,
        ) => {
            return ServerChangeKind::Changed;
        }
        (_, ServerChangeKind::Created) => {
            return ServerChangeKind::Created;
        }
        (Some(ServerChangeKind::Created), ServerChangeKind::Changed) => {
            return ServerChangeKind::Created;
        }
        (_, ServerChangeKind::Changed) => {
            return ServerChangeKind::Changed;
        }
    }
}

/// One watcher a server registered.
#[derive(Debug)]
struct Watcher {
    /// The folder a relative pattern is matched against;
    ///  `None` for a string pattern.
    base: Option<PathBuf>,
    /// The compiled glob pattern.
    glob: GlobMatcher,
    /// The kinds the watcher wants.
    kinds: lsp::WatchKind,
}

/// Compile one protocol glob pattern,
///  or `None` when it is not a valid pattern.
fn compile(pattern: &str) -> Option<GlobMatcher> {
    // What: `literal_separator(true)` stops `*` and `?` at `/`; `build` returns `Result`; `ok()` turns it
    //       into an `Option`; `map` compiles the matcher.
    // Why: The protocol says `*` and `?` match inside one path segment only.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compileGlob(pattern, { literalSeparator: true }); } catch { return undefined; }
    // ```
    let built = GlobBuilder::new(pattern).literal_separator(true).build();
    // What: `match` on the build result: a glob, or the reason the pattern is invalid.
    // Why: One invalid watcher is skipped; the server's other watchers still work.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (built instanceof Error) { log.debug(built); return undefined; }
    // ```
    match built {
        Ok(glob) => {
            return Some(glob.compile_matcher());
        }
        Err(error) => {
            tracing::debug!(pattern, %error, "skipped a file watcher whose glob pattern is invalid");
            return None;
        }
    }
}

/// Turn one registered watcher into a matcher,
///  or `None` when its pattern or base is unusable.
fn watcher(source: lsp::FileSystemWatcher) -> Option<Watcher> {
    // What: `unwrap_or` substitutes all three kinds when the watcher named none.
    // Why: The protocol's default is created, changed, and deleted (7).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kinds = source.kind ?? (CREATE | CHANGE | DELETE);
    // ```
    let kinds = source.kind.unwrap_or(lsp::WatchKind::all());
    match source.glob_pattern {
        lsp::GlobPattern::String(pattern) => {
            return Some(Watcher {
                base: None,
                glob: compile(&pattern)?,
                kinds,
            });
        }
        lsp::GlobPattern::Relative(relative) => {
            // What: `OneOf::Left` is a workspace folder and `OneOf::Right` a plain address; both name a folder.
            // Why: Either form is allowed as the base of a relative pattern.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const address = 'uri' in relative.baseUri ? relative.baseUri.uri : relative.baseUri;
            // ```
            let address = match relative.base_uri {
                lsp::OneOf::Left(folder) => folder.uri,
                lsp::OneOf::Right(address) => address,
            };
            // `to_file_path` fails for an address that is not a local file; the watcher is then skipped.
            let base = address.to_file_path().ok()?;
            return Some(Watcher {
                base: Some(base),
                glob: compile(&relative.pattern)?,
                kinds,
            });
        }
    }
}

/// The watcher's kind bit for a change.
fn wanted_kind(kind: ServerChangeKind) -> lsp::WatchKind {
    return match kind {
        ServerChangeKind::Created => lsp::WatchKind::Create,
        ServerChangeKind::Changed => lsp::WatchKind::Change,
        ServerChangeKind::Deleted => lsp::WatchKind::Delete,
    };
}

/// The protocol's kind number for a change.
fn protocol_kind(kind: ServerChangeKind) -> lsp::FileChangeType {
    return match kind {
        ServerChangeKind::Created => lsp::FileChangeType::CREATED,
        ServerChangeKind::Changed => lsp::FileChangeType::CHANGED,
        ServerChangeKind::Deleted => lsp::FileChangeType::DELETED,
    };
}

/// What:
///  What `take` asks of its caller:
///  how servers spell a path,
///  and whether a server still runs.
///  A type that implements this trait carries the state those answers need.
/// Why:
///  A named type carries the answers without an anonymous function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WatchHost<Server> { respell(path: string): string; live(server: Server): boolean }
/// ```
pub(super) trait WatchHost<Server> {
    /// The spelling of a resolved path that the servers use for the project root.
    fn respell(&self, path: &Path) -> PathBuf;
    /// Whether the server still runs.
    fn live(&self, server: Server) -> bool;
}

/// True when `watcher` wants this change of `path`,
///  spelled as the server spells the project root.
/// A string pattern that does not match the whole path is also tried against the path below `root`.
fn matches(watcher: &Watcher, path: &Path, root: &Path, kind: ServerChangeKind) -> bool {
    if !watcher.kinds.contains(wanted_kind(kind)) {
        return false;
    }
    // What: `match &watcher.base` borrows the optional base; `strip_prefix` gives the path below it.
    // Why: A relative pattern is matched against the part of the path below its base.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (base !== undefined) return isInside(path, base) && glob.matches(relative(base, path));
    // ```
    match &watcher.base {
        Some(base) => {
            return match path.strip_prefix(base) {
                Ok(below) => watcher.glob.is_match(below),
                Err(_) => false,
            };
        }
        None => {
            if watcher.glob.is_match(path) {
                return true;
            }
            return match path.strip_prefix(root) {
                Ok(below) => watcher.glob.is_match(below),
                Err(_) => false,
            };
        }
    }
}

/// What:
///  Registrations per server and the changes not yet sent.
///  `Server` is the key that names a server:
///       helix-lsp's `LanguageServerId` in the worker,
///  a plain number in tests.
/// Why:
///  helix-lsp hands out its keys only to running servers,
///  so tests name servers themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WatchedFiles<Server> { }
/// ```
#[derive(Debug)]
pub(super) struct WatchedFiles<Server> {
    /// What:
    ///  per server,
    ///  its registration identifiers and their watchers (`Map<Server, Map<string, Watcher[]>>`).
    /// Why:
    ///  A server may register and unregister watchers by identifier at any time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const registrations = new Map<ServerId, Map<string, Watcher[]>>();
    /// ```
    registrations: HashMap<Server, HashMap<String, Vec<Watcher>>>,
    /// Each changed path with the kind matching its final state,
    ///  in path order.
    pending: BTreeMap<PathBuf, ServerChangeKind>,
    /// When the first pending change arrived.
    first: Option<Instant>,
    /// When the latest pending change arrived.
    last: Option<Instant>,
}

/// Nothing registered and nothing pending.
impl<Server> Default for WatchedFiles<Server> {
    /// An empty set of registrations.
    fn default() -> Self {
        return Self {
            registrations: HashMap::new(),
            pending: BTreeMap::new(),
            first: None,
            last: None,
        };
    }
}

/// What:
///  Registration,
///  gathering,
///  and sending,
///  for any key type that can be copied,
///  compared,
///  hashed,
///       and printed (`Copy + Eq + Hash + Display` are those abilities).
/// Why:
///  The map needs to hash and compare keys;
///  the log prints them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WatchedFiles<Server extends Key> { }
/// ```
impl<Server: Copy + Eq + Hash + Display> WatchedFiles<Server> {
    /// Keep the usable watchers of one registration,
    ///  replacing an earlier one with the same identifier.
    pub(super) fn register(
        &mut self,
        server: Server,
        identifier: String,
        options: lsp::DidChangeWatchedFilesRegistrationOptions,
    ) {
        let requested = options.watchers.len();
        // What: `into_iter` takes the watchers by value; `filter_map` keeps the ones that compile.
        // Why: An unusable watcher (invalid pattern, non-file base) is skipped on its own.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const usable = options.watchers.map(watcher).filter(Boolean);
        // ```
        let usable: Vec<Watcher> = options.watchers.into_iter().filter_map(watcher).collect();
        tracing::debug!(%server, identifier, requested, usable = usable.len(), "a language server registered file watchers");
        self.registrations
            .entry(server)
            .or_default()
            .insert(identifier, usable);
    }

    /// Forget one registration;
    ///  a server with none left hears about nothing.
    pub(super) fn unregister(&mut self, server: Server, identifier: &str) {
        if let Some(server_registrations) = self.registrations.get_mut(&server) {
            server_registrations.remove(identifier);
            if server_registrations.is_empty() {
                self.registrations.remove(&server);
            }
        }
        tracing::debug!(%server, identifier, "a language server removed file watchers");
    }

    /// True while some server has a watcher,
    ///  so the project's folders are worth watching.
    pub(super) fn wanted(&self) -> bool {
        for server in self.registrations.values() {
            for watchers in server.values() {
                if !watchers.is_empty() {
                    return true;
                }
            }
        }
        return false;
    }

    /// Gather one change;
    ///  true when it is the first of a burst,
    ///  so the caller schedules sending.
    pub(super) fn record(&mut self, change: ServerChange, now: Instant) -> bool {
        // Nothing is gathered while no server could want it.
        if self.registrations.is_empty() {
            return false;
        }
        let previous = self.pending.get(&change.path).copied();
        self.pending
            .insert(change.path, merge(previous, change.kind));
        self.last = Some(now);
        if self.first.is_none() {
            self.first = Some(now);
            return true;
        }
        return false;
    }

    /// How long to wait before sending:
    ///  `None` with nothing pending,
    ///  zero when the burst is due.
    pub(super) fn wait(&self, now: Instant) -> Option<Duration> {
        let (first, last) = (self.first?, self.last?);
        let quiet = FORWARD_QUIET.saturating_sub(now.saturating_duration_since(last));
        let limit = FORWARD_LIMIT.saturating_sub(now.saturating_duration_since(first));
        return Some(quiet.min(limit));
    }

    /// What:
    ///  Take the pending changes as one list of protocol events per server that wants some of them.
    ///       `host.respell` turns a resolved path into the spelling the servers use for the project root,
    ///  which
    ///       is `root`;
    ///  `host.live` says whether a server still runs,
    ///  and the registrations of one that does not
    ///       are forgotten.
    /// Why:
    ///  Servers that registered nothing,
    ///  or whose watchers match none of the changes,
    ///  get nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// take(host: WatchHost, root: string): [ServerId, FileEvent[]][]
    /// ```
    pub(super) fn take(
        &mut self,
        host: &dyn WatchHost<Server>,
        root: &Path,
    ) -> Vec<(Server, Vec<lsp::FileEvent>)> {
        let pending = std::mem::take(&mut self.pending);
        self.first = None;
        self.last = None;
        // The servers that stopped are listed first, because the table cannot change while it is walked.
        let mut stopped: Vec<Server> = Vec::new();
        for server in self.registrations.keys() {
            if !host.live(*server) {
                tracing::debug!(%server, "forgot the file watchers of a language server that stopped");
                stopped.push(*server);
            }
        }
        for server in stopped {
            self.registrations.remove(&server);
        }
        // The respelled paths are gathered once, so each server does not respell them again.
        let mut spelled: Vec<(PathBuf, ServerChangeKind)> = Vec::new();
        for (path, kind) in pending {
            spelled.push((host.respell(&path), kind));
        }
        let mut batches = Vec::new();
        for (server, server_registrations) in &self.registrations {
            let mut events = Vec::new();
            for (path, kind) in &spelled {
                let mut wanted = false;
                for watcher in server_registrations.values().flatten() {
                    if matches(watcher, path, root, *kind) {
                        wanted = true;
                        break;
                    }
                }
                if !wanted {
                    continue;
                }
                // `from_file_path` fails only for a relative path, which the change watcher never sends.
                if let Ok(uri) = lsp::Url::from_file_path(path) {
                    events.push(lsp::FileEvent::new(uri, protocol_kind(*kind)));
                }
            }
            if !events.is_empty() {
                batches.push((*server, events));
            }
        }
        return batches;
    }
}

/// The merge rule,
///  glob matching,
///  kinds,
///  and per-server batches,
///  without a server.
#[cfg(test)]
#[path = "watched_files_tests.rs"]
mod tests;

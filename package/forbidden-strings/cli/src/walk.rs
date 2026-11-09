//! walk support for the forbidden-strings scanner:
//!  the working-tree walk and the tracked-file union read from Git's own index.
/// Imports dependencies used by this module.
// What:     `use ignore::WalkBuilder;` imports the type that builds a
//           filesystem walker honoring `.gitignore`, hidden-file rules,
//           and parent-directory ignore files. `ignore` is the crate
//           ripgrep uses for its file walking.
// Why:      `--all` mode walks the working tree to enumerate every file
//           we should scan; `WalkBuilder` does this in parallel and
//           respects `.gitignore` semantics (including `!` negations).
//
// In TS you'd write (pseudocode):
// ```ts
// import { WalkBuilder } from "<no-direct-equivalent>";
// ```
use ignore::WalkBuilder;

/// Imports dependencies used by this module.
// What:     `use ignore::WalkState;` imports the enum returned by the
//           parallel walker's per-entry callback to control whether to
//           keep walking, skip the current subtree, or quit entirely.
// Why:      The parallel walker wants the callback to say
//           `WalkState::Continue` after handling each entry.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent; conceptually a return value steering the walker.
// ```
use ignore::WalkState;

/// Imports the entry type the walker hands the filter and the visitors,
/// and the two traits a worker visitor and its maker implement.
use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder};

/// Imports dependencies used by this module.
// What:     `use anyhow::{anyhow, Result};` imports `anyhow`'s error-construction
//           macro and result alias for the walker boundary.
// Why:      The parallel collector can report lock poisoning through the same
//           application error channel as rule loading.
//
// In TS you'd write (pseudocode):
// ```ts
// type Result<T> = T; // failures throw Error objects
// ```
use anyhow::{anyhow, Result};

/// Imports dependencies used by this module.
// What:     `use std::sync::{Arc, Mutex};` imports two thread-safe
//           wrappers from the standard library.
//             - `Arc<T>` ("atomically reference-counted") is a heap-
//               allocated `T` whose ownership is shared across multiple
//               owners; cloning bumps a refcount, dropping decrements
//               it, the inner `T` is freed when the count hits zero.
//             - `Mutex<T>` is a mutual-exclusion lock guarding a `T`;
//               `lock()` blocks until the current thread is the holder.
// Why:      The parallel walker spawns multiple threads, each calling
//           our callback concurrently. To collect file paths from all
//           threads into a shared `Vec`, we wrap the `Vec` in
//           `Arc<Mutex<...>>`: `Arc` to share across threads, `Mutex`
//           to serialize push operations.
// Gotcha:   `Arc::clone(&x)` is cheap (atomic increment), NOT a deep
//           copy. The pointee is the same `Mutex<Vec>`.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent. Imagine SharedArrayBuffer + a worker-pool lock.
// ```
use std::sync::{Arc, Mutex};

/// Whether the walker may descend into an entry:
/// everything except the `.git` and `.jj` directories,
/// whose version-control internals are not content to scan.
fn is_not_version_control_directory(entry: &DirEntry) -> bool {
    return entry.file_name() != ".git" && entry.file_name() != ".jj";
}

/// Hands the parallel walker one visitor per worker thread,
/// each pushing the files it sees onto one shared list.
struct FileCollector {
    /// The list every visitor pushes onto.
    files: Arc<Mutex<Vec<String>>>,
}

/// Lets the walker ask the collector for a visitor per worker thread.
impl<'scope> ParallelVisitorBuilder<'scope> for FileCollector {
    /// One visitor for one worker thread.
    fn build(&mut self) -> Box<dyn ParallelVisitor + 'scope> {
        return Box::new(FileVisitor { files: Arc::clone(&self.files) });
    }
}

/// The visitor of one worker thread.
struct FileVisitor {
    /// The list every visitor pushes onto.
    files: Arc<Mutex<Vec<String>>>,
}

/// Lets the walker hand each entry its thread walks to this visitor.
impl ParallelVisitor for FileVisitor {
    /// Records the entry when it is a regular file with a UTF-8 path,
    /// and always lets the walk continue;
    /// an entry that errored (for example permission denied on a subtree) is skipped silently.
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
        let Ok(found) = entry else {
            return WalkState::Continue;
        };
        let is_file: bool = match found.file_type() {
            Some(file_type) => file_type.is_file(),
            None => false,
        };
        if !is_file {
            return WalkState::Continue;
        }
        if let Some(text) = found.path().to_str() {
            self.files
                .lock()
                .expect("walk-results mutex poisoned by a panicking walker thread")
                .push(text.to_string());
        }
        return WalkState::Continue;
    }
}

/// The fatal error for a result list whose mutex a panicking walker thread poisoned.
fn walk_poisoned<Poison: std::fmt::Display>(poison: Poison) -> anyhow::Error {
    return anyhow!("walk poisoned: {}", poison);
}

/// What one `--all` listing produced.
pub(crate) struct WorkingTreeListing {
    /// Every path to scan,
    ///  as the walker spells them and as this module joins the index's own.
    pub(crate) files: Vec<String>,
    /// Why the tracked-file union is missing or incomplete,
    ///  so a scan that cannot cover force-added files says so instead of
    ///  quietly reporting less.
    pub(crate) union_gap: Option<UnionGap>,
}

/// Why the tracked-file union is unavailable.
///
/// The path is kept apart from the words so the caller renders it through the
///  pathname scan,
///  which masks a component matching a rule and escapes protocol-sensitive
///  characters,
///  exactly as every other path in a diagnostic is rendered.
///  Printing a raw path here would let the one message about a path we could
///  not read be the one place a pathname leaks.
pub(crate) struct UnionGap {
    /// The file the scan could not use,
    ///  either a `.git` entry or an index.
    pub(crate) at: std::path::PathBuf,
    /// What state it was in.
    pub(crate) cause: String,
}

/// One listed path reduced to the form two spellings of the same path agree on:
///  `.` components dropped and separators normalized,
///  with nothing resolved against the filesystem.
///
/// The walker reports `<root>/<entry>` and this module joins index entries the
///  same way,
///  so the two differ only in `.` components
///  (`./x` from the walker of root `.` against `./x` built by the join).
///  Neither side carries `..`,
///  which is why nothing here collapses one.
fn listing_key(path: &str) -> Vec<u8> {
    let mut without_dots: std::path::PathBuf = std::path::PathBuf::new();
    for part in std::path::Path::new(path).components() {
        if part != std::path::Component::CurDir {
            without_dots.push(part.as_os_str());
        }
    }
    return crate::path_name_bytes::normalized_path(&without_dots)
}

/// What state the index at `index_path` is in,
///  given that `gix-index` could not open it.
///
/// The error text is kept as the scanner received it and the file is asked
///  what state it is in,
///  because the two say different things:
///  a missing or permission-denied file is an operating-system refusal,
///  while a file that opens but does not parse is a format or extension the
///  decoder does not handle.
///  Reopening is safe here:
///  Git never writes into the same index file,
///  it replaces it.
fn unreadable_index_cause(index_path: &std::path::Path, error: &impl std::fmt::Display) -> String {
    return match std::fs::File::open(index_path) {
        Err(open_error) => format!("it cannot be read ({})", open_error),
        Ok(_) => format!("it does not hold a Git index this scanner can parse ({})", error),
    }
}

/// Implements `list_files`.
// What:     `pub fn list_files(root: &str) -> Result<WorkingTreeListing>`
//           walks the working tree starting at `root` and returns the
//           paths to scan (UTF-8) plus, when a repository was found but
//           its index could not be read, why the tracked-file union is
//           missing.
// Why:      `--all` mode calls this once to get every scannable file.
//           The `Result` shape lets us propagate walk errors through
//           anyhow, matching the rest of the binary's error style, and
//           the union gap reaches the caller as a value so the CLI prints
//           it in its own deterministic order rather than from a walker
//           thread.
//
// In TS you'd write (pseudocode):
// ```ts
// export function listFiles(root: string): { files: string[]; unionGap: string | null } {
//   const files = walkBuilder(root).hidden(false).ignore(false).filterEntry(vcsFilter).buildParallel().map((e) => e.path);
//   // union with the working tree's own index below
//   return { files, unionGap };
// }
// ```
pub fn list_files(root: &str) -> Result<WorkingTreeListing> {
    // What:     `Arc::new(Mutex::new(Vec::new()))` allocates an empty
    //           `Vec<String>`, wraps it in a `Mutex`, then heap-
    //           allocates that mutex behind an atomically-refcounted
    //           pointer. We will clone this `Arc` into each worker
    //           closure so every thread can lock-and-push.
    // Why:      Need a shared collection for the parallel walker to
    //           write into.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const files = makeSharedArray<string>();
    // ```
    let shared_files: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // What:     `WalkBuilder::new(root).hidden(false).ignore(false)
    //           .filter_entry(...).build_parallel()` configures and
    //           builds a parallel walker.
    //             - `hidden(false)`: include dotfiles (`.github/`,
    //               `.gitignore`, etc.): git tracks these, so we
    //               must scan them.
    //             - `ignore(false)`: do NOT honor `.ignore` files
    //               (used by tools like `scc`); the repo's `.ignore`
    //               re-excludes things `.gitignore` deliberately
    //               re-includes via `!` negations, so reading
    //               `.ignore` would silently drop tracked files from
    //               the scan set.
    //             - `filter_entry(|e| ...)`: stops the walker from
    //               descending into `.git` and `.jj` directories
    //               (VCS internals are huge and not user content).
    //               We don't rely on `hidden(true)` to do this,
    //               because we need other dotdirs (`.github/`, etc.)
    //               to be visited.
    //             - `build_parallel()`: returns a parallel walker
    //               that runs a callback across worker threads.
    // Why:      Replaces the previous `git ls-files` subprocess with
    //           an in-process walk. On this repo the parallel walker
    //           is ~9x faster than the subprocess; the walker also
    //           drops the runtime dependency on `git` being on PATH.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const walker = walkBuilder(root, {
    //   hidden: false,
    //   ignoreFile: false,
    //   filterEntry: (e) => e.fileName !== ".git" && e.fileName !== ".jj",
    //   parallel: true,
    // });
    // ```
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(false)
        .filter_entry(is_not_version_control_directory)
        .build_parallel();

    // The walker asks the collector for one visitor per worker thread,
    // and each visitor sees every entry its thread walks.
    // Each visitor holds a clone of the shared list,
    // so every worker locks it only to push a path.
    let mut collector = FileCollector { files: Arc::clone(&shared_files) };
    walker.visit(&mut collector);

    // What:     `Arc::try_unwrap(files)` succeeds if this `Arc` is the
    //           sole remaining handle, returning the `Mutex<Vec>`
    //           directly; otherwise returns the `Arc` back as `Err`.
    //           Then `.into_inner()` unwraps the `Mutex` into its
    //           inner `Vec`. We fall back to cloning the contents if
    //           somehow extra refcounts remain (the walker should
    //           have dropped them by now, but a defensive fallback
    //           costs nothing).
    // Why:      Returning the inner `Vec` by value is cheaper than
    //           cloning; we only fall back when the optimization
    //           is unavailable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const out = files.takeOrClone();
    // return out;
    // ```
    let mut files = match Arc::try_unwrap(shared_files) {
        Ok(m) => m.into_inner().map_err(walk_poisoned)?,
        Err(arc) => arc.lock().map_err(walk_poisoned)?.clone(),
    };

    // What:     Union with paths read from this working tree's own Git
    //           index via `gix-index` to recover tracked files the walker
    //           did not visit. The `ignore` crate's `WalkBuilder` always
    //           honours .gitignore (the `.ignore(false)` toggle disables
    //           `.ignore` files, not `.gitignore`). A file that was
    //           force-added with `git add -f` despite matching a
    //           `.gitignore` pattern is tracked by git but skipped by the
    //           walker, leaving a silent gap in `--all` mode -- a
    //           secret-scan on push-to-main must cover every tracked file.
    // Why:      Closes BUG 3 without paying the ~88 ms subprocess wall
    //           the previous `git ls-files --cached --ignored
    //           --exclude-standard -z` invocation cost on Mono
    //           (350 ms on Linux). Reading the index in-process with
    //           `gix-index` is single-digit milliseconds: the same
    //           binary index format `git ls-files` parses, without the
    //           fork+exec+repo-discovery overhead.
    //
    //           We read EVERY index entry (not just gitignored ones),
    //           then dedup against the walker output. The set
    //           difference `index - walker` is exactly the gitignored
    //           tracked files plus any tracked file that vanished from
    //           the workdir between commit and scan. Either case is
    //           valid to report: vanished tracked files surface as
    //           "read error" hits via BUG 4's fix, and gitignored
    //           tracked files are the force-added set we need.
    //
    //           The index is the one belonging to the working tree the
    //           scan walks, which `git_working_tree::discover` finds from
    //           the walked directory itself: a linked worktree and a
    //           submodule keep theirs behind a `.git` file's `gitdir:`
    //           line, and a scan started in a subfolder keeps it at an
    //           ancestor. Joining `.git/index` onto the walked directory
    //           found the first of those nowhere and the last two not at
    //           all (issue 1232).
    //
    //           Index entries are recorded from the working tree root, so
    //           a subfolder scan keeps only the entries below itself and
    //           drops the prefix they carry; adding them unchanged would
    //           scan files outside the directory the caller named.
    //
    //           A directory with no repository above it keeps the previous
    //           behaviour: walker-only output and no diagnostic, since
    //           nothing there claims to be a working tree. Inside a real
    //           working tree whose index is missing or unreadable, the
    //           scan still runs but its caller is told the union is
    //           unavailable, because scanning less in silence would let a
    //           tracked secret in an ignored file pass the gate.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const discovery = discoverWorkingTree(root);
    // let unionGap = null;
    // if (discovery.kind === "unusable") unionGap = { at: discovery.entry, cause: discovery.cause };
    // if (discovery.kind === "working-tree") {
    //   const idx = await openIndex(discovery.tree.indexPath, discovery.tree.hashKind());
    //   for (const entry of idx.entries) {
    //     const below = discovery.tree.belowWalkRoot(entry.path);   // null outside the walked directory
    //     if (below !== null) files.push(join(root, below));
    //   }
    // }
    // ```
    let root_path = std::path::Path::new(root);
    let mut union_gap: Option<UnionGap> = None;
    let mut working_tree: Option<crate::git_working_tree::GitWorkingTree> = None;
    match crate::git_working_tree::discover(root_path) {
        crate::git_working_tree::Discovery::WorkingTree(tree) => working_tree = Some(tree),
        crate::git_working_tree::Discovery::NoRepository => {}
        crate::git_working_tree::Discovery::Unusable { entry, cause } => {
            union_gap = Some(UnionGap { at: entry, cause });
        }
    }
    if let Some(tree) = working_tree {
        let index_path = tree.index_path();
        let opened = gix_index::File::at(
            &index_path,
            crate::git_working_tree::index_hash_kind(&tree.config_path()),
            // What:     `true` skips the trailing-checksum verification
            //           that `gix-index` would otherwise re-hash the entire
            //           file to validate. We do not need integrity here;
            //           git itself owns index integrity, and a corrupt
            //           index will surface on the next git operation.
            //           Skipping the hash check is bench-meaningful: it
            //           drops index-open from ~5 ms to <1 ms on Mono.
            // Why:      Match the perf target: in-process index read should
            //           be a few milliseconds total, not match the
            //           subprocess wall it replaces.
            true,
            gix_index::decode::Options::default(),
        );
        match opened {
            Ok(index) => {
                // What:     Build a HashSet of normalized paths already in
                //           `files` so we can detect duplicates with the
                //           index output. The walker spells a path relative
                //           to the walked directory (`./x`), by absolute
                //           path when the root is absolute, and the index
                //           records it from the working tree root; one
                //           lexical key makes all three compare equal.
                // Why:      Without deduplication tracked-non-ignored files
                //           would scan twice (once via walker, once via index).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const seen = new Set(files.map((p) => lexicalKey(p)));
                // ```
                let mut seen: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
                for listed_above in &files {
                    seen.insert(listing_key(listed_above));
                }
                for entry in index.entries() {
                    // What:     Filter index entries by `Mode`. Only regular
                    //           files (`Mode::FILE`, `Mode::FILE_EXECUTABLE`)
                    //           are scannable; `Mode::COMMIT` is a submodule
                    //           gitlink whose path on disk is a directory (or
                    //           absent), `Mode::SYMLINK` would let `fs::read`
                    //           follow into content outside the repo, and
                    //           `Mode::DIR` (sparse-checkout) is non-content.
                    //           Without this filter, every tracked submodule
                    //           surfaces as a `"./<sub>: Is a directory"` hit
                    //           via BUG 4's read-error path; on the Linux
                    //           kernel that produces ~12 false-positive lines
                    //           per scan.
                    // Why:      Match the original subprocess semantics.
                    //           `git ls-files --cached --ignored
                    //           --exclude-standard -z` filtered to gitignored
                    //           entries; submodules are almost never
                    //           gitignored, so the subprocess output naturally
                    //           excluded them. The in-process replacement
                    //           reads ALL index entries, so we must mode-filter
                    //           explicitly.
                    let mode = entry.mode;
                    if !mode.contains(gix_index::entry::Mode::FILE)
                        && !mode.contains(gix_index::entry::Mode::FILE_EXECUTABLE)
                    {
                        continue;
                    }
                    // What:     `entry.path(&index)` resolves the entry's path
                    //           backing range into a `&BStr` (byte string) slice
                    //           of the index's shared path-storage buffer. The
                    //           bytes are exactly what `git ls-files -z` would
                    //           emit for this entry, recorded from the working
                    //           tree root.
                    // Why:      The path below the walked directory is what we
                    //           feed downstream; index entries carry neither the
                    //           walker's leading `./` nor, for a subfolder scan,
                    //           the directory the caller named.
                    let path_bytes: &[u8] = entry.path(&index);
                    let Some(below) = tree.below_walk_root(path_bytes) else {
                        continue;
                    };
                    // What:     `std::str::from_utf8(below)` validates that
                    //           the path is UTF-8. Non-UTF-8 paths are silently
                    //           skipped to match the walker's existing
                    //           semantics (the walker uses `Path::to_str()`,
                    //           which also requires UTF-8).
                    // Why:      Every downstream consumer expects `&str`; a
                    //           single non-UTF-8 path would force `OsString`
                    //           plumbing across the binary for vanishingly rare
                    //           cases.
                    let Ok(relative) = std::str::from_utf8(below) else {
                        continue;
                    };
                    let Some(listed) = root_path.join(relative).to_str().map(str::to_string) else {
                        continue;
                    };
                    if seen.insert(listing_key(&listed)) {
                        files.push(listed);
                    }
                }
            }
            Err(error) => {
                let cause: String = unreadable_index_cause(&index_path, &error);
                union_gap = Some(UnionGap { at: index_path, cause });
            }
        }
    }

    return Ok(WorkingTreeListing { files, union_gap })
}

/// Registers the `tests` child module.
// What:     `#[cfg(test)] #[path = "walk_tests.rs"] mod tests;` declares a
//           test-only submodule whose code lives in the sibling file
//           `walk_tests.rs`. `#[cfg(test)]` is a conditional-compilation
//           gate (like `#ifdef TEST` in C): the module compiles only under
//           `cargo nextest run` / `cargo test`, never in the release binary.
//           `#[path = "..."]` aims the module at a flat sibling file instead
//           of the default `walk/tests.rs` subdirectory lookup. Because the
//           file is still the `tests` CHILD of `walk`, its `use super::*`
//           reaches `walk.rs`'s private items (e.g. `detect_index_hash_kind`)
//           unchanged.
// Why:      Keep `walk.rs` focused on production code; the verbose
//           git-fixture tests live beside it without inflating this file or
//           consuming its max-lines budget (sibling `*_tests.rs` files are
//           exempt from the linter).
//
// In TS you'd write (pseudocode):
// ```ts
// // walk.unit.test.ts, run only by the test runner
// ```
#[cfg(test)]
#[path = "walk_tests.rs"]
mod tests;

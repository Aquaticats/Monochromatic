//! walk support for the forbidden-strings scanner.
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

/// A listed path without its leading `./`.
fn without_leading_dot_slash(path: &str) -> String {
    return path.trim_start_matches("./").to_string();
}

/// Implements `detect_index_hash_kind`.
// What:     `fn detect_index_hash_kind(repo_root) -> gix_hash::Kind`
//           reads `.git/config` and returns `Sha256` when the repository
//           was initialized with `git init --object-format=sha256`,
//           otherwise `Sha1`. Detection runs in a few hundred
//           microseconds (config files are tiny).
// Why:      `gix_index::File::at(...)` parses entries based on the
//           hash kind: 20-byte object IDs and 20-byte trailer for
//           Sha1, 32-byte for Sha256. Calling with the wrong kind on
//           a Sha256 index produces silently corrupted paths because
//           each entry's flags + path offsets land 12 bytes off. We
//           never compute hashes ourselves (skip_hash = true) but we
//           must still know the right ID length to parse correctly.
//           Configurations using SHA-256 are still rare today, but
//           Git's SHA-256 transition is moving forward; supporting
//           the case keeps BUG 3's fix sound across object formats.
//
// In TS you'd write (pseudocode):
// ```ts
// function detectIndexHashKind(repoRoot: string): "sha1" | "sha256" {
//   const config = fs.readFileSync(path.join(repoRoot, ".git/config"), "utf8");
//   let inExtensions = false;
//   for (const raw of config.split("\n")) {
//     const line = raw.split(/[#;]/, 1)[0].trim();
//     if (line === "") continue;
//     const section = line.match(/^\[(.+?)(?:\s.+)?\]$/);
//     if (section) { inExtensions = section[1].toLowerCase() === "extensions"; continue; }
//     if (!inExtensions) continue;
//     const eq = line.indexOf("=");
//     if (eq < 0) continue;
//     if (line.slice(0, eq).trim().toLowerCase() === "objectformat" &&
//         line.slice(eq + 1).trim().toLowerCase() === "sha256") {
//       return "sha256";
//     }
//   }
//   return "sha1";
// }
// ```
fn detect_index_hash_kind(repo_root: &std::path::Path) -> gix_hash::Kind {
    let config_path = repo_root.join(".git/config");
    let Ok(config) = std::fs::read_to_string(&config_path) else {
        return gix_hash::Kind::Sha1;
    };
    let mut in_extensions = false;
    for raw in config.lines() {
        // What:     Strip line comments (`# ...`, `; ...`) and trim.
        // Why:      Git config supports both comment prefixes; the
        //           value before the comment is the only signal.
        let line = raw
            .split(['#', ';'])
            .next()
            .unwrap_or("")
            .trim();
        if line.is_empty() {
            continue;
        }
        // What:     Section headers look like `[section]` or
        //           `[section "subsection"]`. We only care whether the
        //           current section name (case-insensitive) is
        //           `extensions`; subsections are ignored.
        // Why:      `objectformat` lives directly under `[extensions]`
        //           in git's standard config layout; subsections would
        //           be a custom shape we don't speak.
        if let Some(rest) = line.strip_prefix('[') {
            let section = rest.trim_end_matches(']').trim();
            let name = section.split_whitespace().next().unwrap_or("");
            in_extensions = name.eq_ignore_ascii_case("extensions");
            continue;
        }
        if !in_extensions {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("objectformat")
            && value.trim().eq_ignore_ascii_case("sha256")
        {
            return gix_hash::Kind::Sha256;
        }
    }
    return gix_hash::Kind::Sha1
}

/// Implements `list_files`.
// What:     `pub fn list_files(root: &str) -> Result<Vec<String>>`
//           walks the working tree starting at `root` and returns an
//           owned vector of file paths (UTF-8). `pub` makes it visible
//           to `main.rs`. The signature mirrors the prior
//           `list_tracked_files` to keep the call site simple.
// Why:      `--all` mode calls this once to get every scannable file.
//           The `Result` shape lets us propagate walk errors through
//           anyhow, matching the rest of the binary's error style.
//
// In TS you'd write (pseudocode):
// ```ts
// export function listFiles(root: string): string[] {
//   return walkBuilder(root)
//     .hidden(false)
//     .ignore(false)
//     .filterEntry((e) => e.fileName !== ".git" && e.fileName !== ".jj")
//     .buildParallel()
//     .map((e) => e.path);
// }
// ```
pub fn list_files(root: &str) -> Result<Vec<String>> {
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

    // What:     Union with paths read from `.git/index` via `gix-index`
    //           to recover tracked files the walker did not visit. The
    //           `ignore` crate's `WalkBuilder` always honours .gitignore
    //           (the `.ignore(false)` toggle disables `.ignore` files,
    //           not `.gitignore`). A file that was force-added with
    //           `git add -f` despite matching a `.gitignore` pattern is
    //           tracked by git but skipped by the walker, leaving a
    //           silent gap in `--all` mode -- a secret-scan on
    //           push-to-main must cover every tracked file.
    // Why:      Closes BUG 3 without paying the ~88 ms subprocess wall
    //           the previous `git ls-files --cached --ignored
    //           --exclude-standard -z` invocation cost on Mono
    //           (350 ms on Linux). Reading `.git/index` in-process with
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
    //           When `.git/index` is absent (not a repo) or unreadable
    //           the call returns `Err` and we silently fall back to
    //           walker-only output, matching the previous subprocess
    //           behaviour.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const idx = await openIndex(path.join(repoRoot, ".git/index"));
    // for (const entry of idx.entries) {
    //   const rel = entry.path;
    //   // dedupe + normalize prefix as before
    // }
    // ```
    let root_path = std::path::Path::new(root);
    let index_path = root_path.join(".git/index");
    let hash_kind = detect_index_hash_kind(root_path);
    if let Ok(index) = gix_index::File::at(
        &index_path,
        hash_kind,
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
    ) {
        // What:     Build a HashSet of normalized paths already in
        //           `files` so we can detect duplicates with the
        //           index output. WalkBuilder paths typically start
        //           with `./`; index paths are repo-root-relative with
        //           no leading `./`. Normalize by stripping `./` and
        //           comparing the remainder.
        // Why:      Without deduplication tracked-non-ignored files
        //           would scan twice (once via walker, once via index).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const seen = new Set(files.map(p => p.replace(/^\.\//, "")));
        // ```
        let mut seen: std::collections::HashSet<String> = files
            .iter()
            .map(String::as_str)
            .map(without_leading_dot_slash)
            .collect();
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
            //           emit for this entry.
            // Why:      The repo-relative path is what we want to feed
            //           downstream scanning; index entries do not carry
            //           the leading `./` walker convention.
            let path_bytes: &[u8] = entry.path(&index);
            // What:     `std::str::from_utf8(path_bytes)` validates that
            //           the path is UTF-8. Non-UTF-8 paths are silently
            //           skipped to match the walker's existing
            //           semantics (the walker uses `Path::to_str()`,
            //           which also requires UTF-8).
            // Why:      Every downstream consumer expects `&str`; a
            //           single non-UTF-8 path would force `OsString`
            //           plumbing across the binary for vanishingly rare
            //           cases.
            let Ok(rel) = std::str::from_utf8(path_bytes) else {
                continue;
            };
            let normalized = rel.trim_start_matches("./").to_string();
            if seen.insert(normalized.clone()) {
                files.push(format!("./{}", normalized));
            }
        }
    }

    return Ok(files)
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

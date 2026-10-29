//! What:     Classifies the directory a tree walk would start from: a repository root, the home directory, the whole
//!           filesystem, or something bounded.
//! Why:      `grep -r` and `find` are refused only when they would start from one of the three wide places, so the
//!           policies for both tools share this one question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // walk-target.ts
//! export function classify(pathText: string, workingDirectory: string, home: string): WalkTarget { ... }
//! ```

/// Imports the file-system module (resolve symlinks, test for `.git`).
use std::fs;

/// Imports `Path` (borrowed) and `PathBuf` (owned) path types.
use std::path::{Path, PathBuf};

/// What:     `pub enum WalkTarget { ... }` declares a type that is exactly one of four named cases (like a TS union of
///           string literals). `#[derive(Debug, PartialEq)]` asks the compiler to generate printing and `==`.
/// Why:      The policies need the case to choose their wording, and the tests compare cases with `==`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkTarget = 'bounded' | 'repository-root' | 'home' | 'filesystem-root';
/// ```
#[derive(Debug, PartialEq)]
pub enum WalkTarget {
    /// A directory whose walk is limited to its own subtree and not a repository, home or filesystem root.
    Bounded,
    /// A directory that contains `.git`, so a walk reads the whole repository and its ignored build output.
    RepositoryRoot,
    /// The user's home directory.
    Home,
    /// The filesystem root `/`.
    FilesystemRoot,
}

/// What:     `pub fn classify(...) -> WalkTarget` declares a public function returning one of the four cases.
/// Why:      A path that does not exist, or cannot be resolved, counts as bounded: the tool will report the missing path
///           itself and walks nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function classify(pathText: string, workingDirectory: string, home: string): WalkTarget { ... }
/// ```
pub fn classify(path_text: &str, working_directory: &Path, home: &Path) -> WalkTarget {
    // `working_directory.join(path_text)` appends a relative path; an absolute `path_text` replaces the base entirely.
    let candidate: PathBuf = working_directory.join(path_text);

    // What:     `fs::canonicalize(&candidate)` resolves `.`, `..` and symlinks to the final absolute path, as a `Result`.
    //           `.is_err()` asks whether it failed; `.unwrap_or_default()` then takes the path out (an empty path only in
    //           the failure case, which the check returns from first).
    // Why:      Comparing resolved paths makes `.`, `./` and an absolute spelling of the same directory agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let directory: string; try { directory = fs.realpathSync(candidate); } catch { return 'bounded'; }
    // ```
    let resolved = fs::canonicalize(&candidate);
    if resolved.is_err() {
        return WalkTarget::Bounded;
    }
    let directory: PathBuf = resolved.unwrap_or_default();

    if directory == Path::new("/") {
        return WalkTarget::FilesystemRoot;
    }
    // An unresolvable home becomes an empty path, which no real directory equals.
    let home_resolved: PathBuf = fs::canonicalize(home).unwrap_or_default();
    if directory == home_resolved {
        return WalkTarget::Home;
    }
    if directory.join(".git").exists() {
        return WalkTarget::RepositoryRoot;
    }
    return WalkTarget::Bounded;
}

/// What:     `pub fn phrase(target: &WalkTarget) -> &'static str` declares a function returning a text slice that lives for
///           the whole program (`'static`), because every answer is a literal baked into the binary.
/// Why:      The refusal messages say what the walk would cover.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function phrase(target: WalkTarget): string { ... }
/// ```
pub fn phrase(target: &WalkTarget) -> &'static str {
    if *target == WalkTarget::RepositoryRoot {
        return "a whole repository";
    }
    if *target == WalkTarget::Home {
        return "your whole home directory";
    }
    if *target == WalkTarget::FilesystemRoot {
        return "the whole filesystem";
    }
    return "a directory tree";
}

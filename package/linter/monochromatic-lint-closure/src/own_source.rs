//! What:
//!  Decide whether a source file belongs to the package under lint or was generated for it.
//! Why:
//!  A build script writes code such as Slint's `include_modules!` output below the build directory and pastes it into the
//!  crate,
//! so its spans look like the author's own code although nobody can rewrite it by hand;
//!  the lint skips a closure whose span lies in such a file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class OwnSource {
//!   constructor(readonly manifestDir?: string, readonly outDir?: string) {}
//!   excludes(path: string): boolean { /* generated or foreign */ }
//! }
//! ```

/// Import the borrowed operating-system string type that environment variables arrive as.
use std::ffi::OsString;
/// Import the path types and the component enum used to resolve `..` without touching the file system.
use std::path::{Component, Path, PathBuf};
/// Import the once-per-process cell that caches the environment's roots.
use std::sync::OnceLock;

/// Unit tests of the path decision.
#[cfg(test)]
#[path = "own_source_tests.rs"]
mod tests;

/// What:
///  The two directories cargo exports to the compiler process,
/// which decide ownership of a source file.
/// Why:
///  `CARGO_MANIFEST_DIR` is the package's own directory;
///  `OUT_DIR` is where its build script writes generated code,
/// which can sit inside the package's `target` directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnSource = { manifestDir?: string; outDir?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OwnSource {
    /// The package directory;
    /// absent when the compiler was not started by cargo.
    manifest_dir: Option<PathBuf>,
    /// The build script's output directory;
    /// absent for a package without a build script.
    out_dir: Option<PathBuf>,
}

/// What:
///  Resolve `.` and `..` components of a path without reading the file system.
/// Why:
///  A module reached through `#[path = "../x.rs"]` keeps its `..` in the span's file name,
/// and a prefix test on the raw
///  components would count `inside/../outside` as inside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lexicallyNormal(path: string): string { return pathModule.normalize(path); }
/// ```
fn lexically_normal(path: &Path) -> PathBuf {
    let mut normal: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normal.pop() {
                    normal.push(Component::ParentDir);
                }
            }
            other => normal.push(other),
        }
    }
    return normal;
}

/// What:
///  Normalize the value of a path variable.
/// Why:
///  A named function is the mapping that `Option::map` takes without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function normalOfText(value: string): string { return lexicallyNormal(value); }
/// ```
fn normal_of_text(value: OsString) -> PathBuf {
    return lexically_normal(Path::new(&value));
}

/// Build the ownership test from the two exported directories and decide a path.
impl OwnSource {
    /// What:
    ///  Read the roots from the values of `CARGO_MANIFEST_DIR` and `OUT_DIR`.
    /// Why:
    ///  Taking the values as arguments lets the unit tests cover every combination without changing the process environment.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fromValues(manifestDir?: string, outDir?: string): OwnSource;
    /// ```
    pub fn from_values(manifest_dir: Option<OsString>, out_dir: Option<OsString>) -> OwnSource {
        return OwnSource {
            manifest_dir: manifest_dir.map(normal_of_text),
            out_dir: out_dir.map(normal_of_text),
        };
    }

    /// What:
    ///  True when the file was generated for the package or lies outside its directory.
    /// Why:
    ///  A package source reaches the compiler as a path relative to the package root,
    ///  while build-script output and files from other directories are absolute.
    ///  A relative path is therefore always the package's own,
    /// and an absolute one counts as own only below the package
    ///  directory and outside the build script's output directory.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// excludes(path: string): boolean {
    ///   const normal = normalize(path);
    ///   if (this.outDir && isInside(normal, this.outDir)) return true;
    ///   return isAbsolute(normal) && this.manifestDir !== undefined && !isInside(normal, this.manifestDir);
    /// }
    /// ```
    pub fn excludes(&self, path: &Path) -> bool {
        let normal: PathBuf = lexically_normal(path);
        if let Some(out_dir) = &self.out_dir
            && normal.starts_with(out_dir)
        {
            return true;
        }
        if let Some(manifest_dir) = &self.manifest_dir {
            return normal.is_absolute() && !normal.starts_with(manifest_dir);
        }
        return false;
    }

    /// What:
    ///  The roots of this compiler process,
    ///  read once.
    /// Why:
    ///  The lint asks for every closure of the crate,
    ///  and the environment cannot change while the compiler runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const environmentRoots = memoize(() => fromValues(process.env.CARGO_MANIFEST_DIR, process.env.OUT_DIR));
    /// ```
    pub fn from_environment() -> &'static OwnSource {
        /// The roots of this compiler process.
        static ROOTS: OnceLock<OwnSource> = OnceLock::new();
        return ROOTS.get_or_init(environment_roots);
    }
}

/// What:
///  Read the two variables into an `OwnSource`.
/// Why:
///  A named function is the initializer that `OnceLock` takes without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function environmentRoots(): OwnSource;
/// ```
fn environment_roots() -> OwnSource {
    return OwnSource::from_values(
        std::env::var_os("CARGO_MANIFEST_DIR"),
        std::env::var_os("OUT_DIR"),
    );
}

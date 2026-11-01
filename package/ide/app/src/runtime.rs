//! The application's own language runtime:
//!  parser libraries,
//!  highlighting queries,
//!  and the manifest.
//!
//! What:
//!  The one place syntax highlighting reads language files from.
//!  The application binary
//!       installs the table its build embedded ([`RuntimeSource::Embedded`]);
//!  tests and development
//!       tools,
//!  which have no embedded table,
//!  read the directory that `HELIX_RUNTIME` names
//!       ([`RuntimeSource::Directory`]),
//!  prepared by the package `runtime` task.
//! Why:
//!  Helix's own lookup (`helix_loader::runtime_file`) searches several directories and takes the
//!      first that has a file,
//!  with `$XDG_CONFIG_HOME/helix/runtime` ahead of every other choice,
//!  so
//!      a stray file there silently replaced a bundled query.
//!  Nothing here calls that lookup:
//!  the
//!      application reads only its own files,
//!  whatever is installed elsewhere.

/// The shared private cache directory,
///  where embedded parser libraries are unpacked.
use crate::app_cache::application_cache;
/// Errors carry the file,
///  what happened,
///  and the remedy for the source in use.
use anyhow::{Result, anyhow, bail};
/// Paths,
///  the not-found error kind,
///  and the process-wide once-set value.
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Unpack parser libraries into the private cache,
///  compared byte for byte on every load.
pub mod cache;
/// The license texts of the Rust crates the executable is built from,
///  as cargo-about collected them.
pub mod crate_licenses;
/// The table of files the build script compiled into the application binary.
pub mod embedded;
/// The embedded license and notice texts,
///  as `--licenses` prints them.
pub mod notices;
/// Renew the current build's cache folder and remove other builds' folders unused for 30 days.
pub mod retention;

/// What:
///  Where language files come from.
///  `enum` is a value that is exactly one of these variants,
///       like a TS discriminated union.
///  `&'static EmbeddedRuntime` borrows the table stored in the
///       executable;
///  `PathBuf` owns a directory path.
/// Why:
///  The application binary and the test or tool programs have different sources,
///  and every
///      reader goes through this one type,
///  so neither can fall back to Helix's directory search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuntimeSource = { kind: 'embedded'; runtime: EmbeddedRuntime } | { kind: 'directory'; path: string };
/// ```
#[derive(Debug)]
pub enum RuntimeSource {
    /// The files compiled into the application executable.
    Embedded(
        /// The table the build script generated.
        &'static embedded::EmbeddedRuntime,
    ),
    /// A prepared runtime directory,
    ///  used by tests and development tools only.
    Directory(
        /// The directory holding `manifest.json`,
        ///  `grammars/`,
        ///  and `queries/`.
        PathBuf,
    ),
}

/// What:
///  What to do about an unusable embedded runtime;
///  `&str` borrows text stored in the program.
/// Why:
///  A user of the single executable cannot run a package task,
///  so the remedy is a fresh copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EMBEDDED_REMEDY = 'Replace the executable with a fresh copy ...';
/// ```
pub const EMBEDDED_REMEDY: &str = "Replace the executable with a fresh copy of the application, or build it again from source, then restart the application.";

/// What:
///  What to do about an unusable runtime directory.
/// Why:
///  Only tests and development tools read a directory,
///  and their users can run the package task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DIRECTORY_REMEDY = 'Prepare the directory with the package runtime task ...';
/// ```
pub const DIRECTORY_REMEDY: &str = "Prepare the directory with the package runtime task (mise run //package/ide/app:runtime); only tests and development tools read a language runtime from a directory.";

/// What:
///  The source chosen for this process.
///  `OnceLock` holds a value that is set once and then
///       only read,
///  safely from any thread (sibling:
///  `OnceCell`,
///  the same for one thread only).
/// Why:
///  Highlighting runs on worker threads that start after `main` chose the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let installed: RuntimeSource | undefined;
/// ```
static INSTALLED: OnceLock<RuntimeSource> = OnceLock::new();

/// What:
///  Choose this process's source;
///  the first choice stays.
/// Why:
///  The application binary calls this before any window or worker exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function install(source: RuntimeSource): void { installed ??= source; }
/// ```
pub fn install(source: RuntimeSource) {
    tracing::debug!(source = %source.describe(), "language runtime chosen");
    // What: `set` returns `Err(value)` when a value was already set; the old one stays.
    // Why: A second choice is a programming mistake worth a log line, not a crash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (installed) log.warn('language runtime was already chosen'); else installed = source;
    // ```
    if let Err(ignored) = INSTALLED.set(source) {
        tracing::warn!(ignored = %ignored.describe(), "language runtime was already chosen; keeping the first");
    }
}

/// What:
///  At start,
///  renew the cache folder of `runtime`'s key,
///  then remove the folders of other
///       builds unused for longer than [`retention::UNUSED_LIMIT`].
/// Why:
///  The application binary calls this once,
///  before any window exists,
///  so the current key is in
///      place before anything is removed.
///  It runs on the starting thread:
///  it lists one small
///      folder and removes at most a few old ones,
///  and a removal must not be cut short by an exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tidyCache(runtime: EmbeddedRuntime): void { markUsed(keyFolder); removeUnused(...); }
/// ```
pub fn tidy_cache(runtime: &embedded::EmbeddedRuntime) {
    let Some(cache) = application_cache() else {
        tracing::warn!(
            "no private cache directory (neither XDG_CACHE_HOME nor HOME is absolute); the language parser cache is not tidied"
        );
        return;
    };
    let folder = cache.join("runtime");
    // What: `if let Err(error) = ...` runs the block only for a failed renewal; `{error:#}` prints
    //       the message with its causes.
    // Why: Removal of other folders still runs: it never touches the current key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { markUsed(join(folder, runtime.key)); } catch (error) { log.warn(error); }
    // ```
    if let Err(error) = retention::mark_used(&folder.join(runtime.key)) {
        tracing::warn!(error = %format!("{error:#}"), "language parser cache use not recorded at start");
    }
    retention::remove_unused(
        &folder,
        runtime.key,
        std::time::SystemTime::now(),
        retention::UNUSED_LIMIT,
    );
}

/// What:
///  The chosen source;
///  without one,
///  the directory `HELIX_RUNTIME` names.
///  `&'static` borrows
///       the stored value for the rest of the program.
/// Why:
///  Tests and tools never call [`install`];
///  the application binary always does,
///  so it never
///      reads `HELIX_RUNTIME`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function current(): RuntimeSource // throws when neither exists
/// ```
pub fn current() -> Result<&'static RuntimeSource> {
    if let Some(source) = INSTALLED.get() {
        return Ok(source);
    }
    let Some(directory) = std::env::var_os("HELIX_RUNTIME") else {
        bail!(
            "No language runtime is available to this program: it has no embedded language files and HELIX_RUNTIME names no directory. The application binary embeds its own; tests and development tools read the directory HELIX_RUNTIME names, which the package runtime task prepares (mise run //package/ide/app:runtime)."
        );
    };
    // What: `set` stores the value unless one is present; `get` then reads the stored one.
    // Why: Concurrent first callers agree on one value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // installed ??= { kind: 'directory', path: directory }; return installed;
    // ```
    // `set` fails when another caller stored a value first; that value is the one every caller reads.
    let _stored = INSTALLED.set(RuntimeSource::Directory(PathBuf::from(directory)));
    let Some(installed) = INSTALLED.get() else {
        bail!("The language runtime source was not stored");
    };
    return Ok(installed);
}

/// Reading the manifest,
///  queries,
///  and parser libraries from whichever source is in use.
impl RuntimeSource {
    /// What:
    ///  A short description for logs.
    ///  `&self` borrows the value the method is called on.
    /// Why:
    ///  A log line must say which source a process used.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// describe(): string
    /// ```
    pub fn describe(&self) -> String {
        return match self {
            Self::Embedded(runtime) => format!(
                "embedded ({} files, key {})",
                runtime.files.len(),
                runtime.key
            ),
            Self::Directory(path) => format!("directory {}", path.display()),
        };
    }

    /// What:
    ///  What the reader of an error should do,
    ///  depending on the source.
    /// Why:
    ///  A user of the single executable cannot run a package task;
    ///  a developer can.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// remedy(): string
    /// ```
    pub fn remedy(&self) -> &'static str {
        return match self {
            Self::Embedded(_) => EMBEDDED_REMEDY,
            Self::Directory(_) => DIRECTORY_REMEDY,
        };
    }

    /// What:
    ///  The manifest text and where it came from,
    ///  for messages.
    ///  `(String, String)` is a pair.
    /// Why:
    ///  The manifest lists the bundled grammars;
    ///  recognition depends on it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// manifest(): [text: string, location: string]
    /// ```
    pub fn manifest(&self) -> Result<(String, String)> {
        match self {
            Self::Embedded(runtime) => {
                let path = "runtime/manifest.json";
                let bytes = runtime.required(path)?;
                // What: `String::from_utf8(bytes.to_vec())` copies the bytes into owned text,
                //       failing when they are not UTF-8.
                // Why: The JSON decoder needs text; the digest already proved these are the built bytes.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
                // ```
                let text = match String::from_utf8(bytes.to_vec()) {
                    Ok(found) => found,
                    Err(error) => {
                        return Err(anyhow::Error::new(error)
                            .context(format!("The embedded {path} is not UTF-8 text")));
                    }
                };
                let location = format!("{path} embedded in {}", embedded::executable_name());
                return Ok((text, location));
            }
            Self::Directory(directory) => {
                let path = directory.join("manifest.json");
                let text = match std::fs::read_to_string(&path) {
                    Ok(found) => found,
                    Err(error) => {
                        return Err(anyhow::Error::new(error).context(format!(
                            "Cannot read the language manifest {}. {}",
                            path.display(),
                            self.remedy()
                        )));
                    }
                };
                return Ok((text, path.display().to_string()));
            }
        }
    }

    /// What:
    ///  The text of one query file of one language,
    ///  or empty text when the runtime has none.
    /// Why:
    ///  Languages ship only some query kinds (many have no `locals.scm`),
    ///  and Helix's own reader
    ///      treats a missing file as empty in the same way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// query(language: string, file: string): string // '' when absent; throws when damaged
    /// ```
    pub fn query(&self, language: &str, file: &str) -> Result<String> {
        match self {
            Self::Embedded(runtime) => {
                let path = format!("runtime/queries/{language}/{file}");
                // `&path` lends the text to the lookup.
                let Some(bytes) = runtime.verified(&path)? else {
                    return Ok(String::new());
                };
                return match String::from_utf8(bytes.to_vec()) {
                    Ok(found) => Ok(found),
                    Err(error) => Err(anyhow::Error::new(error)
                        .context(format!("The embedded {path} is not UTF-8 text"))),
                };
            }
            Self::Directory(directory) => {
                let path = directory.join("queries").join(language).join(file);
                return match std::fs::read_to_string(&path) {
                    Ok(text) => Ok(text),
                    Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
                    Err(error) => Err(anyhow!(
                        "Cannot read the highlighting rules {}: {error}. {}",
                        path.display(),
                        self.remedy()
                    )),
                };
            }
        }
    }

    /// What:
    ///  A file path of the parser library for one grammar:
    ///  unpacked into the private cache for
    ///       the embedded source,
    ///  or inside the directory.
    /// Why:
    ///  The dynamic loader opens libraries by path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// grammarLibrary(name: string): string // throws when missing, damaged, or not unpackable
    /// ```
    pub fn grammar_library(&self, name: &str) -> Result<PathBuf> {
        match self {
            Self::Embedded(runtime) => {
                let bytes = runtime.required(&format!("runtime/grammars/{name}.so"))?;
                let Some(cache) = application_cache() else {
                    bail!(
                        "Cannot unpack the language parser {name}: neither XDG_CACHE_HOME nor HOME is an absolute path, so the application has no private cache directory. Set HOME or XDG_CACHE_HOME and restart the application. Source remains readable without coloring."
                    );
                };
                let key_folder = cache.join("runtime").join(runtime.key);
                return retention::unpack_marked(&key_folder, name, bytes);
            }
            Self::Directory(directory) => {
                let path = directory.join("grammars").join(format!("{name}.so"));
                if !Path::is_file(&path) {
                    bail!(
                        "The parser library {} is missing. {}",
                        path.display(),
                        self.remedy()
                    );
                }
                return Ok(path);
            }
        }
    }
}

//! Which directory Helix would root a language server at,
//!  checked before anything is spawned.
//!
//! Helix derives the root from the process working directory,
//!  which it reads once and keeps
//! (`helix-stdx/src/env.rs`),
//!  walking up to the first `.git`,
//!  `.svn`,
//!  `.jj`,
//!  or `.helix`.
//!  The
//! application sets the working directory to the project root at startup;
//!  this module never
//! changes it.
//!  It asks Helix's own public functions what they will compute and refuses every
//! answer that is not inside the project,
//!  so a server is never started on an enclosing tree.

/// Helix's per-language configuration supplies the root-marker file names.
use helix_core::syntax::config::LanguageConfiguration;
/// What:
///  `Path` is a borrowed filesystem path and `PathBuf` its owned sibling.
/// Why:
///  Roots are compared by their resolved locations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { realpathSync } from 'node:fs';
/// ```
use std::path::{Path, PathBuf};

/// What:
///  Why no server may be started for a file.
///  An `enum` with data is a tagged union.
/// Why:
///  A wrong working directory is a wiring fault of the application;
///  an enclosing root is a
///      property of the project.
///  The reader reports them differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootRefusal = { kind: 'wrongWorkingDirectory'; directory: string }
///                  | { kind: 'outsideProject'; root: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RootRefusal {
    /// The directory Helix resolves roots from neither is nor contains the project root.
    WrongWorkingDirectory(
        /// The directory Helix resolves roots from.
        PathBuf,
    ),
    /// Helix would root the server at this directory,
    ///  which is outside the project.
    OutsideProject(
        /// The directory that would have become the server's root.
        PathBuf,
    ),
}

/// What:
///  The project root in two spellings.
///  `PathBuf` owns its path.
/// Why:
///  The application uses the resolved (canonical) path;
///  Helix uses the working directory
///      as the shell spelled it,
///  which differs when the project is reached through a symbolic
///      link (for example `/home` linking to `/var/home`).
///  Paths handed to Helix must use
///      Helix's spelling,
///  or Helix concludes the file is outside its workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootView = { canonical: string; helix: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RootView {
    /// Resolved project root,
    ///  as `Workspace::root` reports it.
    canonical: PathBuf,
    /// The same directory in the spelling Helix derives from its working directory.
    helix: PathBuf,
}

/// Root computations.
impl RootView {
    /// What:
    ///  Derive Helix's spelling of the project root from Helix's own workspace lookup.
    ///       `Result<Self, RootRefusal>` is the view (`Ok`) or the refusal (`Err`).
    /// Why:
    ///  `helix_core::find_workspace` is exactly what `helix_lsp` consults when it starts a
    ///      server,
    ///  so this view cannot disagree with it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static discover(canonical: string): RootView // throws the refusal
    /// ```
    pub(super) fn discover(canonical: &Path) -> Result<Self, RootRefusal> {
        // The tuple's second member says whether the workspace is the working directory itself.
        let (workspace, _is_working_directory) = helix_core::find_workspace();
        // What: `canonicalize` resolves symbolic links and returns `Result`; `match` unpacks it.
        // Why: The two spellings can only be related through their resolved locations.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let resolved: string; try { resolved = realpathSync(workspace); } catch { throw refusal; }
        // ```
        let resolved = match std::fs::canonicalize(&workspace) {
            Ok(resolved) => resolved,
            Err(error) => {
                tracing::error!(workspace = %workspace.display(), %error, "cannot resolve the directory Helix roots servers from");
                // `Err(...)` is the failure variant of `Result`.
                return Err(RootRefusal::WrongWorkingDirectory(workspace));
            }
        };
        // What: `strip_prefix` returns the part of the project path below the workspace, or an
        //       error when the project is not inside it.
        // Why: A workspace that does not contain the project means the application did not make
        //      the project root its working directory before the first Helix call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!canonical.startsWith(resolved)) throw refusal;
        // const below = relative(resolved, canonical);
        // ```
        let Ok(below) = canonical.strip_prefix(&resolved) else {
            return Err(RootRefusal::WrongWorkingDirectory(workspace));
        };
        // Joining an empty remainder would append a trailing separator; the workspace is then the root.
        let helix = if below.as_os_str().is_empty() {
            workspace
        } else {
            workspace.join(below)
        };
        // `Ok(...)` is the success variant of `Result`.
        return Ok(Self {
            canonical: canonical.to_path_buf(),
            helix,
        });
    }

    /// The project root in Helix's spelling;
    ///  passed as the only extra root directory.
    pub(super) fn helix(&self) -> &Path {
        return &self.helix;
    }

    /// True when the resolved location of `path` is the project root or below it.
    pub(super) fn contains(&self, path: &Path) -> bool {
        // `is_ok_and` is true only when resolution succeeded and the closure accepts the result.
        return match std::fs::canonicalize(path) {
            Ok(resolved) => resolved.starts_with(&self.canonical),
            Err(_) => false,
        };
    }

    /// What:
    ///  Respell a resolved path below the project root the way Helix spells the root.
    ///       A path outside the project is returned unchanged.
    /// Why:
    ///  Helix compares the document path with its workspace textually.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toHelix(path: string): string
    /// ```
    pub(super) fn to_helix(&self, path: &Path) -> PathBuf {
        return match path.strip_prefix(&self.canonical) {
            Ok(below) => self.helix.join(below),
            Err(_) => path.to_path_buf(),
        };
    }

    /// What:
    ///  Compute the root Helix would give a server for `document` (in Helix's spelling) and
    ///       accept it only inside the project.
    /// Why:
    ///  With no root-marker file in the project,
    ///  Helix returns the enclosing version-controlled
    ///      tree even when the project is passed as a root directory (`helix-lsp/src/lib.rs`,
    ///      `find_lsp_workspace`).
    ///  Such a root is refused before any process exists.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lspRoot(config: LanguageConfiguration, document: string): string // throws the refusal
    /// ```
    pub(super) fn lsp_root(
        &self,
        config: &LanguageConfiguration,
        document: &Path,
    ) -> Result<PathBuf, RootRefusal> {
        let (workspace, is_working_directory) = helix_core::find_workspace();
        // What: `parent()` and `to_str()` both return `Option`; `and_then` chains them and
        //       `unwrap_or(".")` substitutes the working directory, as `helix_lsp` does.
        // Why: The arguments must equal the ones `helix_lsp` passes, or the answer would differ.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const directory = dirname(document) ?? '.';
        // ```
        let mut directory = ".";
        if let Some(parent) = document.parent()
            && let Some(text) = parent.to_str()
        {
            directory = text;
        }
        let own_roots = [self.helix.clone()];
        // `as_deref()` borrows the configured list; `unwrap_or` substitutes the project root.
        let root_directories = config.workspace_lsp_roots.as_deref().unwrap_or(&own_roots);
        let found = helix_lsp::find_lsp_workspace(
            directory,
            &config.roots,
            root_directories,
            &workspace,
            is_working_directory,
        );
        // Without a root Helix runs the server in the workspace directory itself.
        let root = found.unwrap_or(workspace);
        if self.contains(&root) {
            return Ok(root);
        }
        return Err(RootRefusal::OutsideProject(root));
    }
}

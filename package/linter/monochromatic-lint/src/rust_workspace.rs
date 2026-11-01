//! What:
//!  Load Cargo metadata and generated semantic inputs through a fixed native backend.
//! Why:
//!  Build-script errors cannot be logged and silently treated as complete type information.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Discover an explicit Cargo manifest and compiler source, prepare the requested inputs, then own the loaded session.
//! ```

/// Import typed setup failures,
///  session ownership and validated toolchain discovery.
use crate::rust_semantic_error::SemanticError;
/// Import the semantic session that the workspace loader fills in.
use crate::rust_semantic_session::RustSemanticSession;
/// Import the toolchain model and discovery that choose which compiler loads the workspace.
use crate::rust_toolchain::{RustToolchain, absolute_utf8, discover_toolchain};
/// Import the exact inspected backend loaders and their owned result types.
use ra_ap_ide_db::RootDatabase;
/// Import the rust-analyzer workspace loader and its configuration types.
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
/// Import the procedural-macro client type that workspace loading produces.
use ra_ap_proc_macro_api::ProcMacroClient;
/// Import the project-model types that describe the manifest and build settings.
use ra_ap_project_model::{
    CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource, TargetDirectoryConfig,
    WorkspaceBuildScripts,
};
/// Import the virtual file system types that the workspace loader populates.
use ra_ap_vfs::{AbsPathBuf, Vfs};
/// Import native manifest discovery and explicit I/O failure kinds.
use std::io::ErrorKind;
/// Import the path types used to name the manifest and the workspace root.
use std::path::{Path, PathBuf};

/// Preparation is fixed by the host workflow,
///  never an arbitrary executable from JSONC.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspacePreparation {
    /// Read source/metadata only;
    ///  unavailable generated definitions remain explicit resolution failures.
    SourceOnly,
    /// Run locked/offline Cargo checking and load its macro artifacts for complete generated-source context.
    BuildGenerated,
}

/// Find the nearest Cargo manifest without searching unrelated child projects or rust-project command descriptors.
pub fn discover_manifest(input: &Path) -> Result<PathBuf, SemanticError> {
    absolute_utf8(input)?;
    let Some(directory): Option<&Path> = input.parent() else {
        return Err(SemanticError::new(
            "Rust input has no parent directory in which to discover Cargo.toml.",
        ));
    };
    for ancestor in directory.ancestors() {
        let candidate: PathBuf = ancestor.join("Cargo.toml");
        match std::fs::metadata(&candidate) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err(SemanticError::new(
                        format!(
                            "Rust workspace manifest {} is not a regular file.",
                            candidate.display()
                        )
                        .as_str(),
                    ));
                }
                return Ok(candidate);
            }
            Err(error) => {
                if error.kind() != ErrorKind::NotFound && error.kind() != ErrorKind::NotADirectory {
                    return Err(SemanticError::new(
                        format!(
                            "Cannot inspect Rust workspace manifest {}: {error}.",
                            candidate.display()
                        )
                        .as_str(),
                    ));
                }
            }
        }
    }
    return Err(SemanticError::new(format!("No Cargo.toml owns Rust input {}. Provide a Cargo workspace context or configure this semantic rule off for an intentional standalone snippet.", input.display()).as_str()));
}

/// What:
///  The backend's Cargo settings for one validated toolchain.
/// Why:
///  Each field is a policy the backend would otherwise decide by itself.
///  `sysroot_src` names the
/// standard-library source `discover_toolchain` checked;
///  without it the backend reads `RUST_SRC_PATH`
/// first.
///  `--offline --locked` goes to `cargo metadata` and to the build-script `cargo check`:
///  Cargo
/// fetches nothing,
///  and a stale or missing lockfile is reported instead of resolved again.
///  The
/// backend's own build output stays in a subdirectory of the target directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cargoConfig(toolchain: RustToolchain): CargoConfig;
/// ```
pub(crate) fn cargo_config(toolchain: RustToolchain) -> CargoConfig {
    return CargoConfig {
        sysroot: Some(RustLibSource::Path(toolchain.sysroot)),
        sysroot_src: Some(toolchain.library),
        metadata_extra_args: vec![String::from("--offline"), String::from("--locked")],
        extra_args: vec![String::from("--offline"), String::from("--locked")],
        target_dir_config: TargetDirectoryConfig::UseSubdirectory,
        ..CargoConfig::default()
    };
}

/// Load one explicit manifest using named progress handling and no tool-installation/config-command fallback.
pub fn load_cargo_workspace(
    manifest_path: &Path,
    preparation: WorkspacePreparation,
    progress: fn(String),
) -> Result<RustSemanticSession, SemanticError> {
    let absolute: AbsPathBuf = absolute_utf8(manifest_path)?;
    if manifest_path.file_name() != Some(std::ffi::OsStr::new("Cargo.toml")) {
        return Err(SemanticError::new(format!("Rust workspace input {} must name Cargo.toml; alternate project-command descriptors are not loaded.", manifest_path.display()).as_str()));
    }
    let Some(directory): Option<&Path> = manifest_path.parent() else {
        return Err(SemanticError::new("Cargo.toml has no parent directory."));
    };
    let config: CargoConfig = cargo_config(discover_toolchain(directory)?);
    let manifest: ProjectManifest = match ProjectManifest::from_manifest_file(absolute) {
        Ok(value) => value,
        Err(error) => {
            return Err(SemanticError::new(
                format!(
                    "Cannot read Rust manifest {}: {error:#}.",
                    manifest_path.display()
                )
                .as_str(),
            ));
        }
    };
    let mut workspace: ProjectWorkspace = match ProjectWorkspace::load(manifest, &config, &progress) {
        Ok(value) => value,
        Err(error) => return Err(SemanticError::new(format!("Cannot load Cargo metadata for {}: {error:#}. Prepare its lockfile and dependencies before retrying; semantic checking does not fetch packages.", manifest_path.display()).as_str())),
    };
    if preparation == WorkspacePreparation::BuildGenerated {
        let scripts: WorkspaceBuildScripts = match workspace.run_build_scripts(&config, &progress) {
            Ok(value) => value,
            Err(error) => {
                return Err(SemanticError::new(
                    format!(
                        "Cannot prepare generated Rust inputs for {}: {error:#}.",
                        manifest_path.display()
                    )
                    .as_str(),
                ));
            }
        };
        if let Some(error) = scripts.error() {
            return Err(SemanticError::new(format!("Cargo checking failed while preparing generated Rust inputs for {}: {error}. This workspace was not semantically verified.", manifest_path.display()).as_str()));
        }
        workspace.set_build_scripts(scripts);
    }
    let macro_server: ProcMacroServerChoice = if preparation == WorkspacePreparation::BuildGenerated
    {
        ProcMacroServerChoice::Sysroot
    } else {
        ProcMacroServerChoice::None
    };
    let loading: LoadCargoConfig = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: macro_server,
        prefill_caches: false,
        num_worker_threads: 2,
        proc_macro_processes: 1,
    };
    let (database, files, client): (RootDatabase, Vfs, Option<ProcMacroClient>) =
        match load_workspace(workspace, &config.extra_env, &loading) {
            Ok(value) => value,
            Err(error) => {
                return Err(SemanticError::new(
                    format!(
                        "Cannot construct the Rust semantic workspace for {}: {error:#}.",
                        manifest_path.display()
                    )
                    .as_str(),
                ));
            }
        };
    return Ok(RustSemanticSession::from_workspace(database, files, client));
}

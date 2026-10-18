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
    // THROWAWAY SPIKE: selective cfg overrides for one package and RUSTFLAGS through the backend's own `extra_env`.
    let mut cfg_overrides: ra_ap_project_model::CfgOverrides = ra_ap_project_model::CfgOverrides::default();
    if let (Some(package), Some(atoms)) = (
        std::env::var_os("MONOCHROMATIC_LINT_SPIKE_CFG_PACKAGE"),
        std::env::var_os("MONOCHROMATIC_LINT_SPIKE_CFG_ATOMS"),
    ) {
        let enable: Vec<ra_ap_cfg::CfgAtom> = atoms
            .to_string_lossy()
            .split(',')
            .map(|name| ra_ap_cfg::CfgAtom::Flag(ra_ap_hir::Symbol::intern(name)))
            .collect();
        cfg_overrides.selective.insert(package.to_string_lossy().into_owned(), ra_ap_cfg::CfgDiff::new(enable, Vec::new()));
    }
    let mut extra_env: ra_ap_project_model::CargoConfig = CargoConfig::default();
    if let Some(flags) = std::env::var_os("MONOCHROMATIC_LINT_SPIKE_EXTRA_ENV_RUSTFLAGS") {
        extra_env.extra_env.insert(String::from("RUSTFLAGS"), Some(flags.to_string_lossy().into_owned()));
    }
    return CargoConfig {
        cfg_overrides,
        extra_env: extra_env.extra_env,
        sysroot: Some(RustLibSource::Path(toolchain.sysroot)),
        sysroot_src: Some(toolchain.library),
        metadata_extra_args: vec![String::from("--offline"), String::from("--locked")],
        extra_args: vec![String::from("--offline"), String::from("--locked")],
        target_dir_config: TargetDirectoryConfig::UseSubdirectory,
        set_test: crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_SET_TEST"),
        no_deps: crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_NO_DEPS"),
        all_targets: crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_ALL_TARGETS"),
        ..CargoConfig::default()
    };
}

/// Load one explicit manifest using named progress handling and no tool-installation/config-command fallback.
pub fn load_cargo_workspace(
    manifest_path: &Path,
    preparation: WorkspacePreparation,
    progress: fn(String),
) -> Result<RustSemanticSession, SemanticError> {
    let preparation: WorkspacePreparation = if crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_BUILD_GENERATED") {
        WorkspacePreparation::BuildGenerated
    } else {
        preparation
    };
    crate::rust_closure_static_spike::mark(&format!("load: begin preparation={preparation:?}"));
    let absolute: AbsPathBuf = absolute_utf8(manifest_path)?;
    if manifest_path.file_name() != Some(std::ffi::OsStr::new("Cargo.toml")) {
        return Err(SemanticError::new(format!("Rust workspace input {} must name Cargo.toml; alternate project-command descriptors are not loaded.", manifest_path.display()).as_str()));
    }
    let Some(directory): Option<&Path> = manifest_path.parent() else {
        return Err(SemanticError::new("Cargo.toml has no parent directory."));
    };
    let config: CargoConfig = cargo_config(discover_toolchain(directory)?);
    crate::rust_closure_static_spike::mark("load: toolchain discovered, config built");
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
    crate::rust_closure_static_spike::mark("load: ProjectWorkspace::load done (cargo metadata, sysroot, rustc cfg)");
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
        crate::rust_closure_static_spike::mark("load: run_build_scripts done");
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
    crate::rust_closure_static_spike::mark("load: load_workspace done (vfs read, crate graph, proc macros)");
    if crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_TIMING") {
        let crates: usize = ra_ap_hir::Crate::all(&database).len();
        let vfs_files: usize = files.iter().count();
        eprintln!("SPIKE-GRAPH crates={crates} vfs_files={vfs_files} proc_macro_client={}", client.is_some());
    }
    return Ok(RustSemanticSession::from_workspace(database, files, client));
}

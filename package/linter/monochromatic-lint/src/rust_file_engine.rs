//! What:
//!  Route Rust inputs through selected syntax checks or their owning semantic workspace.
//! Why:
//!  Ordinary Rust rules must not launch Cargo,
//!  while explicit-type checks must not use a syntax-only approximation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse locally when no semantic rule is enabled; otherwise reuse one workspace session per manifest.
//! ```

/// Import actual rule results,
///  typed selection and syntax-only execution.
use crate::diagnostic::Diagnostic;
/// Import the syntax-rule dispatcher that runs the enabled checks on one Rust source.
use crate::rust_dispatch::check_syntax_rules;
/// Import the resolved settings that tell the dispatcher which rules are enabled.
use crate::rust_rule_settings::RustRuleSettings;
/// Import the semantic workspace ownership and its typed failures.
use crate::rust_semantic_error::SemanticError;
/// Import the semantic session that owns the loaded workspace for type-aware checks.
use crate::rust_semantic_session::RustSemanticSession;
/// Import the parsed Rust source model that each check reads.
use crate::rust_source::RustSource;
/// Import the workspace discovery and loading steps that prepare the semantic session.
use crate::rust_workspace::{WorkspacePreparation, discover_manifest, load_cargo_workspace};
/// Import native path keys and a deterministic owned workspace map.
use std::collections::BTreeMap;
/// Import the path types used to name each Rust file and its manifest.
use std::path::{Path, PathBuf};

/// What:
///  Invocation-owned workspace cache,
///  initialized only by a selected semantic rule.
/// Why:
///  A caller can check several files without rerunning Cargo preparation for their shared manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RustFileEngine { private workspaces = new Map<Path, RustSemanticSession>(); }
/// ```
pub struct RustFileEngine {
    /// Each session owns its database and process lifetimes until this invocation ends.
    workspaces: BTreeMap<PathBuf, RustSemanticSession>,
    /// Host-selected preparation policy,
    ///  never an executable command from repository configuration.
    preparation: WorkspacePreparation,
    /// Named progress function supplied by the executable's diagnostics layer.
    progress: fn(String),
}

/// Consumer-boundary dispatch controls stay outside release artifacts.
#[cfg(test)]
#[path = "rust_file_engine_tests.rs"]
mod tests;

/// Dispatch exact snapshots without rewriting their files or consulting Cargo for disabled semantic rules.
impl RustFileEngine {
    /// Construct an empty invocation cache;
    ///  no compiler,
    ///  manifest or workspace is opened here.
    pub fn new(preparation: WorkspacePreparation, progress: fn(String)) -> RustFileEngine {
        return RustFileEngine {
            workspaces: BTreeMap::<PathBuf, RustSemanticSession>::new(),
            preparation,
            progress,
        };
    }

    /// Read the number of initialized workspaces for debug reporting and fast-path verification.
    pub fn workspace_count(&self) -> usize {
        return self.workspaces.len();
    }

    /// Check supplied source bytes;
    ///  canonicalization identifies the workspace without changing the reported filename.
    pub fn check(
        &mut self,
        path: &Path,
        source: String,
        filename: String,
        settings: RustRuleSettings,
    ) -> Result<Vec<Diagnostic>, SemanticError> {
        crate::rust_closure_static_spike::mark("engine.check enter");
        if settings.explicit_types.is_none() && !crate::rust_closure_static_spike::enabled() {
            let context: RustSource = RustSource::new(filename, source);
            return Ok(check_syntax_rules(&context, &settings));
        }
        if settings.explicit_types.is_none()
            && crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_PREFILTER")
        {
            let prefilter_started: std::time::Instant = std::time::Instant::now();
            let context: RustSource = RustSource::new(filename.clone(), source.clone());
            let needs: bool = crate::rust_closure_static_spike::has_direct_argument_closure(&context);
            if crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_TIMING") {
                eprintln!("SPIKE-PREFILTER needs_hir={needs} prefilter_ms={:.3} file={}", prefilter_started.elapsed().as_secs_f64() * 1000.0, filename);
            }
            if !needs {
                return Ok(check_syntax_rules(&context, &settings));
            }
        }
        let file_started: std::time::Instant = std::time::Instant::now();
        let physical: PathBuf = match std::fs::canonicalize(path) {
            Ok(value) => value,
            Err(error) => return Err(SemanticError::new(format!("Cannot locate Rust input {} in a Cargo workspace: {error}. Semantic checking needs an existing workspace file; configure this rule off for an intentional virtual or standalone input.", path.display()).as_str())),
        };
        let manifest: PathBuf = discover_manifest(&physical)?;
        if !self.workspaces.contains_key(&manifest) {
            crate::rust_closure_static_spike::mark("engine: workspace load begins");
            let workspace: RustSemanticSession =
                load_cargo_workspace(&manifest, self.preparation, self.progress)?;
            crate::rust_closure_static_spike::mark("engine: workspace load ends");
            self.workspaces.insert(manifest.clone(), workspace);
        }
        let workspace: &mut RustSemanticSession = self
            .workspaces
            .get_mut(&manifest)
            .expect("inserted workspace exists");
        let check_started: std::time::Instant = std::time::Instant::now();
        let result = workspace.check_rules(&physical, source.as_str(), filename.as_str(), settings);
        if crate::rust_closure_static_spike::flag("MONOCHROMATIC_LINT_SPIKE_TIMING") {
            eprintln!(
                "SPIKE-FILE check_ms={:.3} syntax_ms={:.3} classify_ms={:.3} lookup_ms={:.3} ok={} file={}",
                check_started.elapsed().as_secs_f64() * 1000.0,
                crate::rust_closure_static_spike::LAST_SYNTAX_NANOS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e6,
                crate::rust_closure_static_spike::LAST_CLASSIFY_NANOS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e6,
                (check_started - file_started).as_secs_f64() * 1000.0,
                result.is_ok(),
                filename
            );
        }
        return result;
    }
}

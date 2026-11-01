//! What:
//!  Own a loaded Rust workspace and analyze exact selected source snapshots.
//! Why:
//!  Workspace types,
//!  source overlays and registered syntax must stay synchronized across lint/fix passes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // A workspace owns its resolver; checkFile replaces only the selected in-memory source before querying.
//! ```

/// Import shared findings and selected severity.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the syntax-rule dispatcher used by the same per-file checks.
use crate::rust_dispatch::check_syntax_rules;
/// Import the complete rule rather than duplicating checks in the workspace adapter.
use crate::rust_explicit_types::check_explicit_types;
/// Reuse typed rule selection and syntax-only dispatch over the same registered parse.
use crate::rust_rule_settings::RustRuleSettings;
/// Import typed setup/query failures.
use crate::rust_semantic_error::SemanticError;
/// Import the lossless registered-tree source wrapper.
use crate::rust_source::RustSource;
/// Import tracked source changes and database attachment.
use ra_ap_hir::{ChangeWithProcMacros, EditionedFileId, Semantics, attach_db};
/// Import the scoped database accessor used by named query functions.
use ra_ap_hir_ty::{db::HirDatabase, with_attached_db};
/// Import ownership of the semantic database.
use ra_ap_ide_db::RootDatabase;
/// Keep any workspace macro process alive until its database is released.
use ra_ap_proc_macro_api::ProcMacroClient;
/// Import exact syntax roots and file identities.
use ra_ap_syntax::{AstNode, ast};
/// Import the virtual file system types that record which files the loader includes.
use ra_ap_vfs::{FileExcluded, FileId, Vfs, VfsPath};
/// Import setters for typed per-database request data.
use salsa::Setter;
/// Import the native caller path,
///  without changing the process working directory.
use std::path::Path;

/// Request payload owned by the semantic database instead of a captured callback or request-global variable.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectedFile {
    /// File already loaded into the workspace's virtual filesystem.
    file: FileId,
    /// Real or virtual display name to retain in findings.
    filename: String,
    /// Effective complete Rust selection for the current file.
    settings: RustRuleSettings,
}

/// What:
///  One typed request per database,
///  initially with no selected input.
/// Why:
///  Named callbacks retrieve the same request that check_file installed before entering the query scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { selected?: SelectedFile };
/// ```
#[salsa::input(singleton)]
struct Request {
    /// Absence is the pre-query state,
    ///  not a valid request for a successful empty result.
    selected: Option<SelectedFile>,
}

/// Resolve only a file associated with an actual loaded crate,
///  without guessing a fallback crate.
fn inspect(database: &dyn HirDatabase) -> Result<Vec<Diagnostic>, SemanticError> {
    let request: Request = Request::get(database);
    let Some(selected): Option<SelectedFile> = request.selected(database) else {
        return Err(SemanticError::new(
            "Rust semantic checking has no selected input.",
        ));
    };
    let semantics: Semantics<'_, dyn HirDatabase> = Semantics::new_dyn(database);
    let Some(file): Option<EditionedFileId> = semantics.attach_first_edition_opt(selected.file)
    else {
        return Err(SemanticError::new(
            format!("Rust input {} is not part of a loaded Cargo target. Include it in the intended target, load the appropriate target configuration, or configure this semantic rule off for an intentional standalone snippet.", selected.filename).as_str(),
        ));
    };
    let root: ast::SourceFile = semantics.parse(file);
    let source: RustSource = RustSource::from_syntax(selected.filename, root.syntax().clone());
    let mut findings: Vec<Diagnostic> = check_syntax_rules(&source, &selected.settings);
    if let Some(severity) = selected.settings.explicit_types {
        findings.extend(check_explicit_types(&semantics, &source, severity));
    }
    return Ok(findings);
}

/// Use the backend's existing scope to pass its database to a named function.
fn attached_inspection() -> Result<Vec<Diagnostic>, SemanticError> {
    return with_attached_db::<Result<Vec<Diagnostic>, SemanticError>>(inspect);
}

/// Retain a caught panic's message without allowing its payload to escape the query boundary.
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return String::from(*message);
    }
    return format!("non-text panic payload {:?}", payload.type_id());
}

/// What:
///  Catch backend unwinding at the named query boundary.
/// Why:
///  A parser/resolver panic cannot be mistaken for an empty findings list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function protectQuery(operation): Diagnostic[] { try { return operation(); } catch (error) { throw typed(error); } }
/// ```
fn protect_query(
    operation: fn() -> Result<Vec<Diagnostic>, SemanticError>,
) -> Result<Vec<Diagnostic>, SemanticError> {
    let outcome: std::thread::Result<Result<Vec<Diagnostic>, SemanticError>> =
        std::panic::catch_unwind(operation);
    match outcome {
        Ok(result) => return result,
        Err(payload) => {
            let detail: String = panic_message(payload.as_ref());
            return Err(SemanticError::new(
                format!("Rust semantic query panicked: {detail}. This input was not verified.")
                    .as_str(),
            ));
        }
    }
}

/// Protected callback passed as a named function,
///  without capturing the selected file.
fn protected_inspection() -> Result<Vec<Diagnostic>, SemanticError> {
    return protect_query(attached_inspection);
}

/// What:
///  Semantic database,
///  loaded file identities and macro-process ownership for one workspace.
/// Why:
///  The database is dropped before its macro process,
///  and no query output borrows these owners.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RustSemanticSession { checkFile(path, source, filename, severity): Diagnostic[]; }
/// ```
pub struct RustSemanticSession {
    /// Borrowed only while the named semantic query runs.
    database: RootDatabase,
    /// File membership supplied by the workspace loader,
    ///  not arbitrary filesystem discovery.
    files: Vfs,
    /// Updated before each query and consumed within its database scope.
    request: Request,
    /// Release after the database so backend references do not outlive the process owner.
    _macro_server: Option<ProcMacroClient>,
}

/// Protected-query controls are not included in release code.
#[cfg(test)]
#[path = "rust_semantic_session_tests.rs"]
mod tests;

/// Own and query a preloaded workspace;
///  Cargo/toolchain discovery is a separate boundary.
impl RustSemanticSession {
    /// Adopt the loader's owners without launching a second loader or macro process.
    pub fn from_workspace(
        database: RootDatabase,
        files: Vfs,
        macro_server: Option<ProcMacroClient>,
    ) -> RustSemanticSession {
        let request: Request = Request::new(&database, None);
        return RustSemanticSession {
            database,
            files,
            request,
            _macro_server: macro_server,
        };
    }

    /// Convenience entry for the explicit-types rule alone;
    ///  uses the same production dispatch as full selections.
    pub fn check_file(
        &mut self,
        path: &Path,
        source: &str,
        filename: &str,
        severity: Severity,
    ) -> Result<Vec<Diagnostic>, SemanticError> {
        let settings: RustRuleSettings = RustRuleSettings {
            explicit_types: Some(severity),
            ..RustRuleSettings::default()
        };
        return self.check_rules(path, source, filename, settings);
    }

    /// Check the supplied snapshot without rewriting the user's file or changing the current directory.
    pub fn check_rules(
        &mut self,
        path: &Path,
        source: &str,
        filename: &str,
        settings: RustRuleSettings,
    ) -> Result<Vec<Diagnostic>, SemanticError> {
        if !path.is_absolute() {
            return Err(SemanticError::new(
                format!(
                    "Rust semantic input {} needs an absolute path in its loaded workspace.",
                    path.display()
                )
                .as_str(),
            ));
        }
        let Some(text): Option<&str> = path.to_str() else {
            return Err(SemanticError::new(
                "The Rust semantic backend requires UTF-8 project paths. Rename or relocate this input to a UTF-8 path, or configure this semantic rule off; syntax-only checks can still run.",
            ));
        };
        let virtual_path: VfsPath = VfsPath::new_real_path(String::from(text));
        let Some((file, excluded)): Option<(FileId, FileExcluded)> =
            self.files.file_id(&virtual_path)
        else {
            return Err(SemanticError::new(format!("Rust input {} was not loaded from this Cargo workspace. Load its owning workspace or configure this semantic rule off for an intentional standalone input.", path.display()).as_str()));
        };
        if excluded != FileExcluded::No {
            return Err(SemanticError::new(format!("Rust input {} is excluded from the loaded workspace. Correct its workspace inclusion before requesting semantic checking.", path.display()).as_str()));
        }
        // The source overlay changes the database only; the on-disk snapshot remains untouched.
        let mut change: ChangeWithProcMacros = ChangeWithProcMacros::default();
        change.change_file(file, Some(String::from(source)));
        self.database.apply_change(change);
        self.request
            .set_selected(&mut self.database)
            .to(Some(SelectedFile {
                file,
                filename: String::from(filename),
                settings,
            }));
        let result: Result<Vec<Diagnostic>, SemanticError> = attach_db::<
            Result<Vec<Diagnostic>, SemanticError>,
        >(
            &self.database, protected_inspection
        );
        // Clear the selection after both successful and caught-panic outcomes to avoid retaining the prior filename.
        self.request.set_selected(&mut self.database).to(None);
        return result;
    }
}

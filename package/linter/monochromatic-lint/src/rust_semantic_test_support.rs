//! What:
//!  Disposable Cargo-backed semantic fixtures using the production workspace loader.
//! Why:
//!  Tests exercise discovery,
//!  source overlays and query protection rather than parallel test-only glue.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Own a temporary project and reuse its real semantic session between source snapshots.
//! ```

/// Import production findings and session ownership.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the session type that these fixtures construct for the semantic tests.
use crate::rust_semantic_session::RustSemanticSession;
/// Import the actual Cargo loader and its fixed preparation choices.
use crate::rust_workspace::{WorkspacePreparation, load_cargo_workspace};
/// Import exclusively owned temporary-directory cleanup.
use crate::test_fs::Fixture;
/// Import native paths and captured fixture-preparation commands.
use std::path::{Path, PathBuf};
/// Import the process types that run Cargo or the toolchain for the fixtures.
use std::process::{Command, Output};

/// Keep loader progress in the test harness's captured diagnostics.
pub(crate) fn progress(message: String) {
    eprintln!("semantic fixture: {message}");
}

/// Generate the fixture lockfile with Cargo,
///  not by hand-writing a package-manager artifact.
pub(crate) fn prepare_lockfile(directory: &Path) {
    let output: Output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(directory)
        .output()
        .expect("run fixture lock generation");
    assert!(
        output.status.success(),
        "fixture lock generation: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// What:
///  Owned source directory and production semantic session.
/// Why:
///  The session is released before cleanup deletes the physical fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SemanticFixture { check(source: string, severity: Severity): Diagnostic[]; }
/// ```
pub(crate) struct SemanticFixture {
    /// Reused production session,
    ///  including source-overlay handling.
    pub(crate) session: RustSemanticSession,
    /// Path already present in the loaded workspace.
    pub(crate) source_path: PathBuf,
    /// Keep source files alive until the session has been released.
    pub(crate) directory: Fixture,
}

/// Create and query the disposable production consumer.
impl SemanticFixture {
    /// Load a dependency-free fixture;
    ///  source-only preparation never executes its build scripts or macros.
    pub(crate) fn new() -> SemanticFixture {
        let directory: Fixture = Fixture::new();
        let source_directory: PathBuf = directory.path.join("src");
        std::fs::create_dir(&source_directory).expect("create fixture source directory");
        let manifest: PathBuf = directory.path.join("Cargo.toml");
        std::fs::write(&manifest, "[package]\nname = \"semantic-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\nmembers = [\".\"]\n").expect("write fixture manifest");
        let source_path: PathBuf = source_directory.join("main.rs");
        std::fs::write(&source_path, "fn main() {}\n").expect("write initial fixture source");
        prepare_lockfile(&directory.path);
        let session: RustSemanticSession =
            load_cargo_workspace(&manifest, WorkspacePreparation::SourceOnly, progress)
                .expect("load fixture through production boundary");
        return SemanticFixture {
            session,
            source_path,
            directory,
        };
    }

    /// Replace exact in-memory bytes while leaving the physical fixture unchanged.
    pub(crate) fn check(&mut self, source: &str, severity: Severity) -> Vec<Diagnostic> {
        return self
            .session
            .check_file(&self.source_path, source, "input.rs", severity)
            .expect("check fixture snapshot");
    }
}

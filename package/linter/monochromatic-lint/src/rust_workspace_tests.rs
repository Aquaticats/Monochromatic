//! What:
//!  Cargo discovery and generated-source loading at the production boundary.
//! Why:
//!  A source-only context must not pretend generated definitions exist,
//!  and failed Cargo preparation must remain an error.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Create owned fixtures, load with each fixed preparation mode, and inspect real rule outcomes.
//! ```

use crate::diagnostic::{Diagnostic, Severity};
use crate::rust_semantic_error::SemanticError;
use crate::rust_semantic_session::RustSemanticSession;
use crate::rust_semantic_test_support::{prepare_lockfile, progress};
/// Import the validated toolchain model and the backend's Cargo settings and path types.
use crate::rust_toolchain::{RustToolchain, absolute_utf8};
/// Import production workspace loading and typed findings.
use crate::rust_workspace::{
    WorkspacePreparation, cargo_config, discover_manifest, load_cargo_workspace,
};
/// Import owned fixture creation and named progress/lock preparation.
use crate::test_fs::Fixture;
use ra_ap_project_model::{CargoConfig, RustLibSource, TargetDirectoryConfig};
use ra_ap_vfs::AbsPathBuf;
/// Import native paths without hardcoded user directories.
use std::path::{Path, PathBuf};

/// The nearest explicit manifest wins and unrelated project descriptors are rejected.
#[test]
fn cargo_discovery_keeps_its_owner_boundary() {
    let fixture: Fixture = Fixture::new();
    let nested: PathBuf = fixture.path.join("nested/src");
    std::fs::create_dir_all(&nested).expect("create nested fixture");
    let outer: PathBuf = fixture.path.join("Cargo.toml");
    let inner: PathBuf = fixture.path.join("nested/Cargo.toml");
    std::fs::write(&outer, "[workspace]\nmembers = []\n").expect("outer manifest");
    std::fs::write(&inner, "[package]\nname = \"inner\"\nversion = \"0.0.0\"\n")
        .expect("inner manifest");
    assert_eq!(
        discover_manifest(&nested.join("lib.rs")).expect("nearest"),
        inner
    );
    assert!(discover_manifest(Path::new("relative.rs")).is_err());
    let descriptor: PathBuf = fixture.path.join("rust-project.json");
    let result: Result<RustSemanticSession, SemanticError> =
        load_cargo_workspace(&descriptor, WorkspacePreparation::SourceOnly, progress);
    let error: SemanticError = match result {
        Err(failure) => failure,
        Ok(_) => panic!("alternate project-command descriptor was accepted"),
    };
    assert!(error.message.contains("must name Cargo.toml"));
}

/// Discovery walks past ancestors that are regular files:
///  the operating system answers "not a directory" for a
/// manifest below a file,
///  which means no manifest there,
///  not an unreadable one.
///  No Cargo command runs.
#[test]
fn manifest_discovery_walks_past_a_regular_file_on_the_path() {
    let fixture: Fixture = Fixture::new();
    let manifest: PathBuf = fixture.path.join("Cargo.toml");
    std::fs::write(&manifest, "[workspace]\nmembers = []\n").expect("manifest");
    std::fs::write(fixture.path.join("blocker"), "").expect("regular file");
    assert_eq!(
        discover_manifest(&fixture.path.join("blocker/deeper/lib.rs"))
            .expect("the manifest above the regular file"),
        manifest
    );
}

/// The backend receives the checked standard-library source and `--offline --locked` for every Cargo command,
/// and every other setting at the backend's default.
///  Without `sysroot_src` the backend reads `RUST_SRC_PATH`
/// before the toolchain's source;
///  without `--locked` it resolves a stale or missing lockfile again in a temporary
/// copy,
///  so the analysis would describe dependencies the workspace has not locked.
///  Neither difference shows in the
/// container,
///  which sets no `RUST_SRC_PATH` and whose fixtures have current lockfiles,
///  so the settings are pinned here.
#[test]
fn cargo_settings_name_the_checked_library_and_keep_cargo_offline_and_locked() {
    let fixture: Fixture = Fixture::new();
    let sysroot: AbsPathBuf =
        absolute_utf8(&fixture.path.join("sysroot")).expect("absolute sysroot");
    let library: AbsPathBuf =
        absolute_utf8(&fixture.path.join("library")).expect("absolute library");
    let offline_locked: Vec<String> = vec![String::from("--offline"), String::from("--locked")];
    assert_eq!(
        cargo_config(RustToolchain {
            sysroot: sysroot.clone(),
            library: library.clone(),
        }),
        CargoConfig {
            sysroot: Some(RustLibSource::Path(sysroot)),
            sysroot_src: Some(library),
            metadata_extra_args: offline_locked.clone(),
            extra_args: offline_locked,
            target_dir_config: TargetDirectoryConfig::UseSubdirectory,
            ..CargoConfig::default()
        }
    );
}

/// Real Cargo-generated definitions are available only after successful preparation:
///  both a build script's output
/// and a function-like procedural macro of a workspace member,
///  which only the proc-macro server of the generated
/// preparation expands.
#[test]
fn generated_definitions_and_build_failures_are_distinct() {
    let fixture: Fixture = Fixture::new();
    let source_directory: PathBuf = fixture.path.join("src");
    std::fs::create_dir(&source_directory).expect("source directory");
    let manifest: PathBuf = fixture.path.join("Cargo.toml");
    std::fs::write(&manifest, "[package]\nname = \"generated-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nfixture-macro = { path = \"macro\" }\n[workspace]\nmembers = [\".\", \"macro\"]\n").expect("fixture manifest");
    std::fs::create_dir_all(fixture.path.join("macro/src")).expect("macro source directory");
    std::fs::write(fixture.path.join("macro/Cargo.toml"), "[package]\nname = \"fixture-macro\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\nproc-macro = true\n").expect("macro manifest");
    std::fs::write(fixture.path.join("macro/src/lib.rs"), "#[proc_macro]\npub fn make_generated(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {\n    return \"pub fn macro_generated() -> u16 { return 1; }\".parse().expect(\"macro output\");\n}\n").expect("macro source");
    let source: &str = "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\")); fixture_macro::make_generated!(); fn main() { let value: u16 = generated(1_u16); let made: u16 = macro_generated(); }";
    let source_path: PathBuf = source_directory.join("main.rs");
    std::fs::write(&source_path, source).expect("fixture source");
    let build_path: PathBuf = fixture.path.join("build.rs");
    std::fs::write(&build_path, r#"
fn main() {
    let directory: String = std::env::var("OUT_DIR").expect("OUT_DIR");
    let path: std::path::PathBuf = std::path::PathBuf::from(directory).join("generated.rs");
    std::fs::write(path, "pub fn generated<T>(value: T) -> T { return value; }").expect("generated source");
}
"#).expect("owned build script");
    prepare_lockfile(&fixture.path);

    {
        let mut source_only: RustSemanticSession =
            load_cargo_workspace(&manifest, WorkspacePreparation::SourceOnly, progress)
                .expect("source-only context loads");
        let findings: Vec<Diagnostic> = source_only
            .check_file(&source_path, source, "generated.rs", Severity::Error)
            .expect("query completes without treating missing definitions as verified");
        assert!(!findings.is_empty());
        let mut unavailable: bool = false;
        for finding in &findings {
            if finding.processing_failure {
                unavailable = true;
            }
        }
        assert!(
            unavailable,
            "missing generated function must be an explicit coverage failure"
        );
    }
    {
        let mut complete: RustSemanticSession =
            load_cargo_workspace(&manifest, WorkspacePreparation::BuildGenerated, progress)
                .expect("generated context loads after real Cargo preparation");
        let findings: Vec<Diagnostic> = complete
            .check_file(&source_path, source, "generated.rs", Severity::Warn)
            .expect("query generated function");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(!findings[0].processing_failure);
        assert_eq!(findings[0].code, "rust/require-explicit-types");
        assert_eq!(findings[0].severity, Severity::Warn);
    }

    // This changes only the owned fixture. No real project's build scripts or target directory are touched.
    std::fs::write(
        &build_path,
        "fn main() { panic!(\"fixture build failure\"); }\n",
    )
    .expect("failing fixture build script");
    let result: Result<RustSemanticSession, SemanticError> =
        load_cargo_workspace(&manifest, WorkspacePreparation::BuildGenerated, progress);
    let error: SemanticError = match result {
        Err(failure) => failure,
        Ok(_) => panic!("failed build preparation was accepted"),
    };
    assert!(error.message.contains("fixture build failure"), "{error}");
    assert!(
        error.message.contains("not semantically verified"),
        "{error}"
    );
}

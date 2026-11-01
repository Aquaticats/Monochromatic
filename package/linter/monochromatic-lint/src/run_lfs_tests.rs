//! What:
//!  Controls for per-run LFS repository sharing and the order of exclusion and discovery.
//! Why:
//!  An excluded file must be unaffected by an unusable endpoint,
//!  and one repository must be
//! discovered once however many files it holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('LfsRepos', () => { /* inert, excluded-first, shared cache, cached failure */ });
//! ```

/// Import the store under test and its typed inputs.
use super::LfsRepos;
use crate::diagnostic::Severity;
use crate::markdown_lfs_context::{LfsImageContext, LfsImageTarget};
use crate::markdown_lfs_patterns::PathPatterns;
use crate::markdown_rule_settings::LfsSetting;
use crate::markdown_source::MarkdownSource;
use crate::run_test_support::write;
use crate::test_fs::Fixture;
use std::path::PathBuf;

/// Measured with coreutils `sha256sum` for the bytes `image bytes`.
const IMAGE_OID: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";

/// A selected rule with the given exclusion patterns.
fn setting(exclude: &[&str]) -> LfsSetting {
    let mut patterns: Vec<String> = Vec::<String>::new();
    for pattern in exclude {
        patterns.push(String::from(*pattern));
    }
    return LfsSetting {
        severity: Severity::Error,
        exclude: PathPatterns::new(patterns.as_slice()).expect("patterns compile"),
    };
}

/// Parse a one-image document.
fn document() -> MarkdownSource {
    return MarkdownSource::new(
        String::from("pkg/README.md"),
        String::from("![shot](asset/shot.png)\n"),
        false,
    )
    .expect("fixture parses");
}

/// Without a `.lfsconfig` ancestor,
///  or with one that declares no endpoint,
///  the rule is inert.
#[test]
fn files_without_a_declared_endpoint_have_no_context() {
    let fixture: Fixture = Fixture::new();
    let file: PathBuf = write(&fixture.path, "pkg/README.md", "");
    let repos: LfsRepos = LfsRepos::new();
    assert_eq!(
        repos
            .context_for(&file, &document(), &setting(&[]))
            .expect("no repository"),
        None
    );
    write(&fixture.path, ".lfsconfig", "[core]\n\tx = 1\n");
    assert_eq!(
        repos
            .context_for(&file, &document(), &setting(&[]))
            .expect("no endpoint"),
        None
    );
}

/// A tracked image resolves through the shared repository,
///  and later files reuse its resolved targets.
#[test]
fn one_repository_serves_every_file_below_its_root() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        ".lfsconfig",
        "[lfs]\n\turl = https://lfs.example/\n",
    );
    write(&fixture.path, ".gitattributes", "*.png filter=lfs\n");
    write(&fixture.path, "pkg/asset/shot.png", "image bytes");
    let file: PathBuf = write(&fixture.path, "pkg/README.md", "");
    let repos: LfsRepos = LfsRepos::new();
    let first: LfsImageContext = repos
        .context_for(&file, &document(), &setting(&[]))
        .expect("context")
        .expect("repository found");
    assert_eq!(first.object_base, "https://lfs.example");
    assert_eq!(first.repo_root, fixture.path);
    assert_eq!(
        first.resolve_target("pkg/asset/shot.png"),
        LfsImageTarget::Lfs {
            oid: String::from(IMAGE_OID)
        }
    );
    // The same store keeps the first answer even after the image and the endpoint change on disk.
    write(&fixture.path, "pkg/asset/shot.png", "changed");
    write(
        &fixture.path,
        ".lfsconfig",
        "[lfs]\n\turl = https://other.example/\n",
    );
    let second: LfsImageContext = repos
        .context_for(&file, &document(), &setting(&[]))
        .expect("context")
        .expect("repository found");
    assert_eq!(second, first);
    // A new store, as in a new run, reads the changed files.
    let fresh: LfsImageContext = LfsRepos::new()
        .context_for(&file, &document(), &setting(&[]))
        .expect("context")
        .expect("repository found");
    assert_eq!(fresh.object_base, "https://other.example");
    assert_ne!(
        fresh.resolve_target("pkg/asset/shot.png"),
        first.resolve_target("pkg/asset/shot.png")
    );
}

/// Exclusion is decided before the endpoint is read;
///  an unusable endpoint fails only files that are not excluded.
#[test]
fn excluded_files_are_unaffected_by_an_unusable_endpoint() {
    let fixture: Fixture = Fixture::new();
    write(
        &fixture.path,
        ".lfsconfig",
        "[lfs]\n\turl = ssh://git@lfs.example/x\n",
    );
    let file: PathBuf = write(&fixture.path, "pkg/README.md", "");
    let repos: LfsRepos = LfsRepos::new();
    assert_eq!(
        repos
            .context_for(&file, &document(), &setting(&["pkg/"]))
            .expect("excluded before discovery"),
        None
    );
    let error = repos
        .context_for(&file, &document(), &setting(&["other/"]))
        .expect_err("unusable endpoint");
    assert!(error.message.contains(".lfsconfig"), "{}", error.message);
    // The failure is remembered for the run: correcting the file does not change this store's answer.
    write(
        &fixture.path,
        ".lfsconfig",
        "[lfs]\n\turl = https://lfs.example\n",
    );
    assert!(
        repos
            .context_for(&file, &document(), &setting(&[]))
            .is_err()
    );
    assert!(
        LfsRepos::new()
            .context_for(&file, &document(), &setting(&[]))
            .expect("corrected endpoint")
            .is_some()
    );
}

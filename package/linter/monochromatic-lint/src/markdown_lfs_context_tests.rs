//! What:
//!  Repository discovery and per-file target resolution in disposable repositories.
//! Why:
//!  Root discovery,
//!  exclusion and target kinds are frozen by
//! `package/cli/markdown-lint/src/lfs-image-context.unit.test.ts`;
//!  the added cases cover traversal
//! out of the repository,
//!  pointers,
//!  caching and unreadable inputs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(prepareLfsImageContext.name, () => { /* lfs, plain, missing, unreferenced */ });
//! ```

/// Import the operations under test and their collaborators.
use super::{
    LfsImageContext, LfsImageRepo, LfsImageTarget, discover_lfs_image_repo, find_lfs_repo_root,
    is_excluded, prepare_lfs_image_context,
};
use crate::markdown_lfs_patterns::{PathPatterns, lfs_tracked_patterns};
use crate::markdown_source::MarkdownSource;
use crate::test_fs::Fixture;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Bytes of the tracked image fixture and their independently measured SHA-256.
const IMAGE_BYTES: &[u8] = b"image bytes";
/// Measured with coreutils `sha256sum`.
const IMAGE_OID: &str = "de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec";
/// Object base the fixture repository uses.
const BASE: &str = "https://lfs.example";

/// Write the incumbent fixture tree:
///  tracked image,
///  plain file,
///  and a nested Markdown file.
fn write_tree(root: &Path) {
    std::fs::write(
        root.join(".gitattributes"),
        "*.png filter=lfs diff=lfs merge=lfs -text\n",
    )
    .expect("attributes");
    std::fs::create_dir_all(root.join("pkg/asset")).expect("asset directory");
    std::fs::write(root.join("pkg/asset/shot.png"), IMAGE_BYTES).expect("tracked image");
    std::fs::write(root.join("pkg/asset/plain.svg"), "<svg/>").expect("plain image");
    std::fs::write(root.join("pkg/README.md"), "![shot](asset/shot.png)\n").expect("document");
}

/// Build repository facts directly,
///  independent of endpoint normalization.
fn repository(root: &Path) -> LfsImageRepo {
    let text: String =
        std::fs::read_to_string(root.join(".gitattributes")).expect("fixture attributes");
    let patterns: Vec<String> = lfs_tracked_patterns(text.as_str());
    return LfsImageRepo {
        repo_root: root.to_path_buf(),
        object_base: String::from(BASE),
        tracked: PathPatterns::new(patterns.as_slice()).expect("patterns compile"),
        resolved: Mutex::new(BTreeMap::<String, LfsImageTarget>::new()),
    };
}

/// Parse one Markdown fixture.
fn document(source: &str) -> MarkdownSource {
    return MarkdownSource::new(String::from("README.md"), String::from(source), false)
        .expect("fixture parses");
}

/// The nearest `.lfsconfig` file wins;
///  a directory with that name is skipped;
///  absence finds nothing.
#[test]
fn the_nearest_regular_configuration_file_marks_the_root() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    std::fs::create_dir_all(root.join("r/sub/deep")).expect("nested directories");
    std::fs::create_dir_all(root.join("r/pkg/.lfsconfig")).expect("directory named like the file");
    assert_eq!(
        find_lfs_repo_root(&root.join("r/sub/deep")).expect("search"),
        None
    );
    std::fs::write(root.join("r/.lfsconfig"), "").expect("outer configuration");
    assert_eq!(
        find_lfs_repo_root(&root.join("r/sub/deep")).expect("search"),
        Some(root.join("r"))
    );
    assert_eq!(
        find_lfs_repo_root(&root.join("r/pkg")).expect("search"),
        Some(root.join("r"))
    );
    assert_eq!(
        find_lfs_repo_root(&root.join("r")).expect("search"),
        Some(root.join("r"))
    );
    std::fs::write(root.join("r/sub/.lfsconfig"), "").expect("nearer configuration");
    assert_eq!(
        find_lfs_repo_root(&root.join("r/sub/deep")).expect("search"),
        Some(root.join("r/sub"))
    );
    // A start directory that does not exist yet still finds its nearest existing ancestor's file.
    assert_eq!(
        find_lfs_repo_root(&root.join("r/sub/absent/deeper")).expect("search"),
        Some(root.join("r/sub"))
    );
    // A start path below a regular file cannot hold a configuration. The operating system answers
    // "not a directory" there, not "not found", and the search walks past it instead of failing.
    std::fs::write(root.join("r/sub/blocker"), "").expect("regular file");
    assert_eq!(
        find_lfs_repo_root(&root.join("r/sub/blocker/deeper")).expect("search past a file"),
        Some(root.join("r/sub"))
    );
}

/// Without a configuration,
///  or with one that declares no endpoint,
///  the rule has no repository.
#[test]
fn a_repository_needs_a_declared_endpoint() {
    let fixture: Fixture = Fixture::new();
    assert!(
        discover_lfs_image_repo(&fixture.path)
            .expect("no configuration")
            .is_none()
    );
    std::fs::write(fixture.path.join(".lfsconfig"), "[core]\n\tx = 1\n").expect("configuration");
    assert!(
        discover_lfs_image_repo(&fixture.path)
            .expect("no endpoint")
            .is_none()
    );
}

/// Discovery from a nested directory finds the root,
///  the credential-free base and the tracked patterns.
#[test]
fn discovery_reads_the_base_and_tracked_patterns_from_the_root() {
    let fixture: Fixture = Fixture::new();
    write_tree(&fixture.path);
    std::fs::write(
        fixture.path.join(".lfsconfig"),
        "[lfs]\n\turl = https://lfs:token@lfs.example\n",
    )
    .expect("configuration");
    let found: LfsImageRepo = discover_lfs_image_repo(&fixture.path.join("pkg/asset/../asset"))
        .expect("discovery")
        .expect("repository");
    assert_eq!(found.repo_root, fixture.path);
    assert_eq!(found.object_base, BASE);
    assert_eq!(
        found.resolve_target("pkg/asset/shot.png").expect("tracked"),
        LfsImageTarget::Lfs {
            oid: String::from(IMAGE_OID)
        }
    );
    assert_eq!(
        found.resolve_target("pkg/README.md").expect("plain"),
        LfsImageTarget::Plain
    );
    // An endpoint outside the supported form fails discovery instead of leaving the rule inert.
    std::fs::write(
        fixture.path.join(".lfsconfig"),
        "[lfs]\n\turl = ssh://git@lfs.example/x\n",
    )
    .expect("unsupported endpoint");
    let error = discover_lfs_image_repo(&fixture.path).expect_err("unsupported scheme");
    assert!(
        error.message.contains("https://host/path"),
        "{}",
        error.message
    );
    // Attributes that cannot compile fail discovery and name their file.
    std::fs::write(
        fixture.path.join(".lfsconfig"),
        "[lfs]\n\turl = https://lfs.example\n",
    )
    .expect("supported endpoint");
    std::fs::write(
        fixture.path.join(".gitattributes"),
        "[z-a].png filter=lfs\n",
    )
    .expect("attributes with a reversed range");
    let invalid = discover_lfs_image_repo(&fixture.path).expect_err("pattern");
    assert!(
        invalid.message.contains(".gitattributes"),
        "{}",
        invalid.message
    );
}

/// Exclusion patterns are relative to the repository root and never reach files outside it.
#[test]
fn exclusion_is_relative_to_the_repository_root() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    let file: PathPatterns =
        PathPatterns::new(&[String::from("pkg/README.md")]).expect("pattern compiles");
    assert!(is_excluded(root, &root.join("pkg/README.md"), &file));
    assert!(is_excluded(root, &root.join("pkg/./x/../README.md"), &file));
    assert!(!is_excluded(root, &root.join("other.md"), &file));
    assert!(!is_excluded(root, Path::new("/elsewhere/README.md"), &file));
    let directory: PathPatterns =
        PathPatterns::new(&[String::from("pkg/")]).expect("pattern compiles");
    assert!(is_excluded(root, &root.join("pkg/README.md"), &directory));
    assert!(!is_excluded(root, &root.join("README.md"), &directory));
    let none: PathPatterns = PathPatterns::new(&[]).expect("empty list compiles");
    assert!(!is_excluded(root, &root.join("pkg/README.md"), &none));
}

/// Tracked,
///  plain,
///  missing,
///  directory and unreferenced paths resolve to their kinds.
#[test]
fn targets_resolve_to_lfs_plain_and_missing() {
    let fixture: Fixture = Fixture::new();
    write_tree(&fixture.path);
    let repo: LfsImageRepo = repository(&fixture.path);
    let file: PathBuf = fixture.path.join("pkg/README.md");
    let context: LfsImageContext = prepare_lfs_image_context(
        &repo,
        &file,
        &document(
            "![a](asset/shot.png)\n![b](asset/plain.svg)\n![c](asset/gone.png)\n![d](asset)\n",
        ),
    )
    .expect("context");
    assert_eq!(context.file_path, file);
    assert_eq!(context.repo_root, fixture.path);
    assert_eq!(context.object_base, BASE);
    assert_eq!(
        context.resolve_target("pkg/asset/shot.png"),
        LfsImageTarget::Lfs {
            oid: String::from(IMAGE_OID)
        }
    );
    assert_eq!(
        context.resolve_target("pkg/asset/plain.svg"),
        LfsImageTarget::Plain
    );
    assert_eq!(
        context.resolve_target("pkg/asset/gone.png"),
        LfsImageTarget::Missing
    );
    assert_eq!(context.resolve_target("pkg/asset"), LfsImageTarget::Missing);
    assert_eq!(
        context.resolve_target("never/referenced.png"),
        LfsImageTarget::Missing
    );
    assert_eq!(context.targets.len(), 4);
    // A trailing separator or dot segment still names the tracked file, as the incumbent's `path.resolve` does.
    // A fresh repository has an empty cache, so each spelling is read from disk.
    for spelled in ["pkg/asset/shot.png/", "pkg/./asset/shot.png/."] {
        let fresh: LfsImageRepo = repository(&fixture.path);
        assert_eq!(
            fresh.resolve_target(spelled).expect("resolved from disk"),
            LfsImageTarget::Lfs {
                oid: String::from(IMAGE_OID)
            },
            "{spelled}"
        );
    }
}

/// Relative destinations,
///  object URLs and image definitions are all candidates;
///  external images are not.
#[test]
fn candidates_come_from_images_and_definitions() {
    let fixture: Fixture = Fixture::new();
    write_tree(&fixture.path);
    let repo: LfsImageRepo = repository(&fixture.path);
    let source: String = format!(
        "![a](asset/a.png)\n![b]({BASE}/{IMAGE_OID}/pkg/asset/shot.png)\n![c](https://x.example/c.png)\n![d][def]\n\n[def]: ../other/d.png\n"
    );
    let context: LfsImageContext = prepare_lfs_image_context(
        &repo,
        &fixture.path.join("pkg/README.md"),
        &document(source.as_str()),
    )
    .expect("context");
    let keys: Vec<&String> = context.targets.keys().collect::<Vec<&String>>();
    assert_eq!(
        keys,
        ["other/d.png", "pkg/asset/a.png", "pkg/asset/shot.png"]
    );
    assert_eq!(
        context.resolve_target("pkg/asset/shot.png"),
        LfsImageTarget::Lfs {
            oid: String::from(IMAGE_OID)
        }
    );
}

/// A checked-out pointer yields its declared id,
///  and an object path leaving the repository is never read.
#[test]
fn pointers_are_read_and_traversal_stays_inside_the_repository() {
    let fixture: Fixture = Fixture::new();
    let root: PathBuf = fixture.path.join("repo");
    std::fs::create_dir(&root).expect("repository root");
    write_tree(&root);
    let declared: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    std::fs::write(
        root.join("pkg/asset/pointer.png"),
        format!("version https://git-lfs.github.com/spec/v1\noid sha256:{declared}\nsize 12\n"),
    )
    .expect("pointer");
    // A tracked-looking file beside the repository must stay unread.
    std::fs::write(fixture.path.join("outside.png"), IMAGE_BYTES).expect("outside file");
    let repo: LfsImageRepo = repository(&root);
    assert_eq!(
        repo.resolve_target("pkg/asset/pointer.png")
            .expect("pointer"),
        LfsImageTarget::Lfs {
            oid: String::from(declared)
        }
    );
    assert_eq!(
        repo.resolve_target("../outside.png").expect("escape"),
        LfsImageTarget::Missing
    );
    assert_eq!(
        repo.resolve_target("pkg/../../outside.png")
            .expect("nested escape"),
        LfsImageTarget::Missing
    );
    assert_eq!(
        repo.resolve_target("").expect("root itself"),
        LfsImageTarget::Missing
    );
    // Redundant segments name the same file as their normalized form.
    assert_eq!(
        repo.resolve_target("pkg/./asset//../asset/shot.png")
            .expect("normalized"),
        LfsImageTarget::Lfs {
            oid: String::from(IMAGE_OID)
        }
    );
    // A path through a regular file does not exist.
    assert_eq!(
        repo.resolve_target("pkg/README.md/shot.png")
            .expect("through a file"),
        LfsImageTarget::Missing
    );
}

/// A resolved target is reused for the run even when the file changes afterwards.
#[test]
fn resolved_targets_are_cached_per_repository() {
    let fixture: Fixture = Fixture::new();
    write_tree(&fixture.path);
    let repo: LfsImageRepo = repository(&fixture.path);
    let first: LfsImageTarget = repo
        .resolve_target("pkg/asset/shot.png")
        .expect("first read");
    std::fs::write(fixture.path.join("pkg/asset/shot.png"), b"changed").expect("rewrite");
    assert_eq!(
        repo.resolve_target("pkg/asset/shot.png").expect("cached"),
        first
    );
    assert_eq!(
        repo.resolve_target("pkg/./asset/shot.png")
            .expect("cached by normalized path"),
        first
    );
    // A fresh repository value reads the new bytes.
    let fresh: LfsImageRepo = repository(&fixture.path);
    assert_ne!(
        fresh
            .resolve_target("pkg/asset/shot.png")
            .expect("fresh read"),
        first
    );
}

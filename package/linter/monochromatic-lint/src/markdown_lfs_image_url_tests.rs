//! What:
//!  The incumbent `lfs-image-url` rule cases,
//!  ported one for one,
//!  plus BOM and astral controls.
//! Why:
//!  What the rule reports and fixes is frozen by `package/cli/markdown-lint/src/rule/lfs-image-url.unit.test.ts`;
//! the added controls prove byte offsets need no astral-character correction.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('lfs-image-url', () => { /* fake resolver: one tracked image, one plain image */ });
//! ```

/// Import the rule under test,
///  its context model and the production grouped-fix applier.
use super::lfs_image_url;
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::{Fix, apply_fixes};
use crate::markdown_lfs_context::{LfsImageContext, LfsImageTarget};
use crate::markdown_source::MarkdownSource;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Object base the fake repository declares.
const BASE: &str = "https://lfs.example";
/// Current oid of the tracked gallery image.
const CURRENT_OID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
/// An oid that no longer matches the gallery image.
const STALE_OID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
/// Repository path of the tracked gallery image.
const GALLERY: &str = "package/player/asset/readme/shot.png";
/// Repository path of an ordinary image.
const PLAIN: &str = "package/player/asset/readme/plain.svg";

/// The object URL the rule should produce for the gallery image.
fn gallery_url() -> String {
    return format!("{BASE}/{CURRENT_OID}/{GALLERY}");
}

/// Per-file context:
///  one tracked image,
///  one plain image,
///  everything else missing.
fn context() -> LfsImageContext {
    let mut targets: BTreeMap<String, LfsImageTarget> = BTreeMap::<String, LfsImageTarget>::new();
    targets.insert(
        String::from(GALLERY),
        LfsImageTarget::Lfs {
            oid: String::from(CURRENT_OID),
        },
    );
    targets.insert(String::from(PLAIN), LfsImageTarget::Plain);
    return LfsImageContext {
        file_path: PathBuf::from("/repo/package/player/README.md"),
        repo_root: PathBuf::from("/repo"),
        object_base: String::from(BASE),
        targets,
    };
}

/// Run only the rule over one source with the fixture context.
fn lint(source: &str, mdx: bool) -> Vec<Diagnostic> {
    let document: MarkdownSource =
        MarkdownSource::new(String::from("README.md"), String::from(source), mdx)
            .expect("fixture parses");
    return lfs_image_url(&document, Severity::Error, &context());
}

/// Run the rule and apply its fixes once through the production applier.
fn fix(source: &str) -> String {
    let mut fixes: Vec<Fix> = Vec::<Fix>::new();
    for finding in lint(source, false) {
        if let Some(group) = finding.fix {
            fixes.push(group);
        }
    }
    return apply_fixes(source, fixes.as_slice())
        .expect("fixes apply")
        .source;
}

/// Relative LFS targets become object URLs;
///  titles,
///  angle brackets and dot segments are preserved or resolved.
#[test]
fn relative_lfs_images_are_rewritten_in_place() {
    let url: String = gallery_url();
    assert_eq!(
        fix("![Wide desktop](asset/readme/shot.png)\n"),
        format!("![Wide desktop]({url})\n")
    );
    assert_eq!(
        fix("![shot](asset/readme/shot.png \"Desktop\")\n"),
        format!("![shot]({url} \"Desktop\")\n")
    );
    assert_eq!(
        fix("![shot](<asset/readme/shot.png>)\n"),
        format!("![shot](<{url}>)\n")
    );
    assert_eq!(
        fix("![shot](./asset/../asset/readme/shot.png?v=2#top)\n"),
        format!("![shot]({url})\n")
    );
}

/// Plain,
///  missing,
///  external,
///  site-absolute,
///  fragment,
///  data and repository-escaping destinations are not this rule's.
#[test]
fn destinations_outside_the_rule_are_left_alone() {
    assert!(lint("![plain](asset/readme/plain.svg)\n", false).is_empty());
    assert!(lint("![gone](asset/readme/gone.png)\n", false).is_empty());
    assert!(
        lint(
            "![a](https://example.com/a.png)\n![b](/a.png)\n![c](#a)\n![d](data:image/png;base64,AA==)\n",
            false
        )
        .is_empty()
    );
    assert!(lint("![up](../../../outside/shot.png)\n", false).is_empty());
}

/// An object URL is accepted when current,
///  refreshed when stale,
///  and ignored without a path segment.
#[test]
fn object_urls_track_the_current_oid() {
    let url: String = gallery_url();
    assert!(lint(format!("![shot]({url})\n").as_str(), false).is_empty());
    assert_eq!(
        fix(format!("![shot]({BASE}/{STALE_OID}/{GALLERY})\n").as_str()),
        format!("![shot]({url})\n")
    );
    let stale: Vec<Diagnostic> = lint(
        format!("![shot]({BASE}/{STALE_OID}/{GALLERY})\n").as_str(),
        false,
    );
    assert_eq!(
        stale[0].message,
        format!("Object URL oid is stale for {GALLERY}; update it to the file's current oid.")
    );
    assert!(lint(format!("![shot]({BASE}/{CURRENT_OID})\n").as_str(), false).is_empty());
}

/// A vanished path is reported without a fix;
///  a path that left LFS returns to a relative link.
#[test]
fn object_urls_follow_missing_and_untracked_targets() {
    let gone: Vec<Diagnostic> = lint(
        format!("![shot]({BASE}/{CURRENT_OID}/package/player/asset/readme/gone.png)\n").as_str(),
        false,
    );
    assert_eq!(gone.len(), 1);
    assert!(gone[0].fix.is_none());
    assert_eq!(
        gone[0].message,
        "Object URL names package/player/asset/readme/gone.png, which no longer exists in the repository."
    );
    assert_eq!(
        fix(format!("![plain]({BASE}/{CURRENT_OID}/{PLAIN})\n").as_str()),
        "![plain](asset/readme/plain.svg)\n"
    );
    let untracked: Vec<Diagnostic> = lint(
        format!("![plain]({BASE}/{CURRENT_OID}/{PLAIN})\n").as_str(),
        false,
    );
    assert_eq!(
        untracked[0].message,
        format!("Object URL names {PLAIN}, which is no longer LFS-tracked; link it relatively.")
    );
}

/// Only definitions an image reference resolves to are rewritten.
#[test]
fn definitions_are_rewritten_only_for_image_references() {
    assert_eq!(
        fix("![shot][gallery]\n\n[gallery]: asset/readme/shot.png\n"),
        format!("![shot][gallery]\n\n[gallery]: {}\n", gallery_url())
    );
    assert!(
        lint(
            "[download][gallery]\n\n[gallery]: asset/readme/shot.png\n",
            false
        )
        .is_empty()
    );
    assert_eq!(
        fix("![shot][gallery]\n\n[gallery]: <asset/readme/shot.png> \"Title\"\n"),
        format!(
            "![shot][gallery]\n\n[gallery]: <{}> \"Title\"\n",
            gallery_url()
        )
    );
}

/// MDX documents are checked outside their JSX,
///  expression and ESM subtrees.
#[test]
fn mdx_documents_are_rewritten_outside_mdx_subtrees() {
    let findings: Vec<Diagnostic> = lint(
        "import X from \"./x\";\n\n![shot](asset/readme/shot.png)\n",
        true,
    );
    assert_eq!(findings.len(), 1);
    let group: &Fix = findings[0].fix.as_ref().expect("fix");
    assert_eq!(group.edits.len(), 1);
    assert_eq!(group.edits[0].replacement, gallery_url());
}

/// The finding is anchored at the image,
///  with the rule id and the incumbent message.
#[test]
fn findings_are_anchored_at_the_image() {
    let findings: Vec<Diagnostic> = lint(
        "# Title\n\nSee ![shot](asset/readme/shot.png) here.\n",
        false,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].labels[0].span.line, 3);
    assert_eq!(findings[0].labels[0].span.column, 5);
    assert_eq!(findings[0].code, "markdown/lfs-image-url");
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(
        findings[0].message,
        "Image targets an LFS-tracked file, which GitHub renders from the pointer as broken; use the object URL."
    );
}

/// One fix pass reaches a fixed point.
#[test]
fn one_fix_pass_is_idempotent() {
    let once: String = fix("![shot](asset/readme/shot.png)\n");
    assert_eq!(fix(once.as_str()), once);
    assert!(lint(once.as_str(), false).is_empty());
}

/// A destination written with an escape or entity differs from the parsed URL and is reported without a fix.
#[test]
fn unlocatable_destinations_are_reported_without_a_fix() {
    let findings: Vec<Diagnostic> = lint("![shot](asset/readme/sh&#111;t.png)\n", false);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].fix.is_none());
    assert!(
        findings[0]
            .message
            .ends_with(" (written URL not located; edit by hand)")
    );
}

/// A leading BOM and astral characters before,
///  inside and after the image leave every other byte identical.
#[test]
fn bom_and_astral_text_keep_exact_byte_edits() {
    let url: String = gallery_url();
    assert_eq!(
        fix("\u{feff}![shot](asset/readme/shot.png)\n"),
        format!("\u{feff}![shot]({url})\n")
    );
    let source: &str = "🚀🚀 ![🚀 alt 🚀](asset/readme/shot.png \"🚀\") 🚀\n\n![🚀][🚀]\n\n[🚀]: asset/readme/shot.png\n";
    assert_eq!(
        fix(source),
        format!("🚀🚀 ![🚀 alt 🚀]({url} \"🚀\") 🚀\n\n![🚀][🚀]\n\n[🚀]: {url}\n")
    );
    let findings: Vec<Diagnostic> = lint(source, false);
    assert_eq!(findings.len(), 2);
    // Two rockets are 8 bytes and 4 UTF-16 units; the image starts after them and one space.
    assert_eq!(findings[0].labels[0].span.offset, 9);
    assert_eq!(findings[0].labels[0].span.column, 6);
    let edit = &findings[0].fix.as_ref().expect("fix").edits[0];
    assert_eq!(&source[edit.start..edit.end], "asset/readme/shot.png");
    let with_bom: String = format!("\u{feff}{source}");
    let shifted: Vec<Diagnostic> = lint(with_bom.as_str(), false);
    assert_eq!(shifted[0].labels[0].span.offset, 12);
    assert_eq!(shifted[0].labels[0].span.column, 6);
}

/// Alt text containing the destination marker does not move the located destination.
#[test]
fn alt_text_markers_do_not_capture_the_destination() {
    assert_eq!(
        fix("![a \\](b) c](asset/readme/shot.png)\n"),
        format!("![a \\](b) c]({})\n", gallery_url())
    );
}

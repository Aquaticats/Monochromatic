//! What:
//!  Controls for the three names of an input file and its language.
//! Why:
//!  A wrong display name misattributes findings,
//!  and a wrong logical path selects the wrong rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('run paths', () => { /* language, absolute, display, relative, logical */ });
//! ```

/// Import the derivations under test.
use super::{Language, absolute_normal, display_name, language_of, logical_path, relative_from};
use crate::processors::ProcessorLanguage;
use std::path::{Path, PathBuf};

/// Only the three exact lowercase extensions select a language.
#[test]
fn extensions_select_languages_exactly() {
    assert_eq!(language_of(Path::new("src/a.rs")), Some(Language::Rust));
    assert_eq!(language_of(Path::new("doc/a.md")), Some(Language::Markdown));
    assert_eq!(language_of(Path::new("doc/a.mdx")), Some(Language::Mdx));
    assert_eq!(language_of(Path::new("doc/a.MD")), None);
    assert_eq!(language_of(Path::new("doc/a.txt")), None);
    assert_eq!(language_of(Path::new("doc/rs")), None);
    assert_eq!(language_of(Path::new("doc/a.rs.bak")), None);
    // A virtual path's final extension decides, as for a real path.
    assert_eq!(
        language_of(Path::new("src/a.rs/42.md")),
        Some(Language::Markdown)
    );
    assert_eq!(Language::Rust.processor(), ProcessorLanguage::Rust);
    assert_eq!(Language::Markdown.processor(), ProcessorLanguage::Markdown);
    assert_eq!(Language::Mdx.processor(), ProcessorLanguage::Mdx);
}

/// Relative inputs join the working directory;
///  dot components are resolved either way.
#[test]
fn absolute_paths_are_lexically_normal() {
    let cwd: &Path = Path::new("/work/repo");
    assert_eq!(
        absolute_normal(Path::new("./doc/../src/a.rs"), cwd),
        PathBuf::from("/work/repo/src/a.rs")
    );
    assert_eq!(
        absolute_normal(Path::new("../other/a.md"), cwd),
        PathBuf::from("/work/other/a.md")
    );
    assert_eq!(
        absolute_normal(Path::new("/elsewhere/./a.md"), cwd),
        PathBuf::from("/elsewhere/a.md")
    );
}

/// Files inside the working directory are named relative to it;
///  others keep their absolute path.
#[test]
fn display_names_are_relative_inside_the_working_directory() {
    let cwd: &Path = Path::new("/work/repo");
    assert_eq!(
        display_name(Path::new("/work/repo/src/a.rs"), cwd),
        "src/a.rs"
    );
    assert_eq!(
        display_name(Path::new("/work/other/a.rs"), cwd),
        "/work/other/a.rs"
    );
    // A sibling whose name starts with the working directory's name is outside it.
    assert_eq!(
        display_name(Path::new("/work/repository/a.rs"), cwd),
        "/work/repository/a.rs"
    );
    assert_eq!(display_name(cwd, cwd), "/work/repo");
}

/// The configuration-relative path climbs out of the base with `..` when the file is outside it.
#[test]
fn relative_paths_climb_out_of_the_base() {
    let base: &Path = Path::new("/work/repo");
    assert_eq!(
        relative_from(base, Path::new("/work/repo/src/a.rs")),
        PathBuf::from("src/a.rs")
    );
    assert_eq!(
        relative_from(base, Path::new("/work/other/a.rs")),
        PathBuf::from("../other/a.rs")
    );
    assert_eq!(
        relative_from(Path::new("/work/repo/deep/base"), Path::new("/work/x.md")),
        PathBuf::from("../../../x.md")
    );
    assert_eq!(relative_from(base, base), PathBuf::new());
    assert!(!relative_from(base, Path::new("/work/other/a.rs")).is_absolute());
}

/// A virtual suffix is appended to the host's logical path without re-encoding the host.
#[test]
fn virtual_suffixes_extend_the_host_path() {
    assert_eq!(
        logical_path(Path::new("src/a.rs"), "/42.md/0.rs"),
        PathBuf::from("src/a.rs/42.md/0.rs")
    );
    assert_eq!(logical_path(Path::new("a.md"), ""), PathBuf::from("a.md"));
}

/// Host bytes that are not UTF-8 survive in the logical path and are replaced only for display.
#[cfg(unix)]
#[test]
fn non_utf8_host_bytes_survive_in_logical_paths() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let host: PathBuf = PathBuf::from(OsString::from_vec(vec![b'n', 0xff, b'.', b'm', b'd']));
    let logical: PathBuf = logical_path(&host, "/0.rs");
    assert_eq!(
        logical.as_os_str().as_bytes(),
        [b'n', 0xff, b'.', b'm', b'd', b'/', b'0', b'.', b'r', b's']
    );
    let absolute: PathBuf = Path::new("/work/repo").join(&host);
    assert_eq!(
        display_name(&absolute, Path::new("/work/repo")),
        "n\u{fffd}.md"
    );
    assert_eq!(language_of(&host), Some(Language::Markdown));
}

/// Only the two places GitHub reads steps from are YAML hosts.
#[test]
fn only_workflows_and_composite_actions_are_yaml_hosts() {
    for host in [
        ".github/workflows/ci.yml",
        ".github/workflows/ci.yaml",
        "/repo/.github/workflows/ci.yml",
        "package/x/.github/workflows/ci.yml",
        ".github/actions/setup/action.yml",
        ".github/actions/group/setup/action.yaml",
    ] {
        assert_eq!(language_of(Path::new(host)), Some(Language::Yaml), "{host}");
    }
    for other in [
        "ci.yml",
        "config/other.yml",
        ".github/dependabot.yml",
        ".github/workflows/nested/deep.yml",
        ".github/workflows/ci.yml.bak",
        ".github/workflows/ci.json",
        ".github/actions/setup/other.yml",
        "actions/setup/action.yml",
        "workflows/ci.yml",
        ".github/workflows/ci.yml/0.sh",
    ] {
        assert_eq!(language_of(Path::new(other)), None, "{other}");
    }
    assert_eq!(Language::Yaml.processor(), ProcessorLanguage::Yaml);
}

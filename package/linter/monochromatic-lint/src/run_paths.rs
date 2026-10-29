//! What:
//!  Path and language facts the executable derives for each input.
//! Why:
//!  One file has three names:
//!  its absolute location (for reads and writes),
//!  its display name
//! (for findings),
//!  and its configuration-relative logical path (for `files` and `ignores` patterns).
//! Keeping the derivations together stops the three from drifting apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // absoluteNormal(path, cwd); displayName(absolute, cwd); relativeFrom(base, absolute); languageOf(path)
//! ```

/// Import lexical normalization and the processor's language tag.
use crate::{markdown_lfs_target::lexical_normal, processors::ProcessorLanguage};
/// Import native path types;
///  `OsStr` compares extension bytes without lossy conversion.
use std::{
    ffi::{OsStr, OsString},
    path::{Component, Path, PathBuf},
};

/// What:
///  The language a file's extension selects.
/// Why:
///  `.rs` is Rust,
///  `.md` is Markdown and `.mdx` is MDX;
///  nothing else is linted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Language = 'rust' | 'markdown' | 'mdx';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    /// A `.rs` file.
    Rust,
    /// A `.md` file.
    Markdown,
    /// A `.mdx` file;
    ///  MDX syntax is parsed,
    ///  never executed.
    Mdx,
    /// A GitHub Actions workflow (`.github/workflows/*.yml` or `.yaml`)
    ///  or a composite action (`action.yml` or `action.yaml` under `.github/actions`).
    Yaml,
}

/// What:
///  Conversions from the file-level language.
/// Why:
///  The processor interface has its own tag for the same three languages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// language.processor()
/// ```
impl Language {
    /// Translate to the processor module's tag.
    pub fn processor(self) -> ProcessorLanguage {
        match self {
            Language::Rust => return ProcessorLanguage::Rust,
            Language::Markdown => return ProcessorLanguage::Markdown,
            Language::Mdx => return ProcessorLanguage::Mdx,
            Language::Yaml => return ProcessorLanguage::Yaml,
        }
    }
}

/// What:
///  Whether a `.yml` or `.yaml` path is a GitHub Actions workflow or composite action.
/// Why:
///  Reading `run:` out of arbitrary YAML would turn any stray key into a failure,
///  so only the two places GitHub reads steps from are hosts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isActionsYaml(path: Path): boolean;
/// ```
fn is_actions_yaml(path: &Path) -> bool {
    let names: Vec<&OsStr> = path
        .components()
        .map(Component::as_os_str)
        .collect::<Vec<&OsStr>>();
    let count: usize = names.len();
    if count >= 3
        && names[count - 2] == OsStr::new("workflows")
        && names[count - 3] == OsStr::new(".github")
    {
        return true;
    }
    let is_action: bool = path.file_stem() == Some(OsStr::new("action"));
    if !is_action || count < 3 {
        return false;
    }
    // Any directory below `.github/actions` may hold an action.
    for index in 0..count - 2 {
        if names[index] == OsStr::new(".github") && names[index + 1] == OsStr::new("actions") {
            return true;
        }
    }
    return false;
}

/// What:
///  Choose a language from a path's extension,
///  or `None` for an unsupported file.
/// Why:
///  Extension bytes are compared exactly;
///  `README.MD` is not a Markdown input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function languageOf(path: Path): Language | undefined;
/// ```
pub fn language_of(path: &Path) -> Option<Language> {
    let extension: &OsStr = path.extension()?;
    if extension == OsStr::new("rs") {
        return Some(Language::Rust);
    }
    if extension == OsStr::new("md") {
        return Some(Language::Markdown);
    }
    if extension == OsStr::new("mdx") {
        return Some(Language::Mdx);
    }
    if (extension == OsStr::new("yml") || extension == OsStr::new("yaml")) && is_actions_yaml(path)
    {
        return Some(Language::Yaml);
    }
    return None;
}

/// What:
///  Resolve a possibly relative path against the working directory and remove `.` and `..`.
/// Why:
///  Later prefix comparisons (display names,
///  configuration bases,
///  repository roots) are lexical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absoluteNormal(path: Path, cwd: Path): Path { return resolve(cwd, path); }
/// ```
pub fn absolute_normal(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        return lexical_normal(path);
    }
    return lexical_normal(&cwd.join(path));
}

/// What:
///  The name findings report:
///  relative to the working directory when the file is inside it.
/// Why:
///  Consumers match findings against the paths they passed;
///  a file outside the working
/// directory keeps its absolute path.
///  Bytes that are not UTF-8 are replaced,
///  since JSON text cannot carry them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function displayName(absolute: Path, cwd: Path): string;
/// ```
pub fn display_name(absolute: &Path, cwd: &Path) -> String {
    if let Ok(below) = absolute.strip_prefix(cwd)
        && below.components().next().is_some()
    {
        return below.to_string_lossy().into_owned();
    }
    return absolute.to_string_lossy().into_owned();
}

/// What:
///  The path from `base` to `absolute`,
///  using `..` for each directory left behind.
/// Why:
///  Configuration patterns match a path relative to the configuration's base,
///  and `--config`
/// may select a file outside that base.
///  Both inputs are absolute and lexically normal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeFrom(base: Path, absolute: Path): Path { return relative(base, absolute); }
/// ```
pub fn relative_from(base: &Path, absolute: &Path) -> PathBuf {
    let from: Vec<Component<'_>> = base.components().collect::<Vec<Component<'_>>>();
    let target: Vec<Component<'_>> = absolute.components().collect::<Vec<Component<'_>>>();
    // Count the leading components both paths share.
    let mut shared: usize = 0;
    // What: `zip` walks both component lists in step, as pairs, and stops at the shorter list by itself.
    // Why: The pair walk needs no bounds checks or hand-stepped index, so no mutation of the count can make it spin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let i = 0; i < Math.min(from.length, target.length) && from[i] === target[i]; i++) shared++;
    // ```
    for (left, right) in from.iter().zip(target.iter()) {
        if left != right {
            break;
        }
        shared += 1;
    }
    let mut relative: PathBuf = PathBuf::new();
    for _ in shared..from.len() {
        relative.push("..");
    }
    for component in &target[shared..] {
        relative.push(component.as_os_str());
    }
    return relative;
}

/// What:
///  Append a virtual-file suffix such as `/42.md/0.rs` to a host's logical path.
/// Why:
///  A virtual file is matched by configuration under its host's path;
///  appending to the native
/// string keeps host bytes that are not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logicalPath(host: Path, suffix: string): Path { return host + suffix; }
/// ```
pub fn logical_path(host: &Path, suffix: &str) -> PathBuf {
    let mut text: OsString = host.as_os_str().to_os_string();
    text.push(suffix);
    return PathBuf::from(text);
}

/// Path-derivation controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_paths_tests.rs"]
mod tests;

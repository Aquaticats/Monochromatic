//! What:
//!  The environment and the attribute files a content check reads:
//! where the system and global attributes files are,
//! what
//!  `.gitattributes` lines convert content,
//! and which macros lead to such attributes.
//! Why:
//!  Git rewrites a file's bytes on the way into the index when an attribute converts it,
//! so a file can be proven equal to its
//!  committed bytes only when no attribute that applies to it converts anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const environment = ConversionEnvironment.fromVariables(variables);
//! const macros = convertingMacros(files); // names of macros that lead to a converting attribute
//! ```

use crate::git_system_config::system_attributes_for;
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_config_text::parse_bool;
use crate::worktree_attributes::{AttributeLine, CONVERTING_ATTRIBUTES, attribute_name};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// Where Git reads the system attributes file from,
/// as far as it can be established.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemAttributes {
    /// `GIT_ATTR_NOSYSTEM` turns the file off.
    Disabled,
    /// The file the executable reads,
    /// at its own prefix.
    File(PathBuf),
    /// The location,
    /// or the variable's value,
    /// could not be established without running Git.
    Unknown,
}

/// What:
///  The environment facts the content decision depends on:
/// the configuration files Git reads and the system attributes file.
/// Why:
///   A control injects both,
/// so a decision never depends on the machine it runs on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConversionEnvironment = { scope: ScopeEnvironment; systemAttributes: 'disabled' | { file: string } | 'unknown'; alternateAttributeSource?: string };
/// ```
#[derive(Clone, Debug)]
pub struct ConversionEnvironment {
    /// Which system and global configuration files Git reads.
    pub scope: ScopeEnvironment,
    /// Where the system attributes file is.
    pub system_attributes: SystemAttributes,
    /// Why Git would take its attributes from a tree instead of the worktree files read here (`GIT_ATTR_SOURCE`,
    /// `--attr-source`,
    ///  `attr.tree`),
    /// when it would;
    /// the content shortcut is then refused.
    pub alternate_attribute_source: Option<String>,
    /// The variables the facts were read from,
    /// which the configuration listing reads its `GIT_CONFIG_COUNT` entries from too,
    /// so
    ///  one environment decides every fact;
    /// empty for a stand-in that names none.
    pub variables: Vec<(OsString, OsString)>,
}

impl ConversionEnvironment {
    /// The facts of an environment given as variables.
    /// The variables name the real Git (the forward-target variable) when the
    ///  system files are to be learned from it;
    /// without it the system files are unknown and the content decision refuses.
    pub fn from_variables(variables: &[(OsString, OsString)]) -> ConversionEnvironment {
        let selected: Option<String> = if sets_variable(variables, "GIT_ATTR_SOURCE") { Some(String::from("GIT_ATTR_SOURCE is set")) } else { None };
        return ConversionEnvironment { scope: ScopeEnvironment::from_variables(variables), system_attributes: system_attributes_from(variables), alternate_attribute_source: selected, variables: variables.to_vec() };
    }
}

/// What:
///  The configuration and attributes files the real Git this command forwards to reads,
/// for a decision about how Git would convert a file.
/// Why:
///   Git rewrites a file's bytes on the way into the index where an attribute converts them,
/// so a file is only proven unchanged under the
///  attributes Git would apply.
/// The facts are learned from the forward-target variable,
/// which names the Git this command resolved itself,
/// never from
///  the wrapper's own environment,
/// which does not carry it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environment = ConversionEnvironment.fromVariables({ ...process.env, ...overlay, [FORWARD_TARGET]: realGit });
/// ```
pub fn for_context(context: &crate::transaction_git::GitContext) -> ConversionEnvironment {
    let mut variables: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    variables.extend(crate::git_system_config::with_real_git(context.overlay.as_slice(), context.real_git.as_path()));
    return ConversionEnvironment::from_variables(variables.as_slice());
}

/// What:
///  Whether the environment sets the variable `wanted`,
/// to any value.
/// Why:
///   Setting `GIT_ATTR_SOURCE` makes Git read attributes from a tree,
/// which the content check does not model.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// variables.some(([name]) => name === wanted)
/// ```
fn sets_variable(variables: &[(OsString, OsString)], wanted: &str) -> bool {
    for (name, _value) in variables {
        if name == wanted {
            return true;
        }
    }
    return false;
}

/// What:
///  Where the system attributes file is for a Git started with `variables`.
/// Why:
///   `GIT_ATTR_NOSYSTEM` is read as Git reads a boolean environment variable (empty is false);
/// the file is where the
///  executable says it is,
/// never derived from the system configuration path,
/// which `GIT_CONFIG_SYSTEM` moves alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function systemAttributesFrom(variables: [string, string][]): SystemAttributes;
/// ```
pub fn system_attributes_from(variables: &[(OsString, OsString)]) -> SystemAttributes {
    // The last setting of the variable wins, so the search runs from the end.
    let mut switch: Option<&OsString> = None;
    for (name, value) in variables.iter().rev() {
        if name == "GIT_ATTR_NOSYSTEM" {
            switch = Some(value);
            break;
        }
    }
    let disabled: Option<bool> = match switch {
        None => Some(false),
        Some(value) if value.is_empty() => Some(false),
        Some(value) => parse_bool(Some(value.as_bytes())),
    };
    return match disabled {
        Some(true) => SystemAttributes::Disabled,
        Some(false) => system_attributes_for(variables).map_or(SystemAttributes::Unknown, SystemAttributes::File),
        None => SystemAttributes::Unknown,
    };
}

/// One attributes file as it was read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeFile {
    /// How the reason names the file.
    pub label: String,
    /// The directory of the file relative to the worktree root,
    /// `/` separated,
    /// empty for the root.
    pub base: Vec<u8>,
    /// Its lines.
    pub lines: Vec<AttributeLine>,
}

/// The attributes files that apply whatever the path is,
/// and the macros among them that set a converting attribute.
#[derive(Clone, Debug)]
pub struct FixedSources {
    /// The files,
    /// in no particular order:
    /// any of them may apply.
    pub files: Vec<AttributeFile>,
    /// Macro names that set a converting attribute,
    /// directly or through another macro.
    pub macros: HashSet<Vec<u8>>,
}

/// What:
///  Whether any line of an attributes file is quoted.
/// Why:
///   A quoted line's pattern and attributes are not modelled,
/// so the file may set anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lines.some((line) => line.quoted)
/// ```
pub fn has_quoted_line(lines: &[AttributeLine]) -> bool {
    for line in lines {
        if line.quoted {
            return true;
        }
    }
    return false;
}

/// What:
///  The first of a line's tokens that sets an attribute that converts content.
/// Why:
///   One converting token on a line that applies to the path makes the file's committed bytes unprovable by stat data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tokens.find((token) => converts(token, macros))
/// ```
pub fn first_converting_token<'a>(tokens: &'a [Vec<u8>], macros: &HashSet<Vec<u8>>) -> Option<&'a Vec<u8>> {
    for token in tokens {
        if converts(token.as_slice(), macros) {
            return Some(token);
        }
    }
    return None;
}

/// Whether a token or a macro name can change a file's bytes on the way into the index.
pub fn converts(token: &[u8], macros: &HashSet<Vec<u8>>) -> bool {
    let name: &[u8] = attribute_name(token);
    return CONVERTING_ATTRIBUTES.contains(&name) || macros.contains(name);
}

/// What:
///  The names of the macros that set a converting attribute,
/// directly or through other macros.
/// Why:
///   Reverse edges (attribute name to the macros that mention it) and a queue visit every definition once,
/// whatever the order of
///  the lines,
/// so a long chain costs work in proportion to its length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertingMacros(files: AttributeFile[]): Set<string>;
/// ```
pub fn converting_macros(files: &[AttributeFile]) -> HashSet<Vec<u8>> {
    let mut mentioned_by: HashMap<Vec<u8>, Vec<Vec<u8>>> = HashMap::new();
    for file in files {
        for line in &file.lines {
            if !line.is_macro {
                continue;
            }
            for token in &line.tokens {
                mentioned_by.entry(attribute_name(token.as_slice()).to_vec()).or_default().push(line.subject.clone());
            }
        }
    }
    let mut known: HashSet<Vec<u8>> = HashSet::new();
    let mut queue: VecDeque<Vec<u8>> = CONVERTING_ATTRIBUTES.iter().copied().map(<[u8]>::to_vec).collect();
    while let Some(name) = queue.pop_front() {
        for macro_name in mentioned_by.get(&name).into_iter().flatten() {
            if known.insert(macro_name.clone()) {
                queue.push_back(macro_name.clone());
            }
        }
    }
    return known;
}

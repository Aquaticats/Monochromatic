//! What:
//! The binaries a new plan entry exposes,
//! each read from the source that owns the names:
//! crates.io's `bin_names` for a crate,
//! npm's `bin` field for a package's manifest,
//! and the last segment of a Go module path.
//! Why:
//! `bins` is what the exposure directory links,
//! so an entry added without it exposes nothing;
//! nothing here guesses a program's name from the tool's own,
//! and a source that publishes no program is refused rather than added empty.
//! A caller imports from `install_argument` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function crateBinNames(context: Context, name: string, range: string): string[];
//! function npmBinNames(context: Context, name: string, range: string): string[];
//! function goBinName(module: string): string;
//! function binsFor(context: Context, route: AddableRoute, request: Request): string[];
//! ```

/// Imports the JSON value the registries answer with.
use serde_json::Value;

/// Imports the registry name check.
use super::super::update_registry::registry_path_name;
/// Imports `GET` and JSON decoding from the resolver helpers.
use super::super::update_resolve::{
    ResolveError,
    get_json,
    text_at,
};

/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports where a request's names come from,
/// and the request whose binaries are read.
use super::super::model::{
    Origin,
    Request,
};

/// Imports the route table the names are read for,
/// and where each route's binaries come from.
use super::the_routes_a_request_can_add::{
    AddableRoute,
    BinsFrom,
};
/// The registry answer as JSON,
/// or a failure naming the URL.
fn fetch(
    context: &Context,
    url: &str,
) -> Result<Value, ToolchainError> {
    return match get_json(context, url, &[]) {
        Ok(document) => Ok(document),
        Err(ResolveError::Failed(message)) => Err(ToolchainError::failure(message)),
    };
}

/// The texts of a JSON array field,
/// or an empty list when the field is absent,
/// null,
/// or not an array.
fn text_list(value: Option<&Value>) -> Vec<String> {
    let Some(entries) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        if let Some(name) = entry.as_str() {
            names.push(name.to_string());
        }
    }
    return names;
}

/// The `bin_names` of one version row in a crates.io crate document,
/// or `None` when the document does not hold that version.
fn crate_row_bin_names(
    document: &Value,
    version: &str,
) -> Option<Vec<String>> {
    let rows: &Vec<Value> = document.get("versions")?.as_array()?;
    for row in rows {
        if text_at(row, &["num"]) == Some(version) {
            return Some(text_list(row.get("bin_names")));
        }
    }
    return None;
}

/// The sorted,
/// de-duplicated list of `names`,
/// which is the order a new plan entry writes.
fn sorted(names: Vec<String>) -> Vec<String> {
    let mut ordered: Vec<String> = names;
    ordered.sort();
    ordered.dedup();
    return ordered;
}

/// What:
/// The binaries of a crate,
/// read from crates.io:
/// the row of the version the request asks for,
/// or of the newest stable one when it asks for `latest`.
/// Why:
/// The registry publishes a crate's `bin_names`,
/// so a person never lists them by hand;
/// a crate with no binary target has none,
/// and the caller refuses it rather than writing an entry that exposes nothing.
fn crate_bin_names(
    context: &Context,
    name: &str,
    range: &str,
) -> Result<Vec<String>, ToolchainError> {
    let path: String = match registry_path_name(name) {
        Ok(text) => text,
        Err(ResolveError::Failed(message)) => {
            return Err(ToolchainError::config(format!("cargo:{name}: {message}")));
        }
    };
    let base: String = format!("https://crates.io/api/v1/crates/{path}");
    if range != "latest" {
        let version_path: String = match registry_path_name(range) {
            Ok(text) => text,
            Err(ResolveError::Failed(message)) => {
                return Err(ToolchainError::config(format!(
                    "cargo:{name}@{range}: {message}"
                )));
            }
        };
        let document: Value = fetch(context, &format!("{base}/{version_path}"))?;
        return Ok(sorted(text_list(
            document
                .get("version")
                .and_then(|row| return row.get("bin_names")),
        )));
    }
    let document: Value = fetch(context, &base)?;
    let Some(version) = text_at(&document, &["crate", "max_stable_version"]) else {
        return Err(ToolchainError::failure(format!(
            "{base}: crates.io reports no stable version for {name}"
        )));
    };
    if let Some(names) = crate_row_bin_names(&document, version) {
        return Ok(sorted(names));
    }
    let version_document: Value = fetch(context, &format!("{base}/{version}"))?;
    return Ok(sorted(text_list(
        version_document
            .get("version")
            .and_then(|row| return row.get("bin_names")),
    )));
}

/// The package's own name in an npm manifest,
/// the command name npm gives a `bin` written as one text.
fn npm_package_name(name: &str) -> String {
    return match name.rsplit('/').next() {
        Some(last) => last.to_string(),
        None => name.to_string(),
    };
}

/// What:
/// The binaries of an npm package,
/// read from the registry's manifest of the version or tag the request names
/// (`latest` when it names none).
/// Why:
/// npm publishes the command names in the package's own `bin` field,
/// so they are read rather than guessed,
/// and a package with no `bin` field has none.
fn npm_bin_names(
    context: &Context,
    name: &str,
    range: &str,
) -> Result<Vec<String>, ToolchainError> {
    let path: String = match registry_path_name(name) {
        Ok(text) => text,
        Err(ResolveError::Failed(message)) => {
            return Err(ToolchainError::config(format!("npm:{name}: {message}")));
        }
    };
    let tag_path: String = match registry_path_name(range) {
        Ok(text) => text,
        Err(ResolveError::Failed(message)) => {
            return Err(ToolchainError::config(format!(
                "npm:{name}@{range}: {message}"
            )));
        }
    };
    let manifest: Value = fetch(
        context,
        &format!("https://registry.npmjs.org/{path}/{tag_path}"),
    )?;
    return match manifest.get("bin") {
        Some(Value::Object(entries)) => {
            let mut names: Vec<String> = Vec::new();
            for key in entries.keys() {
                names.push(key.clone());
            }
            Ok(sorted(names))
        }
        Some(Value::String(_)) => Ok(vec![npm_package_name(name)]),
        _ => Ok(Vec::new()),
    };
}

/// Whether `segment` is a Go major-version suffix (`v2`,
/// `v10`),
/// which is a path element and not the name of a program.
fn is_major_version_suffix(segment: &str) -> bool {
    let Some(rest) = segment.strip_prefix('v') else {
        return false;
    };
    return !rest.is_empty()
        && rest
            .chars()
            .all(|character| return character.is_ascii_digit());
}

/// Whether every character of `segment` may appear in a program name.
fn is_program_name(segment: &str) -> bool {
    if segment.is_empty() || segment.starts_with('-') || segment.contains("..") {
        return false;
    }
    return segment
        .chars()
        .all(|character| return character.is_ascii_alphanumeric() || "._-".contains(character));
}

/// What:
/// The program name `go install <module>@<version>` writes:
/// the last segment of the module path,
/// as Go itself names it.
/// Why:
/// Go installs the package the path names into `GOBIN` under that last segment,
/// so it is read from the request and never looked up;
/// a path that ends in a major-version suffix names no program,
/// and the refusal asks for the package path inside the repository instead.
fn go_bin_name(module: &str) -> Result<String, ToolchainError> {
    let last: &str = match module.rsplit('/').next() {
        Some(segment) => segment,
        None => module,
    };
    if is_major_version_suffix(last) {
        return Err(ToolchainError::config(format!(
            "go:{module} ends with the major-version suffix {last:?}, which names no program; write the package inside the \
             repository instead, for example go:{module}/cmd/<program>"
        )));
    }
    if !is_program_name(last) {
        return Err(ToolchainError::config(format!(
            "go:{module} ends with {last:?}, which is not a program name; write the package path whose last segment is the \
             program `go install` writes"
        )));
    }
    return Ok(last.to_string());
}

/// The `bins` of a new entry,
/// from the source its route owns.
pub(super) fn bins_for(
    context: &Context,
    route: &AddableRoute,
    request: &Request,
) -> Result<Vec<String>, ToolchainError> {
    let range: &str = &request.range;
    let names: Vec<String> = match &route.bins_from {
        BinsFrom::CrateRegistry => match request.origin() {
            Origin::Cargo(name) => crate_bin_names(context, &name, range)?,
            _ => Vec::new(),
        },
        BinsFrom::NpmRegistry => match request.origin() {
            Origin::Npm(name) => npm_bin_names(context, &name, range)?,
            _ => Vec::new(),
        },
        BinsFrom::GoModulePath => match request.origin() {
            Origin::GoModule(module) => vec![go_bin_name(&module)?],
            _ => Vec::new(),
        },
    };
    if names.is_empty() {
        return Err(ToolchainError::failure(format!(
            "{} publishes no program this plan entry could expose, so no `bins` can be filled in and nothing was added; \
             a library package is installed by another tool, not by this one",
            request.text()
        )));
    }
    return Ok(names);
}

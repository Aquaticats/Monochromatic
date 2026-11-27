//! What:
//! One declaration:
//! the fields a plan entry and a lock entry share,
//! and the plan entry that reads them and refuses the two fields `update` owns.
//! Why:
//! The id becomes a directory name under the toolchain root,
//! a `cargo-git` request cannot name its crate,
//! and a `cargo-binstall` crate becomes a directory of its own,
//! so each of those is checked here rather than at the install.
//! A caller imports from `model_decode` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function decodePlanEntry(id: string, value: unknown, path: string): Declaration;
//! function decodeDeclarationFields(id: string, map: Map<string, unknown>, path: string): Declaration;
//! ```

/// Imports the JSON value and object a declaration is read from.
use serde_json::{
    Map,
    Value,
};

/// Imports the model types a declaration is built from.
use super::super::model::{
    Declaration,
    Options,
    OsFamily,
    ROUTE_NAMES,
    Request,
    Route,
};
/// Imports the decoding checks this file shares with its siblings.
use super::super::model_decode_support::{
    DecodeResult,
    as_nonempty_text,
    as_object,
    as_text,
    as_text_list,
    reject_unknown_keys,
};
/// Imports the package name a declaration installs.
use super::super::observe::declared_package_name;
/// Imports the checks that a name is one plain directory name.
use super::super::segment::{
    check_bin_name,
    check_directory_name,
    check_tool_id,
};

/// Imports the request text the declaration names.
use super::a_request_and_its_postinstall::decode_request;
/// Imports the options the declaration carries.
use super::the_options_a_route_takes::decode_options;
/// The declaration fields of the tool `id` in `map`;
/// `extra` lists the field names the lock adds on top.
pub(super) fn decode_declaration_fields(
    id: &str,
    map: &Map<String, Value>,
    path: &str,
) -> DecodeResult<Declaration> {
    // The id becomes a directory name under the toolchain root, and a name such as `..` would leave it.
    if let Err(message) = check_tool_id(id) {
        return Err(format!("{path}: {message}"));
    }
    let Some(request_value) = map.get("request") else {
        return Err(format!("{path}.request is required"));
    };
    let request: Request = decode_request(request_value, &format!("{path}.request"))?;
    let Some(route_value) = map.get("route") else {
        return Err(format!("{path}.route is required"));
    };
    let route_text: String = as_text(route_value, &format!("{path}.route"))?;
    let Some(route) = Route::parse(&route_text) else {
        let mut names: Vec<&str> = Vec::new();
        for (_, route_name) in ROUTE_NAMES {
            names.push(route_name);
        }
        return Err(format!(
            "{path}.route {route_text:?} is not a route; the routes are {}",
            names.join(", ")
        ));
    };
    let manager: Option<String> = match map.get("manager") {
        Some(value) => Some(as_nonempty_text(value, &format!("{path}.manager"))?),
        None => None,
    };
    let package_id: Option<String> = match map.get("packageId") {
        Some(value) => Some(as_nonempty_text(value, &format!("{path}.packageId"))?),
        None => None,
    };
    let bins: Vec<String> = match map.get("bins") {
        Some(value) => as_text_list(value, &format!("{path}.bins"))?,
        None => Vec::new(),
    };
    for (index, name) in bins.iter().enumerate() {
        if let Err(message) = check_bin_name(name) {
            return Err(format!("{path}.bins[{index}]: {message}"));
        }
    }
    let os: Option<Vec<OsFamily>> = match map.get("os") {
        Some(value) => {
            let names: Vec<String> = as_text_list(value, &format!("{path}.os"))?;
            let mut families: Vec<OsFamily> = Vec::new();
            for name in names {
                let Some(family) = OsFamily::parse(&name) else {
                    return Err(format!(
                        "{path}.os {name:?} must be linux, macos or windows"
                    ));
                };
                families.push(family);
            }
            Some(families)
        }
        None => None,
    };
    let mut env: Vec<(String, String)> = Vec::new();
    if let Some(value) = map.get("env") {
        let env_path: String = format!("{path}.env");
        for (name, text) in as_object(value, &env_path)? {
            env.push((name.clone(), as_text(text, &format!("{env_path}.{name}"))?));
        }
    }
    let options: Options = match map.get("options") {
        Some(value) => decode_options(value, &format!("{path}.options"))?,
        None => Options::default(),
    };
    let declaration: Declaration = Declaration {
        id: id.to_string(),
        request,
        route,
        manager,
        package_id,
        bins,
        os,
        env,
        options,
    };
    // A `cargo-git` request names a repository and a revision, never a crate, so `packageId` is the only place the
    // crate name can come from: without it `cargo install --git` was handed the plan key and failed there.
    if declaration.route == Route::CargoGit && declaration.package_id.is_none() {
        return Err(format!(
            "{path}.packageId is required for the cargo-git route: the request names a repository, so it cannot name the crate"
        ));
    }
    // The `cargo-binstall` route installs under `<toolchain root>/cargo/<package name>`.
    if declaration.route == Route::CargoBinstall
        && let Err(message) =
            check_directory_name("the package name", &declared_package_name(&declaration))
    {
        return Err(format!("{path}: {message}"));
    }
    return Ok(declaration);
}

/// What:
/// Decodes one plan entry (`tools.<id>` in `toolchain.plan.jsonc`):
/// the declaration fields only.
/// Why:
/// A plan entry that names `version` or `artifacts` is a hand edit of what `update` owns,
/// so it is rejected
///           with the field named rather than silently ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function decodePlanEntry(id: string, value: unknown, path: string): Declaration
/// ```
pub fn decode_plan_entry(
    id: &str,
    value: &Value,
    path: &str,
) -> DecodeResult<Declaration> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &[
            "request",
            "route",
            "manager",
            "packageId",
            "bins",
            "os",
            "env",
            "options",
        ],
        path,
    )?;
    return decode_declaration_fields(id, map, path);
}

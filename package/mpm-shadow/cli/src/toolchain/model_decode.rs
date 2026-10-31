//! What:
//! Decoding the plan and lock files' JSON into the model:
//! one decoder for a declaration,
//! one for a lock
//!           entry,
//! each naming the path of every wrong field.
//! Why:
//! The model file stays small and holds only types;
//! this file holds the checks that make the types true.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // model-decode.ts
//! export function decodePlanEntry(id: string, value: unknown, path: string): Declaration
//! export function decodeLockEntry(id: string, value: unknown, path: string): LockedTool
//! ```

/// Imports the JSON value types.
use serde_json::{Map, Value};

/// Imports the shared decoding checks.
use super::model_decode_support::{DecodeResult, as_nonempty_text, as_object, as_text, as_text_list, reject_unknown_keys};

/// Imports the model types the decoders build.
use super::model::{
    Artifact, Bootstrap, Declaration, LockedTool, Options, OsFamily, Postinstall, PostinstallLink, PostinstallRun, ROUTE_NAMES,
    Request, Route,
};


/// The request object at `path`.
fn decode_request(value: &Value, path: &str) -> DecodeResult<Request> {
    let map = as_object(value, path)?;
    if map.len() != 1 {
        return Err(format!("{path} must hold exactly one of track, pinned or channel"));
    }
    if let Some(track) = map.get("track") {
        if track.as_str() == Some("latest") {
            return Ok(Request::Latest);
        }
        return Err(format!("{path}.track must be \"latest\""));
    }
    if let Some(pinned) = map.get("pinned") {
        return Ok(Request::Pinned(as_nonempty_text(pinned, &format!("{path}.pinned"))?));
    }
    if let Some(channel) = map.get("channel") {
        return Ok(Request::Channel(as_nonempty_text(channel, &format!("{path}.channel"))?));
    }
    return Err(format!("{path} must hold exactly one of track, pinned or channel"));
}

/// The postinstall object at `path`.
fn decode_postinstall(value: &Value, path: &str) -> DecodeResult<Postinstall> {
    let map = as_object(value, path)?;
    reject_unknown_keys(map, &["when", "link", "run"], path)?;
    if map.get("when").and_then(Value::as_str) != Some("always") {
        return Err(format!("{path}.when must be \"always\""));
    }
    if let Some(link) = map.get("link") {
        let link_path: String = format!("{path}.link");
        let link_map = as_object(link, &link_path)?;
        reject_unknown_keys(link_map, &["sourceBin", "linkName", "linkDir"], &link_path)?;
        let Some(source) = link_map.get("sourceBin") else {
            return Err(format!("{link_path}.sourceBin is required"));
        };
        let Some(name) = link_map.get("linkName") else {
            return Err(format!("{link_path}.linkName is required"));
        };
        let link_dir: Option<String> = match link_map.get("linkDir") {
            Some(directory) => Some(as_nonempty_text(directory, &format!("{link_path}.linkDir"))?),
            None => None,
        };
        return Ok(Postinstall::Link(PostinstallLink {
            source_bin: as_nonempty_text(source, &format!("{link_path}.sourceBin"))?,
            link_name: as_nonempty_text(name, &format!("{link_path}.linkName"))?,
            link_dir,
        }));
    }
    if let Some(command) = map.get("run") {
        let run_path: String = format!("{path}.run");
        let run_map = as_object(command, &run_path)?;
        reject_unknown_keys(run_map, &["sourceBin", "args"], &run_path)?;
        let Some(source) = run_map.get("sourceBin") else {
            return Err(format!("{run_path}.sourceBin is required"));
        };
        let Some(arguments) = run_map.get("args") else {
            return Err(format!("{run_path}.args is required"));
        };
        // Arguments may be empty strings, so they are read with `as_text`, not `as_nonempty_text`.
        let mut args: Vec<String> = Vec::new();
        let Some(items) = arguments.as_array() else {
            return Err(format!("{run_path}.args must be an array of strings"));
        };
        for (index, item) in items.iter().enumerate() {
            args.push(as_text(item, &format!("{run_path}.args[{index}]"))?);
        }
        return Ok(Postinstall::Run(PostinstallRun {
            source_bin: as_nonempty_text(source, &format!("{run_path}.sourceBin"))?,
            args,
        }));
    }
    return Err(format!("{path} must hold a link or a run"));
}

/// The options object at `path`.
fn decode_options(value: &Value, path: &str) -> DecodeResult<Options> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &[
            "rustComponents",
            "rustTargets",
            "cargoFeatures",
            "cargoGit",
            "assetPattern",
            "allowBuilds",
            "androidPackages",
            "postinstall",
            "binAliases",
            "bootstrap",
        ],
        path,
    )?;
    let mut options: Options = Options::default();
    if let Some(list) = map.get("rustComponents") {
        options.rust_components = Some(as_text_list(list, &format!("{path}.rustComponents"))?);
    }
    if let Some(list) = map.get("rustTargets") {
        options.rust_targets = Some(as_text_list(list, &format!("{path}.rustTargets"))?);
    }
    if let Some(list) = map.get("cargoFeatures") {
        options.cargo_features = Some(as_text_list(list, &format!("{path}.cargoFeatures"))?);
    }
    if let Some(git) = map.get("cargoGit") {
        let git_path: String = format!("{path}.cargoGit");
        let git_map = as_object(git, &git_path)?;
        reject_unknown_keys(git_map, &["url"], &git_path)?;
        let Some(url) = git_map.get("url") else {
            return Err(format!("{git_path}.url is required"));
        };
        options.cargo_git_url = Some(as_nonempty_text(url, &format!("{git_path}.url"))?);
    }
    if let Some(pattern) = map.get("assetPattern") {
        options.asset_pattern = Some(as_nonempty_text(pattern, &format!("{path}.assetPattern"))?);
    }
    if let Some(list) = map.get("allowBuilds") {
        options.allow_builds = Some(as_text_list(list, &format!("{path}.allowBuilds"))?);
    }
    if let Some(list) = map.get("androidPackages") {
        options.android_packages = Some(as_text_list(list, &format!("{path}.androidPackages"))?);
    }
    if let Some(step) = map.get("postinstall") {
        options.postinstall = Some(decode_postinstall(step, &format!("{path}.postinstall"))?);
    }
    if let Some(aliases) = map.get("binAliases") {
        let alias_path: String = format!("{path}.binAliases");
        let alias_map = as_object(aliases, &alias_path)?;
        let mut pairs: Vec<(String, String)> = Vec::new();
        for (alias, primary) in alias_map {
            pairs.push((alias.clone(), as_nonempty_text(primary, &format!("{alias_path}.{alias}"))?));
        }
        options.bin_aliases = Some(pairs);
    }
    if let Some(bootstrap) = map.get("bootstrap") {
        let boot_path: String = format!("{path}.bootstrap");
        let boot_map = as_object(bootstrap, &boot_path)?;
        reject_unknown_keys(boot_map, &["requiresTool", "distribution"], &boot_path)?;
        let Some(distribution) = boot_map.get("distribution") else {
            return Err(format!("{boot_path}.distribution is required"));
        };
        let distribution_text: String = as_text(distribution, &format!("{boot_path}.distribution"))?;
        if distribution_text != "wheel" && distribution_text != "standalone-binary" {
            return Err(format!("{boot_path}.distribution must be \"wheel\" or \"standalone-binary\""));
        }
        let requires_tool: Option<String> = match boot_map.get("requiresTool") {
            Some(tool) => Some(as_nonempty_text(tool, &format!("{boot_path}.requiresTool"))?),
            None => None,
        };
        options.bootstrap = Some(Bootstrap { requires_tool, distribution: distribution_text });
    }
    return Ok(options);
}

/// The declaration fields of the tool `id` in `map`;
/// `extra` lists the field names the lock adds on top.
fn decode_declaration_fields(id: &str, map: &Map<String, Value>, path: &str) -> DecodeResult<Declaration> {
    let Some(request_value) = map.get("request") else {
        return Err(format!("{path}.request is required"));
    };
    let request: Request = decode_request(request_value, &format!("{path}.request"))?;
    let Some(route_value) = map.get("route") else {
        return Err(format!("{path}.route is required"));
    };
    let route_text: String = as_text(route_value, &format!("{path}.route"))?;
    let Some(route) = Route::parse(&route_text) else {
        let names: Vec<&str> = ROUTE_NAMES.iter().map(|(_, name)| return *name).collect();
        return Err(format!("{path}.route {route_text:?} is not a route; the routes are {}", names.join(", ")));
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
    let os: Option<Vec<OsFamily>> = match map.get("os") {
        Some(value) => {
            let names: Vec<String> = as_text_list(value, &format!("{path}.os"))?;
            let mut families: Vec<OsFamily> = Vec::new();
            for name in names {
                let Some(family) = OsFamily::parse(&name) else {
                    return Err(format!("{path}.os {name:?} must be linux, macos or windows"));
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
    return Ok(Declaration {
        id: id.to_string(),
        request,
        route,
        manager,
        package_id,
        bins,
        os,
        env,
        options,
    });
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
pub fn decode_plan_entry(id: &str, value: &Value, path: &str) -> DecodeResult<Declaration> {
    let map = as_object(value, path)?;
    reject_unknown_keys(map, &["request", "route", "manager", "packageId", "bins", "os", "env", "options"], path)?;
    return decode_declaration_fields(id, map, path);
}

/// The artifact rows object at `path`.
fn decode_artifacts(value: &Value, path: &str) -> DecodeResult<Vec<(String, Artifact)>> {
    let map = as_object(value, path)?;
    let mut rows: Vec<(String, Artifact)> = Vec::new();
    for (platform, row) in map {
        let row_path: String = format!("{path}.{platform}");
        let row_map = as_object(row, &row_path)?;
        reject_unknown_keys(row_map, &["url", "sha256"], &row_path)?;
        let Some(url) = row_map.get("url") else {
            return Err(format!("{row_path}.url is required"));
        };
        let Some(sha256) = row_map.get("sha256") else {
            return Err(format!("{row_path}.sha256 is required"));
        };
        let sha256_text: String = as_text(sha256, &format!("{row_path}.sha256"))?;
        if !is_sha256_hex(&sha256_text) {
            return Err(format!("{row_path}.sha256 must be 64 lowercase hex characters"));
        }
        rows.push((platform.clone(), Artifact { url: as_nonempty_text(url, &format!("{row_path}.url"))?, sha256: sha256_text }));
    }
    return Ok(rows);
}

/// Whether `text` is 64 lowercase hex characters.
pub fn is_sha256_hex(text: &str) -> bool {
    return text.len() == 64 && text.bytes().all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
}

/// What:
/// Decodes one lock entry (`tools.<id>` in `toolchain.lock.json`):
/// declaration fields,
/// `version`,
/// `artifacts`.
/// Why:
/// The lock is read by `lock --check`,
/// `sync`,
/// `status` and `update`;
/// a malformed entry stops them all with
///           the field named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function decodeLockEntry(id: string, value: unknown, path: string): LockedTool
/// ```
pub fn decode_lock_entry(id: &str, value: &Value, path: &str) -> DecodeResult<LockedTool> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &["request", "version", "route", "manager", "packageId", "bins", "os", "env", "options", "artifacts"],
        path,
    )?;
    let declaration: Declaration = decode_declaration_fields(id, map, path)?;
    let Some(version_value) = map.get("version") else {
        return Err(format!("{path}.version is required"));
    };
    let version: String = as_nonempty_text(version_value, &format!("{path}.version"))?;
    let artifacts: Vec<(String, Artifact)> = match map.get("artifacts") {
        Some(rows) => decode_artifacts(rows, &format!("{path}.artifacts"))?,
        None => Vec::new(),
    };
    return Ok(LockedTool { declaration, version, artifacts });
}

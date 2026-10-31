//! What:
//! Encoding the model back to JSON values with the key order the lock documents.
//! Why:
//! The lock's byte-stable rendering depends on one fixed order;
//! it is written once,
//! here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // model-encode.ts
//! export function lockEntryValue(tool: LockedTool): Record<string, unknown>
//! export function requestValue(request: Request): Record<string, unknown>
//! ```

/// Imports the JSON value types.
use serde_json::{Map, Value};

/// Imports what this module uses from `super::model`.
use super::model::{Declaration, LockedTool, Options, Postinstall, Request};


/// A JSON array of texts.
fn text_array(items: &[String]) -> Value {
    return Value::Array(items.iter().map(|item| return Value::String(item.clone())).collect());
}

/// The options as a JSON object with the fields in the fixed order the lock documents.
fn options_value(options: &Options) -> Value {
    let mut map: Map<String, Value> = Map::new();
    if let Some(list) = &options.rust_components {
        map.insert("rustComponents".to_string(), text_array(list));
    }
    if let Some(list) = &options.rust_targets {
        map.insert("rustTargets".to_string(), text_array(list));
    }
    if let Some(list) = &options.cargo_features {
        map.insert("cargoFeatures".to_string(), text_array(list));
    }
    if let Some(url) = &options.cargo_git_url {
        let mut git: Map<String, Value> = Map::new();
        git.insert("url".to_string(), Value::String(url.clone()));
        map.insert("cargoGit".to_string(), Value::Object(git));
    }
    if let Some(pattern) = &options.asset_pattern {
        map.insert("assetPattern".to_string(), Value::String(pattern.clone()));
    }
    if let Some(list) = &options.allow_builds {
        map.insert("allowBuilds".to_string(), text_array(list));
    }
    if let Some(list) = &options.android_packages {
        map.insert("androidPackages".to_string(), text_array(list));
    }
    if let Some(step) = &options.postinstall {
        map.insert("postinstall".to_string(), postinstall_value(step));
    }
    if let Some(aliases) = &options.bin_aliases {
        let mut alias_map: Map<String, Value> = Map::new();
        for (alias, primary) in aliases {
            alias_map.insert(alias.clone(), Value::String(primary.clone()));
        }
        map.insert("binAliases".to_string(), Value::Object(alias_map));
    }
    if let Some(bootstrap) = &options.bootstrap {
        let mut boot: Map<String, Value> = Map::new();
        if let Some(tool) = &bootstrap.requires_tool {
            boot.insert("requiresTool".to_string(), Value::String(tool.clone()));
        }
        boot.insert("distribution".to_string(), Value::String(bootstrap.distribution.clone()));
        map.insert("bootstrap".to_string(), Value::Object(boot));
    }
    return Value::Object(map);
}

/// The postinstall step as the JSON object the lock stores.
fn postinstall_value(step: &Postinstall) -> Value {
    let mut map: Map<String, Value> = Map::new();
    map.insert("when".to_string(), Value::String("always".to_string()));
    match step {
        Postinstall::Link(link) => {
            let mut inner: Map<String, Value> = Map::new();
            inner.insert("sourceBin".to_string(), Value::String(link.source_bin.clone()));
            inner.insert("linkName".to_string(), Value::String(link.link_name.clone()));
            if let Some(directory) = &link.link_dir {
                inner.insert("linkDir".to_string(), Value::String(directory.clone()));
            }
            map.insert("link".to_string(), Value::Object(inner));
        }
        Postinstall::Run(command) => {
            let mut inner: Map<String, Value> = Map::new();
            inner.insert("sourceBin".to_string(), Value::String(command.source_bin.clone()));
            inner.insert("args".to_string(), text_array(&command.args));
            map.insert("run".to_string(), Value::Object(inner));
        }
    }
    return Value::Object(map);
}

/// The request as the JSON object the plan and lock store.
pub fn request_value(request: &Request) -> Value {
    let mut map: Map<String, Value> = Map::new();
    match request {
        Request::Latest => {
            map.insert("track".to_string(), Value::String("latest".to_string()));
        }
        Request::Pinned(version) => {
            map.insert("pinned".to_string(), Value::String(version.clone()));
        }
        Request::Channel(channel) => {
            map.insert("channel".to_string(), Value::String(channel.clone()));
        }
    }
    return Value::Object(map);
}

/// What:
/// One lock entry as a JSON object with the keys in the fixed order `request, version, route, manager,
///           packageId, bins, os, env, options, artifacts`;
/// absent optional fields are omitted,
/// never written as null.
/// Why:
/// The order keeps the committed file's diffs small;
/// omission matches the TypeScript lock's owner decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lockEntryValue(tool: LockedTool): Record<string, unknown>
/// ```
pub fn lock_entry_value(tool: &LockedTool) -> Value {
    let declaration: &Declaration = &tool.declaration;
    let mut map: Map<String, Value> = Map::new();
    map.insert("request".to_string(), request_value(&declaration.request));
    map.insert("version".to_string(), Value::String(tool.version.clone()));
    map.insert("route".to_string(), Value::String(declaration.route.as_str().to_string()));
    if let Some(manager) = &declaration.manager {
        map.insert("manager".to_string(), Value::String(manager.clone()));
    }
    if let Some(package_id) = &declaration.package_id {
        map.insert("packageId".to_string(), Value::String(package_id.clone()));
    }
    map.insert("bins".to_string(), text_array(&declaration.bins));
    if let Some(families) = &declaration.os {
        let names: Vec<String> = families.iter().map(|family| return family.as_str().to_string()).collect();
        map.insert("os".to_string(), text_array(&names));
    }
    let mut env: Map<String, Value> = Map::new();
    for (name, text) in &declaration.env {
        env.insert(name.clone(), Value::String(text.clone()));
    }
    map.insert("env".to_string(), Value::Object(env));
    map.insert("options".to_string(), options_value(&declaration.options));
    let mut rows: Map<String, Value> = Map::new();
    for (platform, artifact) in &tool.artifacts {
        let mut row: Map<String, Value> = Map::new();
        row.insert("url".to_string(), Value::String(artifact.url.clone()));
        row.insert("sha256".to_string(), Value::String(artifact.sha256.clone()));
        rows.insert(platform.clone(), Value::Object(row));
    }
    map.insert("artifacts".to_string(), Value::Object(rows));
    return Value::Object(map);
}

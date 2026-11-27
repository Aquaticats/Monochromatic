//! What:
//! The `options` object of a plan or lock entry:
//! the Rust components and targets,
//! the `cargo` features,
//! the release asset pattern and its tag prefix,
//! the builds npm may run,
//! the Android packages,
//! the postinstall step,
//! the bin aliases and the bootstrap a PyPI wheel needs.
//! Why:
//! Every option is optional and every key is known,
//! so a misspelled key is refused with its path rather than ignored,
//! and a tag prefix is checked here because it is the start of a release tag.
//! A caller imports from `model_decode` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function decodeOptions(value: unknown, path: string): Options;
//! ```

/// Imports the JSON value the options object is read from.
use serde_json::Value;

/// Imports the model types an option is read into.
use super::super::model::{
    Bootstrap,
    Options,
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
/// Imports the check that a bin alias is one plain program name.
use super::super::segment::check_bin_name;

/// Imports the postinstall step an entry may declare,
/// and the check that a tag prefix may be the start of a release tag.
use super::a_request_and_its_postinstall::{
    decode_postinstall,
    is_tag_character,
};
/// The options object at `path`.
pub(super) fn decode_options(
    value: &Value,
    path: &str,
) -> DecodeResult<Options> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &[
            "rustComponents",
            "rustTargets",
            "cargoFeatures",
            "asset",
            "tagPrefix",
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
    if let Some(pattern) = map.get("asset") {
        options.asset = Some(as_nonempty_text(pattern, &format!("{path}.asset"))?);
    }
    if let Some(prefix) = map.get("tagPrefix") {
        let prefix_path: String = format!("{path}.tagPrefix");
        let text: String = as_nonempty_text(prefix, &prefix_path)?;
        if !text.chars().all(is_tag_character) {
            return Err(format!(
                "{prefix_path}: {text:?} may hold only letters, digits, '.', '_' and '-' (it is the start of a release tag)"
            ));
        }
        options.tag_prefix = Some(text);
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
            let primary_path: String = format!("{alias_path}.{alias}");
            let primary_name: String = as_nonempty_text(primary, &primary_path)?;
            for name in [alias, &primary_name] {
                if let Err(message) = check_bin_name(name) {
                    return Err(format!("{primary_path}: {message}"));
                }
            }
            pairs.push((alias.clone(), primary_name));
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
        let distribution_text: String =
            as_text(distribution, &format!("{boot_path}.distribution"))?;
        if distribution_text != "wheel" && distribution_text != "standalone-binary" {
            return Err(format!(
                "{boot_path}.distribution must be \"wheel\" or \"standalone-binary\""
            ));
        }
        let requires_tool: Option<String> = match boot_map.get("requiresTool") {
            Some(tool) => Some(as_nonempty_text(
                tool,
                &format!("{boot_path}.requiresTool"),
            )?),
            None => None,
        };
        options.bootstrap = Some(Bootstrap {
            requires_tool,
            distribution: distribution_text,
        });
    }
    return Ok(options);
}

//! What:
//! The request text at one path,
//! and the postinstall step an entry's `options.postinstall` describes:
//! one link,
//! or one command with its arguments.
//! Why:
//! The model carries one step,
//! so an object holding both a link and a run is refused with both fields named rather than keeping one and
//! silently dropping the other;
//! and the arguments are read with the check that allows an empty string,
//! which a link's own fields may not be.
//! A caller imports from `model_decode` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function decodeRequest(value: unknown, path: string): Request;
//! function decodePostinstall(value: unknown, path: string): Postinstall;
//! function isTagCharacter(character: string): boolean;
//! ```

/// Imports the JSON value every decoder reads.
use serde_json::Value;

/// Imports the model types a request and a postinstall step are.
use super::super::model::{
    Postinstall,
    PostinstallLink,
    PostinstallRun,
    Request,
};
/// Imports the decoding checks this file shares with its siblings.
use super::super::model_decode_support::{
    DecodeResult,
    as_nonempty_text,
    as_object,
    as_text,
    reject_unknown_keys,
};
/// The request text at `path`:
/// `<origin>@<range>`.
pub(super) fn decode_request(
    value: &Value,
    path: &str,
) -> DecodeResult<Request> {
    let text: String = as_nonempty_text(value, path)?;
    return match Request::parse(&text) {
        Ok(request) => Ok(request),
        Err(message) => Err(format!("{path}: {message}")),
    };
}

/// The postinstall object at `path`.
pub(super) fn decode_postinstall(
    value: &Value,
    path: &str,
) -> DecodeResult<Postinstall> {
    let map = as_object(value, path)?;
    reject_unknown_keys(map, &["when", "link", "run"], path)?;
    if map.get("when").and_then(Value::as_str) != Some("always") {
        return Err(format!("{path}.when must be \"always\""));
    }
    // The model carries one step, so an entry holding both is refused with both fields named rather than keeping the
    // link and silently dropping the run.
    if map.contains_key("link") && map.contains_key("run") {
        return Err(format!(
            "{path} holds both a link and a run; state one step"
        ));
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
            Some(directory) => Some(as_nonempty_text(
                directory,
                &format!("{link_path}.linkDir"),
            )?),
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

/// Whether a character may stand in the start of a release tag.
pub(super) fn is_tag_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-');
}

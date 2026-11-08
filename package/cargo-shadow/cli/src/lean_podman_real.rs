//! What:
//! The real Podman behind `PodmanApi`:
//! each call is one bounded `podman` run as the current user,
//! with its output parsed from JSON or plain lines.
//! Why:
//! Listing 4313 images took 209 s here (measured 2026-11-07),
//! so every call has its own deadline and a call that passes it is an error,
//! never a wait without end.
//! Nothing here forces a removal and nothing here runs as root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const realPodman = (limits: Limits): PodmanApi
//! ```

/// Child processes and time.
use std::{process::Command, time::Duration};

/// The bounded run.
use crate::bounded_run::{Bound, RunError, output_within};
/// The cleanup's interface and records.
use crate::lean_podman_cleanup::{Image, PodmanApi, Volume};
/// Timestamps with an offset.
use crate::time_text::parse_rfc3339_millis;

/// How long a listing may take.
pub const LIST_LIMIT: Duration = Duration::from_secs(1_500);

/// How long one removal call may take.
pub const REMOVE_LIMIT: Duration = Duration::from_secs(900);

/// Real Podman.
pub struct RealPodman;

/// What:
/// `fn run(arguments: &[&str], limit: Duration) -> Result<String, String>` runs `podman` with a deadline and returns standard output.
/// Why:
/// Every Podman call goes through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(args: string[], limit: number): string  // throws
/// ```
fn run(arguments: &[&str], limit: Duration) -> Result<String, String> {
    let bound: Bound = Bound { limit, what: format!("podman {}", arguments.join(" ")), rerun: String::from("cargo-shadow lean") };
    return match output_within(Command::new("podman").args(arguments), None, &bound) {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        Err(RunError::TimedOut { what, limit: passed, .. }) => Err(format!("{what} did not finish within {} s", passed.as_secs())),
        Err(other) => Err(format!("{other:?}")),
    };
}

/// What:
/// `pub fn parse_images(text: &str) -> Result<Vec<Image>, String>` reads `podman images --all --format json`.
/// Why:
/// One row per tag,
/// or one `<none>` row for an untagged image.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseImages(text: string): Image[]
/// ```
pub fn parse_images(text: &str) -> Result<Vec<Image>, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|error| { return error.to_string(); })?;
    let list: &Vec<serde_json::Value> = value.as_array().ok_or_else(|| { return String::from("the image list is not an array"); })?;
    let mut images: Vec<Image> = Vec::new();
    for entry in list {
        let id: String = entry.get("Id").and_then(serde_json::Value::as_str).unwrap_or_default().chars().take(12).collect();
        let created: i64 = entry.get("Created").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let mut named: bool = false;
        if let Some(names) = entry.get("Names").and_then(serde_json::Value::as_array) {
            for name in names.iter().filter_map(serde_json::Value::as_str) {
                if let Some((repository, tag)) = name.rsplit_once(':')
                    && !tag.contains('/')
                {
                    images.push(Image { id: id.clone(), repository: repository.to_string(), tag: tag.to_string(), created });
                    named = true;
                }
            }
        }
        if !named {
            images.push(Image { id, repository: String::from("<none>"), tag: String::from("<none>"), created });
        }
    }
    return Ok(images);
}

/// What:
/// `pub fn parse_volumes(text: &str) -> Result<Vec<Volume>, String>` reads `podman volume ls --format json`.
/// Why:
/// The creation time is an RFC 3339 string with an offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseVolumes(text: string): Volume[]
/// ```
pub fn parse_volumes(text: &str) -> Result<Vec<Volume>, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|error| { return error.to_string(); })?;
    let list: &Vec<serde_json::Value> = value.as_array().ok_or_else(|| { return String::from("the volume list is not an array"); })?;
    let mut volumes: Vec<Volume> = Vec::new();
    for entry in list {
        let name: String = entry.get("Name").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let created: i64 = entry.get("CreatedAt").and_then(serde_json::Value::as_str).and_then(parse_rfc3339_millis).map_or(i64::MAX, |millis| { return millis / 1_000; });
        if !name.is_empty() {
            volumes.push(Volume { name, created });
        }
    }
    return Ok(volumes);
}

impl PodmanApi for RealPodman {
    fn images(&self) -> Result<Vec<Image>, String> {
        return parse_images(&run(&["images", "--all", "--format", "json"], LIST_LIMIT)?);
    }

    fn container_image_ids(&self) -> Result<Vec<String>, String> {
        let text: String = run(&["ps", "--all", "--no-trunc", "--format", "{{.ImageID}}"], LIST_LIMIT)?;
        return Ok(text.lines().map(|line| { return line.trim().chars().take(12).collect::<String>(); }).filter(|id| { return !id.is_empty(); }).collect());
    }

    fn dangling_volumes(&self) -> Result<Vec<Volume>, String> {
        return parse_volumes(&run(&["volume", "ls", "--filter", "dangling=true", "--format", "json"], LIST_LIMIT)?);
    }

    fn remove_images(&self, ids: &[String]) -> Result<(), String> {
        let mut arguments: Vec<&str> = vec!["rmi"];
        arguments.extend(ids.iter().map(String::as_str));
        run(&arguments, REMOVE_LIMIT).map(|_| { return (); })
    }

    fn remove_volume(&self, name: &str) -> Result<(), String> {
        return run(&["volume", "rm", name], REMOVE_LIMIT).map(|_| { return (); });
    }
}

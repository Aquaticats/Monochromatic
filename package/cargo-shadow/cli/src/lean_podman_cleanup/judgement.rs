//! What:
//! Which images to remove and which to keep,
//! and which dangling volumes to remove.
//! Why:
//! Each decision is a pure function of Podman's lists and the sources' words,
//! so it is tested alone,
//! and a kept image carries the reason a person can read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function judgeImages(images: Image[], containerImages: string[], refs: Set<string>, graceSeconds: number, now: number): [string[], [string, string][]]
//! export function judgeVolumes(volumes: Volume[], refs: Set<string>, graceSeconds: number, now: number): [string[], [string, string][]]
//! ```
/// Imports the five questions the judgement asks.
use super::image_checks::{
    any_tag_named,
    is_newest_of_a_repository,
    used_by_a_container,
};
/// Imports one image row and one dangling volume.
use super::podman_api::{
    Image,
    Volume,
};
/// Imports the sets and the repository table the judgement builds.
use std::collections::{
    HashMap,
    HashSet,
};
/// What:
/// `pub fn judge_images(images: &[Image], container_images: &[String], references: &HashSet<String>, grace_seconds: i64, now: i64) -> (Vec<String>, Vec<(String, String)>)`
/// decides which image ids to remove and which to keep with a reason.
/// Why:
/// The decision is a pure function of Podman's lists and the sources' words,
/// so it is tested alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function judgeImages(images: Image[], containerImages: string[], refs: Set<string>, graceSeconds: number, now: number): { remove: string[]; keep: [string, string][] }
/// ```
pub fn judge_images(
    images: &[Image],
    container_images: &[String],
    references: &HashSet<String>,
    grace_seconds: i64,
    now: i64,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut newest_of_repository: HashMap<&str, (i64, &str)> = HashMap::new();
    for image in images {
        if image.repository == "<none>" {
            continue;
        }
        let entry = newest_of_repository
            .entry(image.repository.as_str())
            .or_insert((image.created, image.id.as_str()));
        if image.created > entry.0 {
            *entry = (image.created, image.id.as_str());
        }
    }
    let mut order: Vec<String> = Vec::new();
    let mut by_id: HashMap<String, Vec<&Image>> = HashMap::new();
    for image in images {
        if !by_id.contains_key(&image.id) {
            order.push(image.id.clone());
        }
        by_id.entry(image.id.clone()).or_default().push(image);
    }
    let mut remove: Vec<String> = Vec::new();
    let mut keep: Vec<(String, String)> = Vec::new();
    for id in order {
        let rows: &Vec<&Image> = &by_id[&id];
        let mut created: i64 = 0;
        for row in rows {
            created = created.max(row.created);
        }
        if used_by_a_container(&id, container_images) {
            keep.push((id, String::from("a container (in some state) uses it")));
        } else if now - created <= grace_seconds {
            keep.push((
                id,
                format!(
                    "created {} s ago, within the grace period of {grace_seconds} s",
                    now - created
                ),
            ));
        } else if any_tag_named(rows, references) {
            keep.push((
                id,
                String::from("a tag of it is named by the repository's current sources"),
            ));
        } else if is_newest_of_a_repository(rows, &id, &newest_of_repository) {
            keep.push((
                id,
                String::from("the newest image of its repository, kept as its current one"),
            ));
        } else {
            remove.push(id);
        }
    }
    return (remove, keep);
}

/// What:
/// `pub fn judge_volumes(volumes: &[Volume], references: &HashSet<String>, grace_seconds: i64, now: i64) -> (Vec<String>, Vec<(String, String)>)`
/// decides which dangling volumes to remove.
/// Why:
/// A volume a current source names (a cargo registry cache) is kept,
/// and so is a young one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function judgeVolumes(volumes: Volume[], refs: Set<string>, graceSeconds: number, now: number): { remove: string[]; keep: [string, string][] }
/// ```
pub fn judge_volumes(
    volumes: &[Volume],
    references: &HashSet<String>,
    grace_seconds: i64,
    now: i64,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut remove: Vec<String> = Vec::new();
    let mut keep: Vec<(String, String)> = Vec::new();
    for volume in volumes {
        if references.contains(&volume.name) {
            keep.push((volume.name.clone(), String::from("a current task names it")));
        } else if now - volume.created <= grace_seconds {
            keep.push((
                volume.name.clone(),
                format!(
                    "created {} s ago, within the grace period of {grace_seconds} s",
                    now - volume.created
                ),
            ));
        } else {
            remove.push(volume.name.clone());
        }
    }
    return (remove, keep);
}

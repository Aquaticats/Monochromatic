//! What:
//! The removal of container images and volumes that nothing uses and the repository's current sources do not name,
//! after the grace period.
//! Why:
//! Test and build tasks leave an image per run (3462 images of 391.6 GB and 33 volumes of 87.6 GB on 2026-11-04,
//! `doc/planning/lean-build-dirs.md`),
//! and Podman never removes them.
//! The owner's answer ("Podman images,
//! volumes (Recommended)"):
//! remove images and volumes no container uses and no current image tag of this repository names,
//! each after the grace period,
//! as the user and never as root.
//! A container in any state uses its image,
//! so an image a container holds is kept (a person decides about containers).
//! A Buildah working container holds its image too and is invisible to `podman ps --all`,
//! which needs `--external` to list it (measured 2026-11-08,
//! `lean_podman_real.rs`).
//! The newest image of each repository is kept as the repository's current one.
//! Podman is asked to remove plainly (never with `--force`),
//! so Podman itself refuses an image a container started in the meantime.
//! Each judged image is asked for by every tag it carries,
//! because Podman refuses an image with more than one tag by id;
//! a refusal that says the image is not known counts it as gone,
//! because removing one image also removes an ancestor that only it referenced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function cleanPodman(api: PodmanApi, references: Set<string>, graceSeconds: number, now: number, dryRun: boolean, log: string[]): PodmanOutcome
//! ```

/// The streaming log line.
use crate::lean_run_report::emit;
/// Sets.
use std::collections::{HashMap, HashSet};

/// One image row of `podman images --all` (one per tag).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Image {
    /// The image id as Podman prints it.
    pub id: String,
    /// The repository (`<none>` for an untagged image).
    pub repository: String,
    /// The tag (`<none>` for an untagged image).
    pub tag: String,
    /// The creation time in seconds from the epoch.
    pub created: i64,
}

/// One volume Podman reports as dangling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Volume {
    /// The volume name.
    pub name: String,
    /// The creation time in seconds from the epoch.
    pub created: i64,
}

/// What the cleanup needs from Podman.
pub trait PodmanApi {
    /// Every image row.
    fn images(&self) -> Result<Vec<Image>, String>;
    /// The image id of every container in any state,
    /// the external (Buildah working) ones included.
    fn container_image_ids(&self) -> Result<Vec<String>, String>;
    /// The volumes no container uses.
    fn dangling_volumes(&self) -> Result<Vec<Volume>, String>;
    /// Removes one image through every reference that names it,
    /// never forcing.
    fn remove_image(&self, references: &[String]) -> Result<(), String>;
    /// Removes a volume,
    /// never forcing.
    fn remove_volume(&self, name: &str) -> Result<(), String>;
}

/// What Podman's refusal of one removal says about the image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// Podman does not know the image any more:
    /// it went with another image of this run.
    AlreadyGone,
    /// A container Podman knows holds the image.
    InUse,
}

/// The words of a Podman refusal that says the image is gone,
/// measured on podman 6.1.3 (2026-11-08).
pub const ALREADY_GONE: &str = "image not known";

/// The words of a Podman refusal that says a container holds the image,
/// measured on podman 6.1.3 (2026-11-08).
pub const IN_USE: &str = "image is in use by a container";

/// What:
/// `pub fn refusal_of(message: &str) -> Option<Refusal>` reads the two refusals this cleanup understands.
/// Why:
/// Removing an image also removes an ancestor that only it referenced,
/// so a later call of the same run reads "image not known" for an image that is gone;
/// that is the outcome the run wanted,
/// not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refusalOf(message: string): Refusal | undefined
/// ```
pub fn refusal_of(message: &str) -> Option<Refusal> {
    if message.contains(ALREADY_GONE) {
        return Some(Refusal::AlreadyGone);
    }
    if message.contains(IN_USE) {
        return Some(Refusal::InUse);
    }
    return None;
}

/// What:
/// `pub fn image_references(rows: &[&Image], id: &str) -> Vec<String>` is what Podman is asked to remove for one image:
/// every `repository:tag` it carries,
/// or its id when it carries none.
/// Why:
/// Podman refuses an image with more than one tag by id ("unable to delete image ... by ID with more than one tag"),
/// and untags rather than deletes while another tag of it remains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function imageReferences(rows: Image[], id: string): string[]
/// ```
pub fn image_references(rows: &[&Image], id: &str) -> Vec<String> {
    let mut references: Vec<String> = Vec::new();
    for row in rows {
        if row.repository == "<none>" || row.tag == "<none>" {
            continue;
        }
        let reference: String = format!("{}:{}", row.repository, row.tag);
        if !references.contains(&reference) {
            references.push(reference);
        }
    }
    if references.is_empty() {
        references.push(id.to_string());
    }
    return references;
}

/// What a cleanup did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PodmanOutcome {
    /// Image ids removed or that would be.
    pub images: Vec<String>,
    /// Image ids that were gone when this run asked for them,
    /// because removing another image of the run took them.
    pub gone_with_another: Vec<String>,
    /// Image ids kept,
    /// with the reason of each.
    pub kept_images: Vec<(String, String)>,
    /// Volumes removed or that would be.
    pub volumes: Vec<String>,
    /// Volumes kept,
    /// with the reason of each.
    pub kept_volumes: Vec<(String, String)>,
    /// Failures.
    pub failures: Vec<String>,
}

/// What:
/// `fn same_image(left: &str, right: &str) -> bool` compares two image ids that may be short or full.
/// Why:
/// `podman ps` prints full ids and `podman images` short ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameImage = (a: string, b: string): boolean => a.startsWith(b) || b.startsWith(a);
/// ```
fn same_image(left: &str, right: &str) -> bool {
    return !left.is_empty() && !right.is_empty() && (left.starts_with(right) || right.starts_with(left));
}

/// What:
/// `fn tag_is_named(image: &Image, references: &HashSet<String>) -> bool` says whether the sources name this tag:
/// `repository:tag` spelled out,
/// or the bare repository for the tag `latest`.
/// Why:
/// It is "a current image tag of this repository".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tagIsNamed = (image: Image, refs: Set<string>): boolean
/// ```
fn tag_is_named(image: &Image, references: &HashSet<String>) -> bool {
    if image.repository == "<none>" {
        return false;
    }
    return references.contains(&format!("{}:{}", image.repository, image.tag)) || (image.tag == "latest" && references.contains(&image.repository));
}

/// What:
/// `fn used_by_a_container(id: &str, container_images: &[String]) -> bool` says whether any container's image is this one.
/// Why:
/// A container in any state uses its image.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const usedByAContainer = (id: string, containerImages: string[]): boolean
/// ```
fn used_by_a_container(id: &str, container_images: &[String]) -> bool {
    for used in container_images {
        if same_image(used, id) {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn any_tag_named(rows: &[&Image], references: &HashSet<String>) -> bool` says whether the sources name any tag of an image.
/// Why:
/// One image can carry several tags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anyTagNamed = (rows: Image[], refs: Set<string>): boolean
/// ```
fn any_tag_named(rows: &[&Image], references: &HashSet<String>) -> bool {
    for row in rows {
        if tag_is_named(row, references) {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn is_newest_of_a_repository(rows: &[&Image], id: &str, newest: &HashMap<&str, (i64, &str)>) -> bool` says whether the image is the
/// newest of any repository it is tagged in.
/// Why:
/// The newest image of a repository is kept as its current one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNewestOfARepository = (rows: Image[], id: string, newest: Map<string, [number, string]>): boolean
/// ```
fn is_newest_of_a_repository(rows: &[&Image], id: &str, newest: &HashMap<&str, (i64, &str)>) -> bool {
    for row in rows {
        if let Some(entry) = newest.get(row.repository.as_str())
            && entry.1 == id
        {
            return true;
        }
    }
    return false;
}

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
pub fn judge_images(images: &[Image], container_images: &[String], references: &HashSet<String>, grace_seconds: i64, now: i64) -> (Vec<String>, Vec<(String, String)>) {
    let mut newest_of_repository: HashMap<&str, (i64, &str)> = HashMap::new();
    for image in images {
        if image.repository == "<none>" {
            continue;
        }
        let entry = newest_of_repository.entry(image.repository.as_str()).or_insert((image.created, image.id.as_str()));
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
            keep.push((id, format!("created {} s ago, within the grace period of {grace_seconds} s", now - created)));
        } else if any_tag_named(rows, references) {
            keep.push((id, String::from("a tag of it is named by the repository's current sources")));
        } else if is_newest_of_a_repository(rows, &id, &newest_of_repository) {
            keep.push((id, String::from("the newest image of its repository, kept as its current one")));
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
pub fn judge_volumes(volumes: &[Volume], references: &HashSet<String>, grace_seconds: i64, now: i64) -> (Vec<String>, Vec<(String, String)>) {
    let mut remove: Vec<String> = Vec::new();
    let mut keep: Vec<(String, String)> = Vec::new();
    for volume in volumes {
        if references.contains(&volume.name) {
            keep.push((volume.name.clone(), String::from("a current task names it")));
        } else if now - volume.created <= grace_seconds {
            keep.push((volume.name.clone(), format!("created {} s ago, within the grace period of {grace_seconds} s", now - volume.created)));
        } else {
            remove.push(volume.name.clone());
        }
    }
    return (remove, keep);
}

/// What:
/// `pub fn clean_podman(api: &dyn PodmanApi, references: &HashSet<String>, grace_seconds: i64, now: i64, dry_run: bool, log: &mut Vec<String>) -> PodmanOutcome`
/// removes what the judgements allow.
/// Why:
/// Every removed id has a log line with the proof it was judged on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function cleanPodman(api: PodmanApi, refs: Set<string>, graceSeconds: number, now: number, dryRun: boolean, log: string[]): PodmanOutcome
/// ```
pub fn clean_podman(api: &dyn PodmanApi, references: &HashSet<String>, grace_seconds: i64, now: i64, dry_run: bool, log: &mut Vec<String>) -> PodmanOutcome {
    let mut outcome: PodmanOutcome = PodmanOutcome::default();
    match (api.images(), api.container_image_ids()) {
        (Ok(images), Ok(containers)) => {
            let (remove, keep) = judge_images(&images, &containers, references, grace_seconds, now);
            outcome.kept_images = keep;
            let proof: String = format!("podman listed {} image rows and {} containers (any state); none of the containers uses it, no current source names a tag of it, it is not the newest of its repository, and it is older than {grace_seconds} s", images.len(), containers.len());
            for id in remove {
                let mut rows: Vec<&Image> = Vec::new();
                for row in &images {
                    if row.id == id {
                        rows.push(row);
                    }
                }
                let named: Vec<String> = image_references(&rows, &id);
                emit(log, format!("{} image {id} ({}): {proof}", if dry_run { "would remove" } else { "removing" }, named.join(" ")));
                if dry_run {
                    outcome.images.push(id);
                    continue;
                }
                match api.remove_image(&named) {
                    Ok(()) => outcome.images.push(id),
                    Err(reason) => match refusal_of(&reason) {
                        Some(Refusal::AlreadyGone) => {
                            emit(log, format!("image {id} was gone when this run asked: {reason}"));
                            outcome.gone_with_another.push(id);
                        }
                        Some(Refusal::InUse) => {
                            emit(log, format!("keeping image {id}: {reason}"));
                            outcome.kept_images.push((id, format!("a container holds it: {reason}")));
                        }
                        None => outcome.failures.push(format!("podman rmi {}: {reason}", named.join(" "))),
                    },
                }
            }
        }
        (Err(reason), _) | (_, Err(reason)) => outcome.failures.push(format!("podman could not list images or containers: {reason}")),
    }
    match api.dangling_volumes() {
        Ok(volumes) => {
            let (remove, keep) = judge_volumes(&volumes, references, grace_seconds, now);
            outcome.kept_volumes = keep;
            for name in remove {
                emit(log, format!("{} volume {name}: podman reports it dangling (no container in any state mounts it), no current source names it, older than {grace_seconds} s", if dry_run { "would remove" } else { "removing" }));
                if dry_run {
                    outcome.volumes.push(name);
                    continue;
                }
                match api.remove_volume(&name) {
                    Ok(()) => outcome.volumes.push(name),
                    Err(reason) => outcome.failures.push(format!("podman volume rm {name}: {reason}")),
                }
            }
        }
        Err(reason) => outcome.failures.push(format!("podman could not list volumes: {reason}")),
    }
    return outcome;
}

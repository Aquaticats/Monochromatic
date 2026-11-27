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
//!
//! The cleanup's parts are one file each:
//! what it asks of Podman and what it did (`podman_api.rs`),
//! Podman's refusals and the words it reads them by (`refusals.rs`),
//! the five questions the judgement asks (`image_checks.rs`),
//! and the two judgements (`judgement.rs`).

/// The streaming log line.
use crate::lean_run_report::emit;
/// The set the current sources' words are held in.
use std::collections::HashSet;

/// The five questions the judgement asks of Podman's lists.
mod image_checks;
/// Which images and dangling volumes to remove.
mod judgement;
/// What the cleanup asks of Podman and what it did.
mod podman_api;
/// Podman's refusals and the words it reads them by.
mod refusals;

/// Re-exports the two judgements.
pub use self::judgement::{
    judge_images,
    judge_volumes,
};
/// Re-exports what the cleanup asks of Podman and what it did.
pub use self::podman_api::{
    Image,
    PodmanApi,
    PodmanOutcome,
    Volume,
};
/// Re-exports Podman's refusals and the words they are read by.
pub use self::refusals::{
    ALREADY_GONE,
    IN_USE,
    Refusal,
    refusal_of,
};

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
pub fn image_references(
    rows: &[&Image],
    id: &str,
) -> Vec<String> {
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
pub fn clean_podman(
    api: &dyn PodmanApi,
    references: &HashSet<String>,
    grace_seconds: i64,
    now: i64,
    dry_run: bool,
    log: &mut Vec<String>,
) -> PodmanOutcome {
    let mut outcome: PodmanOutcome = PodmanOutcome::default();
    match (api.images(), api.container_image_ids()) {
        (Ok(images), Ok(containers)) => {
            let (remove, keep) = judge_images(&images, &containers, references, grace_seconds, now);
            outcome.kept_images = keep;
            let proof: String = format!(
                "podman listed {} image rows and {} containers (any state); none of the containers uses it, no current source names a tag of it, it is not the newest of its repository, and it is older than {grace_seconds} s",
                images.len(),
                containers.len()
            );
            for id in remove {
                let mut rows: Vec<&Image> = Vec::new();
                for row in &images {
                    if row.id == id {
                        rows.push(row);
                    }
                }
                let named: Vec<String> = image_references(&rows, &id);
                emit(
                    log,
                    format!(
                        "{} image {id} ({}): {proof}",
                        if dry_run { "would remove" } else { "removing" },
                        named.join(" ")
                    ),
                );
                if dry_run {
                    outcome.images.push(id);
                    continue;
                }
                match api.remove_image(&named) {
                    Ok(()) => outcome.images.push(id),
                    Err(reason) => match refusal_of(&reason) {
                        Some(Refusal::AlreadyGone) => {
                            emit(
                                log,
                                format!("image {id} was gone when this run asked: {reason}"),
                            );
                            outcome.gone_with_another.push(id);
                        }
                        Some(Refusal::InUse) => {
                            emit(log, format!("keeping image {id}: {reason}"));
                            outcome
                                .kept_images
                                .push((id, format!("a container holds it: {reason}")));
                        }
                        None => outcome
                            .failures
                            .push(format!("podman rmi {}: {reason}", named.join(" "))),
                    },
                }
            }
        }
        (Err(reason), _) | (_, Err(reason)) => outcome.failures.push(format!(
            "podman could not list images or containers: {reason}"
        )),
    }
    match api.dangling_volumes() {
        Ok(volumes) => {
            let (remove, keep) = judge_volumes(&volumes, references, grace_seconds, now);
            outcome.kept_volumes = keep;
            for name in remove {
                emit(
                    log,
                    format!(
                        "{} volume {name}: podman reports it dangling (no container in any state mounts it), no current source names it, older than {grace_seconds} s",
                        if dry_run { "would remove" } else { "removing" }
                    ),
                );
                if dry_run {
                    outcome.volumes.push(name);
                    continue;
                }
                match api.remove_volume(&name) {
                    Ok(()) => outcome.volumes.push(name),
                    Err(reason) => outcome
                        .failures
                        .push(format!("podman volume rm {name}: {reason}")),
                }
            }
        }
        Err(reason) => outcome
            .failures
            .push(format!("podman could not list volumes: {reason}")),
    }
    return outcome;
}

/// The tests of the Podman cleanup.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "lean_podman_cleanup_tests.rs"]
mod lean_podman_cleanup_tests;

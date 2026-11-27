//! What:
//! What the cleanup asks of Podman:
//! one image row,
//! one dangling volume,
//! the four calls it makes,
//! and what a cleanup did.
//! Why:
//! The judgement and the run read Podman's answers through this one trait,
//! so both are tested against lists a test writes instead of a Podman to start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Image = { id: string; repository: string; tag: string; created: number; size: number };
//! type Volume = { name: string; created: number };
//! interface PodmanApi { images(): Image[]; containerImageIds(): string[]; danglingVolumes(): Volume[]; removeImage(named: string[]): void; removeVolume(name: string): void }
//! type PodmanOutcome = { images: string[]; goneWithAnother: string[]; keptImages: [string, string][]; volumes: string[]; keptVolumes: [string, string][]; failures: string[] };
//! ```
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
    fn remove_image(
        &self,
        references: &[String],
    ) -> Result<(), String>;
    /// Removes a volume,
    /// never forcing.
    fn remove_volume(
        &self,
        name: &str,
    ) -> Result<(), String>;
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

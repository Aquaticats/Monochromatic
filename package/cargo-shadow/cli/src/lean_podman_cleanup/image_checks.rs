//! What:
//! The five questions the judgement asks of Podman's lists:
//! whether two image ids name one image,
//! whether a current source names a tag of an image,
//! whether a container uses it,
//! whether any tag of an image is named,
//! and whether it is the newest of a repository it is tagged in.
//! Why:
//! Each question is one line of the rule the owner set,
//! and each is asked of Podman's answers alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sameImage = (a: string, b: string): boolean => a.startsWith(b) || b.startsWith(a);
//! const tagIsNamed = (image: Image, refs: Set<string>): boolean => refs.has(`${image.repository}:${image.tag}`) || (image.tag === 'latest' && refs.has(image.repository));
//! const usedByAContainer = (id: string, containerImages: string[]): boolean => containerImages.some((held) => sameImage(id, held));
//! const anyTagNamed = (rows: Image[], refs: Set<string>): boolean => rows.some((row) => tagIsNamed(row, refs));
//! const isNewestOfARepository = (rows: Image[], id: string, newest: Map<string, [number, string]>): boolean => ...
//! ```
/// Imports one image row of `podman images --all`.
use super::podman_api::Image;
/// Imports the two maps the answers are looked up in.
use std::collections::{
    HashMap,
    HashSet,
};
/// What:
/// `fn same_image(left: &str, right: &str) -> bool` compares two image ids that may be short or full.
/// Why:
/// `podman ps` prints full ids and `podman images` short ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameImage = (a: string, b: string): boolean => a.startsWith(b) || b.startsWith(a);
/// ```
fn same_image(
    left: &str,
    right: &str,
) -> bool {
    return !left.is_empty()
        && !right.is_empty()
        && (left.starts_with(right) || right.starts_with(left));
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
fn tag_is_named(
    image: &Image,
    references: &HashSet<String>,
) -> bool {
    if image.repository == "<none>" {
        return false;
    }
    return references.contains(&format!("{}:{}", image.repository, image.tag))
        || (image.tag == "latest" && references.contains(&image.repository));
}

/// What:
/// `pub(super) fn used_by_a_container(id: &str, container_images: &[String]) -> bool` says whether any container's image is this one.
/// Why:
/// A container in any state uses its image.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const usedByAContainer = (id: string, containerImages: string[]): boolean
/// ```
pub(super) fn used_by_a_container(
    id: &str,
    container_images: &[String],
) -> bool {
    for used in container_images {
        if same_image(used, id) {
            return true;
        }
    }
    return false;
}

/// What:
/// `pub(super) fn any_tag_named(rows: &[&Image], references: &HashSet<String>) -> bool` says whether the sources name any tag of an image.
/// Why:
/// One image can carry several tags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anyTagNamed = (rows: Image[], refs: Set<string>): boolean
/// ```
pub(super) fn any_tag_named(
    rows: &[&Image],
    references: &HashSet<String>,
) -> bool {
    for row in rows {
        if tag_is_named(row, references) {
            return true;
        }
    }
    return false;
}

/// What:
/// `pub(super) fn is_newest_of_a_repository(rows: &[&Image], id: &str, newest: &HashMap<&str, (i64, &str)>) -> bool` says whether the image is the
/// newest of any repository it is tagged in.
/// Why:
/// The newest image of a repository is kept as its current one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNewestOfARepository = (rows: Image[], id: string, newest: Map<string, [number, string]>): boolean
/// ```
pub(super) fn is_newest_of_a_repository(
    rows: &[&Image],
    id: &str,
    newest: &HashMap<&str, (i64, &str)>,
) -> bool {
    for row in rows {
        if let Some(entry) = newest.get(row.repository.as_str())
            && entry.1 == id
        {
            return true;
        }
    }
    return false;
}

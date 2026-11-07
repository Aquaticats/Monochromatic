//! What:
//!  A probe that answers the check's questions from an in-memory world,
//! and counts every question.
//! Why:
//!  The shared table drives the check against worlds written in JSON,
//! and the gate controls assert how many questions a command costs.
//! Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const probe = probeOf(world); // `probeOf` in check-link.unit.test.ts
//! ```

use crate::repository_link_probe::{EntryKind, LinkProbe};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What:
///  One world:
/// the kind of each path,
/// the text of each link,
/// what each path resolves to,
/// the text of each small file,
/// the filesystem UUID of each path,
/// and the mount points.
/// Why:
///  Mirrors `World` of `check-link.unit.test.ts`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type World = { entries; links; real; texts; uuids; mounted };
/// ```
#[derive(Clone, Debug, Default)]
pub(crate) struct World {
    /// The kind of each path;
    /// a path that is absent is missing.
    pub entries: BTreeMap<String, EntryKind>,
    /// The text of each link.
    pub links: BTreeMap<String, String>,
    /// What each path resolves to;
    /// a path that is absent does not resolve.
    pub real: BTreeMap<String, String>,
    /// The text of each small file.
    pub texts: BTreeMap<String, String>,
    /// The filesystem UUID of each path.
    pub uuids: BTreeMap<String, String>,
    /// The paths that are mount points.
    pub mounted: BTreeSet<String>,
}

/// The kind a table row names.
fn kind_named(name: &str) -> EntryKind {
    return match name {
        "directory" => EntryKind::Directory,
        "file" => EntryKind::File,
        "link" => EntryKind::Link,
        "missing" => EntryKind::Missing,
        _ => EntryKind::Other,
    };
}

/// An object of text values,
/// as a map.
fn text_map(world: &Value, key: &str) -> BTreeMap<String, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    if let Some(Value::Object(entries)) = world.get(key) {
        for (name, value) in entries {
            map.insert(name.clone(), value.as_str().unwrap_or_default().to_string());
        }
    }
    return map;
}

impl World {
    /// What:
    ///  The world a table row describes.
    /// Why:
    ///  The same rows run in TypeScript.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// row.world
    /// ```
    pub(crate) fn from_json(world: &Value) -> World {
        let mut entries: BTreeMap<String, EntryKind> = BTreeMap::new();
        for (path, kind) in text_map(world, "entries") {
            entries.insert(path, kind_named(kind.as_str()));
        }
        let mut mounted: BTreeSet<String> = BTreeSet::new();
        if let Some(Value::Array(paths)) = world.get("mounted") {
            for path in paths {
                mounted.insert(path.as_str().unwrap_or_default().to_string());
            }
        }
        return World {
            entries,
            links: text_map(world, "links"),
            real: text_map(world, "real"),
            texts: text_map(world, "texts"),
            uuids: text_map(world, "uuids"),
            mounted,
        };
    }
}

/// The probe over a world,
/// with the name of every question asked.
#[derive(Debug, Default)]
pub(crate) struct FixtureProbe {
    /// The world.
    pub world: World,
    /// The name of each question asked,
    /// in order.
    pub asked: Mutex<Vec<&'static str>>,
}

impl FixtureProbe {
    /// A probe over a world.
    pub(crate) fn over(world: World) -> FixtureProbe {
        return FixtureProbe { world, asked: Mutex::new(Vec::new()) };
    }

    /// The names of the questions asked so far.
    pub(crate) fn questions(&self) -> Vec<&'static str> {
        return self.asked.lock().expect("question log").clone();
    }

    /// Records one question.
    fn note(&self, name: &'static str) {
        self.asked.lock().expect("question log").push(name);
    }
}

/// A path as the text the world is keyed by.
fn key(path: &Path) -> String {
    return path.to_string_lossy().into_owned();
}

impl LinkProbe for FixtureProbe {
    fn lstat(&self, path: &Path) -> EntryKind {
        self.note("lstat");
        return self.world.entries.get(&key(path)).copied().unwrap_or(EntryKind::Missing);
    }

    fn readlink(&self, path: &Path) -> String {
        self.note("readlink");
        return self.world.links.get(&key(path)).cloned().unwrap_or_default();
    }

    fn realpath(&self, path: &Path) -> Option<PathBuf> {
        self.note("realpath");
        return self.world.real.get(&key(path)).map(PathBuf::from);
    }

    fn read_text(&self, path: &Path) -> String {
        self.note("read_text");
        return self.world.texts.get(&key(path)).cloned().unwrap_or_default();
    }

    fn uuid_of(&self, path: &Path) -> String {
        self.note("uuid_of");
        return self.world.uuids.get(&key(path)).cloned().unwrap_or_default();
    }

    fn is_mount_point(&self, path: &Path) -> bool {
        self.note("is_mount_point");
        return self.world.mounted.contains(&key(path));
    }
}

//! In-memory Search index (D112): normalized names with first-character and two-character buckets.
//!
//! What: `SearchIndex` holds, for every track, its display path, its trimmed normalized title and that title's UTF-16
//! length, plus buckets that list the tracks having a word that starts with a given character or character pair, and a
//! small table of top-level folders. Tracks are only ever appended, so a track's position never changes.
//! Why: the matcher that this index replaces rebuilt the folder index and normalized every name on every keystroke,
//! which cost seconds at one million tracks. Here a keystroke reads one bucket, tests only its members, and keeps
//! the best five hundred. Because the index only grows, a cached answer plus the tracks appended since is exact.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class SearchIndex { paths: string[]; norm: string[]; byFirst = new Map<string, number[]>(); append(batch) {} search(q) {} }
//! ```

use std::collections::HashMap;

use crate::search_index_batch::PreparedBatch;
use crate::search_index_text::TextArena;
use crate::search_index_top::TopCache;
use crate::search_matcher::normalize_for_search;

/// What:
///      `pub(crate) const FOLDER_BIT: u32 = 1 << 31`.
///  Marks a ranked candidate as a folder.
/// Why:
///       A candidate is one `u32`; tracks use the low bits for their position and folders set this bit.
pub(crate) const FOLDER_BIT: u32 = 1 << 31;

/// What:
///      `pub(crate) const ROOT_FOLDER_SLOT: u32 = FOLDER_BIT - 1`.
///  The folder number of the root-level group when it is listed as a folder.
/// Why:
///       The group is not a key of the folder table, so it needs a number that no table slot reaches.
pub(crate) const ROOT_FOLDER_SLOT: u32 = FOLDER_BIT - 1;

/// What:
///      `pub(crate) const UNFILED: &str = "Unfiled"`.
///  The name of the root-level group, and of any real folder called the same.
/// Why:
///       The folder index uses this name for tracks without a top-level folder, and the Search folder rules
///  single it out.
pub(crate) const UNFILED: &str = "Unfiled";

/// What:
///      `pub(crate) struct FolderRecord`.
///  One top-level folder with its search form.
/// Why:
///       Folders are matched on every query, so their normalized names are kept ready.
#[derive(Debug)]
pub(crate) struct FolderRecord {
    /// Folder name as displayed.
    pub(crate) name: String,
    /// Trimmed normalized name.
    pub(crate) norm: String,
    /// UTF-16 length of `norm`.
    pub(crate) nlen: u32,
    /// Whether a track of this folder has a subfolder or a path that starts with `Unfiled/`.
    pub(crate) nested_or_prefixed: bool,
}

/// What:
///      `pub struct SearchIndex`.
///  The searchable form of the library.
/// Why:
///       See the module description: it is the structure a keystroke reads instead of the whole library.
#[derive(Debug)]
pub struct SearchIndex {
    /// Number of the library this index describes; a new library starts a new generation.
    generation: u64,
    /// Display path of each track, by track position.
    paths: TextArena,
    /// Trimmed normalized title of each track.
    norm: TextArena,
    /// UTF-16 length of each normalized title.
    nlen: Vec<u32>,
    /// Tracks having a word that starts with the character, in track order.
    by_first: HashMap<char, Vec<u32>>,
    /// Tracks having a word that starts with the character pair, in track order.
    by_pair: HashMap<(char, char), Vec<u32>>,
    /// Top-level folders in first-seen order.
    folders: Vec<FolderRecord>,
    /// Position of each folder name in `folders`.
    folder_slots: HashMap<String, usize>,
    /// Whether some track has fewer than two path segments.
    has_root_track: bool,
    /// Whether a root-level track starts with `Unfiled/`.
    root_prefixed: bool,
    /// The best-five-hundred answer of each single-character query.
    top: TopCache,
}

/// Methods of `SearchIndex`.
impl SearchIndex {
    /// What: `new` makes an empty index for library `generation`.
    /// Why: a rescan that cannot be expressed as an append starts over with a fresh generation.
    #[must_use]
    pub fn new(generation: u64) -> SearchIndex {
        return SearchIndex {
            generation,
            paths: TextArena::new(),
            norm: TextArena::new(),
            nlen: Vec::new(),
            by_first: HashMap::new(),
            by_pair: HashMap::new(),
            folders: Vec::new(),
            folder_slots: HashMap::new(),
            has_root_track: false,
            root_prefixed: false,
            top: TopCache::new(),
        };
    }

    /// What: `generation` is the library number given at creation.
    /// Why: callers that cache an answer compare it to know the library was replaced.
    #[must_use]
    pub fn generation(&self) -> u64 {
        return self.generation;
    }

    /// What: `set_generation` renumbers the library.
    /// Why: an index built on the side is numbered when it is installed.
    pub fn set_generation(&mut self, generation: u64) {
        self.generation = generation;
    }

    /// What: `track_count` is the number of tracks appended so far.
    /// Why: a track's position is its index in the library, and the count is the next position.
    #[must_use]
    pub fn track_count(&self) -> usize {
        return self.nlen.len();
    }

    /// What: `reserve` makes room for tracks and text that are about to be appended.
    /// Why: growing a buffer copies it, and a copy of a hundred megabytes under the write lock would stall a keystroke.
    pub fn reserve(&mut self, tracks: usize, path_bytes: usize, norm_bytes: usize) {
        self.paths.reserve(tracks, path_bytes);
        self.norm.reserve(tracks, norm_bytes);
        self.nlen.reserve(tracks);
    }

    /// What: `path` is the display path of track `position`, or an empty text past the end.
    /// Why: results show the title and the folder path of a track, which derive from this path.
    pub(crate) fn path(&self, position: usize) -> &str {
        return self.paths.get(position);
    }

    /// What: `norm` is the trimmed normalized title of track `position`.
    /// Why: matching tests this text.
    pub(crate) fn norm(&self, position: usize) -> &str {
        return self.norm.get(position);
    }

    /// What: `nlen` is the UTF-16 length of track `position`'s normalized title.
    /// Why: it is the second key of the result order.
    pub(crate) fn nlen(&self, position: usize) -> u32 {
        return self.nlen.get(position).copied().unwrap_or(0);
    }

    /// What: `bucket_first` lists the tracks having a word that starts with `character`.
    /// Why: a one-character term can match only these tracks.
    pub(crate) fn bucket_first(&self, character: char) -> &[u32] {
        return self.by_first.get(&character).map_or(&[], Vec::as_slice);
    }

    /// What: `bucket_pair` lists the tracks having a word that starts with the two characters.
    /// Why: a term of two or more characters can match only these tracks.
    pub(crate) fn bucket_pair(&self, pair: (char, char)) -> &[u32] {
        return self.by_pair.get(&pair).map_or(&[], Vec::as_slice);
    }

    /// What: `top` is the cache of single-character answers.
    /// Why: the query path reads and refreshes it.
    pub(crate) fn top(&self) -> &TopCache {
        return &self.top;
    }

    /// What: `first_characters` lists each first character that has a bucket, with the bucket's size.
    /// Why: the warm-up pass starts with the largest buckets.
    pub(crate) fn first_characters(&self) -> impl Iterator<Item = (char, usize)> + '_ {
        return self.by_first.iter().map(|(character, list)| return (*character, list.len()));
    }

    /// What: `folder` is the record of folder `slot`, or `None` for a slot that is not a table entry.
    /// Why: the query path reads names of folders by the slot a ranked candidate carries.
    pub(crate) fn folder(&self, slot: usize) -> Option<&FolderRecord> {
        return self.folders.get(slot);
    }

    /// What: `listed_folders` lists the slots of the folders that Search offers as results.
    /// Why: a real folder named `Unfiled` is offered only when it has a subfolder or an `Unfiled/` path, because
    /// otherwise it is indistinguishable from the root-level group, which is never offered.
    pub(crate) fn listed_folders(&self) -> impl Iterator<Item = usize> + '_ {
        return self
            .folders
            .iter()
            .enumerate()
            .filter(|(_, record)| return record.name != UNFILED || record.nested_or_prefixed)
            .map(|(slot, _)| return slot);
    }

    /// What: `root_group_listed` tells whether the root-level group is offered as a folder result.
    /// Why: the group is offered only when a root-level track starts with `Unfiled/`, matching the folder index rule.
    pub(crate) fn root_group_listed(&self) -> bool {
        return self.has_root_track && self.root_prefixed;
    }

    /// What: `append` adds a prepared batch after the tracks already known.
    /// Why: this is the push done under the write lock. It copies text, shifts offsets and extends bucket lists;
    /// all normalization was done by `prepare_batch`.
    ///
    /// # Panics
    /// When a library passes four billion tracks, which is far above the supported size.
    pub(crate) fn append(&mut self, batch: PreparedBatch) {
        let base: u32 = u32::try_from(self.nlen.len()).expect("a library holds under four billion tracks");
        self.paths.append(&batch.paths);
        self.norm.append(&batch.norm);
        self.nlen.extend_from_slice(&batch.nlen);
        let mut run_start: usize = 0;
        while run_start < batch.by_first.len() {
            let key: char = batch.by_first[run_start].0;
            let run_len: usize = batch.by_first[run_start..].partition_point(|entry| return entry.0 == key);
            let bucket: &mut Vec<u32> = self.by_first.entry(key).or_default();
            bucket.extend(batch.by_first[run_start..run_start + run_len].iter().map(|entry| return entry.1 + base));
            run_start += run_len;
        }
        run_start = 0;
        while run_start < batch.by_pair.len() {
            let key: (char, char) = batch.by_pair[run_start].0;
            let run_len: usize = batch.by_pair[run_start..].partition_point(|entry| return entry.0 == key);
            let bucket: &mut Vec<u32> = self.by_pair.entry(key).or_default();
            bucket.extend(batch.by_pair[run_start..run_start + run_len].iter().map(|entry| return entry.1 + base));
            run_start += run_len;
        }
        for sighting in batch.folders {
            match self.folder_slots.get(&sighting.name) {
                Some(slot) => self.folders[*slot].nested_or_prefixed |= sighting.nested_or_prefixed,
                None => {
                    let norm: String = crate::search_matcher::trim_kotlin_whitespace(&normalize_for_search(&sighting.name)).to_string();
                    let nlen: u32 = u32::try_from(norm.encode_utf16().count()).expect("a folder name is short");
                    self.folder_slots.insert(sighting.name.clone(), self.folders.len());
                    self.folders.push(FolderRecord {
                        name: sighting.name,
                        norm,
                        nlen,
                        nested_or_prefixed: sighting.nested_or_prefixed,
                    });
                }
            }
        }
        self.has_root_track |= batch.has_root_track;
        self.root_prefixed |= batch.root_prefixed;
    }

    /// What: `heap_bytes` estimates the memory the index holds.
    /// Why: the scale tests and the report state the cost per million tracks.
    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        let buckets: usize = self.by_first.values().map(|list| return list.capacity() * 4).sum::<usize>()
            + self.by_pair.values().map(|list| return list.capacity() * 4).sum::<usize>();
        return self.paths.byte_len() + self.norm.byte_len() + self.nlen.capacity() * 12 + buckets;
    }
}

/// Parity tests against the replaced matcher in `search_index_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "search_index_tests.rs"]
mod tests;

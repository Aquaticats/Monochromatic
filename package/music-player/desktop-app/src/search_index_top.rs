//! Best-five-hundred answer per first character, kept exact while the library grows.
//!
//! What: `TopCache` remembers, for each single-character query, the best tracks among the first `through` tracks and
//! how many tracks matched. Asking again merges that answer with the tracks appended since, and stores the merge.
//! Why: a one-character query matches a large share of the library (tens of percent for common letters), so ranking
//! its whole bucket costs tens of milliseconds at one million tracks. The library only grows, and the result order is
//! total, so the best five hundred of the first `through` tracks plus every match among the later ones is exactly the
//! best five hundred of all of them. The previous answer is therefore served immediately, never out of date, and
//! the cost of a keystroke during a scan is the size of the new tracks, not of the library.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merge = (cached: Top, delta: Ranked[]) => ({ through: now, matched: cached.matched + delta.length, best: best500([...cached.best, ...delta]) });
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::search_index::SearchIndex;
use crate::search_index_rank::{Ranked, fast_tier, select_best};

/// What:
///      `pub(crate) const TRACK_KEEP: usize = 500`.
///  How many best tracks one cached answer keeps.
/// Why:
///       A result list shows at most five hundred rows, and the whole answer is the best five hundred of the cached
///  tracks and the folders, so five hundred tracks are enough.
pub(crate) const TRACK_KEEP: usize = 500;

/// What:
///      `const REBUILD_ABOVE: usize = 200_000`.
///  The number of new matching tracks above which a cached answer is rebuilt instead of merged.
/// Why:
///       Merging ranks the new tracks only; for a delta as large as the library, ranking everything is the same work.
const REBUILD_ABOVE: usize = 200_000;

/// What:
///      `pub(crate) struct TopEntry { through, matched, best }`.
///  The exact answer of a single-character query over the first `through` tracks.
/// Why:
///       See the module description.
#[derive(Debug)]
pub(crate) struct TopEntry {
    /// Tracks at positions below this number are covered.
    through: usize,
    /// Number of covered tracks that match.
    matched: usize,
    /// The best `TRACK_KEEP` covered matches in result order.
    best: Vec<Ranked>,
}

/// What:
///      `pub(crate) struct TopCache`.
///  The cache of single-character answers, shared by threads.
/// Why:
///       The query worker and the warm-up pass both read and store entries; the lock guards only the map, never
///  a ranking.
#[derive(Debug, Default)]
pub(crate) struct TopCache {
    /// One entry per first character.
    entries: Mutex<HashMap<char, Arc<TopEntry>>>,
}

/// Methods of `TopCache`.
impl TopCache {
    /// What: `new` makes an empty cache.
    /// Why: a new library starts with no answers.
    pub(crate) fn new() -> TopCache {
        return TopCache::default();
    }

    /// What: `get` returns the entry of `character`, if any.
    /// Why: the entry is cloned out so ranking never holds the map lock.
    fn get(&self, character: char) -> Option<Arc<TopEntry>> {
        return match self.entries.lock() {
            Ok(map) => map.get(&character).cloned(),
            Err(poisoned) => poisoned.into_inner().get(&character).cloned(),
        };
    }

    /// What: `put` stores the entry of `character`.
    /// Why: the newest merge replaces the older one.
    fn put(&self, character: char, entry: Arc<TopEntry>) {
        match self.entries.lock() {
            Ok(mut map) => {
                map.insert(character, entry);
            }
            Err(poisoned) => {
                poisoned.into_inner().insert(character, entry);
            }
        }
    }

    /// What: `covered` is the number of tracks the entry of `character` covers, or `None` without an entry.
    /// Why: the warm-up pass refreshes entries that fall behind.
    pub(crate) fn covered(&self, character: char) -> Option<usize> {
        return self.get(character).map(|entry| return entry.through);
    }
}

/// What:
///      `fn rank_members(index: &SearchIndex, members: &[u32], terms: &[String], joined: &str) -> Vec<Ranked>`.
///  Tiers the bucket members that match.
/// Why:
///       Both a rebuild and a merge rank a list of bucket members the same way.
fn rank_members(index: &SearchIndex, members: &[u32], terms: &[String], joined: &str) -> Vec<Ranked> {
    let mut ranked: Vec<Ranked> = Vec::with_capacity(members.len());
    for member in members {
        if let Some(tier) = fast_tier(index.norm(*member as usize), terms, joined) {
            ranked.push(Ranked {
                tier,
                nlen: index.nlen(*member as usize),
                id: *member,
            });
        }
    }
    return ranked;
}

/// What:
///      `pub(crate) fn single_character_tracks(index: &SearchIndex, character: char) -> (Vec<Ranked>, usize)`.
///  The best tracks for a query that is one character, with the number of matching tracks.
/// Why:
///       Serves the cached answer merged with the tracks appended after it, and stores the merge, so the next
///  keystroke starts from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function singleCharacterTracks(index: SearchIndex, c: string): [Ranked[], number] { /* cached + delta */ }
/// ```
pub(crate) fn single_character_tracks(index: &SearchIndex, character: char) -> (Vec<Ranked>, usize) {
    let terms: Vec<String> = vec![character.to_string()];
    let joined: String = character.to_string();
    let bucket: &[u32] = index.bucket_first(character);
    let now: usize = index.track_count();
    let cached: Option<Arc<TopEntry>> = index.top().get(character);
    if let Some(entry) = &cached {
        let start: usize = bucket.partition_point(|member| return (*member as usize) < entry.through);
        let delta: &[u32] = &bucket[start..];
        if delta.is_empty() {
            return (entry.best.clone(), entry.matched);
        }
        if delta.len() <= REBUILD_ABOVE {
            let mut combined: Vec<Ranked> = entry.best.clone();
            let fresh: Vec<Ranked> = rank_members(index, delta, &terms, &joined);
            let matched: usize = entry.matched + fresh.len();
            combined.extend(fresh);
            let best: Vec<Ranked> = select_best(index, combined, TRACK_KEEP);
            index.top().put(
                character,
                Arc::new(TopEntry {
                    through: now,
                    matched,
                    best: best.clone(),
                }),
            );
            return (best, matched);
        }
    }
    let everything: Vec<Ranked> = rank_members(index, bucket, &terms, &joined);
    let matched: usize = everything.len();
    let best: Vec<Ranked> = select_best(index, everything, TRACK_KEEP);
    index.top().put(
        character,
        Arc::new(TopEntry {
            through: now,
            matched,
            best: best.clone(),
        }),
    );
    return (best, matched);
}

/// What:
///      `pub fn warm_characters(index: &SearchIndex) -> Vec<char>`.
///  The first characters whose cached answer is missing or behind the library, largest bucket first.
/// Why:
///       A background pass refreshes them one at a time, so the first keystroke of a common letter is a cache read.
#[must_use]
pub fn warm_characters(index: &SearchIndex) -> Vec<char> {
    let mut behind: Vec<(char, usize)> = index
        .first_characters()
        .filter(|(character, bucket_len)| {
            return *bucket_len > 0 && index.top().covered(*character).is_none_or(|through| return through < index.track_count());
        })
        .collect();
    behind.sort_by(|left, right| return right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    return behind.into_iter().map(|(character, _)| return character).collect();
}

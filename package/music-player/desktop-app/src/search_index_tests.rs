// What:     Parity tests for `SearchIndex`, pulled in by `#[cfg(test)] #[path = "search_index_tests.rs"] mod tests;` at
//           the bottom of `search_index.rs`. The replaced matcher, kept as `search_results::oracle`, answers every query
//           on the same library, and the two answers must be equal: tier, kind, name, library index, count, the
//           truncated flag and the highlight ranges. Libraries are fixed hostile lists and seeded random ones, queries
//           are fixed hostile texts and random ones, and every random library is also built from random batch splits
//           with queries asked between the splits, because Search must be exact while a scan appends.
// Why:      D112 moves Search off the old matcher; these tests are what keeps the visible behavior unchanged.
use super::*;
use crate::search_index_batch::prepare_batch;
use crate::search_results::oracle::oracle_search_library;
use crate::search_results::{SEARCH_RESULT_LIMIT, SearchResultKind, SearchState};
use crate::track::Track;

// What:     `Rng`, a seeded 64-bit xorshift generator.
// Why:      Random libraries and queries must be reproducible from the seed printed on failure.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        return self.0;
    }

    fn below(&mut self, bound: usize) -> usize {
        return usize::try_from(self.next() % (bound as u64)).unwrap_or(0);
    }

    fn pick<'a>(&mut self, items: &'a [&'a str]) -> &'a str {
        return items[self.below(items.len())];
    }
}

// What:     Fragments random names are built from.
// Why:      A small alphabet with accents, compatibility forms, astral characters, an ideographic space, separators and
//           the capital dotted I makes collisions, ties and every normalization branch common.
const PIECES: &[&str] = &[
    "a", "b", "c", "ab", "ac", "ba", "cam", "Cam", "Camellia", "live", "Live", "at", "e", "E", "é", "É", "e\u{0301}",
    "ö", "o\u{0308}", "İ", "ß", "ｃａｍ", "カ", "か", "が", "夜", "\u{1F3B5}", "x\u{1D11E}", "\u{3000}", " ", " ", "-",
    "_", ".", "(", "[", "  ", "1", "2", "10", "Ω", "ς", "Σ",
];

// What:     `random_name`, one display path built from the pieces.
// Why:      Depth 0 to 3 gives root files, one folder, subfolders, and empty, `.` and `..` segments.
fn random_name(rng: &mut Rng) -> String {
    let depth: usize = rng.below(4);
    let mut parts: Vec<String> = Vec::new();
    for _ in 0..depth {
        let segment: String = match rng.below(12) {
            0 => String::new(),
            1 => ".".to_string(),
            2 => "..".to_string(),
            3 => "Unfiled".to_string(),
            _ => (0..1 + rng.below(3)).map(|_| return rng.pick(PIECES)).collect(),
        };
        parts.push(segment);
    }
    let title: String = (0..1 + rng.below(4)).map(|_| return rng.pick(PIECES)).collect();
    let extension: &str = rng.pick(&[".flac", ".mp3", ".opus", "", ".", ".tar.gz"]);
    parts.push(format!("{title}{extension}"));
    return parts.join("/");
}

// What:     `random_query`, one query: a piece, two pieces, a prefix of a library name, or two terms.
// Why:      Queries must hit, miss, tie, cross separators and use several terms.
fn random_query(rng: &mut Rng, names: &[String]) -> String {
    return match rng.below(5) {
        0 => rng.pick(PIECES).to_string(),
        1 => format!("{}{}", rng.pick(PIECES), rng.pick(PIECES)),
        2 => {
            let name: &String = &names[rng.below(names.len())];
            let count: usize = 1 + rng.below(6);
            name.rsplit('/').next().unwrap_or("").chars().take(count).collect()
        }
        3 => format!("{} {}", rng.pick(PIECES), rng.pick(PIECES)),
        _ => rng.pick(&["a", "e", "c", "l", "ca", "li", "-", "(", " ", "x", "1"]).to_string(),
    };
}

// What:     `tracks_of`, the tracks the oracle searches.
// Why:      The oracle takes `Track` values; the index takes the display paths.
fn tracks_of(names: &[String]) -> Vec<Track> {
    return names
        .iter()
        .enumerate()
        .map(|(position, name)| return Track { uri: format!("u{position}"), display_path: name.clone() })
        .collect();
}

// What:     `assert_same`, compares the index answer with the oracle answer for one query.
// Why:      Every parity test ends here, and a failure names the query and the first difference.
fn assert_same(index: &SearchIndex, names: &[String], query: &str, context: &str) {
    let expected: SearchState = oracle_search_library(query, &tracks_of(names), true);
    let actual: SearchState = index.search(query);
    assert_eq!(actual, expected, "{context}: query {query:?} over {} names", names.len());
}

// What:     `index_of`, builds an index from names in one batch.
// Why:      Most cases need the whole library at once.
fn index_of(names: &[String]) -> SearchIndex {
    let mut index: SearchIndex = SearchIndex::new(1);
    index.append(prepare_batch(names));
    return index;
}

// What:     `strings`, owned names from literals.
// Why:      The fixed libraries are written as literals.
fn strings(literals: &[&str]) -> Vec<String> {
    return literals.iter().map(|literal| return literal.to_string()).collect();
}

// What:     The fixed hostile library.
// Why:      Each name covers a rule that decides whether a track or folder appears and how it is named.
fn hostile_library() -> Vec<String> {
    return strings(&[
        "Cam.flac",
        "Camellia/Camellia Waltz.opus",
        "Camellia/Live at Camellia.flac",
        "Camellia/Scamper.mp3",
        "Camellia/Sub Folder/Cam (remix).flac",
        "Another Xronixle/Camellia/Intro.flac",
        "./Unfiled/Dot Prefixed.flac",
        "Unfiled/Plain.flac",
        "Unfiled/Sub/Deep.flac",
        "Unfiled",
        "Unfiled/",
        "//Rooted/Track.flac",
        "../Parent/Track.flac",
        "a//b/c.flac",
        "Café/Ö.flac",
        "Cafe/O.flac",
        "Mötley Crüe/Dr. Feelgood.mp3",
        "Ｔｏｋｙｏ/Ｃａｍ.flac",
        "藍月/鎖の少女.flac",
        "藍月/カ.flac",
        "Music\u{1F3B5}/x\u{1D11E} note.flac",
        "Istanbul/İstanbul.flac",
        "Istanbul/istanbul.flac",
        "Greek/ΟΔΟΣ.flac",
        "Spaces/\u{3000}Ideographic.flac",
        "Spaces/ trailing .flac",
        ".flac",
        "Hidden/.flac",
        "Dots/..flac",
        "Dots/a.b.c.flac",
        "Dots/a.b.c",
        "Dash/x-cam.flac",
        "Dash/x_cam.flac",
        "Dash/x.cam.flac",
        "Dash/x(cam.flac",
        "Dash/x[cam.flac",
        "Dash/x/cam.flac",
    ]);
}

// What:     The fixed hostile queries.
// Why:      Separators, brackets, quotes, control characters, comparisons and very long text must not break matching.
const HOSTILE_QUERIES: &[&str] = &[
    "", " ", "   ", "\t\n", "\u{3000}", "c", "C", "ca", "cam", "CAM", "cam ", " cam", "cam live", "live cam", "cam cam", "at", "unf",
    "unfiled", "Unfiled/", "-", "--", "_", ".", "(", "[", "/", "x-cam", "x cam", "x.cam", "a-b", "é", "e\u{0301}", "ö", "o",
    "İ", "i", "istanbul", "ß", "ss", "ｃａｍ", "カ", "か", "が", "夜", "藍月", "鎖", "\u{1F3B5}", "x\u{1D11E}", "ΟΔΟΣ", "οδοσ", "ς",
    "dr.", "feelgood", "10", "'; DROP TABLE t;--", "\\", "\"", "%", "*", "?", "^", "$", "\u{0}", "\u{200B}", "cam\u{0301}",
    "zzzzzz", "a b c d e f g h i j k l m n o p q r s t u v w x y z", "ａ", "Ａ", "ｱ", "..", "...",
];

// What:     The fixed library answers every hostile query exactly as the oracle does.
// Why:      This is the baseline parity case with hand-picked names and queries.
#[test]
fn hostile_library_matches_the_oracle_for_hostile_queries() {
    let names: Vec<String> = hostile_library();
    let index: SearchIndex = index_of(&names);
    for query in HOSTILE_QUERIES {
        assert_same(&index, &names, query, "hostile library");
    }
}

// What:     A very long query and a query of many terms match the oracle.
// Why:      Query length must not change the answer or panic.
#[test]
fn long_queries_match_the_oracle() {
    let names: Vec<String> = hostile_library();
    let index: SearchIndex = index_of(&names);
    assert_same(&index, &names, &"cam ".repeat(500), "many terms");
    assert_same(&index, &names, &"a".repeat(5000), "one long term");
}

// What:     Seeded random libraries answer random queries as the oracle does.
// Why:      Random names reach combinations the fixed list does not, and the seed is printed on failure.
#[test]
fn random_libraries_match_the_oracle() {
    let (seeds, queries, base): (u64, usize, usize) = if cfg!(debug_assertions) { (2, 40, 250) } else { (6, 120, 400) };
    for seed in 1..=seeds {
        let mut rng: Rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let names: Vec<String> = (0..base + 150 * usize::try_from(seed).unwrap_or(0)).map(|_| return random_name(&mut rng)).collect();
        let index: SearchIndex = index_of(&names);
        for _ in 0..queries {
            let query: String = random_query(&mut rng, &names);
            assert_same(&index, &names, &query, &format!("seed {seed}"));
        }
    }
}

// What:     Tiering a bucket on several threads gives the answer of tiering it on one, which is the oracle's.
// Why:      The parallel scan only runs for buckets of 32,768 members, which the parity libraries never reach, so the
//           threshold is passed in: one forces every bucket onto several threads, the maximum keeps every bucket on one.
#[test]
fn the_parallel_scan_answers_as_the_sequential_scan_and_the_oracle() {
    let mut rng: Rng = Rng(0xA5A5_5A5A_1234_5679);
    let names: Vec<String> = (0..if cfg!(debug_assertions) { 600 } else { 2500 }).map(|_| return random_name(&mut rng)).collect();
    let index: SearchIndex = index_of(&names);
    let queries: usize = if cfg!(debug_assertions) { 40 } else { 150 };
    for _ in 0..queries {
        let query: String = random_query(&mut rng, &names);
        let parallel: SearchState = index.search_with(&query, 1);
        let sequential: SearchState = index.search_with(&query, usize::MAX);
        assert_eq!(parallel, sequential, "parallel and sequential differ for {query:?}");
        assert_eq!(parallel, oracle_search_library(&query, &tracks_of(&names), true), "parallel differs from the oracle for {query:?}");
    }
}

// What:     A library built from random batch splits answers exactly after every split, with single-character answers
//           cached before each append.
// Why:      D111 requires every track known at that moment to be searched; the cached answer plus the new tracks must
//           equal a fresh ranking, which is the merge the cache relies on.
#[test]
fn growing_library_stays_exact_between_batches() {
    let (last_seed, count): (u64, usize) = if cfg!(debug_assertions) { (11, 400) } else { (14, 900) };
    for seed in 11..=last_seed {
        let mut rng: Rng = Rng(seed.wrapping_mul(0xD6E8_FEB8_6659_FD93) | 1);
        let names: Vec<String> = (0..count).map(|_| return random_name(&mut rng)).collect();
        let mut index: SearchIndex = SearchIndex::new(1);
        let mut known: usize = 0;
        while known < names.len() {
            let size: usize = 1 + rng.below(120);
            let end: usize = (known + size).min(names.len());
            index.append(prepare_batch(&names[known..end]));
            known = end;
            let so_far: &[String] = &names[..known];
            for character in ["a", "e", "c", "l", "1", "é"] {
                assert_same(&index, so_far, character, &format!("seed {seed} after {known}"));
            }
            for _ in 0..8 {
                let query: String = random_query(&mut rng, so_far);
                assert_same(&index, so_far, &query, &format!("seed {seed} after {known}"));
            }
        }
    }
}

// What:     More matches than the cap produce a truncated list whose cut obeys the full order, including ties.
// Why:      Selection compares integer keys first and string keys only for ties; many equal-length names exercise the
//           tie path, and the truncated flag must stay true.
#[test]
fn many_equal_length_matches_are_cut_in_the_oracle_order() {
    let mut names: Vec<String> = Vec::new();
    for number in 0..1500_usize {
        let letter: char = char::from(b'a' + u8::try_from(number % 26).unwrap_or(0));
        names.push(format!("Artist {}/x{letter}{number:04}.flac", number % 7));
    }
    let index: SearchIndex = index_of(&names);
    for query in ["x", "xa", "xa00", "a", "artist", "artist 3", "x0"] {
        assert_same(&index, &names, query, "many ties");
    }
    match index.search("x") {
        SearchState::Results { items, truncated } => {
            assert_eq!(items.len(), SEARCH_RESULT_LIMIT);
            assert!(truncated);
        }
        other => panic!("expected results, got {other:?}"),
    }
}

// What:     A real folder named `Unfiled` appears only when it has a subfolder or an `Unfiled/` track, and the
//           root-level group appears only when a root track starts with `Unfiled/`; a later batch can change that.
// Why:      The folder index rule decides it from all rows together, so appends must re-evaluate it, never cache it.
#[test]
fn the_unfiled_folder_rule_follows_later_batches() {
    let mut index: SearchIndex = SearchIndex::new(1);
    let mut names: Vec<String> = strings(&["./Unfiled/Dot.flac", "Plain.flac"]);
    index.append(prepare_batch(&names));
    assert_same(&index, &names, "unfiled", "dot-prefixed Unfiled only");
    let more: Vec<String> = strings(&["Unfiled/Real.flac"]);
    index.append(prepare_batch(&more));
    names.extend(more);
    assert_same(&index, &names, "unfiled", "after an Unfiled/ track");
    let rooted: Vec<String> = strings(&["Unfiled/"]);
    index.append(prepare_batch(&rooted));
    names.extend(rooted);
    assert_same(&index, &names, "unfiled", "after a root track starting with Unfiled/");
}

// What:     The index counts what it holds and an empty index reports an empty inventory.
// Why:      The page distinguishes an empty query, an empty inventory and a no-match.
#[test]
fn an_empty_index_has_an_empty_inventory() {
    let index: SearchIndex = SearchIndex::new(3);
    assert_eq!(index.generation(), 3);
    assert_eq!(index.track_count(), 0);
    assert_eq!(index.search("cam"), SearchState::EmptyInventory);
    assert_eq!(index.search("  "), SearchState::EmptyQuery);
}

// What:     A folder result sorts before a track result that is equal in every other key.
// Why:      The relevance order breaks the last ties with the kind, then the position.
#[test]
fn a_folder_precedes_an_equal_track() {
    let names: Vec<String> = strings(&["Cam/Other.flac", "Cam.flac"]);
    let index: SearchIndex = index_of(&names);
    match index.search("cam") {
        SearchState::Results { items, .. } => {
            assert_eq!(items[0].kind, SearchResultKind::Folder);
            assert_eq!(items[1].kind, SearchResultKind::Track);
        }
        other => panic!("expected results, got {other:?}"),
    }
}

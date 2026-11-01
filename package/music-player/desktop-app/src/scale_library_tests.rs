//! Deterministic synthetic libraries for the scale tests, shared by the library and the application test builds.
//!
//! What: `build_library` makes the relative names and absolute paths of `artists` folders of `tracks_per_artist` tracks,
//! with the name shapes of the scale generator script (scripts, subfolders, loose root files, long titles).
//! Why: the Search index, the page index and the window projection are all measured at 1,000, 10,000, 100,000 and
//! 1,000,000 tracks (D112), and each measurement must see the same names. The file uses only `std`, so both the library
//! crate and the application crate compile it as a test module.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lib = buildLibrary(1000, 1000); // 1,000 artists, 1,000,000 names
//! ```

use std::collections::HashSet;
use std::path::PathBuf;

/// What: the percentage of library tracks placed as loose files at the root.
/// Why: root files group by first letter into their own pages, which the picker must also bound.
const LOOSE_PERCENT: usize = 3;
/// What: the generator seed shared by every size.
/// Why: the same seed gives the same names in every run, so timings and counts are reproducible.
const SEED: u64 = 1;
/// What: the longest allowed path component, in bytes.
/// Why: this is the filesystem limit the generator must respect, so names never exceed it.
const MAX_COMPONENT_BYTES: usize = 255;
/// What: the longest allowed track title, in characters.
/// Why: a few titles are long on purpose, to test long-name layout and projection costs.
const MAX_TITLE_CHARS: usize = 200;

/// What: `Rng` is a 64-bit xorshift generator.
/// Why: a fixed seed gives the same names on every run, with no dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Rng { private state: bigint; next(): bigint { /* xorshift64 */ } }
/// ```
struct Rng {
    state: u64,
}

/// Methods of `Rng`.
impl Rng {
    /// What: `stream` builds the generator for one seed, index and salt.
    /// Why: each artist and its track list draw from their own stream, so an artist's content does not change with size.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stream = (seed: bigint, index: bigint, salt: bigint) => new Rng(seed ^ index * K1 ^ salt * K2 | 1n);
    /// ```
    fn stream(index: u64, salt: u64) -> Rng {
        let mixed = SEED ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93);
        return Rng { state: mixed | 1 };
    }

    /// What: `next_u64` advances the generator and returns its next value.
    /// Why: xorshift64 is cheap and has a full period, which is enough for name choices.
    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        return value;
    }

    /// What: `below` gives a number from zero up to `bound`, exclusive.
    /// Why: name parts, word counts and track counts all need bounded choices.
    fn below(&mut self, bound: usize) -> usize {
        return usize::try_from(self.next_u64() % (bound as u64)).unwrap_or(0);
    }

    /// What: `chance` is true with the given whole-number percentage.
    /// Why: subfolders, long titles and kana words are minorities with fixed rates.
    fn chance(&mut self, percent: usize) -> bool {
        return self.below(100) < percent;
    }

    /// What: `pick` gives one item of a fixed table.
    /// Why: syllable and symbol tables are fixed lists chosen at random.
    fn pick(&mut self, items: &[&'static str]) -> &'static str {
        return items[self.below(items.len())];
    }
}

/// What: `Script` names the writing system an artist folder's name is written in.
/// Why: the picker's letter rail needs Latin, kana, Cyrillic, digit and symbol sections, so names are drawn per script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Script = "latin" | "japanese" | "cyrillic" | "digit" | "symbol" | "mixed";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    /// Latin syllables, the most common case.
    Latin,
    /// Kana with a few Han characters.
    Japanese,
    /// Cyrillic syllables.
    Cyrillic,
    /// A number followed by a Latin phrase.
    Digit,
    /// A symbol prefix followed by a Latin phrase.
    Symbol,
    /// A Latin phrase followed by a Japanese phrase.
    Mixed,
}

/// What: `script_for` draws the writing system for one artist, with the same weights as the generator script.
/// Why: the mix keeps every rail section populated at every size.
fn script_for(rng: &mut Rng) -> Script {
    return match rng.below(100) {
        0..62 => Script::Latin,
        62..74 => Script::Japanese,
        74..82 => Script::Cyrillic,
        82..91 => Script::Digit,
        91..95 => Script::Symbol,
        _ => Script::Mixed,
    };
}

const LATIN_SYLLABLES: &[&str] = &["ka", "lo", "ri", "ven", "sa", "mo", "tek", "zu", "ash", "lind", "ora", "quin", "bel", "dom", "fe", "yr"];
const KANA: &[&str] = &["あ", "い", "か", "き", "さ", "し", "た", "な", "ア", "カ", "サ", "タ"];
const HAN: &[&str] = &["夜", "光", "風", "海", "空", "月"];
const CYRILLIC: &[&str] = &["ма", "ра", "лу", "ну", "ко", "ти", "шо", "зе", "ри", "бо"];
const SYMBOLS: &[&str] = &["!!!", "#", "$", "&", "+", "~", "(", "[", "*"];

/// What: `capitalise` upper-cases the first character of a word.
/// Why: artist and title words start with a capital, as real folder names usually do.
fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    return match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    };
}

/// What: `latin_word` builds one Latin word of two or three syllables.
/// Why: a shared syllable pool makes words repeat across artists, as real names do.
fn latin_word(rng: &mut Rng) -> String {
    let syllables = 2 + rng.below(2);
    let mut word = String::new();
    for _ in 0..syllables {
        word.push_str(rng.pick(LATIN_SYLLABLES));
    }
    return capitalise(&word);
}

/// What: `latin_phrase` joins one to three Latin words.
/// Why: most folder names are one to three words.
fn latin_phrase(rng: &mut Rng) -> String {
    let words = 1 + rng.below(3);
    let mut parts: Vec<String> = Vec::with_capacity(words);
    for _ in 0..words {
        parts.push(latin_word(rng));
    }
    return parts.join(" ");
}

/// What: `japanese_phrase` builds a run of two to five kana, sometimes with a Han character.
/// Why: the kana and Han sections of the rail need real kana and Han starts.
fn japanese_phrase(rng: &mut Rng) -> String {
    let length = 2 + rng.below(4);
    let mut phrase = String::new();
    for _ in 0..length {
        phrase.push_str(rng.pick(KANA));
    }
    if rng.chance(30) {
        phrase.push_str(rng.pick(HAN));
    }
    return phrase;
}

/// What: `cyrillic_phrase` builds one to three Cyrillic words of two to four syllables.
/// Why: the library has no Cyrillic folders in its seed, so the rail needs synthetic Cyrillic names.
fn cyrillic_phrase(rng: &mut Rng) -> String {
    let words = 1 + rng.below(3);
    let mut parts: Vec<String> = Vec::with_capacity(words);
    for _ in 0..words {
        let syllables = 2 + rng.below(3);
        let mut word = String::new();
        for _ in 0..syllables {
            word.push_str(rng.pick(CYRILLIC));
        }
        parts.push(word);
    }
    return parts.join(" ");
}

/// What: `artist_name` builds the folder name of artist `index` in the script it draws.
/// Why: the name depends only on the index, so the first artists are the same at every size.
fn artist_name(index: usize) -> String {
    let mut rng = Rng::stream(index as u64, 1);
    return match script_for(&mut rng) {
        Script::Latin => latin_phrase(&mut rng),
        Script::Japanese => japanese_phrase(&mut rng),
        Script::Cyrillic => cyrillic_phrase(&mut rng),
        Script::Digit => format!("{} {}", 1 + rng.below(999), latin_phrase(&mut rng)),
        Script::Symbol => format!("{} {}", rng.pick(SYMBOLS), latin_phrase(&mut rng)),
        Script::Mixed => format!("{} {}", latin_phrase(&mut rng), japanese_phrase(&mut rng)),
    };
}

/// What: `track_title` builds a title of one to five words, or thirty to forty-nine words for about two in a hundred.
/// Why: most titles are short, a few are very long, and a shared pool makes titles repeat across artists.
fn track_title(rng: &mut Rng) -> String {
    let words = if rng.chance(2) { 30 + rng.below(20) } else { 1 + rng.below(5) };
    let mut parts: Vec<String> = Vec::with_capacity(words);
    for _ in 0..words {
        if rng.chance(15) {
            parts.push(japanese_phrase(rng));
        } else {
            parts.push(latin_word(rng));
        }
    }
    return parts.join(" ").chars().take(MAX_TITLE_CHARS).collect();
}

/// What: `extension_for` picks an audio extension with the seed library's weights (opus, flac, m4a, mp3).
/// Why: the real library is mostly opus and flac, so the mix keeps extension handling honest.
fn extension_for(rng: &mut Rng) -> &'static str {
    return match rng.below(100) {
        0..60 => "opus",
        60..96 => "flac",
        96..99 => "m4a",
        _ => "mp3",
    };
}

/// What: `file_name` builds one track file name, with a numeric prefix and a byte limit on the whole component.
/// Why: the prefix keeps names unique inside a folder, and the byte limit keeps each component within the filesystem cap.
fn file_name(prefix: &str, title: &str, extension: &str) -> String {
    let stem = format!("{prefix} {title}");
    let budget = MAX_COMPONENT_BYTES - extension.len() - 1;
    let mut kept = String::new();
    for character in stem.chars() {
        if kept.len() + character.len_utf8() > budget {
            break;
        }
        kept.push(character);
    }
    return format!("{kept}.{extension}");
}

/// What: `album_directory` gives the album and CD folder of one track in an artist that has subfolders.
/// Why: subfolders show the folder rail and the track list's nested headers, as real albums do.
fn album_directory(folder: &str, track: usize, count: usize) -> String {
    let album = 1 + (track * 12) / count.max(1);
    let mut directory = format!("{folder}/Album {album:02}");
    if album % 2 == 0 {
        directory = format!("{directory}/CD {}", 1 + (track % 2));
    }
    return directory;
}

/// What: `SyntheticLibrary` is the in-memory library: relative names and absolute paths, in queue order.
/// Why: the projection and the Search index take these shapes, and building them once keeps each scale test's setup the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SyntheticLibrary = { artists: number; names: string[]; paths: string[]; tracks: Track[] };
/// ```
pub(crate) struct SyntheticLibrary {
    /// Number of artist folders, not counting the root.
    pub(crate) artists: usize,
    /// Relative names in queue order, loose root files first.
    pub(crate) names: Vec<String>,
}

/// Methods of `SyntheticLibrary`.
impl SyntheticLibrary {
    /// What: `paths` builds the absolute paths matching `names` by index.
    /// Why: only the window tests need them, and building a million paths takes noticeable time.
    pub(crate) fn paths(&self) -> Vec<PathBuf> {
        return self.names.iter().map(|name| return PathBuf::from(format!("/library/{name}"))).collect();
    }
}

/// What: `build_library` builds the library for `artists` folders of `tracks_per_artist` tracks each.
/// Why: a fixed name plan keeps artist folder names unique and track names unique inside each folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const buildLibrary = (artists: number, tracksPerArtist: number): SyntheticLibrary => { ... };
/// ```
fn generate_library(artists: usize, tracks_per_artist: usize) -> SyntheticLibrary {
    let total = artists * tracks_per_artist;
    let loose = total * LOOSE_PERCENT / 100;
    let artist_tracks = total - loose;
    let mut names: Vec<String> = Vec::with_capacity(total);
    let mut root_rng = Rng::stream(0, 2);
    for number in 0..loose {
        let extension = extension_for(&mut root_rng);
        let title = track_title(&mut root_rng);
        names.push(file_name(&format!("Loose {} -", number + 1), &title, extension));
    }
    let mut taken: HashSet<String> = HashSet::with_capacity(artists);
    for index in 0..artists {
        let folder = unique_folder(&mut taken, index);
        let count = artist_tracks / artists + usize::from(index < artist_tracks % artists);
        let mut rng = Rng::stream(index as u64, 3);
        let subfolders = rng.chance(10);
        for track in 0..count {
            let extension = extension_for(&mut rng);
            let title = track_title(&mut rng);
            let prefix = format!("{:0width$}", track + 1, width = count.to_string().len());
            let directory = if subfolders { album_directory(&folder, track, count) } else { folder.clone() };
            names.push(format!("{directory}/{}", file_name(&prefix, &title, extension)));
        }
    }
    if names.len() <= REPLICATE_ABOVE {
        let unique: HashSet<&String> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "generated names must be unique");
    }
    return SyntheticLibrary { artists, names };
}

/// What: the library size above which a debug build replicates a generated base instead of generating every name.
/// Why: generating a million names with the full name generator takes about ten seconds without optimization, which
/// is too long for the normal suite; a release build generates all of them in about a second.
const REPLICATE_ABOVE: usize = 100_000;

/// What: `build_library` makes the library of `artists` folders of `tracks_per_artist` tracks.
/// Why: a debug build replicates a generated hundred-artist base to reach larger sizes, giving every copy its own
/// folder names; a release build generates every name. Both give unique names of the same shapes.
pub(crate) fn build_library(artists: usize, tracks_per_artist: usize) -> SyntheticLibrary {
    let total = artists * tracks_per_artist;
    if !cfg!(debug_assertions) || total <= REPLICATE_ABOVE {
        return generate_library(artists, tracks_per_artist);
    }
    let base_artists = REPLICATE_ABOVE / tracks_per_artist.max(1);
    let base = generate_library(base_artists, tracks_per_artist);
    let copies = artists.div_ceil(base_artists);
    let mut names: Vec<String> = Vec::with_capacity(base.names.len() * copies);
    for copy in 0..copies {
        for name in &base.names {
            if copy == 0 {
                names.push(name.clone());
            } else {
                names.push(match name.split_once('/') {
                    Some((folder, rest)) => format!("{folder} {copy}/{rest}"),
                    None => format!("c{copy} {name}"),
                });
            }
        }
    }
    return SyntheticLibrary { artists, names };
}

/// What: `unique_folder` gives artist `index` a folder name no earlier artist has.
/// Why: two artists with one folder name would merge into one folder, and the track count would shrink.
fn unique_folder(taken: &mut HashSet<String>, index: usize) -> String {
    let drawn = artist_name(index);
    let base = if drawn.trim().is_empty() { format!("artist {index}") } else { drawn };
    let mut candidate = base.clone();
    let mut counter = 2;
    while taken.contains(&candidate) {
        candidate = format!("{base} {counter}");
        counter += 1;
    }
    taken.insert(candidate.clone());
    return candidate;
}

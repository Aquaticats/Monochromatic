//! Rail model for the folder picker:
//!  one section per writing system present in the library.
//!
//! Ported from `core/FolderRail.kt`.
//!  The JavaScript rail is `design/candidates/buckets.js`,
//!  and
//! its `scriptOf` helper is in `design/candidates/artists.js`.
//!  Only writing systems present in
//! the library get a section.
//!  Every cell is keyed by the first character of a folder name,
//!  with
//! no sub-letter segmentation.

/// Hash map used to group rail entries by key.
use std::collections::HashMap;

/// Adds the Unicode normalization iterators to strings.
use unicode_normalization::UnicodeNormalization;
/// Tests whether a character is a combining mark.
use unicode_normalization::char::is_combining_mark;

/// What:
///      `const KANJI_LABEL: &str = "漢";`.
///  The cell for Japanese names whose first
///  character sits in no kana row.
/// Why:
///       Kanji and other unlisted Japanese characters share one cell,
///  as the JS model does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KANJI_LABEL = "漢";
/// ```
const KANJI_LABEL: &str = "漢";

/// What:
///      `const KANA_ROWS: [(&str, &str); 10]`.
///  Each gojuon row label paired with the
///  hiragana and katakana characters it covers,
///  including small,
///  voiced and prolonged forms.
/// Why:
///       Japanese names are bucketed by row rather than by character,
///  exactly as the JS table does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KANA_ROWS: [string, string][] = [["あ", "あいうえおアイウエオぁぃぅぇぉァィゥェォ"], /* ... */];
/// ```
const KANA_ROWS: [(&str, &str); 10] = [
    ("あ", "あいうえおアイウエオぁぃぅぇぉァィゥェォ"),
    ("か", "かきくけこカキクケコがぎぐげごガギグゲゴ"),
    ("さ", "さしすせそサシスセソざじずぜぞザジズゼゾ"),
    ("た", "たちつてとタチツテトだぢづでどダヂヅデドっッ"),
    ("な", "なにぬねのナニヌネノ"),
    ("は", "はひふへほハヒフヘホばびぶべぼバビブベボぱぴぷぺぽパピプペポ"),
    ("ま", "まみむめもマミムメモ"),
    ("や", "やゆよヤユヨゃゅょャュョ"),
    ("ら", "らりるれろラリルレロ"),
    ("わ", "わをんワヲンー"),
];

/// What:
///      `pub enum RailScript`.
///  The six writing systems that can own a section,
///  in the order the rail shows them.
/// Why:
///       The declaration order is the section order,
///  and `id` prefixes each cell key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RailScript = "latin" | "jpn" | "cyrl" | "grek" | "hang" | "other";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailScript {
    /// Latin letters,
    ///  bucketed by their uppercased first character.
    Latin,
    /// Kana rows plus one kanji cell.
    Japanese,
    /// Cyrillic letters,
    ///  bucketed by their uppercased first character.
    Cyrillic,
    /// Greek letters,
    ///  bucketed by their uppercased first character.
    Greek,
    /// Hangul syllables,
    ///  with one cell per first syllable.
    Hangul,
    /// Digits,
    ///  symbols,
    ///  and every script the rail does not model.
    Other,
}

/// What:
///      `const ALL_SCRIPTS: [RailScript; 6]`,
///  the sections in rail order.
/// Why:
///       The rail walks this list,
///  so absent writing systems are skipped in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALL_SCRIPTS = ["latin", "jpn", "cyrl", "grek", "hang", "other"] as const;
/// ```
const ALL_SCRIPTS: [RailScript; 6] = [
    RailScript::Latin,
    RailScript::Japanese,
    RailScript::Cyrillic,
    RailScript::Greek,
    RailScript::Hangul,
    RailScript::Other,
];

/// Methods of `RailScript`.
impl RailScript {
    /// What:
    ///      `pub fn id(self) -> &'static str`.
    ///  The stable identifier used as the
    ///  prefix of every cell key.
    /// Why:
    ///       Cell keys must not change when a label glyph changes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const id = (script: RailScript) => script;
    /// ```
    pub fn id(self) -> &'static str {
        return match self {
            RailScript::Latin => "latin",
            RailScript::Japanese => "jpn",
            RailScript::Cyrillic => "cyrl",
            RailScript::Greek => "grek",
            RailScript::Hangul => "hang",
            RailScript::Other => "other",
        };
    }

    /// What:
    ///      `pub fn label(self) -> &'static str`.
    ///  The single glyph shown as the section label.
    /// Why:
    ///       Each label uses the script's own letter,
    ///  so the rail reads naturally.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const SCRIPT_LABEL = { latin: "A", jpn: "あ", cyrl: "А", grek: "Α", hang: "가", other: "#" };
    /// ```
    pub fn label(self) -> &'static str {
        return match self {
            RailScript::Latin => "A",
            RailScript::Japanese => "あ",
            RailScript::Cyrillic => "А",
            RailScript::Greek => "Α",
            RailScript::Hangul => "가",
            RailScript::Other => "#",
        };
    }
}

/// What:
///      `pub struct RailCell { key, label, names }`.
///  One rail button and the folder
///  names it leads to,
///  in input order.
/// Why:
///       The picker needs a stable key,
///  a glyph to show,
///  and the names to list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RailCell = { key: string; label: string; names: string[] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RailCell {
    /// Unique key made from the script id and the label.
    pub key: String,
    /// Glyph shown on the rail button.
    pub label: String,
    /// Folder names in library order that fall into this cell.
    pub names: Vec<String>,
}

/// What:
///      `pub struct RailSection { script, label, cells }`.
///  One writing-system section of the rail.
/// Why:
///       Sections are separated by a hairline,
///  so each carries its own label and cells.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RailSection = { script: RailScript; label: string; cells: RailCell[] };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RailSection {
    /// Writing system this section represents.
    pub script: RailScript,
    /// Glyph shown as the section label.
    pub label: String,
    /// Cells of this section,
    ///  sorted by collation key of their label.
    pub cells: Vec<RailCell>,
}

/// What:
///      `pub fn collation_key(text: &str) -> String`.
///  Lowercases,
///  decomposes to NFD,
///  then drops the combining marks.
/// Why:
///       Stands in for the Kotlin `Collator` at PRIMARY strength with canonical decomposition.
///  Accented letters then compare equal to their base letters,
///  so `Ё` sorts with `Е`.
///  Lowercasing runs first so that a character such as `İ` loses its mark too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const collationKey = (s: string) => s.toLowerCase().normalize("NFD").replace(/\p{M}/gu, "");
/// ```
pub fn collation_key(text: &str) -> String {
    let lowered = text.to_lowercase();
    return lowered
        .nfd()
        .filter(|character| return !is_combining_mark(*character))
        .collect();
}

/// What:
///      `pub fn script_of(name: &str) -> RailScript`.
///  Picks the writing system from
///  the first character.
///  An empty name is OTHER.
/// Why:
///       Each name must land in exactly one section,
///  and the empty string has no first character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scriptOf(name: string): string { /* first code point range checks */ }
/// ```
pub fn script_of(name: &str) -> RailScript {
    return match name.chars().next() {
        Some(first) => script_of_char(first),
        None => RailScript::Other,
    };
}

/// What:
///      `fn script_of_char(first: char) -> RailScript`.
///  The range table that `script_of` uses.
/// Why:
///       The ranges mirror the JS `scriptOf` and the Kotlin `isLatin` and `isJapanese` helpers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLatin = (c: string) => /[A-Za-zÀ-ɏ]/.test(c);
/// ```
fn script_of_char(first: char) -> RailScript {
    if first.is_ascii_alphabetic() || ('À'..='ɏ').contains(&first) {
        return RailScript::Latin;
    }
    if ('぀'..='ヿ').contains(&first) || ('一'..='鿿').contains(&first) || ('ｦ'..='ﾝ').contains(&first) {
        return RailScript::Japanese;
    }
    if ('Ѐ'..='ӿ').contains(&first) {
        return RailScript::Cyrillic;
    }
    if ('Ͱ'..='Ͽ').contains(&first) {
        return RailScript::Greek;
    }
    if ('가'..='힯').contains(&first) {
        return RailScript::Hangul;
    }
    return RailScript::Other;
}

/// What:
///      `fn kana_row_label_for(name: &str) -> Option<&'static str>`.
///  The kana row whose
///  characters include the name's first character,
///  or `None`.
/// Why:
///       `None` lets the caller fall back to the kanji cell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kanaRowLabelFor = (name: string) => KANA_ROWS.find(([, chars]) => chars.includes(name[0]))?.[0];
/// ```
fn kana_row_label_for(name: &str) -> Option<&'static str> {
    let first = name.chars().next()?;
    return KANA_ROWS
        .iter()
        .find(|row| return row.1.contains(first))
        .map(|row| return row.0);
}

/// What:
///      `fn uppercase_first(name: &str) -> String`.
///  The first character,
///  uppercased with
///  Unicode rules,
///  so `ß` becomes `SS`.
/// Why:
///       Latin,
///  Cyrillic,
///  Greek and Hangul cells use the uppercased first character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const upper = (name: string) => name[0].toUpperCase();
/// ```
fn uppercase_first(name: &str) -> String {
    return match name.chars().next() {
        Some(first) => first.to_uppercase().collect(),
        None => unreachable!("a name in a letter script always has a first character"),
    };
}

/// What:
///      `fn cell_label_for(name: &str, script: RailScript) -> String`.
///  Chooses the cell label for one name.
/// Why:
///       Each writing system buckets names differently,
///  and this is the only place that decides it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cellLabelFor(name: string, script: string): string {
///   if (script === "jpn") return kanaRowLabelFor(name) ?? "漢";
///   if (script === "other") return "#";
///   return name[0].toUpperCase();
/// }
/// ```
fn cell_label_for(name: &str, script: RailScript) -> String {
    return match script {
        RailScript::Latin | RailScript::Cyrillic | RailScript::Greek | RailScript::Hangul => uppercase_first(name),
        RailScript::Japanese => kana_row_label_for(name).unwrap_or(KANJI_LABEL).to_string(),
        RailScript::Other => script.label().to_string(),
    };
}

/// What:
///      `fn rail_section(script: RailScript, names: Vec<String>) -> RailSection`.
///  Groups one
///  writing system's names into cells,
///  then sorts the cells by collation key.
/// Why:
///       Cells are built per section so each section sorts only its own labels.
///  The sort is
///  stable,
///  so labels whose keys tie keep the order in which they first appeared,
///  as the
///  Kotlin `sortedWith` on the collator does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // group by label in first-seen order, then sort labels by collationKey (stable)
/// ```
fn rail_section(script: RailScript, names: Vec<String>) -> RailSection {
    let mut positions: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for name in names {
        let label = cell_label_for(&name, script);
        match positions.get(&label) {
            Some(&position) => groups[position].1.push(name),
            None => {
                positions.insert(label.clone(), groups.len());
                groups.push((label, vec![name]));
            }
        }
    }
    groups.sort_by_key(|group| return collation_key(&group.0));
    let cells: Vec<RailCell> = groups
        .into_iter()
        .map(|(label, members)| {
            let key = format!("{}{label}", script.id());
            return RailCell { key, label, names: members };
        })
        .collect();
    return RailSection {
        script,
        label: script.label().to_string(),
        cells,
    };
}

/// What:
///      `pub fn rail_for<S: AsRef<str>>(names: &[S]) -> Vec<RailSection>`.
///  Builds the folder
///  picker rail:
///  one section per writing system present,
///  in the fixed rail order.
/// Why:
///       The rail adapts to the library,
///  so writing systems with no names get no section.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function railFor(names: string[]): RailSection[] { /* see buckets.js */ }
/// ```
pub fn rail_for<S: AsRef<str>>(names: &[S]) -> Vec<RailSection> {
    let mut sections: Vec<RailSection> = Vec::new();
    for script in ALL_SCRIPTS {
        let members: Vec<String> = names
            .iter()
            .map(|name| return name.as_ref().to_string())
            .filter(|member| return script_of(member) == script)
            .collect();
        if members.is_empty() {
            continue;
        }
        sections.push(rail_section(script, members));
    }
    return sections;
}

/// What:
///      `#[cfg(test)] #[path = "folder_rail_tests.rs"] mod tests;` declares a test-only submodule
///  whose code lives in the sibling file `folder_rail_tests.rs`.
/// Why:
///       Keep `folder_rail.rs` to production code,
///  with the tests beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // folderRail.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "folder_rail_tests.rs"]
mod tests;

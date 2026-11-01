// Rail model for the folder picker. It ports the JavaScript rail in design/candidates/buckets.js and its
// helper scriptOf in design/candidates/artists.js. Only writing systems present in the library get a
// section (D28). Every cell is keyed by the first character of a folder name, with no sub-letter
// segmentation (D3 and D17).

// What:     `package dev.monochromatic.musicplayer.core` places this model beside the other pure logic.
// Why:      The rail needs no Android types, so it can be unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import java.text.Collator` and `import java.util.Locale` bring in the locale-aware comparator.
// Why:      `Collator` exists on both the JVM and Android, so the sort matches the JS localeCompare.
//
// In TS you'd write (pseudocode):
// ```ts
// // Intl.Collator is the closest TS equivalent.
// ```
import java.text.Collator
import java.util.Locale

// What:     `enum class RailScript(val id: String, val label: String)` lists the six writing systems
//           that can own a section, in the order the rail shows them.
// Why:      The declaration order is the section order, and each `id` prefixes that script's cell keys.
//
// In TS you'd write (pseudocode):
// ```ts
// type RailScript = "latin" | "jpn" | "cyrl" | "grek" | "hang" | "other";
// ```
/** Writing systems that can receive a section in the folder picker rail. */
enum class RailScript(
    /** Stable script identifier, used as the prefix of every cell key. */
    val id: String,
    /** Single glyph shown as the section label in the rail. */
    val label: String,
) {
    // What:     `LATIN("latin", "A")` is the Latin section, labelled with the letter A.
    // Why:      Latin names are the most common case and sort first in the rail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const LATIN = { id: "latin", label: "A" };
    // ```
    /** Latin letters, bucketed by their uppercased first character. */
    LATIN("latin", "A"),

    // What:     `JAPANESE("jpn", "あ")` is the Japanese section, labelled with a kana.
    // Why:      Kana rows and the kanji cell share one section.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const JAPANESE = { id: "jpn", label: "あ" };
    // ```
    /** Kana rows plus one kanji cell for names that begin with a character outside the kana rows. */
    JAPANESE("jpn", "あ"),

    // What:     `CYRILLIC("cyrl", "А")` is the Cyrillic section, labelled with the Cyrillic capital A.
    // Why:      The label uses U+0410 so the rail shows the script's own letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const CYRILLIC = { id: "cyrl", label: "А" };
    // ```
    /** Cyrillic letters, bucketed by their uppercased first character. */
    CYRILLIC("cyrl", "А"),

    // What:     `GREEK("grek", "Α")` is the Greek section, labelled with the Greek capital alpha.
    // Why:      The label uses U+0391 so the rail shows the script's own letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const GREEK = { id: "grek", label: "Α" };
    // ```
    /** Greek letters, bucketed by their uppercased first character. */
    GREEK("grek", "Α"),

    // What:     `HANGUL("hang", "가")` is the Hangul section, labelled with the syllable ga.
    // Why:      The label uses U+AC00 so the rail shows a Hangul syllable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const HANGUL = { id: "hang", label: "가" };
    // ```
    /** Hangul syllables, with one cell per first syllable. */
    HANGUL("hang", "가"),

    // What:     `OTHER("other", "#")` is the catch-all section for digits, symbols, and unsupported scripts.
    // Why:      Every name lands in some section, so no folder is hidden from the rail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const OTHER = { id: "other", label: "#" };
    // ```
    /** Digits, symbols, and every script the rail does not model, under one `#` cell. */
    OTHER("other", "#"),
}

// What:     `data class RailCell(val key: String, val label: String, val names: List<String>)` is one rail
//           button and the folder names it leads to.
// Why:      The picker needs a stable key for each button, a glyph to show, and the names to list.
//
// In TS you'd write (pseudocode):
// ```ts
// type RailCell = { key: string; label: string; names: string[] };
// ```
/** One rail button and the folder names it leads to. */
data class RailCell(
    /** Unique key made from the script id and the label. */
    val key: String,
    /** Glyph shown on the rail button. */
    val label: String,
    /** Folder names in library order that fall into this cell. */
    val names: List<String>,
)

// What:     `data class RailSection(val script: RailScript, val label: String, val cells: List<RailCell>)`
//           is one writing-system section of the rail.
// Why:      Sections are separated by a hairline in the rail, so each carries its own label and cells.
//
// In TS you'd write (pseudocode):
// ```ts
// type RailSection = { script: RailScript; label: string; cells: RailCell[] };
// ```
/** One writing-system section of the rail, with its cells sorted by label. */
data class RailSection(
    /** Writing system this section represents. */
    val script: RailScript,
    /** Glyph shown as the section label. */
    val label: String,
    /** Cells of this section, sorted by label. */
    val cells: List<RailCell>,
)

// What:     `private val KANA_ROWS` maps each kana row label to the hiragana and katakana characters it covers.
//           Small forms, voiced forms, and the prolonged sound mark are included in their row.
// Why:      Japanese names are bucketed by row rather than by character, exactly as the JS table does.
//
// In TS you'd write (pseudocode):
// ```ts
// const KANA_ROWS: [string, string][] = [["あ", "あいうえおアイウエオぁぃぅぇぉァィゥェォ"], /* ... */];
// ```
/** Kana rows in gojuon order, each paired with the characters that belong to it. */
private val KANA_ROWS: List<Pair<String, String>> = listOf(
    "あ" to "あいうえおアイウエオぁぃぅぇぉァィゥェォ",
    "か" to "かきくけこカキクケコがぎぐげごガギグゲゴ",
    "さ" to "さしすせそサシスセソざじずぜぞザジズゼゾ",
    "た" to "たちつてとタチツテトだぢづでどダヂヅデドっッ",
    "な" to "なにぬねのナニヌネノ",
    "は" to "はひふへほハヒフヘホばびぶべぼバビブベボぱぴぷぺぽパピプペポ",
    "ま" to "まみむめもマミムメモ",
    "や" to "やゆよヤユヨゃゅょャュョ",
    "ら" to "らりるれろラリルレロ",
    "わ" to "わをんワヲンー",
)

// What:     `private const val KANJI_LABEL: String = "漢"` is the cell for Japanese names that start
//           outside the kana rows.
// Why:      Kanji and other unlisted Japanese characters share one cell, as the JS model does.
//
// In TS you'd write (pseudocode):
// ```ts
// const KANJI_LABEL = "漢";
// ```
/** Cell label for Japanese names whose first character is not in a kana row. */
private const val KANJI_LABEL: String = "漢"

// What:     `private fun isLatin(first: Char): Boolean` tests whether a first character is in a Latin range.
// Why:      The ranges mirror the JS scriptOf exactly, including the accented Latin block.
//
// In TS you'd write (pseudocode):
// ```ts
// const isLatin = (c: string) => /[A-Za-zÀ-ɏ]/.test(c);
// ```
/** Reports whether a first character belongs to the ASCII letters or the accented Latin block. */
private fun isLatin(first: Char): Boolean =
    first in 'A'..'Z' || first in 'a'..'z' || first in 'À'..'ɏ'

// What:     `private fun isJapanese(first: Char): Boolean` tests kana, CJK ideographs, and halfwidth katakana.
// Why:      The JS scriptOf treats these three ranges as one Japanese script.
//
// In TS you'd write (pseudocode):
// ```ts
// const isJapanese = (c: string) => /[぀-ヿ一-鿿ｦ-ﾝ]/.test(c);
// ```
/** Reports whether a first character is kana, a CJK ideograph, or halfwidth katakana. */
private fun isJapanese(first: Char): Boolean =
    first in '぀'..'ヿ' || first in '一'..'鿿' || first in 'ｦ'..'ﾝ'

// What:     `internal fun scriptOf(name: String): RailScript` picks the writing system from the first character.
//           An empty name returns OTHER.
// Why:      Each name must land in exactly one section, and the empty string has no first character.
//
// In TS you'd write (pseudocode):
// ```ts
// function scriptOf(name: string): RailScript { /* first-character range checks */ }
// ```
/** Maps a folder name to the writing system of its first character. */
internal fun scriptOf(name: String): RailScript {
    if (name.isEmpty()) {
        return RailScript.OTHER
    }
    /** First UTF-16 unit of the name, the character the rail buckets by. */
    val first = name[0]
    return when {
        isLatin(first) -> RailScript.LATIN
        isJapanese(first) -> RailScript.JAPANESE
        first in 'Ѐ'..'ӿ' -> RailScript.CYRILLIC
        first in 'Ͱ'..'Ͽ' -> RailScript.GREEK
        first in '가'..'힯' -> RailScript.HANGUL
        else -> RailScript.OTHER
    }
}

// What:     `private fun kanaRowLabelFor(name: String): String?` returns the kana row label whose characters
//           include the name's first character, or null when no row contains it.
// Why:      A null result lets the caller fall back to the kanji cell.
//
// In TS you'd write (pseudocode):
// ```ts
// function kanaRowLabelFor(name: string): string | undefined {
//   return KANA_ROWS.find(([, chars]) => chars.includes(name[0]))?.[0];
// }
// ```
/** Finds the kana row label whose characters contain the name's first character. */
private fun kanaRowLabelFor(name: String): String? {
    /** First character of the name, matched against each kana row. */
    val first = name[0].toString()
    return KANA_ROWS.firstOrNull { row -> row.second.contains(first) }?.first
}

// What:     `private fun cellLabelFor(name: String, script: RailScript): String` picks the cell label for one name.
//           Latin, Cyrillic, Greek, and Hangul use the uppercased first character, Japanese uses its kana row
//           or the kanji cell, and OTHER uses `#`.
// Why:      Each writing system buckets names differently, and this is the only place that decides it.
//
// In TS you'd write (pseudocode):
// ```ts
// function cellLabelFor(name: string, script: RailScript): string {
//   if (script === "jpn") return kanaRowLabelFor(name) ?? "漢";
//   if (script === "other") return "#";
//   return name[0].toUpperCase();
// }
// ```
/** Chooses the rail cell label for one folder name inside its writing-system section. */
private fun cellLabelFor(name: String, script: RailScript): String = when (script) {
    RailScript.LATIN -> (foldedLatinInitial(name[0]) ?: name[0]).uppercase()
    RailScript.CYRILLIC, RailScript.GREEK, RailScript.HANGUL -> name[0].uppercase()
    RailScript.JAPANESE -> kanaRowLabelFor(name) ?: KANJI_LABEL
    RailScript.OTHER -> script.label
}

// What:     `internal fun railCollator(): Collator` returns a fresh PRIMARY-strength collator for the root locale.
//           Canonical decomposition makes an accented letter such as `Ё` sort with its base letter `Е`.
// Why:      The JS localeCompare with base sensitivity treats accented letters as equal to their base letters,
//           and `java.text.Collator` only does that once decomposition is on. Collators are not thread-safe,
//           so each caller gets its own instance.
//
// In TS you'd write (pseudocode):
// ```ts
// const railCollator = new Intl.Collator(undefined, { sensitivity: "base" });
// ```
/** Creates the case-insensitive, accent-insensitive collator shared by the rail and the folder index. */
internal fun railCollator(): Collator =
    Collator.getInstance(Locale.ROOT).apply {
        strength = Collator.PRIMARY
        decomposition = Collator.CANONICAL_DECOMPOSITION
    }

// What:     `private fun railSection(script: RailScript, names: List<String>): RailSection` groups the names of
//           one writing system into cells and sorts the cells by label.
// Why:      Cells are built per section so each section only sorts its own labels. The sort is stable, so
//           labels that compare equal keep the order in which they first appeared.
//
// In TS you'd write (pseudocode):
// ```ts
// function railSection(script: RailScript, names: string[]): RailSection { /* group, then sort labels */ }
// ```
/** Builds one section, with its cells grouped by label and sorted by the rail collator. */
private fun railSection(script: RailScript, names: List<String>): RailSection {
    /** Names of this section grouped by cell label, in first-seen label order. */
    val namesByLabel: Map<String, List<String>> = names.groupBy { name -> cellLabelFor(name, script) }
    /** Cell labels sorted by the rail collator, keeping first-seen order for ties. */
    val labels: List<String> = namesByLabel.keys.sortedWith(railCollator())
    /** Cells of this section, one per sorted label, each listing its names in input order. */
    val cells: List<RailCell> = labels.map { label ->
        RailCell(script.id + label, label, namesByLabel.getValue(label))
    }
    return RailSection(script, script.label, cells)
}

// What:     `fun railFor(names: List<String>): List<RailSection>` builds the rail for a list of folder names.
//           It returns one section per writing system present, in the enum order latin, jpn, cyrl, grek,
//           hang, other. An empty input gives an empty list.
// Why:      The rail adapts to the library (D28), so absent writing systems get no section at all.
//
// In TS you'd write (pseudocode):
// ```ts
// export function railFor(names: string[]): RailSection[] { /* see buckets.js */ }
// ```
/** Builds the folder picker rail for a list of folder names, one section per writing system present. */
fun railFor(names: List<String>): List<RailSection> {
    /** Names grouped by writing system, used to skip absent systems. */
    val namesByScript: Map<RailScript, List<String>> = names.groupBy { name -> scriptOf(name) }
    return RailScript.entries
        .filter { script -> namesByScript.containsKey(script) }
        .map { script -> railSection(script, namesByScript.getValue(script)) }
}

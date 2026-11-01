// Rail model cases, with expected values taken from running design/candidates/buckets.js on the same inputs.

// What:     `package ...core` places these tests beside the rail model they exercise.
// Why:      The tests reach `railFor` and `scriptOf` through the same package without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit's value assertion and test annotation register each rail case.
// Why:      Each case compares the Kotlin rail with the output the JS rail produced for the same names.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Test

/** Verifies the folder picker rail against the JS model for every writing system and edge case. */
class FolderRailTest {
    // What:     `private fun cellLabelOf(name: String): String` returns the rail cell label of one name.
    // Why:      A one-name library has exactly one section and one cell, so the label reads directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cellLabelOf = (name: string) => railFor([name])[0].cells[0].label;
    // ```
    /** Returns the cell label that a single folder name lands in. */
    private fun cellLabelOf(name: String): String = railFor(listOf(name)).single().cells.single().label

    // What:     `private fun singleSection(names: List<String>): RailSection` returns the only section of a rail.
    // Why:      Most cases use one writing system, so this states that expectation and fails loudly otherwise.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const singleSection = (names: string[]) => railFor(names)[0];
    // ```
    /** Returns the only section a rail built from these names produces. */
    private fun singleSection(names: List<String>): RailSection = railFor(names).single()

    // What:     `@Test fun latinOnlyGivesOneSectionWithAtoZCellsInOrder()` checks a Latin-only library.
    // Why:      The JS model shows a single Latin section with one cell per letter, sorted A to Z.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("latin only gives one section with A to Z cells", () => { /* ... */ });
    // ```
    /** Verifies one Latin section whose cells run A to Z regardless of input order. */
    @Test
    fun latinOnlyGivesOneSectionWithAtoZCellsInOrder() {
        /** One name per letter, supplied in reverse order. */
        val names = ('A'..'Z').reversed().map { letter -> "${letter}x" }
        /** The rail built from the reversed names. */
        val section = singleSection(names)
        assertEquals(RailScript.LATIN, section.script)
        assertEquals("A", section.label)
        assertEquals(('A'..'Z').map { letter -> letter.toString() }, section.cells.map { cell -> cell.label })
        assertEquals(listOf("Ax"), section.cells.first().names)
    }

    // What:     `@Test fun japaneseOnlyOpensOnKanaCell()` checks a Japanese-only library.
    // Why:      The JS model opens a Japanese-only rail on the kana cell, with the kanji cell last.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("japanese only opens on kana", () => { /* ... */ });
    // ```
    /** Verifies a Japanese-only library opens on the kana cell and places kanji last. */
    @Test
    fun japaneseOnlyOpensOnKanaCell() {
        /** The rail built from one hiragana, one katakana, and one kanji name. */
        val section = singleSection(listOf("漢字", "あいさつ", "カラオケ"))
        assertEquals(RailScript.JAPANESE, section.script)
        assertEquals("あ", section.label)
        assertEquals(listOf("あ", "か", "漢"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("あいさつ"), section.cells[0].names)
        assertEquals(listOf("カラオケ"), section.cells[1].names)
        assertEquals(listOf("漢字"), section.cells[2].names)
    }

    // What:     `@Test fun mixedLatinAndJapaneseShowsLatinSectionFirst()` checks a bilingual library.
    // Why:      The JS model lists the Latin section before the Japanese section whatever the input order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("mixed latin and japanese shows latin first", () => { /* ... */ });
    // ```
    /** Verifies the Latin section comes first in a library that also has Japanese names. */
    @Test
    fun mixedLatinAndJapaneseShowsLatinSectionFirst() {
        /** Sections of a library whose Japanese name is listed first. */
        val sections = railFor(listOf("あいさつ", "Camellia"))
        assertEquals(listOf(RailScript.LATIN, RailScript.JAPANESE), sections.map { section -> section.script })
        assertEquals(listOf("A", "あ"), sections.map { section -> section.label })
    }

    // What:     `@Test fun kanaRowsIncludeSmallAndVoicedForms()` checks that each kana row catches its variants.
    // Why:      Small kana, voiced kana, and the prolonged sound mark all belong to their row in the JS table.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("kana rows include small and voiced forms", () => { /* ... */ });
    // ```
    /** Verifies small kana, voiced kana, and the prolonged sound mark map to their kana rows. */
    @Test
    fun kanaRowsIncludeSmallAndVoicedForms() {
        /** Pairs of folder name and the kana row label the JS model assigns it. */
        val expected: List<Pair<String, String>> = listOf(
            "ぁい" to "あ",
            "がっこう" to "か",
            "ギター" to "か",
            "ざっし" to "さ",
            "だいこん" to "た",
            "っ" to "た",
            "ぱん" to "は",
            "ばん" to "は",
            "ゃく" to "や",
            "ー" to "わ",
            "ン" to "わ",
            "らくだ" to "ら",
            "まど" to "ま",
            "なつ" to "な",
        )
        expected.forEach { (name, label) ->
            assertEquals(label, cellLabelOf(name))
        }
    }

    // What:     `@Test fun kanaCellsSortInGojuonOrderBeforeKanji()` checks the order of all kana rows and kanji.
    // Why:      The JS model sorts kana rows in gojuon order and places the kanji cell after them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("kana cells sort in gojuon order before kanji", () => { /* ... */ });
    // ```
    /** Verifies one name per kana row and one kanji name produce cells in gojuon order with kanji last. */
    @Test
    fun kanaCellsSortInGojuonOrderBeforeKanji() {
        /** One name per row, supplied out of order, plus one kanji name. */
        val names = listOf("わたし", "らくだ", "やさい", "まど", "なつ", "はる", "さくら", "たぬき", "かさ", "あさ", "漢字")
        assertEquals(
            listOf("あ", "か", "さ", "た", "な", "は", "ま", "や", "ら", "わ", "漢"),
            singleSection(names).cells.map { cell -> cell.label },
        )
    }

    // What:     `@Test fun kanjiFallsUnderKanjiCell()` checks that ideographs map to the 漢 cell.
    // Why:      The JS model collects every ideograph outside the kana rows into one kanji cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("kanji falls under the kanji cell", () => { /* ... */ });
    // ```
    /** Verifies that names starting with an ideograph land in the 漢 cell. */
    @Test
    fun kanjiFallsUnderKanjiCell() {
        assertEquals("漢", cellLabelOf("漢字"))
        assertEquals("漢", cellLabelOf("中国"))
    }

    // What:     `@Test fun katakanaOutsideKanaRowsFallsToKanjiCell()` checks katakana that no row lists.
    // Why:      The JS model puts katakana missing from KANA_ROWS, such as ヴ and halfwidth forms, in the kanji cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("katakanaOutsideKanaRowsFallsToKanjiCell", () => { /* ... */ });
    // ```
    /** Verifies unlisted katakana and halfwidth katakana follow the JS model into the kanji cell. */
    @Test
    fun katakanaOutsideKanaRowsFallsToKanjiCell() {
        /** Section built from one katakana name with ヴ and one halfwidth name. */
        val section = singleSection(listOf("ヴィオラ", "ｱｲｳ"))
        assertEquals(listOf("漢"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("ヴィオラ", "ｱｲｳ"), section.cells.single().names)
    }

    // What:     `@Test fun cyrillicCellsSortByUppercasedLetter()` checks the Cyrillic section.
    // Why:      The JS model labels the section with А, uppercases each first letter, and sorts Ё after Б.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("cyrillic cells sort by uppercased letter", () => { /* ... */ });
    // ```
    /** Verifies the Cyrillic section groups by uppercased first letter with Ё sorted after Б. */
    @Test
    fun cyrillicCellsSortByUppercasedLetter() {
        /** Cyrillic section built from names whose Ё cell gathers both cases. */
        val section = singleSection(listOf("Бор", "Ёж", "Аня", "ёлка"))
        assertEquals(RailScript.CYRILLIC, section.script)
        assertEquals("А", section.label)
        assertEquals(listOf("А", "Б", "Ё"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("Ёж", "ёлка"), section.cells[2].names)
    }

    // What:     `@Test fun greekCellsSortByUppercasedLetter()` checks the Greek section.
    // Why:      The JS model labels the section with Α and groups lowercase and uppercase alpha together.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("greek cells sort by uppercased letter", () => { /* ... */ });
    // ```
    /** Verifies the Greek section groups names by uppercased first letter. */
    @Test
    fun greekCellsSortByUppercasedLetter() {
        /** Greek section built from one name per letter with a lowercase alpha name. */
        val section = singleSection(listOf("Γάμμα", "Αλφα", "αβ"))
        assertEquals(RailScript.GREEK, section.script)
        assertEquals("Α", section.label)
        assertEquals(listOf("Α", "Γ"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("Αλφα", "αβ"), section.cells[0].names)
    }

    // What:     `@Test fun hangulCellsKeyOnePerFirstSyllable()` checks the Hangul section.
    // Why:      The JS model gives each first syllable its own cell, not one cell per block.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("hangul cells key one per first syllable", () => { /* ... */ });
    // ```
    /** Verifies each Hangul first syllable gets its own cell in the Hangul section. */
    @Test
    fun hangulCellsKeyOnePerFirstSyllable() {
        /** Hangul section built from names that share and differ in their first syllable. */
        val section = singleSection(listOf("한국", "가나", "한두"))
        assertEquals(RailScript.HANGUL, section.script)
        assertEquals("가", section.label)
        assertEquals(listOf("가", "한"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("한국", "한두"), section.cells[1].names)
    }

    // What:     `@Test fun digitsAndSymbolsFallToOtherHashCell()` checks digits and symbols.
    // Why:      The JS model sends every name without a known writing system to the `#` cell of OTHER.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("digits and symbols fall to the other hash cell", () => { /* ... */ });
    // ```
    /** Verifies digit-led and symbol-led names land in the `#` cell of the OTHER section. */
    @Test
    fun digitsAndSymbolsFallToOtherHashCell() {
        /** Section built from a digit-led name and a symbol-led name. */
        val section = singleSection(listOf("123", "!wow"))
        assertEquals(RailScript.OTHER, section.script)
        assertEquals("#", section.label)
        assertEquals(listOf("#"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("123", "!wow"), section.cells.single().names)
    }

    // What:     `@Test fun emptyNameFallsToOtherHashCell()` checks the empty string.
    // Why:      The JS model reads the empty name as `other` and never reads a first character from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty name falls to the other hash cell", () => { /* ... */ });
    // ```
    /** Verifies the empty name lands in the `#` cell of OTHER without a crash. */
    @Test
    fun emptyNameFallsToOtherHashCell() {
        /** Section built from the single empty name. */
        val section = singleSection(listOf(""))
        assertEquals(RailScript.OTHER, section.script)
        assertEquals(listOf(""), section.cells.single().names)
    }

    // What:     `@Test fun emptyLibraryGivesNoSections()` checks an empty input list.
    // Why:      The JS model returns no sections for no names, so the picker shows no rail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty library gives no sections", () => { expect(railFor([])).toEqual([]); });
    // ```
    /** Verifies an empty list of names yields an empty rail. */
    @Test
    fun emptyLibraryGivesNoSections() {
        assertEquals(emptyList<RailSection>(), railFor(emptyList()))
    }

    // What:     `@Test fun caseInsensitiveGroupingPutsCapsuleAndCamelliaUnderC()` checks case folding.
    // Why:      The JS model groups by the uppercased first letter, keeping the input order inside the cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("capsule and Camellia both fall under C", () => { /* ... */ });
    // ```
    /** Verifies lowercase and uppercase first letters share one cell and keep their input order. */
    @Test
    fun caseInsensitiveGroupingPutsCapsuleAndCamelliaUnderC() {
        /** Single Latin cell holding both names. */
        val section = singleSection(listOf("capsule", "Camellia"))
        assertEquals(listOf("C"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("capsule", "Camellia"), section.cells.single().names)
    }

    // What:     `@Test fun accentedLatinStaysUnderItsBaseLetter()` checks an accent after the first letter.
    // Why:      The JS model reads only the first character, so an accent later in the name changes nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("accented latin stays under its base letter", () => { /* ... */ });
    // ```
    /** Verifies `Cö shu Nie` lands under the C cell. */
    @Test
    fun accentedLatinStaysUnderItsBaseLetter() {
        assertEquals("C", cellLabelOf("Cö shu Nie"))
    }

    // What:     `@Test fun accentedLeadingLetterKeepsItsOwnCellAsTheJsModelDoes()` checks an accented first letter.
    // Why:      The JS model does not fold accents when bucketing, so `Örjan` gets its own cell. Equal
    //           primary keys keep first-seen order, so Ö comes before O here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("accented leading letter keeps its own cell", () => { /* ... */ });
    // ```
    /** Verifies an accented leading letter gets its own cell and ties keep first-seen order. */
    @Test
    fun accentedLeadingLetterSharesItsBaseLetterCell() {
        /** Latin section built from an accented leading name followed by a plain one. */
        val section = singleSection(listOf("Örjan", "Oscar"))
        assertEquals(listOf("O"), section.cells.map { cell -> cell.label })
        assertEquals(listOf("Örjan", "Oscar"), section.cells[0].names)
    }

    /** Verifies É, Ó and Ü sit with E, O and U (D17, D28), and letters without a decomposition keep their own cell. */
    @Test
    fun accentedInitialsFoldIntoTheirBaseLetterCells() {
        assertEquals("E", cellLabelOf("Émilie Simon"))
        assertEquals("E", cellLabelOf("élan"))
        assertEquals("U", cellLabelOf("Über"))
        assertEquals("N", cellLabelOf("ñandú"))
        assertEquals("Ø", cellLabelOf("Øystein"))
    }

    // What:     `@Test fun eszettUppercasesToSsLikeTheJsModel()` checks the German sharp s.
    // Why:      Uppercasing `ß` gives `SS` in both JS and Kotlin's `Char.uppercase()`, so the cell label matches.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("eszett uppercases to SS", () => { /* ... */ });
    // ```
    /** Verifies a leading `ß` produces the two-letter `SS` cell. */
    @Test
    fun eszettUppercasesToSsLikeTheJsModel() {
        assertEquals("SS", cellLabelOf("ßx"))
    }

    // What:     `@Test fun sectionsFollowLatinJapaneseCyrillicGreekHangulOther()` checks section order.
    // Why:      The JS model lists sections in the fixed order latin, jpn, cyrl, grek, hang, other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("sections follow the fixed script order", () => { /* ... */ });
    // ```
    /** Verifies every writing system appears once, in the fixed rail order. */
    @Test
    fun sectionsFollowLatinJapaneseCyrillicGreekHangulOther() {
        /** Sections of a library that has one name in each writing system, listed in reverse. */
        val sections = railFor(listOf("123", "한국", "Αλφα", "Бор", "中国", "あ", "Zed"))
        assertEquals(
            listOf(
                RailScript.LATIN,
                RailScript.JAPANESE,
                RailScript.CYRILLIC,
                RailScript.GREEK,
                RailScript.HANGUL,
                RailScript.OTHER,
            ),
            sections.map { section -> section.script },
        )
        assertEquals(listOf("A", "あ", "А", "Α", "가", "#"), sections.map { section -> section.label })
    }
}

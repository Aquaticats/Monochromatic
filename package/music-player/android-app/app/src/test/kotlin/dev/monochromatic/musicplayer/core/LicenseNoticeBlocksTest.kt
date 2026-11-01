// What:     `package dev.monochromatic.musicplayer.core` places the tests beside the splitter they check.
// Why:      `splitLicenseNotices` and `LicenseBlock` live in this package, so the tests reach them without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Rule of equal signs, as wide as the notice file draws it. */
private val EQUALS_RULE: String = "=".repeat(78)

/** Rule of hyphens, as wide as the notice file draws it. */
private val HYPHEN_RULE: String = "-".repeat(78)

/** Line of prose long enough to count as hard-wrapped. */
private val WRAPPED_LINE: String = "The quick invented fox jumps over the lazy invented dog and keeps running far"

/** Tests for how the notice text is cut into the blocks of the Licenses page. */
class LicenseNoticeBlocksTest {
    /** An empty text has no block. */
    @Test
    fun emptyTextHasNoBlock() {
        assertEquals(emptyList<LicenseBlock>(), splitLicenseNotices(""))
        assertEquals(emptyList<LicenseBlock>(), splitLicenseNotices("\n\n"))
    }

    /** A title between two equal-sign rules is a section heading, in file order with the text after it. */
    @Test
    fun aTitleBetweenTwoRulesIsASection() {
        /** Notice text with one section and one short paragraph. */
        val notices = "$EQUALS_RULE\nAndroid libraries (2 artifacts)\n$EQUALS_RULE\n\nfoo:bar:1.0\nfoo:baz:2.0\n"
        assertEquals(
            listOf(
                LicenseBlock(LicenseBlockKind.SECTION, "Android libraries (2 artifacts)"),
                LicenseBlock(LicenseBlockKind.LINES, "foo:bar:1.0\nfoo:baz:2.0"),
            ),
            splitLicenseNotices(notices),
        )
    }

    /** A title above a hyphen rule is a subsection heading. */
    @Test
    fun aTitleAboveAHyphenRuleIsASubsection() {
        /** Notice text with a subsection that follows a paragraph without a blank line between them. */
        val notices = "Intro line\nUnder Apache-2.0: 2 artifacts\n$HYPHEN_RULE\n\nfoo:bar:1.0"
        assertEquals(
            listOf(
                LicenseBlock(LicenseBlockKind.LINES, "Intro line"),
                LicenseBlock(LicenseBlockKind.SUBSECTION, "Under Apache-2.0: 2 artifacts"),
                LicenseBlock(LicenseBlockKind.LINES, "foo:bar:1.0"),
            ),
            splitLicenseNotices(notices),
        )
    }

    /** A text without a final newline keeps its last line. */
    @Test
    fun theLastLineSurvivesWithoutATrailingNewline() {
        assertEquals(listOf(LicenseBlock(LicenseBlockKind.LINES, "last line")), splitLicenseNotices("last line"))
    }

    /** Windows line ends split the same way as Unix ones. */
    @Test
    fun carriageReturnsAreIgnored() {
        /** Notice text with Windows line ends. */
        val notices = "$EQUALS_RULE\r\nTitle\r\n$EQUALS_RULE\r\nbody\r\n"
        assertEquals(
            listOf(LicenseBlock(LicenseBlockKind.SECTION, "Title"), LicenseBlock(LicenseBlockKind.LINES, "body")),
            splitLicenseNotices(notices),
        )
    }

    /** Wrapped prose is joined into one paragraph without the indentation it was cut with. */
    @Test
    fun wrappedProseIsJoined() {
        /** Notice text with a paragraph of three hard-wrapped lines, the last one short. */
        val notices = "  $WRAPPED_LINE\n$WRAPPED_LINE\nshort end."
        assertEquals(
            listOf(LicenseBlock(LicenseBlockKind.PROSE, "$WRAPPED_LINE $WRAPPED_LINE short end.")),
            splitLicenseNotices(notices),
        )
    }

    /** Short lines, such as a centered title, keep their line breaks and their indentation relative to each other. */
    @Test
    fun shortLinesKeepTheirBreaks() {
        assertEquals(
            listOf(LicenseBlock(LicenseBlockKind.LINES, "     Apache License\nVersion 2.0, January 2004")),
            splitLicenseNotices("      Apache License\n Version 2.0, January 2004"),
        )
    }

    /** The indentation that all lines share is dropped, so a centered title starts at the left edge. */
    @Test
    fun theSharedIndentationIsDropped() {
        assertEquals(
            listOf(LicenseBlock(LicenseBlockKind.LINES, "TITLE\n  Sub title")),
            splitLicenseNotices("                   TITLE\n                     Sub title"),
        )
    }

    /** A very long line, such as the 952 character line of the real file, stays whole in one block. */
    @Test
    fun aVeryLongLineStaysWhole() {
        /** One line of 952 characters. */
        val longLine: String = "x".repeat(952)
        assertEquals(listOf(LicenseBlock(LicenseBlockKind.LINES, longLine)), splitLicenseNotices(longLine))
    }

    /** Two rules in a row make no heading and no block. */
    @Test
    fun consecutiveRulesMakeNoHeading() {
        assertEquals(
            listOf(LicenseBlock(LicenseBlockKind.LINES, "after")),
            splitLicenseNotices("$EQUALS_RULE\n$EQUALS_RULE\n$HYPHEN_RULE\nafter"),
        )
    }

    /** A rule of another width inside a license text is plain text, not a heading. */
    @Test
    fun aShorterRuleIsNotAHeading() {
        /** Equal signs that are not as wide as the file's rules. */
        val shortRule: String = "=".repeat(73)
        /** Notice text with a title framed by short rules. */
        val notices = "$shortRule\nNot a heading\n$shortRule"
        assertTrue(splitLicenseNotices(notices).none { block -> block.kind == LicenseBlockKind.SECTION })
    }

    /** A paragraph of many lines without a blank line is cut into blocks of at most the line limit. */
    @Test
    fun aLongParagraphIsCutIntoBoundedBlocks() {
        /** Notice text of 100 short lines with no blank line. */
        val notices: String = (1..100).joinToString(separator = "\n") { number -> "line $number" }
        /** Blocks the splitter makes of it. */
        val blocks: List<LicenseBlock> = splitLicenseNotices(notices)
        assertEquals(3, blocks.size)
        assertTrue(blocks.all { block -> block.text.lines().size <= LICENSE_BLOCK_MAX_LINES })
        assertEquals(notices, blocks.joinToString(separator = "\n") { block -> block.text })
    }

    /** A large text splits into many bounded blocks, none larger than the line limit allows. */
    @Test
    fun aLargeTextSplitsIntoManySmallBlocks() {
        /** One license-like paragraph of 30 wrapped lines. */
        val paragraph: String = (1..30).joinToString(separator = "\n") { WRAPPED_LINE }
        /** Notice text of 2,000 such paragraphs under a section. */
        val notices: String = "$EQUALS_RULE\nBig\n$EQUALS_RULE\n" + (1..2000).joinToString(separator = "\n\n") { paragraph }
        /** Blocks the splitter makes of it. */
        val blocks: List<LicenseBlock> = splitLicenseNotices(notices)
        assertEquals(2001, blocks.size)
        assertTrue(blocks.all { block -> block.text.length <= LICENSE_BLOCK_MAX_LINES * (WRAPPED_LINE.length + 1) })
    }
}

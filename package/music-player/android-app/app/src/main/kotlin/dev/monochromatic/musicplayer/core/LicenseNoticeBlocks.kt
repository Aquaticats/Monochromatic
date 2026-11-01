// Splits the text of the license notices asset into the blocks the Licenses page lists. The notice file is about 1.7 MB
// and 35,000 lines, so the page cannot hold it in one text node: each block is one list item of bounded size.
// Format of the file, written by `desktop-app/bin/third-party-notices.mjs`: a title between two rules of 78 equal signs
// opens a section, a title above a rule of 78 hyphens opens a subsection, and blank lines end paragraphs.

// What:     `package dev.monochromatic.musicplayer.core` places the splitter beside the other pure rules.
// Why:      The splitter needs no Android type, so the JVM unit tests run it without Robolectric.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `private const val RULE_LENGTH: Int = 78` is the width of every rule line in the notice file.
// Why:      The file draws each rule at exactly 78 characters; matching the width keeps a license text that happens to
//           contain a shorter line of equal signs or hyphens from being read as a heading.
//
// In TS you'd write (pseudocode):
// ```ts
// const RULE_LENGTH = 78;
// ```
/** Width of the equal-sign and hyphen rules that frame headings in the notice file. */
private const val RULE_LENGTH: Int = 78

// What:     `internal const val LICENSE_BLOCK_MAX_LINES: Int = 40` is the most source lines one block holds.
// Why:      A license text without blank lines (a list of 121 library coordinates, or a pasted legal block) must still
//           become several list items, so no item grows large enough to make a frame slow.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LICENSE_BLOCK_MAX_LINES = 40;
// ```
/** Most lines of the notice file that one block takes before the block is closed and a new one starts. */
internal const val LICENSE_BLOCK_MAX_LINES: Int = 40

// What:     `private const val WRAPPED_LINE_MIN: Int = 45` is the shortest length of a line cut to fit a column.
// Why:      Prose in the file is hard-wrapped at 70 to 120 characters, so every line of a wrapped paragraph except the
//           last is long. Lists and centered titles are shorter, and joining them would destroy their shape.
//
// In TS you'd write (pseudocode):
// ```ts
// const WRAPPED_LINE_MIN = 45;
// ```
/** Trimmed length from which a line counts as part of hard-wrapped prose. */
private const val WRAPPED_LINE_MIN: Int = 45

// What:     `private const val SECTION_HEADING_LINES: Int = 3` is how many lines a section heading takes.
// Why:      A section heading is a rule, the title and a rule, so the reader skips all three together.
//
// In TS you'd write (pseudocode):
// ```ts
// const SECTION_HEADING_LINES = 3;
// ```
/** Lines of a section heading: the rule above, the title and the rule below. */
private const val SECTION_HEADING_LINES: Int = 3

// What:     `enum class LicenseBlockKind` names the things the page draws.
// Why:      The page draws a section title, a subsection title, wrapped prose and preformatted lines differently, and
//           the list needs one content type per look.
//
// In TS you'd write (pseudocode):
// ```ts
// type LicenseBlockKind = "section" | "subsection" | "prose" | "lines";
// ```
/** What one block of the notice file is. */
enum class LicenseBlockKind {
    /** A title framed by two rules of equal signs, such as `Android libraries (121 artifacts)`. */
    SECTION,

    /** A title above one rule of hyphens, such as `Under Apache-2.0: 121 artifacts`. */
    SUBSECTION,

    /** Hard-wrapped prose whose lines were joined into one paragraph that wraps to the screen. */
    PROSE,

    /** Lines kept as written, such as a list of libraries or a centered title, drawn in a monospace face. */
    LINES,
}

// What:     `data class LicenseBlock(kind, text)` is one list item of the Licenses page.
// Why:      The list needs only the kind, which picks the look, and the text, which is shown.
//
// In TS you'd write (pseudocode):
// ```ts
// type LicenseBlock = { kind: LicenseBlockKind; text: string };
// ```
/** One list item of the Licenses page: its [kind] and the [text] to show. */
data class LicenseBlock(
    /** Picks how the item is drawn. */
    val kind: LicenseBlockKind,
    /** The text of the item; lines of a [LicenseBlockKind.LINES] block are separated by a newline. */
    val text: String,
)

// What:     `private fun isRule(line: String, mark: Char): Boolean` tells whether a line is one full-width rule.
// Why:      Headings are recognized only by their rules, and the rule must be exactly the width the file writes.
//
// In TS you'd write (pseudocode):
// ```ts
// const isRule = (line: string, mark: string): boolean =>
//   line.length === RULE_LENGTH && [...line].every((c) => c === mark);
// ```
/** True when [line] is exactly [RULE_LENGTH] copies of [mark]. */
private fun isRule(line: String, mark: Char): Boolean = line.length == RULE_LENGTH && line.all { char -> char == mark }

// What:     `private fun blockOf(lines: List<String>): LicenseBlock` turns the lines of one paragraph into a block.
// Why:      Hard-wrapped prose reads badly when each source line breaks again at the screen width, so it is joined;
//           anything shorter keeps its line breaks.
//
// In TS you'd write (pseudocode):
// ```ts
// function blockOf(lines: string[]): LicenseBlock {
//   const wrapped = lines.length >= 2 && lines.slice(0, -1).every((l) => l.trim().length >= WRAPPED_LINE_MIN);
//   return wrapped
//     ? { kind: "prose", text: lines.map((l) => l.trim()).join(" ") }
//     : { kind: "lines", text: lines.join("\n") };
// }
// ```
/**
 * Joins the lines of one paragraph into prose when all but its last line are long, else keeps them as lines with the
 * indentation they all share removed.
 */
private fun blockOf(lines: List<String>): LicenseBlock {
    /** Whether the paragraph has two or more lines and each line before the last was cut to fit a column. */
    val wrapped: Boolean = lines.size >= 2 &&
        lines.take(lines.size - 1).all { line -> line.trim().length >= WRAPPED_LINE_MIN }
    if (wrapped) {
        return LicenseBlock(kind = LicenseBlockKind.PROSE, text = lines.joinToString(separator = " ") { it.trim() })
    }
    /** Spaces that every non-blank line starts with, which a centered title carries and a narrow screen cannot use. */
    val commonIndent: Int = lines.filter { line -> line.isNotBlank() }
        .minOfOrNull { line -> line.length - line.trimStart().length } ?: 0
    return LicenseBlock(
        kind = LicenseBlockKind.LINES,
        text = lines.joinToString(separator = "\n") { line -> line.drop(commonIndent).trimEnd() },
    )
}

// What:     `fun splitLicenseNotices(notices: String): List<LicenseBlock>` cuts the whole notice text into blocks.
// Why:      The Licenses page shows the 1.7 MB file as a lazy list; one pass over the lines yields the items, with a
//           bound on the size of each, and runs on a background dispatcher before the list is shown.
//
// In TS you'd write (pseudocode):
// ```ts
// function splitLicenseNotices(notices: string): LicenseBlock[] { /* one pass over notices.split("\n") */ }
// ```
/**
 * Splits [notices] into section titles, subsection titles and paragraphs, in file order, in one pass.
 * A paragraph ends at a blank line, at a heading, or after [LICENSE_BLOCK_MAX_LINES] lines.
 * Blank lines and rule lines are dropped, and wrapped prose loses the line breaks and indentation it was cut with;
 * every word of the text is kept in some block.
 */
fun splitLicenseNotices(notices: String): List<LicenseBlock> {
    /** Source lines without a carriage return, so a file with Windows line ends splits the same way. */
    val lines: List<String> = notices.split('\n').map { line -> line.trimEnd('\r') }
    /** Blocks found so far, in file order. */
    val blocks: MutableList<LicenseBlock> = mutableListOf()
    /** Lines of the paragraph being read. */
    val paragraph: MutableList<String> = mutableListOf()

    /** Closes the paragraph being read, if it holds any line, as one block. */
    fun closeParagraph() {
        if (paragraph.isNotEmpty()) {
            blocks.add(blockOf(paragraph.toList()))
            paragraph.clear()
        }
    }

    /** Position of the line being read. */
    var index = 0
    while (index < lines.size) {
        /** The line being read. */
        val line: String = lines[index]
        /** The line after it, or an empty text at the end of the file. */
        val next: String = lines.getOrElse(index + 1) { "" }
        /** The line two after it, or an empty text at the end of the file. */
        val afterNext: String = lines.getOrElse(index + 2) { "" }
        if (isRule(line, '=') && next.isNotBlank() && !isRule(next, '=') && isRule(afterNext, '=')) {
            closeParagraph()
            blocks.add(LicenseBlock(kind = LicenseBlockKind.SECTION, text = next.trim()))
            index += SECTION_HEADING_LINES
        } else if (line.isNotBlank() && !isRule(line, '-') && !isRule(line, '=') && isRule(next, '-')) {
            closeParagraph()
            blocks.add(LicenseBlock(kind = LicenseBlockKind.SUBSECTION, text = line.trim()))
            index += 2
        } else if (line.isBlank() || isRule(line, '=') || isRule(line, '-')) {
            closeParagraph()
            index += 1
        } else {
            paragraph.add(line)
            if (paragraph.size >= LICENSE_BLOCK_MAX_LINES) {
                closeParagraph()
            }
            index += 1
        }
    }
    closeParagraph()
    return blocks.toList()
}

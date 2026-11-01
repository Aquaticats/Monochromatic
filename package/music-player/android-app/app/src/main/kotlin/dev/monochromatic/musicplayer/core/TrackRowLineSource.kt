// The supporting lines of one page's track rows, computed row by row when a row is drawn. The list of rows keeps its
// shape while the analysis publishes measures, so a new measure recomposes only the row it belongs to, never the whole
// list (an ANR on a folder of thousands of tracks). This file is pure logic with no Compose imports, so it is
// unit-tested on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places the source beside the row entries it complements.
// Why:      The source evaluates the same template fields as the entries, so it shares their package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `class TrackRowLineSource(...)` answers the supporting line of any row of one page.
// Why:      The host builds the rows once per page, playing row and template. Each visible row then asks this source
//           for its own line, reading its own measure, so the work per analysis step is the visible rows, not the page.
//
// In TS you'd write (pseudocode):
// ```ts
// class TrackRowLineSource {
//   constructor(template: string, pathsByIndex: Map<number, string>, measureOf: (i: number) => TrackMeasure | null) {}
//   lineOf(index: number): string | null { /* path of the row, its measure, the template */ }
// }
// ```
/**
 * Computes the supporting line of one page's track rows on demand, from each row's display path and its own measure.
 *
 * @param template track-row template text that every line is evaluated with
 * @param pathsByIndex display path of each row of the page, keyed by load-order index
 * @param measureOf returns the measure of the track at a load-order index, or null while it is unread; a reader that
 *     tracks state reads (a Compose composition) learns of a change to that measure through this call
 */
class TrackRowLineSource(
    private val template: String,
    private val pathsByIndex: Map<Int, String>,
    private val measureOf: (Int) -> TrackMeasure?,
) {
    /** Returns the supporting line of the row at [indexInLibrary], or null when the template shows nothing for it. */
    fun lineOf(indexInLibrary: Int): String? {
        /** Display path of the row, or null when the index is not on this page. */
        val displayPath: String = pathsByIndex[indexInLibrary] ?: return null
        return supportingLineOf(displayPath = displayPath, measure = measureOf(indexInLibrary), template = template)
    }

    /** Builds the source of one page. */
    companion object {
        // What:     `fun forPage(...)` builds the source of one page from its entries.
        // Why:      The source needs each row's display path by index, which the page entries already hold.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const forPage = (pageItems, template, measureOf) =>
        //   new TrackRowLineSource(template, new Map(pageItems.map((e) => [e.index, e.name])), measureOf);
        // ```
        /** Builds the source of one page, indexing the display path of every page entry once. */
        fun forPage(
            pageItems: List<PageEntry>,
            template: String,
            measureOf: (Int) -> TrackMeasure?,
        ): TrackRowLineSource = TrackRowLineSource(
            template = template,
            pathsByIndex = pageItems.associate { entry -> entry.index to entry.name },
            measureOf = measureOf,
        )
    }
}

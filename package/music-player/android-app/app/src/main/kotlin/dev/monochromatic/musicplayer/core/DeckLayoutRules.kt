// Pure rules that keep the transport deck and the top bar steady: the width the shuffle-folder segment takes, and how
// many lines a heading may use at a given text size. Both depend on the screen and the text size only, never on the
// folder or track name, so browsing never moves the deck or the folder pane's lower edge.

// What:     `package dev.monochromatic.musicplayer.core` places the rules beside the other pure layout helpers.
// Why:      The rules take plain numbers only, so they share the core package and are unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal const val LARGE_TEXT_FONT_SCALE: Float = 1.5f` is the font scale from which headings wrap.
// Why:      D1 records that the mode control already moves to four rows at 150 percent text, so the same step is where
//           a heading stops fitting one line and gets a second one instead of an ellipsis.
//
// In TS you'd write (pseudocode):
// ```ts
// const LARGE_TEXT_FONT_SCALE = 1.5;
// ```
/** Font scale from which the deck heading and the cover title may use two lines. */
internal const val LARGE_TEXT_FONT_SCALE: Float = 1.5f

// What:     `internal fun headingLineBudget(fontScale)` is the number of lines each heading part gets.
// Why:      One line with an ellipsis keeps the deck's height fixed at ordinary text sizes. At large text one line
//           truncates both parts, so each part reserves exactly two lines (minimum and maximum), which wraps a long
//           name and still keeps the deck the same height for every name.
//
// In TS you'd write (pseudocode):
// ```ts
// const headingLineBudget = (fontScale: number) => (fontScale >= LARGE_TEXT_FONT_SCALE ? 2 : 1);
// ```
/** Returns how many lines a heading part reserves and may use at this font scale. */
internal fun headingLineBudget(fontScale: Float): Int = if (fontScale >= LARGE_TEXT_FONT_SCALE) 2 else 1

// What:     `internal fun titleLineBudget(fontScale)` is the number of lines the track title part gets.
// Why:      The title shares the heading line with the folder name and gets the narrower remainder, so at large text it
//           reserves one line more than the folder part before it is cut.
//
// In TS you'd write (pseudocode):
// ```ts
// const titleLineBudget = (fontScale: number) =>
//   headingLineBudget(fontScale) + (fontScale >= LARGE_TEXT_FONT_SCALE ? 1 : 0);
// ```
/** Returns how many lines the track title reserves and may use at this font scale. */
internal fun titleLineBudget(fontScale: Float): Int =
    headingLineBudget(fontScale) + if (fontScale >= LARGE_TEXT_FONT_SCALE) 1 else 0

// What:     `internal fun shuffleFolderTextWidthPx(naturalPx, referencePx)` is the width of the folder name's text.
// Why:      D1 caps the folder name at the rendered width of the reference name `Camellia` and cuts a longer name in
//           the middle; a shorter name keeps its own width.
//
// In TS you'd write (pseudocode):
// ```ts
// const shuffleFolderTextWidthPx = (naturalPx, referencePx) => Math.min(naturalPx, referencePx);
// ```
/** Returns the width of the folder name text inside the shuffle segment, capped at the reference name's width. */
internal fun shuffleFolderTextWidthPx(naturalPx: Int, referencePx: Int): Int = minOf(naturalPx, referencePx)

// What:     `internal fun shuffleLabelSlotWidthPx(prefixPx, referencePx)` is the width the whole label always takes.
// Why:      The segment reserves the prefix plus the reference name's width whatever the folder is called, so the
//           one-row, two-row and side-by-side decisions, which read this width, never change with the name, and the
//           deck stays the same height when the user browses from a folder called `L` to a long one.
//
// In TS you'd write (pseudocode):
// ```ts
// const shuffleLabelSlotWidthPx = (prefixPx, referencePx) => prefixPx + referencePx;
// ```
/** Returns the fixed width of the shuffle label: the prefix plus the reference name, independent of the folder. */
internal fun shuffleLabelSlotWidthPx(prefixPx: Int, referencePx: Int): Int = prefixPx + referencePx

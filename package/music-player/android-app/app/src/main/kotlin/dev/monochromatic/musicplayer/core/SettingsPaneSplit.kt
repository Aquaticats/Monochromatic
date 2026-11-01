// Pure rules for the wide Settings form: how wide the list pane and the editor pane are, and how far each pane's text
// stays from the shared edge. A device with a crease splits the window in half and keeps text out of the crease band
// (E2 and D110); a device without one follows the stock list ratio and needs no clearance. This file has no Compose
// imports, so it is unit-tested on the JVM.

// What:     `package dev.monochromatic.musicplayer.core` places these rules beside the other pure layout helpers.
// Why:      The rules take plain numbers only, so they share the core package and need no import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `import kotlin.math.roundToInt` rounds the list pane's width to the nearest pixel.
// Why:      The stock ratio rarely lands on a whole pixel, and the editor pane takes the rest.
//
// In TS you'd write (pseudocode):
// ```ts
// // Math.round is global.
// ```
import kotlin.math.roundToInt

// What:     `internal const val CREASE_FLOOR_WIDTH_MM: Float = 7.5f` is the least total gap across the crease.
// Why:      E2 fixes 7.5 mm as the minimum total clearance between opposing information, in physical units.
//
// In TS you'd write (pseudocode):
// ```ts
// const CREASE_FLOOR_WIDTH_MM = 7.5;
// ```
/** Least total clearance across the crease in millimeters (E2). */
internal const val CREASE_FLOOR_WIDTH_MM: Float = 7.5f

// What:     `internal const val CREASE_PANEL_WIDTH_MM: Float = 141.08f` is the active inner panel width.
// Why:      The millimeter floor becomes physical pixels through the panel's measured width.
//
// In TS you'd write (pseudocode):
// ```ts
// const CREASE_PANEL_WIDTH_MM = 141.08;
// ```
/** Active width of the Fold inner panel in millimeters, from the published diagonal (E2). */
internal const val CREASE_PANEL_WIDTH_MM: Float = 141.08f

// What:     `internal const val CREASE_PANEL_WIDTH_PX: Float = 2076f` is the inner panel width in pixels.
// Why:      Pairs with `CREASE_PANEL_WIDTH_MM` to turn millimeters into physical pixels.
//
// In TS you'd write (pseudocode):
// ```ts
// const CREASE_PANEL_WIDTH_PX = 2076;
// ```
/** Width of the Fold inner panel in physical pixels (E2). */
internal const val CREASE_PANEL_WIDTH_PX: Float = 2076f

// What:     `internal const val CREASE_FLOOR_WIDTH_PX: Float` is the least crease clearance in physical pixels.
// Why:      The floor is physical, so it does not change with the display size setting; it is about 110 px.
//
// In TS you'd write (pseudocode):
// ```ts
// const CREASE_FLOOR_WIDTH_PX = (7.5 / 141.08) * 2076;
// ```
/** Least total clearance across the crease in physical pixels, derived from the millimeter floor. */
internal const val CREASE_FLOOR_WIDTH_PX: Float =
    CREASE_FLOOR_WIDTH_MM / CREASE_PANEL_WIDTH_MM * CREASE_PANEL_WIDTH_PX

// What:     `internal const val SETTINGS_DEFAULT_LIST_RATIO: Float = 0.3636f` is the list pane's share when the
//           device has no crease.
// Why:      D110 reads the platform default as the stock Settings `config_activity_embed_split_ratio`.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_DEFAULT_LIST_RATIO = 0.3636;
// ```
/** Share of the window width that the list pane takes on a device without a crease. */
internal const val SETTINGS_DEFAULT_LIST_RATIO: Float = 0.3636f

// What:     `internal const val SETTINGS_LIST_TEXT_END_INSET_DP: Float = 32f` is the list text's ordinary end inset.
// Why:      The list's own 16 dp card margin plus the 16 dp inside the row already keep its text 32 dp from the edge.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_LIST_TEXT_END_INSET_DP = 16 + 16;
// ```
/** Distance in dp from the list pane's edge to the end of its row text, before any crease clearance is added. */
internal const val SETTINGS_LIST_TEXT_END_INSET_DP: Float = SETTINGS_PAGE_MARGIN_DP + SETTINGS_ROW_INNER_PADDING_DP

// What:     `internal const val SETTINGS_EDITOR_TEXT_START_INSET_DP: Float = 16f` is the editor text's ordinary inset.
// Why:      The editor starts its text 16 dp from the page edge, which is the ordinary pane inset.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_EDITOR_TEXT_START_INSET_DP = 16;
// ```
/** Distance in dp from the editor pane's edge to the start of its text, before any crease clearance is added. */
internal const val SETTINGS_EDITOR_TEXT_START_INSET_DP: Float = 16f

// What:     `internal class SettingsPaneException` signals a split input that cannot be measured.
// Why:      A negative width or non-positive density has no physical meaning, so the rule throws instead of guessing.
//
// In TS you'd write (pseudocode):
// ```ts
// class SettingsPaneError extends Error {}
// ```
/** Thrown when the window width or the density is outside its measurable range. */
internal class SettingsPaneException(message: String) : IllegalArgumentException(message)

// What:     `internal data class SettingsPaneSplit` holds the pane widths and the extra text clearance of each pane.
// Why:      The host places both panes from the widths in pixels, and each pane adds its clearance to its text inset.
//
// In TS you'd write (pseudocode):
// ```ts
// type SettingsPaneSplit = { listWidthPx: number; editorWidthPx: number;
//   listTextEndExtraDp: number; editorTextStartExtraDp: number };
// ```
/** Widths of the two Settings panes in pixels and the clearance each pane adds beside the shared edge. */
internal data class SettingsPaneSplit(
    /** Width in pixels of the list pane on the left. */
    val listWidthPx: Int,
    /** Width in pixels of the editor pane on the right, which is the rest of the window. */
    val editorWidthPx: Int,
    /** Dp added to the list text's end inset so the text keeps clear of the crease; zero without a crease. */
    val listTextEndExtraDp: Float,
    /** Dp added to the editor text's start inset so the text keeps clear of the crease; zero without a crease. */
    val editorTextStartExtraDp: Float,
)

// What:     `internal fun creaseHalfClearanceDp(creaseWidthPx, density)` is half of the total gap across the crease.
// Why:      E2 asks for `max(7.5 mm, crease width)` in total, shared half to each side, and Settings and Search both
//           apply it, so one function owns the rule.
//
// In TS you'd write (pseudocode):
// ```ts
// const creaseHalfClearanceDp = (creaseWidthPx, density) =>
//   Math.max(CREASE_FLOOR_WIDTH_PX, creaseWidthPx) / 2 / density;
// ```
/**
 * Returns the clearance each side of the crease keeps free of text, in dp.
 *
 * @param creaseWidthPx crease width in physical pixels as the platform reports it, zero when it reports none.
 * @param density physical pixels per dp of the current display.
 */
internal fun creaseHalfClearanceDp(creaseWidthPx: Float, density: Float): Float =
    maxOf(CREASE_FLOOR_WIDTH_PX, creaseWidthPx) / 2f / density

// What:     `internal fun settingsPaneSplit(...)` decides the pane widths and the crease clearance of the wide form.
// Why:      With a crease the window splits in half and each pane keeps half of `max(7.5 mm, crease width)` clear of
//           the center; without one the list takes the stock ratio and no clearance is added.
//
// In TS you'd write (pseudocode):
// ```ts
// function settingsPaneSplit(p: { creasePresent: boolean; windowWidthPx: number; creaseWidthPx: number;
//   density: number }): SettingsPaneSplit { ... }
// ```
/**
 * Splits a window [windowWidthPx] wide into the list pane and the editor pane.
 *
 * @param creasePresent whether the platform reports a vertical crease or hinge.
 * @param windowWidthPx window width in physical pixels.
 * @param creaseWidthPx crease width in physical pixels as the platform reports it, zero when it reports none.
 * @param density physical pixels per dp of the current display, which converts the clearance to dp.
 * @throws SettingsPaneException when the width is negative or the density is not positive.
 */
internal fun settingsPaneSplit(
    creasePresent: Boolean,
    windowWidthPx: Int,
    creaseWidthPx: Float,
    density: Float,
): SettingsPaneSplit {
    if (windowWidthPx < 0) {
        throw SettingsPaneException("Window width must not be negative, but was $windowWidthPx px.")
    }
    if (density <= 0f) {
        throw SettingsPaneException("Density must be positive to convert the clearance, but was $density.")
    }
    if (!creasePresent) {
        /** Width of the list pane at the stock ratio, rounded to the nearest pixel. */
        val listWidth: Int = (windowWidthPx * SETTINGS_DEFAULT_LIST_RATIO).roundToInt()
        return SettingsPaneSplit(
            listWidthPx = listWidth,
            editorWidthPx = windowWidthPx - listWidth,
            listTextEndExtraDp = 0f,
            editorTextStartExtraDp = 0f,
        )
    }
    /** Half of the window, which is the list pane's width and the editor pane's width up to one odd pixel. */
    val half: Int = windowWidthPx / 2
    /** Half of the total clearance in dp: the larger of the 7.5 mm floor and the reported crease, per pane. */
    val halfClearanceDp: Float = creaseHalfClearanceDp(creaseWidthPx = creaseWidthPx, density = density)
    return SettingsPaneSplit(
        listWidthPx = half,
        editorWidthPx = windowWidthPx - half,
        listTextEndExtraDp = (halfClearanceDp - SETTINGS_LIST_TEXT_END_INSET_DP).coerceAtLeast(0f),
        editorTextStartExtraDp = (halfClearanceDp - SETTINGS_EDITOR_TEXT_START_INSET_DP).coerceAtLeast(0f),
    )
}

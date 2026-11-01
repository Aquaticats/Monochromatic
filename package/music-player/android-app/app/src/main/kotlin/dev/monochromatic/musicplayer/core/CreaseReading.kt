// Pure rules that turn what the platform reports about folds into the crease the layout dodges (D110, E2). The Jetpack
// WindowManager `FoldingFeature` list is read by `CreaseDetection.kt` into plain numbers, and this file decides from
// them, so every case runs on the JVM without Android types (a `Rect` cannot be built in a host unit test).

// What:     `package dev.monochromatic.musicplayer.core` places the rule beside the pane split that consumes it.
// Why:      The rule takes plain values only, so it shares the core package and needs no import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal enum class CreaseSource` names where a crease answer came from.
// Why:      The log line and the tests need to tell the platform layout API from the hinge-sensor fallback.
//
// In TS you'd write (pseudocode):
// ```ts
// type CreaseSource = "folding-feature" | "hinge-sensor" | "none";
// ```
/** Where the crease answer came from. */
internal enum class CreaseSource {
    /** A vertical `FoldingFeature` in the window layout (or the layout reported none). */
    FOLDING_FEATURE,

    /** The hinge-angle system feature, used only until the first layout report arrives. */
    HINGE_SENSOR,
}

// What:     `internal data class FoldReading` is one `FoldingFeature` reduced to plain values.
// Why:      The decision needs only the orientation and the bounds' horizontal extent; the other fields are kept so the
//           log can show everything the platform said.
//
// In TS you'd write (pseudocode):
// ```ts
// type FoldReading = { vertical: boolean; separating: boolean; occluding: boolean; leftPx: number; rightPx: number };
// ```
/** One folding feature of the window layout as plain values. */
internal data class FoldReading(
    /** Whether the fold line is vertical (the book-like unfolded Fold), not horizontal (tabletop). */
    val vertical: Boolean,
    /** The platform's "separating" flag: the fold splits the window into two logical areas. */
    val separating: Boolean,
    /** Whether the feature hides part of the display (an occluding hinge) rather than only marking the fold. */
    val occluding: Boolean,
    /** Left edge of the feature's bounds in window pixels. */
    val leftPx: Int,
    /** Right edge of the feature's bounds in window pixels. */
    val rightPx: Int,
) {
    /** Horizontal extent of the feature's bounds in pixels, which is the crease width it reports (0 for a line). */
    val widthPx: Int get() = rightPx - leftPx
}

// What:     `internal data class CreaseState` is the crease answer for one window.
// Why:      Settings and Search both read it: whether to dodge, and how wide the reported crease is.
//
// In TS you'd write (pseudocode):
// ```ts
// type CreaseState = { present: boolean; reportedWidthPx: number; fold: FoldReading | null; source: CreaseSource };
// ```
/** Whether the window has a vertical crease, how wide the platform reports it, and where the answer came from. */
internal data class CreaseState(
    /** Whether the window has a vertical crease that text must keep clear of. */
    val present: Boolean,
    /** Crease width in physical pixels as reported (0 when only the presence is known). */
    val reportedWidthPx: Float,
    /** The fold the answer is based on, or null when none was reported. */
    val fold: FoldReading?,
    /** Where the answer came from. */
    val source: CreaseSource,
) {
    /** The reported width when a crease is present, else null: the form `settingsPaneSplit` callers use. */
    val widthPxOrNull: Float? get() = if (present) reportedWidthPx else null
}

// What:     `internal fun resolveCrease(folds, hingeSensorPresent)` decides whether the window has a crease.
// Why:      `folds` is null until the layout API has reported once, and only then does the hinge-angle system feature
//           stand in. After a report, a vertical fold means a crease (a flat unfolded Fold reports `isSeparating` false
//           and no occlusion, so neither flag is required) and no vertical fold means none, which covers the folded
//           cover, a tablet and tabletop posture.
//
// In TS you'd write (pseudocode):
// ```ts
// const resolveCrease = (folds, hinge) => folds === null ? fallback(hinge) : fromFolds(folds);
// ```
/**
 * Decides the crease of a window from its folding features.
 *
 * @param folds the folding features of the latest window layout, or null when none has been reported yet.
 * @param hingeSensorPresent whether the device declares the hinge-angle feature, used only while [folds] is null.
 */
internal fun resolveCrease(folds: List<FoldReading>?, hingeSensorPresent: Boolean): CreaseState {
    if (folds == null) {
        return CreaseState(
            present = hingeSensorPresent,
            reportedWidthPx = 0f,
            fold = null,
            source = CreaseSource.HINGE_SENSOR,
        )
    }
    /** The widest vertical fold, which is the crease when a layout somehow lists several. */
    val vertical: FoldReading? = folds.filter { fold -> fold.vertical }.maxByOrNull { fold -> fold.widthPx }
    return CreaseState(
        present = vertical != null,
        reportedWidthPx = (vertical?.widthPx ?: 0).toFloat(),
        fold = vertical,
        source = CreaseSource.FOLDING_FEATURE,
    )
}

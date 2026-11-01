// The player's theme (A3, D108): the system accent is the only thing taken from the platform, and every role is derived
// from it in OKLCH by `core/ThemeRoles.kt`, the code the desktop's Rust theme is ported from. This file reads the root,
// maps the derived roles onto Material 3's color slots (so stock components follow the same palette), and carries the
// roles that have no Material slot (the Search match fill and the selected-fill hover and pressed layers) through a
// composition local. Platform dynamic color (`dynamicLightColorScheme`, `dynamicDarkColorScheme`) is not used.

// What:     `package dev.monochromatic.musicplayer` places the theme beside the activity and the pages that read it.
// Why:      The theme needs Compose and Android types, which the pure `core` package must not import.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Imports bring in Android's context and log, Compose's theme types, and the pure derivation.
// Why:      The root comes from an Android resource, the scheme from Compose's `ColorScheme`, the roles from `core`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ColorScheme, MaterialTheme } from "compose/material3";
// ```
import android.content.Context
import android.os.Build
import android.util.Log
import androidx.annotation.RequiresApi
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import dev.monochromatic.musicplayer.core.BASELINE_ROOT
import dev.monochromatic.musicplayer.core.RoleSet
import dev.monochromatic.musicplayer.core.Srgb8
import dev.monochromatic.musicplayer.core.ThemePalette
import dev.monochromatic.musicplayer.core.deriveRoles

// What:     The private constants `THEME_TAG`, `ON_ERROR_DARK` and `ON_ERROR_LIGHT` name the log tag and the fixed
//           on-error colors.
// Why:      The error roles are semantic constants that follow no accent, and Material 3 needs an on-error color
//           for each appearance.
//
// In TS you'd write (pseudocode):
// ```ts
// const THEME_TAG = "PlayerTheme"; const ON_ERROR_DARK = "#690005"; const ON_ERROR_LIGHT = "#FFFFFF";
// ```
/** Log tag of the theme. */
private const val THEME_TAG: String = "PlayerTheme"

/** Semantic dark-scheme color of text on the error fill (Material 3 error tone 20), fixed under every accent. */
private val ON_ERROR_DARK: Color = Color(0xFF690005)

/** Semantic light-scheme color of text on the error fill, fixed under every accent. */
private val ON_ERROR_LIGHT: Color = Color(0xFFFFFFFF)

// What:     `internal val LocalThemeRoles` carries the roles of the scheme in use.
// Why:      The Search match fill and the selected-fill layers have no Material 3 slot, so pages read them here; the
//           default is the baseline light scheme so previews and tests without a theme still draw.
//
// In TS you'd write (pseudocode):
// ```ts
// const LocalThemeRoles = createContext<RoleSet>(deriveRoles(null).light);
// ```
/** The derived roles of the scheme in use, for the roles that Material 3 has no slot for. */
internal val LocalThemeRoles: ProvidableCompositionLocal<RoleSet> =
    staticCompositionLocalOf { deriveRoles(null).light }

// What:     `internal fun Srgb8.toComposeColor()` turns a derived role into a Compose color.
// Why:      The theme and the pages read roles as Compose colors; one conversion keeps the channels exact.
//
// In TS you'd write (pseudocode):
// ```ts
// const toComposeColor = (role) => color(role.red, role.green, role.blue);
// ```
/** Converts a derived role to a Compose color. */
internal fun Srgb8.toComposeColor(): Color = Color(toArgb())

// What:     `@RequiresApi(S) private fun accentFromPlatform(context)` reads the middle tone of the system accent.
// Why:      D108 takes `android.R.color.system_accent1_500` as the root: it is the platform's own accent palette
//           at tone 500, which the wallpaper (or the user's color pick) sets. It is the one thing taken from the
//           platform.
//
// In TS you'd write (pseudocode):
// ```ts
// const accentFromPlatform = (ctx) => toSrgb8(ctx.getColor(R.color.system_accent1_500));
// ```
/** Reads the system accent palette's middle tone on API 31 and later. */
@RequiresApi(Build.VERSION_CODES.S)
private fun accentFromPlatform(context: Context): Srgb8 =
    Srgb8.fromRgbInt(context.getColor(android.R.color.system_accent1_500))

// What:     `internal fun systemAccentRoot(context)` is the theme root, or null where the system has no accent palette.
// Why:      Below API 31 there is no system accent, so the derivation uses the Fold baseline (null means baseline).
//
// In TS you'd write (pseudocode):
// ```ts
// const systemAccentRoot = (ctx) => (sdk >= 31 ? accentFromPlatform(ctx) : null);
// ```
/** Returns the system accent as the theme root, or null below API 31 where the baseline root applies. */
internal fun systemAccentRoot(context: Context): Srgb8? {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) {
        return null
    }
    return accentFromPlatform(context)
}

// What:     `internal fun playerColorScheme(palette, dark)` maps the derived roles onto Material 3's color slots.
// Why:      Stock components (text fields, buttons, menus, list items) read Material slots, so every slot is set from a
//           derived role and none keeps the library's purple default. Slots with no derived role of their own take the
//           nearest one: tertiary is the primary family, primary container is the selected-fill pair, the surface
//           containers run canvas, current row, then the menu surface, and the fixed accents repeat those pairs.
//           One factory builds both appearances because every slot is set and the appearance lives in the roles.
//
// In TS you'd write (pseudocode):
// ```ts
// const playerColorScheme = (palette, dark) => { const set = dark ? palette.dark : palette.light; return {...}; };
// ```
/**
 * Builds the Material 3 color scheme of one appearance from a derived palette.
 *
 * Background and surface are the canvas (pure black in dark, white in light), and the error slots are the fixed
 * semantic colors of the derivation.
 */
internal fun playerColorScheme(palette: ThemePalette, dark: Boolean): ColorScheme {
    /** Roles of the appearance in use. */
    val set: RoleSet = if (dark) palette.dark else palette.light
    /** Roles of the other appearance, for the inverse primary. */
    val other: RoleSet = if (dark) palette.light else palette.dark
    /** Accent fill color. */
    val primary: Color = set.primary.toComposeColor()
    /** Content color on the accent fill. */
    val onPrimary: Color = set.onPrimary.toComposeColor()
    /** Selected-fill color. */
    val container: Color = set.secondaryContainer.toComposeColor()
    /** Content color on the selected fill. */
    val onContainer: Color = set.onSecondaryContainer.toComposeColor()
    /** Page background color. */
    val canvas: Color = set.canvas.toComposeColor()
    /** Menu surface color. */
    val menu: Color = set.surfaceHigh.toComposeColor()
    /** Primary text color. */
    val onSurface: Color = set.onSurface.toComposeColor()
    /** Secondary text color. */
    val onVariant: Color = set.onSurfaceVariant.toComposeColor()
    /** Content color on the error fill. */
    val onError: Color = if (dark) ON_ERROR_DARK else ON_ERROR_LIGHT
    // Every one of the 48 slots is set, so the factory's own defaults (the baseline purple) never show; the same
    // factory serves both appearances because the appearance lives in the roles.
    return lightColorScheme(
        primary = primary, onPrimary = onPrimary, primaryContainer = container, onPrimaryContainer = onContainer,
        inversePrimary = other.primary.toComposeColor(),
        secondary = primary, onSecondary = onPrimary, secondaryContainer = container,
        onSecondaryContainer = onContainer,
        tertiary = primary, onTertiary = onPrimary, tertiaryContainer = container, onTertiaryContainer = onContainer,
        background = canvas, onBackground = onSurface, surface = canvas, onSurface = onSurface,
        surfaceVariant = menu, onSurfaceVariant = onVariant, surfaceTint = primary,
        inverseSurface = onSurface, inverseOnSurface = canvas,
        error = set.error.toComposeColor(), onError = onError, errorContainer = set.errorContainer.toComposeColor(),
        onErrorContainer = set.onErrorContainer.toComposeColor(),
        outline = set.outline.toComposeColor(), outlineVariant = set.outlineVariant.toComposeColor(),
        scrim = Color.Black,
        surfaceBright = menu, surfaceDim = canvas, surfaceContainerLowest = canvas,
        surfaceContainerLow = set.currentRow.toComposeColor(), surfaceContainer = menu,
        surfaceContainerHigh = menu, surfaceContainerHighest = menu,
        primaryFixed = container, primaryFixedDim = primary, onPrimaryFixed = onContainer,
        onPrimaryFixedVariant = onContainer,
        secondaryFixed = container, secondaryFixedDim = primary, onSecondaryFixed = onContainer,
        onSecondaryFixedVariant = onContainer,
        tertiaryFixed = container, tertiaryFixedDim = primary, onTertiaryFixed = onContainer,
        onTertiaryFixedVariant = onContainer,
    )
}

// What:     `@Composable internal fun playerTheme(content)` themes its content with the derived palette.
// Why:      One entry point serves the launcher activity and the debug screen host, so both show the same roles. It
//           reads the configuration so a change of the system accent, which changes the resources, derives again.
//
// In TS you'd write (pseudocode):
// ```ts
// function PlayerTheme({ children }) {
//   const palette = useMemo(() => deriveRoles(root), [root]);
//   return <ThemeProvider>...</ThemeProvider>;
// }
// ```
/** Provides the Material scheme and the derived roles of the current appearance to [content]. */
@Composable
internal fun playerTheme(content: @Composable () -> Unit) {
    /** Appearance of the system, which picks the scheme. */
    val dark: Boolean = isSystemInDarkTheme()
    /** Context that owns the accent resources. */
    val context: Context = LocalContext.current
    /** Read so that a configuration change, which accompanies an accent change, derives the palette again. */
    val configuration = LocalConfiguration.current
    /** The root color, or null for the baseline. */
    val root: Srgb8? = remember(context, configuration) { systemAccentRoot(context) }
    /** Both schemes derived from the root. */
    val palette: ThemePalette = remember(root) {
        Log.i(THEME_TAG, "deriving the theme from root=${(root ?: BASELINE_ROOT).toHex()} " +
            "source=${if (root == null) "baseline" else "system_accent1_500"} api=${Build.VERSION.SDK_INT}")
        deriveRoles(root)
    }
    /** Material slots of the appearance in use. */
    val scheme: ColorScheme = remember(palette, dark) { playerColorScheme(palette, dark) }
    CompositionLocalProvider(LocalThemeRoles provides if (dark) palette.dark else palette.light) {
        MaterialTheme(colorScheme = scheme) { focusRingTheme(content) }
    }
}

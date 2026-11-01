// The theme color roles, derived from one root color in OKLCH (D108, A3). `deriveRoles` turns the root (the system
// accent, or the Fold baseline when none exists) into every color role the player uses, for the dark and the light
// scheme. Each role has a lightness target, a chroma share and a hue offset, all moved in OKLCH (`OklchColor.kt`) and
// converted to 8-bit sRGB with chroma-reduction gamut mapping. This is a line-for-line port of the desktop's
// `desktop-app/src/theme_roles.rs`; the shared vector `app/src/test/resources/theme-roles-vector.txt` (written by the
// Rust code) proves that both give the same roles for the same root.
// Grays (D118): every gray role (surfaces, deck, outlines, text variants) carries the root's hue at its own fixed
// tint, whatever the root's chroma, unless the root is completely neutral (its three 8-bit channels are equal); then
// every gray role is a true gray. Accent roles still scale with the root's chroma, but for a root that is not
// completely neutral they never fall below the largest gray tint of their scheme, so an accent is never grayer than
// the surfaces (the orchestrator's reading of D118, open to the human's veto).
// Fixed on purpose: the dark and light canvas are pure black (D41) and pure white (D34, D45), which are not grays and
// cannot carry a tint; the error roles are semantic constants that follow no accent.
// Contrast guarantee: after the lightness targets are placed, every text role is moved in lightness (chroma and hue
// kept, gamut-mapped again) until its WCAG ratio holds on every surface and state layer it is drawn on, so AA holds
// for ANY root color. The baseline passes without any adjustment.

// What:     `package dev.monochromatic.musicplayer.core` places the derivation beside the other pure helpers.
// Why:      The derivation takes plain numbers only, so it is unit-tested on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `internal val BASELINE_ROOT: Srgb8` is the Fold capture primary of the light scheme, `#4E5E8B`.
// Why:      D108 names the fixed baseline from the Fold captures as the root when no accent exists; the role table was
//           fitted so that this root reproduces the captured values.
//
// In TS you'd write (pseudocode):
// ```ts
// const BASELINE_ROOT = { red: 0x4e, green: 0x5e, blue: 0x8b };
// ```
/** The root used when the system exposes no accent. */
internal val BASELINE_ROOT: Srgb8 = Srgb8(red = 0x4E, green = 0x5E, blue = 0x8B)

// What:     `internal const val ROOT_CHROMA_CAP: Double = 0.15` limits the root chroma that scales the roles.
// Why:      A neon accent must not make every container garish, so chroma above the cap counts as the cap.
//
// In TS you'd write (pseudocode):
// ```ts
// const ROOT_CHROMA_CAP = 0.15;
// ```
/** Largest root chroma that scales the roles, so a neon accent cannot turn every container garish. */
internal const val ROOT_CHROMA_CAP: Double = 0.15

// What:     `internal const val TEXT_TARGET_RATIO: Double = 4.52` is the contrast ratio text is fitted to.
// Why:      AA asks 4.5; the margin keeps another renderer's rounding of a translucent layer from dropping below it.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEXT_TARGET_RATIO = 4.52;
// ```
/**
 * Contrast ratio the derivation fits text to; AA asks 4.5, and the small margin keeps another renderer's rounding of a
 * translucent layer (one 8-bit step) from dropping below it.
 */
internal const val TEXT_TARGET_RATIO: Double = 4.52

// What:     `internal const val NON_TEXT_TARGET_RATIO: Double = 3.02` is the contrast ratio UI parts are fitted to.
// Why:      AA asks 3 for non-text parts such as outlines.
//
// In TS you'd write (pseudocode):
// ```ts
// const NON_TEXT_TARGET_RATIO = 3.02;
// ```
/** Contrast ratio the derivation fits UI parts (outlines) to; AA for non-text asks 3. */
internal const val NON_TEXT_TARGET_RATIO: Double = 3.02

// What:     `internal const val HOVER_ALPHA: Double = 0.08` is the Material 3 hover state layer.
// Why:      Text must stay legible under the layer, so the fit and the tests use the same alpha.
//
// In TS you'd write (pseudocode):
// ```ts
// const HOVER_ALPHA = 0.08;
// ```
/** Alpha of the hover layer (Material 3 state layer). */
internal const val HOVER_ALPHA: Double = 0.08

// What:     `internal const val PRESSED_ALPHA: Double = 0.12` is the Material 3 pressed state layer.
// Why:      Text must stay legible under the layer, so the fit and the tests use the same alpha.
//
// In TS you'd write (pseudocode):
// ```ts
// const PRESSED_ALPHA = 0.12;
// ```
/** Alpha of the pressed layer (Material 3 state layer). The focus layer has the same strength. */
internal const val PRESSED_ALPHA: Double = 0.12

// What:     `internal val TEXT_LAYER_ALPHAS: List<Double>` lists every layer strength text must survive.
// Why:      A pressed or focused layer lasts as long as the press or the focus does, so text holds AA under both the
//           hover and the pressed strength, not only while a button is held.
//
// In TS you'd write (pseudocode):
// ```ts
// const TEXT_LAYER_ALPHAS = [HOVER_ALPHA, PRESSED_ALPHA];
// ```
/** Layer strengths text must hold AA under: hover, then pressed (which is also the focus strength). */
internal val TEXT_LAYER_ALPHAS: List<Double> = listOf(HOVER_ALPHA, PRESSED_ALPHA)

// What:     The four `MATCH_*_SHARE` constants set how far the match fill moves from primary toward the neutral end.
// Why:      D59 derives the fill from the accent in OKLCH; the values are the ones the Android app and the desktop
//           already share.
//
// In TS you'd write (pseudocode):
// ```ts
// const MATCH_LIGHT_LIGHTNESS_SHARE = 0.72; // and the chroma and dark shares
// ```
/** Search match fill in the light scheme: lightness share moved toward white. */
private const val MATCH_LIGHT_LIGHTNESS_SHARE: Double = 0.72

/** Search match fill in the light scheme: chroma share removed. */
private const val MATCH_LIGHT_CHROMA_SHARE: Double = 0.55

/** Search match fill in the dark scheme: lightness share moved toward black. */
private const val MATCH_DARK_LIGHTNESS_SHARE: Double = 0.62

/** Search match fill in the dark scheme: chroma share removed. */
private const val MATCH_DARK_CHROMA_SHARE: Double = 0.35

// What:     `private const val LIGHTNESS_FIT_STEPS: Int = 40` is the number of halvings in the contrast fit.
// Why:      Forty halvings pin the passing lightness far below one 8-bit step.
//
// In TS you'd write (pseudocode):
// ```ts
// const LIGHTNESS_FIT_STEPS = 40;
// ```
/** Number of halvings of the lightness interval in the contrast fit. */
private const val LIGHTNESS_FIT_STEPS: Int = 40

// What:     `internal enum class ThemeScheme { DARK, LIGHT }` names the two schemes.
// Why:      The two schemes use different lightness targets and move text in opposite directions.
//
// In TS you'd write (pseudocode):
// ```ts
// type ThemeScheme = "dark" | "light";
// ```
/** The two color schemes the roles are derived for. */
internal enum class ThemeScheme {
    /** Dark scheme, true-black canvas. */
    DARK,

    /** Light scheme, white canvas. */
    LIGHT,
}

// What:     `internal data class RoleSet` holds every color role of one scheme as finished sRGB.
// Why:      The theme receives one of these per scheme; field names mirror the Rust `RoleSet` field for field.
//
// In TS you'd write (pseudocode):
// ```ts
// type RoleSet = { canvas: Srgb8; deck: Srgb8; /* ... one field per role ... */ };
// ```
/** Every color role of one scheme as opaque 8-bit sRGB. */
internal data class RoleSet(
    /** Page background. */
    val canvas: Srgb8,
    /** Playback deck background. */
    val deck: Srgb8,
    /** Background of the playing row. */
    val currentRow: Srgb8,
    /** Menus, search header and snackbar surface. */
    val surfaceHigh: Srgb8,
    /** Primary text and icons. */
    val onSurface: Srgb8,
    /** Secondary text and icons. */
    val onSurfaceVariant: Srgb8,
    /** Borders and strong dividers. */
    val outline: Srgb8,
    /** Hairline dividers. */
    val outlineVariant: Srgb8,
    /** Accent fill and accent text. */
    val primary: Srgb8,
    /** Content on the accent fill. */
    val onPrimary: Srgb8,
    /** Selected segment and rail cell fill. */
    val secondaryContainer: Srgb8,
    /** Selected fill under the hover layer. */
    val secondaryContainerHovered: Srgb8,
    /** Selected fill under the pressed layer. */
    val secondaryContainerPressed: Srgb8,
    /** Content on the selected fill. */
    val onSecondaryContainer: Srgb8,
    /** Search match highlight (D59), opaque. */
    val matchFill: Srgb8,
    /** Error text and icons. */
    val error: Srgb8,
    /** Error card fill. */
    val errorContainer: Srgb8,
    /** Content on the error card. */
    val onErrorContainer: Srgb8,
) {
    /** Lists every role with its name in declaration order, the order of the shared test vector. */
    fun toNamedList(): List<Pair<String, Srgb8>> = listOf(
        "canvas" to canvas,
        "deck" to deck,
        "currentRow" to currentRow,
        "surfaceHigh" to surfaceHigh,
        "onSurface" to onSurface,
        "onSurfaceVariant" to onSurfaceVariant,
        "outline" to outline,
        "outlineVariant" to outlineVariant,
        "primary" to primary,
        "onPrimary" to onPrimary,
        "secondaryContainer" to secondaryContainer,
        "secondaryContainerHovered" to secondaryContainerHovered,
        "secondaryContainerPressed" to secondaryContainerPressed,
        "onSecondaryContainer" to onSecondaryContainer,
        "matchFill" to matchFill,
        "error" to error,
        "errorContainer" to errorContainer,
        "onErrorContainer" to onErrorContainer,
    )
}

// What:     `internal data class ThemePalette` holds both schemes derived from one root.
// Why:      The theme picks between them itself, so a scheme change needs no re-derivation.
//
// In TS you'd write (pseudocode):
// ```ts
// type ThemePalette = { dark: RoleSet; light: RoleSet };
// ```
/** Both schemes derived from one root. */
internal data class ThemePalette(
    /** Roles of the dark scheme. */
    val dark: RoleSet,
    /** Roles of the light scheme. */
    val light: RoleSet,
)

// What:     `private data class RoleSpec` says where one role sits relative to the root, measured at the baseline root.
// Why:      `chroma` is the role's chroma at the baseline root and `hueOffset` its hue minus the root hue in degrees,
//           so the role keeps its relation to the root for any accent.
//
// In TS you'd write (pseudocode):
// ```ts
// type RoleSpec = { lightness: number; chroma: number; hueOffset: number };
// ```
/** Lightness, baseline chroma and hue offset of one derived role. */
private data class RoleSpec(
    /** OKLCH lightness target, 0 to 1. */
    val lightness: Double,
    /** Chroma of the role when the root is the baseline. */
    val chroma: Double,
    /** Role hue minus root hue, in degrees. */
    val hueOffset: Double,
)

// What:     `private sealed interface Role` says how one role is obtained: fixed, neutral or accent.
// Why:      `Fixed` ignores the root (pure black, pure white and the semantic error colors); `Neutral` roles are the
//           grays: they take the root hue at their own tint, never shrunk by a weak root and never grown by a vivid one
//           (D118), and are true grays only when the root is completely neutral; `Accent` roles scale their chroma with
//           the root chroma.
//
// In TS you'd write (pseudocode):
// ```ts
// type Role = { fixed: Srgb8 } | { neutral: RoleSpec } | { accent: RoleSpec };
// ```
/** How one role of a scheme is obtained. */
private sealed interface Role {
    /** A color used exactly as written, whatever the root. */
    data class Fixed(
        /** The color. */
        val color: Srgb8,
    ) : Role

    /** A gray: the root hue at the role's own chroma, or chroma 0 when the root is completely neutral. */
    data class Neutral(
        /** Target relative to the root. */
        val spec: RoleSpec,
    ) : Role

    /** A role whose chroma scales with the root chroma in both directions. */
    data class Accent(
        /** Target relative to the root. */
        val spec: RoleSpec,
    ) : Role
}

// What:     `neutral`, `accent` and `fixed` build one role-table row each.
// Why:      They keep each row of the two role tables on one line.
//
// In TS you'd write (pseudocode):
// ```ts
// const neutral = (lightness, chroma, hueOffset) => ({ neutral: { lightness, chroma, hueOffset } });
// ```
/** Builds a `Role.Neutral` from the three numbers of a table row. */
private fun neutral(lightness: Double, chroma: Double, hueOffset: Double): Role =
    Role.Neutral(RoleSpec(lightness = lightness, chroma = chroma, hueOffset = hueOffset))

/** Builds a `Role.Accent` from the three numbers of a table row. */
private fun accent(lightness: Double, chroma: Double, hueOffset: Double): Role =
    Role.Accent(RoleSpec(lightness = lightness, chroma = chroma, hueOffset = hueOffset))

/** Builds a `Role.Fixed` from a packed RGB integer. */
private fun fixed(rgb: Int): Role = Role.Fixed(Srgb8.fromRgbInt(rgb))

// What:     `private data class SchemeSpec` says how each table-driven role of one scheme is obtained.
// Why:      The two constants below are the only place lightness targets live.
//
// In TS you'd write (pseudocode):
// ```ts
// type SchemeSpec = Record<"canvas" | "deck" | /* ... */ "onErrorContainer", Role>;
// ```
/** How each table-driven role of one scheme is obtained. */
private data class SchemeSpec(
    /** Page background. */
    val canvas: Role,
    /** Deck background. */
    val deck: Role,
    /** Playing row background. */
    val currentRow: Role,
    /** Menu surface. */
    val surfaceHigh: Role,
    /** Primary content. */
    val onSurface: Role,
    /** Secondary content. */
    val onSurfaceVariant: Role,
    /** Border. */
    val outline: Role,
    /** Hairline divider. */
    val outlineVariant: Role,
    /** Accent. */
    val primary: Role,
    /** Content on the accent. */
    val onPrimary: Role,
    /** Selected fill. */
    val secondaryContainer: Role,
    /** Content on the selected fill. */
    val onSecondaryContainer: Role,
    /** Error text. */
    val error: Role,
    /** Error card fill. */
    val errorContainer: Role,
    /** Content on the error card. */
    val onErrorContainer: Role,
)

// Targets fitted to the Fold captures (the baseline root reproduces them): the player held dark primary #BAC5EE, light
// primary #4E5E8B, secondary container #353B4D and #DDE2F9, and so on. Two light roles were then retuned darker in
// lightness only (B4): primary from 0.4883 to 0.4545 and on-surface-variant from 0.4844 to 0.4685, the smallest steps
// that put every drawn state at AA at the baseline. The numbers are copied from the Rust tables.
/**
 * Dark scheme table. D41, D118: the canvas is pure black under every accent (black is not a gray and, at lightness
 * 0, no chroma can show). The deck is a gray: the baseline reproduces the old literal `#0A0A0D` exactly, and every
 * other root gives it its own hue at the same tint.
 */
private val DARK_SPEC: SchemeSpec = SchemeSpec(
    canvas = fixed(0x000000),
    deck = neutral(lightness = 0.1463, chroma = 0.0064, hueOffset = 17.0),
    currentRow = neutral(lightness = 0.1879, chroma = 0.0103, hueOffset = 7.9),
    surfaceHigh = neutral(lightness = 0.2408, chroma = 0.0115, hueOffset = 9.4),
    onSurface = neutral(lightness = 0.9244, chroma = 0.0149, hueOffset = 13.4),
    onSurfaceVariant = neutral(lightness = 0.8288, chroma = 0.0139, hueOffset = 12.9),
    outline = neutral(lightness = 0.6546, chroma = 0.0135, hueOffset = 7.4),
    outlineVariant = neutral(lightness = 0.3957, chroma = 0.0153, hueOffset = 7.1),
    primary = accent(lightness = 0.8286, chroma = 0.0589, hueOffset = 4.7),
    onPrimary = accent(lightness = 0.3053, chroma = 0.0695, hueOffset = -1.9),
    secondaryContainer = accent(lightness = 0.3544, chroma = 0.0324, hueOffset = 1.8),
    onSecondaryContainer = accent(lightness = 0.9159, chroma = 0.0317, hueOffset = 7.4),
    error = fixed(0xFFB4AB),
    errorContainer = fixed(0x8C1D18),
    onErrorContainer = fixed(0xF9DEDC),
)

/** Light scheme table. D34, D45: a flat white canvas with hairline dividers; on-primary is pure white (chroma 0). */
private val LIGHT_SPEC: SchemeSpec = SchemeSpec(
    canvas = fixed(0xFFFFFF),
    deck = neutral(lightness = 0.9669, chroma = 0.0093, hueOffset = 24.2),
    currentRow = neutral(lightness = 0.9669, chroma = 0.0093, hueOffset = 24.2),
    surfaceHigh = neutral(lightness = 0.9308, chroma = 0.0094, hueOffset = 24.2),
    onSurface = neutral(lightness = 0.3188, chroma = 0.0162, hueOffset = 7.0),
    onSurfaceVariant = neutral(lightness = 0.4685, chroma = 0.0193, hueOffset = 9.6),
    outline = neutral(lightness = 0.5706, chroma = 0.0140, hueOffset = 7.3),
    outlineVariant = neutral(lightness = 0.8288, chroma = 0.0139, hueOffset = 12.9),
    primary = accent(lightness = 0.4545, chroma = 0.0753, hueOffset = 0.0),
    onPrimary = accent(lightness = 1.0, chroma = 0.0, hueOffset = 0.0),
    secondaryContainer = accent(lightness = 0.9159, chroma = 0.0317, hueOffset = 7.4),
    onSecondaryContainer = accent(lightness = 0.2260, chroma = 0.0341, hueOffset = 2.4),
    error = fixed(0xBA1A1A),
    errorContainer = fixed(0xF9DEDC),
    onErrorContainer = fixed(0x410E0B),
)

// What:     `internal data class RootTerms` holds what the derivation takes from the root.
// Why:      `hue` places every role on the color wheel, `scale` is the root chroma relative to the baseline's,
//           capped by `ROOT_CHROMA_CAP`, and `completelyNeutral` (D118) is true only when the root's three 8-bit
//           channels are equal; the root's own lightness is never used, since each role has its own target.
//           `accentChromaFloor` is the least chroma an accent role may have: `rootTerms` leaves it 0 because it
//           depends on the scheme, and `deriveScheme` sets it to the scheme's largest gray tint unless the root is
//           completely neutral.
//
// In TS you'd write (pseudocode):
// ```ts
// type RootTerms = { hue: number; scale: number; completelyNeutral: boolean; accentChromaFloor: number };
// ```
/** The hue, the capped relative chroma, the neutrality and the accent chroma floor of the derivation. */
internal data class RootTerms(
    /** Root hue in degrees, 0 up to 360. */
    val hue: Double,
    /** Root chroma over the baseline chroma, capped; exactly 0 for a completely neutral root. */
    val scale: Double,
    /** True only when the root's red, green and blue are equal; then every gray role has chroma 0. */
    val completelyNeutral: Boolean,
    /** Least chroma of any accent role; set per scheme by `deriveScheme`, 0 from `rootTerms`. */
    val accentChromaFloor: Double,
)

// What:     `internal fun rootTerms(root)` reads the root; null (no system accent) is the baseline.
// Why:      D118: a root is completely neutral only when its three 8-bit channels are equal, which is exactly
//           "no chroma at all" for a stored color. Equality is the test, not a chroma threshold: the 256 equal-channel
//           roots measure at most 3.7e-8 in OKLCH chroma (float noise) and the least chromatic unequal root, #FEFFFF,
//           measures 1.06e-3, so any threshold between them agrees with equality, and equality needs no tuned
//           constant that Rust and Kotlin could disagree on. A neutral root has no hue (the baseline hue is kept so
//           the value is deterministic) and scale 0; any other root, however weakly chromatic, keeps its own hue.
//
// In TS you'd write (pseudocode):
// ```ts
// const used = root ?? BASELINE_ROOT;
// const neutral = used.red === used.green && used.green === used.blue;
// if (neutral) return { hue: baselineHue, scale: 0, completelyNeutral: true };
// const lch = toOklch(used);
// return { hue: lch.hue, scale: Math.min(lch.chroma, 0.15) / baselineChroma, completelyNeutral: false };
// ```
/** Reads the hue, the relative chroma and the neutrality of the root, with the baseline as the default root. */
internal fun rootTerms(root: Srgb8?): RootTerms {
    /** The baseline root in OKLCH. */
    val baseline: Oklch = srgbToOklch(BASELINE_ROOT)
    /** The root, or the baseline when the system exposes no accent. */
    val used: Srgb8 = root ?: BASELINE_ROOT
    if (used.red == used.green && used.green == used.blue) {
        return RootTerms(hue = baseline.hue, scale = 0.0, completelyNeutral = true, accentChromaFloor = 0.0)
    }
    /** The root in OKLCH. */
    val lch: Oklch = srgbToOklch(used)
    return RootTerms(
        hue = lch.hue,
        scale = minOf(lch.chroma, ROOT_CHROMA_CAP) / baseline.chroma,
        completelyNeutral = false,
        accentChromaFloor = 0.0,
    )
}

// What:     `private fun resolveOklch(role, terms)` is the role's target color before gamut mapping.
// Why:      `Neutral` chroma is the role's own tint for every root that is not completely neutral (D118: a weak root
//           does not shrink it and a vivid root does not grow it) and 0 for a completely neutral root; `Accent`
//           chroma scales with the root in both directions but never falls below `accentChromaFloor`, so for a root
//           with any chroma an accent is never grayer than the grays.
//
// In TS you'd write (pseudocode):
// ```ts
// neutral: { lightness, chroma: terms.completelyNeutral ? 0 : chroma, hue: (rootHue + hueOffset) % 360 }
// accent:  { lightness, chroma: Math.max(chroma * terms.scale, terms.accentChromaFloor), hue: ... }
// ```
/** Resolves a role to its OKLCH target for this root. */
private fun resolveOklch(role: Role, terms: RootTerms): Oklch = when (role) {
    is Role.Fixed -> srgbToOklch(role.color)
    is Role.Neutral -> Oklch(
        lightness = role.spec.lightness,
        chroma = if (terms.completelyNeutral) 0.0 else role.spec.chroma,
        hue = wrapHue(terms.hue + role.spec.hueOffset),
    )
    is Role.Accent -> Oklch(
        lightness = role.spec.lightness,
        chroma = maxOf(role.spec.chroma * terms.scale, terms.accentChromaFloor),
        hue = wrapHue(terms.hue + role.spec.hueOffset),
    )
}

// What:     `private fun resolveSrgb(role, terms)` is the finished color of a role without any contrast fit.
// Why:      `Fixed` colors are used as written (no OKLCH round trip), the rest go through `oklchToSrgb8`.
//
// In TS you'd write (pseudocode):
// ```ts
// return "fixed" in role ? role.fixed : oklchToSrgb8(resolveOklch(role, terms));
// ```
/** Resolves a role to 8-bit sRGB for this root. */
private fun resolveSrgb(role: Role, terms: RootTerms): Srgb8 {
    if (role is Role.Fixed) {
        return role.color
    }
    return oklchToSrgb8(resolveOklch(role, terms))
}

// What:     `internal fun fitLightness(start, endpoint, passes)` moves `start` in lightness toward `endpoint` (0
//           black or 1 white), keeping chroma and hue, until `passes` accepts the 8-bit result.
// Why:      This is the contrast guarantee: contrast depends on luminance, so lightness is the one coordinate moved,
//           and `passes` always sees the quantized color that reaches the screen. If even the endpoint fails, the
//           endpoint is returned, because no lightness can do better.
//
// In TS you'd write (pseudocode):
// ```ts
// if (passes(toSrgb8(start))) return start;
// let [failing, passing] = [start.lightness, endpoint];
// for (let i = 0; i < 40; i++) {
//   const mid = (failing + passing) / 2;
//   passes(toSrgb8({ ...start, lightness: mid })) ? (passing = mid) : (failing = mid);
// }
// return { ...start, lightness: passing };
// ```
/** Returns the accepted lightness closest to [start] on the way to [endpoint], keeping chroma and hue. */
internal fun fitLightness(start: Oklch, endpoint: Double, passes: (Srgb8) -> Boolean): Oklch {
    if (passes(oklchToSrgb8(start))) {
        return start
    }
    /** The start moved all the way to the endpoint. */
    val atEndpoint: Oklch = start.copy(lightness = endpoint)
    if (!passes(oklchToSrgb8(atEndpoint))) {
        return atEndpoint
    }
    /** Lightness known to fail. */
    var failing: Double = start.lightness
    /** Lightness known to pass. */
    var passing: Double = endpoint
    repeat(LIGHTNESS_FIT_STEPS) {
        /** Lightness halfway between the bounds. */
        val middle: Double = (failing + passing) / 2.0
        if (passes(oklchToSrgb8(start.copy(lightness = middle)))) {
            passing = middle
        } else {
            failing = middle
        }
    }
    return start.copy(lightness = passing)
}

// What:     `private fun legibleOn(content, backgrounds, layer, layerAlphas, target)` is true when `content` reaches
//           `target` against every background, and also against that background with `layer` at each of `layerAlphas`.
// Why:      Text is drawn on a surface and also under a hover, pressed or focus layer (alpha-only, D108); the caller
//           passes every layer strength the text must survive.
//
// In TS you'd write (pseudocode):
// ```ts
// return backgrounds.every(b => ratio(content, b) >= target
//   && alphas.every(a => ratio(content, compositeOver(layer, a, b)) >= target));
// ```
/** True when [content] holds the text target on every background, at rest and under the layer at every strength. */
private fun legibleOn(content: Srgb8, backgrounds: List<Srgb8>, layer: Srgb8, layerAlphas: List<Double>): Boolean =
    backgrounds.all { background ->
        contrastRatio(content, background) >= TEXT_TARGET_RATIO &&
            layerAlphas.all { layerAlpha ->
                contrastRatio(content, compositeOver(layer, layerAlpha, background)) >= TEXT_TARGET_RATIO
            }
    }

// What:     `private fun containerLayers(container, content)` is the selected fill under the hover and under the
//           pressed state, moved from the container toward its content color in OKLCH.
// Why:      Replaces the sRGB `mix` the segmented button used; the move is in lightness, chroma and hue.
//
// In TS you'd write (pseudocode):
// ```ts
// return [toSrgb8(mixOklch(c, on, 0.08)), toSrgb8(mixOklch(c, on, 0.12))];
// ```
/** Returns the container color under the hover layer and under the pressed layer, in that order. */
private fun containerLayers(container: Srgb8, content: Srgb8): Pair<Srgb8, Srgb8> {
    /** Container in OKLCH. */
    val from: Oklch = srgbToOklch(container)
    /** Content in OKLCH. */
    val to: Oklch = srgbToOklch(content)
    return Pair(
        oklchToSrgb8(mixOklch(from, to, HOVER_ALPHA)),
        oklchToSrgb8(mixOklch(from, to, PRESSED_ALPHA)),
    )
}

// What:     `private data class FixedSurfaces` holds the surfaces and the selected fill, which no contrast fit moves.
// Why:      The text fits need them as backgrounds, so they are resolved once and passed along.
//
// In TS you'd write (pseudocode):
// ```ts
// type FixedSurfaces = { canvas; deck; currentRow; surfaceHigh; secondaryContainer };
// ```
/** The surfaces and fills of one scheme that no contrast fit moves, resolved from the table. */
private data class FixedSurfaces(
    /** Page background. */
    val canvas: Srgb8,
    /** Deck background. */
    val deck: Srgb8,
    /** Playing row background. */
    val currentRow: Srgb8,
    /** Menu surface. */
    val surfaceHigh: Srgb8,
    /** Selected fill. */
    val secondaryContainer: Srgb8,
) {
    /** The surfaces text is drawn on. Settings has no surface of its own (D109), so the canvas stands for it. */
    val textSurfaces: List<Srgb8> get() = listOf(canvas, deck, currentRow, surfaceHigh)
}

// What:     `private data class FittedText` holds the neutral text roles after the contrast fit.
// Why:      The accent fit needs the fitted on-surface color, so the result travels as one value.
//
// In TS you'd write (pseudocode):
// ```ts
// type FittedText = { onSecondaryContainer; onSurface; onSurfaceVariant; outline };
// ```
/** The text roles of one scheme after the contrast fit. */
private data class FittedText(
    /** Content on the selected fill. */
    val onSecondaryContainer: Srgb8,
    /** Primary text. */
    val onSurface: Srgb8,
    /** Secondary text. */
    val onSurfaceVariant: Srgb8,
    /** Border color. */
    val outline: Srgb8,
)

// What:     `private fun fitText(...)` fits the neutral text roles and the outline against the surfaces.
// Why:      Each role moves in lightness only, toward the text end, until it holds its ratio on every surface and
//           state layer.
//
// In TS you'd write (pseudocode):
// ```ts
// const fitText = (spec, terms, surfaces, towardText) => ({ ... });
// ```
/** Fits the neutral text roles and the outline of one scheme against the resolved surfaces. */
private fun fitText(spec: SchemeSpec, terms: RootTerms, surfaces: FixedSurfaces, towardText: Double): FittedText {
    /** On-secondary-container after the contrast fit. */
    val onContainerFit: Oklch = fitLightness(resolveOklch(spec.onSecondaryContainer, terms), towardText) { candidate ->
        val (hovered, pressed) = containerLayers(surfaces.secondaryContainer, candidate)
        contrastRatio(candidate, surfaces.secondaryContainer) >= TEXT_TARGET_RATIO &&
            contrastRatio(candidate, hovered) >= TEXT_TARGET_RATIO &&
            contrastRatio(candidate, pressed) >= TEXT_TARGET_RATIO
    }
    /** Primary text color. */
    val onSurface: Srgb8 = oklchToSrgb8(
        fitLightness(resolveOklch(spec.onSurface, terms), towardText) { candidate ->
            legibleOn(candidate, surfaces.textSurfaces, candidate, TEXT_LAYER_ALPHAS)
        },
    )
    /** Secondary text after the contrast fit. */
    val onSurfaceVariant: Srgb8 = oklchToSrgb8(
        fitLightness(resolveOklch(spec.onSurfaceVariant, terms), towardText) { candidate ->
            legibleOn(candidate, surfaces.textSurfaces, onSurface, TEXT_LAYER_ALPHAS)
        },
    )
    /** Outline after the contrast fit. */
    val outline: Srgb8 = oklchToSrgb8(
        fitLightness(resolveOklch(spec.outline, terms), towardText) { candidate ->
            contrastRatio(candidate, surfaces.canvas) >= NON_TEXT_TARGET_RATIO &&
                contrastRatio(candidate, surfaces.deck) >= NON_TEXT_TARGET_RATIO
        },
    )
    return FittedText(
        onSecondaryContainer = oklchToSrgb8(onContainerFit),
        onSurface = onSurface,
        onSurfaceVariant = onSurfaceVariant,
        outline = outline,
    )
}

// What:     `private data class FittedAccent` holds primary and on-primary after the contrast fit.
// Why:      The match fill and the tests read both together.
//
// In TS you'd write (pseudocode):
// ```ts
// type FittedAccent = { primary; onPrimary };
// ```
/** The accent fill and its content after the contrast fit. */
private data class FittedAccent(
    /** Accent fill and accent text. */
    val primary: Srgb8,
    /** Content on the accent fill. */
    val onPrimary: Srgb8,
)

// Primary is both a fill and text (rail letter, links, action labels), so it holds AA on every surface and under the
// hover and pressed (also focus) layers, on the selected fill at rest and under the layers the rail cell draws over it,
// and against the light scheme's white on-primary. The OKLCH hovered and pressed fills carry only
// on-secondary-container text, so primary is not fitted to them.
/** Fits primary and on-primary against the surfaces and the already fitted neutral text. */
private fun fitAccent(
    spec: SchemeSpec,
    terms: RootTerms,
    surfaces: FixedSurfaces,
    text: FittedText,
    scheme: ThemeScheme,
): FittedAccent {
    /** Fitted on-surface, the tint of the layers over the surfaces. */
    val onSurface: Srgb8 = text.onSurface
    /** The selected fill at rest and under the hover and pressed layers of on-secondary-container. */
    val selectedFillStates: List<Srgb8> = listOf(
        surfaces.secondaryContainer,
        compositeOver(text.onSecondaryContainer, HOVER_ALPHA, surfaces.secondaryContainer),
        compositeOver(text.onSecondaryContainer, PRESSED_ALPHA, surfaces.secondaryContainer),
    )
    /** Lightness endpoint that text moves toward. */
    val towardText: Double = if (scheme == ThemeScheme.DARK) 1.0 else 0.0
    /** Fixed white on-primary of the light scheme, or null in dark. */
    val lightOnPrimary: Srgb8? = if (scheme == ThemeScheme.DARK) null else resolveSrgb(spec.onPrimary, terms)
    /** Accent fill color. */
    val primary: Srgb8 = oklchToSrgb8(
        fitLightness(resolveOklch(spec.primary, terms), towardText) { candidate ->
            legibleOn(candidate, surfaces.textSurfaces, onSurface, TEXT_LAYER_ALPHAS) &&
                selectedFillStates.all { fill -> contrastRatio(candidate, fill) >= TEXT_TARGET_RATIO } &&
                (lightOnPrimary == null || contrastRatio(candidate, lightOnPrimary) >= TEXT_TARGET_RATIO)
        },
    )
    // On-primary is fitted against the primary fill and its own pressed layer (the first-run button).
    /** Content color on the accent fill. */
    val onPrimary: Srgb8 = lightOnPrimary ?: oklchToSrgb8(
        fitLightness(resolveOklch(spec.onPrimary, terms), 1.0 - towardText) { candidate ->
            legibleOn(candidate, listOf(primary), candidate, TEXT_LAYER_ALPHAS)
        },
    )
    return FittedAccent(primary = primary, onPrimary = onPrimary)
}

// Search match fill (D59): the primary moved toward the neutral end in OKLCH, with the Android shares, then fitted so
// both text roles stay legible on it.
// What:     `private fun deriveMatchFill(...)` derives the opaque Search match highlight.
// Why:      D59 wants it derived from the accent in OKLCH, and the fit keeps both text roles legible on it. It is an
//           accent role, so its chroma never falls below the accent floor either.
//
// In TS you'd write (pseudocode):
// ```ts
// const deriveMatchFill = (scheme, primary, onSurface, onSurfaceVariant) => { ... };
// ```
/** Derives the opaque Search match highlight from the fitted primary and text roles. */
private fun deriveMatchFill(
    scheme: ThemeScheme,
    primary: Srgb8,
    texts: Pair<Srgb8, Srgb8>,
    accentChromaFloor: Double,
): Srgb8 {
    /** Primary text color. */
    val onSurface: Srgb8 = texts.first
    /** Secondary text color. */
    val onSurfaceVariant: Srgb8 = texts.second
    /** Whether this is the dark scheme. */
    val dark: Boolean = scheme == ThemeScheme.DARK
    /** Share of lightness moved toward the neutral end. */
    val lightnessShare: Double = if (dark) MATCH_DARK_LIGHTNESS_SHARE else MATCH_LIGHT_LIGHTNESS_SHARE
    /** Share of chroma removed. */
    val chromaShare: Double = if (dark) MATCH_DARK_CHROMA_SHARE else MATCH_LIGHT_CHROMA_SHARE
    /** Lightness of the neutral end. */
    val neutralEnd: Double = if (dark) 0.0 else 1.0
    /** Primary in OKLCH. */
    val primaryLch: Oklch = srgbToOklch(primary)
    /** Match fill before the contrast fit. */
    val matchStart = Oklch(
        lightness = primaryLch.lightness + (neutralEnd - primaryLch.lightness) * lightnessShare,
        chroma = maxOf(primaryLch.chroma * (1.0 - chromaShare), accentChromaFloor),
        hue = primaryLch.hue,
    )
    return oklchToSrgb8(
        fitLightness(matchStart, neutralEnd) { candidate ->
            contrastRatio(onSurface, candidate) >= TEXT_TARGET_RATIO &&
                contrastRatio(onSurfaceVariant, candidate) >= TEXT_TARGET_RATIO
        },
    )
}

// What:     `private fun grayTintCeiling(spec)` is the largest chroma among the gray (`Neutral`) rows of one scheme.
// Why:      The reading of D118 that an accent is never grayer than the grays needs one number per scheme: the
//           strongest tint any gray of that scheme carries (the dark scheme's is 0.0153, the light scheme's 0.0193).
//
// In TS you'd write (pseudocode):
// ```ts
// Math.max(...Object.values(spec).filter((role) => "neutral" in role).map((role) => role.neutral.chroma))
// ```
/** Returns the largest chroma among the gray rows of the scheme table. */
private fun grayTintCeiling(spec: SchemeSpec): Double {
    /** The rows that are grays; the canvas is fixed and carries no tint. */
    val rows: List<Role> = listOf(
        spec.deck,
        spec.currentRow,
        spec.surfaceHigh,
        spec.onSurface,
        spec.onSurfaceVariant,
        spec.outline,
        spec.outlineVariant,
    )
    return rows.filterIsInstance<Role.Neutral>().maxOf { row -> row.spec.chroma }
}

// What:     `internal fun deriveScheme(fromRoot, scheme)` derives every role of one scheme.
// Why:      Places each role from its target, then fits lightness so each text pair holds AA: content on its surface
//           and layers, accent text on every surface it is drawn on, outlines against the canvas and deck.
//
// In TS you'd write (pseudocode):
// ```ts
// const spec = scheme === "dark" ? DARK_SPEC : LIGHT_SPEC;
// const surfaces = [canvas, deck, currentRow, surfaceHigh].map(resolve);
// // fit onSecondaryContainer, onSurface, onSurfaceVariant, outline, primary, onPrimary, then derive matchFill
// ```
/** Derives every role of one scheme from the root terms [fromRoot]. */
internal fun deriveScheme(fromRoot: RootTerms, scheme: ThemeScheme): RoleSet {
    /** Role table of the scheme. */
    val spec: SchemeSpec = if (scheme == ThemeScheme.DARK) DARK_SPEC else LIGHT_SPEC
    // D118 reading: every accent role is at least as chromatic as the strongest gray of this scheme, unless the root
    // is completely neutral, where every accent stays at chroma 0.
    /** The root terms with this scheme's accent chroma floor filled in. */
    val terms: RootTerms = fromRoot.copy(
        accentChromaFloor = if (fromRoot.completelyNeutral) 0.0 else grayTintCeiling(spec),
    )
    // Text moves toward white on a dark scheme and toward black on a light one to gain contrast.
    /** Lightness endpoint that text moves toward. */
    val towardText: Double = if (scheme == ThemeScheme.DARK) 1.0 else 0.0
    /** Resolved surfaces and fills. */
    val surfaces = FixedSurfaces(
        canvas = resolveSrgb(spec.canvas, terms),
        deck = resolveSrgb(spec.deck, terms),
        currentRow = resolveSrgb(spec.currentRow, terms),
        surfaceHigh = resolveSrgb(spec.surfaceHigh, terms),
        secondaryContainer = resolveSrgb(spec.secondaryContainer, terms),
    )
    /** Fitted text roles. */
    val text: FittedText = fitText(spec, terms, surfaces, towardText)
    /** Fitted accent roles. */
    val accentFit: FittedAccent = fitAccent(spec, terms, surfaces, text, scheme)
    val (hovered, pressed) = containerLayers(surfaces.secondaryContainer, text.onSecondaryContainer)
    return RoleSet(
        canvas = surfaces.canvas,
        deck = surfaces.deck,
        currentRow = surfaces.currentRow,
        surfaceHigh = surfaces.surfaceHigh,
        onSurface = text.onSurface,
        onSurfaceVariant = text.onSurfaceVariant,
        outline = text.outline,
        outlineVariant = resolveSrgb(spec.outlineVariant, terms),
        primary = accentFit.primary,
        onPrimary = accentFit.onPrimary,
        secondaryContainer = surfaces.secondaryContainer,
        secondaryContainerHovered = hovered,
        secondaryContainerPressed = pressed,
        onSecondaryContainer = text.onSecondaryContainer,
        matchFill = deriveMatchFill(
            scheme = scheme,
            primary = accentFit.primary,
            texts = Pair(text.onSurface, text.onSurfaceVariant),
            accentChromaFloor = terms.accentChromaFloor,
        ),
        error = resolveSrgb(spec.error, terms),
        errorContainer = resolveSrgb(spec.errorContainer, terms),
        onErrorContainer = resolveSrgb(spec.onErrorContainer, terms),
    )
}

// What:     `internal fun deriveRoles(root)` derives both schemes from one root; null is the baseline.
// Why:      The single entry point the theme calls at startup and on every accent change.
//
// In TS you'd write (pseudocode):
// ```ts
// const terms = rootTerms(root);
// return { dark: deriveScheme(terms, "dark"), light: deriveScheme(terms, "light") };
// ```
/** Derives the dark and the light roles from [root], or from the baseline when [root] is null. */
internal fun deriveRoles(root: Srgb8?): ThemePalette {
    /** Hue and scale taken from the root. */
    val terms: RootTerms = rootTerms(root)
    return ThemePalette(
        dark = deriveScheme(terms, ThemeScheme.DARK),
        light = deriveScheme(terms, ThemeScheme.LIGHT),
    )
}

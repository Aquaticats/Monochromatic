# Android 17 dynamic-color captures can read a new setting while rendering the preceding fabricated overlay

## Symptom

A Pixel 9 Pro Fold Android 17 emulator was used to capture Compose dark-theme candidates
under several deterministic dynamic-color inputs.
 After writing a coral seed to
`theme_customization_overlay_packages`,
 reading the setting returned coral immediately,
but `cmd overlay dump com.android.systemui:dynamic` still returned the preceding blue
roles:

```text
theme_customization_overlay_packages={..."system_palette":"D45D42"...}
system_primary_dark = #B6C6EE
```

A later overlay dump returned the expected coral primary:

```text
system_primary_dark = #F9B7A8
```

A capture taken from setting readback alone can therefore carry the wrong Android role
palette while retaining a plausible file name.

## Root cause

`Settings.Secure` stores the requested customization JSON.
 SystemUI observes that setting,
generates a color scheme,
 creates fabricated overlays,
 and then applies them to Android
resources.
 Those are separate operations.
 Successful setting readback proves only the first
operation completed.

Android 17 source
`packages/SystemUI/src/com/android/systemui/theme/ThemeOverlayController.java`
registers a content observer for
`THEME_CUSTOMIZATION_OVERLAY_PACKAGES`,
 calls
`reevaluateSystemTheme(true)`,
 constructs generated accent,
 neutral,
 and dynamic
fabricated overlays,
 then applies them through `ThemeOverlayApplier`.
 Its diagnostic log prints
`Writing boot animation colors 1:` after resolving the current scheme.

Compose Material 3 does not read the customization JSON.
 Resolved artifact
`androidx.compose.material3:material3-android:1.5.0-alpha27` implements
`dynamicDarkColorScheme(context)` in
`androidx/compose/material3/DynamicTonalPalette.android.kt:217-225`.
 On API 34 and
newer it calls `dynamicDarkColorScheme34(context)`,
 which reads Android's
`mc3_dark_scheme` styled resources.
 Compose therefore sees the fabricated resource overlay
only after SystemUI finishes applying it.

The target also exposes user contrast independently of seed and style.
 Android SDK 37
source `android/app/UiModeManager.java:730-781` defines Default,
 Medium,
 and High as
0.0,
 0.5,
 and 1.0.
 Both the contrast value and palette style must settle before
the screenshot represents the requested environment.

## Verification

Measured environment:

- Android Emulator 37.1.11.0,
  build 15917651.
- Android 17 API 37 Pixel 9 Pro Fold image.
- Compose Material 3 1.5.0-alpha27,
  verified through Gradle `dependencyInsight`.
- AndroidX source artifact:
  `material3-android-1.5.0-alpha27-sources.jar` from Google's Maven repository.
- AndroidX comparison clone:
  `/home/user/temp/agent/androidx-2026-09-09`,
  commit `f6aa13de4610351b83575f0653ff1fccbef5600c`.

The verified harness is preserved on prototype branch
`prototype/music-player-theme-compose` in
`package/music-player/android-app/capture-dark-dynamic.mjs`.
 The owning command is:

```text
ANDROID_SERIAL=emulator-5564 \
mise run //package/music-player/android-app:prototype:capture:dark
```

That address is not on `main`:
branch `prototype/music-player-theme-compose` declares `prototype:capture:dark`
on 2026-09-09 (commit `6c9f10f91f`),
and `main`'s `package/music-player/android-app/mise.toml` has no `prototype:*` task.

It records requested seed,
 style,
 contrast,
 and every settled dark resource role under
`package/music-player/design/question/evidence/dark-dynamic-*-roles.json`.

### Working catalog

- Clear logcat before requesting one environment.
- Write both `contrast_level` and customization JSON.
- Wait until SystemUI reports the requested `mContrast` and numeric `mThemeStyle`.
- Wait until a fresh `ThemeOverlayController` log contains
  `Writing boot animation colors 1:`.
- Read `com.android.systemui:dynamic` and require resolved `primary` plus
  `surface_container_low` roles.
- Launch the candidate only after those checks pass.
- Wait until UI Automator sees both `Camellia` and `Repeat track` before capture.
- Capture explicit unfolded HWC display 0 at 2076 × 2152px.

The task produced distinct blue,
 coral,
 green,
 gold,
 magenta,
 and monochrome role
records plus thirty-six corresponding opaque panel captures for the separated 2 × 3
candidate cross-product.

### Failing catalog

- Setting readback followed immediately by overlay dump returned a preceding palette.
- Requiring visible text `Repeat` in UI Automator never passed because the mode control
  exposes its complete accessible name as `Repeat track`.
- `killall com.android.systemui` as the shell user returned
  `Operation not permitted`.
- `adb root` returned
  `adbd cannot run as root in production builds` on this image.
- Capturing without an explicit display ID prefixed a multiple-display warning to
  stdout and corrupted the supposed PNG byte stream.

## Verified workarounds

### Wait for SystemUI evidence rather than elapsed time

Use the requested SystemUI style,
 contrast,
 fresh controller log,
 and resolved overlay
roles as readiness conditions.
 Tradeoff:
 each palette capture takes longer than setting
readback,
 but it proves the consumed resources rather than assuming observer latency.

### Capture one explicit emulator display

Resolve the line containing `(HWC display 0)` from
`dumpsys SurfaceFlinger --display-id`,
 then pass that identifier to `screencap -d`.
Tradeoff:
 the identifier is runtime-specific and must not be hardcoded across boots.

### Preserve generated role evidence beside rasters

Save the full `com.android.systemui:dynamic` mapping and requested environment beside
each screenshot group.
 Tradeoff:
 evidence volume grows,
 but later review can distinguish
an app mapping change from an Android palette change.

## What does not work

- `settings get secure theme_customization_overlay_packages` is not a readiness probe.
- A fixed sleep cannot prove resource application and can either race or waste time.
- Killing SystemUI is unavailable to the shell user on this production emulator image.
- `adb root` cannot grant that permission on the production image.
- An omitted `screencap -d` is unsafe on the unfolded emulator because multiple displays
  are registered.
- Checking visible label text alone can miss merged Compose accessibility names.

## Upstream filing decision

1.  **Is it really upstream's fault?**
    No.
    SystemUI applies observed settings
    asynchronously,
    and Compose correctly reads resolved Android resources.
2.  **Can upstream fix it?**
    No upstream change is needed;
    the capture consumer must wait for
    the resource boundary it depends on.
3.  **Are they supporting this use case?**
    Dynamic resources and diagnostic dumps are
    supported surfaces;
    direct customization-setting injection is a local test technique.
4.  **Would the repositories welcome our contribution?**
    Not evaluated because no defect
    remains after consumer-side synchronization.
5.  **Will they likely fix it?**
    Not applicable.
6.  **Have we prototyped a minimal fix?**
    Yes.
    The verified capture task waits on SystemUI,
    resolved roles,
    Compose content,
    and an explicit display.

No matching exemption exists under `.out-of-scope/`.
 There is no upstream issue to file.

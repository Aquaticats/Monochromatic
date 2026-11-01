# Device metrics — Pixel 9 Pro Fold

**Do not invent these numbers,
 and do not put invented ones into question options.**
Both mistakes were made in phase one (see review-notes.md #2).

---

## Published hardware figures

<table>
<thead>
<tr>
<th></th>
<th>Cover screen</th>
<th>Inner (unfolded)</th>
</tr>
</thead>
<tbody>
<tr>
<td>Diagonal</td>
<td>6.3 in</td>
<td>8.0 in</td>
</tr>
<tr>
<td>Resolution</td>
<td>1080 × 2424 px</td>
<td>2076 × 2152 px</td>
</tr>
<tr>
<td>Aspect</td>
<td>20:9</td>
<td>1:1 (square)</td>
</tr>
<tr>
<td>Density</td>
<td>~422 ppi</td>
<td>~373 ppi</td>
</tr>
<tr>
<td>Panel</td>
<td>OLED, 120Hz</td>
<td>LTPO OLED, 1–120Hz</td>
</tr>
</tbody>
</table>

Physical:
 **155.2 × 150.2 × 5.1 mm unfolded**,
 **155.2 × 77.1 × 10.5 mm folded**,
 257 g.

---

## Converted to dp (what you design against)

```text
Published-pixel-density estimate for cover: 1080 / 2.625 × 2424 / 2.625 ≈ 411 × 923dp at 420dpi.
Measured Pixel_9_Pro_Fold AVD cover:     1080 / 2.4375 × 2424 / 2.4375 ≈ 443 × 994dp at 390dpi.
Measured Pixel_9_Pro_Fold AVD inner:     2076 / 2.4375 × 2152 / 2.4375 ≈ 852 × 883dp at 390dpi.
```

The original 411 × 923dp cover estimate came from published panel ppi,
 not Android
configuration.
 Direct `adb shell wm density` and both panel entries in
`dumpsys display` report **390dpi for both emulator displays**.
 The current
opaque Compose captures therefore render at approximately 443 × 994dp on the
cover,
 not 411 × 923dp.
 D49's physical resolutions remain unchanged.
 Do not
call an AVD capture 411dp wide or present it at 411 CSS px as "100% dp".
 An
unprobed physical handset could use a different logical density;
 this evidence
specifically describes the current AVD.

## Questionnaire frame geometry

Google's current hardware specification confirms an 8-inch inner display at
2076 × 2152px and an unfolded body measuring 150.2 × 155.2mm in width-first order.
The inner questionnaire frame uses those figures rather than treating the active
display as the outside of the phone.
 The cover frame originally used 411 × 923
dp for the same relative chassis proportions;
 its 100% AVD preview must scale
that cover body and screen to the measured 443 × 994dp while retaining the
published physical-pixel screenshot unmodified.

Distributing the 203.2mm display diagonal by the panel's pixel aspect gives an active
area of approximately 141.08 × 146.24mm.
 Applying the body-to-active-area ratios to
the 852 × 883dp design surface gives 907.09 × 937.08.
 The reusable frame therefore
uses these coordinates at 100%:

- 907 × 937 CSS px for the complete unfolded body.
- 852 × 883 CSS px for the active screen content.
- 27px start,
   28px end,
   and 27px block-axis chassis insets.
- 454 × 937 CSS px for a crop from the fold centre through the right body edge.
- 426px of physical right-half screen content and 28px of outer chassis in that crop.
  In the withdrawn 24dp-spacer prototype,
   this half contained 12dp of
  spacer and a 414dp right pane.
    E2 supersedes that fixed gap.

The user supplied eight current product references on 2026-09-04.
 The straight-on
`google-pixel-9-pro-fold-1.jpg` reference is 629 × 650px,
 an outer ratio of 0.9677.
Google's 150.2 × 155.2mm body is 0.9678.
 Measurements from that reference set the
current approximately 72dp outer corner,
 42dp screen corner,
 28dp inner-camera cutout,
and 44dp hinge cap.
 The other supplied views confirm that the inner camera is at the
top-right,
 the centre fold has hinge hardware only at the outer edges,
 and a right-half
study has no independent rounded left chassis.

The JPEG references carry a visible publisher watermark and are not embedded in the
questionnaire.
 They were used only for measurement and side-by-side inspection.
 The
CSS frame is opaque vector geometry;
 each source PNG remains rectangular screen
content placed inside its opening.

The later `gsmarena_052.jpg` reference shows the unfolded screen with Android system
UI.
 It confirms that time and notification icons occupy the left side,
 connectivity
and battery icons stop before the top-right inner camera,
 and the status bar is
transparent over screen content.
 It also carries a visible publisher watermark and is
not embedded.

Source:
 [Google Pixel phone hardware tech specs][pixel-hardware-specs].

[pixel-hardware-specs]: https://support.google.com/pixelphone/answer/7158570?hl=en
[emulator-hardware-properties]:
  https://android.googlesource.com/platform/prebuilts/android-emulator/+/refs/heads/main/linux-x86_64/lib/hardware-properties.ini

---

## Android system UI geometry

A locally installed Pixel 9 Pro Fold emulator provides the user-boundary check.
 The
probe used Android 17,
 API 37,
 build `CE2A.260420.019`,
 at the AVD's unfolded posture.
`adb shell wm size` reports `2076x2152`;
 `adb shell wm density` reports `390`,
 or
`2.4375` physical pixels per dp.

The emulator's live `WindowInsetsStateController` reports:

- Status-bar frame:
   `[0,0][2076,88]`,
   approximately 36dp high.
- Gesture-navigation frame:
   `[0,2074][2076,2152]`,
   exactly 78px or 32dp high.
- Display-cutout bounding rectangle:
   `[1940,0][2076,136]`,
   which requires about 56dp
  of top safe inset where app controls could otherwise meet the camera.

Live resource lookup confirms `status_bar_height_portrait = 36dp` and
`navigation_bar_gesture_height = 32dp`.
 Compose candidates must therefore use native
`WindowInsets.safeDrawing` rather than hard-coded 36dp and 32dp padding:
 the combined
safe inset also accounts for the deeper 136px camera bounding rectangle.

The device overlay independently supplies the target geometry.
 At LineageOS
`android_device_google_comet` commit `dbc4a6cc10414e004fdd24641b0182e9eaf2f5c6`,
`overlay/FrameworkResOverlayVendorComet/res/values-sw820dp/dimens.xml:9-22` sets all
inner-display status-bar heights to 36dp and rounded-corner content padding to 32dp.
`overlay/FrameworkResOverlayVendorComet/res/values/config.xml:75-83` defines the inner
camera as a 79px-diameter circle centred at `(1987.5, 80)` and its safe bounding
rectangle as the top-right 136 × 136px region.

At 390dpi,
 that physical camera path is approximately 32.41dp in diameter,
 centred at
`(815.38dp, 32.82dp)`.
 Its edge sits approximately 20.41dp from the right screen edge.
The questionnaire's hardware overlay uses these values instead of the earlier
photograph estimate of 28dp.

A live Settings screenshot confirms current Pixel large-screen rendering.
 Its gesture
handle has a 536 × 10px solid core,
 approximately 220 × 4dp,
 with about 14dp between
its solid lower edge and the screen edge.
 This differs from the base SystemUI
`108 × 4dp` phone handle resource because the unfolded large-screen taskbar owns the
rendered handle.
 Do not reproduce either resource manually:
 emulator screenshots own
all system-bar pixels.

Android `screencap` does not paint the physical camera hole into its PNG.
 The Settings
capture's pixel at the configured camera centre remains the app background even though
Window Manager reports the cutout.
 The questionnaire must therefore keep one hardware
camera overlay in its measured frame while removing any simulated system bars.

## Dynamic-color capture state

The Material-compliance recapture uses Android's live `dynamicLightColorScheme`.
Immediately before capture,
 the read-only Pixel 9 Pro Fold AVD reported:

- `settings get secure theme_customization_overlay_packages`:
   `null`.
- `cmd uimode night`:
   `Night mode: no`.
- System wallpaper:
   ID 0,
   SystemUI `ImageWallpaper`,
   no name,
   and
  `isColorExtracted=false`.
- Fallback wallpaper:
   ID 1,
   SystemUI `GradientColorWallpaper`.
- `com.android.systemui:neutral`,
   `:accent`,
   and `:dynamic` overlays:
   disabled.

This pins the source to the stock AVD light resources rather than a personal wallpaper
or manually selected palette.
 Pixel probes from the resulting `light-a/b/c` captures
resolve the role mapping as follows:

```text
surface                    #FAF8FE
surface-container-low      #F3F3FA
surface-container          #EDEDF6
surface-container-high     #E7E7F1
surface-container-lowest   #FFFFFF
surface-dim                #D8D9E4
primary-container          #B9CBFF
```

The captures test role use rather than hard-code these values into the Compose source.
A future capture must first confirm the same AVD theme state or deliberately update the
record and role-based guards.

---

## Layout implications (these are the point)

1. **The inner display is essentially square and slightly taller than wide.**
    It is
   not a landscape tablet.
    Every mockup drawn at 924×600 was wrong in structure.
2. **The hinge is vertical in portrait.**
    Each physical screen half is
   **426 × 883dp** at the measured AVD density.
    The previous player
   prototype placed two 414dp panes around a fixed 24dp blank stripe.
    E2
   replaces that mistaken **informational clearance** with
   `max(min_padding, crease_width)` in physical space.
    It does not require
   a blank surface gap or force symmetric visual pane widths;
    Search need
   not divide its query and results between the halves.
3. **Informational material stays visibly clear of the physical crease**
   (E2).
    Its center is about 426dp,
 physical x 1038 on the 2076px inner
   panel.
    The old 24dp `[414,438)`dp or `[1009,1068)`px band was a
   **prototype player spacer**,
 not a mandatory text-exclusion band or
   accepted final player gap.
    The user corrected the visible dent estimate from 10mm to about 7.5mm,
   approximately 110 **physical px** around center x 1038 on this panel.
    A conversion to roughly 45dp holds only at the current 390dpi setting;
    it changes
   with Android display scaling and is not a design constant.
    Readable text,
   results and other meaning-bearing marks must stay outside that
   approximate band;
    surfaces,
    dividers,
    backgrounds and hit regions
   may span it.
    Evaluate visual clearance using native
   captures,
 not the obsolete all-pixel-strip guard.
    D34 white and D41
   true black describe the player spacer,
 not mandatory Search colors.
    Android-owned system bars are
captured as rendered,
 not repositioned by the app.
4. **The physical cover is taller and narrower than the older 390 × 844dp
   phone mock.**
    On this AVD its 1080 × 2424px active panel is about 443 ×
   994dp at 390dpi;
    judge visible row count from the native captures,
    not the
   earlier 411dp estimate.
5. **Tabletop posture** is detectable on Android only (decisions.md E3).

---

## Physical crease and informational clearance

The user corrected their first estimate to **about 7.5mm of visible dent**.
From the published 8-inch diagonal and 2076 × 2152px aspect,
 the active
inner panel width is about 141.08mm.
 At this physical resolution,
 the
dent covers approximately `7.5 / 141.08 × 2076 = 110` **panel pixels**.
Centered at physical x 1038,
 the approximate information-free interval
is x `[983,1093)`px.
 These endpoints inherit the uncertainty of the
published diagonal and the user's "about 7.5mm" estimate.

Express `max(min_padding, crease_width)` in a common **physical** coordinate
system before positioning opposing information-bearing regions.
 The
crease term is 7.5mm (about 110 panel px here),
 not a hard-coded dp
constant.
 At the AVD's **current** 390dpi scaling,
 it corresponds to
about 45dp;
 changing display scaling changes that dp number but not the
physical dent.
 Compose may convert the current physical clearance to
layout units at rendering time.
 The user set the independent `min_padding` for this boundary to **7.5mm
of total opposing-information separation** after the native player/Search
review.
That physical minimum still applies if a future crease is narrower;
if the actual crease is wider,
the wider crease governs.
Neither term is a fixed dp constant,
and the separate 12dp rule remains mode-button horizontal content padding.
The [selected Search A measurement](evidence/selected-search-crease-geometry.md)
reports app-node bounds outside the approximate dent and a 167px projected
horizontal gap between extrema,
with sampled pixel-band controls.
Those bounds did not determine the user-chosen 7.5mm minimum and do not
guarantee that accessibility rectangles enclose all painted glyphs.
The [native player/Search comparison](evidence/crease-floor-native-comparison.md)
shows the selected P7.5 floor alongside rejected larger hard-floor proposals
at the measured 200% text scale;
it is not a universal implementation pass.
The later [D58 native alignment measurement](evidence/search-header-result-alignment.md)
placed the Back/folder/music icon-paint centers at x `1190` inner and x `97`
cover;
query/result text starts coincided at x `1249` inner and x `156` cover
at both 100% and 200% text.
This applies to the keyboard-closed debug Search fixtures only,
not every font scale,
keyboard state or painted mark near the crease.

This formula is **not** a mandatory unpainted gap between visible panes.
Borders,
 padding,
 backgrounds,
 input/row surfaces and hit regions can cross
the crease;
 their readable text and other informative marks must stay
clear.
 Do not derive final player pane widths by subtracting 110px from the
whole screen unless a candidate actually chooses symmetric content columns.

The Pixel_9_Pro_Fold AVD's `config.ini` separately contains
`hw.sensor.hinge.areas=1038-0-0-2152`.
 Google's emulator schema defines
this as `x-y-width-height`,
 so the **emulated occlusion area** has width 0.
That sensor model does **not** negate the user's visible 7.5mm crease.
Source:
 [Google emulator hardware properties][emulator-hardware-properties],
`hw.sensor.hinge.areas`.

## Viewport sizes to design and test at

```text
443 × 994    measured AVD cover screen (folded, 390dpi; physical 1080 × 2424px)
852 × 883    measured AVD inner display (unfolded, 390dpi; physical 2076 × 2152px)
426 × 883    right half at current AVD density; physical crease is about 110px wide
Desktop      inherits Fold visual decisions; its window size is not a design frame (D49)
```

Use the emulator's opaque panel captures at physical resolution.
 For a review
scaled to 100% Android dp,
 display the cover at approximately 443 × 994 CSS px
and the inner display at approximately 852 × 883 CSS px.
 Older Design Component
preview sizes are historical,
 not substitutes for these native results.

---

## For context: other book-style foldables

Useful only if the target ever changes.
 The Pixel 9 Pro Fold’s inner panel is squarer
than most;
 the original Pixel Fold was 1840×2208 px and physically wider (158.7 mm
across unfolded,
 versus 150.2 mm).
 Galaxy Z Fold 6 and OnePlus Open are also
near-square but slightly wider than tall.
 Do not carry any of these numbers into the
design without re-deriving them.

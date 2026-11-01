# Why the same nominal text size looks different across platforms and typefaces

Primary-source research for the troubleshooting doc
`doc/troubleshooting/music-player-desktop-text-looks-larger-than-android-at-the-same-size.md`.
A research subagent fetched every source on 2026-10-22 and wrote this note; the lead agent saved it here and did not
re-fetch the sources.
Quotes are exact sentences from the fetched text (a few wrapped across lines, words unchanged).
Lines labeled "Arithmetic" are calculations from quoted or measured numbers, not source claims.
No recommendation about the music player is made in this note.
Scratch material (fetched pages, `tools/sfnt-metrics.ts`, the metrics JSON, the Roboto release) is in
`/home/user/temp/agent/text-size-research` on the machine that produced it.

## Perceived size, x-height, cap height and width

### CSS Fonts Level 5, `font-size-adjust`

Source: https://drafts.csswg.org/css-fonts-5/ (section 3.4, read 2026-10-22).

- Apparent size varies with design at a fixed font size.
  > For any given font size, the apparent size and effective legibility of text varies across fonts as a function of
  > their design.
- The relative height of lowercase letters is named as a determining factor.
  > For example, for bicameral scripts such as Latin or Cyrillic that distinguish between upper and lowercase
  > letters, the relative height of lowercase letters compared to their uppercase counterparts is a determining
  > factor of legibility.
- Fallback fonts can look like a different size.
  > In situations where font fallback occurs, fallback fonts might not share the same ratios as the desired font
  > family for key typographic metrics, and will thus appear to be a different size and possibly be less readable.
- The property exists to preserve apparent size.
  > The font-size-adjust property provides a way to preserve the readability and apparent size of text when font
  > fallback occurs.
  > It does this by adjusting the used font size so that the specified metric is the same regardless of the font used.
- The metrics. The grammar is
  `none | [ ex-height | cap-height | ch-width | ic-width | ic-height ]? [ from-font | <number [0,∞]> ]`, and
  `ex-height` is the default metric.
  > ex-height: Normalize the aspect value of the fonts, using the x-height divided by the font size.
  > cap-height: Normalize the cap-height of the fonts, using the cap-height by the font size.
  > ch-width: Normalize the horizontal narrow pitch of the fonts, using the advance width of "0" (ZERO, U+0030)
  > divided by the font size.
  > ic-width: Normalize the horizontal wide pitch of the font, using the advance width of "水" (CJK water ideograph,
  > U+6C34) divided by the font size.
- The adjustment formula: `u = ( m / m′ ) s`, where s is the computed font-size, m the metric as specified by
  `font-size-adjust`, m′ the metric in the actual font, and u the adjusted font size to use.
- The worked example gives concrete aspect values.
  > Verdana has a relatively high aspect value of 0.545, meaning lowercase letters are relatively tall compared to
  > uppercase letters, so at small sizes text appears legible.
  > Times has a lower aspect value of 0.447, and so if fallback occurs, the text will be less legible at small sizes
  > than Verdana unless font-size-adjust is also specified.
- A limit on large x-heights.
  > Note: For text which uses diacritics, too large an x-height will actually decrease legibility as the diacritics
  > become cramped.
- It does not change `em` or `line-height`.
  > The value of font-size-adjust affects the used value of font-size but does not affect the computed value.
  > Therefore it can affect the size of relative units that are based on font metrics such as ex and ch but does not
  > affect the size of em units.

### Peer-reviewed: x-height as the determinant of perceived print size

Source: Legge GE, Bigelow CA. "Does print size matter for reading? A review of findings from vision science and
typography." Journal of Vision 11(5):8, 2011. doi:10.1167/11.5.8, PMID 21828237, PMCID PMC3428264.
Full text read at https://pmc.ncbi.nlm.nih.gov/articles/PMC3428264/ on 2026-10-22.

- Apparent size follows x-height, not body size. The authors flag this as typographic opinion plus informal
  observation, "not proven by rigorous laboratory studies".
  > Although not proven by rigorous laboratory studies, typographic opinion (Williamson, 1966) and informal
  > observation indicate that x-height is a more salient determinant of perceived type size than is body size.
  > For a given body size, the apparent sizes of typefaces are influenced by their x-heights: faces with larger
  > x-height fractions appear to be bigger than those with smaller fractions (Figure 1a).
  > Conversely, types with the same x-heights but different body sizes appear to be approximately the same size.
- The figure caption names three faces at one body size.
  > Three typefaces with the same body size but different x-height fractions (in parentheses), from left to right:
  > Lucida Bright (0.53); Times Roman (0.45); Centaur (0.37).
- x-height fraction varies widely across faces.
  > Karow (1993) measured the x-heights of 1049 roman typefaces and found that the x-height fraction ranged from 0.28
  > to 0.58, with a mean of 0.46.
- Width relates to x-height, for seriffed text faces (the paper says so explicitly).
  > The x-height provides a good estimate of mean letter width.
  > We calculated mean letter widths (weighted by letter frequencies in text) for 23 digital seriffed fonts ... and
  > found a correlation of 0.94 between x-height and mean width, which was 0.47 of body size.
- The size that matters for vision is angular, which needs a viewing distance.
  > Vision scientists usually specify print size in angular units, frequently using angular x-height.
  > Full stimulus description also requires indication of the viewing distance and font.
  > Conversion from print size on the page in points or millimeters to visual angle requires specification of the
  > reading distance.
  > A "standard" reading distance of 40 cm (16 inches) for hardcopy text is often adopted for this conversion.
- Reading distance for screens in a study the paper cites.
  > For instance, Shieh & Lee (2007) studied preferred reading distances and found a mean close to 50 cm for both
  > electronic paper and video display terminals (VDT), greater than their reported value of 36 cm for books and
  > paper.
- Fluent range, in x-height.
  > The fluent range extends over a factor of 10 in angular print size (x-height) from approximately 0.2° to 2°.
  > Assuming a standard reading distance of 40 cm (16 inches), the corresponding physical x-heights are 1.4 mm
  > (4 points) and 14 mm (40 points).
- Optical scale (why small sizes historically had larger x-height fractions).
  > Carter found that when type punches were hand-cut at print size, the x-height fractions of small type sizes were
  > larger than x-height fractions at large sizes, a phenomenon he termed "optical scale".

### Peer-reviewed: typeface style changes glance legibility

Dobres J, Chahine N, Reimer B, Gould D, Mehler B, Coughlin JF. Ergonomics 59(10):1377-1391, 2016.
doi:10.1080/00140139.2015.1137637, PMID 26727912. Abstract read via NCBI E-utilities on 2026-10-22.
> Results indicate that during glance-like reading induced by the psychophysical technique and under the lighting
> conditions considered, humanist-style type is significantly more legible than a square grotesque style, and that
> black-on-white text is significantly more legible than white-on-black.

This is a legibility threshold result for two named typefaces, not a statement about perceived size.

### Standards: ISO 9241-303, ISO 9241-3, ANSI/HFES 100

No primary source accessible. `https://www.iso.org/standard/57992.html` (ISO 9241-303:2011) returned no readable
text, and the ISO Online Browsing Platform preview returned 267 characters with no clause text.
The standards are paywalled.
A web search returned only secondary summaries (a handbook chapter attributing a 1/200 viewing distance ratio and 16 to
22 arcminute character heights to FAA guidance); the subagent did not read that chapter and does not rely on it.

## Font file metrics

### Method

- Reader: `tools/sfnt-metrics.ts` in the scratch directory (no dependencies; `node tools/sfnt-metrics.ts <font.ttf>`).
  It reads the sfnt table directory, then `head.unitsPerEm`, `OS/2` (`sxHeight`, `sCapHeight`, typo and win metrics,
  `fsSelection` bit 7), `hhea`, `hmtx`, `cmap` (format 12 preferred, else format 4), `fvar` axes, and for TrueType
  outlines the `glyf` header `yMax` of `x` and `H` as a cross-check of the `OS/2` values.
- Average lowercase advance: unweighted mean of the `hmtx` advances of `a` to `z` (U+0061 to U+007A), divided by
  `unitsPerEm`; not letter-frequency weighted.
- Line box ratio: (ascender minus descender plus lineGap) divided by `unitsPerEm`, once from `hhea`, once from the
  `OS/2` typo values.
- Variable fonts: base table values, which are the default instance. `HVAR` and `MVAR` deltas were not applied, so
  nothing here describes a non-default `opsz` or `wght`.
- The x-height and cap-height from `OS/2` equal the `glyf` `yMax` of `x` and `H` in every face measured.

### Results

Inter, `~/.local/share/fonts/InterVariable.ttf` (default instance)
- sha256 `4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`
- `fvar`: `opsz` min 14, default 14, max 32; `wght` min 100, default 400, max 900
- `unitsPerEm` 2048; `OS/2` version 4
- x-height 1118 units = 0.5459; cap height 1490 units = 0.7275
- average lowercase advance 1098.42 units = 0.5363 em
- `hhea` 1984 / -494 / gap 0 = 1.2100; typo 1984 / -494 / gap 0 = 1.2100; USE_TYPO_METRICS set

Liberation Sans Regular, `/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf`
- sha256 `76d04c18ea243f426b7de1f3ad208e927008f961dc5945e5aad352d0dfde8ee8`
- `unitsPerEm` 2048; `OS/2` version 3
- x-height 1082 units = 0.5283; cap height 1409 units = 0.6880
- average lowercase advance 1002.58 units = 0.4895 em
- `hhea` 1854 / -434 / gap 67 = 1.1499; typo 1491 / -431 / gap 307 = 1.0884; USE_TYPO_METRICS not set

Noto Sans Regular, `/usr/share/fonts/google-noto/NotoSans-Regular.ttf`
- sha256 `478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823`
- `unitsPerEm` 1000; `OS/2` version 4
- x-height 536 units = 0.5360; cap height 714 units = 0.7140
- average lowercase advance 530.08 units = 0.5301 em
- `hhea` 1069 / -293 / gap 0 = 1.3620; typo 1069 / -293 / gap 0 = 1.3620; USE_TYPO_METRICS set

Roboto 3, release `v3.016` of `googlefonts/roboto-3-classic` (published 2026-06-17), asset `Roboto_v3.016.zip`
(downloaded with `gh release download`; OFL licensed per the brief, the license file was not separately checked)
- zip sha256 `1653dbe12f248da8fb0b9920db7b9496cd677ed3981154f6f15285c8bd4e334f`
- Three files were measured. All three give the same outline metrics, `unitsPerEm` 2048, `OS/2` version 4:
  - x-height 1082 units = 0.5283; cap height 1456 units = 0.7109
  - average lowercase advance 1022.38 units = 0.4992 em
  - `hhea` 1900 / -500 / gap 0 = 1.1719 in all three
- `android/static/Roboto-Regular.ttf`, sha256 `e688a215e0841b6e4edb1207c93f88f4c609f82e870884349a7257e449eb9355`:
  typo 2146 / -555 / gap 0 = 1.3188; USE_TYPO_METRICS not set
- `android/Roboto[ital,wdth,wght].ttf`, sha256 `07f56c49cba5aa0c36f0c8295ffe2e39578e8459a743a7a619ad094b483ba857`
  (default instance; `wght` 100/400/900, `wdth` 75/100/100, `ital` 0/0/1): typo 2146 / -555 / gap 0 = 1.3188
- `web/static/Roboto-Regular.ttf`, sha256 `a6ec0ffaad7339c18c90c6a9728465e9003ae725a8769401af356a8afe523690`:
  typo 1536 / -512 / gap 102 = 1.0498
- Which of these files an Android device or the music player's Compose build uses was not determined.

Arithmetic on the measured values (ratio of Inter's default instance to the other face)
- x-height: Inter over Roboto 0.5459 / 0.5283 = 1.033; over Liberation Sans 1.033; over Noto Sans 1.018
- cap height: Inter over Roboto 0.7275 / 0.7109 = 1.023; over Liberation Sans 1.057
- average lowercase advance: Inter over Roboto 0.5363 / 0.4992 = 1.074; over Liberation Sans 1.096
- `hhea` line box: Inter over Roboto 1.2100 / 1.1719 = 1.033
- Roboto and Liberation Sans have the identical x-height ratio (0.5283) but different cap heights.
- The `font-size-adjust` formula with `ex-height`, m = 0.5283 (Roboto), m′ = 0.5459 (Inter): a 16 px Inter run would be
  drawn at 15.48 px, and a 22 px run at 21.29 px.

### Inter's documented design intent

Source: https://github.com/rsms/inter (default branch commit `353b61b9f4430d5f420d56605a6e7993e0941470`, latest release
tag `v4.1` published 2024-11-16; files read via `gh api` on 2026-10-22).

- `README.md`:
  > Inter is a typeface carefully crafted & designed for computer screens.
  > Inter features a tall x-height to aid in readability of mixed-case and lower-case text.
  > Some trade-offs were made in order to make this typeface work really well at small sizes
- `docs/index.html` (the website source):
  > The smaller "text" optical-size designs features a tall x-height to aid in legibility of lower-case text, with
  > several contrast-enhancing details like ink traps and bridges.
  > The larger "display" optical-size designs offers clean lines, smooth curves and delicate details for excellent
  > rhythm of large text.
  > Optical size ranges from "text" to "display" and there is a true italic variant.
- `docs/_data/faq.yml`, "What are the technical details of Inter?" (these match the measured 1118 and 1490; the source
  writes the range with an en dash, `[14` to `32]`):
  > Variable axis opsz [14 to 32]
  > x-height, opsz=14: 1118 UPM
  > x-height, opsz=32: 1056 UPM
  > Cap height: 1490 UPM
  > UPM: 2048
  Arithmetic: 1056 / 2048 = 0.5156, which is 5.5 percent below the 0.5459 at `opsz` 14.
- `docs/index.html` glyph inspector slider title: `opsz from 14 "text" to 32 "display"`.
- No Inter document names a CSS pixel or point size at which to switch designs: no primary source found for a
  recommended size threshold.

### OpenType `opsz` axis definition

Source: https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxistag_opsz (read 2026-10-22).

> Scale interpretation: Values can be interpreted as text size, in typographic points, as defined in the OpenType
> specification: a physical unit equal to 1/72 of a standard physical inch.

> Recommended or required "Regular" value: A value in the range 10 to 16 is recommended for typical text settings.

> Type designers may develop size-specific design variations based on print or screen rendering, typically
> evaluating the design of these variations at a reading distance of 14 to 16 inches, or 35 to 40 cm.

> The scale for the Optical size axis is text size in points. For these purposes, the text size is as determined by
> the document or application for its intended use; the actual physical size on a display may be different due to
> platform or application scaling methods or intended viewing distance.

### CSS Fonts 4, `font-optical-sizing`

Source: https://drafts.csswg.org/css-fonts-4/ (section 8.1, read 2026-10-22). Initial value `auto`.

> auto: The user agent may modify the shape of glyphs based on the font-size and the pixel density of the screen.
> For OpenType and TrueType fonts using font variations, this is often done by using the "opsz" font variation.

> Note: User agents are expected to supply a value for the "opsz" axis which is close to the used value for font-size.
> However, user agents might wish to consider other factors such as pixel density of the screen, or the solid angle
> the text subtends in the viewer's retina.
> Experiments have shown, however, that disregarding these other ancillary factors and using only font-size might be
> the best way for a user agent to select this value.

> User agents must not select a value for the "opsz" axis which is not supported by the font used for rendering the
> text. This can be accomplished by clamping a chosen value to the range supported by the font.

> font-optical-sizing interacts with font-size-adjust. When applying optical sizing, User agents must apply an
> optical sizing value appropriate for the adjusted font size, subject to the above restrictions.

### What Chromium does by default

Pinned commit of `chromium/src`: `e3f93ce5549ba3fb553fe39a34fef3a187dcc6f3` (main on 2026-10-22).
URL prefix for the paths below:
`https://chromium.googlesource.com/chromium/src/+/e3f93ce5549ba3fb553fe39a34fef3a187dcc6f3/`.
The GitHub mirror `chromium/chromium` returned HTTP 429 from `gh api` for file contents, so googlesource was used.

- `third_party/blink/renderer/platform/fonts/font_custom_platform_data.cc`, lines 241 to 245: when the page set no
  explicit `opsz` and `optical_sizing == kAutoOpticalSizing`, Blink pushes an `opsz` coordinate equal to the adjusted
  specified size.
  ```cpp
  // third_party/blink/renderer/platform/fonts/font_custom_platform_data.cc
  if (!explicit_opsz_configured) {
    if (optical_sizing == kAutoOpticalSizing) {
      SkFontArguments::VariationPosition::Coordinate opsz_coordinate = {
          kOpszTag, SkFloatToScalar(adjusted_specified_size)};
  ```
  The value is passed through with no unit conversion in this function (the file does not state its unit; the OpenType
  axis definition says points). With `font-optical-sizing: none` (lines 246 to 256) it sets the axis default from the
  font's `fvar`.
- `third_party/blink/renderer/platform/fonts/font_description.h`, lines 215 to 219:
  > Returns the result of applying font-size-adjust to the specified size. This is useful as an input to optical
  > sizing and takes zooming out of the equation for determining the font size to be used for font-optical-sizing:
  > auto;.
- Scope limit: this code is in `FontCustomPlatformData`, the web font (`@font-face`) path.
  The path for a system-installed variable font such as a fontconfig-found `InterVariable.ttf` was not traced:
  no primary source found for that case.
  How Slint chooses `opsz` is outside this note (the troubleshooting doc traces it from the Slint source).

## Units and viewing distance

### CSS reference pixel

Source: https://drafts.csswg.org/css-values-4/ (section 6.2, read 2026-10-22).

> The reference pixel is the visual angle of one pixel on a device with a device pixel density of 96dpi and a
> distance from the reader of an arm's length.
> For a nominal arm's length of 28 inches, the visual angle is therefore about 0.0213 degrees.
> For reading at arm's length, 1px thus corresponds to about 0.26 mm (1/96 inch).

- Names a viewing distance: yes, 28 inches (71 cm). The spec also says "a reading distance of 3.5 m (12 feet) results
  in a reference pixel of 1.3 mm."
- Physical size: 1px = 1/96 inch = 0.2646 mm "for reading at arm's length"; the spec also says "1px = 1/96th of 1in"
  and, for screens, "it is recommended instead that the anchor unit be the pixel unit".
  So on a real display the physical size of 1 CSS px is not guaranteed.
- Arithmetic check: 2 * atan(0.2646 mm / 711.2 mm / 2) = 0.0213 degrees.

### Android `dp` and `sp`

Sources, read 2026-10-22: https://developer.android.com/guide/topics/resources/more-resources and
https://developer.android.com/training/multiscreen/screendensities

> dp: Density-independent pixels: an abstract unit that is based on the physical density of the screen. These units
> are relative to a 160 dpi (dots per inch) screen, on which 1 dp is roughly equal to 1 px.

> sp: Scale-independent Pixels - This is like the dp unit, but it is also scaled by the user's font size preference.
> It is recommend you use this unit when specifying font sizes, so they will be adjusted for both the screen
> density and the user's preference.

> One dp is a virtual pixel unit that's roughly equal to one pixel on a medium-density screen (160 dpi, or the
> "baseline" density).

> The sp unit is the same size as a dp, by default, but it resizes based on the user's preferred text size.

> px = dp * (dpi / 160)

- Names a viewing distance: no. Neither page mentions viewing distance (searched for "distance" and "arm").
- Physical size: 1 dp = 1/160 inch = 0.15875 mm (arithmetic from "relative to a 160 dpi screen"); 1 sp equals 1 dp at
  the default text-size preference. 1 dp is therefore 0.6 of a CSS px in physical size (96 / 160).

### Microsoft effective pixel (epx)

Sources, read 2026-10-22:
- https://learn.microsoft.com/en-us/windows/apps/design/layout/screen-sizes-and-breakpoints-for-responsive-design
- https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography
- https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows

> Effective pixels (epx) are a virtual unit of measurement used to express layout dimensions and spacing,
> independent of screen density.

> This scaling algorithm takes into account viewing distance and screen density (pixels per inch) to optimize for
> perceived size rather than physical size. The algorithm ensures that a 24 px font on a large screen 10 feet away is
> just as legible as a 24 px font on a small screen a few inches away.

> However, designing for a 1080p TV that you view from 10 feet away is different from designing for a 1080p monitor
> on your desk. When you account for viewing distance, the TV's 1080 pixels are more like a 540-pixel monitor that's
> much closer.

> DPI unaware applications render at a fixed DPI value of 96 (100%).

- Names a viewing distance: only qualitatively and for TVs. The device-family list gives screen sizes and device
  classes (Small: 20" to 65" TVs; Medium: 7" to 12" tablets; Large: 13" and up, "PCs, Laptops, Surface Hub") and the
  only numeric distance is "10 feet" for a TV. No per-family viewing distance table found on these pages.
- Physical size of one unit: no primary source found for a fixed physical size of 1 epx. The 100 percent scale is
  stated to be 96 DPI, so at 100 percent on a display that really is 96 DPI, 1 epx is one device pixel, 0.2646 mm
  (arithmetic).

### Apple points (macOS, iOS)

Sources, read 2026-10-22: https://developer.apple.com/design/human-interface-guidelines/typography (via the
documentation JSON at `https://developer.apple.com/tutorials/data/design/human-interface-guidelines/typography.json`),
and the `layout`, `sidebars` and `lists-and-tables` pages from the same JSON path.

> People need to be able to read your content at various viewing distances and under a variety of conditions.

- Names a viewing distance: no numeric distance in any of the four HIG pages searched (`typography`, `layout`,
  `sidebars`, `lists-and-tables`; searched for "distance", "pt", "point").
- Physical size of one point: no primary source found. The only density statement found is a table footnote for iOS
  design images: "Point size based on image resolution of 144 ppi for @2x and 216 ppi for @3x designs." That describes
  design-asset resolution, not a physical length of a point.

## Default UI and body text sizes

CSS px conversion follows the brief: 1 pt = 4/3 px at 96 dpi.
For Android `sp` and `dp` the nominal number is given first and then the physical equivalent in CSS px
(1 dp = 0.6 CSS px, from the heading "Android `dp` and `sp`"), because those two differ.
The physical equivalents assume the device's real density matches the nominal baseline; they are arithmetic, not
measured sizes.
Apple and Microsoft sizes are not physical lengths by their own docs, so their conversion is the brief's convention
only.

### Body, label and control text

Material 3 (Android Compose)
- Source: `androidx/compose/material3/tokens/TypeScaleTokens.kt` in
  `https://android.googlesource.com/platform/frameworks/support/+/d51743c7df67993966071b3211557dca5ef8000a/`
  (path under `compose/material3/material3/src/commonMain/kotlin/`, `androidx-main` commit
  `d51743c7df67993966071b3211557dca5ef8000a`, read 2026-10-22).
- `BodyLargeSize get() = 16.sp` (line height `24.0.sp`); `BodyMediumSize get() = 14.sp` (`20.0.sp`);
  `BodySmallSize get() = 12.sp` (`16.0.sp`).
- `LabelLargeSize get() = 14.sp` (`20.0.sp`); `LabelMediumSize get() = 12.sp`; `LabelSmallSize get() = 11.sp`.
- `TitleLargeSize get() = 22.sp` (`28.0.sp`); `TitleMediumSize get() = 16.sp`; `TitleSmallSize get() = 14.sp`.
- Physical equivalent in CSS px: 16 sp = 9.6; 14 sp = 8.4; 12 sp = 7.2; 11 sp = 6.6; 22 sp = 13.2.
- Design doc, https://m3.material.io/styles/typography/applying-type:
  > Body styles are used for longer passages of text in your app.
  > Label styles are smaller, utilitarian styles, used for things like the text inside components or for very small
  > text in the content body, such as captions.
  > As the largest text on the screen, display styles are reserved for short, important text or numerals. They work
  > best on large screens.
  > Take engineering considerations and the conventions of your platform into account when making decisions for
  > typesetting, text resizing, density, and using text in adaptive layouts.
- A statement about desktop or a different body size for large screens: none found. The `applying-type` text was
  searched for "desktop", "large screen", "tablet", "window size", "distance", "viewing". The `type-scale-tokens` page
  rendered its token values in collapsed folders, so the values above come from the Compose token source instead.

Apple HIG (https://developer.apple.com/design/human-interface-guidelines/typography)
- Default and minimum sizes, as listed on the page for each platform:
  - iOS, iPadOS: default 17 pt (22.67 px), minimum 11 pt (14.67 px)
  - macOS: default 13 pt (17.33 px), minimum 10 pt (13.33 px)
  - tvOS 29 pt / 23 pt; visionOS 17 pt / 12 pt; watchOS 16 pt / 12 pt
  > Follow the recommended default and minimum text sizes for each platform ... to ensure your text is legible on all
  > devices.
- iOS Dynamic Type "Large (default)" table: Body 17 pt (22.67 px), Callout 16 (21.33), Subhead 15 (20),
  Footnote 13 (17.33), Caption 1 12 (16), Caption 2 11 (14.67).
- macOS built-in text styles: Body 13 pt (17.33 px), Callout 12 (16), Subheadline 11 (14.67), Footnote 10 (13.33),
  Caption 1 10 (13.33), Headline 13 bold, Title 3 15 (20), Title 2 17 (22.67).
- Optical sizing in the system fonts, same page:
  > On all platforms, the system fonts support dynamic optical sizes, which merge discrete optical sizes (like Text
  > and Display) and weights into a single, continuous design, letting the system interpolate each glyph or
  > letterform to produce a structure that's precisely adapted to the point size.

Microsoft Fluent and the Windows 11 type ramp (epx; at 100 percent scale 1 epx = 1 CSS px by the 96 DPI statement)
- Sources: https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography and
  https://learn.microsoft.com/en-us/windows/apps/design/style/xaml-theme-resources.
  > Windows 11 uses the following values for various types of text in the UI.
- Sizes (style names from the XAML theme resources page): Caption 12 epx (line height 16), Body 14 (20),
  Body Strong 14 semibold, Body Large 18 (24), Subtitle 20 (28), Title 28 (36), Title Large 40 (52), Display 68 (92).
- WinUI source, `microsoft/microsoft-ui-xaml` commit `4bcbff6d6b94c43c2e723cb0f08b51e473fbd235`,
  `controls/dev/CommonStyles/TextBlock_themeresources.xaml` line 3:
  `<x:Double x:Key="CaptionTextBlockFontSize">12</x:Double>`.

GNOME HIG and libadwaita
- Default font: https://gitlab.gnome.org/GNOME/gsettings-desktop-schemas (file
  `schemas/org.gnome.desktop.interface.gschema.xml.in`, `master`, read 2026-10-22):
  `<key name="font-name" type="s"> <default>'Adwaita Sans 11'</default>`. 11 pt = 14.67 px.
- https://developer.gnome.org/hig/guidelines/typography.html:
  > In GNOME, the default font is Adwaita Sans, a custom built variant of Inter
  > Body: The default text style, which is used for control labels and descriptive UI text.
  The page lists style names and CSS classes but no point size.

KDE HIG and Breeze
- https://develop.kde.org/hig/text_and_labels/ states no default point size (searched for "pt", "point", "font size",
  "default font", "Noto", "Breeze").
- Code, `KDE/plasma-integration` commit `46eddde9d38cf907786e567dd4a942140bb1f797`,
  `qt6/src/platformtheme/kfontsettingsdata.cpp` lines 44 to 53:
  ```cpp
  // qt6/src/platformtheme/kfontsettingsdata.cpp
  static const char DefaultFont[] = "Noto Sans";
  {GeneralId, "font", DefaultFont, 10, QFont::Normal, QFont::SansSerif, "Regular"},
  {GeneralId, "menuFont", DefaultFont, 10, QFont::Normal, QFont::SansSerif, "Regular"},
  {GeneralId, "smallestReadableFont", DefaultFont, 8, QFont::Normal, QFont::SansSerif, "Regular"},
  ```
  10 pt = 13.33 px; 8 pt = 10.67 px. This is the framework default; whether the Breeze or Plasma style overrides it
  was not examined.

Side by side, CSS px at 96 dpi (nominal number in parentheses)
- Material 3 body large: 16 sp (physical equivalent 9.6 px); body medium and label large: 14 sp (8.4 px)
- Apple iOS body: 17 pt = 22.67 px; Apple macOS body: 13 pt = 17.33 px
- Microsoft Windows 11 Body: 14 epx = 14 px; Caption: 12 epx = 12 px
- GNOME default and Body: 11 pt = 14.67 px
- KDE default: 10 pt = 13.33 px

### List-row and menu text

Material 3 (same pinned commit)
- `tokens/ListTokens.kt`: `ItemLabelTextFont ... get() = TypographyKeyTokens.BodyLarge` (headline text, 16 sp,
  physical equivalent 9.6 CSS px); `ItemSupportingTextFont ... get() = TypographyKeyTokens.BodyMedium` (supporting
  text, 14 sp, 8.4 CSS px); `ItemOverlineFont ... get() = TypographyKeyTokens.LabelSmall` (11 sp, 6.6 CSS px);
  `ItemOneLineContainerHeight ... get() = 56.0.dp`; `ItemThreeLineContainerHeight ... get() = 88.0.dp`.
- `Menu.kt` (dropdown menu item), lines 2266 and 2267:
  `ProvideTextStyle(MaterialTheme.typography.labelLarge, content = content)` and
  `ProvideTextStyle(MaterialTheme.typography.bodyMedium, content = supportingText)`. So the menu item label is 14 sp
  (8.4 CSS px) and its supporting text is 14 sp.

Apple
- HIG (`sidebars` page):
  > A sidebar's row height, text, and glyph size depend on its overall size, which can be small, medium, or large.
  No numeric point size for sidebar rows or table rows is given in the `sidebars` or `lists-and-tables` pages.
  No primary source found for HIG list-row or sidebar text sizes in points.
- AppKit developer documentation (`documentation/appkit/nstableview/rowheight`):
  > The default row height is `16.0`.
  That is a row height (points), not a text size.
- UIKit (`documentation/uikit/uitableview/rowheight`): row height defaults to automatic sizing from cell content, so no
  fixed text size is given there.
- Not found: the point sizes AppKit or UIKit use for the text inside rows. The only sourced text sizes are the platform
  body sizes above (macOS 13 pt, iOS 17 pt); that rows use them was not found stated.

Microsoft Fluent
- WinUI source, `controls/dev/CommonStyles/ListViewItem_themeresources.xaml` at commit
  `4bcbff6d6b94c43c2e723cb0f08b51e473fbd235`: `<x:Double x:Key="ListViewItemMinHeight">40</x:Double>` (line 14) and
  `<Setter Property="FontSize" Value="{ThemeResource ControlContentThemeFontSize}" />` (lines 236, 263, 571).
- The numeric value of `ControlContentThemeFontSize` is not defined in the WinUI repository files fetched
  (`ListViewItem_themeresources.xaml`, `Common_themeresources.xaml`, `Common_themeresources_any.xaml`,
  `TextBlock_themeresources.xaml`) and is not stated on the Learn pages read. No primary source found for the list
  item font size number.

GNOME libadwaita (`AdwActionRow`)
- Source: https://gitlab.gnome.org/GNOME/libadwaita at tag `1.10.0`, commit `c9ab118612eee92324143783c4641c98eaded3f0`,
  file `src/stylesheet/widgets/_lists.scss`:
  - Lines 124 to 126: `row { label.subtitle { font-size: smaller; @extend .dimmed; }`. The title has no font-size
    rule there, so it inherits the base size: 11 pt = 14.67 px by the default font above.
  - `row.property` (lines 427 to 436) reverses this: `> .title { font-size: smaller; ...}` and
    `> .subtitle { font-size: inherit; ...}`.
  - `box.header { ... min-height: 50px; }` (near line 128).
- What `smaller` means in GTK: `GNOME/gtk` commit `49eeaf8a8c5988051796564f741edc96ae66f81d`,
  `gtk/gtkcssenumvalue.c`, lines 270 to 277:
  ```c
  /* gtk/gtkcssenumvalue.c */
  case GTK_CSS_FONT_SIZE_SMALLER:
    ...
    /* This is what WebKit does... */
    font_size /= 1.2;
  ```
  Arithmetic: subtitle = 14.67 / 1.2 = 12.22 px (9.17 pt).

KDE Breeze and the Kirigami list item
- `KDE/kirigami` commit `9fbdc4050f29e1443b12f2a6b24ee86e6356755c`, `src/delegates/TitleSubtitle.qml` lines 75 and 79:
  `property font font: Platform.Theme.defaultFont` and `property font subtitleFont: Platform.Theme.smallFont`.
- `src/platform/platformtheme.cpp` line 121:
  `QFont smallFont = QFontDatabase::systemFont(QFontDatabase::SmallestReadableFont);`
- With the `plasma-integration` defaults above, title = 10 pt = 13.33 px and subtitle = smallestReadableFont 8 pt
  = 10.67 px, unless a style overrides `smallFont` (not examined).

## Chromium and Firefox default `sans-serif` on Linux

### Chromium

Pinned commit `e3f93ce5549ba3fb553fe39a34fef3a187dcc6f3` of `chromium/src` (URL prefix as under "What Chromium does by
default").

- `chrome/app/resources/locale_settings_linux.grd`, lines 168 to 171:
  ```xml
  <!-- chrome/app/resources/locale_settings_linux.grd -->
  <!-- The default value for |WebPreference::sans_serif_font_family| -->
  <message name="IDS_SANS_SERIF_FONT_FAMILY" use_name_for_id="true">
    Arial
  </message>
  ```
  The Linux default for the generic `sans-serif` is the family name `Arial`. In the same file, standard and serif are
  `Times New Roman` (lines 154 to 166) and fixed is `Monospace` (line 160).
- `third_party/blink/renderer/platform/fonts/skia/font_cache_skia.cc`, `FontCache::CreateTypeface` (line 276), line
  307: `sk_sp<SkTypeface> typeface = MatchFamilyStyle(name.empty() ? nullptr : name.c_str(), ...)`, which
  `FontCache::MatchFamilyStyle` (line 373) forwards to `fm->matchFamilyStyle(family_name, style)` on the Skia font
  manager.
- `components/services/font/font_service_app.cc`: line 7 `#include <fontconfig/fontconfig.h>`; lines 101 to 128
  `FontServiceApp::MatchFamilyName` calls `SkFontConfigInterface::GetSingletonDirectInterface()` and then
  `fc->matchFamilyName(...)`. So the named family is resolved through fontconfig.
- Local observation (not a source claim), on the machine that ran the research, 2026-10-22: `fc-match Arial` gave
  `LiberationSans-Regular.ttf: "Liberation Sans" "Regular"`; `fc-match sans-serif` and `fc-match system-ui` gave
  `NotoSans-Regular.ttf: "Noto Sans" "Regular"`. The fontconfig alias files were not read, so why `Arial` maps to
  Liberation Sans there is not sourced.

### Firefox

Pinned commit `d6fd7a4e69b3c2d4550582eb38d2cdbd9cbad79a` of `mozilla-firefox/firefox` (main on 2026-10-22, read with
`gh api` raw contents).

- `modules/libpref/init/all.js`, line 1609: `pref("font.name.sans-serif.x-western", "");` (empty by default).
- `modules/libpref/init/all.js`, line 2807, inside the block opened at line 2655
  `#if !defined(ANDROID) && !defined(XP_MACOSX) && defined(XP_UNIX)` and closed at line 2836:
  `pref("font.name-list.sans-serif.x-western", "sans-serif");`
- `modules/libpref/init/all.js`, lines 2021 to 2023: `font.default.x-western` is `"serif"`,
  `font.size.variable.x-western` is `16`.
- `gfx/thebes/gfxFcPlatformFontList.cpp`, lines 2362 to 2366:
  > Under Linux, the generics "serif", "sans-serif", "monospace" and "math" are included in the pref fontlist. These
  > map to whatever fontconfig decides they should be for a given language, rather than one of the fonts listed in
  > the prefs font lists (e.g. font.name.*, font.name-list.*)
  and lines 2748 to 2749 take the `sans-serif` list value as the generic to look up. So on Linux, Firefox's
  `sans-serif` is whatever fontconfig returns for `sans-serif`.

## Polarity: do light-on-dark glyphs look larger, bolder or blurrier

No primary source found for the text claim (light text on dark is perceived larger, bolder or blurrier than dark on
light).
One adjacent result on black-white border position follows, plus legibility studies that do not address perceived
size or weight.

### Adjacent result: irradiation at a black-white border

Westheimer G. "Irradiation, border location, and the shifted-chessboard pattern." Perception 36(4):483-494, 2007.
doi:10.1068/p5646, PMID 17564195. Abstract read via NCBI E-utilities on 2026-10-22.

> The position of a black-white border is perceived somewhat towards the black side of its physical location.
> New measurements were performed of the magnitude of this effect for foveal vision in the normal observer and it was
> found to be about 0.4 min of arc at medium photopic luminances.
> The border shift can be accounted for by postulating that the retinal light spread caused by the optics of the eye
> is subjected to a compressive nonlinearity of light intensity (Naka-Rushton equation).
> Such irradiation-induced apparent enlargement of the white elements does not fully account for the shifted-chessboard
> illusion.

This source is about a black-white border and checkerboard elements for the foveal observer.
It does not mention text, type, strokes, or display polarity; reading it onto text strokes is an inference this note
does not make.

### Found: polarity changes legibility (not stated as size or weight)

- Piepenbrock C, Mayr S, Buchner A. Human Factors 56(5):942-951, 2014. doi:10.1177/0018720813515509, PMID 25141597.
  Abstract read via NCBI E-utilities on 2026-10-22.
  > A positive polarity advantage was observed in proofreading performance.
  > Importantly, the positive polarity advantage linearly increased with decreasing character size.
  > The findings are in line with the assumption that the typically higher luminance of positive polarity displays
  > leads to an improved perception of detail.
  (Positive polarity is dark characters on a light background. Sizes tested: 8, 10, 12 and 14 pt, 0.22 to 0.34 degrees
  of vertical visual angle.)
- Dobres J, Chahine N, Reimer B. Applied Ergonomics 60:68-73, 2017. doi:10.1016/j.apergo.2016.11.001, PMID 28166901.
  Abstract (first part read):
  > Recent research on the legibility of digital displays has demonstrated a "positive polarity advantage", in which
  > black-on-white text configurations are more legible than their negative polarity, white-on-black counterparts.
  > Existing research in this area suggests that the positive polarity advantage stems from the brighter illumination
  > emitted by positive polarity displays, as opposed to the darker backgrounds of negative polarity displays.
- Dobres et al. 2016 (heading "Peer-reviewed: typeface style changes glance legibility") also found black-on-white
  more legible than white-on-black for glance reading.

### Searched without finding a statement

- Searched: PubMed E-utilities queries on "irradiation illusion", "irradiation" with perception, border and size, and
  "display polarity reading"; the APCA documentation (`Myndex/SAPC-APCA`, commit
  `cbd08bf26b792438a6b45bde217fb03f55d2f8ff`, files `APCA_FAQ.md`, `APCAeasyIntro.md`, `StandardObserverModel.md`) for
  "irradiat", "halation", "bolder", "polarity"; and the W3C "Accessibility Requirements for People with Low Vision"
  note (https://www.w3.org/TR/low-vision-needs/) for "halation", "irradiat", "polarity", "dark mode".
- The APCA docs say that the contrast model has "polarity sensitivity" and that WCAG 2.x "cannot provide useful
  guidance when designing 'dark mode'", which concerns contrast values, not perceived size.
- Font-vendor documentation on dark-mode weight compensation was not searched beyond the above, and the ISO and ANSI
  polarity clauses are paywalled.

## Arithmetic: nominal size to physical x-height

From the measured and quoted numbers only.
Physical length of 1 unit: CSS px = 25.4 / 96 = 0.2646 mm; Android dp or sp = 25.4 / 160 = 0.1588 mm.
The CSS px figures assume a display whose real pixel density equals the nominal density; the Android figures assume the
nominal 160-dpi-relative scaling.
No viewing distance is part of either definition except the CSS 28 inches.

- 16 px Inter (default instance x-height 0.5459): x-height 2.311 mm.
- 16 sp Roboto (x-height 0.5283): x-height 1.342 mm. Ratio desktop to Android = 1.722, which is (96 / 160 inverted =
  1.667) times (0.5459 / 0.5283 = 1.033).
- 14 px Inter: 2.022 mm. 14 sp Roboto: 1.174 mm.
- 20 px Inter: 2.889 mm. 22 px Inter: 3.178 mm.
- Visual angle of the x-height, `2 * atan(h / 2 / d)`, at three distances that appear in the sources (40 cm: Legge and
  Bigelow standard; 50 cm: Shieh and Lee mean for screens as quoted by Legge and Bigelow; 71.12 cm: the CSS 28 inch
  figure):
  - 16 px Inter, 2.311 mm: 0.331 deg at 40 cm; 0.265 deg at 50 cm; 0.186 deg at 71.12 cm
  - 16 sp Roboto, 1.342 mm: 0.192 deg at 40 cm; 0.154 deg at 50 cm; 0.108 deg at 71.12 cm
  - 14 px Inter, 2.022 mm: 0.290 deg at 40 cm; 0.232 deg at 50 cm; 0.163 deg at 71.12 cm
  - 14 sp Roboto, 1.174 mm: 0.168 deg at 40 cm; 0.135 deg at 50 cm; 0.095 deg at 71.12 cm
  - 22 px Inter, 3.178 mm: 0.455 deg at 40 cm; 0.364 deg at 50 cm; 0.256 deg at 71.12 cm
  - 20 px Inter, 2.889 mm: 0.414 deg at 40 cm; 0.331 deg at 50 cm; 0.233 deg at 71.12 cm
- For reference, Legge and Bigelow quote a fluent range of 0.2 to 2 deg of angular x-height and a critical print size
  "in the range from about 0.15° to 0.3°"; those are for reading speed of continuous text in their cited experiments.
- Viewing distances actually used with the Android phone or the desktop monitor were not measured or sourced here.

## Not found

- ISO 9241-303, ISO 9241-3, ANSI/HFES 100: paywalled, no readable primary text reached.
- A peer-reviewed laboratory test that x-height determines perceived size: Legge and Bigelow themselves say it is
  "not proven by rigorous laboratory studies"; no direct experimental paper on perceived size versus x-height was
  located.
- Any Inter document giving a recommended CSS px or pt threshold between text and display designs.
- How Chromium picks `opsz` for a system-installed (non-web-font) variable font.
- Which Roboto build a given Android device uses at runtime, or which build the Compose app bundles.
- Microsoft: a per-device-family viewing distance table; a fixed physical size of 1 epx.
- Apple: a numeric viewing distance; the physical size of a point.
- Material 3: any statement specific to desktop or a different body size on large screens, beyond "display styles
  ... work best on large screens".
- Apple HIG: numeric point sizes for table rows and sidebar rows.
- Fluent: the numeric value of `ControlContentThemeFontSize`, so the ListView item text size.
- KDE HIG: a stated default point size (the value comes from code); whether Breeze or Plasma overrides `smallFont` for
  Kirigami list subtitles.
- Why fontconfig maps `Arial` to Liberation Sans on the machine that ran the research.
- Any primary source saying light-on-dark text is perceived larger, bolder or blurrier than dark-on-light text; only the
  black-white border irradiation result by Westheimer, which does not address text.

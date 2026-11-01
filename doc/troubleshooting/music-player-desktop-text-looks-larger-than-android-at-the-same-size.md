# Music player desktop text at Android's size numbers (22 and 16) looked too large; 20 and 14 look right

Slint 1.18.1 on a 3840 by 2160 panel at scale factor 3, text in Inter Variable, compared with Chromium, Firefox,
Compose Material 3 text on Android, and four reference pages.
The renderer, the font file, the variable-font axes and the scale factor are measured to match Chromium,
so none of them is the cause.
What the human perceives as "too large" is not explained by any one instrument reading;
the section "Hypotheses" labels every candidate as measured, null or speculation.

## Symptom

The Android app draws these texts with Compose Material 3's stock type scale:
the unfolded header titles and the folder group heading at `titleLarge` (22 sp),
the folder names and track titles at `bodyLarge` (16 sp).
The desktop app drew the same five texts with the numbers 22 px and 16 px (`Theme.title-large`, `Theme.body-large`).
On the human's desktop they looked too large: "The font sizes look a little bit weird."
The human asked for each of the five texts to be 2 smaller, then said the Slint tokens are unreliable and the sizes
should match Android, then (after the plain 22 and 16 values were in place) said "I don't know why it still feels
wrong on desktop. Please still -2 to each of the text I pointed out."
With 20 px and 14 px the human said "Looks good now."
Later the human added that the unchanged 22 and 16 would not look right "even to you",
and that Android's folded and unfolded layouts should look right as well.

The five texts: the left pane title, the right pane title, the group heading letter, the folder names, the track titles.

The target to explain is a ratio: 20 over 22 is 0.91 and 14 over 16 is 0.875.
Each hypothesis below is scored against that.

## Root cause

No single cause is established.
What is established, with the instrument that established it, comes first;
every explanation of the feel itself is labeled speculation.

### Slint draws the size it is told to draw

`ui/theme.slint:142` to `147` define the sizes as absolute lengths:

```slint
out property <length> body-large: 16px;
out property <length> body-medium: 14px;
out property <length> body-small: 12px;
out property <length> title-large: 22px;
```

The window's `default-font-size` is wired to a Rust-side value, and `os-font-size` is `1rem`
(`ui/app_window.slint:63` to `66`).
`src/ui_font_scale.rs:100` to `110` sets `base-font-size` to 0.9 times the OS font, once.
No `Text` in the window uses a `rem` size, so that base affects only elements that name no size,
not the five texts.
The module comment at the top of `ui_font_scale.rs` says "Every font in the window is a rem multiple of the window
`default-font-size`"; that comment is out of date for the hand-drawn text.
It was found here, not fixed.

Measured against Chromium (device scale factor 3, `Inter Variable`, same string, ink box at half the brightest pixel):
- Ink width ratio is exactly 1.000 for 15 elements of the human's own screenshot (22, 16, 14 and 12 px text,
  up to 1136 physical pixels wide); heights are within 1 physical pixel.
- Heading letter "A" at 22 px: 49 physical pixels in the app, 48 in Chromium; folder-name capitals at 16 px:
  36 and 35.
- Ink mass is within about 2 percent at 16 and 22 px, and Chromium is 7 percent heavier at 12 and 14 px.

A sweep run by a subagent in the repository's nested compositor (femtovg renderer, `SLINT_SCALE_FACTOR=3`):
- String width equals Chromium's at 12, 14, 16, 18, 20, 22 and 24 px (one pixel apart at 20 px, 0.12 percent).
- Letter "A" height is within 3 percent.
- Ink mass is 0.94 to 1.005 of Chromium's for femtovg, so the app's text is not heavier.
- The software renderer, used only when `SLINT_BACKEND` contains "software"
  (`package/music-player/desktop-app/src/main.rs:1121` to `1135`), measured 1.02 to 1.08 of Chromium's mass.
  The shipped default is femtovg.

The conclusion that Slint renders Inter at the same size as Chromium rests on these numbers and holds for widths
and heights; it says nothing about how the same size feels.

### The Slint code path, from the source at tag `v1.18.1`

A subagent read the sources (clone `v1.18.1`, commit `372cf0ee5577c3dfec309a45e7b778ba4e81b734`);
the music-player lockfile pins the same crate versions: parley 0.11.1, fontique 0.11.1, harfrust 0.12.0, swash 0.2.10,
femtovg 0.27.0.
- `internal/core/items.rs:1481` to `1490` resolves the logical `font-size`; the default is 12 logical pixels
  (`internal/core/textlayout.rs:37`).
- Parley multiplies by the scale factor and keeps an `f32` (`parley-0.11.1/src/resolve/mod.rs:145` and `:180`:
  `FontSize(*value * scale)`).
  No rounding of the size was found in Slint or parley.
- In femtovg, `draw_glyph_run` rasterizes at the physical font size when the canvas has no scale
  (`femtovg-0.27.0/src/lib.rs:2179` to `2212`); the ceiling of the scale factor is used only for stroke tessellation
  tolerances.
- Glyphs are cached per 0.1 px of size and per 0.1 px of horizontal position
  (`femtovg-0.27.0/src/text.rs:116` and `:492`); quads are placed at whole pixels (`text.rs:549`).
- Hinting is on: swash is built with `.hint(true)` (`femtovg-0.27.0/src/text.rs:759` to `763`), mode
  `HintingMode::Smooth` (`swash-0.2.10/src/scale/hinting_cache.rs:27`).
  What hinting does to Inter Variable specifically was not determined; the width parity above bounds its effect.
- Fallback glyphs are drawn at the same size as the primary font's; no `font-size-adjust` or x-height matching exists in
  parley, fontique or Slint's text path (grep for `size_adjust`, `x_height`, `optical` found only reported metrics).
- The family list is the named family, then the generic sans-serif list, then the generic system-ui list
  (`internal/common/sharedfontique.rs:188`); after that fontique appends the script's fallback family and an
  unconditional Han fallback (`fontique-0.11.1/src/collection/query.rs:95` to `114`).
- Slint sets no locale on text (`FallbackKey` without a locale, `fontique-0.11.1/src/fallback.rs:200`),
  and fontique asks fontconfig with a character-set pattern and no language
  (`fontique-0.11.1/src/backend/fontconfig.rs:735` to `762`), so nothing in the process says Japanese or Chinese.
  Which CJK file results is a fontconfig answer; it is recorded in the section "CJK fonts".
- Not traced: the Skia renderer, `transform-scale` text, and the C-side behavior of fontconfig.

### The variable-font axes are not the cause

The installed `Inter Variable` has two axes: `opsz` 14 to 32 (default 14) and `wght` 100 to 900 (default 400),
and OS/2 weight 400 (a table reader over `~/.local/share/fonts/InterVariable.ttf`).
- Slint sets no `opsz` and no weight variation for weight 400; the font stays at its defaults, `opsz` 14.
- Chromium was tested with `font-optical-sizing` auto and none, `font-variation-settings: "opsz" 14, 22 and 32`,
  and `"wght" 900` on the same local font at 12, 14, 16, 20, 22, 24 and 32 px.
  The width of one string was identical in every `opsz` case and in the `wght` 900 case.
  The positive control `font-weight: 900` did change it (237.77 against 222.42 at 12 px).
  So Chromium on Linux applies the weight through the named weight but ignores `font-variation-settings` for this
  system font, and does not apply automatic optical sizing to it.
- Result: both draw the default instance, so the width parity is not an accident of one side applying `opsz`.
  A web font from a page (such as the blog's) can behave differently; it was not tested at other sizes.

### Display scaling is calibrated

- The panel is 34 by 19 cm (EDID bytes 21 and 22 of `/sys/class/drm/card1-HDMI-A-2/edid`), 3840 pixels wide,
  at scale factor 3 (`kscreen-doctor --outputs`): one logical pixel is 3 times 340 mm over 3840, 0.2656 mm,
  95.6 logical pixels per inch.
- The CSS Values and Units Level 4 reference pixel is "about 0.26 mm" (the sentence quoted from W3C, read 2026-10-22:
  "the visual angle of one pixel on a device with a device pixel density of 96dpi and a distance from the reader of an
  arm's length ... nominal arm's length of 28 inches").
  So this desktop's logical pixel matches the definition to the third digit; display scaling is a measured null.
- The CSS definition names a viewing distance; the viewing distance of the human was not measured.

### Physical letter sizes in each app

Computed from the measured ink boxes and the panel's 0.2656 mm logical pixel (1 physical pixel is 0.0885 mm):
- Desktop heading "A" at 22 px: 49 physical pixels, 4.34 mm, 0.171 inch.
- Desktop folder-name capitals at 16 px: 36 physical pixels, 3.19 mm, 0.125 inch.
- Em of 22 px: 5.84 mm; em of 20 px: 5.31 mm; em of 16 px: 4.25 mm; em of 14 px: 3.72 mm.

Android, from the Pixel 9 Pro Fold emulator capture at 390 dpi (2.4375 pixels per dp, font scale 1.0, a subagent's
measurement with about 0.4 dp of reading error): `titleLarge` 22 sp capital height 16.0 dp, `bodyLarge` 16 sp capital
height 11.5 dp.
One dp is 1/160 inch, 0.1588 mm, so those capitals are 2.54 mm and 1.83 mm on a panel that really is 160 dp per inch.
The desktop's capitals at the same nominal sizes are 1.7 times those.

The human's own phone (Pixel 6, read-only `adb`): physical panel 409.4 by 411.9 pixels per inch, density overridden to
356 from 420, font scale 1.5.
One dp there is 356 over 160 pixels, 2.225 pixels, 0.1376 mm.
Android's nonlinear font scaling from Android 14 maps 16 sp at scale 1.5 to 23 dp (expected from the table, not
measured here), so the em is 23 dp, 3.17 mm; 22 sp maps to 27 dp, 3.72 mm.
Against that, the desktop em of 16 px is 4.25 mm (1.34 times) and the em of 22 px is 5.84 mm (1.57 times).
The desktop's 14 px em of 3.72 mm equals the phone's 22 sp em, and 20 px is 5.31 mm.
Physical size at the same viewing distance would therefore make the desktop larger, which is a null for the claim
that the human wants equal physical size: nobody holds a phone and a 34 cm wide panel at the same distance,
and the human's viewing distance on either device was not measured.

### Faces differ across the references

Measured on this machine with Chromium 155 (`CSS.getPlatformFontsForNode` for the platform font and canvas
`measureText` for a fixed pangram of 27 characters, "Hamburgefonstiv Quartz 0123"; heights from `measureText` are
integer-rounded and are not used):
- Inter Variable 16 px: pangram 226.36 px, 14.15 em.
- zh.wikipedia.org body, generic `sans-serif`: Liberation Sans, 213.45 px at 16 px, 13.34 em, 5.7 percent narrower
  than Inter. CJK glyphs on the same page: Noto Sans CJK SC (Regular 112 glyphs, Bold 3).
- GitHub README body: the stack starts with `Mona Sans VF` and the platform font drawn was Noto Sans, 227.79 px,
  14.24 em, 0.6 percent wider than Inter. The page's computed size is 16 px with a 24 px line height.
- `fourthpost` blog body: computed `"Inter UI", sans-serif`, 19.2 px, line height 30.72 px, yet the platform font drawn
  was Liberation Sans (299 glyphs), 255.97 px, 13.33 em; a face named `Inter UI` shows as loaded in `document.fonts`,
  but the paragraph's glyphs came from Liberation Sans in this browser.
  Whether the human's browser draws the blog in Inter was not checked.
- `en.wikipedia.org/wiki/Cat`: the selector for its paragraphs was not found in this run (the earlier run measured
  16 px paragraphs and 14 px contents links); its face is not recorded here.

So the reference pages the human named are drawn in Liberation Sans or Noto Sans in Chromium on this machine,
not in Inter.
Liberation Sans at 16 px is 5.7 percent narrower than Inter at 16 px; Inter at 15 px is 6.25 percent narrower.
Face difference scores 0.94 against the target 0.875 to 0.91: it covers a part of the 2 px, not all.
Firefox's generic sans-serif on this machine is Noto Sans (same width as Inter within 1 percent), so Firefox's
Wikipedia is not narrower than the app by this route.

### Android draws Roboto, with Material's tracking

- The Pixel 6 `fonts.xml` maps `sans-serif` to Roboto (`Roboto-Regular.ttf`); Compose Material 3's `TypefaceTokens` use
  `FontFamily.SansSerif`, and `PlayerTheme.kt:212` calls `MaterialTheme(colorScheme = scheme)` with no custom
  `Typography`.
  Android therefore draws Roboto, not Inter.
- An earlier claim in this doc, that Android's cap heights of 16.0 dp over 22 sp (0.727) show Inter's cap proportion,
  was wrong: the capital-height reading has 0.4 dp of error and cannot separate Roboto (0.711) from Inter (0.727).
- Material's stock scale adds tracking: `bodyLarge` 16 sp with 0.5 sp letter spacing, `titleMedium` 0.2 sp,
  `bodyMedium` 0.2 sp, `labelLarge` 0.1 sp, `bodySmall` 0.4 sp, `titleLarge` 0 sp.
  The desktop uses no tracking; Android text of the same size is therefore wider per line than the desktop's.
- Metrics read from the font files (`doc/research/text-size-perception-sources.md`, default instances, sources and
  hashes there): x-height over em is 0.5459 for Inter, 0.5283 for Roboto 3 and for Liberation Sans, 0.5360 for Noto
  Sans; cap height is 0.7275 for Inter and 0.7109 for Roboto; the mean lowercase advance is 0.5363 em for Inter and
  0.4992 for Roboto (Inter 7.4 percent wider).
  Inter's x-height is 3.3 percent taller than Roboto's.
  Which Roboto build the Android app draws at runtime was not determined.
- The CSS Fonts Level 5 text for `font-size-adjust` says that for a given font size the apparent size varies with the
  design, and names the lowercase height as a determining factor; Legge and Bigelow (Journal of Vision 2011) report
  that faces with larger x-height fractions appear bigger at the same body size, "not proven by rigorous laboratory
  studies".
  The `font-size-adjust` formula would draw a 16 px Inter run at 15.48 px to match Roboto's x-height, a 3.2 percent
  cut against the 12.5 percent the human chose.

### Why the references draw those faces

From the sources in the research note (pinned commits there):
- Chromium's Linux default for the generic `sans-serif` is the family name `Arial`
  (`chrome/app/resources/locale_settings_linux.grd`, lines 168 to 171), resolved through fontconfig;
  on this machine `fc-match Arial` returns Liberation Sans and `fc-match sans-serif` returns Noto Sans.
  That is why Chromium draws Wikipedia's body in Liberation Sans here.
- Firefox on Linux maps `sans-serif` to whatever fontconfig returns for it
  (`gfx/thebes/gfxFcPlatformFontList.cpp`, lines 2362 to 2366), which is Noto Sans here.
- Chromium applies `opsz` automatically for web fonts: `font_custom_platform_data.cc` lines 241 to 245 push an `opsz`
  coordinate equal to the adjusted font size when `font-optical-sizing` is `auto`.
  Inter's own FAQ gives x-height 1118 units at `opsz` 14 and 1056 at `opsz` 32 of 2048, 5.5 percent lower.
  So a page that loads Inter as a web font at 20 px or more is drawn with a lower x-height than the app's `opsz` 14
  instance; the path for a system-installed variable font was not traced in Chromium source, and the measurement in
  this doc shows it ignores `opsz` for the local font.

### The within-app scale

The desktop app's text sizes are 11, 12, 14, 16 and 22 px (`rg "font-size" ui/`).
Before the cut, folder names and track titles were 16 px next to 12 px captions and 14 px labels: a 1.33 step to
the caption.
After the cut they are 14 px, equal to the app's `label-large` and `body-medium`.
Android keeps 16 sp titles next to 12 sp captions and 14 sp labels with the same step, so the step alone does not
separate the two apps.
This is a hypothesis about density and hierarchy, not a measurement.

### The ambient desktop

All measured on this machine.
- KDE general font: `font=Inter Variable,10` in `~/.config/kdeglobals`, 10 pt, 13.3 px at 96 dpi.
  The window title "Music Player" in the human's screenshot is 232 physical pixels wide;
  Inter at 13.333 px at scale 3 is 238 wide, at 12 px 214, at 14 px 249: the desktop draws its title bar near 13 px.
- 14 over 16 is 0.875, the KDE font over 16 is 0.833: the desktop's own text is a size class below 16 px.
- The four reference pages, computed font size (Chromium 155, defaults, no zoom):
  - `en.wikipedia.org/wiki/Cat`: paragraphs 16 px; by characters of visible text, 14.4 px carries 54,457,
    16 px carries 50,403 and 14.08 px carries 14,701; contents links 14 px.
  - `zh.wikipedia.org` for the same article: paragraphs 16 px with a 26 px line height, contents links 14 px.
  - the `README.md` of this repository on GitHub: 16 px carries 6,267 characters, 13.6 px (code) 5,069,
    14 px 887; dark theme.
  - `fourthpost`: 19.2 px for all body text, light text `rgb(238, 238, 238)` on `rgb(24, 24, 24)`.
- Continuous prose is 16 px or more on every page, and the non-prose text is 13.6 px to 14.4 px.
- The platforms' own defaults, from their documentation and source (the research note has the quotes):
  Windows 11 Body 14 epx and Caption 12 epx; GNOME's default font Adwaita Sans 11 pt, 14.67 px, with libadwaita list
  subtitles at 14.67 divided by 1.2, 12.22 px; KDE's default 10 pt, 13.33 px, with Kirigami list titles at the default
  font and subtitles at 8 pt, 10.67 px; macOS body 13 pt (17.33 px by the 4/3 convention).
  Material 3 list headlines are 16 sp and supporting text 14 sp, in dp units of 0.1588 mm.
  The three desktop environments with a stated default put body and list text at 13.3 to 14.7 px.
  No platform document found states a viewing distance in numbers except the CSS reference pixel (28 inches) and
  Microsoft's 10 feet for a TV.

### What the human's two-pane screenshot and the phone captures show

- The blind panel test below ranked Inter 14 px rows as the right size for a music list on a 3x panel.
- Android's folded and unfolded captures (the build of 0.1.11, emulator at font scale 1.0, folded 443 dp wide,
  unfolded 852 dp wide, invented library) were read by the lead agent: in the folded one, titles at 16 sp with
  12 sp captions are proportioned like a normal phone list; in the unfolded one, the two 22 sp pane titles read as
  clearly the largest text, as in Material's stock proportions.
  That is a language model's reading of an image (see the limits below), not a human's perception.
- The current desktop build (20 px titles, 14 px names and track titles, 12 px captions) in the e2e captures at 599 and
  600 logical pixels wide (scale 1, the repository's nested compositor, the fixture's generated names): the lead agent
  read the folded layout and the two-pane layout as balanced, with the pane title clearly above the 14 px rows and the
  12 px caption clearly below them.
  Those captures are at scale 1 on a 600 by 800 canvas, not the human's scale 3 panel, so they show the proportions
  and not the physical size.

### CJK fonts

Correction, found after the first version of this section was committed: the measurements below were taken with a
scratch `XDG_CONFIG_HOME`, which removes the human's user fontconfig fragment
`~/.config/fontconfig/conf.d/50-noto-cjk-fallback.conf` (present, 420 bytes, dated 2026-08-11).
`doc/troubleshooting/slint-hangul-split-jamo-droid-sans-fallback.md` documents that without that fragment fontique's
`FcFontMatch` returns Droid Sans Fallback, and with it Noto CJK is chosen.
So the Droid Sans Fallback result describes a machine without the fragment, not the human's session.
The earlier text claimed the desktop app draws Droid under the human's locale and queued a decision about embedding a
CJK font; both claims are withdrawn.
A rerun with the real `XDG_CONFIG_HOME` is assigned to the agents that build the comparison; its result is not in this
section yet.

Measured by a subagent in the repository's nested compositor (`--headless --scale 3`, the built release binary,
an invented library of three silent one-second files named with 直骨令刃海角, クリーチャー and a Latin name, a transient
unit capped at 4 GiB and 200 percent CPU, scratch `XDG_CONFIG_HOME`), by `/proc/<pid>/maps`,
`strace --follow-forks --trace=openat` and an image match (ImageMagick RMSE on thresholded ink, lower is closer)
against Chromium renderings at 14 px, scale 3:
- With `LANG=en_US.UTF-8`, which is what this machine's user manager has (`systemctl --user show-environment`),
  and no user fontconfig fragment, the app draws both the Han string and the kana from
  `/usr/share/fonts/google-droid-sans-fonts/DroidSansFallbackFull.ttf` (Droid Sans Fallback, a plain `.ttf`).
  No `.ttc` and no Noto Sans CJK file is opened in the trace, and the maps hold no `NotoSansCJK` line.
  RMSE for the Han string: Droid Sans Fallback 0.0326; Noto Sans CJK SC 0.1586, HK 0.1625, KR 0.1629, TC 0.1638,
  JP 0.1678. For the kana: Droid 0.0269; every Noto Sans CJK face 0.0984.
- With `LANG=ja_JP.UTF-8` the app maps `NotoSansCJK-Regular.ttc` and `-Medium.ttc` and no Droid file, and the Han glyphs
  match the JP face best (0.0698; KR 0.0867). With `LANG=zh_CN.UTF-8` it maps the same two files and the Han glyphs
  match the SC face best, with a weaker margin (0.1048 at weight 450 or 500; ambiguous at weight 400).
- Chromium and Firefox draw untagged Han and kana with Noto Sans CJK SC; Android orders zh-Hans first in its
  `fonts.xml` and draws Noto Sans CJK from `NotoSansCJK-Regular.ttc`.
  Without the fontconfig fragment the app's CJK glyphs come from a different font than every reference;
  with the fragment (the human's session) the expected pick is Noto Sans CJK, not yet confirmed by a run.
- The Latin text is drawn from the Inter Variable embedded in the binary (`import "fonts/InterVariable.ttf"` in
  `ui/app_window.slint`; the bytes occur at offset 3253407 of the release binary, and the file's SHA-256 equals the
  installed `~/.local/share/fonts/InterVariable.ttf`), not from a system file.
  This is why Chromium's local Inter and the app's embedded Inter agree.
- The fontconfig CLI is not what the app calls: a C reproduction of fontique's call (character-set pattern, no language,
  `FcConfigSubstitute` then `FcFontMatch`) returns Noto Sans CJK SC under `LANG=en_US`, yet the app draws Droid.
  Why Droid wins is not determined.
  Speculation: the app takes the first family in the generic sans-serif list that covers the character
  (`fontique-0.11.1/src/collection/query.rs:121`), which is a sort result under the process locale,
  and Droid Sans Fallback appears there ahead of the script fallback; the locale-dependent change of font matches this
  reading and does not prove it.
- Not measured: the glyph size of Droid Sans Fallback against Noto Sans CJK at the same em; the human's real session
  locale beyond the user manager's `LANG` (the app started from the desktop may differ).
  Whether the CJK difference contributes to the feel is not tested: the human named Latin text in their screenshot
  ("Music", "A", "ABM", "32ki", "Creature").

### The human's perception test, round 1

A local page (`text-size-test.html`, scratch only) drew samples in the human's own browser at their display's real
size and asked, for each, how its text looks next to a Wikipedia-style article paragraph (default sans-serif, 16 px),
and whether the size suits a list row or a pane title.
The sample order was random and unlabeled.
One person, one session (2026-10-22 11:35 UTC), so these are ordinal answers from n = 1.

The display it ran on is not the one of the earlier screenshot:
- The page reported `devicePixelRatio` 2, a 1026 by 1840 CSS px window and a 1080 by 1920 screen.
  `kscreen-doctor --outputs` lists three monitors (DP-3, HDMI-A-1, DP-2) at scale 2 in portrait geometry 1080 by 1920,
  and EDID bytes 21 and 22 of each give 60 by 34 cm.
  So the test ran on a 3840 pixel panel 600 mm long, 0.156 mm per physical pixel, 0.3125 mm per CSS px
  (about 1.18 times the 0.2656 mm of the HDMI-A-2 panel at scale 3 that the app screenshot came from).
  Which of the three monitors the browser sat on was not recorded.

Answers (scale: much smaller -2, smaller -1, about the same 0, larger 1, much larger 2):
- List rows, Inter Variable: 13 px gave -1 and "too small"; 14 px gave 0 and "right"; 15 px gave 1 and "right";
  16 px gave 1 and "too large"; 18 px gave 2 and "too large"; 20 px gave 2 and "too large".
- The control (the default sans-serif at 16 px, the same font size as the reference) gave 2 and "too large" as list
  rows, and 1 when shown as part of the typeface comparison.
- Pane titles, Inter Variable: 18 px gave 1 and "right"; 20 px gave 1 and "right"; 22 px gave 2 and "too large";
  24 px gave 2 and "too large".
- Same size number 16 in four typefaces: Inter Variable 1, Liberation Sans 1, Noto Sans 1, the control 1.

The browser was Firefox ESR, not Chromium: the human said `xdg-open` opens their Firefox ESR, which the lead agent
learned after the tests and after verifying every page only in Chromium.
A subagent then read only the font, zoom and scaling preferences of the human's Firefox profile
(`prefs.js`; there is no `user.js`) and rendered the pages in a private temporary profile:
- The human's preferences set `font.name.sans-serif.x-western` to "Inter Variable", `font.name.serif.x-western` to
  "Inter Variable" and `font.name.monospace.x-western` to "JetBrains Mono".
  Nothing else changes size: `font.minimum-size`, `font.size.*`, `layout.css.devPixelsPerPx`, `ui.textScale` and
  `browser.display.use_document_fonts` are unset, and the three `file://` per-site zoom rows are 1.0.
- So in the human's Firefox the generic `sans-serif` is Inter Variable, and the test page's reference paragraph
  (`font: 400 16px/1.5 sans-serif`) and its control sample were Inter at 16 px, the same face as the samples.
  Verified by image match in Firefox with those preferences (RMSE 0.0000 against Inter; Noto Sans 0.312, Liberation
  Sans 0.315); with Firefox defaults it is Noto Sans, and in fresh headless Chromium Liberation Sans.
  The Chromium-based statements in this doc about Liberation Sans on Wikipedia describe Chromium,
  not what the human sees.
  Any page whose stack ends in `sans-serif` and names no installed font, such as Wikipedia's body, is Inter in the
  human's Firefox (inferred from the stack, Wikipedia itself was not rendered).
- Consequence for the perception test: the control (Inter 16 px as list rows against Inter 16 px as a paragraph) was
  the same face and the same size, and the human still answered "much larger" and "larger".
  The face and the renderer are therefore not what made the rows look bigger: it is the row format.
- Firefox renders the replica like Chromium: pixel-identical with the human's preferences and with defaults (the
  replica names its fonts), 34 elements, mean difference 0.2 px at scale 1 and 0.3 px at scale 2 (in the nested
  compositor), all Latin elements within 1 px; only the two kana folder names differ (3.5 to 7 px narrower in Firefox)
  because the replica sets `font-kerning: normal` and Firefox then applies the font's `palt` kerning to kana.
- Neither browser applies Inter's `opsz` axis to the local font, and `font-variation-settings` has no effect for it in
  either (only `font-weight` moves the width), so optical sizing is not a Firefox versus Chromium difference here.
- Not tested: Firefox at scale 3 or at fractional scales, a windowed Firefox on the actual monitors with the human's
  scaling path, and Inter loaded as a web font in Firefox.
An agent is now also measuring Firefox's glyph edges, x-height snapping and ink mass against the real app.

What it shows, and its limits:
- The 2 px cut is reproduced on the human's display in a controlled, unlabeled test:
  rows at 14 px look the same size as 16 px article text, 16 px rows look larger and too large,
  titles at 18 and 20 px are right and 22 px is too large.
  The sizes the human chose by feel (20 and 14) are inside the "right" answers; the original 22 and 16 are not.
- The control failed as a size reference: text of exactly the reference's font size looked one to two steps larger
  when shown as short list rows than when shown as a wrapped paragraph, and the two control answers differ by one step
  (test-retest noise of one step).
  So the "larger" answers measure how the row format looks against a paragraph, not only the font size.
  Speculation: short isolated lines with row padding read as larger than the same text in a dense paragraph;
  round 2 (`?round=2` of the page) shows paragraph samples against the paragraph and rows against a rows reference, to
  separate the format from the size.
- Typefaces were not told apart: Inter, Liberation Sans and Noto Sans at 16 px received the same answer as the control,
  although Liberation Sans is 5.7 percent narrower. In this one test the face difference was not perceived as a size
  difference, a null for the face hypothesis with the limits of n = 1 and a one-step resolution.
- The preferred sizes (rows 14 to 15 px, titles 18 to 20 px) are in CSS px and sit on a monitor whose CSS px is 1.18
  times larger than the monitor of the earlier choice (rows 14 px, titles 20 px, both chosen on the scale 3 panel).
  The same CSS px sizes were accepted on both. Speculation: the judgment follows the CSS px convention more than the
  physical size; two monitors, an unmeasured viewing distance and a different task do not prove it.
- The test showed web renderings, not the Slint app itself. A second page that shows real screenshots of the app at
  true size is being prepared.

### Physical size in millimetres

The human's position: a pixel has no fixed size, only a physical length can be trusted, and the metre is defined from
the speed of light.
A CSS px, a Slint logical px, a dp and a point are all conventions on top of a display's real pixel density, so every
comparison here is also stated in millimetres.

The exact panel sizes come from the first detailed timing descriptor of each monitor's EDID
(`/sys/class/drm/*/edid`, bytes 12 to 14 of the descriptor at offset 54), which is exact to 1 mm; the basic-block
sizes used earlier (bytes 21 and 22) are rounded to 1 cm.
- HDMI-A-2 (scale 3, landscape): 344 by 193 mm, 3840 by 2160 pixels, 11.163 physical pixels per mm.
  One CSS or logical px is 3 / 11.163 = 0.2688 mm, not the 0.2656 mm computed from 34 cm; every millimetre figure
  earlier in this doc that used 0.2656 is 1.2 percent low.
- DP-2 and HDMI-A-1 (scale 2, portrait): 600 by 340 mm, 6.400 pixels per mm; one CSS px is 0.3125 mm.
- DP-3 (scale 2, portrait): 597 by 336 mm, 6.432 pixels per mm; one CSS px is 0.3110 mm.
- The phone (Pixel 6, density 356): about 16.1 pixels per mm (409.4 per inch), one dp 0.1376 mm.

Inter's x-height is 1118 of 2048 units (0.5459 em) and its capital height 1490 (0.7275 em) at `opsz` 14.
What the human's perception test accepted, as physical heights on the monitor it ran on (0.3125 mm per CSS px):
- Rows, x-height: 13 px is 2.22 mm ("too small"), 14 px 2.39 mm ("about the same" as the reference, "right"),
  15 px 2.56 mm ("right"), 16 px 2.73 mm ("too large").
- Titles, capital height: 18 px is 4.09 mm and 20 px 4.55 mm ("right"), 22 px 5.00 mm and 24 px 5.46 mm ("too large").

What the human chose earlier on HDMI-A-2 (0.2688 mm per CSS px):
- Rows at 14 px are an x-height of 2.05 mm (accepted) and at 16 px 2.35 mm (rejected as too large).
- Titles at 20 px are a capital height of 3.91 mm (accepted) and at 22 px 4.30 mm (rejected).

So the two monitors disagree in millimetres: 2.05 mm was accepted on the first and 2.22 mm called too small on the
second, and 2.35 mm was too large on the first while 2.39 to 2.56 mm was right on the second.
The accepted windows differ by a factor near 1.16, and the CSS px sizes of the two monitors differ by 1.163
(0.3125 over 0.2688).
Two readings fit this and the data cannot separate them:
- Speculation: the judgment follows the CSS px convention (14 to 15 px rows, 18 to 20 px titles), not physical length.
- Speculation: the judgment follows visual angle, and the 600 mm monitors are viewed from about 1.16 times the distance
  of the 344 mm panel.
Eye distances to the monitors were not measured. The millimetre mode of the test page asks for them
(`text-size-test-mm.html`, scratch only): it sizes the samples by x-height and capital height in millimetres, draws
a 100 mm ruler for a check against a physical ruler, and records the angle of each x-height at the stated distance.

### The replica beside the real app, on the human's display

The human's own comparison, and the instrument behind it:
- A static HTML and CSS replica of the app's wide two-pane window (`replica.html`, scratch only) uses the app's own
  numbers, colors, icon paths and `font-optical-sizing: none`, with the four tuned text sizes as inputs.
  An agent measured it against the real release binary in the nested compositor at scale 1 and 2, in dark and light,
  with the real fontconfig kept (Noto Sans CJK mapped, no Droid Sans Fallback): every ink box of 17 elements is within
  1 px in x, y and width (within 0.5 px at scale 2) and heights differ by at most 1 px.
  The remaining differences: folder names after the first in a row sit about 1 px left, the selected mode segment's fill
  bleeds about 0.4 px outside its outline in the app only, and one deck title is 1 px off in x and y.
- The human put the replica beside the real app on one monitor and said the two look identical at identical sizes.
  That closes "Slint draws bigger or differently than Chromium" with the human's own eyes, in addition to the
  width and height measurements.
- Build A (titles 20, names 14, track titles 14) looked right to the human.
  Build B (titles 22, heading letter 22, names 16, track titles 16 in the source) needed the replica set to 22, 22, 16
  and 17 (track title) to look the same.
  Measured afterwards, in the nested compositor at scale 1 (dark and light), 2, 1.25 and 1.5: real build B draws all
  seven track titles at 16 px, the replica at track 16 matches them with 0 px difference in x, y, width, height and
  baseline (including the bold playing row and the mixed Latin and Han row), and the replica at 17 is 6.4 px wider on
  average and about 1.6 px taller.
  The advance of "Lantern Row" in Chromium is 94.58 px at 16 and 100.50 px at 17 against a real ink width of 93, and
  `tracks.slint:379` is a fixed size with no scale or transform.
  So the 17 is not a geometric difference: the same pixels needed a larger setting to look alike to the human, only for
  the track titles, while 22, 22 and 16 matched as numbers.
  Why the human needed 17 is not explained by the widths.
  Asked what they saw, the human said the real app's track titles looked bigger than the replica's at 16 px, with
  identical measured boxes (and the real app's ink mass lower, not higher), while at 14 px the two looked identical.
- A subagent then compared glyph rasterization of the real app and the replica in Chromium at scales 1, 2 and 3
  (captures bit-identical across reruns, so every difference below is deterministic; grayscale anti-aliasing on both
  sides; the bundled and the system `InterVariable.ttf` have the same sha256):
  - The font has no `fpgm`, `prep` or `cvt` tables, so skrifa 0.44 uses its autohinter in `SmoothMode::Lcd`, which
    hints vertically only. Widths and spacing are therefore unchanged: "Lantern Row" is 278 device px wide at scale 3
    on both sides, and same-glyph distances stay within 0.2 px of the font's advances on both sides.
  - The autohinter rescales each run vertically so the x-height lands on a whole pixel, rounding up from a fraction of
    0.375 (`skrifa-0.44.0/src/outline/autohint/metrics/scale.rs` lines 160 to 185) and scaling the whole run by the
    fitted over the scaled value when that moves under 2 px.
    The overshoot value was calibrated from the observations (between 0.5495 and 0.556 em), not read from the font.
  - At 16 px the real app's x-height and cap height are, against the replica: scale 1, 9.00 and 12.00 against 9.00 and
    12.00 (equal); scale 2, 18.00 and 24.00 against 17.52 and 23.32; scale 3, 27.00 and 36.00 against 26.30 and 35.00
    (about 3 percent taller, the baseline one device pixel higher).
    The replica at 17 px, scale 3, measures x-height 27.73 and cap 37.00 and 296 device px wide, so the real app at 16
    sits halfway between the replica's 16 and 17 in height and at 16 in width.
  - At 14 px: scale 1 equal; scale 2 the app is 4.6 percent taller in x-height; scale 3 x-height equal (23.00) and cap
    0.56 device px shorter.
    At fractional scales the app is shorter, not taller: x 11 and cap 14 at scale 1.25 against 14.58 for the replica's
    cap, and x 13 and cap 17 at scale 1.5, which the calibrated rule predicted before measuring.
  - Stems are 3 to 7 percent lighter in the app at 16 (coverage sum 1.61 against 1.70 at scale 3) with equal edge
    ramps, so the app's edges are not crisper horizontally; ink mass of the track titles, app over replica, is 0.946 at
    scale 1, 0.985 at scale 2 and 1.003 at scale 3 (build A: 0.937, 0.997, 0.965).
    The earlier "5 to 8 percent lighter" therefore holds at scale 1 only.
  - Whether this explains "bigger at 16, identical at 14" depends on the scale of the monitor the comparison ran on.
    At scale 3 (the 15.6 inch panel) it fits: plus 3 percent in both heights at 16 px, 0 percent or minus 1.8 percent
    at 14 px.
    At scale 2 (the 27 inch panels) it predicts the opposite (14 px is more affected, 4.6 against 3.0 percent), and at
    scale 1 there is no difference.
    The effect is about half the step from 16 to 17, leaves width alone, and also acts on the 16 px folder names and
    the 22 px titles (plus 2 percent in cap height), which the human found matching, so it does not single out the
    track titles.
    The human's comparison ran on the 15.6 inch panel (scale 3), which is the case where the effect fits: 16 px about 3
    percent taller, 14 px not.
    That made the autohinter the candidate for "track titles looked bigger at 16 but identical at 14".
- The same comparison with Firefox ESR 153.5.1 (the human's browser; the replica at scales 1, 2 and 3 in the nested
  compositor with a private profile, captures bit-identical across reruns) nulls that candidate:
  - Firefox applies the same x-height fit as the app: x-height and cap height are equal to the app's at every size
    and scale measured (16 px: 9 and 12, 18 and 24, 27 and 36 at scales 1, 2, 3; 14 px: 8 and 11, 16 and 21, 23 and
    30), where Chromium at scales 2 and 3 is nominal.
  - Ink mass over the app's is 1.000 at all three scales in both builds (Chromium 0.997 to 1.067), the share of pixels
    at 90 percent of peak or more is within 0.2 percent (8243 against 8258 at scale 3), and edge fuzz is equal
    (0.075 and 0.075 at scale 3).
    The earlier "app 5 to 8 percent lighter" is a Chromium property only.
  - Mean luma difference to the app after whole-pixel alignment: Firefox 0.15 to 0.38 at scale 3, Chromium 0.90 to 1.18.
  - What remains between the app and the Firefox replica: the app's baseline is 1 device pixel (0.33 logical px)
    higher at scale 3 (at 14 px as well as 16 px), the bold playing row's stems differ in weight in both directions
    depending on scale (6 to 9 stems), and the app's stem core is slightly thinner at scale 3 (width at 75 percent of
    peak 0.89 against 1.02 device px).
  - None of these singles out 16 px. So the rasterization candidate is null for the human's comparison, the geometry
    was measured equal, and why the 16 px track titles looked bigger remains unexplained by any instrument here.
    The human's perception is the only instrument left, and the text size setting lets them choose the size.
  - Not measured: the human's real fontconfig and `gfx.font_rendering` outcome in a windowed Firefox (the private
    profile used the human's font preferences without reading the rest of the profile), and perception itself.
- The human then asked for a text size setting in the real app, with a stepper of 1 px for each element.
  It shipped in release 0.1.12 (Settings, Text size): four rows (pane titles, group heading letter, folder names, track
  titles) with defaults 20, 20, 14 and 14 px, limits 10 to 32 px, a Reset row, and persistence in `session.json`.
  Desktop only; Android follows the system font size.
  Verified: 1592 package tests, lints, the whole end-to-end suite (123 scenarios), and the 0.1.12 release assets
  against `SHA256SUMS` with one attestation check.
  Not verified by the human yet: how the setting feels on the 15.6 inch panel at the sizes they prefer.

## Hypotheses

Each is scored against the target (0.91 for the titles, 0.875 for the names and track titles).
Measured and null are results of an instrument; speculation is not.

- Slint draws bigger than Chromium: null. Width ratio 1.000 at every size from 12 to 24 px; "A" height within 3 percent;
  and the human saw the replica and the real app as identical side by side.
- Renderer weight: null for femtovg (0.94 to 1.005 of Chromium's mass, depending on scale).
  Renderer hinting: measured difference, vertical only. The autohinter makes the x-height and cap height of 16 px
  text up to 3 percent taller at scales 2 and 3 (equal at scale 1, shorter at fractional scales), with widths equal.
  Null for "bigger at 16 px" as the human saw it: Firefox, the browser they compared with, applies the same fit and
  matches the app in height, ink mass and edge sharpness.
  The software renderer, which is not the shipped default, is not exercised.
- Font file or axes (`opsz`, `wght`): null. Both sides draw the default instance; Chromium ignores `opsz` for this
  system font (positive control moved width, `opsz` did not).
- Display scaling: null. 0.2656 mm per logical pixel against the CSS reference pixel's 0.26 mm.
- Face difference between the references and the app: partial measured. Liberation Sans is 5.7 percent narrower than
  Inter in the same pangram and 3.3 percent lower in x-height; Noto Sans is not. The width covers about half of the
  2 px for the pages drawn in Liberation Sans, the x-height about a quarter, and neither applies to the human's
  Firefox, which draws generic `sans-serif` as Inter Variable (see "The human's perception test, round 1").
  In the human's own test the faces were not told apart at 16 px (see "The human's perception test, round 1").
- Equal physical size across phone and desktop: not what the human wants. The em of 16 px is 1.34 times the phone's
  16 sp em, and 14 px is already larger than the phone's 16 sp em (3.72 against 3.17 mm).
- Size class by role (a desktop list or label row at 14 px, prose at 16 px and above): speculation, the best fit
  to the measurements of the pages and of the KDE font, and not tested on the human.
- Light text on a dark ground looks larger or bolder (irradiation): speculation; no primary source found.
  The only adjacent result is Westheimer 2007 on black-white border position (about 0.4 minutes of arc), which does not
  mention text, and the legibility studies (Piepenbrock 2014, Dobres 2016 and 2017) report a dark-on-light advantage
  that grows as characters get smaller, which concerns legibility and not perceived size.
- The within-app hierarchy: speculation, see the section "The within-app scale".
- Mixed-script lines: without the human's fontconfig fragment the desktop's CJK glyphs come from Droid Sans Fallback
  while every reference uses Noto Sans CJK; in the human's session the fragment selects Noto CJK, so this is open
  until the rerun, and the five texts the human named are Latin.

The only instrument that can measure perceived size is the human; a controlled test on them is proposed in
"Not exercised".

## Verification

Versions: Slint 1.18.1 (`slint-viewer --version`; the crates were bumped to 1.18.1 in `2cfb4b8c13`),
Chromium 155.0.8059.84, Firefox 153.5.1esr, Inter Variable at `~/.local/share/fonts/InterVariable.ttf`
(fontconfig version 262210), KDE Plasma on Wayland, `kscreen-doctor` output scale 3 for `HDMI-A-2`.
The human's screenshot is `Screenshot_20261022_061318.png`, 3838 by 2045 physical pixels (from the scale 3 output).
It shows the human's library, so it is not copied into the repository; only numbers measured from it are recorded here.

Harness, a page with the same string in two faces and two sizes, positioned so each row can be cropped:

```html
<!-- compare.html -->
<style>
  html, body { margin: 0; background: #000; color: #e6e1e5; }
  .row { position: absolute; left: 20px; white-space: nowrap; line-height: 1; font-weight: 400; }
  .inter { font-family: "Inter Variable"; }
  .noto { font-family: "Noto Sans"; }
</style>
<div class="row inter" style="top: 20px; font-size: 16px">NAME</div>
<div class="row inter" style="top: 60px; font-size: 14px">NAME</div>
<div class="row noto" style="top: 100px; font-size: 16px">NAME</div>
<div class="row inter" style="top: 180px; font-size: 16px">H</div>
<div class="row inter" style="top: 220px; font-size: 16px">x</div>
```

Render at scale 3 and measure each row's ink box (the crop covers one row; the threshold decides the box):

```bash
chromium-browser --headless=new --no-sandbox --force-device-scale-factor=3 --hide-scrollbars --window-size=400,400 \
  --screenshot=compare-chrome.png file:///path/to/compare.html
magick compare-chrome.png -crop 1140x102+0+36 +repage -alpha off -colorspace Gray -threshold 35% -trim +repage \
  -format "%w %h\n" info:
```

The `opsz` test is a Chrome DevTools Protocol page that sets each CSS in turn on a span and reads
`getBoundingClientRect().width`; its positive control is `font-weight: 900`.
The font's axes were read with a small TypeScript reader of the `fvar` and `OS/2` tables.
The per-page platform fonts come from `CSS.getPlatformFontsForNode` on the first body paragraph.

Measurements that agree (renderer, font and axis parity, physical pixels at scale 3):
- app 325 by 38, Chromium 325 by 37 for a folder name at 16 px; Inter against Noto Sans 325 and 328.
- Ink mass 0.2496 (app) and 0.2579 (Chromium), 3 percent apart.
- Sweep widths equal at 12 to 24 px.
- `opsz` and `wght` variation settings: identical widths in Chromium; `font-weight` moved them.

Measurements that differ:
- Inter 14 px: 285 wide against 325 at 16 px.
- Pangram widths per em: Inter 14.15, Liberation Sans 13.34, Noto Sans 14.24.
- 1 dp is 0.1588 mm; a logical pixel on this panel is 0.2656 mm.
- Desktop interface font 13.3 px, pages' non-prose text 13.6 to 14.4 px, pages' prose 16 to 19.2 px.

### Blind panel test, by a language model

Seven dark panels (a three-line paragraph and three list rows) were rendered in Chromium at scale 3: Inter Variable at
13, 14, 15, 16, 18 and 20 px, and Liberation Sans at 16 px (the face Chromium uses for `sans-serif` on Wikipedia),
shuffled with a fixed seed, and judged by a subagent that had not seen the key or the hypothesis.
- Its order from smallest to largest: Inter 13, 14, 15, Liberation 16, Inter 16, 18, 20.
  It could not tell Inter 15 from Liberation 16 (equal paragraph and row widths).
- It named Inter 14 and 15 as typical article body text and Inter 13 and 14 as right for music-list rows;
  Inter 16, 18 and 20 as too large for rows.
- It named Liberation as the only different face, with a lower x-height and narrower letters.
- Its own limit: it estimated sizes from pixel extents, not from perceived comfort.
- Limit of this test: a language model reading an image has no viewing distance, no display density and no
  reading comfort; it supports the ordering and the Liberation against Inter equivalence, and nothing about a human's
  perception.
  The lead agent saw the 20 px panel before the judging (it read the key first by mistake); the judge did not.

## Verified workarounds

- Write the five sizes as plain lengths, 20 px for the two pane titles and the group heading
  (`ui/bars.slint:353` and `385`, `ui/folders.slint:311`) and 14 px for the folder names and track titles
  (`ui/folders.slint:320` and `333`, `ui/tracks.slint:379`), commit `9783adb90d`.
  Tradeoff: the desktop sizes now differ from Android's by 2 on purpose, so the two apps no longer share one type
  scale; any later change to the scale has to be made at these sites, because no `Theme` token carries them.
  The width probe in `folders.slint` and the visible name use the same 14 px, so a selected name never re-wraps.
  1546 desktop tests passed.
  The human confirmed "Looks good now."

## What does not work

- Cutting the two tokens by 2 px (`Theme.title-large - 2px`, commit `574de143f6`): it gave the sizes the human first
  asked for, and the human then said the tokens are unreliable and must not be used.
  Measured, the tokens were exact; the instruction was followed anyway.
- Plain 22 px and 16 px, Android's numbers (commit `1b078497b9`): matches Android's source and its measured cap heights
  and is the same size the desktop had; the human said it still felt wrong.
- Changing the font: Inter against Noto Sans moves the width under 1 percent, against 12 percent for the cut.
- Reading Wikipedia's 14 px as the cause: Wikipedia's article prose computes to 16 px; the 14 px is on contents links.
- Reading Slint as drawing larger than Chromium: the sweep and the screenshot comparison show equal widths.
- Reading the viewing-distance as equal-physical-size: the desktop em is already larger than the phone's.

## Not exercised

- A controlled test on the human: the same list at several sizes and faces in the same panel, asking which feels like
  Wikipedia's 16 px; the only instrument that measures perception.
- Firefox at scale 3 (headless `--screenshot` ignored the `layout.css.devPixelsPerPx` preference and wrote a 1 times
  image; the logical widths still equal Chromium's).
- Viewing distance on the desktop and on the phone; a second human; Wikipedia in dark mode.
- The software and Skia renderers; Slint's `transform-scale` text.
- The Android build's text as drawn on the human's Pixel 6 at font scale 1.5 and density 356, beyond the arithmetic
  above.

## Upstream filing decision

Nothing is filed and no draft is kept: the audit fails the first constraint.

1. Is it really upstream's fault? No. Slint drew 16 px as 16 px at scale factor 3, the same width as Chromium,
   and the same default `opsz` instance. The difference is in unit conventions, faces and our choice of size.
2. Can upstream fix it? Not applicable; there is nothing to fix.
3. Are they supporting this use case? Not applicable.
4. Would the repo welcome our contribution? Not checked, because the first constraint already fails.
5. Will they fix it? Not applicable.
6. Have we prototyped a minimal fix? The fix is on our side (commit `9783adb90d`).

`.out-of-scope/` was read (`bun-install.md`, `cargo-workspace.md`, `claude-code-upstream-bugs.md`, `codex-harness.md`,
`jsr.md`, `lightningcss.md`, `low-impact-typescript-formatting.md`, `module-es-monolith.md`,
`pi-gpt55-long-context.md`, `terminal-title-fork-parity-tests.md`, `typescript-project-references.md`);
no entry names Slint, Inter, Android typography or the browsers.
No upstream tracker was searched, because there is no upstream defect to report.

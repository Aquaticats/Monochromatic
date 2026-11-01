# Author background or border on `<progress>` disables native theming in Gecko and Blink, making `accent-color` inert and reverting the fill to engine defaults

Styling a native `<progress>` element's own background or border (for
example `background-color: transparent` to get a see-through track)
switches both Firefox and Chromium from the natively themed widget,
which honors `accent-color`,
 to an unthemed fallback rendering that
ignores it.
 The fallback fill is a UA blue in Firefox
(`rgb(0, 100, 180)`,
 plus a blue-tinted `rgb(143, 143, 157)` border)
and exactly CSS `green` on `gray` in Chromium (`rgb(0, 128, 0)` on
`rgb(128, 128, 128)`).
 In that fallback state,
 only the engine-specific
pseudo-elements (`::-moz-progress-bar`,
`::-webkit-progress-bar`/`::-webkit-progress-value`) recolor the bar.

## Symptom

No error is logged anywhere;
 the failure is visual.
 For a bar meant to
render a near-white fill on a transparent track (the wc frequency
bars):

- Firefox paints the fill in its default blue although the element
  declares `accent-color`,
   and keeps a one-pixel border in
  `rgb(143, 143, 157)`,
   a slightly blue-tinted gray,
   even with the
  element's background transparent.
- Chromium paints a green fill on an opaque gray track,
   which reads as
  "`accent-color` was ignored" although the actual trigger is the
  author background.

Surface patterns that trigger the fallback (verified in both engines):
`background-color` on the element,
 opaque or `transparent`;
 `border`
on the element;
 `appearance: none`.

## Root cause

Both engines implement the same architecture,
 specified by
[CSS-UI-4 §7.2.1][css-ui-4] ("Properties Disabling Native
Appearance"):
 a widget with author-origin border or background
properties is rendered as a "devolved widget" instead of the native
one.
 `accent-color` only feeds the native painting path,
 so it dies
with the devolution.
 Source pins:
 Gecko at
`mozilla-firefox/firefox@d92a7ec0e622782fe62529bb3a4809780da01d6c`
(HEAD;
 every file cited below was diffed against the
`FIREFOX_151_0_RELEASE` tag `79eeaf756b8cee11ca4c173a7b86b04b22db707b`
and is byte-identical except where noted),
 Chromium at the
149.0.7827.55 tag `chromium/chromium@3188f8a607ae7e067593be8aab7f02d2451fec07`.

### Gecko

The devolution trigger.
 `layout/forms/nsProgressFrame.cpp:237-240`:

```cpp
bool nsProgressFrame::ShouldUseNativeStyle() const {
  return StyleDisplay()->HasNativeAppearance() &&
         !Style()->HasAuthorSpecifiedBorderOrBackground();
}
```

consumed by `widget/nsNativeTheme.cpp:144-161` (`IsWidgetStyled`
returns `!progressFrame->ShouldUseNativeStyle()` for
`StyleAppearance::ProgressBar`),
 which `widget/Theme.cpp`
(`ThemeSupportsWidget`,
 lines 1578/1599) uses to reject the widget.
The flag is set by Stylo whenever **any** border or background
longhand has an author-origin cascaded value,
 regardless of the value,
so `background-color: transparent` and `border-style: none` both
count;
 `servo/components/style/properties/cascade.rs:1246-1252`:

```rust
if self
    .author_specified
    .contains_any(LonghandIdSet::border_background_properties())
{
    builder.add_flags(ComputedValueFlags::HAS_AUTHOR_SPECIFIED_BORDER_BACKGROUND);
}
```

Only the element's own style is consulted,
 never the pseudo's,
 which
is why styling `::-moz-progress-bar` alone does nothing on a pristine
element.

Why `accent-color` works only while themed.
 The themed painter reads
the element's computed accent;
 `widget/ThemeColors.cpp:106-121`:

```cpp
ThemeAccentColor::ThemeAccentColor(const ComputedStyle& aStyle, ColorScheme aScheme)
    : mDefaultPalette(...) {
  const auto& color = aStyle.StyleUI()->mAccentColor;
  if (color.IsAuto()) {
    return;
  }
  ...
  mAccentColor.emplace(accentColor);
}
```

feeding `Theme::ComputeProgressColors` (`widget/Theme.cpp:435-442`,
`return std::make_pair(aColors.Accent().Get(), aColors.Accent().GetDark())`);
the default when `auto` is `sDefaultAccent = 0xff0060df`
(`widget/ThemeColors.h:15-16`),
 matching the measured pristine fill
`rgb(0,96,223)`.
 When themed,
 the `::-moz-progress-bar` child is
never painted at all;
 `layout/forms/nsProgressFrame.cpp:86-93`:

```cpp
void nsProgressFrame::BuildDisplayList(...) {
  if (IsThemed()) {
    DisplayBorderBackgroundOutline(aBuilder, aLists);  // native widget paint only
  } else {
    BuildDisplayListForInline(aBuilder, aLists);       // CSS boxes incl. bar child
  }
}
```

The devolved colors are hardcoded in the UA sheet,
 and the fill is a
literal color,
 not the `AccentColor` keyword,
 which is why no accent
can influence it;
 `layout/style/res/forms.css:617-646` (lines 598-627
in the 151 tag,
 byte-identical):

```css
progress {
  appearance: auto;
  -moz-default-appearance: progress-bar;
  ...
  /* Default style in case of there is appearance: none; */
  border: 1px solid ThreeDShadow;
  border-right-color: ThreeDHighlight;
  border-bottom-color: ThreeDHighlight;
  /* #e6e6e6 is a light gray. */
  background-color: #e6e6e6;
  ...
}

progress::-moz-progress-bar,
progress::slider-fill {
  ...
  /* Default style in case of there is appearance: none; */
  background-color: #0064b4; /* blue */
}
```

`#0064b4` is exactly the measured devolved fill `rgb(0,100,180)`.
 The
border color:
 `ThreeDShadow` is forced,
 for web content,
 to a
stand-in value;
 `widget/nsXPLookAndFeel.cpp:660-668`:

```cpp
      // deprecated in CSS Color Level 4, same as Buttonborder:
    case ColorID::Threedhighlight:
    case ColorID::Threedlightshadow:
    case ColorID::Threedshadow:
    ...
      return NS_RGB(0x8f, 0x8f, 0x9d);
```

`0x8f8f9d` is exactly the measured border `rgb(143,143,157)`;
 all four
sides look uniform because `ThreeDShadow` and `ThreeDHighlight` map to
the same stand-in.

### Blink

Same devolution trigger;
`third_party/blink/renderer/core/layout/layout_theme.cc:442-449`:

```cpp
bool LayoutTheme::IsControlStyled(AppearanceValue appearance, const ComputedStyleBuilder& builder) const {
  switch (appearance) {
    ...
    case AppearanceValue::kProgressBar:
      return builder.HasAuthorBackground() || builder.HasAuthorBorder();
```

after which `AdjustAppearanceWithAuthorStyle` sets the effective
appearance to `kNone`.
 The `::-webkit-progress-*` pseudos are real
shadow-DOM elements that are `display: none` while native appearance
is in effect;
`third_party/blink/renderer/core/html/shadow/progress_shadow_element.cc:49-58`:

```cpp
void ProgressShadowElement::AdjustStyle(ComputedStyleBuilder& builder) {
  const ComputedStyle* progress_style = ProgressElement()->GetComputedStyle();
  auto appearance = progress_style->EffectiveAppearance();
  if (appearance != AppearanceValue::kNone && ...) {
    builder.SetDisplay(EDisplay::kNone);
  }
}
```

so pseudo rules are inert on a pristine element and spring to life
exactly when author styling (or `appearance: none`) devolves the
widget.
 The devolved colors are UA-stylesheet defaults;
`third_party/blink/renderer/core/html/resources/html.css`:
`progress::-webkit-progress-bar { background-color: gray; … }`
(lines 1444-1448) and
`progress::-webkit-progress-value { background-color: green; … }`
(lines 1466-1472);
 CSS `green`/`gray` are exactly the measured
`rgb(0,128,0)` and `rgb(128,128,128)`.

On the themed path,
 the author accent is used as-is for the fill,
near-white included;
 `ui/native_theme/native_theme_base.cc:843-844`:

```cpp
flags.setColor(GetAccentOrControlColorForState(
    accent_color, kAccentColors, state, dark_mode, contrast, color_provider));
```

(`GetAccentOrControlColorForState`,
 lines 540-554,
 returns
`accent_color.value()` unmodified for the normal state;
 the default
without an accent is `kColorWebNativeControlAccent`,
 measured
`rgb(0,117,255)`).
 There is no near-white rejection anywhere;
 what
exists is a scheme flip for the rest of the control;
`third_party/blink/renderer/core/paint/theme_painter_default.cc:136-151`:

```cpp
// If there is enough contrast between `accent_color` and `color_scheme`, then
// let's keep it the same. Otherwise, flip the `color_scheme` to guarantee
// contrast.
if (color_scheme == mojom::ColorScheme::kDark) {
  if (contrast_with_dark < color_utils::kMinimumVisibleContrastRatio &&
      contrast_with_dark < contrast_with_light) {
    ...
    return mojom::ColorScheme::kLight;
  }
} else { ... }
```

which is why `accent-color: #ffffff` yields a white fill on a
dark-scheme `rgb(59,59,59)` track.

### Spec sanction

[CSS-UI-4][css-ui-4] §7.2.1,
 on devolved widgets:
 "Certain
properties,
 when declared in the Author Origin,
 will disable the
native appearance of certain widgets.
 … then that widget is rendered
as a devolved widget.";
 the property list explicitly includes
`background-color` and the border color/style/width longhands.
 And
§7.1 on `accent-color`:
 "The UA must maintain contrast for legibility
of the control,
 and in order to do so may adjust the luminance or
brightness of the color or make color substitutions in other parts of
the control",
 which blesses Chromium's scheme flip.
 The CSSWG
discussed the contrast mechanism in [csswg-drafts#6159][wg6159] and
resolved (2021-07-14) to keep single-color `accent-color` and leave
the contrast behavior to UAs;
 Chromium's flip landed as
[Gerrit CL 2907972][cl2907972] ("Implement new accent-color contrast
algorithm").

[css-ui-4]: https://drafts.csswg.org/css-ui-4/
[wg6159]: https://github.com/w3c/csswg-drafts/issues/6159
[cl2907972]: https://chromium-review.googlesource.com/c/chromium/src/+/2907972

## Verification

Versions under test:
 Firefox 151.0 and Chromium 149.0.7827.55,
 the
browsers bundled with playwright 1.61.1,
 run inside the
`monochromatic-playwright` podman image.
 The Firefox blue-fill symptom
was also user-observed in desktop Firefox (dark scheme).

### Harness

Save the probe below as
`package/wc/app/src/progress-probe.browser.test.ts`
(the path matters:
 `playwright.browser.config.ts` matches
`wc/app/src/**/*.browser.test.ts`),
 run it with

```bash
mise run test:browser:firefox -- progress-probe --reporter=list
mise run test:browser:chromium -- progress-probe --reporter=list
```

and delete the file afterward.
 It renders one `<progress value="50">`
per CSS variant and reports each element screenshot's dominant
quantized colors;
 the test always passes,
 the console output is the
data.

```ts
// progress-probe.browser.test.ts
import { test } from '@playwright/test';
import sharp from 'sharp';

const VARIANTS: readonly { name: string; css: string; }[] = [
  { name: 'pristine', css: '' },
  { name: 'accent-red', css: '.p{accent-color:#ff0000}' },
  { name: 'accent-white', css: '.p{accent-color:#ffffff}' },
  { name: 'accent-near-white', css: '.p{accent-color:#f2f2f2}' },
  { name: 'accent-red-bg-transparent', css: '.p{accent-color:#ff0000;background-color:transparent}' },
  { name: 'moz-fill-red', css: '.p{background-color:transparent}.p::-moz-progress-bar{background-color:#ff0000}' },
  { name: 'moz-fill-red-borderless', css: '.p{background-color:transparent;border-style:none}.p::-moz-progress-bar{background-color:#ff0000}' },
  { name: 'webkit-pseudos-no-appearance-reset', css: '.p::-webkit-progress-bar{background-color:transparent}.p::-webkit-progress-value{background-color:#ff0000}' },
  { name: 'appearance-none-plus-webkit-pseudos', css: '.p{appearance:none}.p::-webkit-progress-bar{background-color:transparent}.p::-webkit-progress-value{background-color:#ff0000}' },
  { name: 'accent-red-author-border-only', css: '.p{accent-color:#ff0000;border:1px solid #000000}' },
  { name: 'accent-red-author-bg-opaque', css: '.p{accent-color:#ff0000;background-color:#ffff00}' },
  { name: 'transparent-bg-plus-webkit-pseudos', css: '.p{background-color:transparent}.p::-webkit-progress-bar{background-color:transparent}.p::-webkit-progress-value{background-color:#ff0000}' },
];

async function dominantColors(png: Buffer): Promise<string[]> {
  const { data, info } = await sharp(png).raw().toBuffer({ resolveWithObject: true });
  const counts = new Map<string, number>();
  for (let i = 0; i < info.width * info.height; i += 1) {
    const o = i * info.channels;
    // Quantize to 8-step buckets to merge antialiasing.
    const key = [data[o], data[o + 1], data[o + 2]]
      .map((c) => Math.round((c as number) / 8) * 8)
      .join(',');
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const total = info.width * info.height;
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 4)
    .map(([key, n]) => `rgb(${key}) ${Math.round((n / total) * 100)}%`);
}

test('progress variant color probe', async ({ page }, testInfo) => {
  const results: Record<string, string[]> = {};
  for (const variant of VARIANTS) {
    await page.setContent(
      `<style>body{background:#ffffff;margin:16px}${variant.css}</style>`
      + '<progress class="p" value="50" max="100"></progress>',
    );
    const shot = await page.locator('.p').screenshot();
    results[variant.name] = await dominantColors(shot);
  }
  console.log(`PROBE ${testInfo.project.name} ${JSON.stringify(results, null, 1)}`);
});
```

### What works cleanly

Measured 2026-07-02;
 colors are quantized dominant screenshot colors,
fill listed first.

- **Firefox,
   natively themed** (no author background/border):
  - `pristine`:
     blue fill `rgb(0,96,224)` on a white track.
  - `accent-color: #ff0000`:
     red fill.
     Honored.
  - `accent-color: #ffffff` and `#f2f2f2`:
     white/near-white fill.
    Honored as-is;
     Firefox does not contrast-adjust the accent.
- **Chromium,
   natively themed**:
  - `pristine`:
     blue fill `rgb(0,120,255)` on a light track.
  - `accent-color: #ff0000`:
     red fill.
     Honored.
- **Firefox,
   author-styled fallback**:
  - `::-moz-progress-bar { background-color: X }` recolors the fill.
  - adding `border-style: none` removes the `rgb(143,143,157)` border,
    leaving only fill and track colors.
- **Chromium,
   author-styled fallback**:
  - `::-webkit-progress-bar` / `::-webkit-progress-value` style track
    and fill (variant `transparent-bg-plus-webkit-pseudos`:
     red fill,
    page background through the track).
  - `appearance: none` plus the webkit pseudos behaves the same.

### What fails, by variant

- **`accent-color` with any author background or border,
   both
  engines**:
   `accent-red-bg-transparent`,
  `accent-red-author-bg-opaque`,
   and `accent-red-author-border-only`
  all ignore the red accent.
   Firefox falls back to a
  `rgb(0,100,180)` fill (a different blue than its themed
  `rgb(0,96,224)`-ish accent,
   confirming a different code path) plus
  the tinted border;
   Chromium falls back to `rgb(0,128,0)` on
  `rgb(128,128,128)`,
   exactly CSS `green` on `gray`.
   Exact values
  from an unquantized pixel probe of the
  `accent-red-bg-transparent` variant:
   Firefox
  `fill=rgb(0,100,180) track=transparent topEdge=rgb(143,143,157)`,
  Chromium `fill=rgb(0,128,0) track=rgb(128,128,128)`.
- **`accent-color: #ffffff` on a themed Chromium progress**:
   the fill
  IS honored (white),
   but Chromium flips the rest of the control to
  the opposite color scheme for contrast,
   so the track turns dark
  (`rgb(59,59,59)` exact;
   `rgb(56,56,56)` in the 8-step-quantized
  probe).
   A design that needs to control the track color cannot rely
  on the themed path there.
- **Webkit pseudos on a pristine Chromium progress**
  (`webkit-pseudos-no-appearance-reset`):
   no effect;
   the screenshot is
  identical to `pristine`.
   The pseudos only style the fallback boxes,
  so they require the element to be author-styled (or
  `appearance: none`) before they do anything.
- **`::-moz-progress-bar` alone on a pristine Firefox progress**:
   no
  effect either;
   the devolution check reads only the element's own
  style,
   and the themed painter never paints the pseudo child at all
  (`nsProgressFrame::BuildDisplayList` in Root cause).
   Verified
  empirically against the same Firefox 151 binary with a lime fill
  that stayed `#0060df`.
- **`appearance: none` plus webkit pseudos in Firefox**:
   fill stays
  the fallback blue;
   Gecko has no `::-webkit-progress-*`.

### Two earlier wrong readings, disproved

Both shipped as code comments in this repo before the probe existed;
recorded here so they do not get re-derived.

- "Chromium ignores near-white `accent-color` values and falls back to
  the default green."
   Wrong twice:
   a near-white accent on a themed
  progress is honored on the fill,
   with the track flipped to the dark
  scheme for contrast (the dark pixels in the `accent-white` probe
  histogram are the track,
   not the fill);
   and the green fill appears
  only in the author-styled fallback,
   where every accent value is
  ignored,
   red included (probe variants `accent-white` vs
  `accent-red-bg-transparent`).
   An intermediate reading of the
  quantized histogram ("the fill is contrast-adjusted to dark gray")
  was also wrong,
   disproved by the unquantized per-pixel sample
  showing a white fill on a `rgb(59,59,59)` track.
- "The webkit pseudos apply without an `appearance` reset."
   True only
  because the element in question also carried an author background;
  on a pristine element the pseudos do nothing
  (`webkit-pseudos-no-appearance-reset` vs
  `transparent-bg-plus-webkit-pseudos`).

## Verified workarounds

### Style the fallback deliberately (shipped in wc)

`package/wc/app/src/styles-results.ts` accepts the
fallback path (the design needs a transparent track,
 which is author
styling) and pins every box in it:

```css
.freq-bar {
  background-color: transparent; /* clears the track; forces the fallback */
  border-style: none;            /* removes Firefox's rgb(143,143,157) border */
  accent-color: var(--color-fg-strong); /* inert today; kept for engines honoring it here */
}
.freq-bar::-webkit-progress-bar { background-color: transparent; }
.freq-bar::-webkit-progress-value { background-color: var(--color-fg-strong); }
.freq-bar::-moz-progress-bar { background-color: var(--color-fg-strong); }
```

Pixel-verified in both engines by
`package/wc/app/src/page.browser.test.ts`.

Tradeoffs:
 the fill relies on nonstandard vendor pseudo-elements,
 one
rule per engine,
 and other engines (WebKit/Safari was not tested here)
need their own audit;
 the `accent-color` declaration does nothing in
the two tested engines and is carried only as intent documentation and
future-proofing.

### Stay natively themed and use `accent-color`

If the design does not require author background or border on the
element,
 `accent-color` alone recolors the fill in both engines with
no vendor CSS.

Tradeoffs:
 the track and border keep native chrome,
 so a transparent
or palette-controlled track is unreachable;
 and Chromium may flip the
track and border to the opposite color scheme when the accent
contrasts poorly with the current one (a white accent turns the track
`rgb(59,59,59)`),
 so the surrounding chrome is engine-chosen and can
leave the palette even though the fill color itself is honored.

## What does not work

- Combining `accent-color` with a transparent element background and
  expecting the accent to survive:
   the background itself is what
  disables the path that reads the accent.
- `::-webkit-progress-*` rules alone (without author styling or an
  `appearance` reset) in Chromium:
   ignored on the themed widget.
- Only `appearance: none` as the cross-engine reset:
   it forces the
  fallback in both engines,
   but Firefox's fill then still needs
  `::-moz-progress-bar`,
   so it saves nothing over the shipped rules
  and strips the themed rendering even where it was acceptable.

## Upstream filing decision

`.out-of-scope/` was checked (2026-07-02):
 no exemption covers
Firefox/Gecko,
 Chromium/Blink,
 or the CSSWG,
 so the audit applies.

Duplicate search:
 Bugzilla REST (quicksearch plus summary-substring
queries for accent-color with progress,
 "accent-color ignored",
progress border color) surfaced no bug about accent-color dying on an
author-styled progress or about the `#8f8f9d` border;
 the closest hits
are the accent-color implementation bugs 1705605 ("Prototype
accent-color") and 1722031 ("Ship accent-color"),
 both RESOLVED FIXED.
On the Chromium side,
 the in-source TODO references crbug 1216137,
which sits behind a sign-in wall and could not be read;
 the Gerrit CL
2907972 implementing the scheme flip is merged and intentional.
 The
CSSWG already debated the contrast mechanism in
[csswg-drafts#6159][wg6159] and closed it 2021-07-14 with "RESOLVED:
Close issue,
 one color accent-color for now",
 leaving the mechanism to
UAs.

The six constraints,
 walked:

1. **Really upstream's fault?**
    No. Every observed behavior is an
   intentional implementation of spec text:
    devolution on
   author-origin border/background is CSS-UI-4 §7.2.1 verbatim
   (`background-color` and the border longhands are in the listed
   property set),
    and Chromium's scheme flip is the "may … make color
   substitutions in other parts of the control" sentence of §7.1.
   This is a documented-behavior gap on our side,
    not an upstream
   defect.
    The audit fails here;
    the remaining constraints are walked
   for the record.
2. **Can upstream fix it?**
    Technically yes:
    e.g. Gecko's UA sheet
   could paint the devolved fill from the element's accent instead of
   the literal `#0064b4`.
    Nothing architectural forbids it.
3. **Are they supporting this use case?**
    No. `accent-color` is
   spec-scoped to parts "that would have otherwise been styled with an
   accent color",
    which a devolved widget no longer has;
    the supported
   path for author-styled progress is exactly the vendor pseudos this
   doc's workaround uses.
4. **Would the repos welcome a contribution?**
    Both engines accept
   bug reports,
    but this report would relitigate a WG-resolved design
   question with no new information.
5. **Will they likely fix it?**
    Leaning no,
    with direct evidence:
    the
   WG resolution on [csswg-drafts#6159][wg6159],
    the merged and
   deliberate [CL 2907972][cl2907972],
    and spec text sanctioning both
   behaviors.
6. **Prototyped a minimal fix?**
    Not applicable:
    the auto-prototype
   rule fires only when constraints 1 through 5 hold,
    and 1 and 5 do
   not.

Decision:
 file nothing,
 comment nowhere.
 [csswg-drafts#6159][wg6159]
is the only live-adjacent thread,
 it is closed by WG resolution,
 and
everything measured here (accent honored while themed,
 scheme flip on
low contrast,
 devolution on author styling) confirms the resolved
design rather than adding to it;
 there is nothing an additive comment
could carry.
 A conceivable enhancement request ("devolved progress
fill should follow `accent-color`") would be a CSSWG spec discussion,
not an engine bug,
 and is not worth opening for a problem the vendor
pseudos already solve.

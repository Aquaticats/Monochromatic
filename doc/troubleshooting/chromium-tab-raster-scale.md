# Chromium at 77586a34: copying screenshot pixels as logical dimensions produces oversized imitation tabs

## Symptom

A screenshot-driven Chromium-like page selector looked recognizably tab-shaped,
but its controls were too tall and too wide on desktop and Android.

The failing implementation used a `58px` or `58dp` control,
a `50px` or `50dp` visible shape,
`8px` or `8dp` top space,
and `24px` or `24dp` horizontal label insets.
Those values came from reading raster pixels as logical UI dimensions.

The desktop implementation uses Chromium's source dimensions as logical pixels.
Android preserves Chromium's contour ratios but scales the visible tab face
to the platform's `48dp` custom-control minimum.
Both preserve this music player's content-width and whole-control wrapping requirements.

## Root cause

The raster reference was captured at a display scale that was not established.
Measuring its device pixels described the screenshot,
not Chromium's density-independent layout.

Chromium commit `77586a34f65c0d6393e8034e28c75477da86beae` defines tab height in
`chrome/browser/ui/layout_constants.cc:102-117`:

```cpp
case LayoutConstant::kTabHeight:
  return 34 + GetLayoutConstant(LayoutConstant::kTabstripToolbarOverlap);
case LayoutConstant::kTabStripHeight:
  return GetLayoutConstant(LayoutConstant::kTabHeight) +
         GetLayoutConstant(LayoutConstant::kTabStripPadding);
case LayoutConstant::kTabStripPadding:
  return 6;
case LayoutConstant::kTabHorizontalPadding:
  return 8;
case LayoutConstant::kTabstripToolbarOverlap:
  return 1;
```

The visible tab is `35` DIPs and its strip adds `6` DIPs,
so one Chromium tab row is `41` DIPs.
The earlier `58`-unit row was not a Chromium metric.

Chromium defines fixed geometry in `chrome/browser/ui/tabs/tab_style.cc:22-29,101-123`:

```cpp
constexpr int kSeparatorThickness = 2;
constexpr int kSeparatorHeight = 16;
constexpr int kTabWidth = 232;

int TabStyle::GetTopCornerRadius() const {
  return 10;
}

int TabStyle::GetBottomCornerRadius() const {
  return 12;
}
```

The project deliberately does not adopt Chromium's fixed standard width.
Its accepted product requirement is content-width labels that wrap as whole tabs.
Chromium's internal horizontal inset remains source evidence,
but the product now deliberately halves it.
`chrome/browser/ui/tabs/tab_style.cc:314-324` composes the `12`-DIP shoulder with `8` DIPs of padding:

```cpp
return gfx::Insets::TLBR(
    GetLayoutConstant(LayoutConstant::kTabVerticalPadding) +
        GetLayoutConstant(LayoutConstant::kTabStripPadding),
    GetBottomCornerRadius() +
        GetLayoutConstant(LayoutConstant::kTabHorizontalPadding),
    GetLayoutConstant(LayoutConstant::kTabVerticalPadding) +
        GetLayoutConstant(LayoutConstant::kTabStripPadding),
    GetBottomCornerRadius() +
        GetLayoutConstant(LayoutConstant::kTabHorizontalPadding));
```

That gives `20` DIPs on each side of text when the favicon and close control are omitted.
The application now uses `10` logical units on each side,
exactly half the source-derived inset,
per requester direction.
The earlier `24`-unit inset made every content-width tab `8` units wider than the former `20`-unit version.

The file-folder silhouette is source behavior,
not a rounded rectangle approximation.
`chrome/browser/ui/views/tabs/tab_style_views.cc:369-408` starts at the lower edge,
draws the bottom shoulder,
then draws the ascender and top corner:

```cpp
path.moveTo(left, extended_bottom);
path.lineTo(tab_left - left_extension_corner_radius, tab_bottom);
path.arcTo(
    SkVector(left_extension_corner_radius, left_extension_corner_radius), 0,
    SkPathBuilder::kSmall_ArcSize, SkPathDirection::kCCW,
    SkPoint(tab_left, tab_bottom - left_extension_corner_radius));
path.lineTo(tab_left, tab_top + top_left_corner_radius);
path.arcTo(SkVector(top_left_corner_radius, top_left_corner_radius), 0,
           SkPathBuilder::kSmall_ArcSize, SkPathDirection::kCW,
           SkPoint(tab_left + top_left_corner_radius, tab_top));
```

## Verification

The source trace used the official Chromium GitHub mirror at
`77586a34f65c0d6393e8034e28c75477da86beae`.
Its verified origin was `https://github.com/chromium/chromium.git`.

A minimal source checkout is:

```sh
gh repo clone chromium/chromium /tmp/chromium-tabs -- --depth 1 --filter=blob:none --sparse
git -C /tmp/chromium-tabs sparse-checkout set --no-cone \
  /chrome/browser/ui/layout_constants.cc \
  /chrome/browser/ui/tabs/tab_style.cc \
  /chrome/browser/ui/views/tabs/tab_style_views.cc
rg --line-number 'kTabHeight:|kTabStripPadding:|kTabHorizontalPadding:|GetTopCornerRadius|GetBottomCornerRadius' \
  /tmp/chromium-tabs/chrome/browser/ui/layout_constants.cc \
  /tmp/chromium-tabs/chrome/browser/ui/tabs/tab_style.cc
```

The consumer verification commands are:

```sh
mise run //package/music-player/desktop-app:lint:slint
mise run //package/music-player/desktop-app:test
mise run //package/music-player/android-app:lint
mise run //package/music-player/android-app:lint:detekt
mise run //package/music-player/android-app:test:unit
```

### Values that reproduce Chromium's logical geometry

- `41` source row height:
   `35` tab height plus `6` strip padding.
- `10:35` top-corner ratio and `12:35` bottom-shoulder ratio.
- `20` Chromium-source horizontal text inset when icon and close controls are omitted.
- `10` application inline text padding on each side after the requester-directed halving.
- `2` by `16` inactive separators.
- `1`-unit active contour.

Desktop uses the source geometry except for the deliberate `10px` inline text padding.
Android uses the same `10dp` inline text padding plus a visible `48dp` face and the `6dp` strip inset,
then scales the corner and shoulder ratios to that face.

### Values that reproduce the oversized imitation

- `58` total row height with a `50`-unit visible shape.
- `8` units of strip headroom.
- `24` units of horizontal text inset.
- `24`-unit separators.
- `1.5`-unit active contour.

## Verified workarounds

Use Chromium's logical geometry as source geometry in both renderers:

- `package/music-player/desktop-app/ui/app.slint` expresses the dimensions directly in Slint logical pixels.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt`
  scales the `10:35` corner and `12:35` shoulder ratios onto a visibly `48dp` face.
  Its total row is `54dp` after Chromium's `6dp` strip inset.

Keep project-specific behavior explicit:

- labels remain content-width instead of Chromium's standard fixed width;
- whole tabs wrap across rows;
- inactive tabs and the backplate inherit the greater page background;
- the active fill is accent-tinted;
- favicon and close affordances are omitted.

The tradeoff is deliberate visual adaptation rather than a byte-for-byte Chromium tab strip.
Chromium itself does not wrap browser tabs into multiple rows,
and its standard tab includes favicon and close-control allocation.

## What does not work

- **Treating raster pixels as DIPs:**
   screenshot density makes geometry scale-dependent.
- **Scaling every measured screenshot dimension by one guessed ratio:**
   text rendering and capture scaling can differ.
- **Copying Chromium's `232`-DIP standard content width:**
   violates the accepted content-width requirement.
- **Using only a rounded rectangle:**
   omits the outward lower shoulders that identify the supplied silhouette.
- **Adding favicon or close glyphs for resemblance:**
   creates affordances the page selector does not support.

## Upstream filing decision

No `.out-of-scope/` entry matches Chromium tab dimensions.
GitHub searches across open and closed `chromium/chromium` issues for
`tab height width logical DIP` found no matching report.

1. **Is it really upstream's fault?**
    No.
   Chromium's source is internally consistent;
   the defect was this project's raster-to-logical conversion.
2. **Can upstream fix it?**
    No.
   Chromium cannot correct dimensions in this repository's Slint and Compose components.
3. **Are they supporting this use case?**
    No.
   Chromium supports its own browser strip,
   not a wrapping third-party music-page selector.
4. **Would the repo welcome our contribution?**
    Not evaluated beyond the public source and tracker search,
   because there is no upstream defect or relevant patch.
5. **Will they likely fix it?**
    Not applicable;
   there is nothing upstream to fix.
6. **Have we prototyped a minimal upstream fix?**
    No upstream patch exists.
   The minimal consumer fix was implemented and verified in this repository.

There is no issue or additive comment to file upstream.

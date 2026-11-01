# Slint 1.17.0 and Compose Foundation 1.11.2 segmented controls fill available rows

## Symptom

A page label such as `Music` or `Text` should produce a segment sized to its label plus padding.
Instead,
 each Android segment fills the phone width.
The desktop sections retain content widths,
but a rounded frame extends through the unused row width and makes that space look like another segment.

The trigger has two forms:

- Compose:
   a decorative child inside a content-sized `Box` uses `Modifier.fillMaxWidth()`.
- Slint:
   a visible frame uses `parent.width` around a wrapping `FlexboxLayout`.

## Root cause

### Compose

The faulty version at commit `d228ebc79` put a bottom divider inside every segment:
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt:2620-2625`.

```kotlin
Box(
    modifier = Modifier
        .align(Alignment.BottomCenter)
        .fillMaxWidth()
        .height(1.dp)
        .background(MaterialTheme.colorScheme.outline),
)
```

AndroidX Foundation commit `490d0e438d2e9129de679fd720be1186bc69ffe5`
measures an unweighted `FlowRow` child with the available constraints.
`compose/foundation/foundation-layout/src/commonMain/kotlin/androidx/compose/foundation/layout/FlowLayout.kt:1525-1540`:

```kotlin
val placeable = measure(constraints).also(storePlaceable)
with(measurePolicy) {
    val mainAxis = placeable.mainAxisSize()
```

`fillMaxWidth()` then turns the available maximum into both the child's minimum and maximum.
`compose/foundation/foundation-layout/src/commonMain/kotlin/androidx/compose/foundation/layout/Size.kt:697-725`:

```kotlin
val width =
    (constraints.maxWidth * fraction)
        .fastRoundToInt()
        .fastCoerceIn(constraints.minWidth, constraints.maxWidth)
minWidth = width
maxWidth = width
```

The divider therefore determines the containing segment's width.
The text's natural width never gets to determine the `Box` width.

The same mechanism affected the LED renderer introduced in commit `5b54692cc`.
Its `SubcomposeLayout` tried to discover each target width by measuring the complete `ledCapTarget`.
A visual cap descendant used `Modifier.fillMaxSize()`.
The probe supplied a bounded row maximum,
 so that visual descendant consumed the maximum and made every cap its own full-width row.

The corrected probe measures only the one-line `Text` legend.
It derives target width as measured legend plus both 24-unit insets,
clamped between the 48-unit owned target minimum and available row width.
The renderer then composes the visual cap under fixed constraints at that derived width.
This keeps measurement tied
to actual Compose text shaping without letting greedy decoration participate in intrinsic width.

### Slint

The faulty version at commit `d228ebc79` drew an overlay using the wrapping container's full width.
`package/music-player/desktop-app/ui/app.slint:230-236` at that commit:

```slint
Rectangle {
    width: parent.width;
    height: parent.height;
    border-width: 2px;
    border-radius: 8px;
    background: transparent;
}
```

Binding that wrapper to the inner `FlexboxLayout` preferred width did not help under Slint 1.17.0.
The installed `i-slint-compiler` version and crates.io checksum are recorded at
`package/music-player/desktop-app/Cargo.lock:2562-2565`.

Slint issue [#11936][] identifies the relevant 1.17 layout-info defect.
The merged fix is visible in Slint commit
`c181dc45135c91e77a2694aed20c29e1aa9c62f7`.
`internal/compiler/llr/lower_layout_expression.rs:1300-1333` now detects a nested wrapping flex layout and clamps its
reported preferred size:

```rust
let Some(flex) = crate::layout::FlexboxLayout::from_element(elem) else {
    return layout_info;
};
let unwrapped = flexbox_unwrapped_main_expr(&flex, orientation, ctx);
let clamped = llr_Expression::MinMax {
    op: MinMaxOp::Min,
```

That fix is newer than the installed 1.17.0 crate.
The consumer-side workaround cannot rely on a wrapping flex layout reporting a content width to its parent.

## Verification

Versions under test:

- Slint `1.17.0`,
   with `i-slint-compiler` checksum
  `290c2247e87d3653b9b7c3bc1cbe19647b2f93a4b99a40aae6a7140717700d37`.
- Compose BOM `2026.05.01`,
   resolving Foundation Layout Android `1.11.2`.
- Pixel 6 device `1C171FDF600KWW`.

The desktop harness copies `ui/app.slint` to `target/segmented-repro.slint`,
changes the default labels to `A`,
 `Abcdef`,
 and `Abcdefghijklmnopq`,
selects style `3`,
 and renders it:

```sh
cd package/music-player/desktop-app
node --input-type=module <<'EOF'
import {
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';

mkdirSync('target', { recursive: true });
const source = readFileSync('ui/app.slint', 'utf8');
const repro = source
  .replace(
    'in property <[string]> page-labels: [];',
    'in property <[string]> page-labels: ["A", "Abcdef", "Abcdefghijklmnopq"];',
  )
  .replace(
    'in-out property <int> page-control-style: 0;',
    'in-out property <int> page-control-style: 3;',
  );
writeFileSync('target/segmented-repro.slint', repro);
EOF
mise run lint:slint
SLINT_BACKEND=software slint-viewer \
  --screenshot target/segmented-after.png \
  target/segmented-repro.slint
```

The pre-fix image framed the unused width through the right edge of the 480 px render.
The fixed image ends the visible controls immediately after `Abcdefghijklmnopq`.
A second render with enough labels to wrap confirms the final row ends immediately after `Final`.

The Android boundary was rebuilt,
 installed,
 and launched with:

```sh
mise run //package/music-player/android-app:run:release
```

A device screenshot before commit `47ba628d7` showed `Music` and `Text` as separate full-width rows.
The same screenshot probe after that commit shows both labels joined in one content-width group.

Patterns that work cleanly:

- A Compose segment draws its outline with `Modifier.border(...)` on the segment itself.
- `Modifier.wrapContentWidth(Alignment.Start)` removes an inherited minimum while retaining the screen maximum.
- A Slint wrapping container leaves unused width transparent and each segment draws its own content-width outline.

Patterns that fail:

- A Compose decoration uses `fillMaxWidth()` inside the content-sized segment.
- A `SubcomposeLayout` width probe measures a complete target containing `fillMaxSize()` decoration.
- A Slint overlay binds its visible border to `parent.width` around `FlexboxLayout`.
- Slint 1.17.0 tries to derive a parent wrapper width from the wrapping flex layout's preferred width.

## Verified workarounds

### Draw decoration without fill modifiers in Compose

Commit `47ba628d7` replaces the two fill-sized divider children with a `0.5.dp` border modifier on each segment.
The outer group also uses `wrapContentWidth(Alignment.Start)`.

Tradeoff:
 adjacent half-pixel outlines compose into the visible internal divider,
so custom asymmetric divider colors would require a dedicated layout or drawing modifier.

### Keep Slint's unused flex width invisible

Commit `47ba628d7` removes the full-width outer frame and gives each naturally sized segment its own outline.
The flex container can still use all available width for wrapping,
but no paint makes unused width resemble a control.

Tradeoff:
 Slint 1.17.0 cannot show one rounded frame around the exact used width while retaining dynamic wrapping.
The workaround uses square section outlines until the package adopts a Slint release containing [PR #12129][].

## What does not work

- `horizontal-stretch: 0` on the Slint wrapper produced a byte-identical screenshot.
  When all layout children have zero stretch,
   Slint may still distribute available space among them.
- `preferred-width: segment-layout.preferred-width` plus the same `max-width` also produced a byte-identical screenshot.
  The installed flex layout did not provide the parent with the required content width.
- Keeping the outer frame while only making labels content-width leaves a blank framed area.
  The labels are correct,
   but the visible control still appears full-width.
- Adding `wrapContentWidth` alone on Android does not neutralize a descendant `fillMaxWidth()`.
  The descendant still receives and consumes the relaxed maximum width.
- Measuring the full LED target before row packing still gives `fillMaxSize()` a bounded maximum.
  Subcomposition changes when measurement happens,
   not how fill modifiers consume constraints.

## Upstream filing decision

No `.out-of-scope/` entry covers Slint,
 Compose,
 or layout behavior.
Tracker searches found Slint issue [#11936][] and merged [PR #12129][].
The issue comments already contain the corrected root cause,
 before and after evidence,
 and the merged fix.
There is nothing additive to post.

The filing constraints resolve as follows:

1. **Upstream fault:**
    Compose is not at fault because the consumer requested fill width.
   Slint's preferred-size limitation is upstream and is already tracked.
2. **Upstream can fix it:**
    yes for Slint,
    demonstrated by merged [PR #12129][].
3. **Supported use case:**
    yes,
    [#11936][] uses a nested wrapping `FlexboxLayout` reproducer.
4. **Contribution policy:**
    Slint's `CONTRIBUTING.md` welcomes issues and pull requests.
   Its `AGENTS.md` explicitly provides guidance for AI coding assistants and contains no filing ban.
5. **Likely fix:**
    complete,
    because [PR #12129][] merged on June 17, 2026.
6. **Compatible prototype:**
    complete upstream.
   The merged change includes compiler and core fixes plus layout regression cases.

Because the only upstream defect is already fixed and the Compose defect was consumer code,
no new issue or comment draft is warranted.

[#11936]: https://github.com/slint-ui/slint/issues/11936
[PR #12129]: https://github.com/slint-ui/slint/pull/12129

# Compose Foundation 1.11.2 `IntrinsicSize.Min` wraps content-sized tab labels

## Symptom

A multi-row tab selector sizes some single-word labels correctly,
but breaks multi-word labels across lines inside one tab.
Examples observed on the Pixel 6 include:

- `Alan Walker`
- `Alessia Cara`
- `Armin van Buuren`
- `Avril Lavigne`

The intended layout keeps each label on one line and wraps the whole tab to another `FlowRow` line.

The triggering code is:

```kotlin
Column(modifier = Modifier.width(IntrinsicSize.Min)) {
    Text(text = label)
}
```

## Root cause

The app asked Compose for minimum intrinsic width even though the design needs the unwrapped content width.
Android's intrinsic-measurement documentation distinguishes the questions explicitly:

- `IntrinsicSize.Min`:
   minimum width needed to display content.
- `IntrinsicSize.Max`:
   maximum width needed to display content.

The Foundation Layout 1.11.2 source jar has SHA-256
`12a468b3f9be15ec7941ef63cdade939aa0e84d2e766e948a2f61c1f2dc56135`.
Its
`commonMain/androidx/compose/foundation/layout/Intrinsic.kt:174-185`
routes those variants to different text queries:

```kotlin
private class IntrinsicWidthNode(var width: IntrinsicSize, override var enforceIncoming: Boolean) :
    IntrinsicSizeModifier() {
    override fun MeasureScope.calculateContentConstraints(
        measurable: Measurable,
        constraints: Constraints,
    ): Constraints {
        var measuredWidth =
            if (width == IntrinsicSize.Min) {
                measurable.minIntrinsicWidth(constraints.maxHeight)
            } else {
                measurable.maxIntrinsicWidth(constraints.maxHeight)
            }
```

`IntrinsicSize.Min` therefore asks the `Text` subtree for `minIntrinsicWidth`.
For a multi-word label,
that width permits line breaks between words.
The containing tab becomes narrow and the label wraps internally.

`IntrinsicSize.Max` instead asks for `maxIntrinsicWidth`,
which gives the one-line content width needed by this tab design.

## Verification

Versions and target:

- Compose BOM `2026.05.01`.
- Foundation Layout Android `1.11.2`.
- Pixel 6 device `1C171FDF600KWW`.

The user screenshot
`/var/home/user/Downloads/Screenshot_20260813-164633.png`
reproduces the failure with multiple wrapped artist names.

The implementation changed the tab container to:

```kotlin
modifier = Modifier
    .width(IntrinsicSize.Max)
    .defaultMinSize(minHeight = 48.dp)
```

It also pins label behavior at the rendering boundary:

```kotlin
Text(
    text = label,
    maxLines = 1,
    overflow = TextOverflow.Ellipsis,
)
```

Verification commands:

```sh
mise run //package/music-player/android-app:lint
mise run //package/music-player/android-app:lint:detekt
mise run //package/music-player/android-app:test:unit
mise run //package/music-player/android-app:run:release
```

All tasks passed.
A post-install device screenshot showed multi-word labels including `Aitsuki Nakuru`,
`Imagine Dragons`,
`Ivan Torrent`,
`Jay Hardway`,
`Joe Hisaishi`,
`John Wick OST`,
and `Justin Bieber` on one line each.
Whole tabs continue wrapping between `FlowRow` lines.

Patterns that work:

- Single-word labels with `IntrinsicSize.Max`.
- Multi-word labels with `IntrinsicSize.Max`.
- Explicit `maxLines = 1` with ellipsis when one label exceeds the available display width.

Patterns that fail:

- Multi-word labels inside a container using `IntrinsicSize.Min`.

No automated screenshot seam currently renders this Compose screen with representative page labels.
The regression is verified at the user boundary on the connected device.

## Verified workaround

Use `IntrinsicSize.Max` for a control whose width should equal the unwrapped label width.
Also set `maxLines = 1` and an overflow policy on the `Text` itself.

Tradeoff:
 a single label wider than the screen is ellipsized rather than wrapped.
That preserves one-tab-per-label geometry and prevents horizontal overflow.

## What does not work

- Increasing horizontal padding does not change the intrinsic query;
  multi-word labels still wrap inside a wider but undersized tab.
- Keeping `IntrinsicSize.Min` and adding `maxLines = 1` prevents wrapping,
  but the narrow constraint can clip or ellipsize labels that would fit at their natural width.
- Changing only `FlowRow` spacing does not affect child intrinsic width.
- Treating the screenshot as a segmented-button defect targets the wrong component;
  the active style is the MD1 tab variant.

## Upstream filing decision

No `.out-of-scope/` entry covers AndroidX Compose or intrinsic sizing.
A GitHub search of open and closed AndroidX issues for
`IntrinsicSize Min Text wrap`
found no matching report.

The filing constraints resolve as follows:

1.  **Upstream fault:**
     no.
    Compose implements the documented distinction between minimum and maximum intrinsic size.
2.  **Upstream can fix it:**
     not applicable because changing `Min` to mean maximum one-line width
    would break its contract.
3.  **Supported use case:**
     yes.
    The official intrinsic-measurement guide documents both variants.
4.  **Contribution policy:**
     AndroidX source is public,
    but no contribution is appropriate for correct behavior.
5.  **Likely fix:**
     not applicable upstream.
6.  **Compatible prototype:**
     the consumer-side change to `IntrinsicSize.Max` is implemented and device-verified.

No upstream issue or comment draft is retained because this was incorrect consumer API selection,
not an AndroidX defect.

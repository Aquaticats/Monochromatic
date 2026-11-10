# Compose Material 3 1.5.0-alpha27 forces equal-width row segments and has no vertical wrapper

## Symptom

The music-player mode selector must remain a segmented button at 200% Android text
scale.
 Four full labels no longer fit in the 414dp pane's horizontal segmented row.
The installed Compose Material 3 API exposes `SingleChoiceSegmentedButtonRow`,
 but no
vertical segmented wrapper or content-sized segment option.

There is no compiler diagnostic for the missing orientation.
 The failed designs are
visible instead:

- Keeping the row at 200% clips or crowds labels.
- Replacing it with radio rows loses the settled segmented-button component.
- Making the row horizontally scrollable contradicts the required purely vertical
  presentation.

## Root cause

The project pins `androidx.compose.material3:material3:1.5.0-alpha27` in
`package/music-player/android-app/app/build.gradle.kts:82`.

AndroidX source commit `4e931c638cc2fca0f2326b9806a09e9f9d511175` defines the
single-choice segmented element only as an extension on a row scope.
 In
`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SegmentedButton.kt:210-219`:

```kotlin
public fun SingleChoiceSegmentedButtonRowScope.SegmentedButton(
    selected: Boolean,
    onClick: () -> Unit,
    shape: Shape,
    modifier: Modifier = Modifier,
```

The supplied group is explicitly a `Row`.
 The same source at lines 328 to 340 says:

```kotlin
public fun SingleChoiceSegmentedButtonRow(
    modifier: Modifier = Modifier,
    space: Dp = SegmentedButtonDefaults.BorderWidth,
    content: @Composable SingleChoiceSegmentedButtonRowScope.() -> Unit,
) {
    Row(
        modifier =
            modifier
                .selectableGroup()
                .defaultMinSize(minHeight = OutlinedSegmentedButtonTokens.ContainerHeight)
                .width(IntrinsicSize.Min),
        horizontalArrangement = Arrangement.spacedBy(-space),
```

The public scope itself extends `RowScope` at line 482:

```kotlin
public interface SingleChoiceSegmentedButtonRowScope : RowScope
```

The element also appends equal weight internally at lines 231 to 234,
 after the caller's
modifier:

```kotlin
modifier =
    modifier
        .weight(1f)
```

The library therefore supplies horizontal equal-width sizing,
 overlap,
 and group
semantics,
 but no intrinsic-width option or parallel column layout.
 This is an API-shape
limitation,
 not evidence that content-sized or vertical segments are invalid as a
product design.

## Verification

The AndroidX source was read from the immutable Gitiles commit named in the root-cause
section.
 The installed binary API was also inspected directly:

```console
jar tf ~/.gradle/caches/9.5.1/transforms/5482783770c2ea5cd3ac7fbc2b7c0ebd/transformed/material3-api.jar \
  | grep -i SegmentedButton
javap -classpath ~/.gradle/caches/9.5.1/transforms/5482783770c2ea5cd3ac7fbc2b7c0ebd/transformed/material3-api.jar \
  -public androidx.compose.material3.SegmentedButtonKt
```

### Cleanly supported cases

- One horizontal equal-width `SingleChoiceSegmentedButtonRow` at a fitting text scale.
- Two to five real equal-width `SegmentedButton` elements inside that row.
- Selected fill,
   checkmark,
   outline,
   and radio role from the Material component.

### Unsupported layout cases

- No `SingleChoiceSegmentedButtonColumn`,
   vertical orientation parameter,
   or other
  public vertical group appears in the installed jar or current AndroidX source.
- No public parameter disables the element's unconditional `.weight(1f)` to produce
  content-sized cells.

### Consumer-side prototype

Prototype commit `13913118a` builds and Android lint passes.
 Native captures use one
connected content-sized row at 85%,
 connected 2×2 from 100% through 130%,
 and four
rows from 150% through 200%.
 Every segment retains at least 12dp horizontal content
padding;
 overflow adds rows rather than reducing that floor.
 At 200%,
 the complete group clears the navigation
inset after vertical deck scrolling.
 UI Automator exposes all labels and a checked
mode.

A throwaway long-name build renders `Shuffle Extr…ory` for
`ExtraordinarilyLongDirectory` inside the same maximum content width as `Shuffle
Camellia`.
 UI Automator retains the full name in `contentDescription`.

## Verified workarounds

### Multi-row connected groups

The debug-only visual prototype stacks real `SingleChoiceSegmentedButtonRow` instances
inside one outer selectable column.
 It uses custom outside shapes,
 rectangular internal
corners,
 and a negative 1dp gap to overlap shared outlines.
 The implementation is in
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/DesignCandidateActivity.kt:1115-1227`
on the prototype branch.

`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/DesignCandidateActivity.kt`
exists only on an unmerged branch (newest commit `a61a4dd476` (fix(music-player-android-app):
cover
picker P5 container no longer moves the title when it opens) on 2026-10-10) and is not in the
working tree.

```kotlin
Column(
    modifier = Modifier.fillMaxWidth().selectableGroup(),
    verticalArrangement = Arrangement.spacedBy((-1).dp),
) {
    // Each row contains one real SegmentedButton with a position-specific shape.
}
```

This preserves the required visual component,
 full labels,
 selected fill,
 checkmark,
and per-segment radio role without horizontal scrolling.
 Tradeoff:
 each segment is
hosted by a one-item row because the library's segmented element is row-scoped.
 The
production implementation must verify that assistive technology announces the outer
mutually exclusive group rather than separate row groups.

### Content-sized one-row group

Because the library element forces equal weight,
 the one-row branch uses connected
`OutlinedButton` elements at intrinsic content width.
 It explicitly retains the
segmented outline,
 outside shapes,
 selected secondary container,
 checkmark,
 48dp
minimum target,
 radio role,
 and full accessibility description.
 The implementation is in
`DesignCandidateActivity.kt:925-1110` on the prototype branch.

The `Shuffle` prefix and subdirectory are separate text nodes.
 The name node is capped at
the rendered width of `Camellia` and uses `TextOverflow.MiddleEllipsis`.
 Tradeoff:
the one-row branch reproduces the Material segmented treatment rather than using
`SegmentedButton` itself;
 production assistive technology must verify its group and radio
announcements.

## What does not work

- Plain radio rows fit the labels but violate settled component identity.
- A horizontally scrolling segmented row preserves component identity but violates the
  required purely vertical presentation.
- A 2 by 2 arrangement cannot preserve one-row deck height and does not fit labels from 150% through 200%.
- One constrained horizontal row cannot preserve all four full labels at 200% text.
- Adding `wrapContentWidth()` at the group boundary does not remove the element's
  internal equal weight.

## Upstream filing decision

No `.out-of-scope/` entry covers AndroidX segmented buttons.
 Web searches of AndroidX
GitHub results and Google's issue tracker found no trustworthy matching vertical-group
request.
 No issue should be filed.

1.  **Upstream fault:**
     No. AndroidX names and documents a row component;
     it does not
    promise a vertical wrapper.
2.  **Upstream can change it:**
     Yes,
     by adding a column scope and vertical overlap
    layout.
3.  **Supported use case:**
     No evidence says the Compose component supports vertical
    orientation.
4.  **Contribution policy:**
     Not evaluated further because the supported-use-case gate
    fails.
5.  **Likely upstream action:**
     Unknown;
     no matching maintained issue was verified.
6.  **Compatible minimal fix:**
     Not prototyped upstream because constraints one and
    three fail.
     The consumer-side visual workaround is sufficient for this design
    prototype but is not an upstream API proposal.

There is no fileable upstream issue or additive comment.

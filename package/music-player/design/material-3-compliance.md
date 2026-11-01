# Material 3 compliance for the theme questionnaire

## Source and scope

The source is the user-supplied SingleFile archive at
`/var/home/user/Downloads/m3.material.io/`.
Its `manifest.json` identifies `https://m3.material.io/` as the original URL and
records an archive time of 2026-09-03T12:22:11.605Z.
The archive is a primary Google source.

This audit covers every guideline applicable to the UI visible in the six theme
questionnaire captures:
 adaptive layout,
 panes,
 app bars,
 lists,
 buttons,
 icon buttons,
segmented buttons,
 sliders,
 icons,
 color,
 type,
 spacing,
 bidirectionality,
 system safety
regions,
 touch targets,
 focus,
 labels,
 text resizing,
 and truncation.
Guidance for components absent from the captures does not apply to this round.

The project stays on baseline Material 3 under decision A2.
The archive explicitly says baseline lists,
 app bars,
 icon buttons,
 and segmented
buttons remain available even where Material 3 Expressive recommends newer variants.
The questionnaire must therefore satisfy baseline component guidance without silently
switching the product to Material 3 Expressive.

**Geometry supersession.**
 Several historical sections of this audit prescribe
a 24dp blank pane spacer for the unfolded player.
 Those describe an older
prototype,
 not the current E2 boundary:
 the user specified about **7.5mm
of physical visible dent**,
 and `max(min_padding, crease_width)` separates
**informational material** in physical coordinates.
 Pane backgrounds,
borders,
 padding and hit regions may cross;
 a fixed 24dp empty visual gap is
not required.
 Do not let baseline M3 pane spacing override this explicit
device-specific correction.

## Violations in prototype commit `6e8f248c5`

### Adaptive layout and scaffold

- `FullUnfoldedStudy` uses a 16dp pane spacer.
   The archived expanded-breakpoint page
  specifies 24dp leading and trailing margins and a 24dp spacer between panes at
  widths from 840dp to 1199dp:
  `foundations/layout/breakpoints/expanded/index.html`.
- The screen width is 852dp,
   so it is expanded and should use a two-pane layout.
  The two-pane choice is correct,
   but its geometry is not:
  `foundations/layout/canonical-examples/list-detail/index.html` and
  `foundations/layout/scaffold/panes/index.html`.
- The pane backgrounds may remain full bleed.
   Actionable content must align to a
  margin or safety ruler,
   and system UI must not cover it:
  `foundations/layout/grids-spacing/grids/index.html` and
  `foundations/layout/scaffold/bars/index.html`.

Correction:
 use a centered 24dp split-pane spacer,
 keep section backgrounds full bleed,
and keep actionable content after the measured system-gesture and system-bar insets.
Use 24dp screen-edge content alignment wherever the larger measured safety inset does
not already exceed it.

### App bars and page identity

- Neither pane has a real app bar.
   Both use hand-built 56dp rows.
- The right row presents `Camellia` as a button rather than as the page title.
- The three-dot glyph is text in a passive `Box`,
   not an icon button.
- The archive requires app bars at the top of a page,
   a contextual title,
   straight
  corners,
   the default height,
   full pane or window width,
   and no more than one action
  unless a second is necessary:
  `components/app-bars/guidelines/index.html` and
  `foundations/layout/scaffold/bars/index.html`.

Correction:
 use Compose `TopAppBar` in both panes.
 Label the list pane `Folders` and
the detail pane `Camellia`.
 Keep one contextual action in each bar.
 Use the real app
bar height,
 color roles,
 typography,
 insets,
 and icon-button behavior supplied by the
Material component.

### Buttons and folder selection

- The prototype uses `Button` with manually supplied tonal colors for both `Open` and
  the current folder.
- `Open` contains a Unicode block instead of an icon.
- A single persistent current-folder control must not be styled as a chip.
   The archive
  says chips appear as a set of contextual options and explicitly says not to display
  a single chip by itself:
  `components/chips/guidelines/index.html`.
- Button labels must remain visible on one line,
   use sentence case,
   and describe the
  action.
   Leading icons must communicate the action:
  `components/buttons/guidelines/index.html`.

Correction:
 remove the redundant current-folder button.
 Use the right-pane app-bar
title and selected plain-text target for current-folder identity.
 Place a real Material
`TextButton` with the official folder-open icon on the left `Folders` app-bar line.
It remains visible with a 48dp target and consumes no separate source-action row.

### Iconography and icon buttons

- `▰`,
   `⋮`,
   `◀`,
   `Ⅱ`,
   `▶`,
   and the selected-row triangle are font glyphs,
   not
  Material icons.
- The transport faces are passive `Surface` nodes,
   not icon buttons.
   They therefore do
  not inherit Material target,
   state,
   ripple,
   focus,
   role,
   or label behavior.
- Google specifies 24dp as the standard icon size,
   a 48dp default target around it,
  consistent icon weight,
   and action-based accessibility labels:
  `styles/icons/designing-icons/index.html`,
  `styles/icons/applying-icons/index.html`,
   and
  `components/icon-buttons/accessibility/index.html`.

Correction:
 use official Material icons in `Icon`,
 `IconButton`,
`FilledIconButton`,
 and `FilledTonalIconButton`.
 Supply action labels such as
`Previous track`,
 `Pause`,
 `Next track`,
 and `Settings`.
 Keep media controls and their
progress direction left-to-right even in an RTL locale,
 as required by
`foundations/layout/bidirectionality-rtl/index.html`.

### Segmented playback mode

- The four choices are drawn as bordered boxes rather than the Compose Material
  component.
- The control wraps into two rows.
   The archive explicitly says segments must not wrap,
  should remain between two and five choices,
   use a 40dp visual container inside a
  48dp target,
   and indicate selection with both color and a checkmark:
  `components/segmented-buttons/guidelines/index.html`,
  `components/segmented-buttons/specs/index.html`,
   and
  `components/segmented-buttons/accessibility/index.html`.

Correction:
 use one `SingleChoiceSegmentedButtonRow` with four `SegmentedButton`
children on one line.
 Keep all labels visible and let the Material component provide
outline,
 selected color,
 checkmark,
 semantics,
 and interaction states.

### Slider

- The real Compose `Slider` is the correct component,
   but the design gives it no
  adjacent time labels and no semantics label.
- The archive says text or icons outside a slider can make its range understandable,
  and its accessibility label should match adjacent UI text:
  `components/sliders/guidelines/index.html` and
  `components/sliders/accessibility/index.html`.

Correction:
 keep the Material slider,
 add elapsed and duration labels with tabular
figures,
 and label it `Track position`.
 Keep the current 16dp track and 4 × 44dp
handle supplied by the current Material component.

### Lists and selection

- Track rows use the correct 72dp baseline height,
   but they are custom passive
  containers rather than selectable list items.
- The selected row uses `secondaryContainer` with default `onSurface` text.
   The
  archived list mapping uses `primaryContainer` with `onPrimaryContainer` for
  selection.
- Folder choices and rail letters have visual 48dp boxes but no selectable behavior,
  role,
   ripple,
   or state semantics.
- Selection is shown by color and weight or shape in some places,
   but the selected
  track's fake play glyph is not a proper icon.
- Track titles are truncated without an available way to reveal the omitted text.
- The archive requires at least 48dp targets,
   ripple and focus behavior,
   a non-color
  selection cue,
   aligned content,
   and accessible label and role behavior.
   It also says
  information must remain available when text is truncated:
  `components/lists/guidelines/index.html`,
  `components/lists/specs/index.html`,
  `components/lists/accessibility/index.html`,
   and
  `foundations/writing/text-truncation/index.html`.

Correction:
 use selectable Material surfaces with proper roles for folder and letter
choices,
 use the baseline `ListItem` with a clickable Material action boundary,
 and use
a 72dp minimum rather than a fixed height so enlarged or long text can grow instead of
being lost.
 The archive maps `primaryContainer` / `onPrimaryContainer` to a selected
list item,
 but D36 distinguishes current playback from user selection:
 all tracks keep
full-width title placement and only the current track gains a neutral
`surfaceContainerLow` container,
 bold title,
 and semantic current-state description.
 The baseline `ListItem` overload is deprecated only in favor of the Expressive
list.
 A scoped compiler suppression is required because A2 rejects that migration.

### Color roles

- Most literals happen to resemble baseline colors,
   but the implementation bypasses
  the Android runtime scheme and includes an incorrect hand-entered `#DED8E0` value.
- Selected containers and text use unmatched role pairs.
- The flat candidate uses the important-boundary `outline` role in contexts where the
  archive requires decorative `outlineVariant` dividers.
- The archive requires custom components to map to standard roles,
   preserve intended
  role pairs,
   and keep mappings stable across breakpoints:
  `styles/color/roles/index.html`.
- Project decision A3 requires the Android system accent.
   The archive identifies a
  wallpaper-derived dynamic scheme as the user-generated source and recommends first
  checking role mappings against the baseline scheme:
  `styles/color/choosing-a-scheme/index.html` and
  `styles/color/dynamic/index.html`.

Correction:
 derive the light scheme from Android dynamic color on Android 12 and
newer,
 with the baseline light scheme as fallback.
 Express every candidate difference
through `ColorScheme` roles.
 Use only valid paired foreground and container roles.
Use `outlineVariant` for decorative rail,
 row,
 and pane dividers unless an explicit
product override wins.
 D34 keeps `outlineVariant` on the 1dp rail boundary and makes
the 24dp center spacer plus 16dp picker/transport divider white.

### Type, spacing, resizing, and writing direction

- The prototype manually repeats font sizes and weights rather than applying Material
  typography roles.
- Several text containers have fixed heights that cannot accommodate 200% type.
- Hand-built physical placement makes RTL support accidental,
   while the folder library
  explicitly includes non-Latin writing systems.
- The archive requires type roles,
   approximately 1.5 line height for body and label
  styles,
   a 200% text increase without lost content,
   leading/trailing mirroring,
   and
  spacing selected from the Compose spacing tokens:
  `styles/typography/applying-type/index.html`,
  `styles/typography/type-scale-tokens/index.html`,
  `foundations/writing/text-resizing/index.html`,
  `foundations/layout/bidirectionality-rtl/index.html`,
   and
  `styles/spacing/tokens/index.html`.

Correction:
 use `MaterialTheme.typography` roles,
 logical start/end insets,
 flexible
minimum heights,
 vertical scrolling,
 and Material components whose RTL,
 focus,
 and
state behavior is built in.
 Retain fixed left-to-right direction only for media time
and transport controls,
 where the archive explicitly requires it.
 Reflow the same single-select mode control by measured label overflow:
 one connected
row first,
 a connected 2×2 block second,
 then four connected vertical rows.
The active labels are `Repeat`,
 `In order`,
 `Shuffle Camellia`,
 and `Shuffle all`.
Native captures cover every Settings UI preset:
 one content-sized row at 85%,
two rows at 100%,
 115%,
 and 130%,
 then four rows at 150%,
 180%,
 and 200%.
Every segment keeps at least 12dp horizontal content padding;
 the control adds rows
instead of crossing that floor.
 Every target remains at least 48dp.
 The pinned Compose `SegmentedButton`
forces equal child weights and exposes no multi-row group wrapper.
 The one-row branch
therefore uses connected Material-style outlined buttons with intrinsic widths;
 later
branches compose real `SegmentedButton` elements in one outer selectable group.
 Both
apply the same colors,
 outline,
 selection fill,
 checkmark,
 and radio semantics.
The `Shuffle` cell caps its name content at rendered `Camellia` width and middle-ellipsizes
longer names without shortening accessibility semantics.
 No branch adds horizontal
scrolling.

The accessibility-review prototype replaces nested row-level segmented semantics with
direct Material-style radio segments inside one selectable group.
 Installed TalkBack then
announces all four options in accepted order at one row,
 2×2,
 and four rows.
 This is a
semantic correction;
 visible geometry,
 colors,
 12dp padding,
 and target size do not
change.

### State cues and contrast

The user's 2026-09-04 correction forbids color-only state communication.
 The repaired
prototype uses these redundant cues:

- Current track:
   selected treatment 3B removes the play icon and reserved leading
  column.
  Its neutral `surfaceContainerLow` row plus bold title preserves full text
  width while combining visible cues.
  The current row exposes
  `Current track: Another Xronixle` in the native accessibility hierarchy;
  production assistive technology must confirm the announcement.
- Selected playback mode:
   selected segmented fill plus checkmark at default and
  enlarged type.
- Selected letter:
   secondary-container color plus circular container shape.
- Selected folder:
   primary color plus medium weight plus a 2dp indicator spanning the
  bottom edge of the whole target.
   The label itself is never underlined.
- Slider value:
   active color plus handle position and track length.
- Q2b emphasis is not a state,
   but it still uses both foreground role and weight.

The folder treatment borrows MD1 tab state chrome without assigning tab semantics to
the wrapped folder targets.
 Google's archived MD1 tab specification defines a 48dp
height,
 12dp horizontal text padding,
 active accent-colored text,
 and a separate 2dp
accent indicator:
 <https://m1.material.io/components/tabs.html>.
 D31 retains 48dp
natural-width folder targets and uses the separate 2dp target-width indicator so the
label cannot be mistaken for a web link.

The captured AVD resolves the relevant dynamic light roles through Android resources.
WCAG calculations against those resolved values produce these limiting ratios:

- `onSurfaceVariant` on `surfaceDim`:
   4.532:1,
   the lowest text pair used.
- `primary` on `surfaceContainerLow`:
   5.794:1.
- `onSecondaryContainer` on `secondaryContainer`:
   6.124:1.
- `outline` on `surfaceContainerLow`:
   3.868:1,
   the lowest important non-text
  boundary pair used.
- `primary` slider track on `surfaceContainer`:
   5.500:1.

All rendered text pairs meet 4.5:1.
 Important non-text boundaries meet 3:1.
Decorative `outlineVariant` dividers do not communicate state or target boundaries.
The package validator recalculates every `onSurface` and `onSurfaceVariant` pairing
across all six surface roles plus the actual accent,
 container,
 outline,
 and slider
pairs.

### Edge-flush letter rail

The user requires the rail to match Nova Launcher's edge placement.
 The rail's 48dp
selectable bounds now begin at x=0.
 App-bar and source actions keep the measured start
inset.
 The current-screen refinement centers transport controls inside symmetric
horizontal system-gesture insets,
 correcting the prior 15dp fold-side displacement
without putting an interactive target into either edge region.
 Android's current
gesture-navigation guidance says an app
may selectively exclude an edge region only when an app gesture conflicts with Back:
<https://developer.android.com/develop/ui/views/touch-and-input/gestures/gesturenav>.
The Compose API can exclude a layout rectangle with `Modifier.systemGestureExclusion`:
<https://developer.android.com/reference/kotlin/androidx/compose/foundation/systemGestureExclusion.modifier>.

No exclusion is applied here.
 An emulator swipe from x=20px,
 inside the reported 73px
back region,
 scrolled the rail from A-I to I-Q while the activity remained visible.
This proves the vertical rail gesture works without taking the whole rail away from
predictive Back.
 UI Automator reports each rail target at `[0,...][117,...]`,
 exactly
48dp wide and flush with the physical edge.

## Existing-screen refinement boundary

Every current refinement keeps the accepted screen's visible elements and structure.
The 24dp center spacer,
 16dp horizontal divider,
 surface roles,
 folder selection,
track rows,
 current-track cue,
 metadata,
 app bars,
 slider,
 transport actions,
 and mode
control remain present and in the same order.

The accepted transport used 30dp start gesture padding plus 16dp horizontal content
padding,
 but only 16dp at the fold side.
 Its mode-control axis was therefore 15dp away
from the 414dp pane axis.
 Every refinement uses symmetric horizontal system-gesture
padding and no additional horizontal padding.
 The measured mode outline runs from
x=73 through x=935 in the 2076px capture,
 centering it at x=504 within the x=0 through
x=1008 pane while retaining at least the native gesture-safe inset.

The visible `1:06` elapsed and `4:35` duration labels imply a 66/275,
 or 24%,
 seek
position.
 Every refinement uses that value instead of the prior unrelated 16% value.

Rows use 8dp,
 12dp,
 or 16dp spacing from Material's spacing system.
 Columns use only
baseline Material icon-button color styles for the existing skip actions:
 standard,
outlined,
 or tonal.
 The pause action stays filled and all icon-button targets remain
at least 48dp.
 The downloaded Material guidance ranks filled,
 tonal,
 outlined,
 and
standard from highest to lowest emphasis and recommends mixing styles to clarify a
primary action.
 Baseline Material exposes only the default small size,
 so the matrix
does not adopt Expressive size variants.

## Candidate boundary

Every repaired Q1 option must use valid Material surface roles:

- `1a`:
   tonal hierarchy with `surface`,
   `surfaceContainerLow`,
  `surfaceContainer`,
   and `surfaceContainerHigh`.
- `1b`:
   flat `surface` regions separated only by decorative
  `outlineVariant` dividers.
- `1c`:
   `surfaceDim` ground with `surfaceContainerLowest` content and
  `surfaceContainerLow` transport.
   D34 overrides the 24dp center spacer and 16dp
  picker/transport divider to white while retaining the dynamic `outlineVariant` rail
  line.
   White is an explicit product override rather than a `ColorScheme` role;
   both
  white regions are decorative layout separation,
   never state or interactive bounds.

Every repaired Q2 option keeps the same compliant list structure,
 selected-state role
pair,
 type roles,
 icons,
 and targets.
 Only the treatment of the true-peak supporting
value may differ.
 No option may remove required accessibility information or alter the
component structure.
 D35 settles the baseline on `3a`:
 one `onSurfaceVariant`
supporting line,
 leaving custom emphasis to planned display templating.

## Acceptance boundary

Before reopening the questionnaire:

- Build the debug prototype through its `mise` task.
- Record the emulator's wallpaper-derived theme settings before using Android dynamic
  color,
   so another capture can reproduce the resolved role palette.
- Install and capture all six candidates from the unfolded Pixel 9 Pro Fold emulator.
- Confirm native system bars,
   exact source dimensions,
   opacity,
   full-bleed pane
  backgrounds,
   a centered 24dp spacer,
   a 426dp physical right-half crop containing
  12dp of spacer and the 414dp detail pane,
   default-height app bars,
   Material icons,
   one-row
  segmented control,
   48dp targets,
   and valid selected-state role pairs.
- Inspect all captures at panel resolution and in the measured questionnaire frame.
- Rebuild the self-contained form and rerun package lint and unit contracts.
- Prove each new regression guard with a failing throwaway positive control.
- Exercise all previews,
   scaling controls,
   form paths,
   responsive scrollers,
   offline
  loading,
   forced-light presentation,
   and accessibility checks in Helium.
- Clear the form and reopen that exact corrected file in the user's normal Helium
  session.

## Dark dynamic-color evidence

The Android 17 target exposes dynamic color and three user contrast levels.
 Direct
Settings inspection names Default,
 Medium,
 and High;
 Android SDK 37 source
`android/app/UiModeManager.java:730-781` maps them to 0.0,
 0.5,
 and 1.0.
 The target's theme source also accepts
TONAL_SPOT,
 VIBRANT,
 EXPRESSIVE,
 SPRITZ,
 RAINBOW,
 FRUIT_SALAD,
 CONTENT,
MONOCHROMATIC,
 CLOCK,
 CLOCK_VIBRANT,
 and CMF styles.
 These are user and OEM
environment inputs,
 not app preferences.

The debug prototype resolves `androidx.compose.material3:material3-android:1.5.0-alpha27`.
Its published source implements `dynamicDarkColorScheme(context)` by reading Android's
`mc3_dark_scheme` resource on API 34 and newer.
 Android Developer guidance says the
function derives a dark scheme from system wallpaper and changes when wallpaper changes.
The app must therefore consume semantic role pairs rather than remember palette hexes.

The matrix fixes the Android environment long enough to compare two independent
app-owned decisions.
 It uses the measured current AVD wallpaper seed plus warm coral,
forest green,
 gold Vibrant,
 magenta Expressive at medium contrast,
 and Monochromatic
at high contrast.
 These rows are stress inputs,
 never options for the user to choose.
Each row records the resolved Android roles under
`questions/evidence/dark-dynamic-*-roles.json`.

Structural surface reach has three candidates:

- R1 stable keeps the B2 black surface ladder and applies Android-generated roles to
  components,
  selected accents,
  text,
  and outlines.
- R2 zoned keeps both content canvases black but assigns generated
  `surfaceContainerLow` to the letter rail and deck.
- R3 tonal keeps the track canvas black but assigns generated low,
  standard,
  and high
  container roles across the folder canvas,
  rail,
  and deck.

Current-row neutral source has two candidates:

- C1 fixed keeps `#0A0A0D` plus the accepted bold title under every Android palette.
- C2 dynamic uses generated `surfaceContainerLow` plus the same accepted bold title.

The 2 × 3 cross-product prevents a structural-surface choice from silently deciding the
current-row source.
 Every combination retains D36's neutral container and bold title,
D37's transport hierarchy,
 D38's tonal Open,
 D39 traversal,
 and D40 state speech.
R3 is the widest permissible use of dynamic surfaces;
 it does not replace the standing
true-black track canvas or center spacer.
 Pixel probes verify both regions are exactly
black in all thirty-six opaque captures.

Contrast checks recompute selected rendered combinations from every role record instead
of trusting stored minima.
 Minimums are 6.04:1 for `primary`/`onPrimary`,
 4.63:1
for `secondaryContainer`/`onSecondaryContainer`,
 16.71:1 for `onSurface` on
black,
 9.10:1 for `onSurfaceVariant` on black,
 7.09:1 for supporting text on
the highest used generated container,
 and 3.80:1 for generated `outline` against the
darkest adjacent generated container.
 All measured text combinations clear 4.5:1 and
all measured rail-boundary combinations clear 3:1.
`questions/evidence/dark-dynamic-contrast.json` retains every per-environment value.

These captures establish palette-dependent appearance only.
 The prototype is relaunched
after Android roles settle,
 so runtime wallpaper-change propagation remains later
implementation verification rather than a claim of this design matrix.

## Command-bar study correction (2026-09-23)

The user rejected the first desktop command-bar scenes as visibly non-MD3.
 The
Slint renderer made the rasters native,
 but the component was a generic outlined
palette with 8px input corners,
 8px to 16px container corners,
 text glyphs in place of
Material icons,
 and a system Noto fallback.
 Its successful form and pixel checks did
not validate its visual lineage.
 The first `command-round-s*-c*` captures and their
ranking are historical experiments,
 not options to present or carry forward.
 The
corrected I/G/R questionnaire built from those scenes was also withdrawn before any
user choice.

The user's local Material archive at
`~/Downloads/m3.material.io/components/search/specs/index.html` distinguishes
**baseline divided** from **M3 Expressive contained** Search.
 A2 retains baseline
Material 3,
 so the guidelines' contained screenshots
(`components/search/guidelines/images/20.png`,
 `28.png`,
 and `37.png`) are a
visual contrast,
 not permission to switch styles.
 The archive's
`components/search/specs/manifest.json` records original high-resolution URLs for
`images/34.png` and `images/35.png`;
 replacing only the archived `=w40` image
request with `=w1200` exposes the baseline divided measurements at readable size.
The baseline collapsed field is a filled,
 56dp fully rounded bar.
 On expansion,
 the
docked **whole container** is rounded 28dp with a 56dp header and a divider before
results;
 the header is not another pill drawn inside that container.
 The baseline
full-screen search has sharp outer corners,
 a 72dp header,
 and a divider.
 Leading
Back and trailing Clear occupy their measured icon slots.
 Docked results are for
medium and expanded layouts;
 compact focused search is full-screen by default.
A compact desktop dock is an explicit adaptation,
 not compliance by assertion.

AndroidX Material3 source makes the shape and colors explicit:
`SearchBarDefaults` in
`compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
reads `SearchBarTokens.ContainerHeight`,
 `ContainerShape`,
 and
`SearchViewTokens.DockedContainerShape`.
 In the corresponding AndroidX generated
[search bar tokens][search-bar-tokens],
 `SearchBarTokens.ContainerHeight` is 56dp,
 `ContainerShape` is
`ShapeKeyTokens.CornerFull`,
 `ContainerColor` is `SurfaceContainerHigh`,
 and
`InputTextFont` is `BodyLarge`;
 the generated [search view tokens][search-view-tokens] set
`SearchViewTokens.DockedContainerShape` to
`CornerExtraLarge` (28dp),
 `FullScreenContainerShape` is `CornerNone`,
 and
header heights are 56dp docked and 72dp full-screen.
 The baseline
`ExpandedDockedSearchBar` and `ExpandedFullScreenSearchBar` in
`SearchBar.kt` draw a `HorizontalDivider` between their header and results,
whereas the contained style is structurally different.
 AndroidX
[shape tokens][shape-tokens] assign 28dp to `CornerExtraLarge` and a circle to
`CornerFull`.
 These are source-backed
values,
 not inferred from the blurred token thumbnails.
 The archive's list specs
say baseline list items have square corners,
 16dp label padding and 48dp minimum
targets;
 the 16dp selected-item corner belongs to **Expressive** lists and must not
be silently combined with the baseline Search choice.
 `ListItem.kt:1148-1150`
chooses [ListTokens.ItemTwoLineContainerHeight][list-tokens] for a two-line item;
that value is **72dp**,
 with 24dp leading icons and BodyLarge/BodyMedium label text.
The corrected Slint study now uses continuous 72px two-line rows.
 Its 360px
larger-text test uses 96px rows as an explicit stress adaptation,
 not a baseline
token or proof of an automatic breakpoint.
 The command palette's
keyboard-current row is a custom focus state,
 not a persistent selected folder;
its redundant cues must be identified as a desktop adaptation.

The first MD3 redraw in prototype commit `c5b3fa31f` also mixed the Expressive
contained pill and rounded selected rows with A2's baseline requirement;
 it is an
intermediate experiment,
 not presentation evidence.
 The current redraw in prototype
commits `0e9624dd2` and `7adafa092` keeps the Roboto font and official Material
icon assets on the prototype branch while adopting
**baseline divided** anatomy:
 56px header,
 28px docked outer shape,
 1px divider,
72px full-content header with sharp outer corners,
 square baseline list rows with
16px start padding and a disclosed custom keyboard-focus cue.
 Color roles come from
`questions/evidence/cover-round-wallpaper-roles-{light,dark}.json` as study input.
The active native rasters now live at `questions/render/command-md3-*`.
 At native
480 × 600px in the light docked scene,
 pixels at x=200/y=120 (header) are
`#E7E7F1` (`surfaceContainerHigh`),
 x=200/y=148 (divider) are `#797A84`
(`outline`),
 and x=24/y=223 (keyboard-focus boundary) are `#4E5E8B`
(`primary`).
 Dark pairs are `#1E1F26`,
 `#73757F` and `#BAC5EE`.
The normalized 56px header contact at
`questions/evidence/command-md3-baseline-header-comparison.png` places a crop
from the readable baseline docked spec beside the Slint header,
 without enlarging
either logical height.
 The reference still contains Google's pink measurement
annotations;
 it establishes header/divider shape,
 not an exact palette match or
accessible behavior.
 List rows were compared against the separate baseline list
specs;
 the spec's search header diagram contains no actual result data.
 The custom
keyboard focus uses an outline and state-layer fill rather than an Expressive selected
corner.
 Document every desktop
adaptation rather than labelling it compliant by resemblance.
 Slint snapshots still
do not prove accessibility,
 keyboard interaction,
 OS window behavior,
 or global
hotkey feasibility.
 No KWin window-management automation is needed for this round.

## D47/D48: one Search page, one integrated header

The user rejected every I/G/R command-bar appearance and chose a Search icon button
that opens a separate page (D47).
 The first page prototype stacked a "Search" app
bar over a query bar;
 the user removed that redundancy (D48).
 The desktop-sized prototype
then used baseline full-content Search anatomy:
 one 72px header contains Back,
query and Clear,
 a 1px `outline` divider separates results,
 and the page has sharp
outer corners.
 On the player,
 a standard 48px icon-button target holding a 24px Search glyph opens this page;
no player controls persisted inside **that retired desktop-sized experiment**.
This was superseded by D50/D51:
 the unfolded deck must stay visible,
and selected A lifts it above the measured bottom debug keyboard while Search
remains on the right.
 A later real floating Gboard occluded part of that
deck title at 200% text and both cover result labels at 100% text.
On a disposable Fold,
real split inner and full-width cover Gboard keys entered `cam` at
100% and 200% text;
the settled deck and result labels stayed visible.
A Gboard font-update banner briefly clipped the final inner mode.
D53 to D55 accept the measured floating-deck,
brief banner and cover floating-result overlaps respectively;
those exceptions do not establish D50 for other keyboard states or change
Material 3 component requirements.
`package/music-player/design/evidence/gboard-geometry.md` records the separate real-keyboard states,
not a new Material component rule.
 The separate destination is still visually distinct from a command
palette or modal overlay.

The same local archive's Search guidelines say an icon button is appropriate when
search is a secondary action.
 Its baseline specs and AndroidX
`SearchViewTokens.FullScreenHeaderContainerHeight` give the 72dp header;
`ExpandedFullScreenSearchBar` in `SearchBar.kt` draws the divider.
 The normalized
source-versus-study contact at
`questions/evidence/search-page-header-comparison.png` compares the readable
baseline `components/search/specs/images/34.png` request at `=w1200` (pink source
measurements retained) to the Slint page header at the same 72px logical height.
The study's query/result examples are sample content,
 not an approved index or
ranking.

The native light/dark study captures at
`questions/render/search-page-*` show the trigger,
 empty page,
 `cam` sample,
no matches and unavailable library at desktop widths 360 and 480px.
The 1100px stress viewport covers the player trigger,
 empty page and illustrative
results,
 not the no-match or unavailable states.
At compact 480px,
 the rendered header is `#E7E7F1` light / `#1E1F26` dark,
its divider is `#797A84` / `#73757F`,
 and the page body remains white / black.
The Slint source wires Search and Back TouchAreas and gives the input
`input-type: search`;
 offscreen rasters do not prove native click delivery,
 focus
restoration,
 screen-reader output,
 indexing,
 or a backend.
 The HTML walkthrough
is a separate interactive design artifact,
 not a substitute for native integration
verification.
 D21's former global command hotkey and Settings row are not copied
to Search;
 D25's Ctrl+F reservation is part of the keyboard map D102 accepted.

## D49: native Fold geometry replaces desktop-size Search evidence

The user corrected the round's target:
 all visual design follows the Pixel 9 Pro Fold
cover panel (1080 × 2424 physical px,
 approximately 443 × 994dp on this
390dpi AVD) and unfolded inner panel (2076 × 2152 physical px,
 about 852 ×
883dp),
 as measured in `device-metrics.md`.
 The former 411 × 923dp cover
value came from published ppi,
 not the emulator's Android density.
Desktop implementation inherits these treatments even if a distinct desktop layout
would appear more efficient.
 The 360 × 640,
 480 × 600 and 1100 × 640px Slint
images,
 their normalized Search header comparison,
 and the browser walkthrough
verify a previous desktop-sized experiment only.
 None measures the D47/D48 page on
either target panel.
 The withdrawn debug-only Compose study has opaque captures at both real panel
resolutions in both schemes,
 with a player Search target,
 one 72dp
Back/query/Clear header,
 sample results and empty states.
 It placed the
unfolded header above a blank left pane and all Search content in the right
pane.
 The user rejected that composition.
 Its pixel and hierarchy checks
established conformity to an **obsolete** all-paint/all-hit-region connector
ban,
 not to the user's clarified E2.
 Baseline M3 full-content Search guidance
does not prescribe that disconnected pane allocation;
 the withdrawn study
must not be presented as an MD3-endorsed adaptation.
 The one-header direction D48 remains;
 D51 later selected the right-side Search
pane with an IME-lifted left deck.
 D52 removed the separate positive-results
query heading so results follow the integrated header directly.
 A static capture also
does not prove live search,
 accessibility focus or result execution.
 Do not call a Slint-native capture a device-native Android
proof or ask for a desktop window size as a design input.

[search-bar-tokens]: https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SearchBarTokens.kt
[search-view-tokens]: https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SearchViewTokens.kt
[shape-tokens]: https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ShapeTokens.kt
[list-tokens]: https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ListTokens.kt

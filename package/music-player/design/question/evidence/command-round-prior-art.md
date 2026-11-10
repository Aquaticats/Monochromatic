# Command bar round: prior art and scope

This is design evidence,
 not a production specification.
 The first S × C matrix was an
internal draft and has been withdrawn from review:
 it treated in-app placement and
global invocation as exclusive,
 treated D25's shortcut reservation as a requirement for
separate surfaces,
 and compared different input/result states.
 Its native captures remain
historical experiment evidence only;
 no option has been put to the user or selected.
The active round is being reframed around independent in-app placement,
 global
invocation,
 and search-surface relationship decisions.
 D21 already settles a command bar
with a configurable global hotkey off by default.
 D25 reserves Ctrl+F for future
search and Ctrl+O for the folder picker.
 A keyboard-map pass remains open.

## Primary sources

- The local Material 3 archive at
  `~/Downloads/m3.material.io/components/search/{overview,guidelines,specs}/index.html`
  distinguishes a search bar for the current view,
  a search app bar for primary global
  search,
  and an icon entry for secondary search.
  Focused results appear in lists,
  optionally separated by category labels or gaps.
  On medium and expanded layouts,
  docked results are supported;
  compact layouts default to full-screen focused search.
  Its contained search style uses a persistent filled container and recommends
  surface roles separated by more than one tonal step.
  The search bar is 56dp high;
  the docked container is 360 to 720dp wide and 240dp to two-thirds of the screen high.
  These are search-component precedents,
  not a mandate that the D21 command bar must
  perform media search.
- [IntelliJ IDEA Search Everywhere][jetbrains-search] is a single entry point for files,
  actions,
  symbols,
  settings and more,
  with tabs and scoped shortcuts;
  it opens on
  recent files before typing.
  This supports an explicit-scope alternative,
  but does
  not erase D25's already reserved keys in this player.
- [VS Code Command Palette][vscode-palette] uses Ctrl+Shift+P for commands while Ctrl+P
  opens file navigation;
  its quick-input surface can also switch modes.
  This
  supports separating actions from navigation even when a common surface is reused.

## Native design study

The throwaway Slint study is on the prototype branch at
`package/music-player/design/candidate/command-study.slint`.
 Slint Viewer 1.17.1
rendered the opaque rasters in `question/render/command-round-*` at 480 × 600px
(the current desktop app's `preferred-width` and `preferred-height` in
`package/music-player/desktop-app/ui/app.slint`) for every S × C combination and in
both schemes.
 The recommended S1C1 combination also appears at an exploratory
1100 × 640px stress size;
 neither size decides the open default-window question (11c).

The background is a simplified,
 design-only Slint player scene preserving the D43
no-volume choice,
 D38 Open action,
 current track emphasis and folder context.
 It is
not a screenshot of the current production desktop app.
 Role colors use the measured
Android wallpaper evidence in `question/evidence/cover-round-wallpaper-roles-{dark,light}.json`
as design-study references;
 this does not settle the desktop color mapping.
 S3 is an
illustration of a detached quick window drawn inside one Slint scene,
 not a functioning
second operating-system window or a global-hotkey feasibility test.

The independent axes are:

- **S1,
  docked under the player header:**
  the command surface stays with its source
  window,
  covering content below the folder row.
- **S2,
  floating inside the player:**
  centered and scrimmed,
  detached from the header
  but still inside the app.
- **S3,
  detached quick-window concept:**
  a separate-looking palette over the player,
  intended to express the globally invoked state when the player is not focused.
- **C1,
  actions only:**
  results are executable commands;
  Ctrl+F remains a separate
  future media search entry.
- **C2,
  explicit modes:**
  Actions,
  Tracks and Folders scopes share a palette but do not
  mix their results in one list.
- **C3,
  mixed results:**
  folders,
  tracks and actions appear together under group labels.

C2 and C3 use tracks and folders as illustrative result types,
 not a decision on the
future search index or ranking.
 The draft incorrectly asked for one S code when S3
could coexist with either in-app surface;
 the user was never asked to choose these codes.

## Historical first-draft ranking (withdrawn)

- **Surface:**
  S1 > S2 > S3 was not a valid exclusive ranking.
  S1 and S2 address
  in-app placement;
  S3 addresses a globally invoked window relationship that may
  coexist with either.
  The draft did not depict another app in focus or an actual
  detached OS window.
- **Content:**
  C1 > C2 > C3.
  The claimed D25 justification was incorrect:
  D25 reserves shortcuts but does not require separate surfaces.
  C1 might still have a useful separation tradeoff;
  C2 makes the distinction explicit but spends a row on modes and may
  duplicate the picker;
  C3 finds everything in one place but makes result relevance
  and Enter's effect less predictable.

## Second study, independent axes but visually rejected

The second throwaway Slint source is committed on the prototype branch as
`ee1b2d59c`,
 with native-fit fixes in `dbbf5116e` and `6d533f740`.
 Its
`question/render/command-round-{i,g,r,e}*` captures separated the decision axes,
but the user rejected the visible component as non-MD3.
 These captures are also
historical evidence,
 not a live comparison.
 Every proposed state has an opaque light and dark Slint raster:

- **I (in-app placement):**
  I1 docked below the header,
  I2 floating within the player,
  I3 replacing the content region.
  All show the same typed `open` action query,
  matching actions and selected result.
  Each appears at 480 × 600 and 1100 × 640px.
  I1 also has a 360 × 640px synthetic larger-text and long-label stress capture.
- **G (optional global invocation):**
  G1 raises the player from a previously active
  notes app,
  G2 draws a detached quick surface over that app.
  I and G are
  complementary;
  the Slint scenes do not implement an actual second OS window or
  hotkey.
- **R (future Ctrl+F):**
  R1 uses a separate search destination,
  R2 an explicit
  search mode in the command surface.
  Both depict the same illustrative `cam`
  query and folder/track results.
  D25 settles only the reserved shortcut,
  not
  whether those surfaces are shared.
- **Edge probes:**
  E1 names no matching actions;
  E2 explicitly names an action
  unavailable during a scan.
  They are contextual evidence,
  not extra choices.

The native scenes do not establish keyboard navigation,
 screen-reader semantics,
 focus
restoration,
 result execution,
 search indexing,
 or real compositor window behavior.
The simplified player behind the overlays draws D43's absence of volume,
 a 24% seek
state,
 a pause glyph matching the suggested action,
 D38's tonal Open and current-row
emphasis.
 It remains a design representation rather than an exact desktop application
screenshot.
 The native 360px stress uses explicit larger prototype text,
 not a measured
OS UI-font preset.

Withdrawn questionnaire rankings were **I1 > I2 > I3** (source anchor over floating,
context retention over full replacement),
 **G2 > G1** (keep the previous task visible
over raising the player),
 and **R1 > R2** (dedicated navigation room over mode
switching).
 They are neither settled decisions nor recommendations to carry into the MD3 redraw
without a new visual review.
 Search targets,
 result ranking,
 a discoverable in-app entry and the exact
in-app binding remain open.

## Baseline Material 3 redraw for the current I/G/R form

A2 retains baseline M3,
 so the earlier Expressive contained Search redraw
(`c5b3fa31f`) is also an intermediate experiment,
 not the presented source.
The local `components/search/specs/manifest.json` provides the original image URLs
for baseline **divided** Search diagrams `images/34.png` and `images/35.png`;
requesting their existing image IDs at `=w1200` instead of the archive's `=w40`
yields readable 72dp full-content and 56dp docked header measurements.
The AndroidX generated SearchBar,
 SearchView and Shape tokens confirm a 28dp docked
outer shape,
 sharp full-content outer shape,
 a `surfaceContainerHigh` 56dp header,
BodyLarge text,
 and 56dp/72dp expanded header heights.
 Baseline
`ExpandedDockedSearchBar` and `ExpandedFullScreenSearchBar` both draw the header
divider in `SearchBar.kt`.
 Baseline list items remain square with 16dp label padding;
AndroidX `ListTokens.ItemTwoLineContainerHeight` fixes their two-line height at
72dp (via `ListItem.kt:1148-1150`).
 16dp selected corners belong to Expressive
lists and were removed.

The throwaway prototype source on branch `prototype/music-player-theme-compose` is
`package/music-player/design/candidate/command-md3-study.slint`.
 The baseline
redraw is committed in `0e9624dd2`,
 with accepted-player transport backdrop
correction at `7adafa092` and baseline two-line row pitch in `c88786448`.
 Rendered rasters at
`question/render/command-md3-{i,g,r,e}*` are opaque native Slint in both schemes.
The 56px logical-height comparison at
`question/evidence/command-md3-baseline-header-comparison.png` preserves source
and prototype heights without upscaling;
 the source crop retains Google's pink
measurement marks and uses a different palette seed.
 The screenshot comparison
establishes geometry and visible hierarchy,
 not pixel-for-pixel color identity.
The I1 compact scene samples `surfaceContainerHigh` in its header and `outline`
at its divider;
 the focus row has an independent primary outline and state-layer fill.
That keyboard focus is a disclosed desktop adaptation,
 not a persistently selected
folder.

New visual rankings remain **I1 > I2 > I3** for app context,
 **G2 > G1** for
preserving the previous task,
 and **R1 > R2** for keeping search a distinct navigation task rather than adding
palette mode switching.
 R1 and R2 can have the same room if I3 is chosen;
 the R2
illustration uses I2 placement and G1 uses I1 placement without deciding I.
The form asks these axes separately;
 none is a user decision yet.
 The I1 360px
stress capture proposes a full-content fallback to preserve 12px margins and the
360dp docked minimum;
 a static `stress` property chooses it,
 so a runtime responsive
transition was not verified.
 The 96px larger-text row is a documented adaptation,
not a baseline token.
No runtime hotkey,
 OS quick window,
 search index,
 action execution or native
accessibility behavior was built by the static Slint study.
 The exact desktop
window size remains open and the 1100 × 640px rasters are a stress viewport only.

[jetbrains-search]: https://www.jetbrains.com/help/idea/searching-everywhere.html
[vscode-palette]: https://code.visualstudio.com/docs/editing/getting-started/userinterface#_command-palette

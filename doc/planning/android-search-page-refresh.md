# Android Search page refresh proposal

## Status

Accepted on 2026-10-14: option 1, "Restyle within decisions", recorded as D126 in
`package/music-player/design/decisions.md`, which also holds the exact specification the desktop port copies.

Rulings:

- Direction: option 1, with R2 adopted (the header is the canvas color and the divider uses `outlineVariant`).
- R3: no section labels; results stay one list mixed by relevance (D52, D61).
- R4: no recents list.
- R1 (grouped cards, tinted page) was not adopted, and option 4 stays rejected.
- R5: parity on desktop is wanted, as a separate follow-up task.

Implemented on branch `feat/android-search-restyle` (Android only):

- Header and status strip in the canvas tone, `outlineVariant` divider.
- Result rows with a 40 dp `secondaryContainer` circle in the 48 dp slot, a rounded 21.3 dp press and ring shape that
  starts 8 dp before the content and keeps the D117 and D120 ring just inside its edge.
- The empty and no-result states in a 72 dp circle, `titleMedium` at weight 500, starting 72 dp under the divider on
  both panels, padded by the keyboard inset on the cover and scrollable.
- Tests: `SearchLayoutTest` and `SearchPageWiringTest` (pixel and geometry cases under Robolectric native graphics).
- The Android README gallery image `android-cover-search-light.png` was retaken.
- The two items of the unverified list were read from the code and left open: the query survives a fold change because
  the host keeps it with the route, and `Ctrl+F` on an open page only refocuses the field while the caret of the
  rebuilt field restarts at the start of its text, so a second `lan` typed after a fold landed inside the first (the
  device run gave `lanlan` and then `lanxlan` for an extra `x`). D66 covers a new visit, not a posture change.

Desktop port done on branch `feat/desktop-search-restyle` (2026-10-14), parity being the human's ruling (R5):

- `ui/search.slint` now has the canvas header, the `outline-variant` divider, flat rows with the rounded 21.3 px shape
  (hover layer, D117 and D120 ring pair and pointer target inside it), the 40 px `secondary-container` circle in the 48
  px slot, and the one scrollable status block for the five states.
  The desktop choices (pointer target, scrollbar track against the shape's end, no hint badge to keep, no crease inset
  in the app as built) are the addendum in D126.
- Tests: `src/ui_search_restyle_tests.rs` (thirteen window tests on the software renderer) and the updated row case in
  `src/ui_focus_ring_pages_tests.rs`.
- `desktop-search-light.png` and `desktop-search-dark.png` were retaken from the release build of the branch
  (`doc/handover/music-player-readme-and-gallery.md`).
- Unverified: macOS, Windows, a real Plasma session and touch input.

Left:

- A decision on whether the caret should survive the rebuild of the header on a fold change, if the human wants it.

### Final specification for the desktop port

Sizes are dp on Android and logical px on desktop; roles are the shared theme roles (Slint `Theme.<role>` with dashes).
Copy and the Rust page model do not change.

- Header: 72 high, filled with `canvas` (pure white in light, pure black in dark), so the status strip above it, the
  header and the results are one tone. Remove the `Theme.surface-high` fill. Back and Clear keep their 48 targets, 40
  circular state layers and 24 glyphs at the same x as before.
- Divider: 1 high, full pane width, directly under the header (y = 72), role `outline-variant` (was `outline`).
- Result row, flat on the canvas at rest, minimum height 72:
  the row's rounded shape spans x from `content-start-x - 8` to `width - end-inset + 8` (8 before the content on
  each side), corner radius 21.3 on all four corners, and the row content is padded 8 on each side so every glyph
  and title keeps its D58 x.
  On the inner panel `content-start-x` already holds half the crease plus 16, so the shape starts 8 after the crease
  clear zone and never inside it.
- Row icon: a 48 wide slot at the content start holding a 40 circle, centered in the slot (4 inside its edge), filled
  `secondary-container`, with the folder or music glyph at 24, centered, in `on-secondary-container`. The title starts
  at the content start plus 48 with no extra gap. The kind label stays on the glyph for assistive technology (D77).
- Row states: the hover and pressed layers (and the Android indication) are clipped to the 21.3 rounded shape. The
  keyboard highlight is the D117 pair of solid rings, drawn just inside the same rounded shape (D120): the outer ring
  1 wide in `on-primary` at the shape's edge, the near ring 2 wide in `primary` inside it, following the 21.3 corners,
  shown for keyboard focus only, with no fill, tint, bold or size change. The ring and the state layers never leave the
  shape.
- Empty and no-result states (all five status states share one block): the block starts 72 below the divider's bottom
  edge on both layouts (no longer centered in the body), and it holds, top to bottom: a 72 circle filled
  `secondary-container` holding the 32 magnifier glyph in `on-secondary-container`; 24 below it the title in the
  Material 3 `titleMedium` role (16 on a 24 line, tracking 0.15) at weight 500 (was `titleLarge` at 600); then the
  support line in `bodyMedium` in `on-surface-variant`, 8 below the title on the cover and 12 on the inner panel.
  Cover: block centered horizontally, 16 side padding. Inner: block left aligned at the content start, the circle's left
  edge at `content-start-x`.
- Keyboard clearance: on Android the cover block is padded by the keyboard and navigation insets, like the result list,
  and scrolls when the space above the keyboard is shorter than the block (large text, split or floating keyboard).
  The inner panel was already lifted by its row inset and also scrolls. Desktop has no on-screen keyboard; it keeps
  the block scrollable when the window is short.
- Unchanged on desktop and Android: highlight (bold plus the match fill role, square corners), result copy,
  second line, the 72 minimum row height, scrolling, the key hint rule (the Android app has no key hint badges; on
  desktop a hint badge slot at the row's end keeps its place inside the rounded shape's end padding).

The rest of this document is the proposal as written, with the file and line numbers it cited on 2026-10-14 before the
change.


## Reading of the request

The request was "Our search page still looks outdated.
Check Android settings' search page."
The reading used here:

- "Our search page" is the music player's Android Search page:
  `SearchPage.kt` in `package/music-player/android-app`, the Search route of the production app.
- "Android settings' search page" is the search screen of the stock Android Settings app on the Pixel 9 Pro Fold
  emulator (API 37, Android 17, Material 3 Expressive era), taken as the current visual reference.

Other plausible readings, checked and not chosen:

- "Our search page" is the desktop Search page (`ui/search.slint`).
  It mirrors the Android anatomy exactly (72 px header, `Theme.surface-high` fill, 1 px `Theme.outline` divider),
  so every Android finding applies to it too, and the cost section counts the port.
- "Android settings" is the music player's own Settings page, which D109 already built to follow stock Settings.
  The comparison shows that the Settings page and the Search page are now visibly different generations of the same
  app.

## Summary

The header anatomy of the stock search screen is the same as ours: a 72 dp toolbar, a Back target of 48 dp,
a trailing Clear, and a divider (the Settings capture and `SearchViewTokens` agree).
So the dated look does not come from structure or type scale.
It comes from surface and weight: a gray header band under a white status strip, a heavy dark divider,
flat rows with bare 24 dp glyphs, and empty and no-result states that are a small glyph plus a heavy title,
placed low on the cover.

What makes the stock screen look newer (grouped rounded cards, a tinted page, a scalloped empty-state shape,
section labels, a tinted dark surface) is mostly ruled out or already vetoed by accepted decisions:
A2, B1, D52, D61 and D109.
The recommended option therefore changes only what those decisions leave open,
and asks for five rulings before anything that touches them.

Recommended: restyle our own composables inside the accepted decisions (option 1 in the ranking),
with no new color role and no new library component.

## Method and evidence

### Our app

- Debug APK built from this branch, which is `main` at `d972299a30`, with the repo's podman Gradle runner pattern
  (`assembleDebug`, 21 s, mostly cached).
  The native libraries (`.so` files) were copied from another worktree's earlier build.
  That worktree's commit `29dfe1b934` differs from `main` in `package/music-player/android-app` (including its
  `rust` crate) and `package/music-player/truepeak-core` only by six added test files (`git diff --stat`),
  and Search is Kotlin only, so the libraries cannot affect this comparison.
  `SearchPage.kt` last changed in `6afe69c81a` (2026-10-13), so the page is current.
- Library: the invented 35-track gallery library (`Aster Vale`, `Dune Lantern`, `Lantern Season` and others),
  pushed to `/sdcard/Music` of a private reflink copy of the owned AVD;
  `RealAppTest` removed;
  a MediaStore query returned exactly those 35 rows.
- One emulator launch, headful on the host GPU, as the systemd user unit `sp-emulator` with `MemoryMax=12G`,
  console ports 5762 and 5763, private adb server 5291.
  It booted in 41 s and did not crash.
  Settings were recorded before and restored after (font scale 1.0, night mode off, posture OPENED, demo mode off,
  verified equal), then `sync`, 8 s, `sync`, `adb emu kill`;
  the unit is inactive.
- Demo mode was on during captures.
  Every kept capture is cropped at native width to remove the status and navigation bars (cover `1080x2240+0+125`,
  inner `2076x1964+0+110`) with metadata stripped.
  Raws and unreviewed frames were deleted.
  Nothing was committed or uploaded;
  the captures are in `/home/user/temp/agent/sp/kept`, and each difference below names its pair by file name.

### The stock reference

- The search screen on this image is not in the Settings app.
  `SearchFeatureProviderImpl.kt` in AOSP `packages/apps/Settings` builds an `ACTION_APP_SEARCH_SETTINGS` intent
  addressed to the SettingsIntelligence package, and the screen runs in `com.google.android.settings.intelligence`.
- Its view ids (`open_search_view_status_bar_spacer`, `open_search_view_toolbar`, `open_search_view_divider`,
  `open_search_view_clear_button`, `open_search_view_edit_text`,
  `open_search_view_content_container`) are exactly the ids of `mtrl_search_view.xml` in
  material-components-android.
  So the screen is the View-system Material `SearchView`, full-screen.
- The public AOSP `SearchFragment.java` of SettingsIntelligence uses an older toolbar `SearchView` and is not what
  this image runs.
  The result rows and the empty states come from closed source, so their geometry here comes from UI Automator dumps
  and pixel measurements of the captures, not from layout files.
- The home page entry (a collapsed pill) is the Settings app's own `search_action_bar`.

### The Compose library we ship

`androidx.compose.material3:material3:1.5.0-beta01` (`app/build.gradle.kts`, BOM `2026.09.00`),
read from the installed `classes.jar` with `javap`:

- `SearchBarKt` has `SearchBar`, `TopSearchBar`, `AppBarWithSearch`, `ExpandedFullScreenSearchBar`,
  `ExpandedFullScreenContainedSearchBar`, `ExpandedDockedSearchBar`,
  and the state factories `rememberSearchBarState`, `rememberContainedSearchBarState` and
  `rememberSearchBarWithGapState`.
- `ExpandedFullScreenSearchBar` takes a `DialogProperties` parameter and its implementation
  (`ExpandedFullScreenSearchBarImpl`) references `DialogProperties`, so it is a full-screen dialog window.
  That it is a separate window is read from the signature and class references;
  its runtime behavior was not run.
- `SearchViewTokens` (declaration order of the class): container color `SurfaceContainerHigh`,
  divider color `Outline`, docked shape `CornerExtraLarge`, docked header 56 dp, full-screen shape `CornerNone`,
  full-screen header 72 dp.
  So our header fill and divider are the baseline M3 tokens exactly.
- `SearchBarDefaults` has `InputField` (three overloads), `inputFieldHeight`,
  `containedColors` and `fullScreenContainedSearchBarColor`. The contained pair is the Expressive style;
  the divided pair is the baseline style that A2 retains.
- The local Material archive named in `design/material-3-compliance.md`
  (`~/Downloads/m3.material.io/components/search/specs`) exists and separates "contained" from baseline "divided"
  search.
  The live m3.material.io page returns only its title to a fetch, so no live-page value is cited.

## Constraining decisions

Accepted decisions that a refresh must not contradict without a ruling:

- A2:
  Material 3 baseline, not Expressive, because Expressive spends vertical space a dense list cannot afford.
- A3, A4, D49: one visual identity on all platforms, desktop in Slint by hand,
  the Fold panels are the visual source.
- B1: dark is true black, a standing rule never to be re-asked;
  B4: secondary text passes AA on black.
- D47, D48:
  Search is a separate page;
  one integrated Back, query, Clear header of 72 dp with a divider, results directly beneath it.
- D50, D51, D76: the playback deck is never hidden on the unfolded panel;
  Search is the right half with the folder browser and deck on the left;
  Search is not a focus trap;
  the cover has one full-width page.
- D52: no repeated results heading.
- D58:
  Back icon and result icons share one horizontal center, query text and result titles share one start.
- D59: every visible match is emphasized in place, from the accent in OKLCH, with bold as the second cue.
- D60, D61: direct-name matching;
  results mixed by relevance, not grouped by kind.
- D63 to D68, D75 to D80: focus, keyboard, Clear, return and accessibility behavior.
- D66: a new Search visit starts with an empty query;
  D69 to D71: status truth conditions;
  D70: the empty prompt "Search your music" and the no-match title naming the entered text.
- D77: each result is one named action with parent context and a minimum 48 dp target.
- D108, D118, D121: the OS accent is the root and every derived color is OKLCH;
  grays carry the accent hue;
  roles are shared by a Rust and Kotlin fixture of 52 roots in both schemes (`theme-roles-vector.txt`).
- D109: the human rejected Google's tinted backgrounds in Settings ("everything should be in one color,
  white or black");
  page, header, cards and rows use the one canvas color, with no header band, card fill or divider;
  group rounding survives only as the shape of pressed, hovered and selected states;
  the icon circle is 40 dp in the secondary container role.
- D111, D112:
  Search works while scanning and runs on an in-memory index (not affected, no change proposed).
- D117, D120: the focus ring is two solid rings hugging the element;
  on rows of a clipped list it sits just inside the edge.
- C1: 48 dp minimum targets;
  C2: two-line rows are 72 dp;
  C4:
  Roboto 400 and 500 with M3 type roles.

Two facts matter for the redesign.
First, D48 predates D109, and D109 states the human's taste against header bands and tinted surfaces for Settings,
while the Search header is still a `surfaceContainerHigh` band.
Second, the Android README says the key hint badges are left out on Android, so key hints constrain only the desktop
port.

## Differences found

Each item gives what the stock screen does, what ours does with file and line, the decision touched,
the proposed change, the Material 3 token, and cost and risk.
Pixel values were sampled from the cropped native captures: the light pair on the folded cover,
the stock dark capture on the inner panel, ours dark on the cover (the two dark captures are not the same panel).

### Header surface and the status strip

- Stock: status strip, toolbar and page are one tone (`#EDEDF6` light, `#17191F` dark).
  The toolbar and the page meet at a divider only.
- Ours: the status strip is the canvas (`#FFFFFF`, `#000000`: `SearchPage.kt:944` and `:1009`,
  with the canvas set at `:905`) and the header under it is a band of `surfaceContainerHigh` (`#E8E7EE`, `#1E1F25`:
  `:1071`). The result is a white strip over a gray band on both panels;
  it is visible in the committed gallery image `android-cover-search-light.png` as well.
- Touches:
  D48 (anatomy stays), D109 (taste), B1 (dark canvas).
- Proposal: make the header the canvas color, so strip, header and results are one tone,
  keeping the 72 dp toolbar and the divider (D48 anatomy intact).
  The band's only job (separating header from results) is done by the divider.
  Alternative if the human wants the band kept: extend the band color behind the status bar so the strip disappears.
- Token: the `canvas` value that `SearchPage.kt:905` already sets (`surfaceContainerLowest` in light,
  `SEARCH_TRUE_BLACK` in dark);
  no new role.
- Captures: stock `c4-lan-light.png` against ours `a3-lan-light.png` (cover, light).
- Cost: a few lines in `searchHeader` and the two page composables, plus the same two properties in `search.slint`
  (`Theme.surface-high` at line 68).
  Risk: low;
  contrast of the Back, query and Clear glyphs on the canvas is higher than on the band.
- Needs ruling:
  R2 (is the header band part of D48 or only its anatomy?).

### Divider weight and color

- Stock: 2 px (about 1 dp) of `#D6D6DF` on `#EDEDF6`;
  dark `#303238` on `#17191F`.
- Ours:
  `outline` at 1 dp (`SearchPage.kt:946` and `:1016`), which renders as `#757780` in light and `#8E9099` in dark
  against pure black, the strongest line on the page.
  It is the baseline token (`SearchViewTokens` divider color is `Outline`).
- Touches:
  D48 (names a divider, not its role).
- Proposal: use the existing `outlineVariant` role for the Search divider, as the hairline dividers elsewhere do.
- Cost: one argument at two call sites, one color in Slint (line 222).
  Risk: very low.
- Needs ruling:
  R2 (same ruling as the header surface).
- Captures: stock `c4-lan-light.png` against ours `a3-lan-light.png`;
  the line weights are measured in these two.

### Result rows

- Stock: each result is a rounded card of about 68 dp (167 px at 2.4375 px per dp) inset 16 dp,
  corner radius about 20 dp (estimated from the corner profile of the capture), title in body large and a breadcrumb
  line (`Network & internet > Internet`) in body medium, some with a trailing switch.
  Rows of one group sit 2 dp apart;
  groups are separated by a small primary-tinted label.
- Ours: flat rows on the canvas, `heightIn(min = 72.dp)` (`SearchPage.kt:1252`), a bare 24 dp glyph centered in a 48
  dp slot (`:1272`), no container, no pressed shape (rectangular ring at `:1253`). The title is the same type role
  as the stock title;
  the second line box measures 49 px against 53 px, so the type scale is not a difference.
- Touches:
  A2 and D109 for cards and fills (ruled out);
  D58 and D77 for the slot and the target (must stay);
  D61 and D52 for section labels (ruled out);
  D117 and D120 for the ring.
- Proposal, inside the decisions: keep rows flat on the canvas;
  put the glyph in the 40 dp circle of the secondary container role that D109 already uses for Settings rows,
  centered in the same 48 dp slot (so D58 holds);
  give the pressed, hovered and keyboard-focused state the rounded shape D109 already defined
  (`SETTINGS_SELECTED_RADIUS_DP`, 21.3 dp), with the ring following that shape under D117 and D120.
- Token:
  `secondaryContainer` and `onSecondaryContainer` (already derived under D108), no new role.
- Cost:
  `searchResultRow` plus the matching rows in `search.slint`;
  `SearchPageLayoutTest` covers keys and highlight, not geometry, so geometry needs a new test.
  Risk: the circle must not move the title start (D58);
  verify at 200 percent text.
- Needs ruling: only if grouped cards are wanted (R1).
- Captures: stock `c4-lan-light.png`, `s3-wifi-light.png` and `s5-dark-q-light.png` against ours `a3-lan-light.png`
  and `i4-ours-inner-lan-light.png`.

### Match highlight

- Stock: none;
  "lan" does not emphasize "Languages", and "wifi" matches "Wi-Fi" with no emphasis.
- Ours: bold plus an accent fill with square corners (`SearchPage.kt:856`), which reads as a hard rectangle behind
  letters.
- Touches:
  D59 requires emphasis in place with a second cue.
- Proposal: keep both cues and the OKLCH role.
  The span background cannot take a corner radius in Compose text, so a rounded chip would need a custom draw pass;
  that is optional polish and is not proposed unless the human wants it.
- Cost: none for keeping;
  a custom text draw for chips would be moderate and is not recommended.

### Empty state with no query

- Stock: a scalloped (Expressive) tonal shape of about 139 dp (339 px in `s2`,
  measured) holding a primary search glyph, and the line "No recent search results";
  with a recent result it shows a letter-spaced "Recent search results" label, a "Clear" text button and the recent
  row.
- Ours: a 24 dp glyph, a SemiBold `titleLarge` "Search your music" and a body line (`SearchPage.kt:1128` to
  `:1157`). On the cover the block is centered in the whole area under the header,
  not in the area the keyboard leaves free: the status column has no IME inset while the result list does (`:961`),
  so with the keyboard open it sits at 53 to 61 percent of the screen height and its last line ends about 100 px
  short of the keyboard.
  Whether it stays clear at 200 percent text was not tested.
- Touches:
  D70 (keep the prompt and copy), D66 (empty on each visit), D69.
- Proposal: pad the status block by the IME inset and anchor it in the upper half;
  put the glyph in a 72 dp circle of the secondary container role (a plain circle,
  not the Expressive scalloped shape);
  lighten the title to `titleMedium` at weight 500.
  Do not add recents without a ruling (R4).
- Cost: a few lines per panel plus Slint.
  Risk: low;
  the D70 copy is unchanged.
- Captures: stock `c2-empty-kbd-light.png`, `s2-empty-light.png` and `s6-recents-light.png` against ours
  `a1-empty-kbd-light.png`.

### No-result state

- Stock: a primary-filled help glyph, "Try searching in Tips & Support" with the link in the accent color,
  and a small gray "No results for zzqqx".
  Results also end with a footer link, `Search Support for "lan"`.
- Ours: the same block as the empty state with the title "No results for “zzq”" and "Try another name."
  (D70).
- Touches:
  D70 and D71 (no promise of recovery without an owner).
  The stock "Tips & Support" link promises a recovery path that this app has no owner for, so it is not proposed.
- Proposal: same container treatment as the empty state;
  keep the copy.
- Captures: stock `s4-noresult-light.png` against ours `a4-nomatch-light.png`.

### Field and focus

- Stock: the placeholder "Search settings" and the text sit in a toolbar with no field container;
  Back is 48 dp, Clear is 56 dp (137 px);
  the caret is the accent color.
- Ours:
  `BasicTextField` in the header, caret `primary`, Back and Clear 48 dp (`SearchPage.kt:1075` to `:1115`);
  identical anatomy.
- Proposal: no change to the field.
  `SearchBarDefaults.InputField` would add a 56 dp field with its own container and icon padding and would have to
  be checked against D58 alignment;
  it brings no visible gain over what we have, so it is only a ranked alternative.

### Ring and keyboard focus

- Ours (captured):
  Tab moves a rectangular two-ring box inside the row edges, full row width, square corners (`f1-ours-focus.png`),
  as D120 specifies for clipped rows.
- Stock: not captured (see the unverified list);
  this item has no capture pair, only `f1-ours-focus.png`.
- Proposal: if rows get a rounded interaction shape, the ring follows it (D117: the ring hugs the element).
  That is a shape argument, not a new ring.

### Dark scheme

- Stock dark: a blue-gray page (`#17191F`), cards `#292C34`, divider `#303238`;
  the home pill is filled `#000000`.
- Ours dark: true black canvas, `#1E1F25` header band, divider `#8E9099`, highlight a dark blue chip.
- Touches:
  B1, a standing rule: a tinted dark page is not proposed.
  What changes is only what option 1 allows: header equals canvas (the header surface item) and a softer divider.
- Captures: stock `d2-wifi-dark.png` (inner panel) against ours `a5-lan-dark.png` (cover);
  different panels.

### Motion

- Stock: the Material `SearchView` expands from the pill (library behavior, not recorded here).
- Ours: an instant page swap on route change.
- Not recorded, so not compared.
  A shared-element expansion from the player's Search button would be new design work and is not proposed.

### Entry point

- Stock: a full-width 56 dp pill on the Settings home page.
- Ours:
  D47 chose a Search icon button on the player, and A2 retains the baseline style.
  Not proposed.
- Captures: stock `c1-home-light.png`, `s1-home-light.png` and `d3-recents-dark.png` show only the stock pill (the
  last is the Settings home page in dark, not a recents state despite its name);
  ours has no pill to capture.

## Options and ranking

Option 1: restyle our own composables inside the accepted decisions.
Header equals canvas (or the band extended under the status bar), `outlineVariant` divider,
status block anchored clear of the IME in a tonal circle, 40 dp circle icons in result rows,
rounded pressed and focus shape, lighter state titles.

- Pros: closes every visible gap that decisions allow;
  no new color role, no new dependency;
  reuses D109 shapes and the existing roles;
  keeps D58, D59, D63 to D80 and the Rust and Kotlin fixture untouched;
  works on both Fold panels.
- Cons: the page still will not look like stock Settings (no cards, no tint);
  needs a Slint port for parity;
  a header fill change alters a D48 detail and needs ruling R2.

Option 2: change only the row layout (circle icons, rounded pressed and ring shape).

- Pros: smallest change;
  no ruling beyond the D117 shape;
  contained to `searchResultRow`.
- Cons: leaves the white strip, the gray band and the heavy divider, which are the most visible dated cues in the
  captures;
  the empty and no-result states stay low and heavy.

Option 3: put `SearchBarDefaults.InputField` into our own header.

- Pros: uses a maintained component and gets caret, clear and semantics behavior from the library.
- Cons: adds a 56 dp field with its own container inside a 72 dp header, which risks D58 alignment and D77 sizes;
  changes nothing visible that option 1 does not;
  cannot be mirrored in Slint, against A3 and A4.

Option 4: rebuild toward the stock Settings look (grouped rounded cards, tinted page, tonal empty shape,
section labels).

- Pros: the largest visual match to the reference.
- Cons: needs rulings against A2, D109, D52 and D61 (it keeps true black, with cards on black in dark);
  needs new surface roles in both `ThemeRoles.kt` and `theme_roles.rs` and a regenerated 52-root vector;
  the two panes and the crease logic of D51 would have to carry cards.

Option 5: adopt `ExpandedFullScreenSearchBar` wholesale.

- Pros: one maintained component with transitions and predictive back.
- Cons: it is a full-screen dialog window, so on the unfolded panel it covers the folder browser and the deck (D50,
  D51), it is a modal focus trap (D76), key events go to another window (host keyboard shortcuts and the D117 ring
  at risk), it cannot be mirrored in Slint (A3, A4, D49), and its contained variant is the Expressive style A2 rules
  out.

Ranking: 1 > 2 > 3 > 4 > 5, because:

- 1 over 2: the same code area, and 1 also removes the white strip and the heavy divider,
  the cues the captures show most;
  2 is a strict subset of 1.
- 2 over 3: 2 is smaller and risks nothing in D58, while 3 adds a component that changes no visible detail.
- 3 over 4: 3 risks two decisions it might still satisfy (D58 alignment and D77 sizes) and adds no role,
  while 4 needs rulings against A2, D109, D52 and D61 and adds roles to both platforms.
- 4 over 5: 4 can be built inside the existing composables and the D51 layout, while 5 breaks the deck rule on the
  inner panel.

If the human rules yes on R1 and R3 in the rulings section, option 4 becomes the top option and option 1 is its
first step, because every item of option 1 is also part of option 4.

## Rulings requested

Each is a conflict with an accepted decision, put as a question and not adopted.

1.  R1: Grouped rounded cards and a tinted page on Search, as stock Settings has them (A2, D109).
    D109 rejected tinted backgrounds for Settings;
    does that extend to Search?
2.  R2: May the Search header drop the `surfaceContainerHigh` band for the canvas color and the divider move to
    `outlineVariant` (D48 gives the anatomy and the divider but names the baseline token)?
3.  R3: Section labels grouping results by parent, as stock does (D52 and D61).
4.  R4: A recent results list on the empty page (a new feature next to D66 and D70).
    It needs storage design and a privacy decision, so it is not part of any option.
5.  R5: Parity on desktop (A3, A4, D49): the same changes in `ui/search.slint`, or Android first.

## Cost and parity

- Every Android change needs a matching Slint change (`ui/search.slint`: surface at line 68, divider at line 222,
  rows, states) and its Rust page model if copy moves.
  Option 1 touches no copy, so the Rust and Kotlin page models,
  `SearchPageModelTest` and `SearchIndexParityTest` are unaffected.
- No new color role is needed in options 1 to 3:
  `canvas`, `outlineVariant`, `secondaryContainer` and `onSecondaryContainer` exist in `RoleSet`. Any new role
  (option 4) needs `ThemeRoles.kt`, `theme_roles.rs` and the 52-root vector regenerated together.
- Key hints are desktop only;
  if circles widen rows on desktop, the hint badge slot must keep its place.
- Verification the build must carry: both panels, light and dark, 100 and 200 percent text, keyboard open,
  Tab focus, TalkBack names (D77) and the fold change, driven on a headful emulator.

## Mockups described per state

1.  Empty, keyboard open (cover): one tone from the top edge through the toolbar to the page;
    Back at the left, placeholder "Search music", a 1 dp soft divider;
    in the upper half a 72 dp circle in the secondary container color with a 32 dp search glyph,
    "Search your music" in 20 sp medium, "Type a name to explore your library." in body medium.
2.  Typing, results: the same toolbar with "lan" and Clear;
    rows flat on the canvas, each with a 40 dp tonal circle holding the music or folder glyph at the same center as
    Back;
    the title with its highlight, bold with the accent fill;
    the parent line "Aster Vale/Salt Garden" in body medium;
    the pressed or focused row takes a 21.3 dp rounded shape.
3.  No results: the same container as state 1 with a help glyph, the title "No results for “zzq”" and "Try another
    name."
4.  Dark: the page and toolbar pure black, glyph circles in the dark secondary container, divider a faint gray line.
5.  Unfolded inner panel: the same right half beside the browser and deck;
    the toolbar starts clear of the crease as now.

## Unverified

- Motion (opening, closing, predictive back);
  nothing was recorded.
- TalkBack, 200 percent text, floating Gboard, and a real split-keyboard pass on the new states.
- The stock screen's keyboard focus and ring;
  its Tab behavior was not captured.
- Our dark scheme on the inner panel (our dark capture is the cover, the stock dark capture is the inner panel).
- The font of the stock screen;
  it looks rounder than Roboto and was not read from source.
- Settings result geometry beyond pixel measurement (the layout source is closed);
  radius and card height are measured from captures, and the radius is estimated from the corner profile.
- Runtime behavior of `ExpandedFullScreenSearchBar` as a window;
  it is read from the signature and class references.
- Seen but not diagnosed: after folding to the cover and unfolding, the query "lan" survived and a second "lan"
  typed after Ctrl+F gave "lanlan" with the caret between them.
  D66 covers a new visit, not a posture change, so this is recorded as observed and not as a confirmed defect.
- One emulator launch only;
  no crash occurred.

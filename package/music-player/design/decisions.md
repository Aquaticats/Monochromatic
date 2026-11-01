# Settled decisions

## Project context (stated 2026-09-03)

**This is a product with users,
 not a personal tool.**
 The user:
 "this isn't a
one-person project.
 If it was just for me,
 I wouldn't have needed to support 3 desktop
platforms."
 Read every decision below in that light — the library,
 hardware and habits
behind them are a design target,
 not one person's setup.
 Practical consequences that
now apply to all future work:

- **Audience (stated 2026-09-03):**
   "People with big local music libraries."
   Big local
  libraries is the defining trait — not audiophiles,
   not DJs,
   not people who want a
  media-library app.
- **No "my library" reasoning.**
   ~1,000 folders is a representative scale,
   not the
  scale.
   Surfaces must hold up at 50 folders and at 20,000;
   a filter (D17) does,
   a
  list does not.
- **Folder names are not Latin-only.**
   The 27-cell A–Z + # rail assumes they are.
  Japanese,
   Cyrillic and Greek names all bucket into "#" today,
   which is wrong for
  users whose whole library is in one of them.
   Open question.
- **Single-theme,
   true-black (B1) was chosen for one person's OLED.**
   It may still be
  right,
   but it is now a product decision affecting people on LCD panels and people
  who need a light theme for contrast reasons.
   Flagged,
   not reopened.
- **Defaults matter more than affordances.**
   D11's three switches were sized for one
  person who knows what they do;
   each default is now a decision about strangers.
- **Accessibility is not optional.**
   Screen-reader labels,
   focus order and reduced
  motion have never been designed and are not covered by "MD3 gives it for free."

Every entry below was decided by the user,
 in most cases by choosing between built
candidates.
 Each has the decision,
 why it was chosen,
 what was rejected,
 and enough
implementation detail to rebuild it.
 **Do not re-open these without being asked.**

---

## A. Platform and design system

### A1. Material Design 3 is the shared spec, on both platforms
**Why.**
 Compose gives MD3 for free on Android — theme,
 motion,
 touch targets,
 dynamic
colour.
 Adopting anything else means hand-porting a system to Android as well as to
desktop.
 MD3 is also a complete system:
 every state,
 disabled variant and focus ring
is already specified,
 which an in-house system would have to invent.
**Rejected.**
 Adobe Spectrum 2 (deliberately supplies platform-specific variants — the
opposite of one identity — and is Adobe brand identity,
 not a neutral system);
Fluent 2;
 a "pro audio tool" aesthetic;
 a bespoke in-house system.
 The user’s own
argument killed the in-house option:
 every state would have to be designed and
maintained by hand.

### A2. MD3 baseline, not MD3 Expressive
**Why.**
 Expressive’s asymmetric radii,
 heavier type and larger controls spend
vertical space on personality that a dense track list cannot afford.
**Note.**
 Both are free in Compose;
 the other identities explored were not.

### A3. One visual identity across Android, macOS, Windows and Linux
No per-platform look.
 Both platforms follow the **OS accent colour** where one is
exposed (Android dynamic colour;
 desktop system accent) — the user pointed out that
desktops expose an accent too,
 so this is not an Android-only feature.

### A4. Desktop toolkit = Slint (settled 2026-09-03)
The hand-porting cost of MD3 components to Slint was accepted explicitly.
 Consequence:
every component used on desktop must be portable by hand — prefer the small set already
in use (outlined segmented button,
 icon button,
 slider,
 list row,
 snackbar/toast,
 menu)
over reaching for more of the MD3 catalogue.
 A3 (one identity) and C1 (48dp targets on
desktop) still hold,
 and the desktop build owns its own scrollbar (D22).

---

## B. Theme and colour

### B1. Theme follows the OS; dark is TRUE BLACK — standing rule (revised 2026-09-04)
The app follows the system light/dark setting.
 **The dark scheme's background is #000
and this is a standing rule,
 not a preference** — the user,
 when it was re-offered as a
question:
 "true black is a standing rule!"
 It is never to be re-asked.

**Where the bound MD3 design system conflicts,
 MD3 loses.**
 MD3's dark `surface` role
is neutral6 (#141218) with a five-step container ramp above it;
 this project overrides
those roles rather than the rule.
 Everything else in the dark scheme — the primary,
secondary-container,
 outline and on-surface roles — comes from MD3 unchanged.
 A light scheme is therefore now required — MD3 supplies one,
but this project has never drawn a single light surface,
 so the light side of every
component (outlines carrying separation on black do not translate) is unbuilt work.
The original entry,
 which was argued from one person's OLED,
 follows:

### B1 (superseded). A single theme: MD3 dark, true black
**Why.**
 OLED.
 The user raised true black as a constraint that had not been asked
about.
 There is **no light theme** — one theme only.

### B2. True-black surface ladder
MD3’s dark surface levels are remapped so the base is pure black,
 and **outlines carry
the separation that lightness normally provides**:
```text
surface            #000000
surface container  #0A0A0D   (bars, grouped cards)
                   #121216   (hairline dividers inside groups)
container high     #1A1A1F   (icon buttons, menus)
container highest  #22222A   (inactive slider track)
```

### B3. Palette in use
```text
on-surface              #E6E0E9   primary text
on-surface-variant      #CAC4D0   secondary text, icons
outline                 #938F99   1px outlines (verified; older candidates use #6F6A78)
divider (strong)        #26262E
divider (inside group)  #121216
primary                 #D0BCFF   play button fill, active slider track, accents
on-primary              #381E72   icon inside the play button
secondary-container     #4A4458   selected segment / selected mode
on-secondary-container  #E8DEF8   text on the selected segment (verified; older candidates use #EADDFF)
row selected            #332F3F   the currently playing row
error                   #F2B8B5   error text/icons (verified; older candidates use #FFB4AB)
error dim               #F2B8B5   error supporting text
error surface           #2A1416   error row/bar background
error surface line      #4E2A2C
muted                   #8E8894   annotations (6.2:1 on black — passes AA)
disabled                #5A5561   track numbers, inert glyphs
```
**Accuracy caveat.**
 These are MD3 baseline dark values as applied in the candidates.
They resolve in the spec through `md-ref-palette`,
 which has not been read directly.
One likely correction:
 MD3’s `on-secondary-container` in the baseline dark scheme is
**#E8DEF8** (secondary90);
 **#EADDFF** is primary90.
 Verify against the palette file
before treating these hexes as canonical.
 Also note that with dynamic/OS accent colour
(A3) these hexes are only the fallback scheme.

### B4. Contrast rule
Annotation and secondary text must pass AA on pure black.
 #8E8894 was chosen for this
reason after a contrast failure was flagged;
 do not go lighter-dark than it for text.

---

## C. Density, targets and type

### C1. 48dp minimum touch target everywhere, including desktop
**Why.**
 It is a hard Android constraint,
 and parity is the goal.
 The user accepted
the desktop density cost explicitly.

### C2. Track rows are two-line, 72dp
Per the MD3 list spec (one-line 56,
 two-line 72,
 three-line 88).
 Line one is the track
title;
 line two is `duration · true-peak` (e.g. `4:35 · −1.2 dBTP`),
 tabular numerals.
Note added 2026-10-06:
 line two is now what the supporting-line template yields (D81),
 and its default is `$tf(mi(len), m:ss)$ $mi(peak)$`,
 which reads `4:35 −1.2 dBTP` without the dot (D93).
**Rejected.**
 Single-line rows (rows-a) and a columnar table layout (rows-b).

### C3. One spacing scale for both platforms
No separate dense desktop scale.

### C4. Type
Roboto 400/500.
 Sizes follow MD3 typescale roles (see md3-tokens.md):
 body-large 16px
for row titles,
 body-medium 14px,
 body-small 12px for the supporting line,
label-large 14px/500 for buttons and segments.
 Annotation labels use 11–12px with
wide letter-spacing in a monospace face — but note that in the candidates,
 monospace
is used only for **demo annotations**,
 not for product UI.

---

## D. Components and behaviour

### D1. Mode control = adaptive outlined segmented button (revised 2026-09-08)
Keep every option as a connected Material-style radio segment with an outlined container,
48dp minimum target,
 selected fill,
 checkmark,
 radio role,
 and state behavior.
 Use
the fewest connected rows whose complete labels fit:
 one horizontal row first,
 a
connected 2×2 block second,
 then four connected vertical rows.
 One-row cells use
variable intrinsic widths because Compose `SegmentedButton` forces equal weights;
 the 2×2
and vertical fallbacks retain real `SegmentedButton` elements.
 Only outside corners round
in multi-row arrangements;
 internal edges remain shared.
 Never add horizontal scrolling
or replace the control with plain radio rows or a single chip.

Visible labels are `Repeat`,
 `In order`,
 `Shuffle <currentSubDir>`,
 and
`Shuffle all`.
 Replace `<currentSubDir>` with the active subdirectory name;
 the current
prototype therefore says `Shuffle Camellia`.
 Cap one-row subdirectory content at the
rendered width of `Camellia`.
 Longer subdirectory names use a middle ellipsis in the
name only;
 the `Shuffle` prefix remains complete.
 Accessibility descriptions retain the full
subdirectory name and expand only where that adds meaning:
 `Repeat track`,
 `Play in order`,
 `Shuffle Camellia`,
 and
`Shuffle all folders`.

**Why.**
 The user explicitly replaced the fixed one-row-to-four-row transition with
one-to-two-to-four adaptive wrapping,
 required the current subdirectory in the Shuffle
label,
 and accepted the final strict-12dp scale review on 2026-09-08.
 The connected-group note below is history.
 The archive source is
`components/segmented-buttons/` under the user-supplied `m3.material.io` archive;
this multi-row arrangement is an explicit product decision rather than baseline Material
wrapping behavior.
**Verified.**
 Installed Android Settings exposes 85%,
 100%,
 115%,
 130%,
 150%,
180%,
 and 200%.
 Native captures use one content-sized row at 85%,
 2×2 at 100%,
 115%,
 and
130%,
 then four rows at 150%,
 180%,
 and 200%.
 Every segment keeps at least
12dp horizontal content padding;
 the control adds rows instead of crossing that floor.
All targets retain a 48dp minimum.
 A long-name probe renders `Shuffle Extr…ory` for
`ExtraordinarilyLongDirectory` while exposing the unshortened name in semantics.

### D1 (superseded). Mode control = connected button group (candidate mode-d)
Four options:
 **Repeat / In order / Shuffle folder / Shuffle all**.
**Why this component.**
 M3 marks the segmented button as no longer recommended and
directs you to the connected button group.
 Verified token values:
```text
container-height        56px   (medium size)
between-space            2px
container shape          corner-full  (pill outer ends)
inner-corner             8px   (corner-small)
selected inner-corner    50%
pressed inner-corner     4px   (corner-extra-small)
```
It **wraps to a grid on narrow screens**,
 and when it wraps only the four outer corners
round — the block keeps flush sides.
 This wrapping behaviour was specified by the user
via uploads/segmented buttons.png.
**Rejected.**
 Detached pills (mode-a/mode-b:
 read as four unrelated buttons);
the outlined segmented button (mode-c) — kept as the fallback if the connected group
proves impractical,
 and it is the one that matches the user’s reference image exactly.

### D2. The mode control also defines end-of-folder behaviour
**In order** = play to the end of the folder and stop.
 **Shuffle all** = cross folder
boundaries.
 **Repeat** = repeat.
 There is no separate end-of-folder setting and no
"continue into the next folder?"
 prompt.
 Candidates o6-a and o6-b explored this as a
separate question and are moot.

### D3. Folder picker = A–Z jump strip over a flat name-sorted grid
~1,000 artist folders,
 flat (no nesting in the picker),
 sorted by name.
 A 27-cell
strip (A–Z plus #) jumps within it.
 Cells and folder rows are **wrapped 48dp targets**,
not a single narrow column.
**Rejected.**
 A thin iOS-style fast-scroller (too small for the 48dp rule);
 letter
section headers alone (picker-a);
 a persistent sidebar (picker-c).
 Note the Android
fast-scroller drag-bubble idiom was discussed as a possible addition — a large bubble
showing the current letter while dragging — but was not settled.

### D4. Open shares the Folders app-bar line (revised 2026-09-04)
`Open` changes the directory.
 It is a visible Material text-button action with the
folder-open icon on the left pane's `Folders` app bar.
 There is no separate
current-folder control on the left.
 The right-pane `Camellia` app-bar title and the
selected folder target already communicate current-folder identity.
**Why.**
 Session restore makes Open a mid-session action,
 so it does not need a
separate high-emphasis row.
 Removing the duplicate current-folder control gives the
picker more vertical space while Open remains one tap away.
**Rejected.**
 Open in the overflow menu next to Settings (o1-b);
 a lone current-folder
chip;
 a second `Camellia` button above the picker.

### D5. Subfolders are headers inside one flat list — not a layer (confirmed on desc-d, 2026-09-03)
A folder containing subfolders shows **one flat list**:
 its own tracks first,
 then each
subfolder as a **header row** (name + count) followed by that subfolder's tracks.
Subfolders "don't create another layer of interaction" (user's words) — nothing to
enter or leave,
 no breadcrumb,
 no Up.
 Candidate desc-d shows it.
**Rejected.**
 Drill-in navigation with breadcrumb and Up (sub-b,
 desc-a);
 subfolders
as tappable rows above the tracks (sub-a as first read,
 desc-b,
 desc-c).

### D6. In order plays the parent's tracks first, then subfolders in name order
Confirmed 2026-09-03 ("Tracks first",
 not interleaved by name).
 Because of D5 the
"does the list follow?"
 question dissolves:
 playback walks down the flat list and the
highlight follows.
 Deck subtitle counts across the whole folder ("6 of 10").

### D7. Track context menu = candidate ctx-b
Eight items in three groups,
 headed by the track name:
```text
Play
Start shuffle from here
—
File details            (with the dB value shown inline, e.g. −0.8 dBTP)
Re-analyse true peak
—
Show in file manager
Copy filename
Move to trash           (destructive, error colour)
```
**Rejected.**
 A four-item minimal menu (ctx-a).

### D8. Move to trash = delete immediately, offer Undo (revised 2026-09-03)
No confirmation dialog.
 Undo is a **compact toast,
 not a full-width snackbar** — the
Todoist model:
 a small dark pill floating over the list near the bottom edge,
 content
width,
 "Moved to trash · Undo",
 auto-dismissing after a few seconds.
 It does not span
the window and does not push anything.
 Collision with the error bar is now settled by D29.

### D9. Missing files and renamed folders = candidate err-b

The layout-reserving error-bar presentation is superseded by D83.
Vanished-row removal,
count-collapsed failures and renamed-folder behavior remain applicable.
Historical accepted treatment:

The list stays clean:
 a file that has vanished **drops out of the list**,
 and a
dismissible bar explains what happened.
 When several files fail at once the bar
**collapses to a count** rather than stacking one bar per file.
 A folder renamed while
playing changes nothing visible — the open file keeps playing and the folder list
refreshes on the next read.
**Rejected.**
 Errors in place (err-a),
 where the dead row kept its position,
 turned
error-coloured and explained itself,
 and the folder chip carried its own failure state.

### D10. Empty state = candidate empty-a
When nothing is open,
 the screen explains the **first-run analysis up front** — an
hour at high CPU,
 once,
 pausable,
 playback works throughout — alongside a primary
"Open a folder" button and Settings.
**Why.**
 There is room here and nothing to interrupt;
 explaining it later means
explaining it during.
**Rejected.**
 A bare empty state with one line in the bottom bar and detail behind a
"Why?"
 chip (empty-b).

### D11. Settings = candidate settings-a
Three flat switch rows,
 and the pane says out loud that it is short:
```text
Strip common prefixes from filenames      ON
Resume where I left off                   ON
Analyse true peak in the background       OFF in the mock
```
"Strip common prefixes" shows `Another Xronixle` instead of
`かめりあ(Camellia) - Another Xronixle.flac` and is **on by default**.
"Resume where I left off" restores folder,
 track and position,
 **paused**.
"Analyse in the background" off means each track is measured just before it plays.
**Rejected.**
 Grouped cards with section headers and an analysis-status row
(settings-b).

D84 (2026-10-05) removes the third row:
analysis is automatic and non-optional,
so Settings has no analysis switch.
D85 removes the closing sentence that says the pane is short.
D86 (2026-10-05) removes the two remaining rows:
both behaviours are always on and are not settings.
Nothing of settings-a's content is left.

### D12. Analysis status lives nowhere after the first run
The user chose this explicitly:
 once analysis is done it never needs to be seen again.
Consequence:
 the "10,412 of 10,412 analysed" row from settings-b is gone,
 and
**Re-analyse on the track context menu is the only remaining entry point** (removed by D113, 2026-10-12).

### D13. The scan indicator is not permanent
It appears while analysing and leaves afterwards.
 Final visual form unsettled
(scan-b and scan-d each show half of the intended behaviour).
 One known defect worth
remembering:
 in scan-a the Pause button appears and disappears,
 so that variant is not
reflow-free;
 scan-b was the only one that never changes size.

---

### D16. Unfolded screen: picker + transport LEFT, tracks RIGHT (candidate unf-g → unf-h)
Chosen over transport-right (unf-f).
 unf-h is the assembled version with D1 applied.

### D17. Folder picker = filter, never a list (candidate unf-f/unf-g)
One-column letter rail (27 × 48dp,
 scrolls on its own — user asked for one column).
The rule the picker must obey:
 **cope with 1k folders,
 never list 1k rows**
(review-notes 5b).

**Two hard NOs inside this,
 both stated by the user and both violated since — read them
before touching the picker:**
1. **No sub-letter segmentation,
    in any form.**
    The CA / CH / CL–CO split is dead —
   "we don't really need this as tabs" killed the *segmentation*,
    not just its
   presentation as tabs.
    A rail accordion (pk-d) is the same idea wearing a different
   hat and is equally out.
2. **No chip STYLING on the names** — but the wrapped,
    several-per-line LAYOUT is
   fine and in fact required.
    "Chips are out,
    but only the chip-like styling.
    You can
   just … not style them like chips."
    So:
    no pill fill,
    no radius,
    no outline around a
   name;
    names are plain text at their natural width,
    several per line.
3. **One item per row is out** — in one column (pk-a,
    pk-c),
    in two (pk-b) or in three
   (pk-e,
    pk-f).
    It has been rejected at least four times:
    a letter holding 70–100
   names makes the scroll absurdly long,
    which is the original failure
   (review-notes 5b).
    Several names per line is what keeps the extent short.

Settled presentation:
 **D31**.

### D18. Transport block layout (revised 2026-09-04; volume row removed by D20)
Centred on the half's axis:
 title and subtitle centred;
 seek is a full-width Material
slider row with elapsed and duration anchors;
 previous,
 pause,
 and next are centred
Material icon buttons with the recommended 8dp target spacing.
 The one-row mode control hugs its
content instead of stretching edge to edge.
 The earlier left-hugging version was
called "un-balanced."
 The earlier 24dp button gaps and full-width segmented control
were replaced by the supplied Material component and target-spacing guidance.

### D14. Cover screen = full player (candidate cover-c); volume kept until D43 removed it
Chip + Open on top,
 track list,
 deck at the bottom for thumb reach,
 volume inline,
mode group wrapped 2×2.
 **Rejected.**
 Controls-only with the list behind a row
(cover-d).

### D15. The A–Z jump strip must NOT be a grid of boxed keys
The 9×3 grid of rounded cells (unf-a/unf-b/unf-c) "looks like a keyboard,
 which
doesn't fly."
The historical rail-with-bubble (`unf-d`) versus borderless-index (`unf-e`)
comparison is not open:
D17 rejects their one-folder-per-row list model and selects the filtering
picker,
with D31's wrapped plain names.
Retain these files as historical rejection evidence,
not candidate instructions for a new fast-scroller round.

## E. Foldable

### E1. Target device: Pixel 9 Pro Fold
See device-metrics.md for real dimensions.
 The hinge is **vertical** in portrait.

### E2. Keep information off the fold connector (clarified 2026-09-23)
The user corrected their initial 10mm estimate to a **visible crease width
of about 7.5mm for this design**.
 At the panel's approximately 141.08mm
active width,
 this is about **110 physical px** of the 2076px inner display,
centered at x 1038:
 approximately x `[983,1093)`px.
 These are approximate
physical bounds because the published 8-inch diagonal and the user's dent
width are approximate.

Where information is arranged on opposing sides of the fold,
 its clearance
is **`max(min_padding, crease_width)`** after both terms are expressed in the
same physical coordinate system.
The user reconfirmed **`min_padding = 7.5mm` as a minimum total gap between
opposing information** on 2026-09-26,
including on future devices whose physical crease is narrower than 7.5mm.
For a wider crease,
the physical crease width still governs.
This is one total left-to-right information gap,
not 7.5mm on each side and **not** a required empty surface gap between panes.
 Text,
 labels and other informative marks stay out;
backgrounds,
 borders,
 padding,
 field/row containers and hit regions may
span the center.
 The former fixed 24dp player spacer and fixed 414dp
pane geometry do not satisfy this rule.
 D34 white and D41 black remain
accepted player color treatments where those surfaces are used,
 not a
mandated 24dp blank stripe.

Do **not** store `crease_width` as a fixed dp value:
 dp varies with Android
display scaling while the physical dent does not.
 The current AVD density
of 390dpi would convert about 110px to about 45dp,
 but that is only a
runtime conversion for this setting,
 not the design constant.
 The AVD's
zero-width hinge sensor area describes emulated occlusion,
 not the user's
visible 7.5mm dent.
 No other page is required to become two panes.

Here "content" means **information the user must perceive**:
 keep readable
text,
 result data,
 labels and other meaning-bearing marks visibly clear of the
crease itself.
 A continuous page or input surface,
 row background,
 divider
or interaction region may cross the centre.
 A hit region crossing it does
not by itself violate E2.
 Do not infer a fixed 24dp text exclusion from
the player's spacer,
 force every surface to its black/white colors,
 or
strand a Search query and results in different halves to clear the crease.
Judge information placement against the physical crease band in native
panel-pixel captures,
 then convert only the current layout's coordinates to
dp for Compose.
 I
initially read the user's YouTube timestamp/title observation as praise;
that was wrong.
 The captured characters approach the center closely enough
to fall within the approximate 7.5mm band,
 so YouTube is a **negative
near-crease example** for this player's text placement.
 Android's system-owned bars are outside
this app-content requirement.
 The earlier "no app-owned paint or hit region"
version of E2 was an erroneous interpretation and is withdrawn.

The numeric floor was selected only after a debug-only Compose comparison
showed the player with Search closed,
empty Search and results for P7.5,
P14 and P20 on the disposable Pixel 9 Pro Fold.
The earlier measurement-bar form,
Search-only comparison and gray-strip player prototype were rejected as
incomplete or as applying the floor to surfaces.
P14 and P20 were not selected as hard floors;
the decision does not prohibit naturally wider information spacing.
The accepted player browser,
full deck,
right-side Search layout and track-row surfaces remain structurally intact.
The captured 200% keyboard-closed node-box projections support this visual
choice but do not certify every glyph's ink,
long names,
other display settings or keyboard-open fit.
The evidence is in `package/music-player/design/evidence/crease-floor-native-comparison.md`.
No production Search implementation was authorized.

### E3. Tabletop posture = candidate tabletop-c
The user’s own proposal,
 and better than either option offered:
 the **track list stays
continuous** and only the row that would fall in the crease moves out of it.
**Platform note.**
 Compose can detect tabletop posture;
 desktop toolkits cannot — so
this behaviour is Android-only by nature.
**Rejected.**
 Splitting the screen into list-above / controls-below at the hinge
(tabletop-a),
 and a posture-agnostic layout (tabletop-b).

---

## F. Input

### F1. Keyboard map — DRAFT ONLY (candidate keys-a), superseded by D102
Not reviewed by the user; the map the human accepted in D102 replaces it.
 Current draft:
```text
Space        play / pause
Ctrl ←/→     previous / next track
←/→          seek ±5s
↑/↓          volume
Ctrl F       RESERVED for a future search (D25)
Ctrl O       open the folder picker (D25)
Ctrl L       jump to the playing track
Ctrl M       cycle end-of-track mode
any letter   type-to-jump in the folder grid
Ctrl ,       settings
```
Media keys are expected to work on both platforms (MPRIS on Linux,
 media session on
Android).
 D82 accepts Android's standard media notification presentation,
 with no custom notification design round.

---

## G. Round 4 decisions (2026-09-03)

### D19. No album art, permanently
Art-free utility is now an explicit decision,
 not an accident of the work so far.
 No
artwork in the deck,
 in track rows,
 on the cover screen,
 or in the picker.
 The fork
between "art-free utility" and "art-led moody" is closed in favour of the former,
 so
row treatment,
 deck and visual system stay as built.
 Do not propose artwork again.

### D20. Volume = icon button + vertical popover slider, everywhere — superseded in-app by D43
The YouTube model,
 chosen over the inline full-width slider row.
 Spec:
 a 48dp icon
button in the transport block;
 tapping/clicking it opens a vertical slider popover
anchored to the button (MD3 menu surface — `surface container high` #1A1A1F,
 elevation
level 2,
 corner-extra-small container,
 44px slider handle on a 16px track per
md3-tokens.md).
 Same on Android and desktop;
 on desktop the popover also takes ↑/↓ and
scroll.
 **Revises D18** — the transport block loses its volume row,
 which frees a full
row of vertical space in both the unfolded left half and the cover screen.
 **Revises
D14** — the cover screen keeps volume,
 but as the icon,
 not the inline slider.

### D21. Command bar: global hotkey, configurable, off by default
A command bar exists.
 Its hotkey can be registered globally (works when the app is not
focused),
 that setting is **off by default**,
 and it is configurable.
 Consequences to
design:
 a fourth Settings row (D11 said the pane is deliberately short — it stays short
at four),
 plus the in-app binding,
 which must not collide with F1's map.
 Global hotkey
registration differs per platform (Android has no equivalent;
 on Linux/Wayland it is
compositor-dependent) — check feasibility per platform before drawing the setting.

### D22. Scrollbars: desktop bar always visible; the letter rail has none
Touch input keeps Material's non-interactive fading 4dp bar.
 Pointer input gets the
same bar **always visible and draggable**,
 with a 12dp grab zone — it does not fade.
Chosen by last input,
 never the OS-native scrollbar.
 The letter rail (D17) shows **no
scrollbar at all** on either input,
 because its 27 cells are self-describing.
 Closes
the two open sub-points on the scroll-a spec.

Amended by D125 (2026-10-14): the desktop folder names list shows no scrollbar on either input either.
The track list, Search, Settings and the template editor keep the bars described here.

### D23. The picker shows no folder counts
No library total ("1,043 folders") and no per-letter count.
 The picker is a filter
(D17),
 and a count is a list affordance — it implies a length the user is not meant to
care about.
 Removes the header count from unf-h and the count beside the letter.

### D24. Track order within a folder = tag track number, filename as fallback
Where files carry a track-number tag,
 that is the order and that is the number shown in
the row.
 Where the tag is missing,
 the folder falls back to filename order.
 Mixed
folders sort tagged tracks first by number,
 then untagged by filename.
 Folder order
everywhere else stays name order.

### D25. Ctrl+F is reserved for a future search; Ctrl+O opens the folder picker
Reverses F1's draft,
 which had Ctrl+F opening the picker on the grounds that there is
no search.
 Search is now expected to arrive later and keeps its idiomatic binding.
Ctrl+O — previously "open a directory" — opens the **picker**;
 the map is to be aligned
with IntelliJ IDEA conventions generally,
 which needs one pass over the whole list
(open-questions #6) before F1 comes off draft.
 Neither J/K nor Ctrl+arrows was ticked,
so next/previous stays as drafted (Ctrl+arrows) pending that pass.

### D26. Scan indicator = scan-F (candidate scan-ef, right frame)
A 56dp bar at the bottom edge:
 "Analysing true peak · 412 of 1,218" plus a
**fixed-width (100dp) Pause button that is always rendered**,
 so nothing appears,
disappears or reflows mid-scan — the defect that killed scan-a.
 The bar itself appears
when analysis starts and leaves when it finishes (D13).
 **Rejected.**
 scan-E,
 a 2dp
determinate line on the bottom edge with no text or control (too little for an
hour-long operation);
 scan-b;
 scan-d.

### D27. First run opens the system music library and ASKS before analysing
If the platform exposes a designated music library — Android MediaStore,
 XDG
`XDG_MUSIC_DIR` on Linux,
 Music on macOS/Windows — the app **opens it automatically on
first launch**;
 no empty state,
 no "Open a folder" step.
 Analysis does **not** start on
its own.
 The bottom edge asks once,
 with four answers,
 all 48dp:
```text
Scan once        analyse this library now, ask again next time
Always scan      analyse now and whenever a new library appears — no more asking
Dismiss once     don't analyse; ask again next launch
Dismiss forever  never ask; Re-analyse on the track menu is the only entry point (D12)
```
Candidate first-run-a shows all four states.
 Until a library is analysed,
 rows show
duration only and the deck subtitle omits the dB value — the absence is not an error
and is never explained in place.
 Note this pushes against D10's empty state,
 which now
only applies when no system library exists or the user declined it.

D84 (2026-10-05) supersedes the ask-before-analysing prompt and its four
answers:
true-peak analysis is automatic and non-optional.
Automatic opening of the system music library stands.

### D28. The picker rail adapts to the library's writing systems (revises D3, D17)
**Only writing systems present in the library get a section in the rail:**
 Latin A–Z,
Japanese kana rows (+ 漢 for kanji),
 Cyrillic,
 Greek,
 Hangul,
 then #.
 A Latin-only
library shows no other section;
 a Japanese-only library opens on kana.
 Sections are
separated by a hairline in the one-column rail.
 This replaces the single "#" bucket
that every non-Latin name used to fall into.
 `candidates/buckets.js` holds the model.

**This decision covers the rail only.**
 Presentation of the names is D31.

### D29. Undo toast floats above the error bar, both visible (candidate toast-a)

D83 supersedes the error-bar owner and its layout reservation.
The remaining overlay requirements still apply:
content width,
left alignment,
16dp owner separation and no player-layout push.
Historical collision treatment:

Closes the collision.
 The toast is content-width,
 left-aligned,
 48dp,
 over the list,
16dp above whatever owns the bottom edge:
 it lifts when the error bar (D9) is up and
drops when the bar is dismissed.
 It never pushes layout and never spans the window.

### D30. Design for ~1,000 folders; small and huge libraries degrade gracefully
~1k is the design scale.
 A 30-folder library and a 20,000-folder library must not
break,
 but neither is drawn,
 and neither gets special-cased UI for now.

### D31. Picker names = plain text, several per line — CHOSEN (candidate pk-g, 2026-09-03)
Chosen over pk-h (14px with middot separators,
 2.0 screens):
 the 16px/24dp-gap version
reads as a set of targets rather than as running text,
 and the density gain was not
worth it.
 Selection marking uses **primary colour + a 2dp MD1-style indicator at the
bottom edge of the whole 48dp target**,
 never underlined label text and never a filled
pill.
 The indicator is spatial state chrome,
 not link decoration.
 The accepted extent
is "2 to 2.5 screens per letter is not absurd",
 which finally closes the scroll-length
constraint that ran through six
rejected candidates.
The presentation that satisfies every constraint in D17 at once:
```text
16px Roboto 400, on-surface #E6E0E9      plain text — no fill, no radius, no outline
several names per line, natural width    wrapping flex row, column-gap 24dp, row-gap 0
48dp minimum height per name             C1 target rule, met without a pill
nothing truncated                        names take the width they need
selected: primary + 500 + 2dp target-width indicator
          never text underline or filled pill
```
This is the unf-f/g/h/i layout with the chip styling removed — the layout was never the
problem.
 **Measured** extent at 418dp (not estimated — review-notes #1):
 names pack
about 2.1 per line,
 so C (72 names) is 35 lines / 2.2 screens and the worst letter,
 S
(90 names),
 is 41 lines / 2.5 screens — against 4+ screens for one name per row.
 pk-g
reads these off the DOM and displays them,
 so the claim can be re-checked at any width.
**Rejected on the way here:**
 wrapped chips (unf-f/g/h/i — the styling);
 one name per
row in one,
 two or three columns (pk-a,
 pk-b,
 pk-c,
 pk-e,
 pk-f);
 a rail accordion that
re-introduced prefix ranges (pk-d).

### D32. The dark ramp is the project's own, measured down from black (candidate dark-b)
Chosen over 2a (MD3's own dark containers sitting above the #000 window),
 because
MD3's steps are warmer and start higher — the panes read as grey cards floating on
black,
 which defeats the point of true black.
 The ramp,
 in place of MD3's dark
`surface-container-*` roles:
```text
window / lowest      #000000     MD3 would be #0F0D13
low                  #0A0A0D     MD3 #1D1B20
container            #121216     MD3 #211F26
high                 #1A1A1F     MD3 #2B2930
highest              #22222A     MD3 #36343B
```
Five token values now diverge from the bound design system and are this project's to
maintain;
 everything else in the dark scheme (primary,
 secondary-container,
 outline,
on-surface) stays MD3's.
 This is the concrete form of B1's "where MD3 and true black
disagree,
 the surface roles get overridden,
 not the rule".

### D33. Only the surviving candidates get rebuilt on the design-system bundle
unf-j,
 pk-g,
 first-run-a,
 toast-a and scan-ef,
 plus everything built from now on.
 The
other ~60 candidates keep their hand-inlined values as history and are not touched.
Every rebuilt file **pins its scheme inline on its own root** — the host sets
`data-theme="dark"` on `<html>`,
 and inheriting that silently inverts a light design
(review-notes 5g).

### D34. Light uses 1c with white pane spacers and a visible rail line
Use the `1c` tonal structure.
 The vertical spacer between panes and the
16dp horizontal divider between the folder picker and transport are white.
The original 24dp vertical width does not establish safe placement of
information across the user's 7.5mm crease.
 E2 applies
`max(min_padding, crease_width)` to informative material in physical space;
borders,
 backgrounds and padding may cross.
 D34 settles spacer color where
that surface is used,
 not a mandatory blank width.
 Keep the 1dp
letter-rail boundary in dynamic `outlineVariant`;
 keep pane and track-row outlines
absent.
 The user chose clarification option D2 and marked the horizontal divider in
`/var/home/user/Pictures/Screenshots/Screenshot_20260904_191909.png`.
**Why.**
 White removes the two prominent gray bars while the thin rail boundary still
separates the letter targets from folder names.
**Rejected.**
 `1a`,
 which uses another tonal hierarchy;
 `1b`,
 which outlines panes and
rows;
 a white rail line,
 which disappears against adjacent white surfaces;
 making all
three separators white.

### D35. Light supporting text uses one neutral role (candidate dbtp-a)
Use one `onSurfaceVariant` supporting line for both duration and true peak.
 Do not give
the true-peak substring a stronger role or move it to a trailing column by default.
**Why.**
 Custom display templating is planned,
 so users will be able to choose what the
row displays and emphasizes.
 The product default should remain neutral rather than
hard-code true-peak emphasis.
**Rejected.**
 `3b`,
 which strengthens true peak;
 `3c`,
 which creates a trailing
true-peak column.

### D36. Current track uses a soft neutral container and bold title
Do not prefix tracks with interface-generated numbers.
 Remove the play icon and its
reserved leading slot from every row so titles retain the full list width.
 Mark the
current row with `surfaceContainerLow` plus a bold title,
 and expose current state
semantically.
 This is matrix treatment T3.
**Why.**
 The user selected 3B.
 The neutral container plus weight differentiates current
state through more than one visible channel without reducing title width or restoring
the rejected saturated fill.
**Rejected.**
 Sequential UI ordinals;
 saturated `primaryContainer` current-row fill;
 play icon or reserved state column;
 color-only or shape-only treatment;
 T1,
 T2,
 and T4.

### D37. Transport uses tight spacing and outlined skip buttons
Use 8dp spacing between the existing playback groups and outlined styling for the
separate Previous and Next icon buttons.
 Pause remains filled.
 This is matrix choice 1B.
**Why.**
 The user explicitly chose 1B after reviewing all nine native candidates.
**Rejected.**
 The other eight spacing and skip-button combinations from the existing-screen
refinement matrix.

### D38. Open uses a tonal button
Use the Material tonal button treatment for the existing folder `Open` action.
 Retain
both the folder icon and explicit `Open` label.
 This is matrix treatment O2 and completes
selection 3B with D36.
**Why.**
 The user selected 3B.
 Tonal treatment increases prominence through container fill,
shape,
 icon,
 and label while remaining calmer than the filled primary action.
**Rejected.**
 Filled,
 elevated,
 and outlined Open treatments from this matrix.

### D39. Unfolded TalkBack traversal is pane by pane
Traverse the folder area first,
 the playback deck second,
 and the track pane last.
 Keep
the two left-pane regions contiguous before moving to the track pane.
 This is
accessibility treatment F1.
**Why.**
 The user selected F1.
 It is predictable from spatial layout and avoids jumping
between panes or placing playback before browsing context.
**Rejected.**
 F2 browse first,
 which separates the two right-pane regions;
 F3 playback first,
 which front-loads controls before browsing context.

### D40. Current track uses structured TalkBack state speech
Announce `Current track. Another Xronixle. 4:35 · −1.2 dBTP. Button.` through a
state description while retaining structured visible descendants.
 Do not repeat the title
or replace row semantics with one hand-composed sentence.
 This is accessibility treatment
S1.
**Why.**
 The user selected S1.
 It removes the baseline duplicate title while preserving
component structure and deriving speech from visible content.
**Rejected.**
 S2,
 `Selected. Current track: Another Xronixle, 4:35, −1.2 dBTP. Button.`,
because its single hand-composed label duplicates visible strings in code.

### D41. Dark structural surfaces remain stable black
The folder canvas and navigation rail stay true black,
 the playback deck stays the fixed
`#0A0A0D` neutral,
 and only component roles follow Android's generated dynamic color.
This is structural reach R1 from the native 2 × 3 dark matrix.
**Why.**
 The user selected R1.
 Structural surfaces then read identically under every wallpaper,
color style,
 and contrast setting while buttons,
 chips,
 seekbar,
 and text still honor
Android's resolved roles.
 The true-black spacer and track canvas keep the screen identity.
**Rejected.**
 R2 generated rail and deck,
 and R3 generated folder canvas,
 rail,
 and deck.
Both let uncontrolled wallpaper hue occupy structural surfaces and move the structural
contrast boundary with the palette.

### D42. The current row neutral follows Android surfaceContainerLow
The current track's soft neutral container resolves from Android's generated
`surfaceContainerLow` role per environment instead of a fixed hex,
 keeping D36's soft
container plus bold title with no icon,
 ordinal,
 or reserved column.
 This is current-row
source C2.
**Why.**
 The user selected C2,
 overriding the C1 > C2 ranking.
 The row then belongs to
Android's generated container ramp under every wallpaper,
 and the captured role evidence
shows its text pairs meeting the recorded contrast minima in all six environments.
Combined with D41,
 the accepted appearance is the captured `stable-dynamic` column:
fixed black structure carrying a generated current-row neutral.
**Rejected.**
 C1 fixed `#0A0A0D` current row,
 which stays palette-independent but detaches the
row from the generated container ramp its surrounding text roles follow.

### D43. No in-app volume control (2026-09-17)
The user removed the in-app volume control functionality.
 No volume icon,
 slider,
 or
popover remains on any in-app surface:
 unfolded deck,
 cover deck,
 or desktop pane.
D20's icon plus popover treatment and D14's cover volume icon are superseded.
 Volume
adjustment belongs to the system:
 hardware keys,
 the system media session,
 and whatever
Android's standard notification presentation.
D82 removes the custom media-notification design round;
functional media-session integration remains separate.
**Why.**
 The user's instruction of 2026-09-17.
 It removes a control the system already owns and
frees a deck element on both Android surfaces.
**Rejected.**
 Keeping D20's icon button and popover in-app.

### D44. The cover deck takes its content height (2026-09-17)
The cover deck carries no 440dp height cap:
 at 200% text the deck takes its content height and
the track list absorbs the remainder.
 The unfolded deck keeps its cap and internal scroll.
**Why.**
 User correction:
 the 200% cover capture clipped the segmented mode group's bottom
border mid-target.
 On the single-pane cover a nested deck scroll hides control state,
 while a shrinking list keeps every control visible.
**Rejected.**
 Keeping the 440dp cap on the cover,
 which clips controls at 200% text.

### D45. Cover light surface = flat with hairlines at both seams (L3, 2026-09-17)
The cover light scheme uses one flat `surfaceContainerLowest` surface with
`outlineVariant` hairlines at the top-row and deck seams.
**Why.**
 The user selected L3.
 L1's deck-seam hairline plus ramp can make a selected first
track read as detached from the list,
 because the current row's generated low container sits
between two separation cues.
 L3 keeps every row on one surface and separates structure only at
the seams.
**Rejected.**
 L1 ramp with a deck-seam hairline,
 and L2 pure ramp without hairlines.

### D46. Cover picker opened state = P4, temporary pre-1.x decision (2026-09-23)
D104 (2026-10-10) later adds the upward caret while open, drops the pre-1.x revisit and defers further improvement to
another major version.

Use P4 as the current design baseline for the folded cover screen:
 the app-bar folder title
and caret open the picker in the list slot,
 leaving the deck visible.
 This is explicitly a
**temporary pre-1.x decision**, not final acceptance of the picker interaction or a
production implementation authorization.
 The user chose P4 while believing a better
solution exists;
 keep a dedicated improvement question open before 1.x rather than
representing P4 as the ideal solution.

**Why.**
 P4 is the user's selected working variant from the P1 to P4 native comparison.
The form's P4 evidence covers dark at 100% text;
 its dark 200%, L3 light 100%, and L3
light 200% states were not captured.
 The dark 200% and L3 light 100% captures depict
P2; no L3 light 200% picker capture exists.
Check the selected variant in those states before claiming corresponding visual coverage;
validate focus and return behavior through native interaction, not screenshots alone.
 The comparison's
recommendation was P2;
 the user's P4 selection takes precedence.

**Post-decision visual check (2026-09-23).**
 Native folded captures now cover P4 in dark at 200% and L3 light at 100% and 200%
(`questions/render/cover-round-cover-picker-p4-s200.png` and
`cover-round-cover-picker-p4-light-s{100,200}.png`, with matching XML).
 All three are
opaque 1080 × 2424px cover rasters.
 The 200% mode group's final target ends at y=2326
of 2424, with its bottom border visible above the navigation bar.
 L3's top-row and
deck hairlines remain separate from the selected folder indicator.
 These are visual
checks of the opened state.
 A separate debug-only interactive P4 study on the folded
cover at 200% verified closed to open via the title, open to closed via the title and
Android Back, and dismissal by selecting the current folder.
 Keyboard activation and
Back preserved input focus on the trigger; touch dismissal left it unfocused in touch
mode.
 `questions/evidence/cover-round-cover-picker-p4-interaction.json` links the native
hierarchies.
 TalkBack accessibility focus was not measured.
 D46 remains provisional.

**Not selected for the temporary baseline.**
 P1 and P3 use a floating panel;
 P2 uses
the MD3 outlined text-field trigger with the in-slot picker.
 These are comparison
alternatives,
 not permanent prohibitions on exploring a better pre-1.x solution.

### D47. Search button opens a separate search page (2026-09-23)
The user rejected every I/G/R command-bar candidate and directed the design to a
**separate search page opened by a Search button**.
 Do not restyle a command palette
or offer a docked/floating command bar as another option in this round.
 Show the
button in the player context,
 then a coherent Search destination with its own
identity,
 query entry,
 results and Back path.
 "Separate page" names the
navigation/interaction destination,
 **not** a requirement to occupy the whole
unfolded display.
 The user explicitly allows a one-half Search screen as one
possible layout;
 other layouts remain open.
 This is a design-only direction,
not an authorization for production implementation.

**Supersession.**
 D21's user-facing command-bar surface is superseded by this page.
Its configurable global hotkey and extra Settings row applied to that command bar;
do not transfer them to Search or silently keep them as settled page requirements.
Whether a global invocation exists for the new page is open if the user raises it.
D25's reservation of Ctrl+F for future search and Ctrl+O for the folder picker remains,
but the full keyboard map is still a separate unfinished round.

**Still open.**
 Search targets,
 result actions/ranking,
 the search button's exact
placement on each platform,
 and page empty/error behavior need visual and interaction
evidence.
 These details are not determined merely by choosing a page.

### D48. One integrated top bar on the Search page (2026-09-23)
The user corrected the first Search-page prototype:
 it had a separate "Search"
app bar and query bar.
 Merge Back,
 query input,
 and Clear into **one page-level
search header**,
 with results directly beneath it.
 The Search region must not
stack a player header above the query or add another in-app title strip.
Context outside a bounded Search region may remain visible if a candidate uses
only part of the unfolded display;
 that layout is permitted,
 not selected.
 A baseline M3
full-content Search header (72dp with a divider) supplies the visual anatomy.
The player continues to expose a distinct Search button that opens the page
(D47).
 This is a design decision,
 not a production implementation instruction.

### D49. Pixel 9 Pro Fold is the visual source for every platform (2026-09-23)
The user corrected the desktop Search-page round:
 all visual decisions follow the
Pixel 9 Pro Fold's **folded cover** and **unfolded inner** screens.
 The cover is
1080 × 2424 physical px,
 and the inner panel is 2076 × 2152 physical px.
The original cover estimate of about 411 × 923dp was derived from published
pixel density,
 not device configuration.
 Direct AVD probing reports 390dpi on
both panels,
 making its captured cover about 443 × 994dp and its inner display
about 852 × 883dp;
 see `device-metrics.md`.
 This corrects the review scale,
not the user's physical-panel choice.
 Desktop
inherits those treatments even where that is less optimal for a desktop window.
Do not substitute 360 × 640,
 480 × 600,
 or 1100 × 640 desktop mock windows as
visual decision targets.
 The unfolded centre connector keeps **informational material** clear
under clarified E2;
 this does not ban continuous surfaces or hit regions.
 Those native Slint experiments are historical only.

**Effect.**
 Redraw D47/D48's Search button and one-header Search page on both real
Fold panels in a debug-only native Android prototype,
 in light and dark.
 A desktop
port or window-size selection does not overrule the chosen Fold geometry or create a
parallel design vote.
 Production implementation remains unauthorized.

### D50. The unfolded control deck remains visible (2026-09-24)
The user requires the playback/control deck to **never be hidden while the
Fold is unfolded**.
D53 and D54 permit only their explicitly measured floating-Gboard and
font-update-banner exceptions;
other keyboard states remain subject to this requirement.
 This includes the D47 Search destination,
 regardless
of whether Search occupies one side or more of the display.
 A Search layout
may replace folder or track content,
 use a temporary pane,
 or span a
surface,
 but it must reserve visible space for the existing deck instead
of drawing over it or replacing the whole player view.
 Verify its visibility
while the query is focused and Android's keyboard is present,
 not only in
keyboard-closed screenshots;
 "never" includes typing.
 A 200% text probe with real floating Gboard shows a counterexample:
 its x `[274,1180)`, y `[310,1081)` key surface overlaps the deck
 title at `[258,1042][781,1145]` on the 2076 × 2152px inner panel.
 Do not mark D50 fully validated from the debug bottom-IME captures.
The real split-keyboard test passed after settling,
while a Gboard font-update banner briefly clipped the final mode.
D54 accepts that measured brief banner overlap;
D50 still governs non-exempt keyboard states.
 D47's separate-page
interaction does not supersede this persistent control region.

**Rejected evidence.**
 The debug-only `search-layout-docked` study overlays
the deck;
 `search-layout-wide-list` and `search-layout-wide-grid` replace
the view containing it.
 Their native captures do not qualify as review
choices,
 regardless of their query/result placement.
 No production change
is authorized.

### D51. Search uses the IME-lifted left deck (2026-09-24)

The user selected **A** from the native Fold Search comparison.
 On the
unfolded panel,
 Search's integrated Back/query/Clear header and results stay
together on the right;
 the existing playback/control deck remains at the
bottom-left when the keyboard is closed and lifts above the visible system
keyboard while typing.
The upper-left keeps the **same folder browser** when Search is open,
including while typing;
its visible viewport may shorten above the IME-lifted deck.
Do not replace it with a `Current folder` caption or blank it solely because
the keyboard appeared.
Search must not hide the deck or its controls (D50).
At the measured tall keyboard height,
only part of the existing browser header fits.
The user accepts a little viewport clipping,
including around Open,
rather than a different upper-left composition.
Do not claim every folder target is reachable while that viewport is short;
keep the full deck visible.
 The cover uses one
full-width Search destination with the same integrated header.
 D49 makes
this Fold treatment the visual source for desktop too.

B (fixed upper-left deck) and C (Search left with an upper-right deck) were
not chosen.
 Their native captures remain historical comparison evidence,
not active choices.
The corrected A-only review was recaptured with the same folder browser
visible in the shortened upper-left viewport under a 300dp debug IME;
its unfolded deck and query/results remained visible.
On 2026-09-25 the user said “Okay, I like it” after reviewing that
correction.
This affirms the retained-browser composition,
not the separate synthetic 415dp title/transport reflow or universal D50
compliance.
The older real-Gboard PNGs predate that browser correction.
A later real Gboard floating-keyboard probe at 200% text obscured part of
the deck title; moving that keyboard lower obscured more controls.
D53 accepts this specific real floating-Gboard overlap with the unfolded
deck without changing the user's A selection.
On the folded cover, the same real floating mode covered both matching
result labels while a focused `cam` query remained visible.
D55 accepts that separate observed cover overlap.
A separate disposable Fold AVD verified real Gboard split input on the
inner panel and full-width input on the cover at 100% and 200% text.
At settled 200%, its inner IME began at y `1352` and the final mode ended
at y `1313`; cover result labels stayed above its y `1605` keyboard.
The original AVD's active Gboard was an update to versionCode `175981944`,
not the preloaded `175753756`.
Updating the disposable Gboard to the same version did not reproduce its
floating behavior, so version alone is not an explanation.
A Gboard font-update banner on the disposable inner panel temporarily
raised the IME top to y `1140` and clipped the final mode until dismissed.
D54 accepts that measured brief banner overlap;
D53 separately accepts the floating-keyboard deck overlap.
Other D50 states and the user's A selection remain unchanged.
Other heights remain unverified.
A debug-only app probe subsequently observed the docked bottom inset and
bounding rectangle;
its synthetic 415dp banner-height reflow is not a chosen replacement.
`package/music-player/design/evidence/gboard-geometry.md` indexes sanitized real Gboard evidence;
its passing split and full-width states do not supersede the floating and
banner counterexamples.
This is a **design choice only**;
 it does not authorize production changes.

### D52. Remove the redundant positive-results heading (2026-09-24)

The user directed removal of the separate `Results for “cam”` text after
choosing A.
 Positive Search results start directly beneath the single
Back/query/Clear header on both Fold panels;
 do not repeat the query as a
results heading.
 The query remains visible in the header,
 and folder/track
labels still identify result meaning.
 Keep the distinct no-results and
library-unavailable explanations,
 because they convey states rather than
repeat a successful query.
 This is design-only until implementation is
separately authorized.

### D53. Real floating Gboard may obscure the unfolded deck (2026-09-26)

The user explicitly said that “real floating Gboard still obscures the deck”
is **acceptable**.
When real floating Gboard overlays the unfolded playback deck during Search,
that specific occlusion is an exception to D50's otherwise complete-deck
visibility requirement.
Do not move or replace selected Search A merely to make the deck visible
beneath a user-positioned floating keyboard.
The actual folder browser remains above the bottom-left deck;
query and results remain together on the right (D51),
and D52 still removes the repeated positive-results heading.

This exception does **not** permit clipping or hiding the deck under a
docked or split keyboard.
D54 separately accepts the measured transient Gboard font-update banner;
D53 alone does not generalize to other keyboard overlays.
D55 separately accepts the observed floating cover keyboard obscuring both
Search result labels;
D53 alone does not authorize that cover behavior.
The synthetic 416dp anticipatory reservation and inline deck reflow remain
unaccepted debug-only studies.
This is a design-scope clarification,
not production authorization or proof that the remaining D50 states pass.

### D54. The brief Gboard font-update banner may clip the deck (2026-09-26)

When asked about the measured `Keyboard font size updated` state,
the user said “That brief banner is also acceptable.”
On the disposable Fold at 200% text,
this real Gboard banner briefly raised the IME top to y `1140` and clipped
the final mode to `[73,1076][965,1140]` until its `OK` action was tapped.
D54 accepts **that measured transient system banner overlap** as a second
exception to D50;
it is not approval to hide the deck beneath ordinary settled docked or split
keyboards,
nor beneath arbitrary taller keyboards or persistent banners.
The selected A still keeps the actual folder browser above the bottom-left
deck and the integrated Search query/results on the right.

The synthetic 415dp inline title/transport reflow and 416dp anticipatory
reservation remain unaccepted debug-only studies;
no banner-specific layout change is required by this decision alone.
D55 separately accepts the measured cover result-label overlap;
D54 alone does not authorize it.
No production implementation is authorized.

### D55. Real floating Gboard may obscure folded-cover Search matches (2026-09-26)

When asked about the measured cover state and given its sanitized capture
path,
the user answered “Also acceptable.”
At 100% text on the folded cover,
real floating Gboard covered both matching result labels,
`Camellia` and `Another Xronixle`,
while the focused `cam` query remained visible.
D55 accepts that **specific floating-keyboard result overlap** without
changing D47's separate cover page,
D48's single Back/query/Clear header,
or D52's removal of the repeated positive-results heading.
It does not approve an obscured query,
missing or nonfunctional search results,
or ordinary docked/full-width keyboards hiding matching labels.
The accepted A review still shows results with its separate debug IME;
the sanitized real-Gboard capture is the evidence for this exception.
D53 concerns the unfolded deck,
D54 concerns the brief unfolded font-update banner,
and D55 concerns the folded-cover floating keyboard.
No production implementation is authorized.

### D56. The folded-cover Search result list fits above a bottom keyboard (2026-09-26)

The user chose **R** in the cover-viewport design review.
For the selected A cover Search page,
keep the integrated Back/query/Clear header fixed and give **only the scrolling
results viewport** keyboard-aware bottom space while an ordinary docked or
full-width keyboard is open.
Positive matches still start directly beneath the header (D52).
Long result names retain their wrapping and the last result and supporting
text must be reachable by scrolling above the keyboard,
not remain stranded beneath it.
The selected unfolded Search layout,
upper-left folder browser and bottom-left deck are unchanged.

The same installed debug APK supplied a 200% text-scale failure control:
with a 300dp bottom keyboard,
row 18 remained at y `[2151,2254]` below keyboard top y `1693`
even after another end-of-list swipe.
The opt-in cover-only viewport brought its title and supporting line to
y `[1479,1582]` and `[1582,1673]` above that keyboard.
At 100%/200%,
the final row was also scroll-reachable above **settled real full-width Gboard**
in this debug fixture.
These are bounded design-study results,
not validation of animation frames,
all keyboard geometries,
real search ranking or activation.

The measured R prototype required another swipe to recover the final row
after hiding and refocusing the keyboard.
D63 to D68 subsequently settled the interaction direction:
D68 requires the intended row to remain visible after same-query
keyboard refocus.
Native realization and verification remain open;
R selects the cover viewport direction,
not that extra swipe as a requirement.
C's unchanged cover viewport was rejected for the measured end-of-list
occlusion.
D55 remains the separate accepted **floating** cover Gboard exception,
not a waiver for ordinary full-width input.
The comparison and limits are in
`package/music-player/design/evidence/search-result-overflow.md`.
No production implementation is authorized.

### D57. The observed inner floating Gboard may obscure some Search result lettering (2026-09-26)

The user chose **A** in the separate inner floating-result review.
At 200% text in the selected A long-results fixture,
real floating Gboard covered portions of middle **right-pane result labels**
while the `cam` query stayed readable and later results were visible below
the keyboard.
D57 accepts **this specific observed floating overlay** as a bounded
exception to visual result-lettering visibility.
It does not extend to other floating placements,
ordinary docked or split keyboards,
an obscured query,
missing matches or nonfunctional result actions.
Whether the covered rows can be scrolled into clear space,
reached or activated was **not** established;
D57 does not accept their absence or inaccessibility.

D53 independently accepts the measured floating-keyboard overlap with the
left playback deck;
D55 concerns floating results on the folded cover.
D56 keeps the folded-cover result list scrollable above ordinary full-width
keyboards.
No inner layout move,
IME placement mechanism,
production implementation or further device experiment was selected.
The sanitized native capture and bounded fixture are in
`package/music-player/design/questions/floating-results-review.html`
and `package/music-player/design/evidence/search-result-overflow.md`.

### D58. Search header and result rows share leading columns (2026-09-29)

The user corrected a visible offset in the Search-opened state:
**Back arrow and folder/music result icons share one horizontal center**,
and **query text and result titles share one horizontal start**.
The integrated header already used a 48dp leading icon target,
but debug result rows used a 24dp icon plus 12dp spacer.
At 200% text before the correction,
inner query `cam` began at x `1249` while result `Cam` began at x `1220`;
on the cover the same pair began at x `156` and x `127`.

Debug-only commit `baa37caaf` centers 24dp result icons inside the same
48dp leading slot as Back on both panels.
On the subsequently installed disposable Fold APK,
UI Automator reported `cam` and `Cam` both starting at x `1249` on the
inner display and both at x `156` on the cover at **100% and 200% text**.
In native 200% light-scheme screenshots,
the Back,
folder and music glyphs had respective horizontal centers x `1190`
on the inner panel and x `97` on the cover;
the old folder icon centers were x `1161` and x `68`.
The Back `IconButton` remains 48dp in the debug source;
the 48dp result slot is decorative,
not an activated result target.
The corrected icon paint starts beyond the measured inner crease,
and the browser/deck and static fixture order remain as before.
This is not a universal E2 glyph-ink or accessibility pass.
See `evidence/search-header-result-alignment.md` for the pixel method,
positive control and bounded image set.
Result activation,
ranking and accessibility still require separate decisions and verification.
This is a design-only alignment correction,
not a production Search implementation or permission for new IME tests.

### D59. Emphasize each visible Search match using the OS accent in OKLCH (2026-09-29)

Before deciding #116 membership or order,
the user required each visible `cam` match (including case variants such as
`Cam`) to be highlighted **in place** in Search results.
That includes result titles and the supporting parent-folder text when it
contains the match;
`cam` must visibly match `Cam` in the accepted fixture.
This establishes the shown ASCII case equivalence,
not general Unicode casefolding.
The emphasis does not restyle the retained left folder browser or turn a
static row into an action.
The first debug-only highlight used the theme's tertiary-container role
and rendered purple;
the user rejected that color and specified a color **derived from the
selected theme and adjusted in OKLCH**.
A3 makes the OS accent the theme source.
The revised debug fixture uses `MaterialTheme.colorScheme.primary` and the
existing `mixOklchWithNeutral` utility to derive light and dark match fills
(note 2026-10-13: that utility is deleted, and the Search match fill is now a role derived in
`core/ThemeRoles.kt`, D108),
with bold text as a second cue.
No fixed purple swatch,
new production Search feature,
result membership/rank,
Unicode matching grammar or keyboard behavior is decided here.
The corrected disposable-AVD APK SHA-256
`7195af99031158bfb8efce2abab3c98ebaf17928739c290010bc6dea1317244f`
was captured at 100% and 200% in light/dark with the keyboard closed;
`evidence/search-match-emphasis-native.md` records bounded contrast,
text bounds and provenance.
The exact prototype blend is reviewable and not a universal palette pass.

### D60. Search results use direct names, not parent-only track expansion (2026-09-29)

After reviewing D59-highlighted native inner and cover examples,
the user selected **Scope D** over P.
A folder may appear because its **own name** matches;
a track may appear because its **own final filename component** matches.
`Track.displayPath` and `PageEntry.name` can contain relative folder
segments (see
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/Track.kt`
and
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/core/Page.kt`);
indexing either entire displayed path would violate this decision.
Parent and ancestor folder segments remain disambiguating context,
not sources of track matches.
D sets the eligible fields,
not a current-directory-only coverage limit or a file-index implementation.
The synthetic `Another Xronixle` row must **not** appear solely because
its immediate parent is `Camellia` for query `cam`.
This keeps matching-folder results visible without deciding what tapping
that folder does (review task 129).
It does not decide broader Unicode equivalence,
filename extension handling,
mid-word substring matching,
multiple query terms,
result count limits or an index implementation.
The user's choice is a design direction,
not authorization for production Search code.

### D61. Mix result types by relevance rather than grouping by kind (2026-09-29)

The user independently selected **Order M** over folders-first F and
tracks-first T in the same D59-highlighted matrix.
For the accepted `cam` examples,
an exact `Cam` track precedes the prefix `Camellia` folder,
which stays near the top;
`Live at Camellia` remains a match beginning at a later word.
Other direct track and folder hits interleave across those observed
relationships.
The example does not decide whether `cam` inside `Scamper`
should match.
This does not select a concrete scorer,
normalization rule,
locale-aware or deterministic tie-break,
result action,
or a production implementation.
The selected A header,
actual left browser,
complete deck,
D58 columns,
D59 theme-derived highlighting and E2 floor remain unchanged.

### D62. Search-library choice is outside the current UI review (2026-09-29)

After the D/M selection,
a further W/A word-boundary question was prepared.
The user corrected the task boundary:
**the eventual choice of a good fuzzy search library does not affect the
UI design and is not this agent's job right now**.
Withdraw the W/A question rather than asking the user to specify a
hand-rolled matcher or evaluating/installing a library now.
The W/A native fixtures remain historical exploratory evidence only;
neither rule is selected.
D60's direct-name result presentation,
D61's mixed visual order,
D59's in-place OS-accent match highlighting,
and the selected A layout remain the current design.
Future implementation may use a library to produce result identities and
match ranges,
but must not silently change these selected visible behaviors;
a genuine conflict returns for design review then.
Continue the separate UI questions for Back/Clear/focus,
empty/unavailable states,
result activation and accessibility.
No production Search implementation is authorized by this correction.

### D63. Search entry requests query edit focus (2026-09-29)

The user selected the recommended **E-fast** interaction under review task 127:
opening the separate Search destination requests query edit focus and a
keyboard so the user can begin typing without first selecting the field.
This is an interaction intent,
not proof that every native keyboard appears or fits the selected layout.
Actual focus and keyboard behavior need implementation-boundary verification.
Accessibility focus remains a separate review task 118 decision.

### D64. The visible Back arrow exits Search directly (2026-09-29)

The user selected **A-exit**:
activating the page-level Back arrow returns to the player even when a
keyboard is shown.
The arrow does not first dismiss the keyboard while leaving Search open.
D47 required a Back path from a separate destination;
D64 now settles that visible control's first action while typing.
System Back is distinct:
Android SDK 37 documents default IME interception when the keyboard is
shown and a conditional IME-owned bypass,
so the logic walkthrough models keyboard-first system Back only as an
illustrative baseline,
not a cross-keyboard guarantee or a selected app override.
Returning must not replace the retained left folder-browser location or
playback deck with a Search fixture.
No native return-focus target is selected here (review task 118).

### D65. Clear preserves the current edit-focus state (2026-09-29)

The user selected **C-keep**:
Clear erases the query without leaving Search and does not itself force
query refocus or reopen a dismissed keyboard.
If edit focus and the keyboard are already active,
they remain active;
if the keyboard was dismissed or edit focus moved away,
Clear preserves those states.
The debug prototype's empty-query action was not evidence of this focus
policy;
this is the new design decision.

### D66. A new Search visit starts with an empty query (2026-09-29)

The user selected **Q-new**:
after returning to the player,
opening Search again begins with a fresh empty query rather than restoring
the previous text.
D63 requests edit focus on that new visit.
This describes navigation within a running session;
process restoration and posture changes were not compared here.

### D67. A restored query would start results at the top (2026-09-29)

The user also selected the recommended conditional **P-top** preference:
**if** a future design restores a previous query on re-entry,
it should begin at the first results instead of reviving the prior deep
result position.
Under D66's active Q-new behavior,
there is no restored result set and P-top has no immediate effect.
Do not treat P-top as permission to restore the previous query,
nor as a result-activation rule.

### D68. Same-query keyboard refocus preserves row visibility (2026-09-29)

The user selected **S-visible** as the desired interaction:
after scrolling to a result and hiding then refocusing the keyboard
without changing the query,
keep the intended row visible in the resized viewport.
Preserving only a raw list offset is insufficient:
the D56 cover observation required another swipe to recover the final
row after keyboard refocus,
and that extra swipe was not selected.
The middle-row and final-row logic walkthrough illustrates this
visibility distinction but cannot prove a native scroll correction.
Verify it with a future implementation before claiming it works with
arbitrary keyboard geometries;
do not restart IME experiments in this design review without first
making a compelling case to the user.
D57's bounded floating-Gboard overlap and review task 129 result activation remain
separate.

The user accepted all review task 127 recommendations and pointed out that
asking for an additional preference answer was unnecessary.
Treat these choices as selected,
not as pending defaults in the review form.
No production Search change or search-library choice is authorized.

### D69. Search status claims follow verified query and source state (2026-09-29)

The empty/unavailable review has one evidence-led direction,
not an additional preference ballot:
keep **no query yet**,
**completed current-query no match**,
**confirmed empty searchable inventory** and **known source failure**
separate in the selected A result region.
A partial scan,
source refresh,
prior-query results or an empty track list cannot by itself establish
that no folder/track name matches the current query.
Peak analysis is not a prerequisite for name Search under D27.
An explicit source failure takes precedence over ordinary empty-query or
no-match copy;
it does not silently erase the query,
leave Search,
steal edit focus or reopen a dismissed keyboard.
The fixed debug states and
`package/music-player/design/questions/search-status-evidence.html`
illustrate this distinction on both Fold panels at 100% and 200% text
without executing a real lookup.
This selects status **truth conditions**,
not an index scope,
matcher,
backend state implementation or result tap.

### D70. Keep the unqueried prompt and completed-no-match treatment (2026-09-29)

The selected A right pane may show the existing “Search your music” /
“Type a name to explore your library” prompt while a usable source has
no query.
Do not label this first visit “No results.”
For a **fully evaluated** nonempty query with zero direct-name hits,
keep the query and Clear in the fixed header and show a no-match title
naming the entered text with “Try another name” support.
The native `zzq` fixture demonstrates this paint only;
it does not prove a search ran,
long-query wrapping,
keyboard-open fit or arbitrary text.
D52's rejected repeated **positive-results heading** does not ban a
no-match diagnostic from naming the affected input.
The real Search scope and completion signal belong to implementation
work;
without that signal,
do not render a final no-match verdict.

### D71. Do not promise a library recovery that has no owner (2026-09-29)

Reject the debug fixture's broad “Library unavailable” /
“Search returns when the library is available” as selected Search copy.
Its inner screenshot keeps visible folders and an already-playing track,
while the fixed unavailable marker proves neither total library loss nor
automatic recovery.
Only a **known inability to search the current source** may replace the
result region with a Search-specific unavailable explanation.
Name the cause when known and present only a recovery action whose owner
can actually perform it;
changing to another folder is a scope change,
not repair of the original source.
Do not infer unavailability from a zero-length track list or missing
device-wide permission alone:
a held folder may still be a readable source,
while the current production permission gate may prevent Search opening.
The exact cause-specific sentence and control depend on a future real
source-status signal;
no generic button or successful retry is promised by this decision.
Preserve the integrated header,
query,
actual left browser and complete deck without presenting their stale
content as freshly verified.

These D69 to D71 directions were derived from the existing decisions,
production source selection and sanitized native fixture review.
They are recorded with a correction/veto path rather than another
ceremonial preference question,
in response to the user's correction after the D63 to D68 selection.
No production Search change,
fuzzy-library work,
new IME experiment or original-AVD modification is authorized.

### D72. A folder Search result opens its folder without autoplay (2026-09-29)

Carry the existing `selectPage(page)` action into Search:
a folder result returns to the ordinary player folder view with that
folder selected and its queue page scope updated.
It does **not** load another track or stop the already-playing stream.
Changing only the visible left picker while leaving the right Search
pane open would not expose the selected folder's track view on the
unfolded panel,
and would hide the outcome entirely on the folded cover.
This is a coherence decision from the existing player behavior,
not evidence that a Search row is currently clickable.
The new Search visit after this return starts with an empty query (D66).

### D73. Track Search results carry existing play/pause row semantics (2026-09-29)

A directly named result for a different track starts that track,
using the existing non-current player-row meaning.
`PlayerController.playIndex(index)` calls `playCurrent()`;
its source path selects the track's owning page in player state.
A result for the already-current track uses the existing current-row
`togglePlay()` meaning:
pause while playing,
resume when its URI is already loaded,
or load it if necessary.
Do **not** call `playIndex(current)` and describe that as an equivalent
toggle.
The historical parent-only `cam` result for the deck's Another
Xronixle is excluded by D60;
a current-track test needs a direct own-name query such as `Another`.
No result handler,
file-error path,
scroll-to-row or accessibility focus has been implemented or verified
by this design decision.

Amended by D127 (2026-10-14): a result for a different track no longer starts that track.
It selects it, as a row click does (D124); the current-track toggle in this decision stands.

### D74. Successful track result activation returns to player (2026-09-29)

After the source-backed track-action defaults were presented,
the user chose **Return** over **Stay** for the track-navigation
consequence.
A successful other-track start (since D127, an other-track select) leaves Search and presents the ordinary
player view with the selected owning page;
a successful current-track play/pause action likewise returns to player,
without inventing a playhead restart or changing the selected page solely
for that toggle.
The separate Search query ends on return;
a later Search opening is fresh under D66.
Do not claim that the new track's row is scrolled into view or that its
accessibility focus was restored:
`refresh(followCurrent = true)` selects the page,
not a proven row position.
A stale or failed result activation must not claim playback succeeded or
substitute a different target;
it stays in Search for truthful error handling,
with D9's missing-item bar as the incumbent presentation precedent to
verify during future implementation.
The [historical Stay/Return logic comparison](questions/archive/search-result-activation-before-return.html)
was not an Android tap test.
No production Search implementation,
fuzzy matcher selection or new IME experiment is authorized.

### D75. Search opens with query accessibility focus (2026-09-29)

Adopted as an evidence-led recommendation under the user's instruction
to record strongly determined defaults with a correction/veto path,
not as a separately answered questionnaire.
The user subsequently accepted the reviewed goals as “good enough” and
asked to continue design work.
That confirms D75 to D80's design direction,
not the unverified native behavior.
Initial accessibility focus goes to the query field on both Fold panels.
Its accessible name remains “Search music”;
the current value is separate from that name.
Back remains the preceding reachable control in the header's reading order.
This is separate from D63's request for **edit focus** and a keyboard:
neither focus system proves or overrides the other.
Query-first prioritizes the Search task just invoked;
Back-first would announce escape first but require another forward
gesture to reach the field.
Material guidance permits either initial target and does not select ours.
The fixed `cam` fixture did not exercise a real Search-entry transition,
so this is a design goal,
not verified initial TalkBack focus.

### D76. Keep Search together in reading order (2026-09-29)

The linear Search region is Back,
query,
Clear when present,
then the current results or status.
On the inner panel,
the retained folder region follows as a whole,
including its header,
alphabet rail and folder grid,
then the complete playback deck.
Do not interleave the left alphabet rail between the query and results.
The cover has only its full-width Search destination;
do not add a cover Search deck to reproduce the inner layout.
D39 still governs the ordinary player outside Search.

Search is not an accessibility modal or focus trap.
Expose named regions so retained browser/deck controls are discoverable
without traversing every hit in a large result list;
the host application's actual region-navigation route needs verification.
Keep visual composition unchanged and do not add repeated visible headings
or hide the retained controls merely to simplify traversal.

### D77. Each result is one named action (2026-09-29)

Expose each result as one actionable semantic unit:
own filename/folder name once,
track/folder kind,
useful parent context and the real activation meaning.
A folder offers Open folder without autoplay under D72.
A different track offers Play track (D127 changed this to Select track, because a different track is
only selected).
An already-current track offers Pause track while playing,
or Play track while paused/unloaded,
using D73's actual toggle semantics.
“Current track” is structured state,
not a synonym for “playing” or a second copied title.
The resulting playback state remains discoverable on return.

Disambiguate equal names with their relative parent path.
For a root-level item,
use the library-root context rather than inventing a parent or saying
“unknown”.
Parent context describes location,
never eligibility from a parent-only match.
Decorative type icons and D59 highlight fragments are not extra
accessibility stops.
Retain visible title/support text,
actual minimum 48dp action targets and a labeled focus boundary;
do not replace useful child text with a synthetic duplicated sentence.
Exact platform speech and row activation remain implementation gates.

### D78. Announce meaningful current Search changes without focus theft (2026-09-29)

Make the results/status region persistently discoverable and announce
meaningful changes without moving query edit or accessibility focus.
Coalesce result updates rather than interrupting speech for every key,
and cancel superseded announcements after another query,
Clear or leaving Search.
Completion belongs to the **current query and source evaluation**,
not a silence timer.
Announce an exact result count only when its scope is known;
partial/stale results never establish completed no match under D69.
If useful results appear while evaluation continues,
distinguish availability from completion instead of announcing absence.

An empty query retains D70's search invitation,
not a failed lookup announcement.
Completed no match may name the query;
confirmed empty searchable inventory and known source failure retain
their distinct D69/D71 meanings.
Do not announce an unsupported recovery action,
a fabricated total or hidden old-query success.
Exact cause-specific error/recovery language remains dependent on
internal review task 131's future source-status owner.

### D79. Return focus explains the actual navigation outcome (2026-09-29)

Visible Back returns accessibility focus to the player control that
invoked Search when it still exists.
If that control disappeared,
use the current folder context as a deterministic fallback,
not a disposed Search node.
This does not change the distinct system-Back/IME boundary in D64.

After a successful folder action,
focus the selected folder context in the ordinary player:
its selected folder item on the inner panel or folder title on the cover.
After a successful other-track action,
reveal and focus that track's player row on its owning page.
After a successful current-track toggle,
focus its player row if present on the retained page;
otherwise use the deck's current-track context without changing pages
solely for the toggle.
A missing target falls back to the destination's folder context.

Transfer focus only after the selected action genuinely succeeds and
the destination is ready.
A stale/failed activation remains in Search with a truthful,
discoverable diagnostic and no substitute target.
Do not let a delayed completion move focus after later navigation.
These are desired targets,
not proof of scroll-to-row,
native success/error handling or TalkBack return speech.

### D80. Preserve meaningful focus through Search changes (2026-09-29)

For same-query updates,
keyboard refocus and fold/layout changes,
preserve the focused result's identity and make its target visible when
that item still exists.
This complements D68's visibility goal without promising arbitrary
keyboard fit.
If an item vanishes,
focus the next surviving item in the current order,
then the preceding one,
or the query if no result survives;
do not silently activate a replacement.
If Clear removes its own focused control,
accessibility focus falls back to the query while D65's edit-focus and
keyboard-state contract remains unchanged.
A new query cancels old-result focus restoration,
but an update must not steal focus from editing.
Hardware-keyboard mapping remains separate from these screen-reader goals.

These recommendations can be corrected or vetoed in chat.
They close the **design choices** for internal review task 118,
not native accessibility acceptance.
See `evidence/search-accessibility-boundaries.md` and
`evidence/search-talkback-native-baseline.md`.
The inner baseline is fixed-query,
keyboard-closed and 200% only;
cover physical-swipe delivery has no positive control.
No production Search code,
matcher selection,
new IME experiment or original-AVD change is authorized.

---

### D81. Supporting text is user-configurable through templates in Settings

Supporting text is templated and the user can configure it in Settings.
The user reaffirmed this agreement when the filename review presented
supporting text as a fixed placement/visibility policy.
Search filename review must preserve this configurability.
D35's neutral default supporting-text role does not hard-code its content.

The fixed-policy filename question is withdrawn.
The native captures remain evidence of exact authored layouts,
not implemented templates,
Settings functionality or accepted defaults.
Template fields,
grammar,
editor details,
other row-type scope and interactions with required distinguishing
information remain undesigned.
Title customization is not implied by configurable supporting text.
D77's accessible-action naming direction and the need for visible distinction
before activation remain intact;
no automatic suffix restoration or collision fallback is selected.
No production implementation is authorized by this clarification.

### D82. Accept Android's standard media notification presentation

The human explicitly chose no special designs for Android media notifications.
Accept the presentation Android provides;
do not build notification variants or ask for appearance,
layout or custom-action design preferences.
This closes the custom notification design backlog item.
Functional playback/session wiring and truthful metadata remain implementation
responsibilities,
not proof supplied by this design choice.
No production implementation,
live playback or real-device notification mutation is authorized here.

### D83. Error feedback is an auto-dismissing, dismissible overlay

The human rejected a bar that changes player layout.
Use a floating toast/snackbar for error feedback,
with automatic expiry and immediate manual dismissal.
Try to fit the message in two visible lines.
When more detail would need additional lines,
direct the user to capture Android logs rather than expanding the message
or reserving player space.
Full operation details belong in tagged diagnostics,
not a growing on-screen bar.

This supersedes D9's layout-reserving bar and D29's bar-specific collision
owner,
not their truthful outcome requirements or the Undo capability gate.
Feedback appearing,
expiring or being dismissed must not resize the folder browser,
track viewport or deck.
Do not reinterpret an accepted request as successful trash or log capture
as actual storage recovery.

The current work remains an isolated authored design study.
No production storage,
playback,
IME or TalkBack operation is authorized by this presentation change.

### D84. True-peak analysis is automatic and non-optional (2026-10-05)

The human withdrew D11's `Analyse true peak in the background` row:
Settings provides no switch for analysis,
because true-peak analysis is automatic and not optional.
This supersedes D11's third row and D27's ask-before-analysing prompt,
whose four answers (`Scan once`,
`Always scan`,
`Dismiss once`,
`Dismiss forever`) existed only to make analysis optional.
D27's automatic opening of an available system music library stands.
D12's Re-analyse on the track menu stands.
D26's scan bar stands;
reading its Pause/Resume control as a temporary pause that does not make
analysis optional is this record's interpretation,
not the human's statement.
When analysis runs is not selected here.
No production implementation is authorized by this record.

### D85. The Settings pane has no closing sentence (2026-10-05)

The human dropped settings-a's closing paragraph,
`That's everything. There is no library to configure, no tags to read, and no audio processing beyond normalisation — so
this pane stays short until the app grows.`
D11's wording about the pane saying out loud that it is short is withdrawn
with it.
The sentence was mock copy that no decision chose,
and D81 already adds template configuration to Settings.

D86 (2026-10-05) then removes the pane's two remaining rows.

### D86. Prefix stripping and resume are always on, not settings (2026-10-05)

The human withdrew D11's two remaining rows:
`Strip common prefixes from filenames` and `Resume where I left off` are
always on and are not settings.
This supersedes the same-day answer that only the analysis switch goes,
which `HANDOFF.md` recorded under the two-row Settings publication.

The behaviours stand as D11 described them,
without a switch:
titles are shown without a common filename prefix,
as in `Another Xronixle` for
`かめりあ(Camellia) - Another Xronixle.flac`,
and launch restores folder,
track and position,
paused.

With D84 and D85,
nothing of settings-a's content remains.
Settings stays a destination only because D81 places template configuration
there;
that content is undesigned (section 11e of `open-questions.md`).
The Fold placement used by the Settings study was adopted from D50,
D51 and existing Settings behaviour,
not accepted,
and this record does not change that.
The published two-row Settings study is withdrawn by this record.

A scoped source reading on the same day,
not an acceptance of production:
Android `PlaybackService` already loads a persisted session through
`SessionStore.load` when it starts,
with no setting;
whether it restores folder,
track and position paused as D11 words it was not checked.
`core/RelPath.kt` strips the longest common directory prefix from displayed
paths and `rowDisplay` strips a folder label;
a search of Android and desktop production source found no code that removes
a shared filename prefix or the extension as D11's example does.
No production implementation is authorized by this record.

D87 (2026-10-05) answers what the Settings page then holds:
nothing.

### D87. The Settings page stays and is empty (2026-10-05)

Asked nothing further,
the human said of the Settings page:
leave it empty.
Settings therefore remains a destination with its header and its way back,
and draws no row,
no sentence and no placeholder below the header.
This supersedes the reading in D86 that Settings stays only because of D81;
it stays because the human kept it.
D81's template configuration is still placed in Settings and still
undesigned (section 11e of `open-questions.md`),
so the page is empty until that is designed.
The Fold placement of the page remains adopted from D50,
D51 and existing Settings behaviour,
not separately accepted.
The human also said that an empty page needs no study,
so no native study of the empty page is built;
the withdrawn two-row study is kept as a record and is not replaced.
No production implementation is authorized by this record.

### D88. Nothing is locked by hash (2026-10-05)

The published first-run study had been left uncorrected on the ground that
its review verification bound the page by hash.
Told so,
the human said:
do not lock by hash,
ever.

What follows from it:

- No check fails or refuses because a page,
  template,
  builder,
  test or record differs from a recorded digest.
- A recorded digest is never a reason to leave something uncorrected or to put a correction off.
- Review-verification records carry no digest of a page,
  template,
  builder,
  test,
  manifest or report.
  Those fields were removed from the published records under
  `questions/evidence/` on this date,
  and each record says so in `pageDigestsRemoved`.

A narrower reading the agent adopted,
not the human's words,
and open to veto:
digests that only name what was captured stay for now.
Those are the digest of each published screenshot in a witness manifest,
the digest of the debug APK that produced a set of captures,
and the digests of browser screenshots that were looked at.
The builders still compare each embedded screenshot with its manifest digest,
and some still require the manifest's APK digest to equal a value written in the builder.
Under the widest reading of the human's words those are locks too.
Removing them is asked of the human,
not done.

Asked the same day,
the human answered:
remove both.
So the narrower reading is withdrawn.
No builder compares a screenshot with a digest,
and no builder requires an APK digest or a prototype commit to equal a
value written in it.
A manifest may still say which APK and which commit produced its captures;
that is a statement,
and nothing is checked against it.
What a builder checks about an image is what it measures from the image itself.

On 2026-10-06 the agent reported that its private emulator-capture scripts,
which are outside this repository,
still compared the study APK's digest:
to keep one set of captures to one build,
and before replacing the package already installed in the emulator.
One such comparison had refused a capturing visit the night before.
Asked whether D88 covers those scripts too,
the human answered yes,
remove them.
The scripts that run a visit now tell builds apart by commit name and replace the study's package by its application id.
Those changed scripts were checked for syntax only;
no visit has run with them yet.
Scripts of finished studies still hold such comparisons;
they are not run any more,
and one that is reused gets the same change first.

Asked the same day whether to make this a rule for the whole repository in `AGENTS.md`,
the human declined:
"There are legit (rare) situations in which locking by hash is desired."
So D88 binds this package's design evidence and its capture tooling,
and is not a ban for the repository at large.

### D89. The template language and editor follow KWGT, with stated omissions (2026-10-05)

The human named KWGT (Kustom Widget Maker) as precedent for D81's template
editor and had it run first-hand in a throwaway emulator.
`doc/planning/music-player-template-editor.md` lists what KWGT's formula
language and editor do,
with a recommendation to keep or omit each.
The human answered:
"I'll go with your recommendations with what to omit and keep."

Kept:

- literal text with formulas between a pair of delimiters;
- field functions that take a mode word,
  with the player's own fields;
- nested calls and formatting functions;
- text conversion for case and for cutting to a length;
- `if` and comparisons,
  so a row can say what to show when a field has no value;
- a live preview on every keystroke;
- signature and argument help while typing;
- error lines that name the function and the problem;
- a field list that inserts at the caret.

Omitted:

- arithmetic,
  unless a use appears;
- global variables;
- markup for colour,
  bold,
  italic and the like;
- functions for weather,
  battery,
  network,
  calendar,
  notifications and web content;
- silence about a formula left open,
  which the player reports instead;
- saved favourite formulas;
- the button row for markup,
  colour and the globe.

Not decided by this answer,
because the note recommended nothing there:
the rest of text conversion,
regular-expression match and combined conditions,
whether the preview uses stand-in values or a real track,
whether ready-made examples are offered,
and whether an edit applies as typed or on an explicit save.
Those go to the built editor study as variants.

Also undecided:
the field inventory and names,
which row types get templates,
default templates,
fallback for an empty field or an invalid template,
the exact delimiter and function spelling,
and the editor's layout on the Fold.
D81's limits stand:
title customization is not implied,
and D77's visible distinction before activation remains required.
No production implementation is authorized by this record.

### D90. The agent's picks for the template entries left open by D89 (2026-10-05)

Asked whether the entries D89 left open were the human's to pick,
the human answered:
you choose,
but for extremely obscure features (for a music player) like regex lean on no.

The agent's picks,
each open to the human's veto:

- Text conversion stays at case and cutting to a length.
  Cutting with an ellipsis,
  padding,
  splitting and regular-expression replace are omitted.
  A row already shortens text that does not fit,
  and the others have no evident use in a track row.
- Regular-expression match is omitted.
- Combining conditions with `&` and `|` is omitted,
  unless a use appears.
  A nested `if` covers the case of two fields that may each be absent.
- The preview shows real tracks from the open library,
  not stand-in values,
  so the user sees their own names and the rows where a field has no value.
  Stand-in values are used only when no library is open.
  (D95,
  2026-10-06:
  a library is always open,
  so stand-ins are used only while the open library holds no track.)
- No list of ready-made examples.
  The default template is the example,
  and the field list inserts at the caret.
- An edit applies as typed while the template is valid,
  as Android settings do,
  with a way back to the default.
  While the template is invalid the rows keep the last valid template and
  the editor shows the error.
  There is no separate save step.

These are picks about scope and behavior,
not about appearance.
The editor's layout on the Fold is still to be built and looked at.
No production implementation is authorized by this record.

### D91. The editor's preview scrolls with the page (2026-10-06)

The editor study built three layouts of the preview,
because at 200% text the baseline loses the preview while typing inside a call:
the preview at the top of one scrolling page,
both preview rows fixed under the header,
and only the two result lines fixed under the header.
The agent ranked the fixed result lines first and the scrolling baseline last.

Asked through the question tool with the review page open,
the human chose none of the offered options and wrote:
"I've seen the preview,
and I have to say this isn't the way.
Least surprises vs the base platform is more appropriate,
and it doesn't prevent users from wanting to see more in a easy manner.
Preview scrolls."

So the preview scrolls with the page,
as the baseline has it,
and nothing is fixed under the header.
The two pinned layouts are rejected.
The reason given is a standard beyond this page:
prefer what surprises least against the base platform,
when the user can still reach the rest easily.
The agent's ranking weighed what stays in view at 200% text over that,
and was wrong for this human.

The rule the second build added,
that the page scrolls while the field has focus so the lines under it clear the keyboard,
was not asked about and is not decided by this answer.
No production implementation is authorized by this record.

### D92. The template language has no conditionals (2026-10-06)

The study's assumptions were put to the human through the question tool:
one template for the track row's supporting line;
KWGT's spelling,
with `mi`,
`tf`,
`tc`,
`if` and `+`;
the fields `title`,
`file`,
`ext`,
`folder`,
`path`,
`len` and `peak`;
the default template;
and the D90 picks.
The human answered:
"Keep except conditionals.
This is templating.
For prior arts:
We're not Vue,
we're just HTML."

So:

- The language has no conditional.
  `if` and the comparisons that exist only to feed it are removed.
  This supersedes the entry of D89 that kept them.
- Everything else put in that question stands and is no longer only assumed:
  the one template studied,
  the spelling without `if`,
  the field list,
  and the D90 picks.
- The default template used `if` to leave out the true peak,
  its separator and its unit while a file is not analysed.
  It cannot stand as written.
  What a row shows around a field that has no value is therefore open again,
  and is asked of the human with options,
  not picked by the agent.

No production implementation is authorized by this record.

### D93. An empty field is plain substitution (2026-10-06)

Asked through the question tool what a template shows around a field that has no value,
now that it has no conditional,
the human chose plain substitution over optional brackets
(foobar2000's `[...]`,
which its reference calls a conditional section)
and over before-and-after arguments
(in the manner of git's `% x` placeholders).

So:

- Text outside `$...$` is always shown,
  and a field with no value yields nothing,
  as text in HTML would.
  A separator a user writes next to an empty field stays in the line.
- `mi(peak)` yields the true peak with its unit,
  for example `−1.2 dBTP`,
  and nothing while the file is not analysed yet.
- The default template is `$tf(mi(len), m:ss)$ $mi(peak)$`.
  An analysed file reads `4:35 −1.2 dBTP`;
  one not analysed yet reads `5:12`.
  The middle dot between duration and peak that C2 describes is gone from the default line.
  Analysis is automatic (D84),
  so the missing peak is temporary.

No production implementation is authorized by this record.

### D94. Where the page rests while typing is the platform's (2026-10-06)

The second editor build replaced the platform's own scrolling to a focused field with an authored rule:
scroll so the lines under the field clear the keyboard by 8dp,
and never scroll the field's upper edge out of view.
Asked through the question tool whether that should also follow the platform,
as D91 asks of the preview,
the human chose the platform default.

So the authored rule is removed.
When the field takes focus and the keyboard opens,
the page rests wherever Android puts a focused text field.
The first build's captures showed what that costs at 200% text:
the last line under the field can end flush against the keyboard,
and the field's label can slide under the header when the field and its help are taller than the visible page.
This supersedes the last paragraph of D91,
which left the rule undecided.

No production implementation is authorized by this record.

### D95. A library is always open (2026-10-06)

Shown the rebuilt editor study,
whose last state previewed sample values under the note that no library is open,
the human wrote:
"What do you mean 'No library open'?
The app should always have a library open,
even if the opened library is empty.
It auto opens OS default library if no sessions are in the store."

So no screen of the player has a state without a library.
When no session is stored,
the app opens the operating system's default library (D27),
and that library may hold no track.
The editor's preview uses sample values only while the open library holds no track,
and says so.
This corrects the reading of D90 that spoke of no library being open.

Whether this also removes the first-run study's state without an opened source
(`evidence/first-run-access-boundaries.md`)
is put to the human,
not assumed.

No production implementation is authorized by this record.

### D96. Scrolling suffices to reach the field list (2026-10-06)

With the keyboard open the list of insertable fields sits under the template field and is mostly out of view.
Asked through the question tool whether that is worth designing for under D91's standard,
the human answered that scrolling suffices.
The field list stays where it is.

No production implementation is authorized by this record.

### D97. Which rows get templates: one built version, for approval (2026-10-06)

Asked what comes next in the template round,
the human wrote:
"Field list and other rows are pretty inconsequential and you only need to build one version
(according to the best of your design skills)
and ask for me to approve.
Do both."

The field list needs nothing more (D96).
For the other rows the agent builds one version,
its own design,
and asks for approval;
`doc/planning/music-player-template-editor.md` holds the proposal.
Nothing in that proposal is decided until the human approves it.

No production implementation is authorized by this record.

### D98. The template editor is accepted as shown (2026-10-06)

Shown the rebuilt review page with the empty library in place of no library (D95),
and asked through the question tool whether the editor under `What is decided` was right as shown,
the human answered "Right as shown".
The question also stated the agent's reading that the field list stays where it is (D96)
and invited a correction;
none was given.
D89 to D96 stand as the editor's design,
evidenced by `evidence/template-editor-boundaries.md`.
Typing is not connected in that study,
so real typing remains untried.

No production implementation is authorized by this record.

### D99. Two lines get templates: track rows and the playing track (2026-10-06)

The human approved the D97 version as built.
Settings lists two templates under `Templates`:

- `Track rows`,
  the supporting line of a track row,
  with the default `$tf(mi(len), m:ss)$ $mi(peak)$` (D93).
- `Playing track`,
  the line under the playing track's title in the deck,
  with the default `$mi(track)$ of $mi(total)$ $mi(peak)$`,
  which reads `1 of 16 −1.2 dBTP`,
  or `1 of 16` before analysis.
  Its fields are the track fields and two more:
  `track`,
  the file's place in its folder,
  and `total`,
  how many tracks the folder holds.

Each editor is titled with its template's name,
so the track rows' editor is `Track rows`,
not `Supporting line`.
The playing track's preview shows two files as rows.
The deck's line loses its middle dot.
Search results are not templated:
their second line stays the parent folder D77 requires.
Folder names and every title stay as they are.
This closes the row-type coverage D81 left open.

No production implementation is authorized by this record.

### D100. First run: access declined stays, no source opened goes (2026-10-06)

Asked which of the first-run study's states survive D95,
the human chose to keep the state where Android's music permission is refused
and to drop the state with no source opened.
Under D95 the system library is open even when its tracks cannot be read,
so the kept state shows that library as open but unreadable,
offering to grant access or to open a folder.
D10's empty state,
which D27 had narrowed to no system library or a declined one,
now applies only to this declined state.
The first-run study (`evidence/first-run-access-boundaries.md`) is to be rebuilt for this;
the agent's version of its copy is proposed in `doc/planning/music-player-first-run-access.md`.

No production implementation is authorized by this record.

### D101. The declined first-run state is approved as built (2026-10-06)

The human approved the agent's version of the declined state as built:

- Title `The device music library can't be read`.
- Body `It is open as your library, but access to music on this device was not granted. Allow access, or open a folder i
- nstead.`
- Buttons in this order:
  `Allow access` filled,
  then `Open a folder` and Settings outlined.
- The true-peak explanation below the buttons,
  as D10 asks.
- The inner panel's left half blank,
  as in the no-audio states.

The question stated that once Android stops showing its permission prompt after repeated refusals,
`Allow access` can only open the app's page in Android's settings,
and that at 200% text the state now scrolls slightly on both panels.
`evidence/first-run-access-boundaries.md` holds the study.

No production implementation is authorized by this record.

### D102. The keyboard map is accepted as built (2026-10-10)

Asked whether to accept the whole IntelliJ-aligned keyboard map shown at `questions/keyboard-map.prototype.html`,
the human answered `Accept as built`.
This closes D25's request for one whole-map pass and replaces the F1 draft.
On Windows and Linux, then macOS:

- Search `Ctrl+F` and `Command+F`;
  the folder picker `Ctrl+O` and `Command+O` (both from D25).
- Settings `Ctrl+Alt+S` and `Command+Comma`.
- Jump to the playing track `Ctrl+G` and `Command+L`.
- Previous and next track `Ctrl+Left` and `Ctrl+Right`, and `Command+Shift+[` and `Command+Shift+]`.
- Space toggles play and pause, and `Ctrl+M` cycles the end-of-track mode;
  these and previous and next work only while the playback area has focus,
  so editing fields and focused controls keep their own keys.
- Bare arrows keep their native behavior.
  There is no seek or volume on them,
  so Up and Down have no special meaning after D43 removed in-app volume.
- Escape closes the innermost popup, otherwise leaves Search.
- Media and volume keys stay with the operating system.

Which keys Linux compositors and macOS actually deliver, media-key delivery,
accessibility traversal and IME behavior were not exercised;
the implementation has to verify them with their real owners.
Implementing the map is developer work.
No production implementation is authorized by this record.

### D103. Remaining Search behavior is closed as a design item (2026-10-10)

Asked whether to close the open-questions entry on Search behavior left after D51 and D52,
the human chose to close it and keep the verification notes.
Every design choice in it is decided (D58 to D74).
Native focus restoration, system Back, IME and TalkBack behavior stay listed as implementation gates
in `open-questions.md` and the Search evidence records,
and need verification when an implementation is authorized.
No production implementation is authorized by this record.

### D104. The folded-cover picker is P5 without its container (2026-10-10)

Asked whether to approve the agent's P5 for the folded cover's folder picker,
the human answered:
approve P5, but remove the container,
with no further build and capture round.
The human added that they are still unsatisfied with the picker,
and will deal with that and any further improvement in another major version of the app.

So the design is P4 whose caret points up while the picker is open and down otherwise.
The title is not wrapped in a tonal container,
and nothing else about P4 changes:
the app-bar title and caret open the picker in the list area while the deck stays visible,
and the folder names, letter rail and deck are as D46 and D17 settle.
The open state is shown by the caret's direction and by the picker replacing the track list;
a second marker on the trigger itself was not chosen.
The captures in `questions/cover-picker-revisit.html` show P5 with its container,
so they are evidence of the built candidate and not of this decision;
no capture of the decided state exists,
and opening, closing, Back, focus return and TalkBack are unmeasured for it.
D46 stays a provisional baseline that the human has said they will revisit in another major version;
the before-1.x revisit is closed by this answer.

No production implementation is authorized by this record.

### D105. Move to trash on Android follows what the platform allows (2026-10-10)

Asked how to implement D8's immediate delete with Undo on Android,
where a delete reaches files by two routes that behave differently,
the human chose:
for a folder the user opened through the picker (a persisted storage-access grant)
the app deletes immediately and Undo copies the file back,
and for the device-wide system music library the app uses Android's own trash request with its system confirmation.
The copy-back creates a new document with the same name and bytes,
so its timestamps and identity change;
the system library's Undo is Android's own restore, not the in-app overlay.
D8 stays the design for every source where the platform allows it.

This is a platform-feasibility decision made during production implementation.

### D106. Nearly every action has a keyboard shortcut, chosen as JetBrains would (2026-10-11)

The human asked that nearly every action have a shortcut, for two reasons: better end-to-end testing through real key
input, and a more keyboard-friendly app, and said that when in doubt the choice is whatever JetBrains would have picked.
The orchestrator decided the table on the human's standing instruction to decide remaining questions.
It extends D102, whose bindings all stay.
Windows and Linux first; on macOS the same action takes the Command-based JetBrains macOS binding where D102 gives one,
and otherwise Command in place of Ctrl and Option in place of Alt.

Kept from D102: Search `Ctrl+F`, folder picker `Ctrl+O`, Settings `Ctrl+Alt+S`, jump to the playing track `Ctrl+G`,
previous and next track `Ctrl+Left` and `Ctrl+Right`, play and pause `Space`, end-of-track mode `Ctrl+M`,
`Escape` closes the innermost popup and otherwise leaves the current page.

Playback:
- Seek back and forward 10 seconds: `Alt+Left` and `Alt+Right` (JetBrains navigates back and forward with these);
  60 seconds with `Shift` added.
- Pick a playback mode directly: `Ctrl+Shift+1` to `Ctrl+Shift+4` (Repeat, In order, Shuffle the folder, Shuffle all).
- Restart the track: `Ctrl+Home`.

Browsing:
- Previous and next folder: `Alt+Up` and `Alt+Down` (JetBrains moves to the previous and next member with these).
- Jump to the folder of the playing track: `Ctrl+Shift+G`.
- Focus the folder picker `Alt+1` and the track list `Alt+2`, the way JetBrains opens its tool windows.
- Inside a focused list: `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown` move the focus ring, `Enter` plays the row,
  `Space` on a list keeps play and pause, a letter key moves to the first folder starting with it.
- Open a folder with the system dialog: `Ctrl+Shift+O`.
- Rescan the library: no binding (D113 removed `F5`; the library refreshes by itself).
- Quit (desktop): `Ctrl+Q`.

On the focused track row, or the row the menu is open on:
- Open its menu: `Menu` key or `Alt+Enter` (D114 replaced `Shift+F10`, which is Run in JetBrains).
- Move to trash: `Delete`. Copy path: `Ctrl+Shift+C` (JetBrains copies the path with this).
- Show in file manager: `Ctrl+Shift+E`. File details has no dedicated key (D114: reach it through the menu).
- With the menu open, `Up`, `Down` and `Enter` choose a row and `Escape` closes it.

Pages:
- In Settings, `Enter` opens the focused row; in the template editor `Ctrl+Enter` applies and leaves, and `Escape`
  leaves, and `Ctrl+Shift+Backspace` resets the template to its default.
- In Search, `Up` and `Down` move through the results, `Enter` opens the focused result, `Escape` leaves.

Search also opens on a double tap of `Shift` (JetBrains Search Everywhere): two presses and releases of Shift within
400 ms with no other key between them, on the player page only. Settings and the template editor are modal pages
where Search is not offered, so `Ctrl+F` and double Shift do nothing there (a human correction of 2026-10-11).

Key hints (asked for by the human on 2026-10-11): while the modifiers of a shortcut are all held down, every control
that has such a shortcut shows its key as a small badge on or beside it, for example `F` on the Search button while
`Ctrl` is held, and `Alt+Left` style remainders such as `Left` and `Right` on the seek controls while `Alt` is held;
holding more modifiers narrows the badges to the bindings that use them. The badges appear after the modifiers have
been held for 300 ms, so quick chords never flash them, and disappear when any required modifier is released.
Badges never take pointer input and do not move other content.

Discoverability: the shortcut is shown in each menu row and in the tooltip of every button that has one.
A command palette on `Ctrl+Shift+A` (JetBrains Find Action) that lists every action with its shortcut is wanted and
is a later item, because it is a new screen.

Bare keys never fire while a text field has focus, and the playback keys follow D102's focus rule.
Every shortcut is covered by an end-to-end test that presses the real keys through the nested compositor and reads the
resulting state, which is the second reason for this decision.

### D107. True-peak analysis progress is a system notification, never an in-app banner (2026-10-11)

The human said the analysis banner should never block anything, changed their mind about the bar, and asked for a
system notification with a progress bar, noting that both Android and KDE can show one.
This supersedes D26's 56dp bottom bar and D13's appearance and disappearance of that bar on both platforms:
the player screen draws no analysis indicator and reserves no space for one, so nothing it shows can cover or move
content.

The orchestrator decided the details on the human's standing instruction to decide remaining questions:

- The notification is titled `Analyzing true peak`, its text is the count `412 of 1,218`, and it shows a determinate
  progress bar.
  It is silent, low priority and ongoing while analysis runs, and it is removed when analysis finishes.
- D26's Pause and Resume control stays as the action button on the notification, with the same temporary-pause reading
  that D84 records.
- Android posts it on its own low-importance channel. Android 13 and later need the notification permission: the app
- asks
  for it once, when the first analysis starts, and if it is declined the analysis runs without any indicator.
- Linux desktop on KDE Plasma reports it as a Plasma job through `org.kde.kuiserver` (the job view that Dolphin's
  compress action uses and that the human's own `compress-to-sqfs` service menu script drives): the notifications applet
  shows the app's name and icon, the title `Analyzing true peak`, the count as the info line, a determinate bar with the
  percent, and a native Pause button (the job is suspendable, not killable, because D84 keeps analysis non-optional).
  Where `org.kde.kuiserver` is absent, Linux falls back to one freedesktop notification replaced in place, with the
  progress hint, updated at a limited rate. Where neither answers, analysis runs without an indicator.
  macOS and Windows get the same behavior through their own notification APIs in a later step.
- The taskbar progress of the desktop app (the `LauncherEntry` hint) keeps its own meaning and is not changed here.

### D108. The OS accent is the theme root and every derived color is computed in OKLCH (2026-10-11)

The human confirmed that the desktop has to follow the OS accent as A3 says (it did not: `ui/theme.slint` used fixed
primaries), and stated that every color manipulation after a root color is obtained must be done in OKLCH.
Obtaining the root color is not a manipulation, and it is the only thing taken from the platform on either system:
the desktop root is the portal, Windows or macOS accent that Slint 1.18 exposes, and the Android root is the system
accent
(on API 31 and later the middle tone of the system accent palette, `android.R.color.system_accent1_500`; the human
corrected an earlier reading of this record that kept the platform's own `dynamicLightColorScheme` and
`dynamicDarkColorScheme` roles, so Android's HCT-derived roles are not used).
From the root, on both platforms, everything the app derives (tonal steps, container and on-container roles, hover and
pressed layers,
mixes with neutral, match highlights as in D59, gradients and any lightening or darkening) is computed by moving
lightness, chroma and hue in OKLCH and converting back to sRGB with gamut handling.
It is never computed by sRGB mixing or by HSV or HSL operations, such as `Color.brighter`, `Color.darker` or
`Color.mix` in
Slint, or `lerp` and `copy(alpha)` on sRGB values in Compose.
When no accent is available the fixed baseline from the Fold captures is the root and the same derivation applies.
Alpha-only layering (state layers over a surface) is allowed because it does not change a color's lightness, chroma or
hue.

Retune of 2026-10-12 under B4 (the orchestrator's decision, open to the human's veto): counting the 8 and 12 percent
state
layers, four drawn light-scheme text pairs of the old fixed baseline were below 4.5. The fixed light baseline moved by
lightness only in OKLCH: primary `#4E5E8B` to `#455481` (delta-E 3.46), on-surface-variant `#5C5E6A` to `#585A65` (1.48)
and the search match fill `#CFD9F0` to `#C9D2E9` (2.08). Dark is unchanged. Primary is not fitted on the hovered and
pressed deck-segment fills because primary text is never drawn on them. Rust and Kotlin share one fixture of 52 roots in
both schemes that a test on each side checks.

### D109. The single-pane Settings is one full-width column with 16dp margins (2026-10-12)

The human asked for the stock Android Settings layout to be measured on the owned Pixel 9 Pro Fold with its display size
changed only through Settings (and Developer options' Smallest width as the fallback), then on a Pixel Tablet, and said
that if neither shows two panes the conclusion is that two-pane Settings does not exist.
It was measured with a headful emulator (record in `/home/user/temp/agent/agent-settings-ref3/notes.txt`):
stock Settings of the Play-store Android 17 image shows one full-width column at every size tried, from 320 dp to
1200 dp on the Fold's inner display (default 851 dp), at all five Display size steps, on the folded cover (443 dp),
and at 1101, 1280 and 1506 dp on a Pixel Tablet.
The first conclusion drawn from that, that two-pane stock Settings does not exist, was wrong and is withdrawn:
a later round with the human's instruction to set the large-screen option found that the split is gated by the system
properties `persist.settings.large_screen_opt.enabled` and `persist.settings.large_screen_opt_for_dp.enabled` (either
one; read once per Settings process in `ActivityEmbeddingUtils`), which a production build such as the Play-store image
does not let
a shell set; on a rootable image (`google_apis_ps16k`, API 37.1, `userdebug`) setting one gives two panes at 720 dp
current width and above.
Measurements of that stock two-pane form are in `/home/user/temp/agent/agent-settings-ref3/round4/` (list pane 0.3632 of
the window, rows 72.6 dp, selected row filled in the detail pane's color as a pill of about 27.5 dp corners, no divider,
cross-fade when the selection changes, the detail pane starting on the first destination, a sub-page in the detail pane
gets a Navigate up button) and are the reference for D110, not the target: D110 overrides the ratio on a crease and the
tinted backgrounds.

The music player's Settings page therefore has one form on every window width, following the measured stock layout:
the list is full width with no maximum column width and no centering (this replaces the 640 dp centered column of the
earlier redesign), 16 dp side margin so a card is the page width minus 32 dp, rows 72 dp tall with a 40 dp circular icon
at x = 32 dp and the title at x = 84 dp, 2 dp between rows of one group and 16 dp between groups, with the group corner
rounding already built.
The human then rejected Google's tinted backgrounds ("the non-transparent gray-ish bg is uncalled for; everything should
be in one color, white or black"): the page, the header, the cards and the rows all use the one canvas color of the
player, pure white in light (D34) and pure black in dark (D41), with no separate page tint, card fill or header band
(this removes the `settings-page`, `settings-card` and header-band tokens of the earlier redesign).
Groups and rows are therefore separated by the spacing above and by their text and icons alone, with no fill, border or
divider line (the divider lines were already removed on 2026-10-11), and the group corner rounding survives only as the
shape of the pressed, hovered and selected states.
Fills that carry state or an icon are not backgrounds in this sense and stay: the 40 dp icon circle and the selection
fill
of D110, each a role derived from the accent under D108.
The music player's template rows (D87 and D99) had no icon; the human asked that an icon be picked for each from
Material Symbols and embedded, so every row has the stock leading icon and keeps the 72 dp two-line height.
The orchestrator picked `queue_music` for the Track rows template and `play_circle` for the Playing track template, in
the Outlined style at weight 400 on the 24 px grid, drawn in a 40 dp circle filled with the secondary container role
(D108 derivation) with the icon in the on-secondary-container role.
They are embedded as vector path data in each app, with no icon font, no download and no network access at run time,
the way the app already embeds its other icons; the Material Symbols license (Apache 2.0) text goes under `LICENSES/`
and the README names the source.

### D110. Settings is two panes of half the screen each when unfolded, and its text dodges the crease (2026-10-12)

The human has seen Google's two-pane Settings and does not fully agree with it, and decided that this player's Settings
still takes two panes on the unfolded screen, each exactly half of the screen, with text kept out of the crease.
This replaces D109's "one form on every width" for the wide case; D109's measurements, row geometry and icons stand for
the list.

- Wide means the same condition under which the player shows two panes (the unfolded Fold inner display on Android, a
  window of 600 px or more on desktop). Below that, Settings stays the single full-width page of D109, where a tap on a
  row
  opens that template's editor page and Back returns.
- Wide: the left pane is the Settings list of D109 (heading, rows with icons, 72 dp rows, group rounding) and the right
  pane is the editor of the selected row (D89 to D99), shown in place so selecting a row never navigates.
  On a device with a crease the panes are exactly half of the window each. The human then added that a device without a
  crease does not need that: whether the device has a crease is detected through the platform API (on Android the
  Jetpack WindowManager `FoldingFeature` of a vertical fold or hinge, from `WindowInfoTracker`; the desktop has none),
  and
  without one the split follows the platform default ratio, which the orchestrator reads as the stock Settings value
  `config_activity_embed_split_ratio` = 0.3636 for the list pane that the stock Settings APK ships. The human
  confirmed on 2026-10-12 that 0.3636 is the ratio on a device without a crease; the setting is closed.
  No row or editor moves when the selection changes (the same stability as the Folders panel).
  The selected row is shown with a filled selection shape in the secondary container role plus a bold title (two
  channels, not color alone), and the first row, Track rows, is selected when Settings opens so the right pane is never
  empty.
- Crease: only on a device where the API reports a crease. E2 governs there. Text, labels and other informative marks
- in either pane keep a
  clearance of `max(7.5 mm, crease width)` in total across the center, shared half to each pane, measured in physical
  units and never stored as a fixed dp value; backgrounds, row and card containers, hit regions and the selection fill
  may
  span the center. Where a row's title would reach the band, it ends before the band (ellipsis) and the right pane's
  content starts after it.
  Desktop windows have no crease, so there the split follows the default ratio and the clearance is the pane's ordinary
  16 dp inset. The same holds for a tablet or any other large screen whose API reports no crease.
- Back closes Settings in the wide form and in the single form alike, and in the wide form Escape closes it too.

### D111. Search works while scanning is still running (hard requirement, 2026-10-12)

The human stated it as a hard requirement on Search: it must still be functional while the library is still being
scanned.
It binds both platforms and whatever engine D47 to D74 end up on:

- Search is available from the moment any track is known. It is never disabled, hidden or replaced by a scanning state,
  and the analysis sweep (D84) never gates it either.
- Each keystroke searches every track known at that moment. While the scan keeps adding tracks, the open Search page
  refreshes its results in place, keeping the query, the caret and the scroll position, and without moving a row the
  user
  is about to tap.
- Neither the scan nor the sweep blocks a keystroke: the index takes track additions incrementally in small batches, no
- lock
  or transaction is held across a scan batch, and the per-keystroke latency target holds while scanning, as it does at
  rest.
- The requirement is tested with a scan that is deliberately slow, on both platforms, by typing while tracks arrive and
  checking that results appear for the tracks already known and later for the new ones.

### D112. Search runs on an in-memory index, not on Turso search (2026-10-12)

The human suggested replacing the app's own search algorithm with Turso's built-in search. Both Turso mechanisms were
measured against D47 to D74, D111 and the 1,000,000-track bar (scratch record in
`/home/user/temp/agent/agent-tursofts/`):

- Turso's two ways to search are full-text search (Tantivy, `USING fts`) and vector search. Neither is exact for the
- rules
  without Rust re-applying them: FTS has no custom ranking or rerank hook (only `fts_score` ordering is pushed into the
  index; any other `ORDER BY` makes SQLite sort every candidate, which measured seven to ten times slower than Rust at
  1,000,000), and `fts_highlight` highlights whole tokens only and returns tagged text, not ranges.
  Vector search is similarity, not containment: no query had an exact result at the 500 cap, and exact results needed k
  at or above the match count at 250 to 390 ms per query; the only vector index (`toy_vector_sparse_ivf`) built 200,000
  rows in 209 s.
- Turso FTS with word-start grams does satisfy the rules (parity 44 of 44) and stays usable while a scan adds tracks,
- but
  it still needs the same in-memory table of normalized names, adds a 186 MiB index file, a 26 to 33 s rebuild and
  multi-second write transactions at 10,000-row batches, needs the `turso_core` source patched to compile on the newest
  nightly, makes the Android library 26 percent larger, and is experimental. In 0.8.2 a plain `ORDER BY ... LIMIT n`
  over
  an FTS-matched table returns wrong rows unless written `LIMIT n OFFSET 0`.

Decision (the orchestrator's, on the human's standing instruction to decide, open to the human's veto): Search uses an
in-memory index of normalized names in each app, with first and second character buckets, a best-500 list per first
character, off-thread batch normalization with an append-only write lock, search on a worker thread, and a refresh
signal
after every batch so open results update in place (D111). Turso stays the store of the peak cache only.

### D113. Refresh and re-analysis are automatic and never exposed to the user (2026-10-12)

The human said that refreshing the library and re-analyzing true peaks must always be handled perfectly by the app and
not be exposed as user actions. This supersedes D12's menu entry "Re-analyse true peak" (the last manual entry point)
and
the D106 rows `F5` (rescan) and `Ctrl+Shift+F5` (re-analyse).

- No rescan or re-analyze command exists anywhere in the user interface, keyboard map, menus, tooltips or settings, on
  both platforms. Trash and Undo keep reloading the library internally.
- The library follows the disk by itself: changes are detected and applied without a user action (desktop: the live
  watcher with its catch-up rescan; Android: observers of the source plus a reload when the app returns to the front).
- Analysis follows the files by itself: a new, replaced or edited file (its fingerprint changed) is measured again, a
- file
  that failed to decode is retried when the library next changes or the app next starts, and a failure is reported once
  per file in plain words (D83) instead of offering a retry button.
  Current state (note added 2026-10-14): on Android the report is a system notification, not a D83 message (D123).
- The D107 notification stays the only analysis surface.

### D114. The track menu opens with Menu or Alt+Enter, and every menu row has an access key (2026-10-12)

The human noticed that `Shift+F10` is Run in JetBrains' keymap, which D106 uses as its tie-breaker. Decided by the
human:

- The track row's menu opens with the `Menu` key or `Alt+Enter` (JetBrains' key for the actions on the item under the
  cursor). `Shift+F10` is not bound.
- File details has no dedicated key; the menu reaches it.
- Every menu row has an access key, as JetBrains and desktop menus do: one letter per row, drawn underlined in the row
  label (a second channel besides the letter itself is the underline), and pressing the letter while the menu is open
  runs
  that row and closes the menu. Letters are unique within the menu, chosen from the row's own words (for example Play,
  Shuffle from here, Details, Copy filename, Show in file manager, Move to trash), and the rows' `Up`, `Down`, `Enter`,
  `Escape` keys stay. On Android the access keys work with an attached keyboard.

### D115. Readings made while fixing the list defects (2026-10-12)

The orchestrator's readings of earlier decisions, adopted from the desktop lists work and open to the human's veto:

- D22 and C1: the scrollbar's grab zone is 12 px across the scroll axis (D22), and the thumb is never shorter than 48 px
  along it (C1). The thumb widens from 4 to 8 px on hover or drag, so state is shown by color and width.
- D31: names take the width they need, up to the pane. A longer name ends in an ellipsis, with the full name as a
- tooltip
  (only while it is cut) and as the accessible label. A highlighted Search match is never the part that is cut.
- D1 and D18: the Shuffle segment keeps the width of "Shuffle Camellia" for every folder name, so the deck never
  re-lays itself out while the user browses; a longer name gets the middle ellipsis on the name only.
- D28: each writing system after the first starts with a 1 px hairline in the rail, which also separates look-alike
- Latin,
  Cyrillic and Greek letters.
- D36: the playing row carries a 4 px bar in the primary role on its left edge besides bold and the soft fill, so the
  emphasis does not depend on the script of the title.
- The Hangul syllable that renders as two jamo in some sessions is a font fallback gap, not an app or renderer defect:
  `doc/troubleshooting/slint-hangul-split-jamo-droid-sans-fallback.md`. Open question: bundle a Hangul-capable font.

### D116. No bundled Hangul font, and no optimized test profile (2026-10-12)

Two questions left open by earlier work, decided by the orchestrator on the human's standing instruction to decide,
open to the human's veto:

- No Hangul font is bundled. The app already depends on system fonts for every CJK script (Inter has none), so bundling
  one script would be inconsistent and a full CJK bundle costs tens of megabytes. A session whose fontconfig picks a
  fallback without that syllable shows it as two jamo; the cause and the Slint constraint are in
  `doc/troubleshooting/slint-hangul-split-jamo-droid-sans-fallback.md`.
- `[profile.dev]` and `[profile.test]` keep their defaults. An `opt-level = 1` test profile ran a 771-test suite in
  3.99 s instead of 8.9 s in one sandbox, but made a cold dependency build 12.5 minutes instead of about 4, and that
  cold build hits every fresh worktree and CI run. Revisit if the suite grows.

### D117. A focus ring is only a ring, drawn the way Firefox draws it but hugging the element (2026-10-12)

The human asked that focus rings look like what HTML and CSS draw, except that CSS rings are rectangles and ours hug the
element, that a focus ring be merely a ring with no other change such as a background color, and, when told that browser
default outlines are broken lines, that the app follow whatever the latest Firefox desktop source does. The source
(mozilla-firefox/firefox, `layout/style/res/ua.css` rule `:focus-visible { outline: 1px auto; }`, painted by
`Theme::PaintAutoStyleOutline` in `widget/Theme.cpp`) draws two solid rings, not a broken line. This replaces the
"2 px ring plus the state layer" wording of D106 and the focus fill of the earlier focus work.

- The ring is the pair Firefox paints: an inner ring 2 px (dp on Android) wide in the `primary` role, flush against the
  element's edge (Firefox's default `outline-offset` is 0, so there is no gap), and an outer ring 1 px wide in the
  `on-primary` role around it, which keeps the ring readable on any surface because `on-primary` contrasts with
  `primary`.
- Both rings are solid and follow the element's own shape: the inner ring's inner radius is the element's corner radius
  and each ring's outer radius is the radius plus the widths inside it, so a pill stays a pill, a circle a circle and a
  rounded rectangle keeps its curve; an element with square corners gets a 2 px outer radius, as in Firefox.
- The ring is drawn outside the element, is never clipped by a parent and never moves content.
- It appears for keyboard focus only (CSS `:focus-visible`).
- Focus adds no fill, tint, state layer, bold or size change. Hover and pressed keep their own state layers.
- The human chose Firefox over Chromium on 2026-10-12. Chromium draws the same two solid rings (2 px next to the
- element,
  1 px outside, flush) in a neutral focus color with a white or #101010 outer ring; it was rejected because it carries
  no
  accent and its neutral pair conflicts with D118.

### D118. Grays are always accented unless the root color is completely neutral (2026-10-12)

The human decided that every gray the app draws carries the accent's hue, unless the root color itself has no hue at
all.
This narrows D108's near-gray rule and D41's stable neutrals.

- A root is completely neutral only when it has no chroma at all (its three 8-bit channels are equal). Then every gray
- is
  a true gray. A root with any chroma, however small, gives every gray the root's hue; a nearly gray root is not
  treated as gray.
- The tint of a gray is the role's own tint (it does not shrink when the root is weakly chromatic and does not grow when
  it is vivid); accent roles still scale with the root's chroma.
- Pure black and pure white (the canvas of D109) are not grays and stay as they are. The dark deck, the rail seam, the
  outlines, the surfaces and the text variants are grays and take the tint, in both schemes and on both platforms, from
  the same derivation and the shared fixture.

### D119. Context menus sit on the pure canvas color (2026-10-12)

The human decided that the background of a context menu is pure white in the light scheme and pure black in the dark
scheme, the same canvas color as D109, with no tint. The menu stays distinguishable from the content under it by its
existing boundary (outline or shadow), not by a fill; if it has none, it gets a 1 px outline in the `outline-variant`
role, because a pure black menu over a pure black page would otherwise have no edge.

### D120. Focus ring on filled elements and clipped rows, and the dead page controls (2026-10-12)

The human answered three questions raised by the D117 build.

- An element filled with `primary` (the desktop Play circle, the first-run primary pill) keeps the D117 ring as written:
  the 2 px `primary` near ring merges with the fill and only the 1 px `on-primary` outer ring shows. There is no
  per-element color swap, gap or inside ring for them.
- Rows and fields that sit flush inside a clipped list (track rows, folder rows, Search results, template editor
  fields) draw the same two rings just inside their own edge, because an outside ring would be clipped by the list.
  This is the only exception to "outside the element" in D117.
- The Android page-control chain (`FoldablePageControls.kt`, `LedPageControls.kt`, `PageControlStyle.kt`, the
  `pageTabs` block in `MainActivity.kt`, the session-store style functions and their tests) had no callers and is
  deleted, which removes the last opaque gray literals outside D118's derivation.
  Current state (note added 2026-10-13): the human approved the deletion on 2026-10-12 and `1635237e73` made it,
  but that commit left `LedPageControls.kt` as a 24-line stub holding `ledSelectedFill`. The stub is gone too, and with
  it the Compose OKLCH adapters in `OklchComposeColor.kt` (`mixOklchWithNeutral`, `OklchNeutralMix`, `withOklchAlpha`,
  `oklchGradientStops`, `OKLCH_BLACK_LIGHTNESS`, `OKLCH_WHITE_LIGHTNESS`), which only the stub and `OklchColorTest.kt`
  still called; the adapters and that test file are deleted. The only OKLCH code left in the Android app is
  `core/OklchColor.kt`, called by `core/ThemeRoles.kt`. The page-control runbook, fold plan and handover under `doc/`
  are deleted for the same reason, and the Android README no longer describes the styles. Desktop has had no
  page-control styles since the `AppWindow` replacement of 2026-10-10.
- D118 as built also floors accent chroma at the largest gray tint of the scheme for every non-neutral root, so no
  accent is grayer than a gray; this is the orchestrator's reading and stays open to the human's veto.
- Update, 2026-10-14 (D130): the first build of the flush rows' ring started the box at the crease clearance and held
  the text 8 dp from the ring, which pushed row text right of the heading column beside the crease. The box spans the
  pane and the text takes the heading column; D130 states the rule.

### D121. Closed veto windows: accent floor, AA retune, Turso bugs, targetSdk (2026-10-13)

The human answered four questions that had been open as orchestrator adoptions.

- The accent chroma floor of D118 stays: for a non-neutral root, accent chroma is at least the largest gray tint of the
  scheme. A completely neutral root keeps accents at chroma 0.
- The D108 light retune stays (primary `#4E5E8B` to `#455481`, delta-E 3.46; on-surface-variant `#5C5E6A` to
  `#585A65`; match fill `#CFD9F0` to `#C9D2E9`), so the D108 veto window is closed.
- The two Turso bugs written up in issue 678 are not filed upstream for now; nothing is published under the human's
  identity until they ask. D112 (in-memory index) stands.
- Android `targetSdk` stays on 36 with no extra emulator passes for API 33, 34 and 36; D107 on those levels remains
  unverified.

### D122. The report of a file that cannot be played is a system notification, not an overlay (2026-10-14)

The human asked for it: the in-window message "`name` can't be played." appeared at launch for a library file that could
not be decoded, and it was misplaced on screen.
The request for the system notification is an owner decision; the bullet marked as the implementer's reading is the
only part left open to a veto.

- D83's overlay no longer carries the unplayable-file message.
  The window draws nothing for the report; the row keeps its "Can't be played" mark, which stays as the only in-window
  sign (D113's mark and its removal when a peak is measured are unchanged).
- Why: the report happens at launch and during sweeps, not in answer to anything the person just did, so it belongs in
  the desktop's notification area (D107's reasoning for the analysis indicator) and not on top of the player.
  D113's rule of one report per file in plain words stands; only the surface changes.
- Linux sends one `org.freedesktop.Notifications.Notify` call per report: the app's name, icon and desktop entry, the
  fixed title `Cannot play a file` (it names no file and no folder), the old wording as the text (one file named;
  several collapse to a count that names the first, D9), normal urgency, the server's default expiry and no action
  buttons.
  Files in one engine report give one notification.
- The implementer's reading, open to the human's veto: each file is announced once for a run of the app, so a repeated
  report, a folder opened again or a file that decodes and then breaks again posts nothing more, while its row is marked
  each time. The human asked only that a repeated report not repeat the notification and that the engine's once-per-file
  rule stay; the engine itself still reports again after a folder is opened again (which re-marks the rows) and after a
  file decoded once. Without this reading those cases would post the notification again.
- The call runs on its own thread with its own session bus connection and a bounded wait, so the engine and the window
  never wait on the bus, and a failure here never disables the analysis indicator (D107) or the other way round.
- When the notification cannot be posted (no bus, no notification service, any failure) nothing is shown in the window:
  one warning is logged and the row mark is the only sign.
  There is no in-window fallback in any case.
- Platform limit: macOS and Windows have no notification code yet (the analysis indicator has none either, D107 leaves
  it to a later step), so there the row mark is the only sign and the unplayable-file message is not shown at all.
  This was not run on either system.
- What stays an overlay: every other D83 message, including the removed-file message (`file_gone_message`) and the
  file-action errors.
- Android is handled in D123.
### D123. On Android the analysis-failure message is a system notification, never an in-app message (2026-10-14)

The human decided that only the report of files whose true-peak analysis failed (D113) leaves the app's own message
overlay (D83) on Android; the desktop app makes the same move under D122 on its own branch.
An analysis failure is an event the human did not ask for, and it can appear while they are in another app, so an
in-app message that waits for a screen (the D113 hand-off) was either missed or arrived long after the sweep.

- The service that ran the sweep posts one system notification when the failed files are first reported, the same moment
  the in-app message used to appear. Several files in one report give one notification.
  A file keeps the D113 once-per-file rule (the sweep ledger tells the first failure from a repeat), so a repeat
  failure posts nothing.
- The wording is the old message word for word: `Could not analyze <file>.` as the title and
  `Its file could not be decoded. The app tries again when your library changes.` as the text; several files collapse to
  `Could not analyze 3 files, including <first>.` with `They could not be decoded.` and the same retry sentence.
  The text is shown expanded, so the retry sentence is not cut off.
- It is a separate notification on its own channel (`analysis-failure`, "Analysis problems") with its own id, never the
  D107 progress notification, which is neither replaced nor removed by it.
  The channel has low importance, which Android words as "Makes no sound": the event is not urgent, so no sound and
  no heads-up, and the notification waits in the shade.
  It has the app's small icon, cancels itself when tapped, has no action buttons, and tapping it opens the app.
  A later report replaces an earlier one that is still unread, as the in-app message did.
- What stays in the app: the tap-time message `Could not play <file>` (the track the human just tapped could not be
  opened) is a reply to their own action and stays an in-app message exactly as in D83.
- No fallback: when the notification cannot be posted (the notification permission is denied on Android 13 or later,
  notifications for the app are turned off, the channel is turned off, or the post itself fails), the app does not show
  the message in the app. It writes one warning line to the log (the reason and the number of files, no file names) and
  shows nothing.
  The failure then reaches only the log, and the failed file keeps no mark on its row on Android either.
  A failure to post never crashes the sweep or blocks it.
- Consequence: a person who declined the notification permission at the D107 prompt never hears about an undecodable
  file, as they never see the progress indicator either.

This amends D113's statement that a failure is "reported once per file in plain words (D83)": the report is now this
notification, still once per file.
### D124. A row click or tap selects, the selected row toggles play (2026-10-14)

The human decided this on 2026-10-14, after asking that clicking a track always select it when it is still unselected,
and that clicking a selected track toggle play and pause.
It reverses the earlier README text that said a click on a row other than the playing one, and a tap on any row on
Android, played it at once.

- A click, a tap or Enter on the focused row of a track that is not the current track only selects it.
  The row becomes current, the deck shows it, the track is loaded paused at its start, and any playing track stops.
  The desktop `Command::SelectIndex` always did this; its UI wiring `play_row` followed it with `Play`, and that
  follow-up is removed.
  Android gets `PlayerController.selectIndex` (queue cursor, engine pause, paused load).
- A click, a tap or Enter on the current track toggles play and pause, whether it is playing or paused.
  Android calls the deck's `togglePlay`, so a current track that was restored from the saved session and never started
  starts, and a current track whose load failed is loaded again on the next tap.
  Desktop first differed on a failed load: its `load_current` skipped an unreadable file and reported it, so selecting
  such a row moved the selection to the next playable file. The human ruled otherwise the same day, and D128 makes
  desktop keep the unreadable row selected like Android.
  Android's `PlayerController.activateRow` holds the whole rule, and the host only forwards the tap.
- The Play item of the track menu keeps starting a track at once (desktop `play_row_now`, Android `playIndex`);
  it is an explicit Play, not a row click.
  Search results were left open when this decision was taken; the human ruled the same day that they follow this rule
  (D127).
- The folder and letter rails, long press and the right click are not track-row selects and are unchanged.
- A tap during the first library load keeps its choice as before, because `selectIndex` loads the track, and the
  loaded track is what `finishLoad` keeps.
- D129 adds the Auto-play on select setting: with it on, selecting a track that is not current also starts it. The
  rule above is the default.
### D125. The folder names list shows no scrollbar (2026-10-14)

The human asked that the scrollbar on the folders panel go, because the panel has several other ways to say where the
user is. This amends D22: the desktop folder names list, the column right of the letter rail, draws no scrollbar on
either input, neither the always visible draggable bar for the pointer nor the fading bar for touch.

- The panel already shows position without a bar. The selected folder is primary, heavier and carries a 2 px bar on its
  bottom edge (D31). The letter rail (D17) circles the letter of the current folder's cell, and the names run in rail
  order under a heading for each cell (D28), so the heading in view says where in the alphabet the list is. The list
  order is the alphabet itself, and a scrollbar would add a length that D17 and D23 say the user is not meant to care
  about.
- What stays for moving the list: the wheel, a touch drag, the keyboard (Up, Down, the page keys, Home and End scroll
  the ringed folder into view), a rail tap that scrolls to its cell, and the reveal of the selected folder after Search,
  a shuffle or a folder key chord (D79). None of them changed.
- The names keep the 16 px right inset of the Android list (`NAMES_END`), so names wrap and end where they did before.
  Only the 12 px grab zone constant and its comment went.
- Other lists keep D22: the track list, Search results, Settings and the template editor still draw the always visible
  12 px bar for the pointer and the fading 4 px bar for touch. The rail already had none.
- The folder names still record whether the last pointer event was a touch, because the bars of those other lists
  follow the last input the app saw.
- Android needed no change: `FolderPickerPane.kt` draws no scrollbar and no scroll indicator on the names.
- Tests: two window tests in `desktop-app/src/ui_lists_window_tests.rs` assert that the picker has no scrollbar in the
  wide and the folded layout on both inputs, while the track list keeps its own bar in the wide layout.

### D126. The Search page is restyled on Android within the earlier decisions (2026-10-14)

The human accepted option 1 of `doc/planning/android-search-page-refresh.md` ("Restyle within decisions") after the
proposal compared the Search page with the stock Search screen of Android Settings, and ruled on its conflicts.

- Direction: restyle our own composables inside the accepted decisions, with no new color role and no new library
  component. R2 of the proposal is adopted as part of it: the header is the canvas color, not the
  `surfaceContainerHigh` band, and the divider uses `outlineVariant`, not `outline`. D48 keeps its anatomy (72 dp
  header, Back, query, Clear, divider); only the fill and the divider role change.
- No section labels: results stay one list mixed by relevance (D52, D61). The grouped rounded cards and the tinted page
  of the proposal's option 4 are not adopted (A2, D109, B1).
- No recents list on the empty page (D66, D70 stand).
- Desktop parity is wanted. The Slint port is a separate follow-up task; this change touches no desktop file. The
  specification below is what the port copies.
- Not diagnosed items of the proposal, read from the code and not changed (D66 covers a new visit, not a posture
  change): the query survives a fold change because `PlayerHost` keeps it with `rememberSaveable` next to the route,
  and a second `lan` typed after `Ctrl+F` landed between the two because `Ctrl+F` on an open Search page only refocuses
  the field (`focusSearch`), and the field's caret restarts at the beginning of the text whenever the page rebuilds the
  header for the other panel (`BasicTextField(value: String)` keeps its selection in a `remember` that the layout
  switch drops). The emulator reproduced both: `lan` survived a fold, and `lan` typed again after `Ctrl+F` gave
  `lanlan`, then `x` gave `lanxlan`, which places the caret at index 0. Both are consequences of earlier designs, not
  violations of D66, so they were reported and left open.
- Update, 2026-10-14, same day: the human asked for the caret to be fixed. The query stays kept by the host across a
  fold change and D66 still empties it for a new visit. A header rebuilt for the other panel starts the field with the
  caret at the end of the kept query, the caret the person placed survives (also across an activity recreation), and
  `Ctrl+F` on an open Search page puts the caret at the end and selects nothing. The desktop query field was not
  examined or changed.
- Update, 2026-10-14, after review: a first version fed the field a `TextFieldValue` and moved the caret in an effect
  after `Ctrl+F`. A character typed right after the chord still landed first: on the emulator the chord was handled at
  0 ms, the typed character reached the field at 7 ms (at the old caret, giving `ylan`), the first recomposition at
  13 ms, the effect that placed the caret at 22 ms and the recomposition showing it at 27 ms. A field fed a
  `TextFieldValue` can only take a new value at composition, one frame at the earliest, so no composition-based
  placement could win against a key 7 ms behind the chord. The field is now a `TextFieldState`, whose edits apply at
  once: the key handler presses a `SearchFocusPort`, and the open page has registered what that does (caret at the
  end, nothing selected, focus, keyboard) before the next key is read. Three consequences follow. The field is the
  source of the query while the page is open, and the host's query only seeds it (edits are mirrored to the host). A
  field saved by an earlier layout must not overwrite a newer kept query (the activity is recreated on every fold
  change and Compose saves an unused restored entry again, so a stale `lan` came back over `lanx`), so
  `restoreOnlyWhileHolding` restores a saved field only while its text equals the query the page started with.
  `SearchFieldCaretWiringTest` pins all of it, including a press and a typed character in one event and with the
  frame clock held.

What stays: D58 columns (Back center and result glyph center share one x, query start and title start share one x),
D59 highlight, D63 to D80 focus and keyboard behavior, D66 and D70 copy, D76 (not a focus trap), D50 and D51 layout
(the deck stays visible and Search is the right half of the unfolded panel), the shared OKLCH roles and the 52-root Rust
and Kotlin fixture (`ThemeRoles.kt`, `theme_roles.rs` and the fixture are untouched), and true-black dark (B1).

#### Specification for the desktop port

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

Verification of the Android side, with the commit's tests: pixel and geometry cases in `SearchPageWiringTest` (header
and status strip pixels equal the canvas in both schemes, divider pixel is `outlineVariant`, the 40 and 72 circles,
the keyboard inset, D58 columns on both panels, the ring inside the rounded row) and `SearchLayoutTest`, and a
headful emulator pass on the Fold (cover, inner panel, light, dark, keyboard open, Tab focus, 200 percent text).

#### Desktop port (addendum, 2026-10-14)

The Slint Search page (`package/music-player/desktop-app/ui/search.slint`) now copies the specification above, on the
branch `feat/desktop-search-restyle`.
The addendum records what was ported and the choices the specification left to the desktop, so no new decision number
is used.

Ported as specified:

- Header filled with `Theme.canvas` (the `Theme.surface-high` fill is gone), 1 px divider at y = 72 in
  `Theme.outline-variant`.
- Each result row is flat on the canvas. A rounded shape (radius 21.3) spans `content-start-x - 8` to the page width
  minus 8, carries the hover and pressed layer (a `StateLayer` with the same radius), the D117 ring pair (a
  `FocusRing` with `inside` and the same radius, so the 1 px outer ring is on the shape's edge and the 2 px near ring
  inside it, following the corners) and the pointer target.
  The glyph and the title keep their D58 columns (the glyph centered on the Back arrow's x, the title on the query's x).
- Row icon: a 40 circle in `Theme.secondary-container` centered in the 48 slot, with the 24 glyph in
  `Theme.on-secondary-container`.
- The five status states (empty query, no match, scanning, unavailable, empty inventory) share one block that starts 72
  below the divider's bottom edge on the cover and the inner panel: a 72 circle in `secondary-container` with the 32
  magnifier in `on-secondary-container`, the title 24 below it (`Theme.title-medium`, weight 500, tracking 0.15 px, a 24
  line), and the support line (`body-medium`, `on-surface-variant`, a 20 line) 8 below the title on the cover and 12 on
  the inner panel. The cover centers the block with 16 on each side; the inner panel starts the circle at
  `content-start-x`.
- The block sits in a `Flickable` whose content is as tall as the block plus its 72 offset, so a window shorter than
  that scrolls it with the wheel and a taller one does not move.

Desktop-specific choices and facts:

- The pointer target is the rounded shape's box, as on Android (the 8 px on each side of the shape no longer take the
  click). The target is a rectangle, so the four corner pixels outside the arc still take a click; the hover layer, the
  ring and the fill follow the arc.
- The scrollbar's 12 px zone starts 12 px from the page edge and its 4 px track 8 px from it, which is exactly where
  the shape ends, so the track never touches the ring at rest. The 8 px thumb of a hovered or dragged bar reaches 2 px
  into the shape's last pixels and can cover the ring's right edge for the length of the thumb, as it did for the old
  square ring.
- The hint badge clause has nothing to keep: the desktop result rows have no key hint badge (`ui/search.slint` has none
  and the page shows no badge over its rows), so no slot was added. A future badge belongs inside the shape's 8 px end
  padding.
- The crease sentence does not apply on desktop as built: `ui/app_window.slint` never sets `content-start-x` on the
  Search page, so both layouts use the default 16 and the inner panel's page already starts at the pane's left edge
  (half
  the window width), after the crease.
- The sentence that the kind label stays on the glyph for assistive technology (D77) has nothing to keep on desktop:
  the desktop result rows and their glyphs expose no accessible role or label today (the only accessible elements of
  `ui/search.slint` are Back and Clear), before and after the port. This predates the port and was left open.
- The status title keeps eliding on one line at a narrow width, as before; Android wraps it. The title weight is the
  500 instance of the variable Inter font (`ui/fonts/InterVariable.ttf`), and `letter-spacing` is a Slint `Text`
  property (checked in the `i-slint-compiler` 1.18.1 `builtin_elements.rs`).
- The result list's scrollbar is drawn for any non-empty result list, scrollable or not, because `search.slint` sets
  `visible: root.rows.length > 0` on it, which replaces the bar's own `scrollable` rule. This predates the port and was
  left alone.

Verification: `ui_search_restyle_tests.rs` (thirteen window tests on the software renderer, light and dark, cover and
inner panel: header and divider pixels, row and circle geometry and roles, the ring pixels in the near band, the hover
layer clipped to the shape, the block geometry of all five states, scrolling, and the square highlight, columns and
copy that stay), the updated row case of `ui_focus_ring_pages_tests.rs`, and a before and after capture of the release
build in the nested compositor (empty, `lan`, no results and the keyboard ring, light and dark, 1000 by 800 and 443 by
900).
Not verified: macOS, Windows, a real Plasma session, touch input.

### D127. Search results follow the row click rule (2026-10-14)

The human decided this on 2026-10-14, through the question tool, after D124 had left Search open: Search results follow
the same rule as the track rows.

- Activating a Search result that names a track, by a click, a tap or `Enter` on the ringed result, and that is not the
  current track, only selects the track.
  It becomes the current track, the deck shows it, it is loaded paused at its start, and any playing track stops.
  The browser still jumps to the track's page as before (D74: the page returns to the player with the owning page
  selected), and the Search query still ends on the return (D66).
- Activating the result of the current track toggles play and pause, as a click on the current row does.
  Starting a track from Search therefore takes two activations, the first to select and the second to play, as for a
  row.
- This changes D73's first paragraph, which said a result for a different track starts that track at once.
  D73's current-track toggle, D72 (a folder result opens its folder without autoplay) and D74's return to the player
  stand.
- D129: with Auto-play on select on, the result of a track that is not current is also started; this is the default
  behavior of D127 only while the setting is off.
- The explicit Play item of the track menu stays play-at-once on both platforms (desktop `play_row_now`, Android
  `playIndex`).
  It is an explicit Play, not a click on a row or a result.
- The spoken action of a result for a track that is not current was "Play track" (D77), which no longer describes the
  action.
  It is now "Select track" on both platforms; the current track keeps "Pause track" while it plays and "Play track"
  while it is paused.
  This follows from the ruling and is the implementer's reading, open to a veto.
- Code: Android `SearchActivation.PlayTrack` became `SelectTrack` and `PlayerHost.openSearchResult` calls
  `PlayerController.selectIndex` (the current-track case still calls `togglePlay`); desktop
  `ui_search::activate_result` calls `ui_wire::play_row`, the click rule, and `play_row_now` serves the menu Play only.
- Tests: `PlayerHostModelTest` (the decision), `PlayerHostWiringTest` (a tap on a result through the real host selects
  paused, the second tap plays), `ui_search_activation_tests.rs` and the Enter case of `ui_keys_window_tests.rs`
  (the commands the desktop sends).

### D128. An explicit select keeps an unreadable row selected on desktop (2026-10-14)

The human decided this on 2026-10-14, through the question tool, after D124 had left it open: desktop behaves like
Android when the file of a clicked row cannot be opened.

- A click, `Enter`, a Search result or the menu Play on a row whose file cannot be opened makes that row the current
  row.
  It stays selected and paused, nothing is loaded (a playing track stops and its decoder is dropped, so a later toggle
  cannot resume it), and the window is told the row is current with no length.
- The failure is reported once through the existing path: the row's "Can't be played" mark (D113) and the system
  notification of D122, which already posts once per file for a run of the app.
  A retry that fails again reports nothing more and leaves the mark where it is.
- The next click on that current row sends the toggle (the window knows the row is current), and the toggle loads the
  row again.
  If the file decodes now, it loads and plays, and the mark goes when a peak is measured for it (D113).
  The Play button does the same when a row is selected and nothing is loaded.
- Only explicit selects stay on the chosen row.
  Playback-driven moves keep walking past unreadable files and report them: the natural end of a track, Next and
  Previous.
  A launch or a folder open that preselects a file, the restore of the last session, the stop at the end of a folder,
  and a Play with nothing selected (the first track of the shown page) also keep the walk, because none of them is a
  choice of one row.
- Code: `Controller::load_selected` (`controller_flow.rs`) opens only the current row's file; `Command::SelectIndex`
  and `start_playing` with a selected row use it, and `load_current` is unchanged.
- Known gap, not changed: the controller forgets a file's report only when a sweep measures a peak for it
  (`forget_reports_of_decoded`), not when the playing path measures one.
  A file that was reported, is replaced by a readable one through a retry, and breaks again later is therefore not
  announced a second time in the same run.
- Tests: `controller_flow_tests.rs` (an unreadable select stays and drops the old decoder, the second click retries and
  reports nothing more, a readable file loads and plays and gets its peak, the Play button stays on the row, Next,
  Previous and the natural end still skip).

### D129. Auto-play on select: a setting that brings back play-on-select, default off (2026-10-14)

The human asked for "a setting to auto play on select, default to off".
The meaning, the Settings row and the stored form below are the implementer's reading of that one line, open to a veto.
It amends D124 and D127, which stay as the rule for the default.
D133 (2026-10-22) later changed the default to on; this record stays as the decision was taken.

- With the setting on, a click, a tap or `Enter` on a track row that is not the current track selects it and starts
  it at once, and so does activating the result of such a track in Search.
  This is the behavior before D124.
- With the setting off, which is the default and what an unmigrated or missing value means, D124 and D127 apply as
  written: the track is only selected.
- Unchanged in both modes: the current row (or result) still toggles play and pause; the explicit Play item of the
  track menu still plays at once; the folder and letter rails, long press and right click are not track selects; an
  unreadable selected row stays selected and is reported once (D128), and with the setting on it simply does not start.
- The setting is one pure rule on each platform, inputs the setting and whether the row is current, output select only,
  select and play, or toggle: `rowActivationFor` in `RowActivation.kt` on Android and `row_activation_for` in
  `row_activation.rs` on desktop.
  Android `PlayerController.activateRow` takes the setting from the host (tap, `Enter`, Search result); desktop
  `ui_wire::play_row` reads it from the screen state (click, `Enter`, Search result), and `play_row_now` forces it on
  for the menu Play.
- Stored in each platform's existing settings storage, outside the template texts: desktop in `session.json` as
  `auto_play_on_select` (a missing field is off, an old file loads unchanged, and the shutdown snapshot keeps the stored
  value); Android in the `session` preferences under `auto_play_on_select` (a missing key is off).
- The Settings page gets a third row in its own group headed "Playback", after the two template rows, in the stock
  Settings switch-row form: 72 dp row, 40 dp icon circle (Material Symbols Outlined `autoplay`), the title
  "Auto-play on select", one supporting line, and a trailing Material 3 switch (52 by 32, 24 dp thumb).
  It toggles in place on click, tap, `Space` or `Enter` and never opens an editor or changes which template is
  selected; on an unfolded window the detail pane keeps the selected template's editor.
  The group heading's 16 dp top padding is the gap between the two groups (D109).
- The state is shown in more than one way (CXD): the thumb sits right when on and left when off, the track is the accent
  when on and the canvas with an outline border when off, the thumb holds a check mark when on and a cross when off, and
  the supporting line names what selecting does, "Select and play at once" or "Select first, then Play".
  Colors are existing roles only (`primary`, `on-primary`, `outline`, canvas); the dark scheme stays true black.
- The spoken action of another track's Search result follows the setting: "Select track" when off (D127), "Play track"
  when on.
  The switch row is announced as a switch with the title as its label and the state as its value.
- Keyboard: the switch row is the last stop of the list (Down, Up, Home and End, Tab on Android), with the focus ring of
  D117 on its group-rounded shape; reaching it selects nothing.
- A narrow list pane cannot hold the title beside the switch (the pane is 218 px at 600 and 309 px at 850, and about
  213 dp on the unfolded Android screen), so below 300 px or dp of row width the switch sits under the supporting line,
  and the title and line wrap instead of being cut.
  At the 443 dp cover and in the wide pane the switch stays beside the text.
- Tests: the pure rule on both platforms, the stored value (default off, missing field, round trip, old file), the
  Settings switch row in real windows (default, click, `Space`, `Enter`, the state it draws, the unfolded detail pane,
  the ring), the click, `Enter` and Search paths with the setting on and off, and the desktop end-to-end session
  `auto-play` (off by default, `Space` on the row turns it on, a click plays at once, and the setting survives a
  restart).

### D130. Beside the crease only text dodges it; fills, state layers and rings may reach it (2026-10-14)

The human ruled it after a screenshot of the inner display showed the track rows' text 11 dp right of the subfolder
headings and the library title, while the cover display (no crease) showed all of them aligned.
It states the rule that D58, D110 and D120 already imply for flush rows, and corrects how D120's ring was built.

- Text columns win. In one pane the title bar, the subfolder headings and the row text start at the same x, which is
  the larger of the ordinary 16 dp inset and the pane's crease clearance (`paneTextInsetDp`, half of
  `max(7.5 mm, crease width)`, 22.64 dp on the Fold inner display at density 390 when the crease reports no width).
  The end side is the mirror in the left pane.
- Beside the crease only text dodges it. The row's box (the current row's fill, the press and hover layers, and the
  D120 focus ring just inside the box edge) is not text and may reach the crease and the pane's edge, so it spans the
  pane edge to edge on every display, as the fill always did on the cover. No clearance applies to it.
- The cause: the D120 ring work started a flush row's box at the pane clearance and kept at least 8 dp between the ring
  and the text (`ROW_RING_TEXT_START_MIN`, 11 dp from the box edge). Beside the crease the clearance already equalled
  the text inset, so the text was pushed right by that minimum, to 33.64 dp (82 px at density 2.4375) while the
  headings sat at 22.64 dp (55 px), and the fill began at the clearance instead of the pane edge.
  Measured from pixels on the emulator, glyph edges stood 84 px and 56 px from the pane edge, and the fill began 55 px
  from it.
- With the box at the pane edge the ring's gap to the text is 13 dp on the cover and 19.64 dp beside the crease, so
  the 8 dp minimum between the ring's inner edge and the text holds with no clamp. `trackRowColumns` keeps the floor
  (ring depth plus gap) for an ordinary inset below 11 dp, so the text never moves off the column while that inset is
  16 dp.
- The same rule fixed the template editor's field rows in two ways. Their 3 dp padding for the ring put their text at
  19 dp while the section headings stayed at 16 dp, with or without the crease inset; the ring lies inside the row edge
  (D120) and needs no padding. And the crease inset padded the whole scrolling column, so a row's box began at the
  inset; each text now takes the inset itself and the field row applies it after its clickable, so the box spans the
  pane. The hairlines between the rows are not text and now start at the pane edge too.
- Audited and unchanged: Search result rows (D126, D58), whose rounded shape starts 8 dp after the clear zone by the
  human's earlier decision and whose content already sits at the header's column (measured on the device: query text
  and result titles 1 px apart), so they are a candidate for this rule only if the human wants the shape to reach the
  crease.
  Search's own column is half the crease plus 16 dp plus the 48 dp glyph slot, which is not the player pane's inset, and
  this change does not touch it.
  Also unchanged: the folder names (outside ring, text column at 12 dp plus the 4 dp name padding, in the left pane,
  which has its crease clearance at the end only), the letter rail cells (ring inside a circle, no crease beside them)
  and the Settings list rows (the fill spans the pane and only the text end takes the extra inset, D110).
- Code: `core/TrackRowGeometry.kt` (`trackRowColumns`), `TrackListPane.kt`, `TemplateEditor.kt`.
  Tests: `TrackRowGeometryTest`, `RowAlignmentWiringTest` (bounds and pixels of the real pane) and
  `TemplateEditorAlignmentWiringTest`.

### D131. On Android a track the player moved to by itself and cannot open is walked past and reported by notification (2026-10-18)

D128 settled it for the desktop (playback-driven moves keep walking past unreadable files and report them), D122 and
D123 put those reports in the system notification area with no in-app fallback, and the tap-time message stays in the
app.
Android had none of the walk: every failed load, however it started, set the one in-app "Could not play" message and
stopped.
The human's rulings are applied here; the readings below are the implementer's, open to a veto.

- The origin of a load decides.
  A tap, Enter, a Search result, a menu Play and the Play button on a current row are taps: the in-app message, the
  row stays selected, nothing moves on.
  The natural end of a track, Next, Previous, a Play with nothing selected, the restore of the last session and the
  readied first track at the end of a folder are moves: the player passes the file and loads the next one the move
  would have reached, with the same play or paused intent, and the in-app message is never shown.
- Direction follows the move: Previous walks backward, the natural end walks the way a natural end goes (an In order
  folder still ends and readies its first track paused, Repeat steps on to the next track), the others walk forward
  and wrap like Next.
  The walk gives up after one try per track of the queue, paused.
- A timeout of the descriptor open ends the walk at once instead of trying the next file, because a music library that
  does not answer would make each next file wait the 8 s limit; the report then says the library did not answer in
  time.
- The report is one system notification on the existing `analysis-failure` channel ("Analysis problems") under its own
  id (4203), so it does not replace an analysis-failure notification (4202) and vice versa; a later report replaces an
  earlier one of its own kind.
  The text is the tap message's words as a title and a text:
  `Could not play <file>.` with `Its file could not be opened.`,
  and for several files `Could not play 3 files, including <first>.` with `They could not be opened.`
  The report names every file of the walk and grows with it; the service posts it 400 ms after the last failure, so a
  walk past many files is one notification and the system's limit on notification updates cannot drop the last one.
- No fallback, as D123: with the permission denied, notifications off or the channel off, one warning without file
  names goes to the log under the tag `UnplayableFiles` and nothing is shown in the app.
- Not decided here: the channel is still named "Analysis problems" in the system settings, which no longer describes
  the unplayable report; renaming it is a separate choice for the human.
  A file already reported is reported again when a later walk meets it (D122's once-per-run reading is not applied).
- Code: `PlayerController` (`loadTrack`, `walkPastUnplayable`), `core/UnplayableWalk.kt` (`LoadOrigin`,
  `UnplayableReport`, the attempt cap), `core/AnalysisFailureNotice.kt` (`unplayableFilesNotice`),
  `AnalysisFailureNotification.kt` (`postUnplayableFilesNotification`) and the wiring in `PlaybackService`.
  Tests: `PlayerControllerUnplayableWalkTest`, `AnalysisFailureNoticeTest`, `AnalysisFailureNotificationTest`.

### D132. Settings gets an About group with one Licenses row on both platforms, designed without mockups (2026-10-18)

The human decided through the agent's question tool: "add a licenses row to settings, doesn't need mockups, just do it,
this one is pretty simple and low-impact in terms of design".
The owner decision of the same day to use Slint under its Royalty-free license
(`doc/decision/music-player-third-party-licenses.md`) is why the row exists:
that license asks for the AboutSlint widget in an About screen.
No mockup, candidate set or native study was made, by that decision; this record and the built screens are the whole
design.
It adds a row to a page whose rows D87, D97, D99 and D129 name, so it is recorded here as the next row of the page.
The readings below are the implementer's, open to a veto.

- The Settings page ends with a third group, headed "About", after Templates and Playback.
  The heading has the form of the other two group headings.
  The group holds one row, Licenses, in the stock 72 px or dp row form, with the 40 px or dp icon circle of the other
  rows and group rounding at all four corners.
  Like the Playback switch row it is never a selection: the selected template and the editor beside it stay as they
  are.
- The icon is the Material Symbols Outlined `license` (weight 400, grade 0, size 24), from the same pinned commit as
  the other Settings icons, on both platforms.
  The first builds drew `license` on desktop and `description` on Android; the agent unified them on `license` on
  2026-10-18, which the human has not been asked about and may veto.
- The supporting line is each platform's own wording, not unified: "Third-party notices" on desktop and "Open-source
  libraries used by this app" on Android.
- The row is a button for assistive technology and the last keyboard stop of the list, after the Playback switch;
  Enter and Space open it on both platforms.
  The focus ring follows the row's group rounding (D117).
- The desktop page (`ui/licenses.slint`) covers the whole window in both the narrow and the wide form, drawn on the
  one canvas color with no band, card or divider.
  Its header holds the Back arrow (spoken "Back") and the title Licenses.
  Under the header sits the Slint "Made with Slint" widget (`AboutSlint`, 112 px band, the light or dark logo by scheme,
  a click opens slint.dev), then a "Notices" heading, then the notice text in a list that builds only the lines in view.
  Back and Escape return to Settings, which reopens with the Licenses row as the keyboard position.
  The arrow keys, Page Up, Page Down, Home and End scroll the text; the keys that hold a modifier pass on.
- The Android page (`LicensesPage.kt`) takes the window on every width, also on the unfolded screen where Settings has
  two panes and a row shows its content in the right pane (D110).
  The text is a 1.7 MB document, not a setting, and it takes the window (the right half of a creased screen, so no text
  crosses the fold).
  Putting it in the right pane instead is an open choice.
  The header is the Settings header with a Back button spoken "Back to Settings" and the title Licenses.
  A `LazyColumn` draws the notices in blocks: titles carry the heading role and differ by size, weight and color,
  wrapped prose is joined into paragraphs, other lines keep a monospace face.
  A spinner with "Loading licenses" shows while the asset loads, and a message shows if it cannot be read.
  Back (the button or the system gesture) returns to Settings, and on the unfolded screen Escape does too.
  The list is one keyboard stop; Up, Down, Page Up, Page Down, Home and End scroll it.
- Not decided here: whether the row should carry the name About, or the page a title that includes it; and whether
  the desktop page counts as the "About" screen "accessible from the top level menu" of section 2(a) of the Slint
  license (the release notes carry the Slint badge for section 2(b) so the answer does not decide whether the license
  is met).
- Code: `desktop-app/ui/settings.slint` (the About heading and `licenses-row`), `desktop-app/ui/licenses.slint`,
  `desktop-app/src/ui_licenses.rs`; `SettingsPage.kt`, `SettingsIcons.kt` (`SETTINGS_LICENSES_ICON`),
  `LicensesPage.kt` and `core/LicenseNoticeBlocks.kt` on Android.
  Facts, measurements and the license reading: `doc/decision/music-player-third-party-licenses.md`.

### D133. Auto-play on select defaults to on on every platform (2026-10-22)

The human asked: "Make 'Auto-play on select' default to on for all platforms."
This changes only the default of D129; the rule, the Settings row and the switch are as D129 describes them.

- A first run and a store with no value for the setting now read on: desktop `session.json` with no
  `auto_play_on_select` field, and the Android `session` preferences with no `auto_play_on_select` key.
- A value the person saved stays as saved, so someone who turned the setting off keeps it off.
  A file or preference that never held the key reads on, including one an older build wrote.
- With the setting on at first launch, D124 and D127 apply only after the person turns it off: a click, a tap or
  `Enter` on another track selects and starts it, and a result in Search does the same.
- Tests: the stored default and the old-file reads on both platforms, the fresh-install switch drawn on in the Android
  wiring test, and the desktop end-to-end session `auto-play`, whose first launch now starts with the setting on.

### D134. On Android a notification sound ducks the music instead of pausing it (2026-10-23)

The human reported that other apps lower their volume for about a second when a message sound plays, while this player
paused until the sound ended, and chose "duck, and pause only for real interruptions".
Cause and the source reading: `doc/troubleshooting/music-player-android-pauses-instead-of-ducking-for-notification-sounds.md`.

- A can-duck focus loss (a notification sound, a navigation prompt) lowers the engine volume to a fifth of the user
  volume and keeps playing; the regained focus restores it.
  A paused player stays paused and is lowered too.
- A transient loss that does not allow ducking (a call), a permanent loss and the headphone unplug still pause, as
  before; a permanent loss also clears a duck.
- The engine does the ducking because Android never ducks AAudio output, which this engine uses, so the focus request
  keeps `setWillPauseWhenDucked(true)` to have the listener told.
- The pure rule is `volumeWhileDucked` in `core/FocusDuck.kt` and the user volume and the duck flag are kept apart in
  `RustEngine.kt`.
- Not verified on a device: the human asked for no loudspeaker playback, so the behavior rests on the Android source
  reading and the unit test of the rule.

## Pending after the theme picks (2026-09-04)

- **Order of remaining work** — my recommendation:
   cover screen,
   accessibility pass,
  command bar.
   `open-questions.md` 11c.

User's instruction closing the session:
 record the session,
 update the handover,
 and
implement nothing further.

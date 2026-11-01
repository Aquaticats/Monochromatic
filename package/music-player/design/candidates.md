# Candidate inventory

## Current status and how to read this archive

`questions/current.html` is the selected Fold Search review through
D75 to D80,
not a candidate-selection questionnaire.
The accessibility round is complete;
file-extension display in result titles remains a separate open visual
question outside that round.
Native Search activation,
announcements,
focus transitions and complete cover traversal remain deferred
implementation acceptance work.
The inspected inner TalkBack fixture is 200% and keyboard-closed only.
No production change or new IME experiment is authorized.

Use `decisions.md` for settled choices and `open-questions.md` section 0b
for open,
provisional and developer-owned work.
D46's P4 cover picker remains provisional;
D17's rejected prefix buckets/one-folder-per-row fast-scroll variants do
not become live options merely because their files still exist.
Candidate descriptions are historical unless a named decision explicitly
makes them the current baseline.
MD3-on-Slint feasibility and implementing the keyboard map are
developer-owned;
the keyboard-map design is settled by D102.

## Historical inventory

66 files in **candidates/**.
 Every one is a standalone Design Component (.dc.html)
that opens in a browser at its declared preview size.
 Each demonstrates exactly one
question.
 Verdicts come from the user unless marked otherwise.

All candidates were retro-fitted with verified MD3 values (72dp two-line rows,
 40dp
segmented buttons,
 4×44 slider handles on 16px tracks,
 circular transport buttons,
max-content segmented controls) **except** md1,
 md2,
 md3-expressive and the id-* files,
which were left as historical style comparisons.

---

## Support files (not designs)

- **artists.js** — ~1,218 generated folder names plus helpers (`bucketOf`,
   `LETTERS`,
  `scriptOf`,
   `SCRIPT_LABEL`,
   `grouped`).
   Now includes **non-Latin names** (Japanese,
  Cyrillic,
   Greek,
   Hangul) — real libraries are not Latin-only.
   **Use this in every
  folder demo.**
   Forgetting the scale rule has caused six rejections.
- **buckets.js** — `railFor(names)` builds the adaptive rail (D28):
   only writing
  systems present get a section;
   cells are single letters / kana rows.
   It deliberately
  does **no** sub-letter splitting — that was killed (D17).
- **support.js** — generated Design Component runtime.
   Never edit.

---

## CHOSEN — these define the product

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**mode-d**</td>
<td>860×620</td>
<td>The mode control question settled: connected button group (56px, 2px gaps, pill outer ends, 8px inner corners,
selected inner corners 50%) in one row and wrapped to a grid, with the outlined segmented button (40px) shown beneath
for comparison. All at real token values.</td>
</tr>
<tr>
<td>**light-c**</td>
<td>852×fill</td>
<td>D34 light colors: tonal panes, `outlineVariant` rail line, white center surface and white 16dp picker/transport
divider. Its depicted 24dp empty center gap is obsolete under clarified E2; only informative material must clear the
user's 7.5mm physical dent.</td>
</tr>
<tr>
<td>**dbtp-a**</td>
<td>426×883</td>
<td>D35 default supporting line: duration and true peak share `onSurfaceVariant`; custom display templating will own
optional emphasis.</td>
</tr>
<tr>
<td>**sub-a**</td>
<td>860×600</td>
<td>A folder containing subfolders: subfolders in a labelled section above the folder’s own tracks, one screen, mixed
content.</td>
</tr>
<tr>
<td>**ctx-b**</td>
<td>860×600</td>
<td>Track context menu, eight items in three groups, headed by the track name, dB value inline, Re-analyse, Move to
trash.</td>
</tr>
<tr>
<td>**err-b**</td>
<td>860×520</td>
<td>Missing file drops out of the list with a dismissible bar explaining it; folder renamed mid-playback changes
nothing visible.</td>
</tr>
<tr>
<td>**empty-a**</td>
<td>860×600</td>
<td>Nothing open: explains the hour-long first analysis up front, with Open a folder and Settings.</td>
</tr>
<tr>
<td>**settings-a**</td>
<td>860×600</td>
<td>Three flat switch rows; says out loud that the pane is short. D84 (2026-10-05) removes the third row,
`Analyse true peak in the background`, because true-peak analysis is automatic and not optional; D85 (2026-10-05)
removes the closing sentence. D86 (2026-10-05) removes the two remaining rows: prefix stripping and resume are always
on and are not settings.</td>
</tr>
<tr>
<td>**tabletop-c**</td>
<td>860×600</td>
<td>Tabletop posture: the track list stays continuous and only the crease row moves. The user’s own proposal.</td>
</tr>
</tbody>
</table>

---

## Conventions added in session 4

- **Every option candidate prints its own PROS / CONS / MY READ** in the caption bar
  at the bottom of the frame (review-notes standing standard 6).
   The same text goes in
  the form's question subtitle and in chat — the user may be reading any of the three.
- **A theme study pins its colour scheme inline on its own root element** and links the
  token sheets from `candidates/_ds/…` (copied there),
   never `../_ds/…`.
   The host sets
  `data-theme="dark"` on `<html>`;
   inheriting it silently inverts a light design
  (review-notes 5g).
- **Light studies sit on a light desk**,
   dark studies on a dark one,
   and frames fill
  the viewport height rather than floating in dead space.

## Historical screen refinement matrix, settled with 1B

The accepted `divider-final.png` remains the before-state evidence.
 Its transport
content is 15dp toward the fold:
 the mode outline spans x=112 through x=969 and has
axis x=540.5,
 while the 414dp pane has axis x=504.
 The nine native refinements correct
that D18 violation and the seek-value mismatch in every cell.
 They vary only two
existing presentation details:

- Rows:
   tight 8dp,
   balanced 12dp,
   or airy 16dp spacing between the same transport
  groups.
- Columns:
   standard,
   outlined,
   or tonal baseline Material styling for the existing
  previous and next buttons.
   Pause remains filled.

Candidate names are `refine-{tight|balanced|airy}-{standard|outlined|tonal}`.
 All are
opaque 2076 × 2152px Pixel 9 Pro Fold captures from prototype commit `2b2ac2e88`.
The mode outline spans x=73 through x=935 in every capture,
 giving axis x=504.
 The
transport surface starts at y=1487,
 y=1460,
 or y=1430 for tight,
 balanced,
 or airy
spacing.
 No content,
 feature,
 state,
 component order,
 pane,
 color,
 divider,
 or list
rule changes.

The designer's full-resolution ranking was
`2A > 2C > 1A > 1C > 3A > 3C > 2B > 1B > 3B`.
`2A`,
 balanced spacing with standard skip buttons,
 was recommended because it gave the filled
pause action the clearest focal point without compressing the stack.
 The user chose
`1B` instead:
 tight 8dp spacing with outlined Previous and Next buttons.
 This choice
settles both matrix dimensions and supersedes the recommendation.

## ROUND 13 — redundant current-track and Open cues

All studies inherit chosen option 1B:
 centered deck,
 corrected 24% seek position,
8dp transport spacing,
 outlined Previous and Next buttons,
 filled Pause,
 and every
settled picker,
 track,
 spacer,
 and mode-control rule.

The first current-track studies retained a play icon.
 The user rejected that entire
approach before presentation because its reserved leading column reduced the width of
every track.
 Those studies are evidence of a corrected failure,
 not active options.

The active rows remove both the play icon and the leading slot from every list item.
Current and ordinary titles share x=1107 at native resolution,
 proving equal text width.
Each row then combines cues without relying solely on color or shape:

- **T1, explicit label:**
   bold title plus bold `Playing` text in the existing
  supporting line.
- **T2, accent:**
   primary-colored bold title.
- **T3, soft container:**
   neutral `surfaceContainerLow` row plus bold title.
- **T4, outlined row:**
   primary rounded boundary plus bold title.

The separate Open-action columns retain the folder icon and explicit `Open` label while
adding a button container:

- **O1, filled:**
   primary fill with contrasting icon and label.
- **O2, tonal:**
   secondary-container fill with contrasting icon and label.
- **O3, elevated:**
   elevated surface container with primary icon and label.
- **O4, outlined:**
   outline boundary with neutral icon and label.

Prototype commits `e4f9d9484` and `4cae242a7` implement the corrected four-by-four
matrix.
 Commit `ed04759ed` adds explicit current-track descriptions;
 evidence commit
`8203ecc12` preserves UI Automator hierarchies for every row treatment.
 All sixteen
opaque captures are 2076 × 2152px.
 At 200% text,
 every row
treatment remains readable and the filled Open button fits beside `Folders`.
 No current
candidate exposes a play-icon accessibility node.

Every Open action exposes a 263 × 117 physical-pixel touch target,
 approximately
108 × 48dp.
 Measured foreground-to-container contrast is 6.065:1 for filled,
6.124:1 for tonal,
 5.794:1 for elevated,
 and 6.359:1 for outlined text.
The soft current-row container resolves to 11.571:1 for its title and 5.757:1 for
supporting text.

Current-track ranking is T1 > T2 > T3 > T4:
 explicit text is clearest;
 accent is
cleaner but less literal;
 the soft container changes more pixels;
 the outline resembles
a separate button.
 Open ranking is O1 > O2 > O3 > O4:
 filled communicates primary
action most strongly;
 tonal is calmer;
 elevation is subtle on the flat app bar;
 the
outlined boundary is weakest.

**Selected:**
 3B.
 T3 uses the soft neutral row container plus bold current title;
O2 uses the tonal folder `Open` button.
 This supersedes the recommendation for 1A
without changing the settled transport treatment.

## ROUND 10 — white-divider clarification

Three native full-device captures keep the chosen `1c` tonal structure and `3a`
neutral metadata treatment.
 They differ only in the two plausible separator colors:

- **divider-a:**
   white 1dp line between the letter rail and folder names;
   tonal-gray
  24dp center spacer.
   This is the unaccepted first interpretation.
- **divider-b:**
   `outlineVariant` rail line;
   white center spacer.
   Recommended because
  it changes the prominent vertical separator while retaining a visible rail boundary.
- **divider-c:**
   rail line and center spacer both white.
   This covers both readings but
  leaves neither boundary visibly drawn.

**Verdict:**
 D2,
 plus the 16dp horizontal picker/transport divider marked in
`Screenshot_20260904_191909.png` must also be white.
 D34 is final:
 gray rail line,
white vertical center spacer,
 and white horizontal section divider.
 The final native
recapture must also apply D36:
 no invented track ordinals,
 no saturated current-row
fill,
 and a leading play icon as the current-track cue.

## ROUND 9 — theme work (all built on the bound MD3 bundle)

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**light-a**</td>
<td>852×fill</td>
<td>Light theme, all-tonal: surface ground and tracks, surface-container-low picker, surface-container transport,
surface-container-high rail. No dividers.</td>
</tr>
<tr>
<td>**light-b**</td>
<td>852×fill</td>
<td>Light theme, one surface role plus 1dp outline-variant dividers at pane, rail, and row boundaries.</td>
</tr>
<tr>
<td>**light-c**</td>
<td>852×fill</td>
<td>**CHOSEN (D34), final:** surface-container panes, visible `outlineVariant` rail line, white vertical center spacer,
and white horizontal picker/transport divider.</td>
</tr>
<tr>
<td>**light-abc**</td>
<td>3-up</td>
<td>All three side by side for comparison.</td>
</tr>
<tr>
<td>**dark-a**</td>
<td>418×fill</td>
<td>REJECTED — MD3&#39;s own dark container ramp above #000; the panes read as grey cards floating on black.</td>
</tr>
<tr>
<td>**dark-b**</td>
<td>418×fill</td>
<td>**CHOSEN (D32)** — the project&#39;s own ramp measured down from black: #000 / #0A0A0D / #121216 / #1A1A1F /
#22222A.</td>
</tr>
<tr>
<td>**dark-ab**</td>
<td>2-up</td>
<td>Both, plus a swatch comparison of the two ramps.</td>
</tr>
<tr>
<td>**dbtp-a/b/c**</td>
<td>426×883</td>
<td>**3a CHOSEN (D35):** one on-surface-variant line, because planned display templating will provide user-selected
emphasis. 3b strengthens true peak; 3c moves it to the trailing slot.</td>
</tr>
<tr>
<td>**light-a/b/c**</td>
<td>852×fill</td>
<td>**1c CHOSEN (D34); final separator treatment is gray rail line plus white vertical and horizontal pane spacers.**</t
d>
</tr>
<tr>
<td>**candidates/_ds/**</td>
<td>—</td>
<td>Copied MD3 token sheets (palette, color, fonts, typography, shape, elevation, spacing, state) so candidate files
resolve them without `../`.</td>
</tr>
</tbody>
</table>

All light-side files are presented on a light desk with a light caption bar
(review-notes 5g);
 the dark pair keeps a dark one.
 For the historical native theme questionnaire,
 the
six `.dc.html` files are now historical design records rather than screenshot sources.
Branch `prototype/music-player-theme-compose` rebuilds the same six keys in native
Compose and captures them from the unfolded Pixel 9 Pro Fold emulator with Android
system bars.
 Prototype commit `8ec92ff7f` rebuilds them against the user-supplied
Material archive with Android dynamic color,
 role-mapped surfaces,
 two 414dp panes and
a 24dp spacer,
 baseline Material list items,
 real app bars,
 buttons,
 icons,
 slider and
one-row segmented control.
 It also retains 30dp edge-gesture insets inside full-bleed
section backgrounds.
 The left app bar now places `Open` beside `Folders` and removes
the redundant current-folder control;
 the right `Camellia` title supplies current
folder identity.
 The visible segmented labels are `Repeat`,
 `In order`,
 `Shuffle`,
and `Shuffle all`;
 accessibility labels expand each mode's full meaning.
 Native
1038 × 2152px right-half captures now include 12dp of pane spacer and the 414dp detail
pane.
 The letter rail begins at the physical start edge,
 while the app bar,
 source
actions,
 and transport retain their measured system-gesture insets.
 Selected folder
state uses primary text,
 medium weight,
 and a separate 2dp MD1-style indicator spanning
the bottom of the whole target;
 the text itself is not underlined.
 At 200% font scale,
the mode control becomes one purely vertical segmented stack with four connected
full-label segments.
 It retains segmented outlines,
 selected fill,
 checkmark,
 and
single-select semantics without horizontal scrolling.
 The corrected native evidence is
`questions/render/refine-large-text-segmented.png` from prototype commit `bf6830f4f`.
The transport was scrolled vertically for that capture,
 placing the full control above
the system navigation inset.
 That fixed large-text transition is now superseded by
prototype commits `f06971b1e`,
 `53fb648ee`,
 `bb5dad664`,
 `d8b2552b6`,
 and `538b71304`.
 Exact accepted 3B captures at every target Settings preset are preserved by prototype
commit `cedc2a668`:
 85%,
 100%,
 115%,
 130%,
 150%,
 180%,
 and 200%.
 The
visible third option is `Shuffle Camellia`.
 Overflow-driven reflow uses one connected
content-sized row at 85%,
 connected 2×2 at 100%,
 115%,
 and 130%,
 then four
vertical rows at 150%,
 180%,
 and 200%.
 Every segment keeps at least 12dp
horizontal content padding;
 overflow adds rows instead of reducing that floor.
 One-row subdirectory content is capped at
rendered `Camellia` width;
 longer names middle-ellipsize the name only and retain full
semantics.
 Earlier 50% and 75% captures are developer-injected diagnostics rather
than ordinary Android states.
The user's
settled baseline combines the `light-c` tonal structure with `dbtp-a`.
 D34 makes the
vertical center spacer and horizontal picker/transport divider white while retaining
the visible dynamic rail line.

## ROUND 8 — verdicts in

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**unf-j**</td>
<td>852×883</td>
<td>**Current best unfolded screen.** unf-i + D31 names (plain text, several per line, primary + underline on the
selection) and the D28 adaptive rail built from buckets.js. Every session-3 decision applied. Supersedes unf-h and
unf-i.</td>
</tr>
<tr>
<td>**pk-h**</td>
<td>418×883</td>
<td>Rejected: 14px with middot separators (2.0 screens vs 2.2). Denser, but reads as running text rather than a set of
targets.</td>
</tr>
<tr>
<td>**pk-g**</td>
<td>418×883</td>
<td>**CHOSEN (D31).**</td>
<td>**The picker presentation proposal (D31).** Plain 16px text, several names per line at natural width, 24dp gaps,
48dp targets, nothing truncated, selection = primary colour + 2dp underline. No pill, no fill, no outline. This is
unf-i&#39;s layout with the chip styling removed. Extent counter is measured off the DOM (C = 2.2 screens, worst letter
S = 2.5), not estimated.</td>
</tr>
</tbody>
</table>

## ROUND 7 — verdicts in

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>Status</th>
</tr>
</thead>
<tbody>
<tr>
<td>**scan-ef**</td>
<td>2×411×923</td>
<td>**scan-F CHOSEN (D26)** — right frame: 56dp bar, count + always-rendered fixed-width Pause, no reflow. scan-E
(left, 2dp bottom-edge line, no text or control) rejected as too little for an hour-long job.</td>
</tr>
<tr>
<td>**first-run-a**</td>
<td>411×923</td>
<td>**Settled behaviour (D27).** Auto-opens the system music library and asks before analysing: Scan once / Always scan
/ Dismiss once / Dismiss forever, all 48dp. Interactive — walks into the scan-F bar, the dismissed state and
always-scan. D84 (2026-10-05) supersedes the ask-before-analysing prompt and its four answers: true-peak analysis is
automatic and not optional. Automatic opening of the system music library stands.</td>
</tr>
<tr>
<td>**toast-a**</td>
<td>411×923</td>
<td>**Settled behaviour (D29).** Undo as a content-width toast floating above the error bar, both visible; it lifts and
drops with the bar. Tap rows to trash, × to dismiss the bar.</td>
</tr>
<tr>
<td>**unf-i**</td>
<td>852×883</td>
<td>Superseded by unf-j. unf-h + D20 volume popover, D23, D24 — but still chip-styled names.</td>
</tr>
<tr>
<td>**pk-a**</td>
<td>418×883</td>
<td>REJECTED — one name per 48dp row, single column. The long-scroll failure again.</td>
</tr>
<tr>
<td>**pk-b**</td>
<td>418×883</td>
<td>REJECTED — two columns of one-name rows. Same failure, halved.</td>
</tr>
<tr>
<td>**pk-c**</td>
<td>418×883</td>
<td>REJECTED — same column with hairline dividers. Same failure.</td>
</tr>
<tr>
<td>**pk-d**</td>
<td>418×883</td>
<td>VOID — bounded the extent by re-introducing prefix ranges (Ca / Ch / Cr) as a rail accordion, which D17 had already
killed, and kept chip styling. Do not build on it.</td>
</tr>
<tr>
<td>**pk-e**</td>
<td>418×883</td>
<td>REJECTED — three columns of one-name rows, plus truncation.</td>
</tr>
<tr>
<td>**pk-f**</td>
<td>418×883</td>
<td>REJECTED — one full-width column bounded by a type-to-narrow field. Rows again; the field also pre-empts the
reserved search (D25).</td>
</tr>
</tbody>
</table>

## Historical round 6, scrollbar choice settled by D22

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**scroll-a**</td>
<td>1100×640</td>
<td>Scrollbar spec: A = Compose M3 nonInteractiveScrollbar exactly (fades); B = same look, interactive, always on
(desktop); C = which surface gets which. Both lists live.</td>
</tr>
</tbody>
</table>

## ROUND 5 — verdicts in

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>Status</th>
</tr>
</thead>
<tbody>
<tr>
<td>**unf-g**</td>
<td>852×883</td>
<td>CHOSEN over unf-f: transport left under the picker, tracks right.</td>
</tr>
<tr>
<td>**unf-h**</td>
<td>852×883</td>
<td>unf-g + desc-f&#39;s outlined segmented control + desc-g&#39;s rebalanced transport; bucket chips removed on
request. **Current best unfolded screen.**</td>
</tr>
<tr>
<td>**desc-f**</td>
<td>418×883</td>
<td>CHOSEN over desc-e: outlined segmented button, all four always visible. Called &quot;un-balanced&quot; →
desc-g.</td>
</tr>
<tr>
<td>**desc-g**</td>
<td>418×883</td>
<td>desc-f rebalanced (D18).</td>
</tr>
<tr>
<td>**desc-e**</td>
<td>418×883</td>
<td>Rejected: connected button group.</td>
</tr>
<tr>
<td>**unf-f**</td>
<td>852×883</td>
<td>Rejected side (transport right); picker itself accepted.</td>
</tr>
</tbody>
</table>

## ROUND 4

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**unf-f**</td>
<td>852×883</td>
<td>Picker that copes with 1,218 folders without listing them: letter rail → optional 2-letter bucket chips → wrapped
content-width chips, ~one screen max. Interactive. Deck right.</td>
</tr>
</tbody>
</table>

## ROUND 3 — rejected (unf-d/unf-e still one folder per row)

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**unf-d**</td>
<td>852×883</td>
<td>Folders in one column with letter headers; 48dp fast-scroll RAIL with drag bubble; deck right.</td>
</tr>
<tr>
<td>**unf-e**</td>
<td>852×883</td>
<td>Same, A–Z as borderless text index under the list (48dp invisible targets).</td>
</tr>
<tr>
<td>**desc-d**</td>
<td>418×883</td>
<td>Subfolders as headers in one flat list; ▶</td>
<td>walks playback down it. Implements D5/D6 as clarified.</td>
</tr>
<tr>
<td>**cover-c**</td>
<td>411×923</td>
<td>CHOSEN (D14). Reframed at fixed device size. HTML-era baseline; the native cover round of 2026-09-17 re-verifies it
at folded geometry, font scales, and the D41/D42 dark policy. Its inline volume element is gone per D43.</td>
</tr>
</tbody>
</table>

## ROUND 2 — verdicts in

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>What it shows</th>
</tr>
</thead>
<tbody>
<tr>
<td>**unf-b**</td>
<td>852×883</td>
<td>REJECTED — letter grid reads as a keyboard. Unfolded, deck RIGHT. ~1,218 folders (artists.js) in two columns of
48dp rows with letter headers + counts; 27-cell A–Z strip (48dp) pinned at the bottom of the left half, tap to jump;
total folder count in the header; tracks + deck right; 16dp seam. Interactive.</td>
</tr>
<tr>
<td>**unf-c**</td>
<td>852×883</td>
<td>REJECTED (same grid). Same file, deck LEFT under the folders and strip; tracks take the whole right half. Known
cost: at 883dp the folder list gets ~340dp (≈7 rows) before scrolling.</td>
</tr>
<tr>
<td>**cover-c**</td>
<td>411×923</td>
<td>Cover screen, full player: chip + Open top, track list, deck at the bottom (thumb reach), volume inline, mode group
2×2. Verified at 411 wide.</td>
</tr>
<tr>
<td>**cover-d**</td>
<td>411×923</td>
<td>REJECTED. Cover screen, controls only: 88dp play, 64dp prev/next, mode group, no volume (hardware keys), list
behind one 72dp &quot;Tracks&quot; row. Verified at 411 wide.</td>
</tr>
<tr>
<td>**desc-a**</td>
<td>418×883</td>
<td>SUPERSEDED by desc-d (wrong model of subfolders). Descend demo, interactive (▶</td>
<td>steps playback): the list FOLLOWS into the subfolder; chip becomes a breadcrumb; &quot;Up to Camellia&quot;
row.</td>
</tr>
<tr>
<td>**desc-b**</td>
<td>418×883</td>
<td>SUPERSEDED. Same demo: the list STAYS; only the deck subtitle names the subfolder (&quot;Live · 1 of 3&quot;).</td>
</tr>
<tr>
<td>**desc-c**</td>
<td>418×883</td>
<td>SUPERSEDED. Same demo: the list STAYS; the playing subfolder row takes the highlight with &quot;Playing ·
track&quot; beneath.</td>
</tr>
</tbody>
</table>

All seven use the verified colour roles (md3-tokens.md):
 outline #938F99,
on-secondary-container #E8DEF8.
 The descend demos assume parent tracks play before
subfolders (unconfirmed — see open-questions.md #3).

## Historical rejected or unresolved candidates

<table>
<thead>
<tr>
<th>File</th>
<th>Size</th>
<th>Status</th>
</tr>
</thead>
<tbody>
<tr>
<td>**unf-a**</td>
<td>852×883</td>
<td>Rejected (15 folders). Superseded by unf-b / unf-c.</td>
</tr>
<tr>
<td>**cover-a**, **cover-b**</td>
<td>411×923</td>
<td>Shown while broken, never verified. Superseded by cover-c / cover-d.</td>
</tr>
<tr>
<td>**keys-a**</td>
<td>900×560</td>
<td>Keyboard map draft. Never reviewed.</td>
</tr>
<tr>
<td>**scan-b**</td>
<td>860×~540</td>
<td>Scan indicator, reflow-free version — half of the intended behaviour.</td>
</tr>
<tr>
<td>**scan-d**</td>
<td>860×~540</td>
<td>Scan indicator, the other half (non-permanent). The settled version has never been built.</td>
</tr>
</tbody>
</table>

---

## SUPERSEDED — keep for reference, do not build on

### Wrong-aspect fold layouts (all drawn in a landscape frame; the inner display is square)
- **fold-a** — single-column folder sidebar;
   contradicted the wrapped-target decision.
- **fold-b** — early split with a tweakable ratio.
- **fold-c** — deck spanning the full width (put the play button on the crease).
- **fold-d**,
   **fold-e** — interactive fold-state demos with props.
- **fold-f** — seam gutter introduced,
   controls left / tracks right,
   quiet left half.
- **fold-g** — folder grid fills the left half,
   deck moved bottom-right.
  fold-f and fold-g were the real O2 comparison and that comparison is void at the
  wrong aspect;
   the question (which half holds the deck) is still open.

### Cover screen at an invented size
- **o7-a**,
   **o7-b** — 370×760,
   a size with no source.
   cover-a/cover-b are their
  resized descendants.

### Moot — the question was already answered elsewhere
- **o6-a** — end of folder:
   playback stops,
   Play again / Pick another folder.
- **o6-b** — end of folder:
   rolls into the next folder with a NEXT chip.
  Both moot:
   the mode control (Repeat / In order / Shuffle folder / Shuffle all)
  already determines end-of-folder behaviour.

### Mode control explorations
- **mode-a** — detached pills,
   wrapping.
- **mode-b** — alternative detached treatment.
- **mode-c** — outlined segmented button built to match uploads/segmented buttons.png
  exactly (one container,
   content-width segments,
   wraps to a grid with flush rounded
  outer edges).
   **Keep:
   this is the fallback if the connected button group is dropped.**

### Historical folder picker explorations, superseded by D17 and D31
- **picker-a** — letter section rows.
- **picker-b** — the jump strip (the direction that won).
- **picker-c** — persistent sidebar.
- **picker-d**,
   **picker-e**,
   **picker-f** — built at ~1,000 folders with real 48dp
  targets;
   **picker-f is interactive**.
   These are the best existing reference for
  handling the real folder count — read them before rebuilding unf-a.

### Row treatments (two-line won)
- **rows-a** — single line.
   **rows-b** — columnar.
   **rows-c** — two line (won).

### Design-generation comparison (MD3 baseline won)
- **md1** — Material 1 era.
   **md2** — Material 2.
   **md3** — MD3 baseline (won).
- **md3-expressive** — MD3 Expressive:
   asymmetric radii,
   heavier type,
   larger
  controls.
   Rejected for spending vertical space on personality.
- **md3-hybrid** — MD3 baseline with selected Expressive touches.

### Identity explorations (MD3 won)
- **id-fluent** — Fluent-flavoured.
   **id-protool** — pro-audio tool aesthetic.
- **id-own** — a bespoke in-house identity.
   Rejected on the user’s own argument:
   an
  in-house system means designing and maintaining every state by hand.

### Scan indicator explorations
- **scan-a** — known defect:
   the Pause button appears and disappears,
   so it is not
  reflow-free.
   **scan-c** — alternative treatment.

### Tabletop explorations
- **tabletop-a** — split at the hinge,
   list above / controls below.
- **tabletop-b** — posture-agnostic.

### Error handling
- **err-a** — errors in place:
   the dead row keeps its position,
   turns error-coloured
  and explains itself;
   the folder chip carries its own failure state.
   Rejected in
  favour of err-b’s cleaner list.

### Other paired alternatives and revised choices
- **o1-a** — originally chosen with Open beside a folder chip;
   superseded by D4's
  removal of that chip and placement of Open on the `Folders` app-bar line.
- **o1-b** — Open in the overflow menu.
- **sub-b** — drill-in navigation with breadcrumb and Up.
- **ctx-a** — four-item minimal context menu.
- **empty-b** — bare empty state,
   scan line in the bottom bar behind "Why?".
- **settings-b** — grouped cards with headers,
   plus the analysis-status row and
  Re-analyse.
   Worth revisiting only if decision D12 (status lives nowhere) changes.

---

## Reading order if you are new

1. **picker-f** — how ~1,000 folders and the jump strip actually behave.
2. **mode-d** — the component the whole UI hinges on,
    at real token values.
3. **sub-a**,
    **ctx-b**,
    **err-b**,
    **empty-a**,
    **settings-a** — the settled surfaces.
4. **unf-a** — the rejected unfolded layout,
    to see the structure being kept.
5. **uploads/music-player-mockup.html** — the user’s original proposal,
    which is the
   baseline this all started from.

## Cover picker opened-state prototypes (P round, native captures)

Supersedes the K round: K1 (full-screen page with Navigate up) rejected as discouraging,
K3 (modal bottom sheet) rejected as non-local, K2 refined into the P set after reading the
chips, menus, sheets, and text-field specs.

- **P1** — outlined MD3 text-field trigger (56dp, floating Folder label, 4dp corners) plus
  an inset floating menu container (4dp corners, fixed high step in dark) over the list,
  ending above the deck. Not selected for the provisional baseline.
- **P2** — same field trigger; the picker replaces the list slot in place while the deck
  stays visible; caret flips to close. The comparison recommended P2, but the user chose
  P4 for the provisional baseline.
- **P3** — app-bar title plus caret trigger with the P1 floating menu container. Not
  selected for the provisional baseline.
- **P4** — app-bar title trigger with the P2 in-slot container. Selected by D46 as a
  **temporary pre-1.x decision**; the user believes a better solution exists.

All four reuse the D28 adaptive rail and the D31 wrapped plain-text names at 48dp with
primary plus 2dp indicator on the current folder. Captures: `cover-round-cover-picker-*`
in `questions/render`, dark at 100%, P2 at 200%, and P2 in light under the settled L3
(D45). Post-decision P4 captures now cover dark 200%, L3 light 100%, and L3 light 200%
under the same folded native geometry; the original P form remains historical and does
not contain those later rasters.
 A separate debug-only interactive P4 study at 200% verified title open/close, Android
Back, selection dismissal, and keyboard focus retained after Back; TalkBack focus is not
covered (`questions/evidence/cover-round-cover-picker-p4-interaction.json`).
 Documented
P1/P3 deviation: the floating menu panel exceeds the baseline menu's 112 to 280dp width
cap because the D31 name wall needs the full panel.

## Historical desktop command-bar visual drafts, withdrawn

The I/G/R questions are independent,
 and no choice was made.
 The first S/C matrix
conflated in-app and global relationships;
 the second I/G/R native Slint form was
rejected as visibly non-MD3.
 The candidates recorded here are historical design
probes,
 not current response options.
 The subsequent replacement used the Search bar/view
visual and token evidence in `material-3-compliance.md`.

The withdrawn I/G/R variant roles were:

- **I1** dock the action surface below the player header (recommended);
  **I2** float
  within the player;
  **I3** replace the content region.
  In-app ranking: I1 > I2 >
  I3,
  based on source anchoring and retained context.
- **G1** raise the player when the optional D21 global hotkey is used from another
  app;
  **G2** show a detached quick-window concept above that app (recommended).
  Global
  ranking: G2 > G1,
  for preserving the previous task.
  The OS second window and
  hotkey are not implemented by this study.
- **R1** give future Ctrl+F a distinct search destination (recommended);
  **R2** enter
  explicit search mode in the command surface.
  Search relationship ranking: R1 >
  R2,
  for dedicated result room rather than mode switching.
  D25 settles the shortcut
  reservation only,
  not this choice.

All in-app variants depict the same typed action query and selected result.
 Each
candidate has opaque native Slint captures in both schemes at the current desktop
app's preferred 480 × 600px size;
 I1 to I3 also appear at exploratory 1100 × 640px.
I1 has a 360 × 640px larger-text/long-label stress capture.
 E1 no-results and E2
unavailable-action captures are contextual probes,
 not options.
 The rejected form was replaced in `questions/current.html` before another review;
 do not solicit codes from the withdrawn rasters.
 The withdrawn native rasters are
`questions/render/command-round-{i,g,r,e}*`.
 The
first S × C draft was not shown to the user;
 its evidence is also historical in
`questions/evidence/command-round-prior-art.md`.
 Native static rasters
do not prove keyboard operation,
 accessible semantics,
 focus return,
 hotkey feasibility,
search indexing,
 or real second-window behavior.

## Baseline MD3 divided Search command round (rejected by D47)

The independent I1 to I3,
 G1 to G2 and R1 to R2 roles remain as documented for the
withdrawn draft,
 but all active scenes have been redrawn under A2's baseline MD3
policy.
 The native prototype on branch `prototype/music-player-theme-compose` uses
`command-md3-study.slint` with Roboto and official Material SVGs.
 It renders the
baseline expanded docked Search container (28dp outer corners,
 56dp flat header,
1dp divider),
 the sharp full-content variant (72dp header,
 divider),
 and baseline
72dp two-line square list rows with a custom outline plus state-layer fill for the
current keyboard result.
 The collapsed 56dp pill is not incorrectly nested inside the
expanded view.
 All color roles are sampled from the documented wallpaper-role
evidence.

`questions/render/command-md3-{i,g,r,e}*` contains each light/dark native scene
at the desktop app's preferred 480 × 600px size;
 I1 to I3 also cover 1100 × 640px,
and the I1 360 × 640px probe draws a proposed full-content larger-text fallback;
its static `stress` input does not verify width-driven responsive behavior.
 E1 no-result and E2
unavailable-action scenes remain contextual probes,
 not independent choices.
The rejected command questionnaire was archived at
`questions/archive/command-igr-rejected.html` when D47 replaced it.
 Its normalized 56px baseline-spec versus native
header comparison is at
`questions/evidence/command-md3-baseline-header-comparison.png`.
 None of these
alternatives has been selected by the user.
 The Slint surfaces are static and do not
verify working shortcuts,
 input focus,
 screen-reader semantics,
 real OS window
ownership,
 result execution,
 or search indexing.

## Search button to separate search page (D47/D48, desktop-size experiment retired by D49)

The user rejected all I/G/R command-bar presentations.
 This earlier Slint prototype has one
direction rather than an option matrix:
 a 48px Search button target with a 24px official icon in the player
opens a separate page,
 whose single 72px baseline MD3 header merges Back,
 query
and Clear (D48).
 That retired desktop-sized Search experiment replaced the player entirely;
D50/D51 later required a persistent unfolded deck.
 No second in-app
title row remains in the selected Search destination.
 The throwaway Slint `search-page-study.slint` on the prototype
branch is committed at `1ecb1f3f7`,
 with top-bar geometry correction `773c8fc11`
and merged-header correction `13b38f1f7`.
 Native 360 × 640 and 480 × 600px light/dark captures show the player trigger,
empty page,
 illustrative `cam` results,
 no results and unavailable library;
1100 × 640px captures cover the trigger,
 empty page and illustrative results only
(`questions/render/search-page-*`,
 main commit `c65154138`).
 Search button and Back
TouchAreas are wired in the design prototype;
 the static captures do not by
themselves prove click delivery or focus restoration.
 The rejected I/G/R form is now archived at
`questions/archive/command-igr-rejected.html`;
 The formerly active desktop walkthrough is now retired;
`questions/current.html` now embeds the selected A-only native Fold review.
The old HTML
flow was exercised,
 but it neither proves the native TouchAreas were activated nor
measures the target phone geometry.
D21's global hotkey and Settings row are not inherited by the Search page.
D25's Ctrl+F reservation is part of the keyboard map D102 accepted.
 At that retired round,
 Search targets and result effects/ranking were still design questions;
 D58 to D61 and D63 to D80 now settle the visible and interaction goals.
 Engine selection and native implementation remain deferred.
D49 makes the Fold the visual source for every platform,
 and D51 later
selected A's unfolded pane placement.

## Rejected native Fold Search-page composition (D49)

The debug-only Compose study on `prototype/music-player-theme-compose` captured
the D47/D48 player trigger and separate one-header Search page on the cover
(1080 × 2424px,
 approximately 443 × 994dp at the measured AVD 390dpi) and inner
display (2076 × 2152px,
 approximately 852 × 883dp) in both schemes.
 `questions/render/fold-search-{cover,inner}-*.png` contains opaque
Android system bars and target-sized results,
 empty,
 no-results and unavailable
states at 100% text;
 player and results at 200% text are fit checks.
 Each PNG
has a paired XML hierarchy in `questions/evidence/` and each panel/scheme has a
native role JSON;
 each frame now has measured panel,
 density,
 night-mode and
font-scale metadata in `questions/evidence/`.
 The rejected unfolded study put Back/query/Clear over an empty left body and
isolated results or instructions in the right body.
 The user rejected the
composition despite the old guard passing.
 That guard also enforced a mistaken
reading of connector "content" as all pixels and hit regions;
 it is not the
new design acceptance gate.
 The captures are historical diagnostic evidence,
not a candidate for endorsement.

The withdrawn split study is archived at
`questions/archive/search-rejected-fold-review.html`.
The active `questions/current.html` now shows selected A only,
 not that old
walkthrough,
 gallery or feedback form.
 The cover chassis preview uses approximately 443 ×
994 CSS px at 100% AVD dp;
 the old 411 × 923dp sizing was an unmeasured
published-ppi estimate.
 The archived canned rows and HTML walkthrough never proved an actual index,
result execution,
 native Back focus or TalkBack speech.
 D46's provisional P4 cover picker and accepted unfolded
baseline are contextual only;
 the desktop Slint captures are history,
 not a
matrix to choose from.

## Selected Fold Search A after D52

The three-way keyboard-visible comparison is archived at
`questions/archive/search-three-way-before-a.html`.
The user selected A (D51):
 Search query and results stay together on the unfolded
right pane,
 and the existing bottom-left deck lifts above the keyboard while
typing.
 D52 removes the duplicate `Results for “cam”` heading from positive
results on the inner and cover panels.
 The initial A-only rasters in
`questions/render/search-selected-review-*` are sanitized native captures
at 100% and 200% text in light/dark.
 The self-contained
`questions/current.html` shows the selected state,
 not a new choice menu.
Native query input was exercised with a measured 300dp debug-only system IME;
Real-Gboard geometry is measured for the linked settled and overlay states;
D53 to D55 accept the observed floating-deck,
brief banner and cover floating-result overlaps.
Other keyboard configurations and production implementation remain unverified.
D58 to D61 refine alignment,
match emphasis,
direct-name membership and mixed relevance.
D63 to D74 settle navigation,
status truth and result-action goals;
D75 to D80 adopt accessibility goals with a chat correction/veto path.
The current selected-only review and
`evidence/search-talkback-native-baseline.md` distinguish those goals from
static captures and bounded observed speech.
There is no native Search result handler,
verified cover swipe sequence or live announcement/return-focus proof.

# Historical Search result-order comparison before D60/D61

## What the fixture does and does not do

This #116 comparison preceded the user's independent selection of
**Scope D** (D60) and **Order M** (D61).
The [active selected-only review][current-review] now shows just that
combination;
the [archived native matrix][archived-matrix] retains the alternatives.
This historical comparison uses only debug fixture rows from
`SearchRankingFixture.kt` on prototype commit `6addfc139`,
D58's aligned row correction from `baa37caaf`,
and D59's OS-accent match spans from `b471ec537`.
The linked historical decision images come from the installed debug APK
whose SHA-256 matched the local build:
`7195af99031158bfb8efce2abab3c98ebaf17928739c290010bc6dea1317244f`.
The earlier `bf9a51c42facfca969e275d5fe55769656c451900f57689dc7c24190e8d6d2ef`
images remain a pre-D59 baseline,
not the active review.
The activity dispatches `search-deck-*` candidates to
`SearchPersistentDeckStudy.kt`;
its cover branch delegates to `SearchLayoutStudy.kt`.
These screenshots exercise the inner persistent result rows and the
cover layout rows,
not the rejected alternate unfolded arrangements.
All samples have a prefilled `cam` query,
a closed keyboard,
200% text and 390dpi.
The inner samples retain the selected folder browser,
complete deck and E2 P7.5 floor;
the cover samples use its full-width Search destination.
The native inner screen is 2076 × 2152 physical px;
the cover is 1080 × 2424px.
The cover-only D56 viewport marker remains in the candidate name.
The fixed result captions are **illustrative**:
there is no search index,
scoring algorithm,
click handler or result navigation in this study.

Two folders are shown:
`Camellia` and `Camera Obscura`.
Four tracks have direct title/filename examples:
`Cam` (exact),
`Camellia Waltz` in `Camellia` (prefix),
the same filename in `Cult of Luna` (same-title disambiguation),
and `Live at Camellia` (contained).
A separately toggled fifth track,
`Another Xronixle`,
illustrates an **immediate-parent-only** `Camellia` match even though its
own title lacks `cam`.
The fixture contains no ancestor-path,
Unicode casefold,
token,
fuzzy,
locale sorting,
large-library cardinality or real filename parser test.
Its manually ordered lists cannot validate any of those matching rules.

## Independent membership choice

- **D:
  direct names only.**
  Folder names and track filenames can contribute hits;
  parent names stay visible as disambiguating context but do not themselves
  expand into track hits.
  The sample keeps the six directly named rows.
  This avoids adding parent-only child tracks to the result set,
  but a track whose own filename lacks the query is not a separate hit.
  Whether activating the folder can lead to its contents remains open
  under review task 129.
- **P:
  include immediate-parent-only track hits.**
  The sample adds `Another Xronixle` with
  `Track · Camellia · parent-only match`.
  It can surface a relevant track directly,
  but a large folder name match could add many child rows in addition to
  its folder result.
  M and F place the parent-only row after direct matches in this fixture;
  T places it before the folders.
  The fixture does not measure real-library result counts or a limit.

On the inner panel,
the [D mixed-order initial screenshot][inner-mixed-direct-top] and
[P mixed-order initial screenshot][inner-mixed-parent-top] are
pixel-identical because the added track sits below the initial viewport.
The [D mixed-order scrolled state][inner-mixed-direct-end] and
[P mixed-order scrolled state][inner-mixed-parent-end] expose the actual
membership difference;
the P final detail remains fully visible.
On the cover,
[P mixed-order initial][cover-mixed-parent-top] and
[P mixed-order scrolled][cover-mixed-parent-end] show the extra row.
A zero pixel difference at one scroll position is not evidence of an
identical result set.

## Independent ordering choice

Membership is held fixed when comparing these orders.
`>` denotes one illustrative result position,
not a searched syntax or a promise that a sorter implements that relation.

### M. Relevance-mixed across folders and tracks

`Cam` track > `Camellia` folder > `Camellia Waltz` in `Camellia` >
`Camellia Waltz` in `Cult of Luna` > `Camera Obscura` folder >
`Live at Camellia` track.
The optional parent-only track is last.
This foregrounds an exact filename while keeping an early matching folder.
Its mixed types can be slower to scan for users who came only to browse folders.
See [inner M/D][inner-mixed-direct-top] and
[cover M/D][cover-mixed-direct-top].

### F. Folders first

`Camellia` folder > `Camera Obscura` folder > `Cam` track >
`Camellia Waltz` in `Camellia` > `Camellia Waltz` in `Cult of Luna` >
`Live at Camellia` track.
The optional parent-only track is last.
This gives the filesystem's folder structure priority,
but a prefix folder can outrank an exact track filename.
See [inner F/D][inner-folders-direct-top] and
[cover F/D][cover-folders-direct-top].

### T. Tracks first

`Cam` track > `Camellia Waltz` in `Camellia` >
`Camellia Waltz` in `Cult of Luna` > `Live at Camellia` track >
`Camellia` folder > `Camera Obscura` folder.
With parent inclusion,
`Another Xronixle` appears before the folders.
This favors immediate track selection,
but folders can be pushed beneath the visible viewport.
See [inner T/D][inner-tracks-direct-top] and
[cover T/P][cover-tracks-parent-end].

Within every displayed fixture,
the track title and parent detail distinguish the two `Camellia Waltz`
entries.
The candidates do **not** prove that the intended exact/prefix/contained
categories could be calculated or tie-broken consistently in the real
library.
D72 to D74 later settled folder and track action goals under review task 129.
D63 to D68 later selected query focus,
visible Back/Clear and same-query row visibility;
that scroll goal still lacks native verification.
Accessibility remains review task 118.

## Bounded verification and capture manifest

The private UI Automator hierarchy was checked for ordered result
identities in initial and scrolled captures.
Across both panels and all membership/order combinations,
the initial state included the first row;
the corrected scrolled state showed the last title and its **complete**
supporting line.
Together those positions exposed every fixed fixture row.
A cropped native light screenshot placed the inner gesture indicator at
y `2108` to `2117` and the cover indicator at y `2390` to `2399`.
The closest inner terminal detail node ended at y `2106`.
One-swipe cover M/P initially exposed only 144px of the expected 182px
parent-only supporting line;
cover T/P exposed only 53px of the 91px final folder detail.
Both truncated nodes ended before the gesture indicator,
so bounds alone had falsely suggested complete visibility.
A second upward swipe produced full 182px and 91px detail nodes,
each ending at y `2345`;
native crops then showed all lettering before the cover gesture indicator.
The active M/P and T/P scrolled images use those **two-swipe** captures.
The verifier rejected both a deliberately swapped result order and a
private clipped M/P hierarchy (positive controls).
This verifies the **synthetic fixture order** and bounded
keyboard-closed terminal visibility,
not a real result algorithm,
an actual scroll boundary,
covered-row activation or post-refocus reachability.
The parent-only row in the inner mixed/folders initial view was below the
viewport,
so the paired scrolled view is essential.

Each archived comparison PNG is named
`search-rank-accent-review-{inner|cover}-{rankfolders|rankmixed|ranktracks}-{direct|parenthits}[-end]-s200.png`.
The corresponding private capture used the same panel/order/membership
suffix under `search-accent-rank-` for initial,
or `search-accent-rank-end-` for scrolled,
with adjacent `.xml` and `.json` source files.
The source metadata records the precise candidate token
`search-deck-right-lift-retain-e2floor7p5-imeviewport-{rank}[optional -parenthits]-results-light`,
disposable device state `2` inner or `0` cover,
physical screen size,
density,
font scale and matching APK hash.
The sanitizer replaced each **entire** status region with a generic
same-panel reference,
verified zero change to every app pixel below that band,
verified the public status matches the reference,
and removed PNG metadata.
Only sanitized PNGs are published;
raw status-bearing screenshots and UI hierarchies are private.

D60 selected direct-name scope and D61 selected mixed cross-type priority
after this comparison.
No concrete relevance scorer,
deterministic tie-break,
result activation or production Search code was added.
The original AVD and IME settings remained unchanged.

[current-review]: ../question/current.html
[archived-matrix]: ../question/archive/search-ranking-before-dm.html
[inner-mixed-direct-top]: ../question/render/search-rank-accent-review-inner-rankmixed-direct-s200.png
[inner-mixed-direct-end]: ../question/render/search-rank-accent-review-inner-rankmixed-direct-end-s200.png
[inner-mixed-parent-top]: ../question/render/search-rank-accent-review-inner-rankmixed-parenthits-s200.png
[inner-mixed-parent-end]: ../question/render/search-rank-accent-review-inner-rankmixed-parenthits-end-s200.png
[cover-mixed-parent-top]: ../question/render/search-rank-accent-review-cover-rankmixed-parenthits-s200.png
[cover-mixed-parent-end]: ../question/render/search-rank-accent-review-cover-rankmixed-parenthits-end-s200.png
[cover-mixed-direct-top]: ../question/render/search-rank-accent-review-cover-rankmixed-direct-s200.png
[inner-folders-direct-top]: ../question/render/search-rank-accent-review-inner-rankfolders-direct-s200.png
[cover-folders-direct-top]: ../question/render/search-rank-accent-review-cover-rankfolders-direct-s200.png
[inner-tracks-direct-top]: ../question/render/search-rank-accent-review-inner-ranktracks-direct-s200.png
[cover-tracks-parent-end]: ../question/render/search-rank-accent-review-cover-ranktracks-parenthits-end-s200.png

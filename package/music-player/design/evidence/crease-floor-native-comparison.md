# E2 native inner player and Search floor comparison

## Correction and fixture

The first E2 form reused one existing Search A screenshot with proportional bars.
The user correctly noted that those were not alternative mockups.
That form was withdrawn without a `min_padding` choice.
The first native replacement showed only positive-results and empty Search.
Those historical six screenshots are indexed here,
but the current [`question/crease-floor-review.html`](../question/crease-floor-review.html)
now shows a separate **player with Search closed** capture for each floor too.
The initial prototype lives only on branch `prototype/music-player-theme-compose`
at commit `469819241` under
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/SearchPersistentDeckStudy.kt`.
No production Search implementation or accepted decision was changed.

The disposable `Fold_No_Hardware_Probe` ran within a Podman container
inspected at 6,442,450,944 memory bytes and 2,000,000,000 NanoCPUs.
It was unfolded in device state `2`,
at 390dpi and 200% text.
Only keyboard-closed Search states were captured;
this is **not** an IME experiment or a D50 keyboard-fit test.
A package-signature mismatch first blocked Gradle's `:app:installDebug`.
After confirming the disposable AVD identity,
only that guest's old debug app was uninstalled and the new debug APK installed.
The installed guest APK bytes matched the built SHA-256
`9e80c29ccfee72e82574a8884b3e4dca89361f05f73db3fc0231b546c971298c`.
The original AVD and Gboard settings were not changed.
The signature behavior is documented in
`doc/troubleshooting/android-37-debug-apk-signature-update.md`.

## Captured variants

These **historical Search-only** images have native 2076 × 2152px panel dimensions.
The full top 136px status strip was replaced with generic status content,
all app pixels outside it matched each raw capture exactly,
and PNG text,
profile,
EXIF and timestamp chunks were removed.
Raw status-bearing captures and full UI hierarchies remain private.

- P0,
  existing Search A with no independent floor:
  [positive results](../question/render/search-e2-floor-0-results-inner-light-s200.png)
  and [empty state](../question/render/search-e2-floor-0-empty-inner-light-s200.png).
- P14,
  proposed 14mm **total** opposing-information floor:
  [positive results](../question/render/search-e2-floor-14-results-inner-light-s200.png)
  and [empty state](../question/render/search-e2-floor-14-empty-inner-light-s200.png).
- P20,
  proposed 20mm **total** opposing-information floor:
  [positive results](../question/render/search-e2-floor-20-results-inner-light-s200.png)
  and [empty state](../question/render/search-e2-floor-20-empty-inner-light-s200.png).

The debug prototype begins with the selected A's approximate 55px half-crease
clearance plus its existing left 8dp and right 16dp content insets.
When the proposed physical total exceeds that nominal spacing,
it divides the additional inset evenly between the left deck/browser host and
right Search pane,
then adds one dp per side to cover measured native rounding in this fixture.
This is **one tested allocation**;
E2 itself does not require equal exterior margins or an empty painted stripe.
A 10mm floor would not visibly alter this sample because its current nominal
separation already exceeds 10mm.
P14 and P20 were selected to show actual space and wrapping consequences,
not because a standard supplies those thresholds.

## Measured sample and limits

In the keyboard-closed **empty** captures,
UI Automator reported the left timer `4:35` right edge and the right
`Search your music` heading left edge as follows:

- P0:
  x `965` and `1132`,
  projected horizontal node-box gap `167px` (approximately 11.35mm).
- P14:
  x `944` and `1153`,
  projected gap `209px` (approximately 14.20mm).
- P20:
  x `900` and `1197`,
  projected gap `297px` (approximately 20.18mm).

Those boxes occupy different vertical positions;
the quoted gaps are **not** nearest same-height pairs or measurements of
actual glyph ink.
The physical panel width and visible 7.5mm crease are estimates.
All captured empty and positive states retained Folders,
Open,
the actual upper-left browser,
Back/query/Clear,
the right Search region and the complete four-mode deck.
At P14 the first browser rows kept their P0 wrapping in these captures.
At P20 the narrower browser wrapped `Celldweller` to a later line and
`Clown Core` fell below its initial viewport despite the keyboard being closed.
The browser was not replaced by a caption;
its scroll reachability in this new variant was not tested.

The screenshots show actual **after-state** layout changes under one APK.
They do not certify every meaning-bearing pixel,
long names,
other text scales,
keyboard-open fit,
focus behavior,
activation or accessibility traversal.
The numeric floor required the user's design decision;
compliance at that floor still needs separate later verification.
The user stated **7.5mm total minimum even on a future narrower-crease
device**,
then required re-asking because the form omitted views with Search closed.
No numeric `min_padding` decision was recorded from that incomplete review;
the later confirmation after player views is documented in the corrected set.
The historical player preview still uses fixed 414dp panes and a 24dp stripe,
so it cannot stand in for a native E2 player-floor variant.
An initial debug-only Search-closed player variant painted a gray central
rectangle by setting the inset Row's background to `palette.window`.
The user rejected that reading of E2:
7.5mm constrains informational marks,
not structural surfaces,
borders,
padding or hit regions.
Those player captures are withdrawn and must remain private.
The next debug-only prototype (`c15a2d5b5`) retains full-size player
surfaces,
current-row backgrounds,
dividers and hit regions;
only the right-side title and track text receive extra information clearance.
Its Search page likewise retains the full left browser/deck and right
surface while shifting right-side informational content.
Commit `678fd7d8f` also keeps the full 200% mode targets in the
Search-closed player.
A private P7.5 native sample shows no gray stripe,
a complete last mode at `[73,1904][965,2035]`,
and a right heading beginning at x `1106`,
just outside the approximate crease endpoint x `1093`.
The corrected nine-state capture set and current form are described next.
No floor decision was recorded from the earlier incomplete review.

## Corrected player and Search comparison with P7.5 selected

Debug-only commits `c15a2d5b5` and `678fd7d8f` moved the E2 floor owner to
**meaning-bearing content** in both the Search-closed player and Search page.
On the player,
the actual folder browser and full-height playback deck fill the left half.
The right track surface,
highlighted current-row background,
row dividers and click bounds remain full-width while its title and track
text use a right-side information inset.
The Search page keeps its full left browser/deck and right surface;
right-side content and its associated control targets move where the floor
demands it.
Those hit bounds are not measured as part of E2's informational floor.
There is no gray structural stripe in the corrected captures.
The user explicitly reconfirmed **P7.5** after receiving the full player,
empty Search and positive-results comparison.
This is an independent 7.5mm **total opposing-information minimum** even on
a future device with a narrower physical crease;
if the crease is wider,
its actual width still governs.

The inspected installed APK bytes matched SHA-256
`d301bebdd35d053ee63cbd9a8500949fb97b8be5dc26c9bb971be1493a3462b5`.
The same disposable unfolded AVD was used at 390dpi and 200% text,
with the existing 6 GiB/2 CPU container cap.
No keyboard was shown or changed;
no production code or original AVD setting was touched.
Each of these nine physical 2076 × 2152px screenshots has its **entire**
136px status strip replaced with generic status content.
Pixels below the strip match their private raw captures exactly,
and text,
profile,
EXIF and timestamp PNG chunks were removed.
Only sanitized images are linked here:

- P7.5,
  **selected independent 7.5mm total** minimum even on narrower-crease
  devices:
  [player, Search closed](../question/render/search-e2-complete-7p5-player-inner-light-s200.png),
  [empty Search](../question/render/search-e2-complete-7p5-empty-inner-light-s200.png),
  [positive results](../question/render/search-e2-complete-7p5-results-inner-light-s200.png).
- P14,
  unselected proposed 14mm total minimum:
  [player, Search closed](../question/render/search-e2-complete-14-player-inner-light-s200.png),
  [empty Search](../question/render/search-e2-complete-14-empty-inner-light-s200.png),
  [positive results](../question/render/search-e2-complete-14-results-inner-light-s200.png).
- P20,
  unselected proposed 20mm total minimum:
  [player, Search closed](../question/render/search-e2-complete-20-player-inner-light-s200.png),
  [empty Search](../question/render/search-e2-complete-20-empty-inner-light-s200.png),
  [positive results](../question/render/search-e2-complete-20-results-inner-light-s200.png).

In the **player** captures,
the left `4:35` timer node ended at x `965` for each floor;
the right `Camellia` heading began at x `1106`,
`1173` and `1262` for P7.5,
P14 and P20.
Their projected horizontal node-box gaps were 141px,
208px and 297px (approximately 9.58mm,
14.13mm and 20.18mm).
In the **empty Search** captures,
the right `Search your music` heading began at x `1132`,
`1172` and `1260` against the same left timer edge,
for projected gaps of 167px,
207px and 295px (approximately 11.35mm,
14.07mm and 20.05mm).
The boxes are at different vertical positions;
none is a nearest-pair glyph-ink measurement or proof of all-content E2
compliance.
The selected 7.5mm crease interval remains an approximation.

A read-only inspector checked every captured app node with nonempty text or
content description against approximate x `[983,1093)`.
None of the nine states reported an intersecting labeled node box.
An in-memory control moved the P7.5 player `Camellia` box from
`[1106,159][1436,268]` to `[1000,159][1330,268]`;
the same inspector then reported exactly that crossing text node.
The accepted debug track-row candidate starts with `dark-`,
and `TrackRow` sets its leading icon slot to `null` for that family in
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/DesignCandidateActivity.kt`.
The P7.5 player hierarchy put the Search and Settings icon descriptions at
x `[1788,1847]` and `[1906,1965]`,
with Pause at x `[491,550]`;
those icon bounds were not near the crease.
This bounded node check does **not** certify paint enclosure,
unlabeled decorative pixels or every future icon.

The complete Search-closed and open four-mode deck kept its final mode at
`[73,1904][965,2035]` in the sampled P7.5 captures.
The corrected player keeps Folders,
Open,
the same browser,
track list and Search action;
no first-screen browser wrapping change was observed among these floors.
P20 moves the empty Search instruction far enough right that `your`
wraps onto the line containing `library.` in the captured state.
Long names,
100% text,
painted glyph bounds,
keyboard-open fit,
scroll reachability,
activation and screen-reader traversal still need separate verification.
The selected P7.5 floor does not ratify these debug inset mechanics or
require surfaces,
borders,
padding and hit regions to avoid the crease.
A naturally wider information gap remains permitted;
P14 and P20 were rejected as **hard minimums**,
not prohibited whitespace.

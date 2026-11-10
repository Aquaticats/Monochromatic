# Selected Fold Search A informational clearance on the inner panel

## Measured boundary

The accepted [Search A review](../question/current.html) uses the physical
2076 × 2152px Pixel 9 Pro Fold inner panel at the measured 390dpi setting.
E2 places the approximate visible dent at x `[983,1093)`px,
centred on x `1038`.
These endpoints inherit uncertainty from the published panel diagonal and
the user's approximately 7.5mm visible-width estimate.
Surfaces,
dividers,
backgrounds and hit regions may cross this band;
meaning-bearing marks must remain clear.
See [device-metrics.md](../device-metrics.md) and
[decisions.md](../decisions.md) under E2.

## Native selected-A measurements

The private debug-only prototype worktree's
`package/music-player/design/question/evidence/search-selected-inner-*.xml`
records UI Automator nodes for Search results,
typing and empty states in light/dark at 100%/200% text.
Each inner fixture's metadata identifies prototype commit `25dc2fb01`,
a 300dp system-managed **debug IME** for typing,
and the 390dpi physical panel.
For the active light 200% typing state,
the sanitized PNG and the prototype capture had identical app-region pixels
from y `136` to the bottom of the screen;
a disposable one-pixel edit produced a nonzero image difference.
The linked active review uses sanitized physical-panel PNGs;
the raw XML is not a public asset and must not be inferred from the
review's embedded bitmap alone.

Across the measured inner fixtures,
text and labelled app-node bounds did not intersect x `[983,1093)`.
The rightmost left-side text bounds ended at x `965` (`4:35`),
18px before the approximate dent start.
The nearest right-side text bounds began at x `1132` in an empty Search
state,
39px beyond the approximate dent end.
The left/right extrema yield a **projected horizontal node-box gap** of
167px,
compared with the approximate 110px dent width.
These extrema can occur at different vertical positions;
this is not a measured nearest pair of glyphs.
The 18px and 39px bounds-to-crease margins are not guaranteed physical
safety margins because the dent location is approximate.
The right-side positive result labels began at x `1220`;
that result state has a wider opposing bound gap than the empty state.

For the active 200% typing capture,
UI Automator placed `4:35` at `[842,629][965,707]`,
Open at `[744,168][894,259]`,
and the matching right-side result labels at x `1220` or farther.
The [light typing screenshot](../question/render/search-selected-review-inner-typing-light-s200.png)
visually shows no meaning-bearing app mark in the central band above the
debug IME;
the continuing pane/background boundary is structural and allowed.
The [dark empty screenshot](../question/render/search-selected-review-inner-empty-dark-s100.png)
shows the empty-state heading near x `1132` without entering the dent.
A separate band scan from y `320` to `1320` found no near-black pixels
(`max(R,G,B) < 140`) in the light typing/result samples and no near-white
pixels (`min(R,G,B) > 180`) in the dark empty sample.
An in-memory dark or bright rectangle drawn across the band changed its
respective count from zero to 100,
so the limited color detector can register a deliberately crossing rendered
mark.
Other text colors,
icons,
animations and longer content still require independent visual review.
UI Automator text bounds may clip glyph paint;
[open-questions.md](../open-questions.md) records a separate stress title with
paint beginning at x `58` while its clipped accessibility bounds began at
x `73`.
This band scan and the sample crops do not prove that every node rectangle
encloses its ink.
The debug IME itself is system-owned test input,
not an app content placement to certify under E2.

## Detector control and remaining choice

The read-only XML inspector classified a text node as crossing when its
bounds began before x `1093` and ended after x `983`.
It reported no crossing app labels in the selected inner fixtures.
A disposable **in-memory** positive control changed one `4:35` box from
`[842,629][965,707]` to `[842,629][1000,707]` without changing repository
files;
the same crossing check changed from zero to one.
This proves the box detector can find an injected intersection,
not that UI Automator bounds perfectly describe glyph ink or every icon.

E2 specifies the opposing-information distance as
`max(min_padding, crease_width)` in physical units.
The **node-box** gap of 167px exceeds the approximate 110px crease width,
but E2 concerns meaning-bearing paint,
not the whitespace between accessibility rectangles.
If those boxes enclose all relevant glyph ink,
their projected gap is a conservative lower bound for those marks;
that enclosure has not been verified for every node.
Neither the 167px node-box measurement nor its approximately 68.5dp
conversion at 390dpi determined the later user-selected 7.5mm
`min_padding` term.
The accepted 12dp mode-button content padding and Material field padding
have different ownership and cannot be transplanted as an opposing-pane
minimum.
If a chosen minimum later exceeds the **measured painted** clearance
in some state,
that state needs a visible layout response;
this before-state reading alone does not establish which minimum fits.
The floor preference was later settled as 7.5mm total opposing-information
separation;
long result names and other screen states remain verification work.
The first policy question at
`package/music-player/design/question/crease-floor-review.html` was
withdrawn after the user noted that it showed just one existing mockup.
Its proportional bars were not native layout variants.
The P0/P10/P12 menu was **not answered**;
no numeric floor was selected at that time.
After another correction requested Search-closed player views and rejected
an interim gray structural stripe,
the replacement at the same review path now embeds distinct debug-only
native Compose captures for P7.5,
P14 and P20 in the player,
empty Search and results states.
The [native floor study](crease-floor-native-comparison.md) records their
after-state informational box positions and continuous surfaces.
The user first stated 7.5mm as the independent **total** floor even on a
future narrower-crease device,
then explicitly re-confirmed **P7.5** after seeing the player and Search
captures.
Decorative marks and hit regions remain outside this numeric rule;
painted glyph clearance and other states still require verification.
No new IME experiment was performed.

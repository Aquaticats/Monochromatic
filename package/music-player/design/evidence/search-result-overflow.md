# Pixel 9 Pro Fold Search result-name and list-overflow study

## Design-only fixture

The accepted Search A keeps an integrated Back/query/Clear header,
results on the right inner pane,
and the same upper-left folder browser above the bottom-left deck.
The cover uses one full-width Search page.
A debug-only result-data stress candidate on prototype commit `c7e0552ee`
uses a spaced synthetic folder title,
an unbroken synthetic track title,
a longer secondary description and additional folder rows.
The tested APK SHA-256 is
`989346cd9b7dde18dbec03255368f6aa4e1fb2744828e04e4ecd4c8bddfabc10`.
It changes no production Search result source or ranking.

The private `Fold_No_Hardware_Probe` AVD was unfolded or folded using its
measured device states at 390dpi and 200% text.
Android UI Automator hierarchies and native-panel PNGs remained in private
scratch because their system status content was not sanitized.
The source variants are
`search-deck-right-lift-retain-overflow-results-light` for the inner panel
and its delegated `search-layout-docked-results-overflow-light` cover form.
A same-APK plain-A control retained `Folders`,
Open,
`cam`,
the complete deck and the ordinary short result titles before the stress
candidate was opened.

## Inner panel at 200% text

With the keyboard closed,
the long spaced folder name wrapped within right-side text bounds
`[1220,338][2037,956]`.
The unbroken track name wrapped by character within
`[1220,1095][2037,2022]` rather than widening into the approximate
x `[983,1093)` crease.
The large first rows push other matches lower in the list,
so vertical scrolling remains part of the result experience.
After scrolling,
the final synthetic `Camellia archive 18` label was fully visible at
`[1220,1856][1858,1959]` while the left browser and mode bounds remained
unchanged.

The 300dp system-managed **debug IME** began at y `1421`;
this is not a real-Gboard pass.
With `cam` focused,
the same left browser remained visible and the final mode occupied
`[73,1270][965,1401]` above that IME.
Independent right-list swipes then brought the last result to
`[1220,1203][1858,1306]` above the IME,
without moving the browser or deck.
This tests result scrolling in the selected A composition,
not activation or ranking.

## Folded cover at 200% text

With the keyboard closed,
the long spaced folder title occupied `[127,350][993,865]` and the
unbroken track title `[127,996][993,1923]`.
The final synthetic folder row was reachable at `[127,2151][765,2254]`
after scrolling,
above the screen's navigation region.

A visible 300dp debug IME began at y `1693` after focusing `cam`.
At the cover list's maximum downward scroll,
`Camellia archive 18` still occupied `[127,2151][765,2254]`,
**behind** that bottom keyboard.
A reverse swipe visibly moved the list back to earlier rows,
so the failure was not an unresponsive gesture;
returning to the scroll end still left the last rows behind the IME.
The physical cover screenshot showed the list cut at the keyboard top,
with only results through `Camellia archive 15` readable in that sample.
`SearchLayoutStudy.kt` currently gives cover results a vertically scrolling
column with navigation-bar padding but no keyboard-height reservation.
This is a **debug-prototype viewport finding**,
not a claim about production Search (which is not yet implemented).
D55 accepts results obscured by **floating real Gboard**,
not these results hidden behind an ordinary bottom-aligned keyboard.

## Opt-in cover viewport comparison at 200% text

Prototype commits `f4a35e817` and `c4cf948e4` add a **separate,
unaccepted** `-imeviewport-` comparison to the selected Search candidate.
The selected Search cover delegation keeps the positive-heading suppression,
so the first result still begins directly beneath the integrated header.
Only the cover result viewport receives Compose `imePadding()`;
Back,
query and Clear remain in the fixed header.
The [Android Compose insets guide][compose-insets] describes keyboard inset
padding as resizing scrollable content during IME changes and consuming
nested system-bar insets.
That describes the intended API behavior,
not a substitute for the rendered device check.

The newly built and installed debug APK matched SHA-256
`0801ea3f44ef84163cbddb2b2f37fbddbb1c293148440381f6f899be7eaee7af`.
On the disposable folded 1080 × 2424 panel at 390dpi and 200% text,
its keyboard-closed result viewport was `[0,330][1080,2365]` and the
first long title started at y `350` with **no heading**.
After the query took focus,
the 300dp debug keyboard began at y `1693`;
the viewport ended at that same y instead of continuing behind it.
A within-list upward swipe visibly moved row 1 from y `2236` to y `1246`,
validating that this fixture responds to scrolling.
At the bottom,
`Camellia archive 18` occupied `[127,1479][765,1582]` and its supporting
text occupied `[127,1582][847,1673]`,
both above the keyboard.
A further upward swipe left row 18 at exactly the same bounds.
The native 1080 × 2424 screenshot showed the complete last title,
its folder icon and supporting copy immediately above the debug keyboard.
A first explicit screenshot inadvertently targeted the black **inactive
inner** display;
the verified cover capture used display ID `4619827551948147201`.
These controls support a **settled 200% debug-IME viewport pass**,
not a real-Gboard or frame-transition pass.
The APK's signing certificate differed from the prior installed debug APK;
only the disposable AVD's app was uninstalled and reinstalled before this
same-hash comparison.

## Same-APK controls and other settled keyboard states

The **opt-out** cover candidate on that installed APK kept the result list
at `[0,330][1080,2365]` while the debug IME visibly began at y `1693`.
A swipe moved row 1 upward from y `2236` to y `1263`;
after scrolling to the end,
row 18 stayed at `[127,2151][765,2254]` after another within-list swipe.
The native cover screenshot showed results through row 15 above the dark
keyboard,
with rows 16 to 18 unavailable until the keyboard was hidden.
Thus the opt-in candidate's smaller viewport has a same-binary,
same-data failure control,
not merely a comparison against an older build.

At **100% text**,
the opt-in cover viewport again ended at y `1693` under the debug IME.
Its last row title was `[127,1542][487,1601]` and supporting text
`[127,1601][513,1650]`;
the screenshot showed both above the input surface.
At 200%,
hiding the keyboard expanded the viewport to y `2365` and moved
row 18 to y `[2151,2254]`;
refocusing shrank the viewport again,
and a further list swipe returned the last title and support text above
that keyboard.
Refocus did **not** automatically preserve the last row's visible position;
continued scrolling recovered it.

With **real settled full-width Gboard** on the cover,
the opt-in viewport ended at y `1605` at both 100% and 200% text.
After scrolling,
row 18 and its support text occupied y `[1454,1513]` and
`[1513,1562]` at 100%,
then `[1394,1497]` and `[1497,1588]` at 200%.
Both physical-panel screenshots showed the complete final row above Gboard.
A real `m` key tap changed `cam` to `camm`,
proving the query field still accepts keyboard input;
the debug fixture deliberately shows results **only for exactly `cam`**,
so the resulting empty state does not test real search filtering or ranking.

On the **inner** panel,
the opt-in marker did not alter the selected A branch.
At 100%/200% text,
the spaced and unbroken stress titles remained bounded to x
`[1220,2037]`,
while Folders,
Open,
the actual folder browser,
the full bottom-left deck and the right query remained in their selected
positions in the captured closed states.
At 100% with the debug IME beginning at y `1421`,
right-side scrolling exposed row 18 at y `[1270,1329]` and its support
text at `[1329,1378]` while the browser and the complete deck stayed
visible.
At 200% with **real floating Gboard**,
the key surface still obscured parts of right-pane result labels as well
as the D53-exempt deck.
The right-pane overlap is a **separate open decision** (#122),
not an implied extension of D53 or the accepted D55 folded-cover case.
A settled real split-keyboard test on this stress fixture was **not**
performed;
the earlier split-keyboard findings used short sample results.

These captures support wrapping,
scrollability and the opt-in cover viewport in the named **settled** states.
They do not test the IME animation's intermediate frames,
all keyboard heights,
result activation,
ranking or accessibility.
The inner text-node boxes stay to the right of the approximate crease x
`[983,1093)` in the inspected stress states;
the screenshots visibly preserve an unlettered central band,
but neither measurement certifies every glyph-ink edge or compliance with
the later user-selected **7.5mm total** E2 floor.
The cover viewport comparison was unaccepted at capture time;
D56 later selected its direction without authorizing production code.
Raw status-bearing screenshots and hierarchies remain private.

## Historical R/C review and D56 selection

`package/music-player/design/question/cover-viewport-review.html` compares the
same-APK 200% opt-out failure control with the opt-in end-of-list state;
it does not replace the selected A review.
Its public captures are `question/render/search-cover-viewport-control-s200.png`
and `question/render/search-cover-viewport-refinement-s200.png`.
Every pixel in the top 151px status strip was replaced with the same
visually checked generic 9:41 clock and signal/battery icon strip from the
previously published selected-cover review.
The source-to-public pixel difference below that strip was exactly zero in
both captures;
no PNG text,
profile,
EXIF or timestamp chunks remained.
The reference strip itself contains no notification or account text.
The user answered **R**,
selecting the cover-only keyboard-aware results viewport as D56.
The R/C comparison remains historical evidence;
`question/current.html` presents only the active design.
D63 to D68 later selected Back/Clear/focus and same-query row visibility;
native restoration after refocus remains unverified.
Result actions and TalkBack traversal remain separate open questions.
This selection does not authorize production implementation.

## Inner floating-result exception question

`package/music-player/design/question/floating-results-review.html` shows a
separate sanitized 2076 × 2152 native inner-panel capture from the same
long-results debug fixture with **real floating Gboard**.
Its middle right-pane result labels are partially covered while the `cam`
query remains visible;
later results are visible below the floating keyboard.
The entire top 136px status strip was replaced from an already published
selected-A capture with generic 9:41 and status icons.
All app pixels beneath the status strip match the private capture exactly;
PNG text,
profile,
EXIF and timestamp chunks were removed.
The form asks whether to accept **this specific observed** floating overlap
or leave resolution of this same state open within selected A;
it does not demand universal floating-keyboard readability or claim a
solution has been built.
Recovery by scrolling or another action was not verified in this fixture.
D53 concerns the separately accepted deck overlap,
and D55 concerns the folded cover.
The user answered **A**;
D57 accepts only the observed inner floating-keyboard result-lettering
overlap while the query remains clear.
It does not establish scroll-reachability or activation of obscured rows,
and does not approve other floating placements or ordinary docked input.
The A/B comparison remains historical evidence;
the active selected-A review shows the accepted state only.
No additional IME experiment was performed for this artifact.

[compose-insets]: https://developer.android.com/develop/ui/compose/system/insets-ui

# Search header and result-column alignment on the disposable Fold

## Scope and provenance

D58 settles horizontal columns in selected Search A:
Back and result icons share a visual center;
the query and result titles share a text start.
This is a design-only study on `Fold_No_Hardware_Probe`,
not production Search or a new keyboard experiment.
The only after-state APK used here has SHA-256
`bf9a51c42facfca969e275d5fe55769656c451900f57689dc7c24190e8d6d2ef`.
A hash of the installed `base.apk` matched the locally built debug APK.
The prototype source correction is commit `baa37caaf` on
`prototype/music-player-theme-compose`.
The native inner panel was 2076 × 2152 physical px,
the cover 1080 × 2424,
and the disposable AVD density 390dpi.

Each linked image is a native keyboard-closed screenshot with the full
status region replaced by a generic reference of the same dimensions;
app pixels are unchanged and PNG metadata removed.
The raw status-bearing frames and UI Automator hierarchies stay private.
At the D58 checkpoint,
the short-results fixture included a parent-only illustrative track with
no activation handler.
D60 later excluded that track from the selected result set;
this historical alignment image does not override the active D/M
membership or order in `package/music-player/design/question/current.html`.

## Before and after at native resolution

Before D58,
`PersistentResultLine` and `SearchLayoutRow` placed a 24dp result icon
before a 12dp spacer;
Back occupied a 48dp leading icon target.
At 200% text on the inner panel,
query `cam` began at x `1249` and result title `Cam` at x `1220`;
on the cover the starts were x `156` and x `127`.
The debug-only correction centers each 24dp icon in a 48dp result slot
and moves result text directly after that slot.

After D58,
UI Automator reported query `cam` and result `Cam` both at x `1249`
on the inner panel and both at x `156` on the cover,
at **both 100% and 200% text**.
In the selected two-row fixture,
`Camellia` likewise starts at the corresponding query x on both panels
and both scales.
The [inner selected Search at 200%][inner-200] and
[cover selected Search at 200%][cover-200]
show the actual upper-left browser and full deck where applicable.
The [inner 100% view][inner-100] and [cover 100% view][cover-100]
show the same selected composition with narrower lettering.
Light and dark variants are available in the
[active selected-only review][current-review].

For paint rather than merely layout bounds,
a fixed grayscale threshold (60%) isolated icons in native light-scheme
PNG crops confined to the leading column.
The inner 200% Back arrow paints across x `1171` to `1209`,
the first folder icon across x `1166` to `1214`,
and the track note across x `1176` to `1204`.
Their horizontal centers are all x `1190`.
The cover Back arrow paints across x `78` to `116`,
the folder icon x `73` to `121`,
and the note x `83` to `111`,
all centered at x `97`.
Different glyphs retain different painted widths.
The 100% after-state note and Back centers also match on both panels.

A positive control on the previous selected 100% captures showed the
old first folder icon centered at x `1161` inner and x `68` cover,
while Back was centered at x `1190` and x `97` respectively.
On the after-state 100% captures,
the folder centers moved to those Back centers;
thus the pixel probe demonstrably detects the prior 29px offset.
This test does not assert identical icon shapes or enforce alignment of
every future icon asset.

The header's Back `IconButton` remains sized to 48dp in the debug source.
The 48dp result slot is **decorative layout**,
not proof of an interactive
result target:
`PersistentResultLine` has no click handler.
On the inner panel,
the corrected first result icon begins at x `1166`,
to the right of the approximate crease interval x `[983,1093)`.
That measured result says nothing universal about glyph ink,
other text,
longer titles,
future devices,
keyboard-open fit or accessibility.
The debug activity dispatches the `search-deck-*` candidate to
`SearchPersistentDeckStudy.kt`.
Its **cover** branch delegates to `SearchLayoutStudy.kt` and renders
`SearchLayoutRow`;
therefore these native captures exercise both the selected inner
`PersistentResultLine` and delegated cover `SearchLayoutRow` alignment.
Historical alternate unfolded branches of `SearchLayoutStudy.kt` were not
exercised by this selected comparison.

## Unchanged and unverified behavior

The inner screenshots retain the selected E2 P7.5 floor,
left folder browser and complete deck;
the cover screenshots retain the full-width Search destination and D56
viewport marker.
They do not validate activation,
ranking,
TalkBack,
arbitrary keyboards or IME transitions.
The active review retains pre-D58 keyboard-open and older E2 evidence for
their bounded findings with explicit provenance labels,
not as final alignment captures.
No production Search file or original AVD was changed.

[inner-200]: ../question/render/search-selected-aligned-review-inner-results-light-s200.png
[cover-200]: ../question/render/search-selected-aligned-review-cover-results-light-s200.png
[inner-100]: ../question/render/search-selected-aligned-review-inner-results-light-s100.png
[cover-100]: ../question/render/search-selected-aligned-review-cover-results-light-s100.png
[current-review]: ../question/current.html

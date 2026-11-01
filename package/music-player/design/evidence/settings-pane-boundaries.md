# Native Settings-pane verification boundary

## Withdrawn on 2026-10-05

This document describes a study that is no longer a current design.
D86 made `Strip common prefixes from filenames` and `Resume where I left off`
always-on behaviour and not settings,
which withdrew the two-row pane the same day it was published.
D87 keeps the Settings page and leaves it empty until D81's template
configuration is designed,
and the human said an empty page needs no study.
The viewer carries a withdrawal notice,
and everything from `Purpose and accepted pane` on is kept as the record of
what was built and checked.
D88 removed the builder's digest rules on the same day.
The rule and rejected-input counts given in this document are those of the
publication.
The builder now has 94 rules and its consumer test rejects 130 changed inputs;
both passed on the withdrawn page,
as did a four-context offline browser check,
and the review record carries that recheck.
One diagnostic became worse with the removal and was left so on a withdrawn study:
if one of the Search images the left half is compared with is missing,
validation now stops with the file system's own `ENOENT` and no rule name,
and a Search image with a byte added after its end is accepted.

## Purpose and accepted pane

This design-only study carries the accepted Settings pane into the current
Fold player and its light/dark schemes.
It does not reopen Search or D81's template editor,
whose fields and controls remain undesigned (section 11e of
`open-questions.md`).

D11 selects `candidates/settings-a.dc.html`:
three flat switch rows and a closing sentence saying the pane is short.
D12 removed settings-b's analysis-status row,
and D47 superseded the command bar together with D21's fourth row.
D84 (2026-10-05) then removed settings-a's third row,
`Analyse true peak in the background`:
true-peak analysis is automatic and not optional,
so Settings provides no analysis switch.
D85 (2026-10-05) removed the closing sentence.
The study therefore draws exactly two rows,
`Strip common prefixes from filenames` and `Resume where I left off`,
and nothing after them.
Scene `accepted` is the mock's state for those rows:
both switches on.
Scene `inverse` puts both switches off.

The mock is an 860 by 600px desktop window.
D49 makes the Fold panels the layout source,
so its proportions do not govern this study,
and its 40px Back control does not override the 48dp layout floor.
Each row is the platform's Material 3 list item with a 52 by 32dp switch;
the whole row is the two-state control,
so the row supplies the 48dp target rather than the switch's 40dp state layer.
The unselected switch is the platform's,
with a 16dp handle and an outline,
not the mock's unoutlined 24dp handle;
`md3-tokens.md` records that re-check.

## Adopted Fold placement

No accepted decision placed Settings on the Fold.
D50 keeps the deck visible while unfolded,
D51 puts the Search page on the right half with the same folder browser and
deck kept on the left,
and production Settings already replaces the track region while other
chrome stays.
The study follows those:
on the inner panel Settings takes the right half under a 72dp header with a
48dp Back target and the page title,
and on the cover it is one full-width page with the same header.
This placement was adopted from those decisions,
not separately chosen;
the viewer says how to object to it.

## Isolated artifact and input owner

The current source is prototype commit
`67eae28d1f31689bcc9f132e4db97ec87426e572`,
with APK SHA-256
`5f3a23911f2a5c32859a78e533704298bfb5afa042cf968f4fc82691ef35e9ab`.
`SettingsPaneActivity` extends `ComponentActivity`,
not the production activity.
Its merged manifest disables production activity/services and removes
WorkManager auto-initialization.
The player's own Settings button is not wired;
the page opens from an authored scene or one explicit debug event.

The immutable record holds two switch values,
the order and copy of D11's remaining rows and one checked toggle per row.
The removed analysis toggle is rejected like any unknown event.
Its 12 pure fixture tests passed,
and fresh unknown-scene,
copy-drift,
row-position-swap,
unknown-event,
cross-row-toggle and reinstated-analysis-toggle mutants each failed their
intended test before exact restoration.
Both authored scenes set the two switches equal,
so the row-position test also checks a mixed record;
only that record tells the rows' fields apart.

## Superseded builds and interrupted visits

The first owned visit captured a three-row build,
APK `40d0b0e4ab592e920372be8d4771fb9381e4e885501b54f575ac3db567a11f0d`,
and passed every measured rule and input check.
Fresh inspection then rejected it.
In dark,
the header's Back glyph and title,
and the retained deck's title and transport buttons,
drew black on the dark page:
nothing above them supplied a content colour,
while the accepted Search page is hosted inside a region that does.
Each separator was also a pixel different across the fold connector,
because the opaque row fill began after the connector inset.
Measured rectangles could not show either defect.
Its captures stay private.

Prototype `483f16cdd4c0bda6269ae2e4666732db6407ce75`,
APK `85e4a2080d1d737eb01a16bdcc5172bcc7103fe014770891d68d3cdf1854ca50`,
fixed those defects and was inspected and published with three rows and the
closing sentence.
D84 and D85 withdrew that publication;
its files were replaced by the current cohort and remain in git history.

Some visits supplied no evidence.
The second,
on the withdrawn build,
and the fifth both met a 15 s `podman inspect` bound that expired under host
load;
both bounds are now 120 s.
The fourth stopped when the guest killed a hierarchy dump during a startup
dialog.
The fifth also stopped a capture because a late configuration change
recreated the host after its launch;
every launch is now watched and relaunched within a bound,
and each capture records the attempts its launch used.
The sixth captured and was inspected on both panels,
then its emulator crashed in the GL translator during the input stage,
before restoration;
a recovery boot restored that visit's own recorded baseline.
Each of those visits ended with every recorded field restored and no matching
runtime remaining.

## Native fit

The seventh visit captured 24 views:
both panels,
light and dark,
100% and 200% text,
the closed player and both switch scenes.
At the measured 390dpi density the header is 176px high,
the Back target 117 by 117px and every switch 127 by 78px.
Rows are 216px high at 100% text and 435 to 629px at 200%,
all above the 117px floor.
No row title,
supporting line or page title reports overflow.
On the inner panel no title,
supporting line or switch starts inside the fold connector.
Neither a switch position nor the theme changes any measured rectangle.

No column scrolls on either panel at either text size,
and both rows are wholly shown above the navigation area.
Below the last row's separator,
1324px of empty page remain on the inner panel at 100% text and 589px at
200%;
on the cover,
1599px and 967px.
The viewer's builder re-measures from the images that this region is one
uniform colour,
so nothing is drawn where the closing sentence used to be.
Rectangles are device layout reports,
not glyph bounds or accessibility acceptance.

## Native input

Inputs passed in all eight panel,
theme and text-scale environments.
A touch on each row toggled it once and a second touch toggled it back.
Each toggle changed pixels only inside its own row,
redrew its switch,
and left every rectangle unchanged;
toggling back restored the app pixels exactly.
No row needed scrolling to be reached.
The Back target closed the page once,
and the closed player matched a freshly launched closed player in geometry and
app pixels.
One authored reopen restored the opened page exactly,
then system Back closed it once and the closed player matched again.
The host was created once across all of those inputs,
and a per-step event ledger matched every capture.
Changed-pixel,
changed-geometry and changed-count controls were rejected.

Opening Settings changes about 79,000 of 2,011,644 left-half pixels at 100%
text and about 70,000 at 200%:
the folder browser reflows narrower,
as it does for the Search page.
The live input matrix was recomputed offline from its 72 retained
captures,
and every field of its manifest was reproduced.

## Fresh inspection

All 12 light/dark full-region pairs were inspected on 2026-10-05,
after every crop was re-derived from its private raw capture with a
changed-byte control.
An enlarged detail confirmed that the last separator has the same colour
inside the fold connector as beyond it.
In dark the header and deck text are drawn in the theme's light text colour,
as on the accepted Search page,
and the page below the rows is true black.

These observations are recorded for the human,
not turned into a ballot:
at 200% text on the inner panel `Resume where I left off` leaves `off`
alone on its second line;
with two rows most of the page below them is empty,
most visibly on the cover at 100% text;
opening Settings narrows the left folder browser;
and the 24dp Back arrow reads small beside the 200% title.

## Verification limits

Every switch position is authored debug state.
No preference is stored,
no filename prefix is stripped,
no session is restored and no analysis runs.
D84 makes true-peak analysis automatic and not optional;
when it runs is not decided,
and this study neither draws nor exercises it.
D81 places template configuration in Settings;
no template entry is drawn,
and that stays with the 11e design round.
No new IME,
TalkBack or keyboard-default experiment is included.

Full Android lint is not passing.
Its five inherited errors are three `MissingSuperCall` and two `NewApi`
findings in unchanged files;
65 warnings and two hints remain,
and no finding names a Settings study file.
The measured adapter was LLVM 20.1.2 llvmpipe software rendering despite the
requested host GPU path;
startup reported that the GPU cannot be used for hardware rendering.

## Privacy and restoration

Raw screenshots,
Android hierarchies,
logs and generated identities remain private.
Published crops start at the measured application root,
keep exact RGB,
are opaque 8-bit PNGs with essential chunks only,
and show only authored labels and the anonymous gesture handle.
One image,
the inner closed player in light at 100% text,
shows no gesture handle in the system's navigation strip.
It was the first capture of the visit;
the same state captured later in the input stage shows the handle,
and the cause was not established.
That strip is below the measured application root and no rule or comparison
reads it.

Only the freshly recorded fields were restored and read back:
font scale,
night mode,
device state,
accessibility state and services,
display geometry and the system image fingerprint.
The first readback matched every field.
The owner's exit status `0` was recorded by its own parent process,
and no matching container or emulator process remained.
Original AVDs and library data remain untouched.

## Publication and consumer verification

[The witness manifest](../questions/evidence/settings-pane-witnesses.json)
and its 24 PNGs carry each view's measured rectangles,
visible parts and line counts.
[The native verification record](../questions/evidence/settings-pane-native-verification.json)
holds the input contexts,
their replay,
the superseded builds,
the interrupted visits,
fixture reports,
lint totals and restoration.
The publisher removed the withdrawn three-row files only after each matched
the hash its own witness file recorded.
Publication preflights rejected eight changed inputs before any public write.
The [offline viewer](../questions/settings-pane.html) states the measured
findings from that data and the by-eye observations separately.

Its builder holds 99 named rules,
one per line.
Validation re-measures drawn switch positions,
header contrast,
the empty page after the rows and the retained left half from the embedded
images;
the left half must match the published accepted Search images.
The consumer test rejects 135 changed inputs,
at least one per rule.
Each of the 99 rules and the script-data encoding was deleted once in
a disposable copy,
and the test failed on that rule by name every time.
The pixel rules decode the embedded PNGs in process;
that decoder matched ImageMagick byte for byte on all 28 images it
reads.

Four offline Chromium desktop/mobile light/dark contexts decoded and opened
all 24 previews,
exercised every panel,
scale and theme combination,
optional blank observations,
inert adversarial notes,
stale-reply invalidation and modal zoom/pan/reset/focus.
The sixteen resulting review,
gallery,
open-boundary and modal screenshots were inspected.
Each closed-page axe audit had 22 passes and no violations or incomplete
results.
Open-dialog axe,
Firefox ESR140 and native accessibility acceptance were not exercised.
The browser closed and its owned container is absent.
The [review verification record](../questions/evidence/settings-pane-review-verification.json)
records that check of the viewer,
builder,
test,
manifest and native result.
No new preference answer is required.

The [continuation plan](../../../../doc/planning/music-player-light-settings.md)
records superseded attempts and remaining checks.

# Native scan-indicator verification boundary

## Purpose and settled input

This design-only study carries D26's accepted scan-F indicator into the
current Fold player and its light/dark schemes.
It does not reopen Search,
menu actions,
D83 overlay placement,
notification presentation,
D27 consent choices or D81's Settings-template requirement.
D84 (2026-10-05) supersedes D27's consent choices:
true-peak analysis is automatic and not optional.
D26's scan bar stands and this study is unchanged by it.

D26 selects a bottom-edge 56dp bar with analysis text and counts plus an
always-rendered 100dp-wide Pause/Resume control.
D13 makes the bar non-permanent:
it appears during analysis and leaves when analysis finishes.
D12 rejects a permanent completed-analysis status row.

The accepted source is the right frame of
`candidates/scan-ef.dc.html`.
Its historical mock uses a 56px row,
a fixed 100px control and a single-line ellipsized status label.
The control changes between Pause and Resume without changing its slot.
Its 40px control height is not authority to violate the current 48dp
layout floor.
The accepted source reserves the bar row in normal layout;
this is not D83's floating overlay rule.

## Isolated artifact and input owner

The current source is prototype commit
`25a95411750c5c5356631fc508513c42a81ad1c4`,
with APK SHA-256
`c06ec80240e41641fee1fdec79313a8294b9595544740145f4fa6f2b1f77ddf1`.
`ScanIndicatorActivity` extends `ComponentActivity`,
not the production activity.
Its merged manifest disables production activity/services and removes
WorkManager auto-initialization.

Prototype `a10baf8b4` adds validated immutable scan records and checked
synthetic events.
The 21 pure fixture tests passed,
with fresh unknown-scene,
completed-bar,
paused-progress and stalled-running mutants failing their intended tests
followed by exact restoration.

The host owns one immutable state record.
Native Pause/Resume uses the actual button.
Explicit same-instance debug events deliver authored start and progress
without recreating the activity.
A synthetic event is not a real worker outcome.

## Measured padding boundary

The first native draft (`56b6170fd`,
APK `60fe847ebbd5d5c6694fbecddb08b87142b6d258137579ff0dee93901541bf29`)
added 12dp horizontal button content padding.
That value is not specified by D26 or scan-F's source;
the separate playback-mode control's 12dp floor must not be promoted into
a scan-control requirement.
The inner/light/200% Resume label failed with
`ScanIndicator.text:control:Resume:lines=1,overflow=true`.
Pause reported no overflow.
The probe stopped before cover checks and the full cohort.

Prototype `25a954117` retains the padded draft as an explicit same-APK
diagnostic control and defaults to scan-F's source padding.
For inner/light/200% Resume,
native intrinsic text width was 233.5px.
The padded draft laid out 186 by 91px and reported height overflow under
its single-line limit.
Source padding laid out 234 by 91px with neither overflow axis set.
The measured outer bar and control slots and player geometry were
identical.
This is the paired padding result,
not a universal text-fit claim.

## Native fit and state verification

The focused probe passed on both panels at 200% in light:
Pause and Resume each fired once in the same activity,
control text fit,
active geometry stayed unchanged and changed-rectangle controls failed.
The status stayed on one line;
it was ellipsized on cover and not inner.

The complete 32-pose cohort now covers both panels,
light/dark and 100%/200% text across idle,
running,
paused and wide-count scenes.
Six cover 200% active views report the accepted status ellipsis.
Fresh inspection reads `412…` in the running and paused views,
where the total is absent,
and `9,9…` in the wide-count view,
where neither count is complete.
Layout rectangles are not glyph bounds or accessibility acceptance.

Native controls passed in all eight panel/theme/scale environments:
native Pause/Resume exactly once each,
synthetic start and progress delivered without activity recreation,
paused progress not advancing,
unchanged active player geometry and app pixels,
unchanged fixed bar/control slots,
completion removing the whole bar,
and the completed player matching the idle baseline in geometry and app
pixels.
Changed-pixel and changed-geometry controls were rejected.
While active the bar reserves 137px of player height at 390dpi;
this is scan-F's own row reservation,
not a floating-overlay measurement.

## Fresh inspection and label clearance

A separate continuation session inspected all 16 light/dark full-region
pairs and four native-resolution bar strips on 2026-10-05.
Each crop was re-derived from its private raw capture,
with a changed-byte control,
before the record was bound to its hash.
No prior inspection was transferred.

A rendered-pixel measurement reads the clear background gap between the
control outline and the label ink in every active crop.
At 200% the Resume label stays 1 to 2 physical pixels clear on the leading
side and 5 on the trailing side,
on both panels and themes.
Pause at 200% keeps 29 to 32 pixels,
and both labels at 100% keep at least 57.
No glyph is cut and the native overflow flag is false in each case,
but Resume at 200% reads as touching the outline.
The measurement counts a pixel as ink when any channel differs from the
control background by more than 24 of 255;
it is not glyph metrics or accessibility acceptance.

While the bar is present,
scrollable player content is clipped 137px sooner.
The inner folder list and letter rail show one fewer row.
The track list shows one fewer row or part-row everywhere except inner at
100%,
where its visible rows still fit.

These are observations of D26's fixed 100dp control and single-line status,
recorded for the human's inspection.
They do not reopen the padding question or create a ballot.

## Verification limits

Every progress value and completion is authored debug input.
No real peak computation,
cache write,
source discovery,
media playback,
notification change or worker pause occurs.
Authored completion removes the row at the next composition;
no production dwell time or animation is selected.
No new IME,
TalkBack or keyboard-default experiment is included.

Full Android lint is not passing.
Its five inherited errors are `MissingSuperCall` in the old debug IMEs and
`NewApi` in the prior Search inset experiment.
Those files are unchanged by this work.
No finding names a `ScanIndicator` source file;
65 warnings and two hints remain.
No lint rule or unrelated source was changed.

The measured adapter was LLVM 20.1.2 llvmpipe software rendering despite
the requested host GPU path.
The retained startup output reports that the GPU cannot be used for
hardware rendering.
Successful capture and shutdown are not diagnostic-free or general
stability claims.

## Privacy and restoration

Raw screenshots,
Android hierarchies,
logs and generated identities remain private.
Sanitized publication inputs use measured application/root bounds,
exact retained RGB,
opaque 8-bit PNGs and essential chunks only.
Ordinary viewport clipping and temporary bar occlusion are retained.

Only the freshly recorded fields are restored and exactly read back:
font scale `1.0`,
night mode no,
device state identifier 2 OPENED,
accessibility disabled,
no enabled accessibility services,
2076 by 2152 at 390dpi and the system image fingerprint.
Separate base-state and override equality is not claimed.
The restored fields were read back twice:
the first readback still reported the cover geometry,
and the second matched every recorded field.
The owner exited `0`;
matching containers and QEMU are absent.
That exit code comes from the capturing session's lifecycle notification,
retained beside the private runtime evidence;
the owner's own process logs were not kept.
Runtime absence was verified again at publication.
Original AVDs and library data remain untouched.

## Publication and consumer verification

The live action matrix was recomputed offline from its 64 retained
captures,
with a per-step event ledger,
and reproduced every field of the live manifest.
Publication preflights rejected six changed inputs before any public write.

[The witness manifest](../questions/evidence/scan-indicator-witnesses.json)
and its 32 PNGs carry the inspected status reading and label clearance for
each view.
[The native verification record](../questions/evidence/scan-indicator-native-verification.json)
holds the action contexts,
their replay,
the padding control,
fixture reports,
lint totals and restoration.
The [offline viewer](../questions/scan-indicator.html) states the inspection
findings from that witness data and carries its artifact provenance.
Its builder holds 86 named rules,
one per line.
Validation re-measures every label clearance from the embedded images,
so those figures are not only asserted by the manifest.
The status readings stay read by eye;
the manifest names each image they were read from by its digest.
The consumer test rejects 106 changed inputs,
at least one per rule,
and checks HTML escaping behind the rule that normally hides it.
Each of the 86 rules and the escape was deleted once in a disposable copy;
the test failed on that rule by name every time.
Those counts are the publication's.
D88 (2026-10-05) removed the builder's digest rules,
which leaves 82 rules and 102 rejected inputs;
the test passed again and the built page did not change.
The [review verification record](../questions/evidence/scan-indicator-review-verification.json)
records that check of the viewer,
builder,
test,
manifest and native result.

An independent read-only review by a separate agent session,
given the code without this session's conclusions,
prompted that rule-per-line form.
It found that string font scales and array panels passed coerced
comparisons,
that bytes after the PNG end chunk were accepted,
that bar and player geometry was not tied to the crop,
that unquoted radio inputs,
required selects and CSS imports passed the ballot and offline patterns,
that thirteen guard deletions left the earlier test passing,
and that some page statements claimed more than the checks enforced.
All of these are closed in the published builder,
test and page.
Measured rectangles are still checked for mutual consistency and against
the crop,
not re-derived from the device.

Four offline Chromium desktop/mobile light/dark contexts exercised all
32 previews,
environment combinations,
status notes,
optional blank observations,
inert adversarial notes,
stale-reply invalidation and modal zoom/pan/reset/focus.
All distinct previews were opened in the first desktop/light pass;
representative modal controls were separately exercised in every context.
The sixteen resulting review,
gallery,
open-boundary and modal screenshots were inspected.
Each closed-page axe audit had 22 passes,
40 inapplicable checks and no violations or incomplete results.
Open-dialog axe,
Firefox ESR140 and native accessibility acceptance were not exercised.
The browser closed and its owned container is absent.
No new preference answer is required.

The [continuation plan](../../../../doc/planning/music-player-light-scan-indicator.md)
records terminal outcomes,
superseded attempts and remaining checks.

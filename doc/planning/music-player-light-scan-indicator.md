# Light scan-indicator continuation

## Purpose and settled input

Continue the authorized light-surface queue after the completed D7 menu
publication.
This item carries D26's accepted scan-F into the current Fold player and
light/dark schemes.
Settings/template editor design remains separate.
No new scan-policy ballot or production analysis is authorized.

D26 in `package/music-player/design/decisions.md` selects a bottom-edge
56dp bar with analysis text and counts,
plus an always-rendered 100dp-wide Pause/Resume control.
D13 makes the bar non-permanent:
it appears during analysis and leaves when analysis finishes.
D12 rejects a permanent completed-analysis status row.
D27's consent choices remain settled and are not part of this fit study.
D84 (2026-10-05) supersedes those consent choices:
true-peak analysis is automatic and not optional.
D26's scan bar stands,
and this fit study drew no consent prompt.

The accepted source is the right frame of
`package/music-player/design/candidates/scan-ef.dc.html`.
It uses a 56px minimum-height row in its historical dp-sized mock,
a fixed 100px control and a single-line ellipsized status label.
The control changes between Pause and Resume without changing its slot.
Its 40px minimum control height is not authority to violate the current
48dp layout floor.
The native study must measure both labels at the target text scales rather
than shrinking them to fit.

The accepted source places the scan row after the deck in normal layout.
D26 specifically prevents reflow during the scan as counts/control state
change;
it does not turn this bar into D83's floating error/Undo overlay.
The historical section 7 notes predate the selected scan-F structure.
Do not transfer the error overlay's no-reservation rule to the scan bar.
No permanent blank scan slot or completed-status row should remain.
The existing inner folder browser,
full deck and information-only crease clearance remain the player baseline.

## Initial executable-source audit

Android `MainActivity.kt:1356` starts `PeakSweepService`.
The service's `runSweep` sends count updates through `postProgress` at
`PeakSweepService.kt:91`.
`buildNotification` at line 109 renders those counts in an ongoing system
notification,
not a D26 app bar.
Its inspected implementation does not expose a Pause/Resume action.
Those are source observations,
not a reason to run or alter the real sweep.

`PlayerUiState.kt:72` exposes `loading` for source discovery.
`MainActivity.kt:3652` renders that through `loadingNotice`,
a spinner and `Loading your library…` text.
Source enumeration is not true-peak analysis;
that loading flag must not become a substitute for authored scan progress.

Scoped searches across Android production source,
desktop source and Slint UI found no `scanLabel`,
`Analysing true peak`,
`Analyzing true peak`,
`Pause analysis`,
`Resume analysis` or `isScanning` menu/bar owner.
The positive matches identify the Android sweep service,
worker and scheduler instead.
This is a scoped absence finding,
not a platform-capability claim.

The historical scan-F and first-run candidates increment their authored
count to a cap,
but their inspected timers do not transition out of scanning at that cap.
Their captions describe disappearing on completion;
the executable study must actually exercise that boundary rather than
crediting the caption as lifecycle evidence.
No historical candidate is rewritten by this continuation.

## Isolated preparation and verification

Prototype `a10baf8b4` adds validated immutable scan records and checked
synthetic events.
The 21 pure fixture tests passed,
along with fresh unknown-scene,
completed-bar,
paused-progress and stalled-running mutants.
Each mutant produced its intended assertion failure;
exact restoration and the complete unit task followed.
The authored final step actually enters a completed state and removes bar
visibility;
no worker outcome is implied.

Prototype `56b6170fd` adds the isolated native host and accepted scan-F row.
The full unit task and APK build passed.
APK SHA-256 is
`60fe847ebbd5d5c6694fbecddb08b87142b6d258137579ff0dee93901541bf29`.
The merged manifest disables production activity/services and removes
WorkManager initialization.
Android lint still fails on the same five inherited errors;
no finding names a new `ScanIndicator` source file.
No lint rule or unrelated source was changed.
No native fit or input acceptance is claimed yet.
The host uses explicit same-instance debug events for start/progress,
while Pause/Resume are native button actions.
The inspected installed SDK `Activity.onNewIntent` contract pauses/resumes
the existing top activity without recreating it when SINGLE_TOP is used.
The original scene intent is retained.
Installed Material3 `1.5.0-alpha27` source and class declarations expose the
classic outlined button,
its content padding and label style;
the study keeps 100dp width,
56dp bar height and a 48dp control floor.
Text-layout diagnostics will decide fit,
not source inspection or guessed text width.

## Independently verifiable queue

- [x] Identify D26's selected bar,
  its state transitions and the production loading/analysis boundaries.
- [x] Build pure authored scan state and an isolated native host using the
  accepted player,
  without calling production analysis or persistence.
- [x] Measure light/dark fit on both panels at 100% and 200% text,
  including Pause/Resume labels and count-width stress.
  Preserve the accepted label ellipsis and current layout floors.
- [x] Exercise native Pause/Resume,
  progress changes and disappearance after authored completion.
  Verify stable active geometry and exact return to the no-bar baseline;
  record any deliberate appearance/disappearance reservation separately.
- [x] Publish freshly inspected,
  sanitized and exact-artifact-bound evidence and verify its offline viewer.

## Boundaries and next action

Use only authored progress and debug events.
No real peak computation,
cache writes,
media playback,
source discovery,
consent changes or notification redesign occurs.
A prototype pause or completion is not a real worker outcome.
No new IME,
TalkBack or keyboard-default experiment belongs to this item.

Native work retains the owned Fold runtime's 6 GiB/2 CPU cap;
browser verification retains 2 GiB/2 CPU.
Original AVDs remain untouched.
Every visit records original settings before mutation,
restores those exact fields and verifies owner shutdown/runtime absence.
The owned visit reached the exact isolated activity with a fresh baseline.
The initial probe failed on inner/light/200% Resume text:
`ScanIndicator.text:control:Resume:lines=1,overflow=true`.
Pause reported no overflow;
the bar and control rectangles were unchanged after the native pause input.
The bar measured 137px high;
the control measured 244px wide and 131px high within it at 390dpi.
This is a real text-fit failure,
not failed touch delivery or accepted visual evidence.
The probe stopped before cover checks and before the complete cohort.
Recorded settings were restored exactly;
the owner exited `0` and its matching container/QEMU were absent.

The first native draft supplied 12dp horizontal button content padding.
That value is not specified by D26 or scan-F's source,
whose control has no horizontal padding declaration.
The separate playback-mode control's 12dp floor must not be silently
promoted into a scan-control requirement.
Prototype `25a954117` retains the rejected padding as an explicit same-APK
diagnostic control and defaults to scan-F's source padding.
It logs constrained text size,
intrinsic width and width/height overflow separately.
The complete unit task and build passed;
APK SHA-256 is
`c06ec80240e41641fee1fdec79313a8294b9595544740145f4fa6f2b1f77ddf1`.
Repeated Android lint still has the inherited errors and no new scan-file
finding.
A separate owned visit reproduced the 12dp rejection and proved the
source-padding result in the same APK.
For inner/light/200% Resume,
native intrinsic text width was 233.5px.
The padded draft laid out 186 × 91px and reported height overflow under its
single-line limit;
source padding laid out 234 × 91px with neither overflow axis set.
The measured outer bar/control slots and player geometry were identical.
This establishes the paired padding result,
not a universal text-fit claim.

The subsequent native button probe passed on both panels at 200% in light:
Pause and Resume each fired once in the same activity,
control text fit,
active geometry stayed unchanged and changed-rectangle controls failed.
The status stayed on one line;
it was ellipsized on cover and not inner.
Measured app-area crops and bar details were inspected:
Resume occupies most of its fixed slot at this scale,
and the cover's total count is not fully visible under accepted ellipsis.
No claim of fully visible counts or universal glyph/accessibility acceptance
follows from layout rectangles.

Authored completion removes the row at the next composition;
this introduces no production dwell-time or animation requirement.
The separator preserves scan-F's existing `border-top` rather than adding
a new status surface.

## Current artifact and publication state

Prototype `25a954117` is the current source,
with APK SHA-256
`c06ec80240e41641fee1fdec79313a8294b9595544740145f4fa6f2b1f77ddf1`
in the owned private study directory.
The complete 32-pose cohort covers both panels,
light/dark and 100%/200% text across idle,
running,
paused and wide-count scenes.
Six cover 200% active views report the accepted one-line status ellipsis.

The native action matrix passed in all eight panel/theme/scale environments:
actual Pause/Resume buttons fired exactly once each in the same activity,
authored start/progress events arrived without recreation,
paused progress did not advance,
active player geometry and app pixels stayed unchanged,
the fixed bar/control slot stayed unchanged,
completion removed the whole bar,
and the completed player matched the idle baseline in geometry and app
pixels.
Changed-pixel and changed-geometry controls were rejected.
The active bar reserves 137px of player height at 390dpi;
that is scan-F's own row reservation,
not a floating-overlay result.

The guest's recorded fields were restored exactly,
the owner exited `0` and the matching container/QEMU are absent.
Full Android lint still reports the same five inherited errors,
65 warnings and two hints;
no finding names a `ScanIndicator` source file.
Sanitization is complete:
32 measured-inset crops with retained-RGB proofs are under the private
study directory.

## Fresh inspection

A separate continuation session inspected the cohort on 2026-10-05.
It opened all 16 light/dark full-region pairs and four native-resolution
bar strips,
re-derived every crop from its private raw capture with a changed-byte
control,
and bound the record to the exact crop hashes.
No prior inspection was transferred.

Only authored labels and anonymous system navigation remain in the images.
Idle views show no bar,
separator or reserved slot.

On cover at 200% the status label is ellipsized in all six active views.
Running and paused show `412…`,
so the total is absent;
the wide-count view shows `9,9…`,
so neither count is complete.
Every other active view shows both counts in full.

At 200% on both panels and themes the Resume label spans nearly the whole
fixed control.
Its ink stays 1 to 2 physical pixels clear of the outline on the leading
side and 5 on the trailing side;
no glyph is cut.
Pause at 200% keeps 29 to 32 pixels clear,
and both labels at 100% keep at least 57.
The native overflow flag is false in every one of these views,
so that flag alone does not describe this visual result.

Both findings are observations of D26's fixed 100dp control and
single-line status.
They are recorded for the human's inspection,
not as a reopened padding question or a new ballot.

## Offline replay and publication

The live action matrix was recomputed offline from its 64 retained
screenshots,
hierarchies and logs,
including a per-step event ledger.
Every field of the live manifest was reproduced.
Disposable publication controls rejected a changed inspection hash,
a contradicted ellipsis reading,
changed restoration,
a changed owner exit,
missing action context and a changed replay binding before any public write.

`questions/evidence/scan-indicator-witnesses.json` and its 32 PNGs,
`questions/evidence/scan-indicator-native-verification.json` and
`questions/scan-indicator.html` are published.
The viewer states the inspection findings from the witness data.
Its build and exact validation passed.

An independent read-only review by a separate agent session then found
gaps in the first builder and test:
coerced type comparisons,
unchecked bytes after the PNG end chunk,
geometry not tied to the crop,
ballot and offline patterns that missed unquoted and non-textarea forms,
thirteen guard deletions the test did not notice,
and some page statements beyond what the checks enforced.
The published builder now holds 86 named rules,
re-measures label clearance from the embedded images at validation,
and limits the findings paragraph to validated data.
The consumer test rejects 106 changed inputs,
at least one per rule.
Each rule and the HTML escape was deleted once in a disposable copy,
and the test failed on that rule by name every time.

Four offline Chromium contexts decoded and opened all 32 images,
exercised every environment combination,
status notes,
optional observations,
inert notes,
stale-reply invalidation and modal zoom/pan/reset/focus.
All sixteen review,
gallery,
open-boundary and modal screenshots were inspected.
Closed-page axe checks report zero violations or incomplete results;
open-dialog,
Firefox ESR140 and native accessibility acceptance remain untested.
`questions/evidence/scan-indicator-review-verification.json` records that check of the
viewer,
builder,
test and evidence.
The owned browser container is absent after browser closure.

This scan-indicator item is complete within its declared design/debug scope.
The remaining light surface is the Settings pane;
its template editor remains a separate design problem under D81.
No scan-policy ballot or production implementation follows from this
completion.

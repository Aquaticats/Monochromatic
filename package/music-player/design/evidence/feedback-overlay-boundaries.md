# Transient feedback overlay verification boundary

## Purpose and accepted behavior

D83 replaces the layout-reserving error bar.
Error feedback uses floating,
automatically expiring and immediately dismissible toast/snackbar content.
Messages target two visible lines;
longer detail directs the user to capture Android logs.
The browser,
track viewport and playback deck must not resize when feedback appears,
expires or is dismissed.

D8's no-app-confirmation treatment does not bypass Android consent.
D9's vanished-row removal and count-collapsed failures remain distinct from
successful trash.
D29 supplies content-width,
left-aligned overlay placement with a 16dp owner separation.
Its former error-bar layout owner is superseded.

This is isolated design work,
not production storage or log-export implementation.
No real file is trashed,
restored,
renamed or deleted.
No playback,
library scan,
IME or TalkBack experiment is performed.

## Native source and artifact

The owned debug branch retains the accepted shared player renderer and its
E2 information-only inset route.
The isolated `LightFeedbackActivity` has no production activity lifecycle.
The merged manifest disables `MainActivity`,
`PlaybackService` and `PeakSweepService` and removes automatic WorkManager
initialization.
Production source remains separate from this debug work.

The current E2 overlay artifact is bound to prototype commit
`743c5b378c6333f75857a3d5b7c87e4c284b53c5`
and APK SHA-256
`2b556131d86e39acb8ebcf194f39da9ed8e36e2f146a86ab4fb971f4c8f06480`.
The earlier fixed-pane overlay artifact and layout-reserving bar cohorts
are not accepted evidence for this artifact.

`LightFeedbackStudy` puts snackbar hosts in a sibling overlay,
not in Scaffold's `bottomBar` or player content measurement.
The actual track viewport supplies the placement rectangle.
Simultaneous notices use measured host heights so separation remains while
an outgoing native animation still occupies space.
This source structure motivates no-layout-change verification;
it is not itself proof of unchanged rendered geometry.

## Authored outcome boundary

The fixture distinguishes:

- A known unavailable track,
  omitted from the authored list.
- A verified successful trash premise with a live restoration handle.
- A count-collapsed failure independent of a successful-trash premise.
- Failed trash,
  retaining the row without successful-trash feedback.
- A pending request,
  retaining rows and displaying no success.
- A long multiline diagnostic with non-ASCII text and a distinctive tail.

An accepted or finished Android request is not automatically per-item
verified success.
The [storage result source audit](../../../../doc/troubleshooting/android-trash-request-outcome-boundary.md)
records the deciding SDK and comparable-provider boundary.
A restore-handle premise is not proof that actual recovery succeeds.

The pure outcome report contains 16 passing cases;
the message-fit report contains nine.
Fresh third-line and overflow guard-removal controls fail the intended
message tests,
followed by exact source restoration and the complete package unit task.
These tests do not evaluate native text layout or a provider.

## Presentation and lifecycle distinction

The default renderer uses the native short-duration snackbar lifecycle.
Immediate Close and Undo controls act on the native host.
The platform host owns timeout adjustment;
there is no separately invented fixed-time accessibility policy.
Automatic disappearance must be observed without manual input or activity
recreation before being called expiry evidence.

Static acquisition uses an explicit,
false-default `hold-for-capture` flag.
A held image proves neither automatic expiry nor successful activation.
The review must label these images as held poses and keep native lifecycle
results separate.

The error message is measured at its actual constrained text width.
If it would exceed two lines or overflow,
the renderer selects concise failure copy and a named Android-log direction.
Actual `Text` layout is checked separately from premeasurement.
Full authored detail goes to the tagged Android diagnostic sink.
The diagnostic control emits only a private capture intent in this study;
it neither exports a log file nor announces capture success.

## Published evidence and verification

The final evidence set distinguishes:

- Native layout rectangles from glyph bounds and accessibility acceptance.
- Same-scene before/after geometry from comparisons that also change rows.
- Unchanged application pixels from independently animated Android
  status/navigation regions.
- Notification/shadow exclusions from ordinary player content.
- Native timeout,
  manual dismissal,
  Undo intent and diagnostic intent from held poses.
- A full multiline log read-back from a matching prefix or final marker alone.
- The exact new artifact from earlier bar or fixed-pane captures.

Changed-geometry controls reject a shifted viewport before unchanged-player
results are accepted.
The [offline viewer](../questions/feedback-overlay.html) contains 48
inspected held poses with exact panel/scene/theme/font combinations,
image digests and fresh acquisition assertions.
[Its witness manifest](../questions/evidence/feedback-overlay-witnesses.json)
records measured crop origins,
application roots,
physical dimensions,
image fingerprint and actual renderer/library environment.
These crops retain the application's RGB,
including ordinary viewport clipping and temporary notice occlusion.
They are not universal row-visibility evidence.
The viewer is evidence only,
not another policy ballot.

Build,
validation and consumer tests passed,
including a changed-inset positive fixture,
invalid crop/provenance negatives,
fresh exact-cohort/hold mutants and restored positives.
[Consumer verification](../questions/evidence/feedback-overlay-review-verification.json)
records that check.
Offline Chromium checks decoded and opened every image and exercised
availability controls,
optional empty/blank observations,
inert adversarial notes,
stale-reply invalidation and representative modal zoom/pan/reset/focus in
four viewport/theme contexts.
Closed-page axe checks reported no violations or incomplete results,
and no browser console errors were observed.
This is not Firefox ESR140,
open-dialog or native accessibility acceptance.
The fresh API37 guest completed the inner lifecycle contexts under
SwiftShader and the cover contexts under container-native LLVM20 llvmpipe.
Each group covers light/dark at 100%/200% text,
auto/manual dismissal,
Undo/log intent,
full diagnostic read-back and unchanged measured player geometry.
Those groups retain their separate renderer provenance;
exposing a render node did not make the llvmpipe run hardware-rendered.
[Native verification](../questions/evidence/feedback-overlay-native-verification.json)
retains each owning visit's actual adapter output.
The cover lifecycle group's raw records inherited a stale SwiftShader field;
the published group uses its matching owner's measured LLVM20 adapter.
The final held poses and separate Undo Close checks have a separately
verified LLVM20 owner.
All expiry logs contain exactly one activity creation and study entry;
duplicated-initialization controls are rejected.
Manual Undo Close passed every panel/theme/font context,
retaining the independent error without emitting a restoration intent.
The earlier fixed-pane diagnostic cohort showed native expiry and manual
completion with unchanged application geometry;
its corrected offline pixel comparison retained `3743106` identical
application pixels outside notices/shadows in each tested transition.
That is not final E2-artifact acceptance.
Native startup dialogs,
readiness errors and mismatched evidence scopes remain rejected attempts,
not images to relabel as successful captures.
The [continuation plan](../../../../doc/planning/music-player-light-error-undo.md)
records current terminal outcomes and the next authorized step.

## Privacy and runtime cleanup

Raw screenshots,
Android hierarchies,
logs,
ADB identities and emulator metadata remain private.
Public images must be inspected,
remove the system-status strip,
retain exact application RGB and use opaque PNGs with essential chunks only.

The human authorized fresh Pixel 9 Pro Fold AVD provisioning and alternate
system-image acquisition after the reused guest remained unreliable.
Native visits now use separately owned fresh AVDs under 6 GiB/2 CPU caps;
the retired probe and original user AVD are not reset or overwritten.
A fresh API37 guest passed the pre-app baseline and inner lifecycle checks,
but a later cover cold-start check rejected a System UI ANR.
An API36 Google APIs guest is provisioned as a separate comparison.
Image selection is not a proved root cause,
and results from different images are not interchangeable.
The final capture visit recovered one observed startup System UI dialog
before acquisition and retained the same guest across both panels.
Interrupted command attempts stayed private;
completed records were preserved and resumed with fresh hierarchy requests.
The final visit restored and exactly read back the recorded original fields,
then requested graceful shutdown.
Its owner exited `0`,
and matching QEMU and native/browser containers were verified absent.
Separate base-state/override configuration equality is not claimed unless
it was independently recorded and compared.
Browser verification runs in an owned 2 GiB/2 CPU container and closes its
browser at completion.

# First-run access and empty-library design continuation

## Scope and stop conditions

The human directed continued work on the design queue after rejecting
inconsequential filename questions.
The next bounded area is D10's unresolved no-system-library treatment,
not production implementation.
This study preserves D27's automatic use of an available system library
and its separate opt-in peak-analysis prompt.
It does not reopen the scan answers,
Search decisions,
matcher choices,
template defaults or IME behavior.

D84 (2026-10-05) supersedes that prompt and its scan answers:
true-peak analysis is automatic and not optional.
This plan records the study as it was built before D84;
its statements about a separate choice to analyse describe that study,
not current behaviour.
When analysis runs is not decided by D84.

This continuation is a design-only source audit and authored native study.
It cannot manufacture source-status signals or bind recovery actions to
production.
No original AVD,
real library or user permissions will be changed.
Disposable native work retains the prior 6GiB/2CPU bounds;
browser checks retain 2GiB/2CPU bounds.
No new IME or TalkBack experiment follows.

## Independently verifiable queue

- [x] Audit current first-run access,
  folder-grant,
  discovery and analysis boundaries against D10/D27.
  Completion requires executable-branch citations and explicit missing
  signals,
  not interpreting an empty list as confirmed absence.
- [x] Build debug-only first-run state fixtures from settled actions and
  inspect keyboard-closed fit on both panels,
  themes and native text scales.
  Completion requires tests rejecting unknown scenes and assumptions
  about failed/partial reads,
  native captures and inspected sanitized evidence.
- [x] Publish the verified design evidence with settled-action rationale
  and an implementation gate.
  Ask only for a consequential unresolved product choice;
  do not turn text or spacing details into another ballot.
  Restore and stop any owned native runtime before completion.

The comparison explores states,
not competing cosmetic policies.
A genuine product constraint discovered during the audit can change the
bounded study;
unsupported recovery controls stay out rather than pretending they work.

## Settled requirements

D10 selects a primary `Open a folder` action,
Settings access and an up-front explanation of true-peak analysis when
nothing is open.
D27 narrows this to no available system library or a declined system source.
An available system library opens automatically;
analysis does not begin merely because it was discovered.
D27's scan prompt has its recorded scan/dismiss answers,
and unanalysed music remains playable without invented peak values.
D84 (2026-10-05) supersedes the scan prompt and those answers.

D9 and D69/D71 require truthful operation/failure boundaries.
A chosen empty folder does not imply that the device has no music.
A failed or partial read does not establish that the chosen folder is empty.
Changing the source is not repair of the old source.

## Fresh source findings

This audit reads current production files without modifying them.
The source revision at the initial inspection is
`0c391a82556eeda61a285de2d70c1270fbc8994d`.
The older Search-state evidence remains bounded to its inspected revision;
this continuation does not claim the code is frozen against concurrent work.

- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt:1767`
  defines `appRoot`.
  Its final access branch renders `playerScreen` only for device-wide audio
  permission;
  the denied branch renders only `permissionGate`.
  Folder choice is therefore not exposed by that denied branch,
  although a held folder grant is a distinct source capability.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/LibrarySource.kt:179`
  defines `load`.
  A held folder wins before the device-wide permission check.
  With neither source,
  the function returns an empty track list.
  That list is not a complete source-status result.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/LibrarySource.kt:331`
  defines `scanRoot`.
  Cancellation propagates;
  another whole-walk exception is logged and converted to an empty list.
  Individual unreadable directories can also be skipped inside
  `SafTreeSource.scanDirectory`.
  Normal list return does not establish complete coverage.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/LibraryRoot.kt:260`
  defines `heldRoot`.
  A saved folder without a live persisted read grant is logged,
  cleared and returned as absent.
  This erases the distinction between never chosen and revoked at that
  return boundary.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/PlayerUiState.kt:46`
  supplies rows,
  queue size and loading state,
  but no complete/partial/failed source-discovery outcome.
- `package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt:1356`
  calls `PeakSweepService.startIfNeeded` during activity creation.
  `PeakSweepService.startIfNeeded` checks only its initial-completion flag
  before starting the foreground service;
  `runSweep` returns without analysis for an empty list and otherwise calls
  the parallel sweep.
  No D27 scan/dismiss preference is consulted along that inspected path.
  This is a source observation,
  not a live-run measurement or authority to revert concurrent production
  work.
  `package/music-player/android-app/DECISION.peak-sweep-parallelism.md:24`
  explicitly records accepted provisional automatic initial indexing and a
  planned better launch UX.
  Preserve that provisional runtime independently from D27's target;
  do not label it an unauthorized regression or revert it.

These are design/source discrepancies,
not authorization to fix production or proof of installed behavior.

## Authored native study in progress

The new branch `prototype/music-player-first-run-access` starts at
`a5560abb223af9f700b9d9465eac1991a02aac07`,
not the freshly inspected production revision.
It lives in the owned worktree
`${HOME}/temp/agent/music-player-first-run-access`.
The other prototype's concurrent lock and evidence changes remain untouched.

Exact scenes are declined device-wide access with no held source,
no source opened,
confirmed complete device-library enumeration with no eligible audio,
and confirmed complete chosen-folder enumeration with no eligible audio.
The latter scenes are authored premises,
not production status evaluation or claims that the device/folder has no files.
Partial-with-results,
partial-without-results,
failed and unread coverage must not permit those scoped empty claims.

The activity inherits `ComponentActivity`,
not `MainActivity`.
The debug manifest disables the production launcher,
playback service and initial sweep service.
`Open a folder` and in-app `Settings` callbacks emit private debug events only;
they do not open a picker,
change grants,
read music or demonstrate successful recovery.
The rebuilt merged manifest disables those production components and
omits the WorkManager initializer;
native service-isolation verification remains a capture gate.
The study includes no production scan/dismiss preference implementation.
The 12 fixture tests passed,
unknown-scene and partial-zero guard-removal mutants failed the intended
assertions,
then exact restoration,
complete tests and APK build passed.
Prototype `fca92d9d0` introduces the study;
`a534bf5ac` additionally removes automatic WorkManager initialization.

The corrected owned runtime booted within its 6GiB/2CPU cap.
A fresh snapshot records font 1.0,
light appearance,
inner panel 2 and accessibility off.
These are newly measured settings,
not a claim that an older run's snapshot applies.
The first capture attempt invoked uninstall unconditionally and received
`Failure [DELETE_FAILED_INTERNAL_ERROR]`.
Re-probes via package enumeration and package dump showed no installed
`dev.monochromatic.musicplayer` package.
The driver now checks presence before uninstalling and preserves the
retained APK bytes on a same-artifact retry.
No frame was accepted by the rejected attempt.
The underlying uninstall diagnostic is not independently diagnosed.
The next rejected attempt passed a host-only APK path to container-side
ADB and failed its local file lookup before install.
The driver now copies the retained artifact into the owned container,
checks exact bytes by digest,
then verifies the guest's installed APK digest.
Neither rejected attempt accepted a frame.
See [the separate install-boundary notes](../troubleshooting/android-37-debug-apk-signature-update.md).

D10's old `empty-a` statement that the first run analyses every file is
superseded by D27's separate choice to analyse.
D84 (2026-10-05) in turn supersedes that separate choice;
it does not decide when analysis runs,
so it does not restore the `empty-a` statement either.
No universal hour/fan prediction is copied from that historical candidate.
The in-app Settings action is not Android permission settings.
Source picker cancellation leaves the production callback unused
(`MainActivity.kt:1124`);
a non-null pick takes a persisted grant,
saves the tree and requests source reload
(`MainActivity.kt:1659`).
These source paths were read,
not exercised against real state.

The current access flag is remembered on composition entry and updated
by the permission-result callback.
No independent Settings-return access refresh was observed inside
`appRoot`;
this scoped observation does not establish whole-app behavior.

## Verified publication and next item

The native run acquired 32 initial views and eight post-swipe attempts.
Only two inner no-source 200% attempts showed changed retained RGB and
identified body-coordinate movement.
Six unchanged attempts remain private controls,
not scroll witnesses.
All 34 published app-area frames were inspected;
verified crops were generated separately from the rejected partial output.
The full final paragraph is visible in each retained scroll witness,
with the heading partly cropped.
Initial semantic clipping proves no missing glyphs,
and no visible-ink recovery claim is made.

`package/music-player/design/evidence/first-run-access-boundaries.md`
and the witness/consumer manifests retain exact provenance and limits.
The evidence-only review at
`package/music-player/design/questions/first-run-access.html`
passed build,
validation,
disposable guard-removal proofs and bounded offline consumer checks.
It has optional observations,
not another policy ballot.
The recorded fresh native settings were restored and the owned runtime
stopped;
separate base-state/override configuration was not retained or proved equal.

The human settled Android notification presentation in D82:
accept what Android provides and do no special designs.
The next substantive design item is the whole IntelliJ-aligned keyboard-map
revision from `open-questions.md` section 6,
not a notification matrix or binding-by-binding questionnaire.
Production shortcuts and live playback remain unauthorized.

## Rebuild after D84 (2026-10-05)

D84 made true-peak analysis automatic and not optional,
which the published explanation contradicted with
`After opening a library, choose whether to analyse it.`
The human authorized a rebuild and ruled that nothing is locked by hash (D88).

Prototype `61e2cf7764f627d40a77bdf6039143e229ad36ea` changes only that
paragraph,
which now reads
`True peak is measured automatically for every audio file. Analysis uses CPU and battery; playback remains available while it runs.`
The wording is study copy that no decision chose.
Its fixture tests and guard-removal mutants passed again.

The rebuilt cohort has 32 first views and no second views:
with the shorter text nothing scrolls on either panel at either text size.
Two readings agree on that,
the drag outcome and the hierarchy's scrolling mark,
and the first publication's one moved state shows that the mark can appear.
The captures span two emulator boots because the first ended in a
segmentation fault;
each view names its visit.
All views were inspected in full,
and one shows no gesture handle in the navigation strip.

The viewer,
its builder and its test were rewritten to derive the cohort from the
captures and to compare no digest.
Build,
validation,
the consumer test,
three guard-removal proofs and a four-context offline browser check passed.
`package/music-player/design/evidence/first-run-access-boundaries.md`
holds the scope and limits.
Everything this plan says about 34 published frames and two scroll
witnesses describes the first publication.

## Rebuild for D100 (2026-10-06)

D95 says a library is always open;
when no session is stored the app opens the system's default library.
D100 keeps the state where Android's music permission is refused,
shown as that library open but unreadable,
offering to grant access or to open a folder,
and drops the state with no source opened.
D10's empty state,
which D27 had narrowed to no system library or a declined one,
therefore remains only for the declined case.

### The agent's version, approved as built (D101)

The human approved building one version and asking afterwards for D97;
the same was done here,
and the human approved this version as built (D101).

- Title:
  `The device music library can't be read`,
  naming the open library as the no-audio state names it
  (`No audio found in the device music library`).
- Body:
  `It is open as your library, but access to music on this device was not granted. Allow access, or open a folder instead.`
  It names only the assessed source and does not claim the device has no music.
- Buttons,
  in this order:
  `Allow access` as the filled primary,
  `Open a folder` outlined,
  and `Settings` outlined as before.
  `Allow access` asks Android for music access;
  once Android stops showing its prompt after repeated refusals,
  the same button can only open the app's page in Android's settings.
  The study's button emits a debug event and opens neither.
- The true-peak explanation stays under the buttons,
  as D10 asks.
- The inner panel's left half stays blank,
  as in the other states.
  The open library has nothing readable to list,
  so a folder browser there would be a header and an index over no rows;
  the title names the open library instead.
  The no-audio states,
  which also have an open library,
  keep their blank half for the same reason.

The `not-opened` scene is removed from the fixture,
the capture and the review page.

## Continuation correction

Completing the filename correction should have advanced the authorized
queue,
not ended the session on a status report.
The consequence gate filters needless user questions;
it does not stop substantive design work.

The proposed tightening of `AGENTS.md`'s existing `PXQ` rule is recorded
here only.
No instruction file is edited in this round:

```text
# AGENTS.md (proposed, not applied)
PXQ:
 Finish an authorized item,
 then inspect the queue and start the next unblocked item.
 A consequence gate filters questions,
 not work.
 Stop only at queue completion or a named blocker or user choice.
```

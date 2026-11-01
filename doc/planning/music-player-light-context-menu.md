# Light track context-menu continuation

## Purpose and settled input

Continue the authorized design queue after the completed D83 overlay
publication.
The next independent light surface is D7's track context menu.
This does not reopen Search,
notifications,
supporting-text configurability or the rejected minimal-menu proposal.
Settings templates and the scan bar remain separate work.

D7 and `package/music-player/design/candidates/ctx-b.dc.html` specify a
track-name heading and grouped actions:

- Play.
- Start shuffle from here.
- File details with an inline peak value.
- Re-analyse true peak.
- Show in file manager.
- Copy filename.
- Move to trash with destructive meaning.

D7's prose and the historical candidate caption say eight items,
but both explicit action lists contain seven action rows plus the heading.
Use the enumerated actions;
do not invent an additional action to satisfy the caption.
The dark candidate's old row cues and fixed page dimensions are historical
context,
not authority to undo the accepted player or Fold geometry.

## Initial executable-source audit

The Android row in
`package/music-player/android-app/app/src/main/kotlin/dev/monochromatic/musicplayer/MainActivity.kt:3578`
uses `Text` with `Modifier.clickable`.
The inspected handler toggles the current track or calls `playIndex` for
another row.
It does not contain a long-press context-menu owner.
The `DropdownMenuItem` occurrences in that file belong to transport-button
overflow,
not D7's track actions.

The inspected selected-page desktop row in
`package/music-player/desktop-app/ui/app.slint:1961`
uses `TouchArea.clicked` to toggle or select a track.
That row is not a verified D7 context-menu implementation.
Searches across the current Android production source,
desktop source and Slint UI found no `Copy filename`,
`File details`,
`Show in file manager` or `Move to trash` labels,
or context-menu/long-press handler markers.
The broader menu search located the Android transport-overflow incumbent.
These are scoped source findings,
not a universal platform-capability claim.

The repository has a separate file-manager row-menu study in
`doc/planning/file-manager.md`,
under its row context-menu spike result.
That record is a relevant input-ownership precedent,
not music-player menu verification or authority for new keyboard defaults.
Any deciding Slint behavior must be rechecked against its cited source and
installed version before reuse.
The immediate visual study remains native Compose on the Fold target.

## Native API and fixture preparation

Installed Material3 is `1.5.0-alpha27`.
Its source archive was acquired from the URL in the installed Gradle module
metadata and matched SHA-256
`d6621ebb05d76d72ae7fd4cbd38a7766461bf658b45056fc2e55d2fcff55be8b`.
The installed AAR also matched its metadata digest.
The inspected `Menu.kt` declaration provides the existing classic
`DropdownMenu`/`DropdownMenuItem` path,
with a focusable popup,
native outside/Back dismissal and a vertically scrollable content column.
This is API/source preparation,
not native fit or input-delivery verification.
The grouped Expressive alternative is not substituted for the accepted
classic menu.

The owned prototype now shares its unchanged default track records between
panels and exposes an optional row interaction modifier.
A supplied modifier replaces the default click owner rather than stacking
long-press handling over a competing clickable owner.
The fixture records the actual source index and row data;
duplicate visible names are not identity keys.
Ordinary,
lower-anchor,
long-name and duplicate-identity inputs are authored controls,
not template presets or live library state.
The 12 pure fixture tests and fresh unknown-scene,
unknown-action and target-identity mutants passed,
with exact restoration and the complete unit task.
Prototype `647230359` also builds the isolated `TrackMenuActivity`:
row long press owns target selection,
a measured anchor supplies the native popup position,
and action clicks emit debug intents only.
The two icon tests cover every accepted vector mapping and unknown input;
a fresh fallback-icon mutant fails the intended rejection test,
followed by restoration and the complete unit task.
The native APK build passed.
The first wrapping-heading APK SHA-256 is
`f58c8967e2796076f413728e45e9b08c1e541f4b7460c369762f063b2d059689`.
Its merged manifest disables production activity/services and removes
WorkManager auto-initialization.
The next owned visit pins the exact native-library container image and
starts its ADB server inside the 6 GiB/2 CPU cgroup.
An explicit numeric-loopback client refuses to spawn a missing server;
the disposable positive/negative control and source boundary are recorded
in `doc/troubleshooting/android-emulator-console-token-container-home.md`.

The first update install rejected incompatible debug signatures.
The previously installed package was checked against the exact retained
D83 APK hash before resetting only that owned debug installation.
Original settings and old artifacts were preserved,
and the new APK then installed and passed digest read-back.

The first capture guards had two harness assumptions wrong:
this package dump reports `appId`,
and the Compose popup title is localized as `Pop-Up Window`,
not the guessed `PopupWindow:` prefix.
The retained windows already showed the intended activity and owned popup.
The corrected guard checks user-0 UID/package,
`APPLICATION_SUB_PANEL` kind and the `TrackMenuActivity` parent;
retained positive and wrong-owner/window-kind controls pass.
Those initial rejections are not native menu activation failures.
The corrected inner/light/100% probe passed:
actual long press opened the intended row's native menu,
no underlying row tap fired,
and Android Back dismissed it while retaining the original root/viewport
geometry.
Its measured-inset private crop was visually inspected;
the complete menu and its inline peak value are visible in that context.
Large-text long-heading probes then passed on both panels.
The cover initially exposed every full-size action;
the inner needed one observed scroll to expose Move to trash.
Those captures also exposed an implementation deviation:
the accepted `ctx-b.dc.html:19` heading explicitly uses one-line ellipsis,
but the native prototype omitted the line cap and rendered eight heading
lines in the inner stress pose.
The source property had been overlooked,
not left as a new user preference.
Prototype `5af5e3b10` restores `maxLines = 1` and explicit ellipsis while
retaining full authored identity in the model and diagnostics.
The wrapping-heading APK and captures are preserved as superseded probes;
they will not be relabelled as the corrected artifact.
The corrected build and full unit task passed,
and the installed APK matched SHA-256
`a23f02c51a02b10ee1eb2b8ed21d8ab345fed4d9deb4fdc1a8d38c7e0e54d574`.
New large-text records confirm one heading line with intentional overflow;
the retained eight-line record fails the current heading assertion.
This is an actual prior-artifact negative,
not a new source-mutation claim.
The corrected study uses a separate private evidence directory.
That artifact completed all 24 native menu poses with every action
initially visible,
including lower-row and long-title contexts.
The native action matrix also passed:
each accepted intent at every panel/theme/font context,
plain tap without menu opening,
outside dismissal without a leaked row tap,
unchanged app pixels/geometry and distinct duplicate-name targets.
The guest was exactly restored and stopped;
all sanitized pairs were visually inspected.

Full Android lint then reported five inherited errors:
`MissingSuperCall` in the three old debug IMEs and `NewApi` in the prior
Search inset experiment.
It also identified `ModifierParameter` warnings introduced by the new
nullable row modifier.
Those conventions require an outermost `modifier`,
first among optional parameters and defaulting to `Modifier`.
Prototype `d1d19dfed` moves the sole input owner to that standard boundary;
unrelated IME/Search code and lint rules are not changed.
The updated source passed the complete unit task and APK build;
its APK SHA-256 is
`1e2092fe9e11e0aaf6b56b125db77ba19798f5d7c0706ceccbe6c0323551870a`.
A repeated Android lint run reports the same five inherited errors and
65 warnings rather than 68;
the new row's three `ModifierParameter` findings are gone.
This is not a full-package lint pass.
A separate owned visit and evidence directory verified the new artifact:
all 24 poses and all seven action intents in every panel/theme/scale context
passed,
along with tap/long-press separation,
outside dismissal,
unchanged player geometry/app pixels and duplicate-name targeting.
The recorded settings were restored exactly;
the owner exited `0` and its container/QEMU were absent.
The inspected `a23f02c5` captures retain their original APK/source identity.

Byte comparison rejected inspection transfer for all 24 new crops.
Their measured roots and menu bounds match the prior records,
but RGB channels differ by up to 3 on the 8-bit scale.
This comparison does not establish a cause or visual equivalence.
Every current full-region pair was opened and inspected independently,
then bound to the current APK/source and crop hashes.
The rejected transfer record is retained as resolved by fresh inspection,
not erased or converted into an equality claim.
Current PNGs,
scoped native verification and `questions/track-menu.html` are published.
The viewer build and exact validation passed.
Consumer positives,
fresh exact-cohort/native-input guard removals and restored positives passed.
Four offline Chromium contexts decoded and opened all 24 images,
exercised every environment combination,
optional observations,
inert notes,
stale-reply invalidation and modal zoom/pan/reset/focus.
All eight browser review/modal screenshots were inspected.
Closed-page axe checks report zero violations or incomplete results;
open-dialog,
Firefox ESR140 and native accessibility acceptance remain untested.
`track-menu-review-verification.json` records that check.
The owned browser container is absent after browser closure.
Disposable publication controls rejected a changed inspection hash,
changed restoration and missing action context before any public write.

## Independently verifiable queue

- [x] Identify D7's explicit actions and distinguish historical design from
  executable row/menu ownership.
- [x] Inspect current native menu declarations and accepted player hooks,
  then build an isolated debug study without production callbacks.
- [x] Verify target identity,
  opening/dismissal,
  action-intent ownership and fit across both panels,
  themes and 100%/200% text.
  If content exceeds a viewport,
  prove access to it rather than shrinking layout floors.
- [x] Publish only inspected,
  sanitized and exact-artifact-bound evidence with explicit action limits.

## Boundaries

No real playback,
peak analysis,
clipboard write,
file-manager launch,
trash or restore is authorized by this design study.
Action selection records an authored target and debug intent only.
D8's consent and truthful success boundaries remain intact.
No new IME,
TalkBack or keyboard-default experiment is part of this item.
Original AVDs and completed D83 artifacts remain untouched.
Native work retains 6 GiB/2 CPU bounds;
browser verification retains 2 GiB/2 CPU bounds.
The successful D83 runtime bridge is reusable infrastructure,
not a claim that cold-start system failures are fixed.

No new preference question is established by this initial audit.
This menu item is complete within its declared design/debug scope.
Continue the independent light-surface queue with the scan indicator;
Settings/template editor design remains separate.
No colour ballot or production implementation follows from this completion.

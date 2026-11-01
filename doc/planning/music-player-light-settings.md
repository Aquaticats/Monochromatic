# Light Settings-pane continuation

## Purpose and settled input

Continue the authorized light-surface queue after the completed D26
scan-indicator publication.
This item carries D11's accepted Settings pane into the current Fold player
and its light/dark schemes.
D81's template editor remains a separate design problem.
No Settings-template ballot or production implementation is authorized.

D11 in `package/music-player/design/decisions.md` selects candidate
settings-a:
three flat switch rows and a closing sentence that says the pane is short.
D12 removed settings-b's analysis-status row;
Re-analyse lives on the track menu.
D21 added a fourth row for the command bar's global hotkey,
and D47 superseded that command bar together with its row.
Do not transfer the fourth row to this pane.
D84 (2026-10-05) removes the third row:
true-peak analysis is automatic and not optional,
so Settings provides no analysis switch.
D85 (2026-10-05) removes the closing sentence.
The pane then held two rows.

D86 (2026-10-05) made both remaining rows always-on behaviour and not settings,
and D87 keeps the Settings page and leaves it empty,
with no study of the empty page.
The two-row study this plan produced is therefore withdrawn and kept as a record.
Everything this plan says about the two-row pane is history,
and this item is closed.

The accepted source is
`package/music-player/design/candidates/settings-a.dc.html`.
It is an 860 by 600px dark mock shaped like a desktop window.
Its top row is 56px high and holds an outlined `Back to Camellia` control and
a `Settings` label.
Each setting row has a 64px minimum height,
a 15px title,
a 13px supporting line and a 52 by 32px switch.
A 13px closing paragraph follows the rows.
The mock's 40px Back control is not authority to violate the current 48dp
layout floor,
and its window proportions are not Fold geometry under D49.

D11 records these rows and mock states:

- `Strip common prefixes from filenames`,
  on by default.
- `Resume where I left off`,
  on,
  restoring folder,
  track and position while paused.
- `Analyse true peak in the background`,
  off in the mock;
  off means each track is measured just before it plays.
  D84 removes this row.

## Initial executable-source audit

Android `MainActivity.kt:2635` declares `settingsPage`.
It renders a `Page controls` heading,
one radio option per page-control style and a `Back to library` button.
`showingSettings` at line 2140 opens it,
`BackHandler` at line 2148 returns on system Back,
and line 2220 swaps it with the track pager while the surrounding chrome
stays.
Desktop `app.slint:1786` renders the same page-control chooser.

A scoped search of Android production source,
desktop source and Slint UI found none of D11's three row labels.
That is a scoped absence finding,
not a platform-capability claim.
Production Settings is therefore a page-control experiment,
not D11's pane.
It contributes one structural precedent only:
Settings replaces the track region,
other chrome remains,
and system Back returns to the library.

## Constraints from later decisions

D49 makes the Fold's cover and inner panels the visual source for every
platform,
so the desktop mock's width does not govern this study.
D50 requires the playback/control deck to stay visible while the Fold is
unfolded.
D51 keeps the Search page and its header on the right of the unfolded panel,
leaves the same folder browser and the deck on the left,
and uses one full-width destination with the same header on the cover.
E2 keeps information off the fold connector,
approximately x `[983,1093)`px of the 2076px inner display.
D34,
D35 and D45 settle the light roles;
true black remains the dark background under standing standard 7.
The whole-map keyboard proposal lists `Ctrl ,` for Settings,
but that proposal is not adopted.

`package/music-player/design/md3-tokens.md` held switch values with a note
to re-check them if switches become prominent.
This pane makes them prominent,
so the re-check was done on 2026-10-05 against the current token files.
The 52 by 32px track and 24px selected handle of settings-a match.
Its unselected switch does not:
the mock draws a 24px handle with no track outline,
while the token file gives a 16px handle and a 2px outline.
The switch's 40px state layer is below the 48dp layout floor,
so the row or switch target must supply that floor.
Draw the study with the platform's Material 3 switch,
not a copy of the mock's off state.
The same re-check read 10px for the list item's top and bottom space,
where `md3-tokens.md` had recorded 12px;
that file now holds both readings and changes no drawn surface.

## Interactions the audit cannot settle

### Analysis preference model

D11's third row is a two-state switch.
D27's first-run prompt has four answers,
and two of them persist:
`Always scan` and `Dismiss forever`.
D27 says Re-analyse on the track menu is the only entry point after
`Dismiss forever`.
Neither decision says whether the Settings switch reflects or reverses a D27
answer.
The fit study can draw the row in both positions without settling this.
Raise it with the human only if a built state shows a concrete conflict.

The human resolved this on 2026-10-05 by withdrawing the row (D84):
analysis is automatic and not optional,
which also supersedes D27's ask-before-analysing prompt and its four answers.
When analysis runs is not decided by D84.

### Title prefixes, templates and Search

The first row changes displayed titles.
D81 makes supporting text templated and states that title customization is
not implied.
The filename investigation recorded that D11's example does not settle
Search's extension display.
Draw the row as accepted;
do not extend it to Search or to templates.

### Template entry and the closing sentence

D81 puts template configuration in Settings,
while section 11e of `open-questions.md` leaves its fields,
grammar and editor controls undesigned.
The closing sentence was accepted before D81 and says the pane holds
everything.
Do not invent a template entry.
Record that the closing sentence depends on the 11e design round.

D85 (2026-10-05) removes the closing sentence,
so nothing in this pane now depends on the 11e round;
the template entry itself stays undesigned.

### Placement on the Fold

settings-a is a whole-window desktop page.
No accepted Fold placement exists.
D50,
D51 and the production precedent point one way:
on the inner panel,
Settings on the right under a Back and title header,
with the same folder browser and the deck retained on the left;
on the cover,
one full-width Settings destination with the same header.
That is the proposed placement to build,
not an accepted decision.
Under standing standards 2 and 3 it is a designer call made with built
evidence and a recommendation,
not a verbal question.

## Independently verifiable queue

- [x] Identify D11's selected pane,
  the later decisions that changed its content,
  and the production Settings boundary.
- [x] Re-check the MD3 switch and list-row values against the current
  specification source and record them in `md3-tokens.md`.
- [x] Build pure authored Settings state with checked toggle events,
  without persistence or production wiring.
- [x] Build an isolated native host using the accepted player and the
  proposed Fold placement.
- [x] Measure light/dark fit on both panels at 100% and 200% text,
  including the long supporting lines and both switch positions.
  Preserve the 48dp layout floor.
- [x] Exercise native switch toggles,
  the Back control and system Back.
  Verify the return to an unchanged player.
- [x] Publish freshly inspected,
  sanitized and exact-artifact-bound evidence and verify its offline viewer.

## Isolated preparation and verification

Prototype `af793e436` adds an immutable three-switch record,
D11's row order and copy,
and one checked toggle event per row.
Scene `accepted` is D11's mock state;
scene `inverse` puts every switch in the other position.
The complete unit task passed with 12 Settings fixture tests.
Fresh unknown-scene,
copy-drift,
row-position-swap,
unknown-event and cross-row-toggle mutants each failed their intended test.
Exact restoration and a freshly executed complete unit task followed.

Prototype `1cf2f2cb3` adds the isolated native host.
Each row is the platform's Material 3 list item with a display-only switch;
the whole row is the two-state control,
so the row supplies the 48dp floor rather than the switch's 40dp state layer.
Both text lines may wrap.
The page header follows the accepted Search header:
a 48dp Back target and a one-line title in a 72dp bar.
The unfolded arrangement reuses the Search page's deck host and folder
browser on the left;
the cover page is full width.
Prototype `6bc622ff7` then makes each measured element report its unclipped
rectangle beside its visible one,
so a row scrolled partly out of view is not judged shorter than it is.
The complete unit task and APK build passed for that commit.
The current APK SHA-256 is
`40d0b0e4ab592e920372be8d4771fb9381e4e885501b54f575ac3db567a11f0d`;
the first host build,
`2faeddc682baf92aaf3d0bfcb4f5c53d4c1d29dc6d2803b2fe46e40513305ee1`,
was never installed.
The merged manifest disables the production activity and services,
removes WorkManager initialization and exports `SettingsPaneActivity`.
The player's own Settings button is not wired;
the study opens the page from an authored scene or an explicit debug event.

## First native visit and what inspection rejected

The first owned visit installed APK `40d0b0e4…` and captured both panels in
light and dark at 100% and 200% text.
Every measured rule passed:
72dp header,
48dp Back target,
52 by 32dp switches,
rows above the 48dp floor,
no text overflow and no information inside the fold connector.
At 200% text the column scrolls on both panels,
so each open state also has an end-of-column capture.

Fresh inspection of those captures then rejected the build for publication.
In the dark theme the header's Back glyph and title,
and the retained deck's title and transport buttons,
were drawn black on the dark page.
Nothing above them supplied a content colour;
the accepted Search page is hosted inside a filled region that does.
Each separator was also one pixel different across the fold connector,
because the opaque row fill began after the connector inset.
Measured rectangles could not show either defect.

Prototype `483f16cdd` puts the opened study in the same kind of filled region,
names the header's colour and makes the row fill transparent.
Its complete unit task and APK build passed.
The current APK SHA-256 is
`85e4a2080d1d737eb01a16bdcc5172bcc7103fe014770891d68d3cdf1854ca50`.
The first visit's captures are superseded and stay private;
they are not publication inputs.

## Visits on the fixed build

A second visit installed the fixed APK,
then failed before its first capture:
a 15 s bound on `podman inspect` expired under a host load near 50.
It restored every recorded field,
its owner exited `0`,
and it supplies no evidence.
That bound is now 120 s,
and a failed or timed-out inspection reads as unknown,
never as an absent runtime.

The third visit captured all 32 views:
both panels,
light and dark,
100% and 200% text,
the closed player,
both switch scenes and an end-of-column view wherever the column scrolls.
Every measured rule passed,
and the geometry matches the first build exactly.
The column scrolls only at 200% text:
787 physical pixels on the unfolded panel and 318 on the cover.

Fresh inspection of all 16 light/dark pairs found the dark header and deck
text legible and the separators uniform across the connector.
It also records four observations:
at 200% text on the unfolded panel `Resume where I left off` leaves `off`
alone on its second line;
at the start of a scrolling column the last visible line runs under the
gesture handle;
opening Settings narrows the left folder browser exactly as the Search page
does;
and the 24dp Back arrow reads small beside the 200% title.

The viewer builder now re-measures three things from the embedded images:
header text contrast,
drawn switch positions,
and the retained left half against the published Search images.
Run on the first build's crops,
those rules reject it by name:
the left half first,
then the dark title and Back arrow once that rule is removed.

## Boundaries

Use only authored switch state.
No preference is persisted,
no prefix stripping or session restore is implemented,
and no analysis runs.
No new IME,
TalkBack or keyboard-default experiment belongs to this item.

Native work retains the owned Fold runtime's 6 GiB/2 CPU cap;
browser verification retains 2 GiB/2 CPU.
Original AVDs remain untouched.
Every visit records original settings before mutation,
restores those exact fields and verifies owner shutdown and runtime absence.

## Inputs, publication and completion

Row toggles,
the Back target,
one authored reopen and system Back passed in all eight environments.
Each toggle changed pixels only inside its own row and toggling back
restored the app pixels exactly;
both Back paths returned to a player identical in geometry and app pixels
to a freshly launched one,
and the host was created once.
The 90 retained input captures were replayed offline and reproduced every
field of the live manifest.
Recorded guest fields were restored,
the owner exited `0` and no matching runtime remains.

`questions/evidence/settings-pane-witnesses.json`,
its 32 PNGs,
`questions/evidence/settings-pane-native-verification.json` and
`questions/settings-pane.html` are published.
Seven changed publisher inputs were rejected before any public write.
The viewer's 105 rules each have at least one rejected consumer input among
142 and a deletion proof;
its pixel rules decode PNGs in process,
matching ImageMagick byte for byte on all 36 images they read.
Four offline Chromium contexts passed with no axe violations,
and all sixteen screenshots were inspected.
`questions/evidence/settings-pane-review-verification.json` records that check of the
viewer,
builder,
test and evidence;
`evidence/settings-pane-boundaries.md` holds the scope.

That three-row publication completed the item within its declared
design/debug scope,
then D84 and D85 withdrew its third row and closing sentence.

## Two-row rebuild after D84 and D85

Prototype `17c739da6` removes the analysis field,
row and toggle and the closing paragraph;
the former analysis event now falls through to the unknown-event rejection.
Prototype `67eae28d1` adds a mixed switch record to `rowPositionsFollowState`,
because both authored scenes set the two remaining fields equal
and only a mixed record tells the rows' fields apart.
The complete unit task passed with 12 Settings fixture tests.
Fresh unknown-scene,
copy-drift,
row-position-swap,
unknown-event,
cross-row-toggle and reinstated-analysis-toggle mutants each failed their
intended test,
and the restored complete unit task passed.
The APK SHA-256 is
`5f3a23911f2a5c32859a78e533704298bfb5afa042cf968f4fc82691ef35e9ab`.
Android lint keeps the five inherited errors and reports no Settings finding.

A fourth visit,
the first on this build,
failed before its first capture.
The guest raised a startup `Application Not Responding` dialog,
as it did on the third visit,
and killed the hierarchy dump the bootstrap uses to find its close action
(remote exit status 137).
It restored every recorded field,
its owner exited `0`,
no matching runtime remained,
and it supplies no evidence.
The shared hierarchy helper is unchanged.

A fifth visit captured all 12 inner views,
then stopped at the cover's first 200% capture:
the host was created twice in one process,
4.5 s apart,
because a configuration change from the environment step reached the app
after its launch.
The one-creation guard refused that capture.
Its restoration then failed on a 15 s `podman inspect` bound in
`restore-stop-fresh-fold.ts`,
the same bound that ended the second visit;
that bound is now 120 s,
and the restoration was rerun by hand.
It restored every recorded field,
the owner exited `0` and no matching runtime remained.
Every launch is now watched for 8 s and relaunched,
at most twice,
if it is created again;
each capture records the attempts its launch used.
A sixth visit repeats the run.

The fifth visit's inner cohort shows two rows fit without scrolling at both
text sizes:
the last row ends at 748px at 100% and 1483px at 200%,
above a root bottom of 2074px,
and the divider between rows measures 2px.

The sixth visit captured all 24 views:
no column scrolls on either panel at either text size,
and every launch needed one attempt.
Fresh inspection of its 12 light/dark pairs found the rows,
switch positions,
dark colours and separators as intended,
with nothing drawn below the last separator.
The updated viewer built,
validated and passed its consumer test on those crops in a disposable copy.
The emulator then crashed during the input stage
(segmentation fault in the gfxstream GL translator,
exit status 139),
before restoration,
so the guest kept the visit's font scale and night mode.
The crash left lock files in the owned AVD copy,
and the first recovery boot was refused as a second instance.
After `podman ps` and `pgrep` showed no container or emulator,
those two lock files were deleted
and a recovery boot restored the sixth visit's own recorded baseline.
Deleting them,
with only those two owner checks,
departed from the verified workaround in
`doc/troubleshooting/android-emulator-37-disposable-avd-lock-after-hard-stop.md`,
which moves the files to a backup after wider owner checks;
that document now records this recurrence and the departure.
Why the lock blocked the boot is not established.
The crashed owner's records are kept beside the visit.
A seventh visit repeats the whole run,
so one visit supplies every published capture and input check.

## Two-row publication and completion

The seventh visit completed every stage.
Its 24 views have the same geometry as the sixth visit's,
every launch needed one attempt and nothing scrolls.
Row toggles,
the Back target,
one authored reopen and system Back passed in all eight environments,
and the 72 retained input captures were replayed offline and reproduced
every field of the live manifest.
Recorded guest fields were restored on the first readback,
the owner exited `0` and no matching runtime remains.

Fresh inspection of its 12 light/dark pairs matched the sixth visit's.
It also found one capture artifact:
the first screenshot of the visit,
the inner closed player in light at 100% text,
shows no gesture handle in the system's navigation strip,
alone among the visit's 96 screenshots.
The same state captured later shows the handle.
The cause was not established;
the strip is below the measured application root,
no rule or comparison reads it,
and the viewer and boundary document state it.

The publisher removed the withdrawn three-row files only after each matched
the hash its own witness file recorded,
then wrote the 24 PNGs,
`questions/evidence/settings-pane-witnesses.json` and
`questions/evidence/settings-pane-native-verification.json` in one commit.
Eight changed publisher inputs were rejected before any public write.
At publication the viewer's 99 rules each had at least one rejected consumer input among
135 and a deletion proof;
D88 later removed its digest rules,
which leaves 94 rules and 130 rejected inputs;
its in-process PNG decoder matched ImageMagick byte for byte on all 28 images
it reads.
Four offline Chromium contexts passed with no axe violations,
and all sixteen screenshots were inspected.
`questions/evidence/settings-pane-review-verification.json` records that check of the
viewer,
builder,
test and evidence;
`evidence/settings-pane-boundaries.md` holds the scope.

This Settings item is complete within its declared design/debug scope.
One follow-up is recorded and not done:
the published first-run study still says
`After opening a library, choose whether to analyse it.`,
which D84 contradicts.
It was left alone at first on the ground that its review verification bound the page by hash.
D88 (2026-10-05) rules that out:
nothing is locked by hash.
The rebuild is tracked in `package/music-player/design/HANDOFF.md`.

The viewer builder now expects two rows,
reads the column end from the last row and the measured divider between the
rows,
and re-measures from the images that nothing is drawn between the last row
and the application root bottom.

- [x] Remove the analysis row and the closing sentence from the prototype,
  with fixture tests and mutants.
- [x] Build the two-row APK and lint it.
- [x] Capture both panels in light and dark at 100% and 200% text,
  and exercise every input again.
- [x] Inspect every light/dark pair afresh and record the findings.
- [x] Replace the withdrawn three-row evidence with the inspected two-row
  cohort and rebuild and verify the viewer.

D81's template editor (`open-questions.md` 11e) is the next design item and
needs the human's direction.

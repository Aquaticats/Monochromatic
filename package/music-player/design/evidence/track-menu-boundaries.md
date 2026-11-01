# Native track context-menu verification boundary

## Purpose and accepted menu

This design-only study carries D7's accepted track menu into the light
surface and current Fold player geometry.
It does not reopen Search,
notification presentation,
keyboard defaults or D81's Settings-template requirement.

The explicit D7 list and accepted `candidates/ctx-b.dc.html` contain seven
action rows in three groups plus a track-name heading.
Their old caption says eight items;
no extra action is invented to reconcile that wording.
The accepted actions are Play,
Start shuffle from here,
File details with the peak value inline,
Re-analyse true peak,
Show in file manager,
Copy filename and Move to trash.
Destructive meaning uses wording,
an icon and the error role,
not color alone.

The accepted heading uses one-line ellipsis.
The native implementation preserves that cap rather than claiming every
long title is fully visible.
Full authored title text remains in the fixture and private diagnostics.
Action labels can wrap while retaining their 48dp layout floor.

## Isolated artifact and input owner

The current source is prototype commit
`d1d19dfed8b47cbe6d589fc0b47176e022242aad`,
with APK SHA-256
`1e2092fe9e11e0aaf6b56b125db77ba19798f5d7c0706ceccbe6c0323551870a`.
`TrackMenuActivity` extends `ComponentActivity`,
not the production activity.
Its merged manifest disables production activity/services and removes
WorkManager auto-initialization.

The shared row accepts its standard outermost modifier.
Its caller supplies one click/long-press owner;
a second click modifier is not stacked over it.
The actual authored source index and row data determine the selected target.
An index here is a fixture identity,
not a promised durable production-library identifier.

The native classic `DropdownMenu` uses a measured row anchor.
Screen-coordinate conversion distinguishes the popup window from the
activity's local root.
Focusable native popup behavior owns outside and Android Back dismissal.
The popup may temporarily obscure list or deck content;
it does not allocate player layout space.
After touch dismissal,
unchanged-player checks compare the same authored state.

## Authored operation limits

Selecting an action emits a checked debug intent only.
No audio playback,
analysis,
clipboard write,
file-manager launch,
trash or restore occurs.
A callback is not operation success,
and D8's consent and verified-outcome requirements remain unchanged.
The synthetic peak value is not an analysis result.

Duplicate-name probes distinguish source indices using the selected intent
and each row's authored peak.
A repeated title string alone is not evidence of target identity.
No native accessibility,
IME or new keyboard-default experiment is included.

## Tests and current verification status

The pure fixture suite contains 12 passing cases;
the icon suite contains two.
Fresh unknown-scene,
unknown-action,
target-identity and icon-fallback mutations failed their intended tests,
followed by exact restoration and the complete unit task.
The fixture source is unchanged from its mutation-proven version.
The current APK build passed.

Full Android lint is not passing:
its five inherited errors are `MissingSuperCall` in the old debug IMEs and
`NewApi` in the prior Search inset experiment.
Those files are unchanged by this work.
The new row's `ModifierParameter` warnings were corrected through the
standard modifier boundary;
no lint rule or unrelated IME implementation was changed.

The earlier `a23f02c5` artifact completed 24 keyboard-closed poses across
both panels,
themes and font scales,
plus all seven action intents in every environment,
plain-tap/long-press separation,
outside dismissal,
unchanged app pixels/geometry and duplicate-name target checks.
Those records retain their own source and APK identity.
They are not relabelled as the current modifier-cleanup artifact.
The first `f58c8967` wrapping-heading artifact is also retained as a
superseded probe,
not current visual evidence.

The current artifact completed the same 24-pose cohort and full action
matrix in its own bounded visit.
All seven actions were initially visible in every captured menu.
Plain tap emitted one row event without a menu;
long press opened the target menu without a row tap.
Outside dismissal preserved player geometry and app pixels without leaking
a tap to the underlying row.
Changed-pixel controls rejected differences;
an offline replay also rejected changed root and viewport coordinates.
Duplicate-name coverage is copy-name intent on source indices 1 and 2,
not every action on both duplicates.

Byte equality to the prior inspected artifact failed for all 24 captures.
Measured roots and menu bounds match,
while RGB channels differ by at most 3 on the 8-bit scale.
No cause or cross-artifact visual equivalence is inferred.
All current full-region pairs were inspected afresh and named by digest in the manifest before
publishing `questions/evidence/track-menu-witnesses.json` and its PNGs.
`questions/evidence/track-menu-native-verification.json` records exact
contexts,
operation limits and geometry replay.
The verified consumer requires exact cohort membership,
measured crop bounds,
actual runtime provenance,
heading behavior and native-input evidence.
The [offline viewer](../questions/track-menu.html) and its build/validation,
consumer tests and exact-cohort/native-input guard-removal proofs passed.
The [verification record](../questions/evidence/track-menu-review-verification.json)
records that check of the viewer,
manifest and native result.
Four offline Chromium desktop/mobile light/dark contexts exercised all
24 previews,
environment combinations,
optional blank observations,
inert adversarial notes,
stale-reply invalidation and modal zoom/pan/reset/focus.
All distinct previews were opened in the first desktop/light pass;
representative modal controls were separately exercised in every context.
The eight resulting review/modal screenshots were inspected.
Each closed-page axe audit had 22 passes,
40 inapplicable checks and no violations or incomplete results.
Open-dialog axe,
Firefox ESR140 and native accessibility acceptance were not exercised.
The browser closed and its owned container is absent.
No new preference answer is required.

## Privacy and restoration

Raw images,
Android hierarchies,
logs,
ADB identities and emulator metadata remain private.
Public images require full-region visual inspection,
measured status-strip removal,
unchanged retained RGB and opaque PNGs with essential chunks only.
Ordinary viewport clipping and temporary popup occlusion are retained,
not erased.

Native visits use an owned Pixel 9 Pro Fold profile under 6 GiB/2 CPU caps.
The native-library container is pinned by image ID;
its actual renderer is recorded from the owning visit,
not inferred from a requested GPU flag.
Only the recorded original settings are restored and exactly read back.
Separate base-state/override equality is not claimed.
The current native owner exited `0` after exact recorded-field restoration;
its matching container and QEMU are absent.
The measured adapter was LLVM 20.1.2 llvmpipe software rendering despite
the requested host GPU path.
The owner log still contains GPU fallback,
`bad color buffer handle` and `stop: Not implemented` diagnostics.
Successful capture and shutdown are not diagnostic-free or general
stability claims.
Original user AVDs and library data remain untouched.
The [continuation plan](../../../../doc/planning/music-player-light-context-menu.md)
records terminal outcomes,
superseded attempts and remaining checks.

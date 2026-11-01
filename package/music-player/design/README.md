# Music player design

Design source and decision record for the shared Android and desktop music player interface.
The work targets the Pixel 9 Pro Fold first while preserving one Material Design 3 identity across
Jetpack Compose and Slint.

This package contains exploratory Design Components,
rendered comparisons,
device metrics,
settled decisions,
and the active handover.
It does not describe shipped application behaviour unless a decision explicitly says it has been implemented.

## Start here

Read these files in order:

1. [`HANDOFF.md`](HANDOFF.md)
2. [`review-notes.md`](review-notes.md)
3. [`decisions.md`](decisions.md)
4. [`open-questions.md`](open-questions.md)
5. [`md3-tokens.md`](md3-tokens.md)
6. [`material-3-compliance.md`](material-3-compliance.md)
7. [`device-metrics.md`](device-metrics.md)
8. [`candidates.md`](candidates.md)

## Active design continuation

The D46 folded-cover picker revisit is in
[`questions/cover-picker-revisit.html`](questions/cover-picker-revisit.html):
P4 beside the agent's P5, eight inspected native views,
not a ballot.

The first-run/no-library study is in
[`evidence/first-run-access-boundaries.md`](evidence/first-run-access-boundaries.md).
Its [verified offline review](questions/first-run-access.html) presents
24 first views and 4 end views,
not a policy ballot.
The study was rebuilt on 2026-10-05 after D84 made true-peak analysis
automatic,
and again on 2026-10-06 for D100:
a library is always open,
so the declined state shows the device music library as open but unreadable,
with `Allow access` and `Open a folder`,
and the state with no source opened is gone.
That state's wording and layout were approved as built (D101).
The source states are authored inputs,
not production permission/discovery evaluation or recovery acceptance.
The owned native runtime was stopped after restoring its recorded fields.

D82 accepts Android's standard media-notification presentation;
no special notification design remains.
The [whole-map keyboard design](questions/keyboard-map.prototype.html)
was browser-verified and accepted as built (D102),
with editing and focused-control ownership preserved.
[Its boundary record](evidence/keyboard-map-boundaries.md) distinguishes
proposal,
protocol and actual OS/native delivery.
The completed light error/Undo study follows D83:
auto-dismissing,
manually dismissible overlays that never resize the player,
with two-line messages and Android-log direction for longer detail.
The former layout-reserving bar studies are superseded;
D8/D9 outcome requirements and D29's non-reserving placement context remain.
[The overlay evidence boundary](evidence/feedback-overlay-boundaries.md)
separates held poses,
native lifecycle controls and actual storage/log-export implementation.
The [verified offline viewer](questions/feedback-overlay.html) presents
48 inspected native held poses across both panels,
themes and text scales.
Native expiry,
error dismissal,
Undo intent and Undo Close are separately verified;
held images are not timing evidence.
The guest's recorded settings were restored and its owner exited cleanly.
The consumer passed scoped tests and four-context Chromium checks,
not Firefox ESR140 or native accessibility acceptance.
D7's light track-menu continuation is also complete.
The [menu viewer](questions/track-menu.html) presents 24 freshly inspected
native poses with the accepted actions and one-line ellipsized heading.
[Its evidence boundary](evidence/track-menu-boundaries.md) records all seven
debug intents across both panels,
themes and text scales,
input ownership,
dismissal and unchanged-player checks.
The viewer's tests,
guard-removal proofs and four-context offline Chromium checks passed.
Recorded native settings were restored and owned runtimes are absent.
Real operations,
native accessibility and Firefox acceptance are not claimed;
full Android lint still has inherited failures.
D26's light scan-indicator continuation is also complete.
The [scan viewer](questions/scan-indicator.html) presents 32 freshly
inspected native views of the accepted scan-F bar across both panels,
themes and text scales.
[Its evidence boundary](evidence/scan-indicator-boundaries.md) records the
rejected 12dp padding draft,
native Pause/Resume,
authored progress and completion,
and an offline replay of that action matrix.
Inspection found the cover status ellipsized to `412…` at 200% text and
the Resume label within 1 to 2 physical pixels of its outline at 200%;
the viewer states both rather than deciding them.
The viewer's tests,
guard-removal proofs and four-context offline Chromium checks passed.
Recorded native settings were restored and owned runtimes are absent.
Real analysis,
native accessibility and Firefox acceptance are not claimed;
full Android lint still has inherited failures.
The [Settings viewer](questions/settings-pane.html) is a withdrawn record,
not a current design.
It presents 24 inspected native views of a two-row Settings pane across both panels,
themes and text scales,
with the closed player.
D84 removed D11's true-peak analysis row,
because analysis is automatic and not optional,
and D85 removed the closing sentence.
D86 then made the two remaining rows always-on behaviour and not settings,
and D87 keeps the Settings page and leaves it empty until D81's template
configuration is designed;
no study of the empty page is built.
[Its evidence boundary](evidence/settings-pane-boundaries.md) records the
adopted Fold placement,
the withdrawn three-row publication,
native row toggles,
both Back paths and an offline replay of those inputs.
Nothing scrolls at either text size,
most of the page below the two rows is empty,
and on the unfolded panel at 200% text `Resume where I left off` leaves
`off` alone on its second line;
the viewer states these rather than deciding them.
Real preferences,
native accessibility and Firefox acceptance are not claimed;
full Android lint still has inherited failures.
D81's template editor is a separate design problem with its own built study:
[`evidence/template-editor-boundaries.md`](evidence/template-editor-boundaries.md) describes it,
and its [verified offline review](questions/template-editor.html) shows the editor as decided (D89 to D96, D98)
and the two lines that get a template (D99).
The design round for templates is closed (D96 keeps the field list where it is).
No light surface from the original queue is left undrawn.
No production implementation is authorized by this continuation.

## Current review and historical rounds

The Search accessibility design review is complete through D75 to D80.
The recorded Search A decisions remain selected.
Whether result titles display file extensions is a separate open visual
question outside this completed round;
no extension-display preference or matcher eligibility rule was selected.
`questions/current.html` shows selected Search A and adopted accessibility
goals with a correction/veto path,
not an unanswered questionnaire.
The [Search accessibility boundaries](evidence/search-accessibility-boundaries.md)
and [inspected TalkBack baseline](evidence/search-talkback-native-baseline.md)
separate desired behavior from the 200% keyboard-closed inner fixture.
Cover traversal,
native result activation,
live announcements and return focus still need separately authorized
implementation verification.
No new IME experiment or production Search work is authorized.
The accessibility study's disposable emulator was stopped.
The next design-only,
keyboard-closed filename-label study used the capped disposable AVD.
`evidence/search-filename-presentation-frontier.md` records the inspected
24-view native matrix and the separately inspected 32-view controls.
`questions/evidence/search-filename-witnesses.json` and
`questions/evidence/search-filename-control-witnesses.json` link sanitized
app-area crops with separate provenance.
The bounded assessment warrants a consequential comparison,
not incumbent-only advice;
`doc/planning/music-player-search-filename-comparison.md` records its
proposed scope.
The inspected 32-view actual production-renderer baseline is now in
`evidence/search-filename-actual-player-baseline.md` and
`questions/evidence/search-filename-actual-player-witnesses.json`.
Its synthetic paused inputs close the copied-row rendering gap,
not real playback or complete `MainActivity` integration.
Unlike the narrower replica,
the actual ordinary player retains long suffixes on inner at 100%.
Usable placement/visibility alternatives are now built,
captured and inspected.
`evidence/search-filename-usable-comparison.md` and
`questions/evidence/search-filename-comparison-witnesses.json`
record 64 initial and six scrolled native views.
The fixed-policy filename question is withdrawn.
D81 preserves supporting-text templates configurable in Settings.
Task 141 corrected `questions/search-filename-comparison.html` to an
evidence-only review with optional observations and the separately
provenanced actual-renderer baseline.
The comparison captures are authored layouts,
not template-engine output or implemented presets.
Offline checks decoded and opened all 102 images.
Separate desktop/mobile light/dark checks exercised modal/viewer controls
and optional empty/blank observations,
inert adversarial notes and stale-observation invalidation.
Closed-page A/AA audits had no violations or incomplete checks.
Open-dialog desktop metadata contrast remained axe incomplete;
flat opaque computed-color and live-hit-test review resolved that bounded
manual check without changing the axe result.
`questions/evidence/search-filename-comparison-review-verification.json`
retains the exact coverage;
no console errors were observed.
The current disposable runtime was restored to its fresh snapshot and stopped.
The policy recommendations are withdrawn,
not accepted defaults.
Template/default and distinguishing-information interactions remain open.
No dependent implementation or native activation is established.

Status vocabulary follows `open-questions.md` section 0:
settled decisions are not shipped behavior,
provisional choices have a revisit point,
open entries need design,
and developer-owned entries are feasibility or implementation work.
Archived candidates and past round descriptions do not reopen settled
questions.

An earlier dark-color round applied the standing true-black dark requirement while
following Android dynamic color.
 It preserved accepted option 3B,
 transport 1B,
 strict
adaptive mode behavior,
 and the completed accessibility decisions.
 The native Compose
matrix separated three structural surface reaches from two current-row neutral sources
across six representative Android palette environments at the target's opaque 2076 ×
2152px panel resolution.
 Wallpaper rows were uncontrolled stress inputs,
 not choices.
The user settled the round with R1 stable black structure and C2 generated
`surfaceContainerLow` current row;
 decisions D41 and D42 record the choices.
 The form
prepared a visible answer and directed the user to reply in chat;
 it intentionally has no
clipboard API or copy button.

The folded-cover screen round compared four native subdirectory-picker prototypes:
P1 outlined text field plus floating menu panel, P2 the same field plus in-slot picker,
P3 app-bar title plus floating panel, and P4 title plus in-slot picker.
 D46 selects P4
as a **temporary pre-1.x decision**: it is the current design baseline, but the user
believes a better picker exists.
 Post-decision native P4 captures now cover dark at 200% and accepted L3 light at
100% and 200%, in addition to dark at 100%; the opened-state visuals fit without mode
border clipping.
 A debug-only native interaction study verifies title open/close, Back, same-folder
dismissal, and keyboard focus retained after Back at 200%; TalkBack focus remains
unmeasured.
 The light surface itself is settled by D45 (flat with hairlines
at both seams).
 P4 remains provisional and a better picker may be explored before 1.x,
 without a promised replacement.

D47 replaces the command-bar presentation with a **Search button opening a separate
page**.
 D48 merges Back,
 input and Clear into one baseline MD3 Search header.
D49 corrects the design target:
 every platform follows the Pixel 9 Pro Fold cover
(1080 × 2424px) and unfolded inner display (2076 × 2152px).
 This AVD reports
390dpi on both panels,
 so its native captures represent approximately 443 ×
994dp on the cover and 852 × 883dp on the inner display.
 The older 411 × 923dp
cover estimate was not measured on the AVD.
 The earlier 360,
 480 and 1100px desktop Slint scenes are
not design evidence for these screens.
 The native Compose Search study captured both Fold panels,
 but its unfolded
page placed the Back/query/Clear header above an empty left pane and isolated
results in the right pane.
 The user rejected that composition.
The withdrawn form and its captures remain archived at
`questions/archive/search-rejected-fold-review.html`.
 Its empty connector
passed a superseded geometric guard but did not make the page usable.
D50 additionally requires the unfolded playback deck to stay visible during
Search,
 including with a keyboard in view,
 except for the measured real-Gboard overlays accepted by D53 and D54.
 The user selected A (D51):
 Search stays on the unfolded right,
 and the
bottom-left playback deck rises above the keyboard while typing.
D52 removes the redundant positive-results `Results for “cam”` heading on
both panels.
 The former three-way matrix remains at
`questions/archive/search-three-way-before-a.html`.
`questions/current.html` now shows only the selected design in light/dark
at 100% and 200% text.
The corrected unfolded typing captures keep the **same folder browser** above
the lifted deck;
the user accepts a little crop of that browser in short keyboard viewports
and rejected a substitute `Current folder` caption.
A system-managed 300dp debug keyboard verified bounded bottom-keyboard
occlusion and input routing;
it is not Gboard.
The linked older real-Gboard PNGs predate this browser correction.
Real floating Gboard at 200% text obscured part of the unfolded deck title;
D53 accepts that measured overlap.
On the cover at 100% text,
the floating keyboard obscured both matching result labels while typing;
D55 separately accepts that state while the query remains visible.
The user's A selection is unchanged.
A disposable Fold then verified real split inner and full-width cover Gboard
at 100% and 200% text, with the deck and cover results visible in settled
states.
Its Gboard font-update banner briefly clipped the final inner mode;
D54 accepts that measured brief overlap.
Ordinary settled docked/split keyboards still require the full deck,
and other keyboard states are not verified by those exceptions.
The original AVD's active Gboard was updated versionCode `175981944`,
not the preloaded `175753756`;
updating the disposable Gboard to that same build did not reproduce the
floating layout.
`package/music-player/design/evidence/gboard-geometry.md` indexes sanitized real-keyboard captures
and bounded accepted overlays.
The [selected Search A crease measurement](evidence/selected-search-crease-geometry.md)
records the visible central band and app-node geometry;
it did not determine a floor.
After the [native player/Search comparison](evidence/crease-floor-native-comparison.md),
the user chose **7.5mm total** minimum between opposing informational marks,
even for a future narrower-crease device.
E2 still permits backgrounds,
borders,
padding and hit regions to cross the center.
D58 corrects the Search-opened result columns:
Back and result-icon paint share a leading center,
while the query and result titles share a start on the disposable Fold at
100% and 200% keyboard-closed text scales.
D59 emphasizes each visible result-title or matching-parent `cam` substring
in place with bold ink over an OS-accent-derived OKLCH fill,
not the rejected purple tertiary container.
The [native alignment evidence](evidence/search-header-result-alignment.md),
[native match-emphasis evidence](evidence/search-match-emphasis-native.md)
and refreshed selected-only `questions/current.html` show these corrections;
historical keyboard-open captures remain labeled as pre-D58-to-D61
geometry evidence.
D60 selects direct folder/filename matches,
not parent-only track expansion.
D61 selects mixed relevance across folder and track types,
not grouped folder-first or track-first display.
The [archived native comparison](questions/archive/search-ranking-before-dm.html)
and [fixture evidence](evidence/search-ranking-native-comparison.md) retain
all D/P and M/F/T alternatives as decision history,
not active choices or a selected scorer.
D62 records that choosing an eventual fuzzy search library does not
change this UI review and is not work to undertake now.
The [matching boundary ledger](evidence/search-matching-acceptance-ledger.md)
separates selected visible examples from future engine concerns.
The W/A [native mid-word exploration](questions/archive/search-word-boundary-deferred.html)
and [its evidence](evidence/search-word-boundary-native-comparison.md)
are historical and unadopted;
no matcher rule or library is selected by them.
D63 to D68 select Search entry focus,
visible Back,
Clear focus state,
new-visit query behavior,
conditional restored-query position and same-query row visibility.
The [navigation and focus evidence](evidence/search-navigation-focus-boundaries.md)
and [archived logic comparison](questions/archive/search-navigation-focus-before-selection.html)
explain the alternatives;
the [active Search A review](questions/current.html) shows only selected
goals alongside the retained browser and deck.
Native realization of focus/scroll behavior remains unverified.
D69 to D71 distinguish an unqueried prompt,
a fully evaluated no-match state and a known Search-source failure;
they reject the debug fixture's unsupported automatic-recovery claim.
The [status evidence](evidence/search-empty-unavailable-boundaries.md)
and [keyboard-closed native gallery](questions/search-status-evidence.html)
show these states on the selected Fold layout,
not an implemented lookup or a selected cause-specific recovery control.
D72 to D74 now select folder hits opening their folder without autoplay,
track hits carrying the existing play/pause row behavior,
and successful track actions **returning to player**.
The [activation evidence](evidence/search-result-activation-frontier.md)
and [archived Stay/Return logic comparison](questions/archive/search-result-activation-before-return.html)
explain the choice;
`questions/current.html` shows only the selected action goals.
No Search result tap handler had been tested or implemented when D72 to D74 were written;
D127 (2026-10-14) later changed the other-track result from "start" to "select, paused", as for a track row.
D75 to D80 adopt query-first accessibility entry,
contiguous Search reading order,
single-action rows,
non-stealing truthful announcements,
outcome-aware return focus and identity continuity.
These design recommendations have a chat correction/veto path;
complete native accessibility acceptance is deferred,
not demonstrated by the static screenshots.
Selection does not authorize production implementation.
 Desktop implementation inherits
the Fold visual choices even if its own proportions would suggest a different layout.
The rejected command review remains at `questions/archive/command-igr-rejected.html`.
D21's configurable global hotkey and extra Settings row belonged to the command bar;
they do not silently move to Search.
 D25's Ctrl+F reservation is part of the keyboard map D102 accepted.
 Result actions and status truth conditions are settled design goals under
D69 to D74.
Exact cause-specific recovery needs a future source-status owner;
arbitrary-keyboard fit remains unverified and does not authorize further
IME studies.
 Matcher grammar and tie-break implementation are deferred;
 the canned captures do not constitute a running search index.
 The live backlog and developer-owned items are in
`open-questions.md` section 0b.

The preceding accessibility review settled pane-by-pane TalkBack traversal (F1) and
structured current-track speech (S1).
 The prototype repairs the adaptive mode control as
one four-item radio list at one row,
 2×2,
 and four rows;
 reduced motion forbids
autonomous or decorative motion.

The preceding accepted scale review covers every font scale exposed by the target Android
Settings UI:
 0.85,
 1.0,
 1.15,
 1.30,
 1.50,
 1.80,
 and 2.0.
 The mode
group uses one content-sized row at 85%,
 2×2 from 100% through 130%,
 and four
rows from 150% through 200%.
 Every segment retains at least 12dp horizontal content
padding.
 The `Shuffle` cell caps longer subdirectory names at rendered `Camellia`
width with a middle ellipsis while retaining the full accessibility name.

Each round embeds its visual evidence,
 option assessments,
 recommendation,
 and answer
controls in one file.
 Questionnaire HTML follows the viewer's light or dark system preference;
never force a light review surface.
 Each review file is rendered and interaction-checked.
Headless verification is sufficient unless the user explicitly asks for
visible presentation;
no KWin/window-management step is a completion gate.
 Android evidence comes from a non-functional Jetpack Compose prototype
installed on the target emulator,
 not from CSS redraws.
 The active prototype is
preserved on `prototype/music-player-theme-compose`.

After each correction,
answer,
decision,
candidate,
or verification result,
update [`HANDOFF.md`](HANDOFF.md) immediately.
Do not defer the handover update until the end of a session.

## Historical source

The imported archive remains represented by the original root documents,
`candidates/`,
`screenshots/`,
`uploads/`,
and the bound Material Design 3 files under `_ds/`.
Historical candidates remain evidence for prior decisions,
including rejected directions.

## License

Repository-authored work is licensed under the
[GNU Lesser General Public License, version 3 or later](../../../LICENSE).
The imported Material Design 3 recreation documents its source and attribution requirements in
[`_ds/material-design-3-design-system-503c3571-4db9-4b54-8695-7f53861f5059/readme.md`][].

[`_ds/material-design-3-design-system-503c3571-4db9-4b54-8695-7f53861f5059/readme.md`]:
  _ds/material-design-3-design-system-503c3571-4db9-4b54-8695-7f53861f5059/readme.md

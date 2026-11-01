# One IntelliJ-aligned music-player keyboard map

## Purpose and authority

This is the next design-only queue item after verified first-run evidence.
The human closed custom Android notification design in D82:
accept the platform presentation.
No notification variants or new filename questions are queued.

D25 requests one whole-map IntelliJ alignment pass,
not binding-by-binding questions.
D25 already reserves `Ctrl+F` for Search and `Ctrl+O` for the folder picker.
F1's other bindings remain a draft;
no production keyboard implementation is authorized by this plan.
The artifact must make focus-dependent consequences inspectable before
any remaining meaningful preference question is asked.

## Independently verifiable queue

- [x] Compare the settled app actions and draft map against current
  IntelliJ defaults and existing input handling.
  Completion requires exact key sequences and named action/scope
  differences,
  not assuming every editor command has a music-player analogue.
- [x] Build an isolated,
  no-audio whole-map behavior demonstration with editing,
  focused-control and application-level cases.
  Completion requires guarded tests showing keys reach their intended
  owner without hidden playback or source changes.
- [x] Verify the demonstration and publish one coherent proposed map,
  with rationale and any genuinely unresolved product consequence.
  No cosmetic key-by-key ballot or production implementation follows.

No system-wide key injection,
KWin automation,
real playback or new Android IME experiment is involved.
Browser-scoped keyboard demonstration cannot prove OS-level shortcut or
media-key delivery.

## Settled app boundaries

`Ctrl+O` opens the picker for the current library's folders.
D25 explicitly replaced its earlier directory-opening meaning.
It is not silently repurposed to the filesystem chooser that changes the
library source.
D10's `Open a folder` source action remains distinct.

D43 removes in-app volume control.
The old draft's bare Up/Down volume mapping is superseded;
normal focused-widget navigation does not become another volume feature.
D82 supplies no custom notification control design to imitate.

D63/D67 distinguish entering Search from refocusing the same visit.
A Search shortcut while already there must not manufacture a fresh visit
that loses the query.
D65's Clear behavior and D66's genuine new-visit behavior stay separate.
The shortcut map does not settle unsaved-template handling,
editor grammar,
matching or source-recovery ownership.

## Fresh existing-input audit

The current source scan covered Android's main activity and the desktop
Slint surface plus Rust sources.
The Android activity did not expose an explicit global shortcut handler
in that inspected scope.
This is a source-scope observation,
not a claim that platform defaults or all dependency input paths are absent.

Desktop `package/music-player/desktop-app/ui/app.slint` contains local
`key-pressed` handlers on page disclosure,
mode and transport focus scopes.
At lines 1099,
1356 and 1461,
Space/Return activates the focused control and other keys are rejected by
that handler.
Those controls own Space;
an application-level toggle must not fire a second action after native
control activation.
Source presence proves neither keyboard reachability nor OS input delivery.

## Current IntelliJ evidence

The official [Windows keymap][windows-map] and
[macOS keymap][mac-map] were fetched,
including their Markdown forms to avoid the rendered page's concatenated
shortcut labels.
The literal sequences include:

- Search:
  `Ctrl+F` on Windows;
  `Command+F` on macOS.
- Settings:
  `Ctrl+Alt+S` on Windows;
  `Command+Comma` on macOS.
- Caret word movement:
  `Ctrl+Left`/`Ctrl+Right` on Windows;
  `Option+Left`/`Option+Right` on macOS.
- Caret line start/end:
  `Home`/`End` on Windows;
  `Command+Left`/`Command+Right` on macOS.
- Go to line/column:
  `Ctrl+G` on Windows;
  `Command+L` on macOS.
- Select previous/next tab:
  `Alt+Left`/`Alt+Right` on Windows;
  `Command+Shift+[`/`Command+Shift+]` on macOS.
- Back/forward context navigation:
  `Ctrl+Alt+Left`/`Ctrl+Alt+Right` on Windows;
  `Command+[`/`Command+]` on macOS.

These are primary-source precedents,
not adopted music-player bindings or proof of Linux compositor/macOS
shortcut availability.
D25's explicit app assignments take precedence over a mechanical clone
of unrelated IDE coding actions.
The final proposal must state product exceptions rather than label them
exact IntelliJ defaults.

## Consequence and verification gate

The consequential concern is ownership:
a caret-motion or focused-control key must not unexpectedly change the
playing track,
source or playback mode.
A whole-map demonstration can make that visible without real audio.
No measurable fit or reversible key-label detail needs another questionnaire
by itself.
The human's existing request for the whole map is preserved without
turning its individual entries into separate rounds.

The audit and bounded behavior demonstration are complete.
`package/music-player/design/questions/keyboard-map.prototype.html`
is the verified self-contained proposal,
with optional whole-map observations and no per-key ballot.
`package/music-player/design/evidence/keyboard-map-boundaries.md`
and its verification manifest retain exact scope and limitations.
Committed classifier tests and fresh copied-program removal of player,
logical-composition and popup guards produced intended failures;
positive/restored checks passed.
The browser checks exercised caret motion after an End-key positive control,
focused-control ownership,
Search visit behavior,
popup containment and current-row reveal in four responsive/theme contexts.
The human accepted the whole map, including the playback-region scope, as built (D102).
The next independent design item is the settled light error/Undo state
family,
recorded in `doc/planning/music-player-light-error-undo.md`.
Mac/Linux global reservations,
media-key delivery,
accessibility traversal and native production binding remain verification
limits until exercised through their actual owners.

[windows-map]: https://www.jetbrains.com/help/idea/reference-keymap-win-default.html.md
[mac-map]: https://www.jetbrains.com/help/idea/reference-keymap-mac-default.html.md

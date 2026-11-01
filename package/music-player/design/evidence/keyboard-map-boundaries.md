# Whole-map keyboard proposal and input-owner evidence

## Purpose and acceptance boundary

D25 requests one IntelliJ-aligned keyboard map,
not individual key-label questions.
[The self-contained behavior demonstration](../questions/keyboard-map.prototype.html)
presents one coherent proposal with optional observations.
D25 settled the Search/picker assignments,
and the human accepted the whole map, including the playback-region scope, as built (D102) on 2026-10-10.
No production keyboard implementation is authorized.

[The planning record](../../../../doc/planning/music-player-keyboard-map.md)
contains the current official keymap sources and executable input audit.
The HTML is an isolated behavior sandbox,
not an Android or Slint screen mockup.
It reads no library,
performs no source operation and plays no audio.
It implements no Search matcher or template engine.

D82 accepts Android's standard media notification presentation.
This proposal introduces no custom notification design or media-key UI.

## Proposed action vocabulary

The demonstration's platform selector changes the displayed proposal,
not an actual OS keymap.

- Search retains `Ctrl+F` or the proposed macOS `Command+F` equivalent.
  A same-visit refocus preserves query text;
  a genuine new visit starts empty in the authored model.
- The folder picker retains `Ctrl+O` or the proposed macOS `Command+O`
  equivalent.
  It means the current-library folder picker,
  not choosing a new filesystem source.
- Settings uses the documented IntelliJ conventions:
  `Ctrl+Alt+S` and `Command+Comma`.
- Reveal-current proposes `Ctrl+G` and `Command+L`,
  using the IDE's go-to-location precedent.
  It focuses the current authored row without a transport or source action.
- Previous/next proposes `Ctrl+Left`/`Ctrl+Right` in the Windows/Linux
  playback area and `Command+Shift+[`/`Command+Shift+]` in the macOS model.
  The latter uses the IDE's previous/next-tab chord precedent.
  The area itself must have focus;
  its descendant buttons and slider are control owners.
  These transport meanings are product exceptions,
  not literal IDE music operations.
- Space toggles the authored player only while the playback shortcut area
  itself has focus.
  Editing and focused controls retain Space.
- `Ctrl+M` cycles authored end-of-track mode in the playback area,
  an explicit product exception in both displayed models.
- Bare arrows retain native caret,
  focus and focused-control behavior;
  no global seek or volume alias is introduced.
- Unmodified letters remain widget-owned.
  Native folder typeahead is not modelled or promised by this artifact.
- Escape dismisses the innermost authored popup,
  otherwise returns from Search when its enclosing surface owns Escape.
  A logical composing flag delegates the key to its owner;
  no real IME behavior was studied.
- Hardware media and volume keys remain platform-owned delivery work,
  outside the demonstration.

The Windows/Linux grouping is a proposal,
not proof that every Linux compositor delivers these combinations.
macOS display/protocol checks do not prove macOS event delivery or
shortcut availability.

## Ownership controls

The consequential concern is preventing a text/control key from also
changing playback.
The demonstration separates editor,
focused-control,
playback-area,
popup and surrounding-review scope.

The browser consumer established:

- A native End key first placed the query caret at its end,
  providing a positive control.
  Control+Left then moved it toward the preceding word without a
  transport action;
  Space inserted text without toggling playback.
- Refocusing the same Search visit preserved its value;
  returning and opening a new visit cleared it.
- Space on a focused button produced exactly one native button action,
  not an additional application toggle.
- ArrowRight changed a focused range control without changing the track.
- The authored folder/Settings popups contained transport commands;
  Escape closed the popup.
- Reveal-current focused the identified authored row without starting or
  substituting another track.
- Surrounding review-note events did not leak into application shortcuts.

The popup is an operation-intent model,
not real folder enumeration,
Settings editing,
permission recovery or source selection.
The demonstrated rows are authored labels,
not indexed files.

Committed tests execute the actual inline classifier through a disposable
consumer.
Fresh copied-program removal of the playback-owner,
logical-composition and popup-owner guards failed the intended assertions;
positive and restored tests passed.
The playback-owner guard verifies the proposed scope,
not a universally accepted shortcut policy.
Logical composition cases prove no real Android or desktop IME behavior.

## Consumer verification

[The verification summary](../questions/evidence/keyboard-map-prototype-verification.json)
pins the HTML and committed classifier test.
Checks ran offline in a 2GiB/2CPU Chromium container.
Browser-dispatched keys using the Windows/Linux model exercised the
four desktop/mobile light/dark contexts and authored actions,
editing/control ownership,
popup handling,
current-row reveal and optional inert observations.
macOS assignments were exercised through protocol classification and
rendered map selection,
not native macOS key delivery.

Visible closed-page controls met the 48CSSpx check.
Four closed-page A/AA axe audits had zero violations or incomplete checks.
No console errors or horizontal document overflow were observed.
No open-popup audit,
Firefox acceptance or native accessibility claim follows.
The scripted screenshots were inspected after reading their rendered state.
The owned browser was closed and its container removed.

An initial implicit-click verification failed at row reveal.
Explicit center scrolling before trusted native clicks passed the unchanged
artifact.
This uses the separately measured
[partly visible target workaround](../../../../doc/troubleshooting/agent-browser-partial-target-center.md),
not a change to the proposed keymap or evidence that every missed click
shares one cause.

## Remaining boundary and queue

The artifact is the whole map the human accepted (D102),
not an invitation to vote on each key.
Actual OS reservations,
media keys,
native bindings,
widget traversal,
IME handling and unsaved-template ownership need their real operation
owners and separately authorized implementation verification.

The next independent design area is the undrawn light error/Undo state
family under the already settled D8/D9/D29 behavior.
That work does not depend on adopting this proposal,
reopen Android notification design or authorize real trash/restore actions.

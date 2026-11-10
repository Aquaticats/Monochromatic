# Search navigation and focus evidence for D63 to D68

## Selected surfaces and distinct focus meanings

D47 gives the player a Search button that opens a separate destination
with a Back path.
D48 puts Back,
query and Clear in one header.
D51 keeps actual folder browsing and the complete playback deck on the
left of the unfolded Fold,
with query/results on the right;
its cover Search is full-width.
D52 starts positive results directly beneath the header.
D56 keeps the cover result viewport scrollable above an ordinary bottom
keyboard while its header remains fixed.
The currently selected D/M results and D59 match highlights appear in
`package/music-player/design/question/current.html`.
None of those visual decisions says whether **text edit focus** is
requested when Search opens,
or what a second system Back does after the keyboard has hidden.
Screen-reader initial accessibility focus is a separate review task 118 concern and
must not be inferred from keyboard/edit focus.

The archived Material 3 Search guidance indexed in
`package/music-player/design/evidence/search-result-behavior-boundaries.md`
says a focused search bar's Back icon releases focus and returns the bar
to its original state;
they also keep submitted result-query text visible but unfocused.
That is about an expanding **search bar**,
not D47's separate Search
page;
its Back transition cannot be copied as a rule that the selected
page merely collapses a field.
The [Material Search accessibility guidance][m3-accessibility]
says assistive-technology initial focus may land on a leading icon or
text field and must announce changes in suggestions/results.
It does not choose initial **edit** focus for this separate destination.
The archived guidelines were inspected through the local copy indexed in
`package/music-player/design/material-3-compliance.md`;
no Material Search component is a production dependency here.

## What the debug host does, not what was selected

In the **prototype branch** `prototype/music-player-theme-compose`
(commit `cc66a0dcf`,
not production files in the main worktree),
the debug-only inner
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/SearchPersistentDeckStudy.kt`
and folded-cover delegate
`package/music-player/android-app/app/src/debug/kotlin/dev/monochromatic/musicplayer/SearchLayoutStudy.kt`
each initialize temporary `opened` and `query` state from a candidate
marker.
Both set `opened = false` and `query = ""` in an `onBack` callback,
register `BackHandler(enabled = opened)` and reopen from the player with
an empty query.
Both headers' Clear action invokes `onQueryChange("")` without closing
the page.
They use a `BasicTextField` but do not explicitly request text focus
on opening Search.
The inner and cover result regions call `rememberScrollState()` in their
composable view;
that is an in-memory fixture detail,
not accepted scroll-restoration semantics across edits or visits.
These observations come from debug source on
`prototype/music-player-theme-compose`,
not a production Search implementation or a live user-boundary flow.

## Observed refocus limitation and Android Back boundary

`package/music-player/design/evidence/search-result-overflow.md` records
that one cover viewport candidate showed its final result above a measured
bottom keyboard after scrolling.
Hiding then refocusing that keyboard shrank the viewport;
the list offset did not need to jump to the top for the final row to
require another upward swipe.
D56 chose the cover-only keyboard-aware viewport,
**not** that extra swipe as desired behavior.
That is a bounded observation from the completed debug and settled-Gboard
studies,
not a claim about every input method,
posture or animation frame.
A scroll **anchor** and whether its row is **visible** are different
facts;
the logic prototype must represent both.
The desired same-query visibility guarantee still needs native
verification before any production implementation claims it works.
No new IME experiment is authorized without first explaining a compelling
need to the user.

Android SDK 37 `android/inputmethodservice/InputMethodService.java`
documents that its default `onKeyDown` intercepts `KEYCODE_BACK` while the
IME is shown and its default `onKeyUp` hides that IME UI.
The same source describes a conditional IME-owned
`BACK_DISPOSITION_ADJUST_NOTHING` path under which Back can instead reach
the app.
This is not evidence that our app can make one system Back exit Search
across arbitrary keyboards.
Treat keyboard-first system Back as an illustrative platform baseline,
not a user-selected cross-IME guarantee or a universal native claim.
The visible header Back action is a distinct design choice:
D47 promises a Back **path** from a separate destination,
not the first tap's exact outcome while typing.

## Compared directions and selected behavior

- **Entry edit focus (D63):**
  request query focus/keyboard on entry,
  not wait for explicit field focus.
  E-fast was selected;
  E2 remains the unrelated accepted crease-clearance code.
- **Visible Back arrow (D64):**
  return directly to player even with a
  keyboard shown,
  not hide the keyboard first while retaining Search.
  Material's in-place bar-collapse guidance did not determine this.
- **Clear (D65):**
  erase query and stay in Search while retaining the
  existing edit-focus/keyboard state,
  not force new focus.
  The logic model compared typing,
  dismissal with edit focus retained,
  and moved-away edit focus.
- **Re-entry query and position (D66/D67):**
  open with a fresh empty
  query on the next visit.
  Independently,
  if a future design restores query text,
  start those results at the top rather than the prior deep position.
  Conditional P-top does not override the active Q-new direction.
  The temporary debug `onBack` reset was not decision evidence.
- **Same-query refocus (D68):**
  preserve the intended row's **visibility**
  after the viewport shrinks;
  raw offset alone produced the observed final-row extra swipe.
  The comparison separately modeled a middle row and final row;
  it did not demonstrate a native correction.
  A changed query is a different event from same-query refocus.

The Search result position is distinct from the visible left folder
browser's own location and scroll position.
The demonstration fixes a non-default browser context and playback deck
across Search entry/exit;
it does not prove native retention or define a return focus target.
The re-entry comparison concerns visits in one running session;
posture changes and process restoration are not modeled.
Result activation (review task 129),
empty/unavailable content (review task 128),
TalkBack traversal and announcements (review task 118),
and search-library selection (D62,
future implementation work) stay separate.
`package/music-player/design/question/archive/search-navigation-focus-before-selection.html`
retains the interactive **logic-only** comparison as historical evidence,
not a current question or new Android/IME verification run.
The selected D63 to D68 goals are documented in `decisions.md` and the
active `question/current.html` without presenting rejected variants.

[m3-accessibility]: https://m3.material.io/components/search/accessibility

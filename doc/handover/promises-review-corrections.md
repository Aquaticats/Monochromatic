# Promise review corrections

## Authorization and scope

The user delegated implementation with “Address whatever you want to address.”
The assessment is in [the review disposition](../planning/promises-teaching-review-assessment.md).
The review's `Concerns that require real learner observation` section remains vetoed as fine.
No learner-study gate or final teaching-skill approval is being added.

Repair the controller-ownership teaching bridge,
timed Stop interactions,
capstone comparison workflow,
supplied-source boundaries,
fixture metadata explanation,
and `wait`'s Promise-returning contract.
Fold the isolated identity-guard observation into the overlapping-send exercise.
Leave the optional observations alone.

## Preservation and authoring

The pre-change artifact is preserved as
`doc/planning/promises-teaching-fable-reviewed.local.html`.
Its SHA-256 is `aef8e02094efce04096989c4e102c7416edf342e2ff0affd81b8f72c2f478e3b`.
It and the delivered artifact remain ignored local files;
do not force-add them.
The original preserved versions remain untouched.

Authoring directory:
`/var/home/user/temp/agent/promises-revision`.
Build through its `mise run build` task.
The output remains `doc/planning/promises-teaching.local.html`.
Do not reload the user's existing presentation tabs or discard their drafts.
Use disposable verification sessions,
then open a new presentation tab.

## Work log

### Promise-returning wait

`build.mjs` now changes the supplied `wait` declaration to `async`.
Its early abort check therefore rejects its returned Promise rather than throwing out of the call.
Timer startup and cooperating abort cleanup remain in the function body.

`verify-reviewed-helpers.mjs` separates the call and await boundaries,
checks identity of the supplied rejection reason,
checks fulfillment listener removal,
and checks that pending abort clears the actual timer handle.
`review-browser.mjs` owns disposable sessions and closes them through a resource disposer.

The preserved before-state failed in `proc_2c5c` with:

```text
AssertionError [ERR_ASSERTION]: Already-aborted wait must return a rejected Promise instead of throwing at call time
actual: { phase: 'call', sameReason: true }
expected: { phase: 'await', sameReason: true }
```

`proc_b4cd` rebuilt and passed the return-boundary checks.
`proc_ac07` passed the additional timer-handle check and the retained policy/workshop regression in 74 seconds.
This covers the existing advanced verifier,
whose capstone execution omission is still being repaired separately.
No full-lesson completion claim is made by this helper check.

### Ordered Stop without changing clocks

The isolated cancellation/deadline previews now have a labelled lesson control that activates Send then Stop
within one callback,
using the learner's actual listeners.
An unwired Stop still does nothing;
ordinary Send still reaches the real reply or native deadline.
The control records the state after Send rather than requiring the learner to catch an intermediate paint.
It is included in standalone practice downloads.

The reference has `Reset and stop A during backoff`.
Its policy creates the real backoff wait before notifying the UI.
The prearranged example then activates A's ordinary Stop button from that notification.
A has overlapping sends;
B continues independently.
This is a scripted action order,
not a manual clock or a longer click window.
The example's fixed scenarios are disclosed separately from subsequent manually sent messages.

`proc_bdb5` passed starter/solution behavior,
the deadline path,
exported controls,
a directly observed pending backoff timer at Stop,
and existing reference ownership regressions in 32 seconds.
`proc_4997` rejected the reversed-notification-order mutant:
Stop observed zero pending backoff timers instead of one.

### Overlapping-send ownership bridge

`ownership.html` teaches identity,
named `filter` predicates,
returned arrays,
and `for…of` iteration.
It also explains the reference's equivalent arrow-function spelling.
`ownership-solution.mjs` supplies the worked controller-list implementation;
the build derives an intentionally broken clear-the-whole-list starter from it.
The isolated identity guard is now described as defensive where Send is disabled,
not presented as proof of an interleaving that cannot occur through those controls.

`ownership-fixture.mjs` supplies explicitly manual reply completion,
rejection,
and abort.
Its held-reply list is distinguished from the learner's controller list and browser queues.
Lesson controls can send a pair or request Stop and immediately send again in one callback.
They and the helper are included in the practice download.

`proc_027d` passed in 15 seconds:

- Broken starter exposes loss after partial completion and after Stop followed by a fresh send.
- Worked solution keeps the remaining controller and preserves drafts.
- Newest-first completion,
  rejection,
  and blank input behave as documented.
- Already-aborted calls return rejected Promises;
  the fixture's 20-call bound is enforced.
- The print appendix includes the new task and source.
- The actual downloaded exercise retains the interactive controls and ownership behavior.

Full PDF and cross-feature validation remain separate final steps.

### Complete capstone comparison

The capstone comparison now combines the actual policy source and `capstone-ui.mjs`.
Its duplicate fixture destructuring is removed,
and the common workshop prefix explicitly supplies `ServiceFault`.
`capstone-controls.mjs` supplies fixed markup only;
the comparison wires the actual interface.
The comparison has two independent conversations,
per-send controllers,
a shared runner,
scenario selection,
overlapping-send examples,
and the prearranged backoff Stop path.
Its histories preserve line breaks and unsent drafts.

The lesson now states the supplied names and additional control IDs,
and explains destructuring,
`ServiceFault`,
and the comparison's complete execution context.
Printing takes the complete comparison from the same source definition.

`proc_2fe3` passed the real editor/Run workflow across scenarios,
Stop/new-send ownership,
input limits,
printed source,
and the actual downloaded capstone in 63 seconds.
`proc_49d5` repeated it with the event-log eviction boundary and passed in 69 seconds.
`proc_d8a4` rejected the preserved syntax defect at the same comparison-entry assertion.

### Source and fixture boundaries

`shop-source-guide.html` explains the opening's additional object-parameter,
DOM,
string,
focus/scroll,
and callback-list operations.
The disclosure no longer promises the main function/callback chapters explain every preview line.
The fixed-panel route remains the required-build alternative to dynamic DOM construction.
The reference guide also explains its shorthand/default/conditional spellings and numeric display helpers.

The API explanation and retry hint now distinguish the application's retry counter
from the fixture's explicitly supplied outcome selector.
They state that the fixture does not remember earlier calls,
and do not claim real APIs forbid transmitting retry metadata.
The workshop now accurately calls its encoded downloads runnable snapshots,
not convenient plain-source editing layouts.

`proc_5be0` passed the guide's demonstrated operations,
the declared-third-attempt-first and repeated-attempt-one fixture cases,
and the retained shop behavior in 11 seconds.

### Secondary-review availability

No post-change Advisor review has been obtained.
The default Advisor attempt timed out after 600,000 ms;
a second scoped provider returned a billing error.
These attempts are not counted as review evidence.

## Integrated verification and presentation

`proc_0776` passed the combined suite and exports in 256 seconds.
Visual inspection then exposed a separate layout problem:
the new workshop controls still occupied the inherited fixed-height iframe.
The pre-layout-correction artifact is preserved as
`doc/planning/promises-teaching-review-clipped.local.html`,
SHA-256 `a11ced0a8c0bb136928c84163b82cc85388f3378ac506751141de292a785082c`.

`workshop-layout.mjs` now reports content height from the opaque child.
The parent validates the source window,
current run,
finite height,
bounds,
and clipping flag.
The iframe grows in document flow,
up to a 3,000 CSS-pixel bound with an explicit overflow notice.
History and event records have bounded,
keyboard-focusable scrolling areas;
print removes their height limits.
The Stop walkthrough's prose record wraps rather than requiring horizontal scrolling.

`proc_bad2` rejected the preserved layout with:

```text
Workshop content must participate in page flow rather than a clipped pocket: {"content":789,"viewport":238}
```

`proc_1b88` passed native opaque-frame pointer input,
mobile/desktop fitting,
positive resizing,
wrong-source/stale-run/type/range rejection,
and oversized-content recovery.
An additional ownership probe resolves a reply then aborts before its continuation;
removing the success guard in a disposable editor makes the stale-reply assertion fail.

### Final combined run

`proc_8c3a` passed the complete suite,
exports,
and PDF rendering in 298 seconds.
This includes all workshop choices through their explicit execution drivers,
opening and reference regressions,
neutral surfaces and sampled contrast,
native editing,
complete PDF inventories,
native Print/Cancel,
and HTTP-denied standalone exports.

Current artifact:

- `doc/planning/promises-teaching.local.html`.
- 463,920 bytes.
- SHA-256 `319ffcb20481d468beb6353d6bbe94b682e55cb682c3542dffefc983f6e82eb7`.
- The preserved Fable-reviewed artifact still has its original `aef8e020…` hash.

Current PDF:
102 Letter pages,
2,138,735 bytes.
Inventory checks cover 584 teaching entries and 66 appendix entries.
They normalize whitespace;
they are not a claim that every page was visually inspected.

### Firefox verification

The [Firefox BiDi route](../troubleshooting/firefox-bidi-opaque-previews.md) uses a fresh ESR 140.15.0 profile.
`proc_a4b4` passed the semantic corrections in 26 seconds.
After the layout change,
`proc_7b59` read the first allocation too early:
content height was 1,288 while the viewport still measured 1,007.
`proc_241b` read the same unchanged frame later and measured 1,288 for both.
The verifier now waits for actual content fit,
not just receipt of the first height report.
`proc_f316` passed two complete Firefox runs with that readiness condition and rendered the PDF pages in 51 seconds.
This is not a Firefox implementation change or a relaxation of iframe isolation.

### Visible handoff and resource cleanup

`proc_7007` opened a new public tab at `#pending-ownership`.
Its target is `117B0A085A92ED273577EB120AF200F3`,
tab `t3`.
The previous `t1` and `t2` target IDs and URLs were preserved without reloading their content.
The new document exposes the completed capstone source and the current layout protocol;
its measured root background is `rgb(30, 30, 30)`.

After monitor/geometry changes,
focus was applied again after the move settled.
KWin's `PI_PROMISES_REVIEW_PRESENTED` record showed the lesson active and non-minimized on `DP-3`,
with `DP-3` also the current output.
The heading was then realigned after the viewport resize:
its top measured 0.078125 CSS pixels in a 1,812 CSS-pixel-high viewport.
`review-corrections-visible.png` was recaptured and inspected in that state.
The temporary KWin scripts were unloaded.

The reviewed PDF pages were 2,
32,
33,
35,
36,
48,
49,
88,
89,
91,
92,
93,
and 94.
These include the new source guide,
ownership explanation and exercise,
Stop trace,
capstone execution contract,
and policy/interface comparison.
Source continues across pages;
the whole-source inventories separately check retention.

`proc_e124` completed cleanup in 8 seconds.
It closed only `promises-print-ui` and `promises-revision-verify`,
stopped processes identified by their owned profiles or fixture working directory,
and removed the explicitly named print,
export,
and Firefox verification profiles.
The public `promises-open-chat-present` session remained,
including its preserved older tabs.
Some older fixture process registrations were no longer available;
OS process inspection established their surviving children before cleanup,
rather than starting duplicates.

## Completion boundary

The delegated concrete lesson corrections are implemented,
verified,
and presented.
The learner-observation veto remains settled.
This is not final approval or implementation of the teaching skill.

## Subsequent correction to presentation policy

The user clarified that the visible-window requirement had been added to `AGENTS.md` in error
and explicitly requested its removal.
The canonical visible-window delivery rule was removed;
its retired shortcode is reserved in the existing local appendix.
The overlapping automatic-open clause in the HTML-form rule and its visible-artifact wording were also removed.
`CLAUDE.md` is regenerated through file-enforcer rather than edited separately.
The music-player handoff and its binding review notes now reflect the same correction;
historical window observations remain,
but their activation directives no longer act as completion gates.
Browser verification remains distinct from a requirement to show an active window.
The window observations in this handover are historical evidence,
not a requirement for future HTML work.

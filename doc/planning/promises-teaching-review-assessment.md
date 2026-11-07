# Assessment of Fable's Promise lesson review

## Scope and the user's veto

The user supplied [Fable's independent review](promises-teaching-independent-review.md)
and marked its section `Concerns that require real learner observation` as fine.
That section is not a blocker and does not create a requirement for learner studies.
Do not reopen its volume,
retention,
or learning-outcome questions as prerequisites to proceeding.

The separately identified source,
exercise,
and runtime findings remain reviewable.
This assessment recommends changes;
it does not treat the review as authorization to implement them.
The reviewed artifact still has SHA-256
`aef8e02094efce04096989c4e102c7416edf342e2ff0affd81b8f72c2f478e3b`.
Neither the lesson nor its authoring sources was changed during this assessment.

## Findings to retain

### Controller ownership needs a teaching step: findings 1 and 7

The required outcome includes overlapping sends with independently owned cancellation.
The cancellation exercise demonstrates a single operation with Send disabled,
while the capstone requires a conversation's list of pending controllers.
The workshop definitions at `doc/planning/promises-teaching.local.html:2317` through `2346`
contain no intervening exercise that builds this list.

The paragraph in `#threads` does explain what its `filter` expression retains.
Fable's wording that there is no explanation at all overstates that part of the evidence.
However,
the paragraph neither introduces arrow-function syntax nor provides practice with the ownership transition.
The narrower prerequisite and exercise gap remains.

Recommendation:
add a bridge that has the learner manage overlapping sends with constructs explained at that point.
Make the identity guard meaningful through an actual older-completion/newer-send interleaving,
rather than only showing it in the send-disabled exercise.
This addresses the code and curriculum structure without requiring a learner study.

### Fixture metadata needs an explicit boundary: finding 2

`#provided-api` states that `attempt` supplies the attempt number;
the retry hint tells the learner to pass it into `fakeReply`.
The fixture uses it to choose the scripted `recover` and `limited` outcomes.
Explain that role alongside the application's own retry bookkeeping.

Do not adopt the proposed universal statement that a real request carries no attempt counter.
[OpenAI Python's request-header source](../troubleshooting/api-retry-count-metadata.md)
is a counterexample:
it supplies `x-stainless-retry-count` metadata.
That does not imply a server promises success after a client-selected number of attempts.

### The opening-source promise needs an honest boundary: finding 3

The disclosure says the function and callback chapters explain the shop source.
Its DOM construction,
string matching,
queue removal,
and parameter syntax extend beyond that teaching path.
Make clear which parts illustrate the taught structure and which are supplied UI/fixture machinery.
Distinguish that machinery from the fixed-panel,
`join`/`textContent` route the learner may use.
A source guide must actually explain its additional operations;
simply naming more methods is not the same as teaching them.

### The cancellation demonstrations still contain timed opportunities: finding 4

The isolated exercises use a 1,600 ms reply and a 1,800 ms deadline.
There is also a remaining example outside those exercises:
`#threads` asks the learner to click Stop during a retry delay.
The shown policy's waits are 500 ms and 1,000 ms.
The automatic reference example removes the race to start overlapping sends,
not this separate race to catch a particular phase.

Increasing a timer to 6 or 60 seconds retains a finite opportunity to perform the action.
Recommendation:
give the learner control over advancing the relevant events,
with the simulation boundary stated explicitly.
The check should establish the intended Stop path at the chosen phase,
not merely successful cancellation after one selected delay.
This is an interaction requirement,
separate from the vetoed learner-observation concern.

### The capstone comparison has an actual execution defect: finding 5

Confirmed in a disposable copy through the workshop's editor and Run control:

```text
JavaScript error: Uncaught SyntaxError: Identifier 'wait' has already been declared
```

`#policy-runtime` declares `wait`,
`fakeReply`,
and `ServiceFault` from `window.promiseLab`.
`exerciseHtml` also prefixes the submitted code with declarations for `fakeReply`,
`wait`,
and `seededRandom`.
Both declarations therefore occupy the same module.

The proposed deletion of the policy's destructuring line is not a complete repair.
In the disposable editor,
it removes the syntax error but:

- Clicking Send leaves the result at `Ready`;
  the policy module does not wire the workshop controls.
- Invoking that runner with the denied scenario produces
  `ReferenceError: ServiceFault is not defined`,
  because the remaining prefix does not supply it.

The solution's execution context and advertised behavior must be repaired together.
A check for an empty error display immediately after loading is insufficient.

The original advanced verifier excludes capstone from the loop that runs worked comparisons.
Its capstone check only loads the task text and exercises surrounding controls.
This was a concrete omission in the previous verification,
not evidence against Fable's finding.

### `wait` has a mixed failure surface: finding 6

The existing `wait` returns a Promise for a fresh signal but throws at call time for an already-aborted signal.
Its implementation calls `signal.throwIfAborted()` before creating or returning the Promise.
A disposable async wrapper changes that observed failure into rejection at the await boundary.

Recommendation:
make the supplied helper's behavior consistent with its Promise-returning teaching contract,
and cover fresh,
already-aborted,
and later-aborted calls.
The wrapper probe establishes the call-versus-await distinction,
not complete verification of a future implementation change.

## Findings to leave as choices

Finding 8 identifies Enter-to-send behavior,
adoption-chapter placement,
and delayed offscreen timers.
It does not establish a required redesign.
The user's veto does not need to be reopened through these observations.
The existing shop purpose,
independent sending,
neutral reading surfaces,
and complete print presentation should be preserved.

## Verification performed for this assessment

Private workspace:
`/var/home/user/temp/agent/promises-review-assessment.qUhUdO`.
`mise run probe`,
process `proc_0927`,
completed successfully in 8 seconds.
The probe ran against a byte-identical copy,
used its own browser session,
closed it,
and verified the supplied artifact's hash again afterward.

Observed results:

- Greeting comparison positive control:
  `Welcome Ada` after Send.
- Original capstone comparison:
  duplicate `wait` syntax error.
- Comparison with only the destructuring removed:
  no Send behavior and an undefined `ServiceFault` on failure.
- Fresh-signal `wait`:
  returned a Promise and fulfilled.
- Already-aborted `wait`:
  `AbortError` thrown during the call.
- Async-wrapped already-aborted `wait`:
  `AbortError` observed when awaiting the returned Promise.

The source audit also checked the workshop definitions,
supplied-helper contract,
controller-list paragraph,
reference Stop instructions,
and the advanced verifier's coverage.
It did not repeat Fable's Firefox or complete lesson review.
Fable's reported Firefox coverage extends the previous Chromium-only evidence;
it remains bounded by the interactions listed in that review.

## Recommended direction

Prioritize the missing controller-ownership bridge and the broken capstone workflow,
then remove timed interaction requirements and clarify the source/API/helper contracts.
Fold finding 7 into the ownership bridge.
Keep the vetoed observation section and finding 8 out of the required repair queue.
No lesson fixes or final teaching-skill decisions have been made by this assessment.

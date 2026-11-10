# Critic's brief for the Promise lesson

This brief introduces the artifact completely enough that a critic can attack it without first
reverse-engineering it:
what it claims to do,
what is in it,
why each part is shaped that way,
what the verification proves and what it cannot,
where the weak spots are,
and which sequences
to run with what they actually produce.
It is not a defense.
The weak-spot section is the
intended starting point,
and the decisions at the end are the ones I made without asking.

## What you are looking at

- Current artifact:
  `doc/planning/promises-teaching.local.html`,
  718,990 bytes,
  SHA-256 `270049535caaf4a86063fd9b3322b2f1be3c838f52566c9abcc77fd16941d818`,
  build identity `895a0c508be2a32ffa21cc8c9eae48c9a170d91e09fe77c7633a62d245f47e13`.
- It is a single self-contained file:
  no network requests,
  no build step,
  no dependencies at
  read time.
  Lezer grammars and the Showify inspector are bundled inside it.
- `*.local.*` files are ignored by the repository,
  so this artifact is local evidence rather
  than shipped package code.
  Only the planning documents around it are committed.
- It is open in a headed Helium tab in the `promises-open-chat-present` session,
  scrolled to
  the top.
  Four tabs are open:
  this build plus the two predecessor builds and an earlier
  focused review.
  None of the older tabs was reloaded,
  so their state is intact.
- Print counterpart:
  the latest verified PDF run measured 133 Letter pages,
  2,589,705 bytes,
  with 894 teaching inventory entries and 70 appendix entries,
  each matched against text
  extracted from that PDF.
- Predecessor artifacts preserved for comparison,
  each the exact input to one repair round:
  `promises-teaching-before-ticket.local.html` (`5401b0df…`,
  the crowded factory scene),
  `promises-teaching-before-belt.local.html` (`e90aa8fe…`,
  receipts delayed to Create),
  `promises-teaching-before-review-repairs.local.html` (`fb4631d8…`,
  stale result panels,
  orientation after the shop,
  review scaffolding),
  `promises-teaching-before-pacing.local.html` (`e0f770f8…`,
  no sittings or takeaways),
  and
  `promises-teaching-before-waiting-exercise.local.html` (`33914cd2…`,
  pacing without the
  diagnosis exercise).

## The contract it must meet

- Assumed learner:
  can independently write a JavaScript hello-world program,
  and build a
  single-file HTML page whose script prompts for a name and renders `Hello <name>` in the DOM.
  Nothing beyond that is assumed.
  The lesson states this and does not claim the learner wrote
  any particular greeting program.
- Destination:
  independently build,
  and explain to an interviewer,
  a dependency-free browser
  message interface with a shared rate limit,
  accepted cancellation,
  exponential retries with
  backoff and jitter,
  tolerance of service faults,
  and multiple conversations.
  Promises are
  part of that path,
  not the whole objective.
- Presentation requirements the user set:
  visual,
  interactive,
  self-contained HTML with
  automatic light and dark theming,
  plus a print counterpart that keeps the full teaching
  substance.
  Print may replace interactive mechanisms with static equivalents;
  it may not
  delete content,
  and printability may not be used as a reason to remove browser interactivity.
- Explicit non-goals:
  no real AI provider,
  no credentials,
  no network calls,
  no claim of
  game fidelity for the Yum-Bot-inspired artwork,
  and no claim that passing checks or liking
  the design proves a learner can build the application.

## Map of the artifact

### Masthead and orientation

Kicker,
a one-paragraph situation (a parcel,
a Saturday delivery question,
a shop support
chat),
the starting profile,
the destination,
a chapter nav (12 links),
a sitting nav
(6 links),
a whole-lesson estimate computed at load,
a usage callout,
and then the shop
preview as the first interactive artifact.
Orientation deliberately precedes the artifact.

### Six sittings over fourteen chapters

1.  Foundations:
    `functions` (define,
    call,
    return,
    write to the DOM),
    `wiring` (registering a
    click listener versus observing a click),
    `data` (bindings,
    objects,
    arrays,
    conditions,
    `null` and `Error`).
2.  Why a returned string is not enough:
    `delayed` (a timer callback finishing after the caller
    returned;
    thread,
    event loop,
    call stack,
    closure),
    `callbacks` (completion callbacks,
    out-of-order batch collection,
    a repaired callback version presented as valid).
3.  Promises:
    `promise` (constructor,
    resolve and reject,
    `.then`,
    `Promise.withResolvers`,
    `Promise.all`,
    the manual settlement toy,
    value representations),
    `adoption`
    (resolved versus fulfilled),
    `await` (suspension versus blocking,
    tasks versus microtasks,
    the live trace exercise).
4.  Failure and control:
    `provided-api` (the fixture contract),
    `failure` (rejection,
    validation of a fulfilled-but-unusable value,
    unhandled rejections),
    `control`
    (`wait`,
    retries,
    backoff,
    jitter,
    the shared start gate,
    cancellation,
    deadlines,
    `Promise.race` limits,
    idempotency caveats).
5.  Separate conversations:
    `threads` (per-conversation history and per-send controllers,
    captured destinations,
    the reference chat and its scenario list).
6.  Build and explain:
    `workshop` (12 exercises in an isolated frame),
    `interview`
    (a demonstration script,
    five answer-before-reveal questions,
    the critique notes field).

Every chapter opens with an answer-first `The point:` paragraph placed before its first code
or experiment.
Every sitting header states its chapters,
who needs it,
what the learner can do
afterwards,
and a derived duration;
every sitting ends with a stop marker naming the next one.

Derived numbers on the current build,
computed in the browser from rendered text:
sitting 1
2,767 words and 3 hands-on blocks (25 to 35 minutes);
sitting 2 2,000 and 3 (20 to 30);
sitting 3 3,640 and 5 (35 to 45);
sitting 4 2,982 and 1 (15 to 25);
sitting 5 2,435 and 1
(15 to 25);
sitting 6 4,506 and 1 (25 to 35).
Total 18,330 words,
14 blocks,
145 to 155 minutes.

### Ten in-page labs

`goal` (the shop preview),
`function`,
`listener`,
`data`,
`delayed`,
`callback-one`,
`callback-batch` (with a worked counted repair),
`constructor`,
`promise-batch`,
`adoption`.
Each has an editable source textarea,
Run,
Reset,
Download,
an isolated preview frame,
a
teaching monitor,
and a static `.paper` counterpart for print.
Labs whose examples write no
page text say so instead of showing a stale result panel.

### Non-lab exercises

A rejection prediction quiz with both answers giving feedback,
the live `await` trace with an
independent-click button,
the manual settlement toy,
five interview questions behind
disclosures,
and the critique notes field with a local download.

### Workshop (12 exercises)

`greeting`,
`await`,
`waiting`,
`failure`,
`retry`,
`cancel`,
`deadline`,
`gate`,
`validation`,
`threads`,
`ownership`,
`capstone`.
Each has a task statement,
a hint disclosure,
and a worked
comparison disclosure.
Learner code runs in an opaque-origin,
network-disabled frame with its
own diagnostics (`JavaScript error:` and `Unhandled rejection:` lines) and an event record.
Starters for `await`,
`failure`,
`threads`,
and `ownership` are deliberately broken.
The capstone lists eight independent-build checkpoints and a transfer test.

### Reference chat

Seven scenarios (`success`,
`recover`,
`limited`,
`denied`,
`malformed`,
`timeout`,
`random`),
per-conversation Send and `Stop pending sends`,
plus lesson controls:
`Add conversation`,
`Reset and send example messages`,
`Reset and stop A during backoff`,
`Choose a fresh seed`,
and `Stop all and reset experiment`.
A shared event record names attempts,
gate waits,
backoff,
and cancellation.
Stated teaching limits:
4 conversations,
20 sends each,
200 event lines.

### Print and downloads

Print hides `nav`,
`button`,
`iframe`,
and `.controls`;
it shows every `.paper` counterpart and
an appendix holding each lab's starter,
worked comparison,
current draft,
and observed preview
output,
plus every workshop task,
starter,
hint,
and comparison and the current learner draft.
Disclosures expand on `beforeprint` and restore on `afterprint`.
Downloads produce standalone
offline files:
any lab example,
the learner's current workshop code,
the reference chat,
and
the critique notes.

## Design decisions and the failure each answers

- Shop opening rather than an abstract definition:
  an earlier opening was rejected as
  "extremely confusing and detached from any kind of real world example".
- Sending stays available while replies wait:
  forced turn-taking was rejected as not how chat
  works;
  each send owns its controller and captured destination.
- Achromatic long-form surfaces:
  colored reading backgrounds were rejected;
  only bounded state
  cues carry hue.
- Manual settlement keeps `"Hello Ada"` and `Error("No reply")` and is labelled a manual
  experiment,
  not a service request:
  calling it a service request taught a wrong model.
- Yum-Bot supplies artwork only,
  with an explicit disclaimer:
  a game-faithful
  pudding-as-Promise mapping was audited and retracted because the game's cues,
  conveyor,
  laser disposal,
  and face grading contradict Promise semantics.
- Five numbered one-shot controls per row,
  and later calls execute and are logged as ignored:
  the user ruled that an unused control still does something when it calls the resolver.
- Observed-outcome receipts are written by the observer,
  not by the next Create:
  the delayed
  version implied that Create caused the previous outcome.
- Lab result panels describe what that example actually writes:
  six frames previously showed
  `Program result: No greeting yet.` beside a monitor full of completions.
- Orientation precedes the first artifact:
  profile,
  destination,
  and both navs were previously
  about three viewports below a 1,267 px shop preview.
- Review scaffolding removed from the learner surface:
  `Tell me in Pi` and
  `revised teaching prototype` shipped to learners and into print.
- Pacing and takeaways added,
  with durations derived from rendered word and block counts:
  the measured complaint was an unannounced 23,598-word load with no stated time or short path.
- Audience labels per sitting rather than splitting the file:
  splitting would break the
  confirmed single self-contained artifact requirement.
- Broken starters with visible failure:
  a correct reference alone does not teach diagnosis.
- The `waiting` exercise exists because two different throwaway programs produced identical
  `Waiting…` text with an empty error record,
  one unresolved and one resolved-but-unrendered.
- Learner code runs in an opaque frame with `connect-src 'none'`:
  isolation was required and
  is asserted rather than assumed.

## What the verification proves, and what it cannot

Proves,
by executed checks:
control behavior and visible state in Chromium and Firefox ESR
140.16.0;
that resolving functions are really called (a native-call ledger,
with a
suppressed-call mutant failing the guard);
receipt timing,
ordering,
multiplicity,
and the
final Promise;
print disclosure expansion and restoration across two cycles;
that every
inventoried teaching passage and source block appears in extracted PDF text;
that downloads
reopen and run offline behind a proxy whose HTTP denial was positively tested;
forced-colors
legibility;
390 px width without horizontal overflow;
and zero page errors in each session.

Cannot prove:
that any learner understands,
retains,
or can build the application;
that the
derived duration estimates match a real person's pace (they come from word and block counts
at an assumed 200 words per minute and 4 minutes per hands-on block);
that the takeaways are
pedagogically well-chosen (the guard checks presence,
placement,
and length,
not accuracy);
or that the artwork reads as bounded to a first-time viewer.

## Known weak spots: start here

1.  The capstone's worked comparison is 8,668 characters that the learner pastes into the same
    editor holding their own attempt.
    There is deliberately no "load comparison" button,
    on the
    theory that friction between trying and comparing is desirable.
    That is a judgment,
    and it
    is the one I would most expect a critic to overturn.
2.  The settlement scene's correctness depends on its disclaimer paragraph.
    Robot,
    belt,
    and
    ticket vocabulary invites the physical reading the disclaimer denies.
3.  The shop's replies are revealed by a lesson control.
    It is labelled teaching equipment and
    the prose says real conversations differ,
    but it is still the first thing a learner touches.
4.  Sitting 4 carries 2,982 words with one hands-on block:
    policy-heavy prose about gates,
    backoff,
    jitter,
    timeouts,
    and idempotency with little to do.
5.  Takeaways are hand-written.
    A chapter can change and its takeaway silently drift;
    the guard
    would still pass.
6.  Both pacing constants are assumptions presented as estimates.
    If they are wrong,
    every
    number on the page is wrong in the same direction.
7.  Two audiences share one path.
    Labels mark interview depth,
    but nothing stops the linear
    order from being slow for one reader and steep for the other.
8.  Print is 133 pages.
    Completeness was required,
    but a critic may judge that unusable on paper.
9.  The `waiting` exercise is new and has exactly one pointer into it,
    from the `await` chapter.
10. The fixture's numbers (3 attempts,
    1,000 ms gate,
    1,800 ms deadline,
    250 ms jitter span,
    500 ms base backoff) are teaching choices.
    The lesson says a real service sets its own,
    but
    the specificity may still read as a recommendation.
11. The derived estimates count hands-on blocks as `.lab` and `.exercise` elements,
    so the
    workshop's 12 exercises count as one block and sitting 6's 25 to 35 minutes almost
    certainly understates the time an independent build takes.
    The formula is honest about its
    inputs but the inputs are incomplete here.

## Reproducible sequences, with what they actually produced

Every sequence below was executed against the current build in a disposable owned session
(`mise run review:critic-tour`,
driver `review-critic-tour.mjs`),
at a 944 by 741 CSS px
dark viewport unless stated.
Page errors were empty at the end of the run.
Quoted strings are
the observed DOM text,
trimmed.

**S1 Orientation.**
Load the page.
Observed:
kicker `A browser-programming workshop`;
`Starting profile` at y 390 inside the
first 741 px viewport;
chapter nav at 629;
sitting nav at 861;
whole-lesson line
`Whole lesson: estimated 145 to 155 minutes across 6 sittings (18,330 words, 14 hands-on blocks)`;
shop preview begins at y 1566;
6 sitting headers and 14 takeaways present.

**S2 The shop does not make you wait.**
Send the prefilled Saturday question;
type
`What does delivery cost?` and send it;
type `An unsent draft` without sending;
click
`Show next simulated reply`.
Observed history:
`You Do you deliver on Saturday?`,
`You What does delivery cost?`,
`Shop Yes. This practice shop offers Saturday delivery.`;
the composer still holds
`An unsent draft`;
`Simulated replies waiting: 1`;
Send is not disabled.

**S3 Define,
call,
and write are three operations.**
In the `functions` lab,
Run as given,
then remove the `// ` markers and Run again.
Observed:
first run leaves `No greeting yet.` with an empty monitor;
after uncommenting,
the
page reads `Hello Ada` and the monitor holds `Function body ran with: "Ada"` and
`Returned string stored in message: "Hello Ada"`.

**S4 What removing a listener stops.**
In the `listener` lab,
Run,
click the greeting button,
then `Attach listener`,
greet,
then `Remove listener`,
change the name to Grace,
greet.
Observed:
`No greeting yet.` with `No greeting listener attached`;
then `Hello Ada`;
after
removal the result stays `Hello Ada` while the connection line reads
`No greeting listener attached`.
Removal does not fake an undo.

**S5 A batch rule that looks right.**
In `callback-batch`,
Run with `count = 4`,
then load the
worked comparison.
Observed starter monitor:
`One operation completed with index: 3`,
then
`Report says the batch is complete; actual finished count: 1` with
`Replies collected so far in start order: [<empty slot>, <empty slot>, <empty slot>, "Reply 3"]`,
then indexes 2,
1, 0.
Observed comparison tail:
`Batch complete; successful reply count: 4`
and `Every reply in start order: ["Reply 0", "Reply 1", "Reply 2", "Reply 3"]`.

**S6 Construction,
return,
observation.**
In the `constructor` lab,
Run.
Observed monitor:
`Constructor calls start before returning: "Timer is registered here"`,
`Initiating call returned this value: Promise object; no public state field`,
`The script reaches its end before the timer callback: "End of setup"`,
`Fulfillment observer receives: {"text":"Hello Ada"}`;
page result `Hello Ada`.
Observed snapshots:
the returned Promise is `{}` in JSON and `Promise { <state unknown> }` in
Showify both before and after fulfillment,
while the observer's value is
`{"text":"Hello Ada"}` / `{ text: "Hello Ada" }`.

**S7 Settlement,
ignored calls,
receipts.**
In the settlement toy,
click Create 1,
Reject 1,
then Resolve 1,
then Create 2.
Observed:
face `Promise 1: observed rejected with Error(“No reply”) · fixed` with ticket `01`
and one receipt `Promise 1 · observed rejected Error(“No reply”)` present immediately;
Resolve 1 gives `Ignored: Resolve control 1 called; Promise 1 keeps its first outcome.` with
the receipt count still 1 and the ticket still `01`;
Create 2 shows ticket `02`,
face
`Promise 2: pending from creation; no outcome observed`,
and the earlier receipt retained once.

**S8 Resolved is not fulfilled.**
In the `adoption` lab,
Run,
click `Fulfill inner`;
Run again
and click `Reject inner`.
Observed:
`Outer resolved with inner: "Neither has an outcome yet"`,
then
`Outer fulfilled with: "The inner reply"` in the first run and
`Outer rejected with: "The inner failure"` in the second.

**S9 Suspension is not blocking.**
Click `Run the actual trace`,
then the independent button,
then `Fulfill the awaited Promise`.
Observed:
`A: before the call`,
`B: inside receive`,
`C: after the call`;
the independent
counter reads `Independent clicks: 1` while the invocation is suspended;
after fulfillment the
trace adds `D: received Hello Ada`.

**S10 An unhandled rejection leaves the UI lying.**
In the workshop,
load `failure`,
Run,
click Send inside the preview;
then load its worked comparison and repeat.
Observed starter:
result `Waiting…` with
`Unhandled rejection: Permission denied. Retrying unchanged inputs will not repair this.`
Observed comparison:
`Could not reply: Permission denied. Retrying unchanged inputs will not
repair this.` and an empty error record.

**S11 The new diagnosis exercise.**
Load `waiting`,
Run,
click Send;
then load its comparison.
Observed starter:
result `Waiting…`,
record `operation started` / `no observer registered`,
no errors,
and a snapshot labelled `The Promise the call returned` showing
`Promise { <state unknown> }`.
Positive control:
calling the fixture directly returned
`Simulated reply to “control”` while the page still read `Waiting…`,
so the silence belongs to
the missing observer.
Observed comparison:
result `Simulated reply to “Hello Ada”` with the
record `operation started` / `operation settled` / `page updated`.

**S12 Ownership of overlapping sends.**
Load `ownership`,
Run,
click the example pair,
then
the oldest send's completion control;
repeat with the worked comparison.
Observed starter:
`Pending sends: 0` with Stop disabled,
although a reply is still held.
Observed comparison:
`Pending sends: 1` with Stop enabled.

**S13 Real backoff and scoped cancellation.**
In the reference chat choose
`Fail twice, then succeed` and Send;
then click `Reset and stop A during backoff`.
Observed first run:
attempt 1 rejected `Service temporarily unavailable.`,
`Backoff: wait 500 ms before a NEW attempt`,
attempt 2 started with `gap 1700.7 ms ✓ spacing
met` and rejected,
`Backoff: wait 1000 ms`,
attempt 3 started with `gap 2200.4 ms` and
fulfilled,
then `Send fulfilled`.
Observed second run:
A's send 1 is rejected,
logs
`Lesson control activates Stop after the backoff timer was started`,
then both A sends log
`Cancelled: Stopped by the learner.`,
while Thread B's send 3 starts at `gap 1003.4 ms` and
fulfils.
No quick click is required.

**S14 Print.**
Click `Print the complete lesson`,
or print from the browser.
Observed by the guards:
all 15 disclosures expand on `beforeprint` and return to their prior
state on `afterprint`,
including across two consecutive print cycles;
the produced PDF holds
133 Letter pages with 894 teaching and 70 appendix inventory entries,
each matched against
extracted text;
`nav`,
`button`,
`iframe`,
and `.controls` do not print,
while every `.paper`
counterpart and the appendix do.

**S15 Downloads.**
Use any lab's `Download this example`,
the workshop's
`Download your current HTML`,
`Download reference chat HTML`,
or `Download these notes`.
Observed by `test:exports`:
each file reopens and runs offline behind a proxy whose HTTP
denial was positively tested first;
the reference download accepts overlapping sends,
preserves drafts,
and cancels during backoff;
an adversarial draft
(`</ScRiPt><img src=x onerror=alert(1)> & "quoted" 🟢`) survives round-trip as literal text
with no injected markup and no page errors.

**S16 Narrow width and forced colors.**
Resize to 390 px wide;
then enable forced-colors.
Observed:
`document.documentElement.scrollWidth` equals 390 with no horizontal overflow;
with `forced-colors: active` the Promise ticket remains `visible` and the observer status
renders `rgb(255, 255, 255)` rather than transparent.

## What kind of finding is useful

- Name the sitting,
  chapter,
  lab,
  or exercise,
  and quote the phrase or control.
- Say what a novice would conclude from it,
  and what is actually true.
- Classify it:
  wrong causal model,
  missing prerequisite,
  misleading affordance,
  unusable
  action,
  print loss,
  unverifiable claim,
  or wasted effort.
- A finding that contradicts a measured observation above is welcome;
  say which sequence to
  re-run and what you saw instead.

Out of scope,
by prior decision:
demanding a learner study as an approval gate (vetoed),
editing `AGENTS.md`,
connecting a real AI provider,
and restoring game-faithful Yum-Bot
mechanics.

## Decisions I made without asking, veto invited

- Audience labels inside one file instead of splitting the lesson by reader.
- Adding the `waiting` exercise,
  and pointing at it from the `await` chapter.
- Deriving every duration from the rendered page instead of authoring estimates.
- No load button for the capstone comparison,
  keeping the paste friction.
- Moving the presenter's default landing from the settlement section to the document top.
- Repairing all three measured defects in one increment rather than staging them.

# Independent review of the Promise teaching lesson

Reviewed artifact:
`doc/planning/promises-teaching.local.html`,
SHA-256 `aef8e02094efce04096989c4e102c7416edf342e2ff0affd81b8f72c2f478e3b`,
422,859 bytes,
2,476 lines.
Review date:
2026-09-08.
Reviewer:
Claude Fable 5.1,
without access to the authoring conversation.
The artifact was examined,
exercised,
and printed before the planning records were read.
Nothing in the lesson,
the authoring directory,
or `AGENTS.md` was changed.

## Overall assessment

### Semantic correctness

The Promise semantics hold up.
Every claim I could probe was confirmed at runtime,
in headless Chromium 149 and in Firefox ESR 140.15:
construction runs the executor before returning,
`then` registration does not invoke observers,
settlement is fixed after the first resolving call,
resolving with a pending Promise adopts its later fulfillment or rejection,
`await` on an already-fulfilled Promise still suspends until a microtask checkpoint,
`Promise.all` orders results by input and rejects on the first rejection without cancelling the rest,
`AbortSignal.any` reports a `TimeoutError` reason distinct from a user abort,
and the shared gate spaces starts by the configured interval even when retries compete.
Retries,
limits,
cancellation,
and turn-taking are consistently attributed to application code,
with one exception recorded as finding 2.
I found no misleading explanation of what a Promise is or does.

### Teachability

The chapter sequence is sound for the stated profile through the callbacks and Promise chapters.
Every prerequisite the explanations use in those chapters is introduced first,
the callback batch demonstrates a real limitation before Promises are named,
and the experiments execute the shown source rather than animating a claim.
The weaknesses are concentrated after the `#control` chapter:
the reference application and the capstone rely on syntax and data-structure idioms the lesson never introduces,
and two isolated exercises depend on a click race that the lesson elsewhere promises to avoid.

### Independent-build preparation

This is the weakest area.
The workshop exercises each isolate one policy against a single-operation page,
and the cancellation exercise deliberately teaches a single controller slot with Send disabled.
The capstone then requires per-conversation lists of pending controllers,
removal of one controller on completion,
and Stop over a list,
but no exercise has the learner write that structure,
and the only demonstration of it uses `filter` with an arrow function and `for...of`,
none of which the lesson teaches.
A learner who follows the lesson honestly reaches the capstone with the hardest new structure unpracticed;
the likeliest outcomes are copying the reference or building a single-slot Stop that fails checkpoints 5 and 7.

### Browser and print usability

Good.
Every interactive element I drove behaved as its label and the surrounding text describe.
Reading surfaces are achromatic in both themes,
the page has no horizontal overflow at 390 CSS pixels,
all controls except inline links meet a 44 pixel minimum,
the PDF is complete with an appendix that carries every starter,
solution,
task,
and hint,
and disclosure state is restored after PDF generation.
Exported files run standalone.
The only usability observations are an unrunnable capstone comparison that produces a confusing error
(finding 5) and two exercises whose intended Stop path needs a reaction within about 1.6 to 1.8 seconds
(finding 4).

## Findings ranked by consequence for this learner

### 1. No exercise bridges single-slot cancellation to the per-send controller list the capstone requires

Location:

- `#workshop` checkpoints 2,
  5,
  and 7 (lines 886 to 891):
  "Do not use one shared controller slot for overlapping sends" and
  "share one controller across a send's gate,
  attempts,
  and waits".
- `#threads` paragraph at line 825:
  "On completion,
  `filter(active => active !== controller)` creates a list retaining every other controller.
  Removing only the completed object prevents an older completion from clearing a newer send.
  Stop loops through the conversation's current list and aborts those controllers."
- `#starter-capstone` (lines 1796 to 1809):
  comments only.
- Capstone worked comparison is `#policy-runtime` (line 2345),
  which contains no controller list at all.
- The only cancellation exercise,
  "Wire Stop" (`#solution-cancel`,
  lines 1566 to 1596),
  uses one outer `controller` binding and disables Send while an operation is active.

Blocked learner action:
the learner must invent the list structure,
its removal idiom,
and the abort loop
using constructs they have not seen taught.
The prose introduces `for` with a counter and `while`,
never `for...of`;
`filter` and arrow functions appear once,
in the quoted sentence,
without explanation
(a search of the learner-facing prose,
lines 300 to 935,
finds zero occurrences of "arrow",
"for (const",
"destructur",
"class ",
or "instanceof").

Evidence:
source inspection plus the exercised workshop.
Every exercise runs as documented;
the gap is in what the sequence asks the learner to produce,
not in what the artifact does.
This is a pedagogical judgment that real learners would confirm or refute.

Repair direction:
add one exercise between "Wire Stop" and "Keep the destination",
for example "Two sends in one conversation:
Stop cancels both".
Its starter reuses the single-slot pattern with Send left enabled,
so that appending `sendFirst(); sendFirst();` shows Stop cancelling only the newer send.
Its solution keeps `controllers: []` on the conversation object,
pushes on send,
removes its own entry on completion using taught constructs
(a counter `for` loop with `indexOf` and `splice`,
or introduce `filter` and arrow functions
with two sentences in the `#data` chapter),
and aborts through a counter loop.
Check:
the new starter shows the defect with the documented paired calls,
the solution cancels both sends and leaves a later send untouched,
and the exercise appears in the print appendix through the existing `exercises` record.

### 2. The fixture's `attempt` input is presented as an API contract rather than lesson equipment

Location:
`#provided-api`,
line 637:
"The `attempt` property supplies the attempt number."
Also the workshop contract (line 865):
"Add `scenario: "recover"` to fail unless `attempt` is at least 3",
the `#control` retry helper,
the retry hint "Pass it into fakeReply",
and `attemptOnce` in `#policy-runtime`,
which forwards `attempt` to `fakeReply`.

Mistaken inference:
a real service accepts a client-declared attempt number and uses it to decide whether to succeed,
so a retry policy needs to report the attempt to the API.
The same chapter explicitly flags `retryable` and `retryAfterMs` as fixture-supplied
and `scenario` as a reproducibility control;
`attempt` receives no such flag,
and the retry code passes it as though the service needed it.

Evidence:
source only.
The fixture's `recover` outcome is keyed on `attempt < 3` (`#fixture-runtime`,
line 1150).

Repair direction:
in `#provided-api`,
state that `attempt`,
like `scenario`,
exists only so the local fixture can reproduce "fails twice,
then succeeds",
that a real request carries no attempt counter,
and that the `attempt` variable in the retry loop is the application's own bookkeeping.
Renaming the fixture input to `simulatedAttempt` would make the boundary visible in every code sample.
Check:
search the artifact and the PDF text for the added sentence,
and confirm the retry exercise hint no longer reads as an API requirement.

### 3. The opening source promises an explanation the lesson never delivers

Location:
`#goal-preview` disclosure summary,
line 311:
"Source for the shop preview,
explained through the function and callback chapters."
The `#goal-source` script (lines 1810 to 1897) uses `createElement`,
`append`,
`dataset`,
`shift`,
a destructured `{ speaker, text }` parameter,
`trim`,
`toLowerCase`,
`includes`,
`!reply` truthiness,
and `scrollTop`.
None of these operations is explained anywhere in the learner-facing prose
(zero prose occurrences of each between lines 300 and 935).

Blocked learner action:
the `#functions` chapter sends the learner into this disclosure to press Restart.
A learner who reads the source there meets roughly nine unexplained operations
with a promise that later chapters explain them;
the later chapters explain `querySelector`,
`textContent`,
`addEventListener`,
arrays,
and callbacks only.
For the independent build,
the only taught history-rendering technique is
`messages.join("\n")` into `textContent`,
while both the opening and the reference build DOM nodes.
The learner is not told which path is expected of them.

Evidence:
source inspection;
the chapter's static trace and the PDF text (page 2) confirm the wording.

Repair direction,
either of:

- Keep the source and make the summary honest:
  "The function and callback chapters explain its structure;
  the DOM operations it uses are listed here",
  followed by one line each for `createElement`,
  `append`,
  `shift`,
  `includes`,
  and `trim`.
- Rewrite `#goal-source` with taught operations only:
  an array of message strings rendered with `join("\n")`,
  and a counter-based removal instead of `shift`.

Check:
list every identifier in `#goal-source` and confirm each is defined by prose the learner has passed,
or by the disclosure itself.

### 4. Two isolated exercises require a click within the reply window

Location:
`#starter-cancel` (line 1553) uses `scenario: "slow"`,
a 1,600 ms reply;
the task text (line 2318) says "Stop is intentionally unwired.
Make it cancel a pending reply."
`#starter-deadline` and its task (line 2323):
"Run again and press Stop before it expires",
where the deadline is 1,800 ms.

Blocked learner action:
the intended observation ("Cancelled by you" or "Cancelled by you.
Do not retry.")
requires activating Stop within about 1.6 or 1.8 seconds of Send.
A slower reaction shows the success or timeout branch instead,
and the learner cannot tell whether their Stop wiring is wrong.
The lesson's own requirement elsewhere is that demonstrations be observable without a clicking race,
and the reference chat and shop preview satisfy it.

Evidence:
exercised in the browser.
With scripted clicks 100 to 200 ms apart the cancellation branch appears;
with the preview scrolled off screen,
Chromium delayed a 1,200 ms fixture timer to between 1.6 and 2.0 seconds,
which also shortens or lengthens the window unpredictably.

Repair direction:
use the hanging `timeout` scenario (60 seconds) for the cancellation exercise,
so Stop can be pressed at leisure,
and give the deadline exercise a longer deadline for the manual Stop run
(for example 6,000 ms) or a scenario selector inside the preview.
Check:
press Stop five seconds after Send and still observe the cancellation branch.

### 5. Running the capstone's worked comparison produces a misleading syntax error

Location:
`exercises.capstone.solution` is `'policy-runtime'` (line 2345).
`exerciseHtml` prefixes learner code with
`const { fakeReply, wait, seededRandom } = window.promiseLab;` (line 2384),
and `#policy-runtime` begins with `const { wait, fakeReply, ServiceFault } = window.promiseLab;`.

Reproducible interaction:
load "Independent build",
open "Compare with a worked solution after trying",
paste it into the editor,
press Run.
The preview reports `JavaScript error: Uncaught SyntaxError: Identifier 'wait' has already been declared`.

Mistaken inference:
the learner attributes the error to their environment or to the reference,
or concludes that the comparison is broken.
Every other exercise's comparison runs when pasted,
so the workshop has trained the learner to expect this to work.

Evidence:
observed in Chromium;
the cause is the duplicate declaration in the concatenated module.

Repair direction:
show a policy-only excerpt without the destructuring line as the capstone comparison,
or label the comparison as read-only reference source and disable Run for it,
or have `exerciseHtml` skip the access prefix when the code already destructures `window.promiseLab`.
Check:
paste and Run leaves `#preview-errors` empty.

### 6. The `wait` fixture throws synchronously for an already-aborted signal

Location:
`#fixture-runtime`,
line 1103,
`function wait({ milliseconds, signal = new AbortController().signal })` followed by `signal.throwIfAborted()`.
The workshop contract (line 866) says it "returns an abortable timer Promise".

Observed:
`wait({ milliseconds: 5, signal: abortedSignal })` throws an `AbortError` synchronously
instead of returning a rejected Promise,
while `fakeReply`,
which is `async`,
rejects.

Consequence:
low.
Every call in the lesson is `await wait(...)` inside an async function,
where the difference is invisible.
A learner who writes `const timer = wait(...)` before a `try` block would get an exception
the `#failure` chapter has taught them to expect as a rejection.

Repair direction:
make `wait` an `async` function,
or document the behavior.
Check:
the probe returns a rejected Promise instead of throwing.

### 7. The cancellation solution teaches a guard the exercise cannot exercise

Location:
`#control`,
line 743:
"This lets completion code avoid clearing a controller belonging to a different operation."
`#solution-cancel` disables Send during the operation,
so `controller !== operation` can never be true in that exercise.

This is an alternative teaching choice rather than a defect:
the guard becomes meaningful only once overlapping sends exist.
If finding 1's exercise is added,
move the explanation there and drop the guard from the single-slot solution.

### 8. Minor observations, no repair required

- The reference chat inputs are not inside a form,
  so Enter does not send.
  This matches "usable with native editing controls" but differs from common chat expectations.
  Alternative choice;
  observe with learners.
- The `#adoption` chapter precedes `#await`.
  Teaching "resolved is not fulfilled" before the learner has used `await` is defensible,
  but it is the densest distinction in the lesson at the point of least motivation.
  Alternative choice;
  observe with learners.
- Chromium throttles timers in the sandboxed previews when they are scrolled off screen.
  The lesson already states that timer delays are minimums,
  but a learner reading the text below a running retry will see events later than the stated milliseconds.

## What is working and should be preserved

- The opening is a meaningful activity.
  Two questions can be sent before either reply,
  an unfinished draft survives reply arrival,
  blank messages are refused without clearing the box,
  and the reply control is labeled and located as lesson equipment.
  Verified in the embedded preview,
  the exported standalone shop,
  and Firefox.
- The callback batch demonstrates a real limitation before Promises are named:
  the naive "last started completes last" rule succeeds at count one and fails at count four,
  the repaired counter is presented as valid,
  and the Promise version then removes the bookkeeping.
  Observed monitor output matches the printed static traces exactly.
- Experiments execute the displayed source.
  The monitor,
  the settlement badge,
  the await trace with independent clicks,
  and the adoption lab
  all report real runtime behavior,
  and the monitor prints a Promise as "Promise object;
  no public state field" rather than inventing a state.
- Fixture policy is separated from Promise semantics almost everywhere:
  `retryable`,
  `retryAfterMs`,
  one reply per send,
  the 4 conversation and 20 send limits,
  and the scenario selector are all named as fixture or demonstration choices.
- The reference chat behaves as every paragraph claims.
  Verified:
  overlapping sends in one conversation,
  Stop during backoff cancelling without a later attempt,
  Stop in one conversation leaving another's reply intact,
  Stop cancelling two pending sends at once,
  denied and malformed stopping after one attempt,
  the requested 1,500 ms delay honored,
  timeout exhausting three attempts,
  seeded random completing,
  and reset clearing everything.
- Reading surfaces are achromatic:
  measured `rgb(30, 30, 30)` and `rgb(41, 41, 41)` in dark,
  `rgb(244, 244, 244)`,
  `rgb(255, 255, 255)`,
  and `rgb(239, 239, 239)` in light.
  Accents remain on the primary button and the active state panel only.
- Print is a complete alternate presentation.
  My own PDF has 89 Letter pages,
  its text differs from the supplied `revised-lesson-print.pdf` only in session state,
  the appendix carries every lab source,
  comparison,
  latest preview output,
  workshop task,
  starter,
  hint,
  and comparison,
  and disclosure open states were identical before and after generation.
- Preview diagnostics surface unhandled rejections and syntax errors inside the preview,
  which makes the "Handle rejection" starter's failure visible without a console.
- Downloads reconstruct as self-contained files:
  the reference,
  a workshop file,
  and the shop all ran in a separate browser session with no page errors.

## Challenges to the planning records

Every historical complaint in `teaching-skill.md` is addressed in the current artifact:
the detached opening,
the send lock in the shop and in the reference chat,
undefined "chat",
"wait",
"fail",
"retry",
"stop",
"define",
and "display",
the browser-user click model,
the unexplained thread model,
"now",
objects before their use,
clickable-looking state cards,
`Promise.withResolvers` without predecessor,
resolved versus fulfilled without an example,
missing highlighting,
and chromatic backgrounds.
None of them is a current defect.

The records' provisional conclusions that I would qualify:

- "Independent advisor review identified a prerequisite gap before the capstone.
  The added gate,
  timeout,
  validation,
  and fixed-conversation exercises address that concern structurally."
  They address single-policy prerequisites.
  The controller-list structure that distinguishes the capstone from every exercise remains unpracticed
  (finding 1).
  The later correction that made the reference accept overlapping sends widened this gap
  rather than closing it,
  because it added a required structure without a teaching step.
- "Capstone controls passing is not evidence of independent learner mastery."
  Agreed,
  and the capstone comparison also cannot be run (finding 5),
  which the advanced driver does not check because it asserts only that the task text loads.
- The test suites encode the lesson's own policies faithfully.
  `verify-reference-sending.mjs` and `verify-shop-sending.mjs` now assert the corrected behavior,
  including a mutant that clears all controllers and is rejected.
  `verify-advanced.mjs` checks each workshop solution for an empty error record,
  but only the exercises it iterates;
  the capstone comparison is excluded.
  No assertion contradicts the lesson text.
  No assertion checks that a prerequisite is introduced before use;
  that remains a content-review question the suites cannot answer.
- `teaching-skill-acceptance.md` requires "account for the concepts and operations each explanation requires".
  Findings 1 and 3 are instances where the current artifact does not meet that proposed check,
  so the check is discriminating and worth keeping.
- Firefox ESR 140 was recorded as untested.
  I exercised it (see coverage);
  the recorded gap can be narrowed.

The authoring fragment `chat-ui.mjs` matches the delivered `#chat-runtime` byte for byte after trailing-space
normalization.

## Coverage statement

Artifact:
SHA-256 `aef8e02094efce04096989c4e102c7416edf342e2ff0affd81b8f72c2f478e3b`.
All work used a copy in the session scratchpad,
a fresh agent-browser session named
`promises-independent-review`,
a second session for exports,
and a fresh Firefox profile.
All three were closed or stopped at the end.

Sections read in full:
masthead and goal preview,
all 14 chapters,
footer,
both templates,
`#fixture-runtime`,
`#policy-runtime`,
`#chat-runtime`,
`#lesson-runtime`,
every `starter-*`,
`solution-*`,
and lab `*-source` script,
and the three bundled modules on the final line
(snapshot and lab runtime,
print appendix;
the Lezer highlighter was inspected only by its header).

Interactions exercised in headless Chromium 149 through agent-browser 0.36.0 and direct CDP frame evaluation:

- Shop preview:
  two sends before any reply,
  draft kept across two reveals,
  blank refusal,
  unsupported question,
  restart through the disclosure.
- Labs:
  function starter and solution;
  listener attach,
  greet,
  remove,
  greet;
  data at four entries;
  delayed at 1,200 ms and 0 ms;
  callback one;
  callback batch starter,
  repair,
  and repair with `failAt = 1`;
  constructor before and after fulfillment;
  Promise batch with and without failure;
  adoption with inner fulfillment and inner rejection.
- Settlement in four sequences;
  await trace with two independent clicks;
  both prediction buttons.
- Workshop:
  every starter and every worked comparison for all 10 exercises,
  the retry solution with `denied`,
  threads with the documented `sendFirst(); sendSecond();`,
  and the capstone comparison pasted and run.
- Reference chat:
  example messages,
  draft preservation,
  Stop during backoff,
  Stop isolation across conversations,
  Stop over two pending sends,
  denied,
  malformed,
  limited,
  timeout,
  seeded random,
  blank input,
  the 4 conversation limit,
  the 20 send limit,
  reset.
- Exports:
  reference,
  workshop,
  and shop files reconstructed through the page's own builders
  and run in a separate session.
- Presentation:
  computed colors in both themes,
  control sizes,
  1,280 and 390 pixel viewports,
  screenshots of the opening,
  chat,
  workshop,
  a lab,
  and the interview in dark and light.
- Print:
  `Page.printToPDF` from the copy,
  page inventory,
  text diff against the supplied PDF,
  disclosure restoration.

Interactions exercised in Firefox ESR 140.15 through WebDriver BiDi:
load with highlighting and print material ready,
settlement,
await trace,
example messages,
Stop during backoff,
the threads solution,
and print appendix population.

Runtime probes outside the lesson:
adoption,
await on a fulfilled Promise,
`Promise.all` ordering and
rejection,
`AbortSignal.any` reason,
gate spacing,
the threads starter misroute,
`wait` with an aborted
signal,
microtask before task.

Print pages visually inspected:
2,
34,
68,
and 81 of my generated PDF.
Pages 1,
29,
36,
52,
and 84 were rendered but not viewed.

Remaining gaps:

- No native print dialog was opened;
  the print path was verified through PDF generation only.
- Download buttons were not clicked;
  downloads were reconstructed by calling the page's builder functions.
- No forced-colors,
  screen reader,
  or physical keyboard traversal.
- Firefox coverage is a subset of the Chromium coverage.
- Preview timers were throttled while off screen in headless Chromium,
  so timing observations there are upper bounds,
  not measurements of the fixture.
- No learner was observed.

## Concerns that require real learner observation

- Whether a learner with the stated profile can produce the per-send controller list for the capstone
  after the current exercises (finding 1),
  or whether they copy the reference.
- Whether the callback batch's scaled failure produces the intended realization,
  or whether the learner accepts the repaired counter as sufficient and sees Promises as optional.
- Whether the volume is digestible:
  the page scrolls 44,407 CSS pixels and prints to 89 pages.
- Whether the adoption distinction placed before `await` is retained or skipped.
- Whether the transfer test (4 attempts,
  300,
  600,
  and 1,200 ms) actually distinguishes understanding
  from a working copy when the learner may keep the reference open.
- Whether "Wire Stop" and "Deadline or Stop" are completed without the click race in finding 4.

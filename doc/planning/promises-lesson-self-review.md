# Self-review of the Promise lesson artifact

Status:
the primary agent's own review of the generated local lesson,
`doc/planning/promises-teaching.local.html` (694,025 bytes,
SHA-256 `fb4631d8…`).
It records what was exercised,
what measured well,
what measured badly,
and what remains
uncertain.
It is not learner evidence,
not a skill decision,
and not approval.
A separate Pi session running Astra was started for an independent review,
produced no
output in 1 hour 47 minutes,
and the user cancelled it and took its troubleshooting
write-up.
Everything recorded here is the primary agent's own observation.
All browser work ran in disposable owned sessions (`promises-review-own-teaching-review-*`);
the human's presented tab was never reloaded.
Scratch drivers:
`/home/user/temp/agent/promises-revision/review-promise-lesson-own{,-2,-3,-4,-5}.mjs`.

## Good

- The opening shop demonstrates the property the lesson later needs:
  sending stays enabled
  while replies wait.
  Measured in the frame after two sends:
  history holds both questions,
  the composer keeps an unsent draft,
  `Simulated replies waiting: 2`,
  and `#send` is not
  disabled.
  The reply control is labelled lesson equipment and the prose states real
  conversations may reorder,
  combine,
  or omit replies.
- The listener lab lets the learner remove the connection and observe the consequence.
  Measured:
  before attach,
  `No greeting yet.` and `No greeting listener attached`;
  after attach,
  `Hello Ada`;
  after removal with a new name,
  the result stays `Hello Ada`
  and the connection line reports no listener.
  The removal does not fake an undo.
- The callback batch lab produces the demonstrated need for Promises rather than asserting it.
  Measured monitor tail after settling:
  `One operation completed with index: 1`,
  then `0`,
  with the starter's `finishBatch` claim contradicted by the actual completed count.
  The repair is presented as a valid callback solution,
  not as a strawman.
- Broken starters fail visibly and differently from their repairs.
  Measured in the ownership
  exercise:
  the starter ends at `Pending sends: 0` with `Held replies: 1` and Stop disabled,
  while the repair holds `Pending sends: 1` with Stop enabled.
  The rejection starter leaves
  `Waiting…` plus `Unhandled rejection: Permission denied…` and its task and hint name the
  repair;
  the worked comparison shows `Could not reply: Permission denied…` with no error.
- The settlement toy keeps its boundaries honest.
  Measured:
  the ticket stays `01` through
  settlement and an ignored call,
  the receipt appears when the observer reports,
  the ignored call adds no second receipt,
  and the artwork disclaimer states that no pudding
  is processed and that one-current-Promise plus one-shot controls are toy policies.
- Value views distinguish representation from state.
  Measured in the settled run:
  the bundle
  renders `{"promise":{}}` and the inner Promise renders `Promise { <state unknown> }`,
  with the lesson stating that showify's marker is not the pending state.
- Print keeps teaching substance and restores learner state.
  Measured:
  the host controller
  expands all 15 `details` on `beforeprint` and restores the prior 0 open on `afterprint`;
  a direct headless PDF (110 pages) carries the reference fixture,
  policy,
  and interface
  source headings,
  capstone checkpoints,
  interview questions,
  settlement receipts,
  and the
  MDN source list.
  Interactive control labels are absent from print by design,
  with static
  counterparts present.
- Independent practice is real,
  not nominal:
  11 workshop exercises,
  eight capstone
  checkpoints,
  and a transfer test that changes attempt counts and pause timings and asks
  which Promise represents one attempt versus the whole send.

## Bad

- Six of nine example frames show a panel headed `Program result` reading `No greeting yet.`
  after their program has already reported completion.
  Measured with a 2,000 ms settle:
  `callback-batch` (6 monitor entries),
  `promise-batch` (monitor ends
  `Every reply in start order: ["Reply 0", …]`),
  and `adoption`
  (monitor ends `Outer resolved with inner`) all keep `No greeting yet.`;
  `function`,
  `listener`,
  and `data` also keep it before their first interaction.
  Cause:
  `exampleHtml` in `/home/user/temp/agent/promises-revision/labs.mjs` emits
  `<h2>Program result</h2><p id="result">No greeting yet.</p>` for every non-shop lab,
  while those examples write only to the monitor.
  Why it matters:
  a novice reading `Program result: No greeting yet.` beside a monitor full
  of completions gets contradictory evidence about whether their program worked.
  Repair direction:
  label the panel per lab (`This example reports to the monitor`),
  or omit it when the example never assigns `#result`.
- Orientation arrives after the first large artifact.
  Measured at a 944 by 741 CSS px
  viewport:
  the shop lab spans y 402 to 1669 (1,267 px tall);
  `Starting profile` begins at
  1693;
  `Destination` at 1784;
  `What to do here` at 1940;
  the learning-path `nav` at 2279;
  the first chapter at 2527.
  A learner therefore scrolls about three viewports of shop
  before learning who the lesson is for,
  where it goes,
  or what the path is.
  Repair direction:
  move the profile,
  destination,
  and path above or beside the shop,
  or shrink the first artifact's initial height.
- Review-era scaffolding ships inside the learner artifact and its printout.
  Measured on
  screen and in the extracted PDF text:
  `This revision is for critique. Tell me in Pi where
  the explanation still assumes knowledge or teaches the wrong model.` and
  `There is no submission endpoint. Reply in Pi with your critique.` The masthead kicker
  reads `A browser-programming workshop · revised teaching prototype`.
  A learner who opens this file outside that chat has no `Pi` to reply in,
  and the sentence
  invites them to address a person who is not present.
  Repair direction:
  keep critique
  instructions generic (the notes field and download already work offline) and drop
  prototype framing from the learner's reading surface.

## Ugly

- Reading load is large and unannounced.
  Measured:
  23,598 words in `main`,
  14 chapters,
  39 `pre code` blocks,
  15 `details`,
  and 11 `iframe` previews.
  A direct headless PDF with
  disclosures closed measured 110 and 111 pages across two passes (the loaded workshop draft
  differs);
  the earlier verified print run,
  which expands disclosures through the lesson's own
  `beforeprint` path,
  measured 126 Letter pages.
  The masthead states no time expectation and offers no
  short path,
  so a learner cannot tell whether this is one sitting or several.
  The 12-link nav is the only structural aid.
- The capstone worked comparison is 8,668 characters that the learner is asked to paste into
  one editor and run.
  That is a legitimate way to inspect a complete policy plus interface,
  but its size makes the "try your own build first" instruction easy to skip and hard to
  compare against,
  since the editor holds both attempts.
- The settlement scene needs a full disclaimer paragraph to keep its artwork from implying a
  physical mapping.
  That paragraph is accurate,
  but a design whose correctness depends on a
  long caveat is fragile:
  the same robot,
  belt,
  and ticket vocabulary invites the reading the
  caveat denies.

## Things tried

- Opened the generated lesson in four disposable sessions at 944 by 741 (dark),
  read the
  chapter sequence,
  and enumerated labs,
  workshops,
  and disclosures from the DOM.
- Drove the shop frame:
  two sends before any reply,
  an unsent draft,
  one revealed reply;
  recorded history,
  draft,
  waiting count,
  and Send availability.
- Drove the listener frame through attach,
  greet,
  remove,
  rename,
  greet;
  recorded result and
  connection text at each step.
- Drove the settlement ticket:
  Create 1,
  Reject 1,
  waited for the `rejected` state,
  then a
  later Resolve 1;
  recorded face text,
  belt receipts,
  bundle JSON,
  and inner-Promise view.
- Loaded the `failure`,
  `ownership`,
  `threads`,
  and `capstone` workshop exercises,
  ran the
  starters and the worked comparisons in the opaque frame,
  and compared visible outcomes,
  task text,
  and hints.
- Settled every non-shop lab for 2,000 ms (longer than the lesson's largest 1,200 ms timer)
  and recorded `#result`,
  its heading,
  monitor entry count,
  and the last monitor lines.
- Dispatched `beforeprint` and `afterprint` to measure disclosure expansion and restoration;
  produced a direct headless PDF and searched its normalized text for source,
  checkpoint,
  hint,
  interview,
  receipt,
  and scaffolding strings.
- Checked page errors after each pass:
  empty in every session used here.

## Repairs applied after this review

The user chose “Repair everything now,
then re-ask”,
so all three measured defects were
repaired before the deferred structure questions.
Pre-repair artifact preserved as `doc/planning/promises-teaching-before-review-repairs.local.html`,
694,025 bytes,
SHA-256 `fb4631d86bd99bb532b142372e61aaaabe985a0d324cf61993f8d36900dc4875`.
Repaired artifact:
`doc/planning/promises-teaching.local.html`,
695,026 bytes,
SHA-256 `e0f770f8f31e3c0b6425c530c6f13415a284d272d2f48e3f0d8f2ee956d887f6`,
build identity `983a61d22da2dcab…`.

- **Result panel.**
  `labs.mjs` now classifies each lab from its authored sample plus worked
  comparison;
  a lab whose examples never write `#result` gets a `Page text` panel saying the
  example reports to the monitor.
  Measured after the rebuild with a 2,000 ms settle:
  `callback-batch`,
  `promise-batch`,
  and `adoption` show the honest text with 6,
  6,
  and 1
  monitor entries;
  `listener` still shows `Program result: No greeting yet.` and `constructor`
  still shows `Program result: Hello Ada`.
  The workshop's `Ready` panel is **not** a matching defect:
  the capstone's fixed panels are
  `#result` and `#result-b`,
  and its worked comparison writes `Pending sends: 0` and
  `Cancelled: Delivery cost?` there,
  so `Ready` is a true initial state.
- **Orientation.**
  Profile,
  destination,
  learning path,
  and the usage callout now precede the
  shop artifact.
  Measured at 944 by 741 CSS px after the rebuild:
  profile y 390 (inside the
  first viewport,
  was 1693),
  destination 481,
  nav 629,
  callout 869,
  shop lab 1209 (was 402),
  first chapter 2515.
- **Scaffolding.**
  The kicker is `A browser-programming workshop`;
  `Tell me in Pi`,
  `This revision is for critique`,
  and `Reply in Pi with your critique` are absent from both
  screen and extracted print text.
  The notes caption now says the download is a local file the
  learner can keep or share with whoever is teaching them,
  and the callout says the path links
  every chapter so the learner can stop and return.

Guard discipline:
`verify-review-repairs.mjs` (task `test:review-repairs`,
included in
`test:all`) was run red against the preserved pre-repair artifact first,
failing on the kicker,
the `Pi` instructions,
the prototype framing,
and the panel text;
it passed after the rebuild.
Task `probe:review-repairs-before` keeps that red run reproducible.
Its print assertions follow the lesson's own rule that `nav`,
`button`,
`iframe`,
and `.controls`
do not print,
so the panel text is asserted on screen while the `.paper` counterparts are
asserted in the PDF.

Verification after the repairs:
`mise run test:all` passed in 372 s,
including the new guard and
the ticket-boundary guards;
the print inventory held 126 Letter pages at 2,477,051 bytes with
793 teaching and 66 appendix entries;
`test:exports` passed against a proxy whose HTTP denial
was positively tested;
Firefox ESR 140.16.0 passed the value,
receipt,
pagehide,
chapter,
and
prototype checks.

Presentation:
the headed presenter opened the repaired build in a separate tab
(2 tabs before,
3 after;
build identity `983a61d22da2dcabcc0d7ae5a5932ebd35536ca527dc6b473ba55819da9e4ed4`)
and scrolled it to the top,
so the repaired kicker and `Starting profile` are the first things
visible.
The tab holding the pre-repair build was not reloaded or closed.
The presenter's default anchor moved from `#promise-settlement` to the document top so a
presentation lands on the orientation the lesson now puts first.
Owned fixtures were stopped and their profiles removed after verification.

Still open at that point,
and taken up in `Second increment: pacing, takeaways, audience labels, and a diagnosis exercise`:
the unannounced reading load and
the two unsettled judgment items about audience and `Waiting…`.
Still open now:
the 8,668-character capstone paste sharing one editor with the learner's own
attempt,
and the settlement scene's reliance on its disclaimer paragraph.

## Second increment: pacing, takeaways, audience labels, and a diagnosis exercise

The user chose paced sittings plus answer-first summaries (“A+B,
of course”) and objected
that the question itself was unnecessary,
because the two options are complements and the
stated ranking left one answer standing.
Both were implemented together.
Pre-increment artifact preserved as `doc/planning/promises-teaching-before-pacing.local.html`,
695,026 bytes,
SHA-256 `e0f770f8f31e3c0b6425c530c6f13415a284d272d2f48e3f0d8f2ee956d887f6`.
The pacing-only build,
before the new exercise,
is preserved as
`doc/planning/promises-teaching-before-waiting-exercise.local.html`,
SHA-256
`33914cd23e7ccb0879560ad8b6c734cdf5c8d7255a23b664b2364832c7915c9d`.

- **Paced sittings.**
  `chapter-orientations.mjs` groups the 14 chapters into six sittings,
  each with a header (chapters,
  capabilities gained,
  who needs it) and a stop marker naming
  the next sitting.
  Coverage is exact:
  every chapter appears once,
  in document order.
- **Derived durations.**
  `lesson-pacing.mjs` counts words with `Intl.Segmenter` and hands-on
  blocks in the rendered DOM at load,
  then prints a five-minute band from
  200 words per minute plus 4 minutes per block.
  No duration is authored by hand.
  Measured on the pacing build:
  18,250 words,
  14 hands-on blocks,
  145 to 155 minutes total;
  per sitting 25 to 35,
  20 to 30,
  35 to 45,
  15 to 25,
  15 to 25,
  and 25 to 35 minutes.
  The verifier recomputes the same formula from the page,
  so a hand-edited number fails.
- **Answer-first takeaways.**
  Every chapter opens with a `The point:` paragraph written from
  that chapter's own content,
  placed before its first code or experiment;
  the guard requires
  14 of them,
  each at least two sentences and 120 characters.
- **Audience labels.**
  Each sitting states who needs it,
  marking the rate gate,
  backoff,
  jitter,
  timeout,
  and ownership material as interview depth.
  This addresses the two-audience
  seam inside the single self-contained file rather than splitting it,
  which the confirmed
  presentation default rules out.
- **`Waiting…` diagnosis exercise.**
  The workshop gained `waiting`,
  placed after `await`:
  its starter starts the operation and lets the fixture reply while the page keeps showing
  `Waiting…`;
  the learner must decide which boundary is missing and repair it so the record
  names start,
  settlement,
  and render.
  The `await` chapter points at the exercise by name and
  states that the same page text can mean an unresolved Promise or a settled one nobody rendered.

Guard discipline for this increment:
`probe:lesson-pacing-before` failed against the preserved
pre-pacing artifact (no `window.lessonPacing`) and `test:lesson-pacing` passed after the rebuild;
`probe:waiting-exercise-before` failed against the preserved pacing-only artifact (the dropdown
had no `waiting` entry) and `test:waiting-exercise` passed after the rebuild.
The exercise guard includes a positive control:
`window.promiseLab.fakeReply` returned
`Simulated reply to “control”` while the page still read `Waiting…`,
which attributes the
silence to the missing observer rather than to a dead fixture.
The repaired version shows
`Simulated reply to “Hello Ada”` with all three record lines.

Two verifier expectations needed correction rather than exemption:
the print inventory's
workshop `Task` count rose from 11 to 12,
and an initial guess that `Starter source` items
would also rise was wrong,
because those ten items come from the in-page labs
(`window.lessonLabs.labs`),
not from workshop exercises.

Verification of this increment:
`mise run test:all` passed in 376 s with the owned offline and
native-print fixtures,
including the pacing,
waiting-exercise,
repair,
and ticket-boundary
guards.
`test:exports` passed against a proxy whose HTTP denial was positively tested.
Firefox ESR 140.16.0 passed the value,
receipt,
pagehide,
chapter,
capstone,
and prototype
checks.
The final artifact is `doc/planning/promises-teaching.local.html`,
718,990 bytes,
SHA-256 `270049535caaf4a86063fd9b3322b2f1be3c838f52566c9abcc77fd16941d818`,
build identity `895a0c508be2a32ffa21cc8c9eae48c9a170d91e09fe77c7633a62d245f47e13`.
Its print run measured 133 Letter pages at 2,589,705 bytes with 894 teaching and
70 appendix inventory entries,
all matched against extracted PDF text.
The presenter opened this build in a separate headed tab at the document top
(3 tabs before,
4 after) without reloading the tabs holding the two earlier builds.
Owned fixtures were stopped and their profiles removed afterwards.

## Unease (judgment, not measurement)

- The lesson may be teaching two audiences at once:
  a novice who needs the greeting,
  listener,
  and data chapters,
  and an interview candidate who needs rate limits,
  jitter,
  idempotency,
  and cancellation ownership.
  Both are in the confirmed destination,
  but the
  single linear path makes the early chapters feel slow for one reader and the later
  chapters feel steep for the other.
  I have no learner evidence for where the seam hurts.
  **Partly addressed:**
  each sitting now states who needs it and marks the policy material as
  interview depth,
  so a reader can skip forward inside one file.
  Whether labels are enough,
  or the seam still costs the novice something,
  remains unmeasured.
- The shop's manually revealed replies are labelled teaching equipment,
  yet the first
  interactive thing a learner touches is a chat whose timing they control.
  I am unsure
  whether that primes "replies arrive when I click" before the lesson says otherwise.
  **Still open:**
  no change was made,
  and I have no evidence either way.
- The `Waiting…` ambiguity I measured in disposable learner code (unresolved Promise versus
  resolved-but-unrendered) is a real novice trap.
  The lesson explains awaiting and rendering
  separately,
  but I did not find a place where the learner practices telling those two apart
  in their own code.
  **Addressed:**
  the `waiting` workshop exercise is that practice,
  and the
  `await` chapter names the ambiguity explicitly.
  Whether one exercise is enough practice is
  a judgment I cannot settle without a learner.
- Whether the lesson is the right size for this destination is a value judgment I cannot
  settle from the artifact.
  **Partly addressed:**
  the user chose paced sittings plus
  answer-first summaries,
  so the load is now stated per sitting (derived:
  145 to 155 minutes
  across six sittings) instead of being invisible.
  Whether that total is too long for the
  destination,
  or should be split into separate artifacts,
  remains open and depends on how
  the lesson is meant to be used.

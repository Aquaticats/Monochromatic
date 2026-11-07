# Candidate teaching-skill decision outline

Status:
planning-only process sketch,
not `SKILL.md` or permission to implement the final skill.
Its overall sequence and invocation remain proposed;
confirmed presentation requirements,
the refusal to require a learner study,
and the current lesson's manual fixture remain settled.
Every applicable branch obligation **and** its local completion criterion must hold.
Its examples and falsifiers live in [the working acceptance checks](teaching-skill-acceptance.md),
[the cross-domain probe](teaching-skill-transfer-probe.md),
and [the learner-result research note](teaching-skill-research-feedback.md).
This outline tests whether the same process could guide live teaching and authored material
without copying Promise-specific UI or unverified recipe details into unrelated topics.

## Invocation hypothesis

A future model-invoked pointer would fire when the agent is asked to teach a person,
explain material for a stated learner,
or create or review instructional material.
Those are distinct branches;
the pointer should not fire merely because an answer contains
an explanatory sentence.
A model-invoked description is a permanent context cost,
but relying on the user to remember a skill would miss ordinary teaching requests.
The trigger and its tradeoff remain a planning hypothesis,
not a package decision.

## Determine the authorized teaching mode

- Classify the request as live tutoring,
  authored teaching material,
  or a review of material.
  The same standards can inform each mode,
  but the requested verb controls the action.
  A review reports evidence and recommendations;
  it does not silently rewrite the lesson.
- For live teaching or authoring,
  establish what the user actually requested before
  building a new artifact or changing an existing one.
  A live turn within an authored
  lesson does not silently opt out of the user's confirmed HTML default or print parity.
- **Completion criterion:**
  the mode,
  deliverable,
  and mutation authority are explicit;
  no review has been treated as an implementation request.

## Decide what the learner must accomplish

- Inspect discoverable context,
  then identify the learner's evidenced capabilities,
  immediate task,
  and any separately desired transferable background.
  A worked example of a capability is not a biography of every learner.
- Form a reasoned interpretation before asking.
  Clarify an ambiguity only if plausible readings change the method,
  equipment,
  audience,
  or explanatory goal.
  A demonstration-only fixture belongs to the teacher;
  an actual integration contract belongs to the requested real task.
  Research discoverable contract facts instead of asking a novice to specify them.
- Before asking,
  name a goal-critical unresolved decision and show the concrete paths
  selected by its possible answers.
  Researching first does not redeem an open-ended
  “what failure did I miss?”
  question or a teacher-owned fixture choice.
- **Completion criterion:**
  the target and capabilities select the next teachable action;
  every question names a goal-critical fork that evidence cannot settle for the user.

The “rotated chicken” request tests target ambiguity.
The rejected AI-service setup question tests fixture ownership.
The earlier broad “what failure did I miss?”
question tests whether the teacher did its own
research before asking the user to diagnose a defect.

## Decide which causal relation to teach next

- Identify the operation or inference the learner must construct,
  perform,
  or explain.
  Teach prerequisites before depending on them and use the causal model for that role.
- When a new abstraction answers a predecessor method's limit,
  establish that method
  first if the learner does not know it.
  Demonstrate the same concrete problem's limit,
  then introduce the abstraction and its reference material in response.
  Showing the old method after the new one does not reverse that order.
  This is not a demand for a failure experience before every fact.
- Keep an operation separate from its representation and from a fixture policy.
  Verify source-backed claims when analogies,
  runtime ordering,
  or external tools matter.
- **Completion criterion:**
  prerequisites appear before use.
  If this path needs a
  predecessor,
  its concrete limit is visible **before** the new abstraction or table.
  The learner can explain how they register a handler,
  start work,
  observe its result,
  and handle it where those are developer-owned actions,
  not merely repeat those words.

The apples-to-multiplication progression and callback-to-Promise comparison test the
abstraction branch.
Listener registration tests the developer-role branch.
The manual resolver ticket tests the operation-versus-observation boundary.

## Decide how the learner can act

- For each consequential step,
  supply verified quantities,
  units,
  objects,
  tools,
  access paths,
  and ownership needed by that learner.
  Match a delivery product to
  actual availability or open an in-person map when that is the promised route.
  A genuine link alone does not prove usable access.
  Do not fill missing dish,
  location,
  equipment,
  or safety facts with guesses.
- Carry forward applicable user requirements and accepted decisions before choosing
  the demonstration contract.
  Verify its semantics independently of its labels;
  a self-consistent replacement is not parity with the requested operation.
  Controls must perform their advertised native action;
  an event record must update at the event it claims to represent.
- **Completion criterion:**
  consequential uncertainty is resolved before the action;
  the learner can then attempt the next step with available resources **and**
  the promised access action,
  safety check,
  and underlying effect are verified
  wherever each applies.

The quoted salt,
oven,
and grocery instructions test actionability;
the Promise Send,
Stop,
and delayed belt cases test operation and event boundaries.
A receipt label alone does not prove native resolver calls or correct observer timing.

## Decide what the learner performs independently

- Choose a task requiring the learner to construct or adapt a result that exercises the
  destination's consequential responsibilities,
  and explain the choices involved.
  Separate intermediate practice from an independent attempt at the destination.
  Provide taught prerequisites,
  then withdraw supplied answers or scaffolding;
  keep the worked comparison accessible afterward.
- **Completion criterion:**
  the lesson offers a learner-owned attempt and explanation
  of the destination capability,
  not just a cosmetic edit or a supplied solution.
  Offering that opportunity does not claim the learner performed it.

## Decide how the learner checks their own result

- At a consequential action or inference,
  provide an observable success or safety
  criterion using an available,
  appropriate check.
  A reference result or a label alone does not measure the learner's own attempt.
- When their observation differs,
  say what it establishes and what remains unknown.
  Name how this learner obtains the next signal,
  how to interpret it,
  and how to act
  or recheck.
  A safe stop or request for help is valid when evidence is inconclusive.
  Do not infer a Promise's internal state from `Waiting…`,
  or food safety from appearance.
- **Completion criterion:**
  the learner can perform the check with available means
  and has a bounded next decision for correct,
  different,
  or inconclusive results.
  Pending and rejection are not automatically mistakes without an expected outcome.

This is a proposed specificity refinement,
not a universal checkpoint after every
sentence or a claim that the current Promise lesson lacks rejection feedback.
The [local disposable probe](teaching-skill-research-feedback.md) showed existing task,
hint,
and diagnostic support,
plus distinct underlying operations behind the same display.

## Decide how to present and preserve the teaching

- For authored lessons,
  follow the user's confirmed default of a visual,
  interactive,
  self-contained HTML presentation with automatic theme and complete print substance.
  Offer the requested quick Markdown alternative when appropriate.
- Preserve useful browser actions and provide usable static counterparts for print;
  neither medium justifies stripping depth from the other.
  For a live exchange,
  establish the next learner action in the requested channel;
  do not use that mode to waive an authored artifact the user requested.
- **Completion criterion:**
  every promised presentation preserves necessary code,
  examples,
  explanations,
  and current-versus-example labels.
  Where a printout claims to reproduce an event record,
  preserve each event's identity,
  multiplicity,
  and order.
  Verify browser behavior,
  printed content,
  print restoration,
  and continued interaction where applicable;
  text presence alone is not parity.

Code highlighting belongs to code teaching when useful,
not every recipe.
A printed Promise lab needs source,
trace,
and explanation even though its controls cannot run.

## Decide what the evidence warrants claiming

- Exercise teaching examples and artifact integrations through their actual consumer.
  Compare the authored semantic contract with independent runtime evidence,
  including native calls and event order where those claims matter.
  Check prerequisites,
  control effects,
  exports,
  accessibility,
  and print separately
  from learner independence.
- Report observed learner performance only if it occurred.
  A working reference,
  completed browser suite,
  or positive design reaction is not proof of learner mastery.
- **Completion criterion:**
  each claim has evidence sufficient for **that claim**;
  naming browser tests cannot support learner mastery.
  An absent learner observation
  is not turned into a study or approval gate.

## Design gate before a real skill

Use [the static replay ledger](teaching-skill-outline-replay.md) to test applicable
obligations and exit criteria against recorded counterexamples and near misses,
including a precisely worded but wrong cooking procedure,
the rejected API-specification
question,
a working Promise reference with an unhandled learner error,
and a result display that hides an already-settled Promise.
If a branch cannot distinguish those cases,
revise the sketch rather than adding
an all-purpose prohibition.
Confirm shared understanding before creating a real skill.
The user's instruction to leave `AGENTS.md` unchanged remains in force.

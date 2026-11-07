# Teaching-skill transfer probe

Status:
working comparison,
not a confirmed skill design or a new lesson request.
The user's positive response to the corrected Promise toy is feedback on that artifact,
not evidence that a novice has learned from it.
The cooking evidence is the user's critique recorded in [teaching-skill discovery](teaching-skill.md).
The Promise evidence includes the same discovery record,
[proposed acceptance checks](teaching-skill-acceptance.md),
and [the settlement-UX handover](../handover/promises-settlement-ux.md).
No cooking instructions,
product links,
or safety claims are being prescribed by this probe.

## Evidence and transfer candidates

### Resolve the intended target before teaching a procedure

- **Cooking observation:**
  The learner asked about “rotated chicken”.
  The user said this was not the dish name and regarded the wording as a sign of missing
  cooking knowledge.
  The recorded teacher failure is proceeding without resolving the term;
  the intended dish and equipment cannot be determined from the phrase alone.
- **Promise observation:**
  Early lesson openings named Promises and chat machinery before establishing
  what the learner was trying to build or why the abstraction mattered.
  The Yum-Bot physical metaphor later mislabeled the causal role of the Promise.
- **Transfer candidate:**
  Form a reasoned interpretation of the target,
  then clarify consequential ambiguity instead of building steps around an unexamined name.
  Connect the resulting teaching path to the learner's actual destination.
- **Limit:**
  This is not an instruction to demand confirmation of every term.
  Ask a target-disambiguation question when plausible interpretations change the procedure.
  This does not replace the user's separate requirement to confirm shared understanding
  before a final skill design;
  that checkpoint has not been reached.

### Make each learner action usable at its own boundary

- **Cooking observation:**
  “A little bit of salt” gave no usable calibration;
  “300degrees” omitted a temperature scale;
  “Get raw chicken from grocery store” provided no actual procurement path.
  The user wanted a specific delivery product or an opened map for an in-person trip.
- **Promise observation:**
  Undefined prerequisites,
  a missing listener-registration relationship,
  ambiguous Stop ownership,
  and a reference send control that waited for a reply
  blocked actions the learner was meant to perform or explain.
- **Transfer candidate:**
  At an action boundary,
  supply the quantities,
  units,
  objects,
  ownership,
  tools,
  and accessible path that this learner needs to carry out the step.
  Measure discoverable environment facts before asking the learner to provide them.
- **Limit:**
  No oven setting,
  ingredient amount,
  product,
  vendor,
  or travel path is established here.
  A real cooking lesson would need a clarified dish,
  available equipment,
  location or delivery preference,
  and checked food-safety sources before naming those particulars.
  More detail is not inherently better if it specifies the wrong operation.

### Teach the causal model for the learner's role

- **Cooking observation:**
  A procurement command with no usable source assumes the novice can
  bridge from an ingredient name to acquiring the right item.
  An ambiguous oven instruction assumes the novice can infer the intended scale.
- **Promise observation:**
  “Click and something happens” explained a browser user's observation,
  not how the developer registers a listener.
  “A service request” was used for an experiment that waited for learner intervention,
  though its resolver button supplied only a manual outcome;
  a game-performance face was treated as a Promise outcome.
- **Transfer candidate:**
  Explain the decisions and causal links that the learner must perform,
  construct,
  or diagnose,
  not merely the effect an observer notices.
- **Limit:**
  The programming learner's event-listener and Promise internals do not belong in a recipe.
  Cooking instructions likewise require domain-specific safety and handling evidence
  rather than JavaScript-style state diagrams by default.

### Align a record with the event it claims to describe

- **Cooking observation:**
  The quoted critique concerns missing calibration and access,
  not the timing of a cooking-progress display.
  There is no user evidence that a recipe needed a simulated timeline.
- **Promise observation:**
  The outcome belt originally recorded an observed result only when
  the next Create action ran,
  although the Promise observer had already reported it.
  The corrected lesson records the receipt in the observer callback.
- **Transfer candidate:**
  When a teaching aid claims to be an event log,
  its update must follow the event named by the label rather than an unrelated later action.
- **Limit:**
  This is a cross-domain hypothesis,
  not an observed cooking failure.
  Do not add a progress widget to a recipe merely to satisfy it.

### Keep operational and teaching evidence separate

- **Cooking observation:**
  The user's critique establishes that the quoted response was
  unusable in the specified ways;
  no learner cooking attempt or repaired recipe is recorded.
- **Promise observation:**
  Browser,
  Firefox ESR,
  export,
  and PDF tests establish artifact behavior.
  The user's “This is great” supports this design direction but does not measure independent
  application-building or explanation by the intended novice.
- **Transfer candidate:**
  Report which action paths and explanations were checked,
  then separately describe any learner performance actually observed.
- **Limit:**
  Do not convert the settled learner-observation veto into a new study requirement.
  Absence of learner-performance evidence limits the claim;
  it need not block a working artifact.

## Guardrails against over-transfer

- The apples example motivates multiplication by scaling the same familiar counting task.
  The Promise comparison motivates an abstraction against a real predecessor problem.
  The cooking critique does not establish that a novice must first fail at seasoning or shopping.
- Printable,
  interactive,
  self-contained HTML is this user's confirmed default for authored teaching.
  It does not mean every subject requires code highlighting,
  a button,
  or the same state diagram.
  Print must keep teaching substance without suppressing useful browser interactivity.
- The Promise toy's `"Hello Ada"` and `Error("No reply")` are manual fixture values.
  They are not service results or ingredients,
  and no game-faithful pudding processing is claimed.
- The original cooking critique must not be “repaired” by inventing a dish,
  location,
  temperature scale,
  or food-safety endpoint.
  Those details must be established or checked before an actionable recipe is written.

## Discriminating probes for a candidate skill

These are proposed tests of response decisions,
not a completed recipe,
a new Promise lesson,
or confirmation of the skill.
A passing response must meet the stated condition;
a named countercase prevents converting the condition into an unconditional script.

### Probe: ambiguity changes the method

- **Input:**
  The learner says,
  “Teach me to make rotated chicken,”
  with no dish or equipment clarified.
- **Fail:**
  The teacher silently chooses a dish and issues oven or rotisserie steps,
  or asks the novice to choose a vendor before understanding the intended dish.
- **Pass condition:**
  The teacher identifies the ambiguity as consequential to the method,
  explains its current interpretation as tentative,
  and asks a targeted question about the
  intended result or method before issuing a procedure.
- **Countercase:**
  Once the intended dish is already explicit,
  asking that same question again
  delays teaching without reducing uncertainty.
  The rule is not “always ask a question first.”

### Probe: specificity cannot repair an unverified premise

- **Input:**
  The quoted instructions say “A little bit of salt,”
  “300degrees,”
  and “Get raw chicken from grocery store.”
- **Fail:**
  The teacher patches each with invented numbers,
  a guessed temperature scale,
  or an unrelated shopping link before establishing the dish,
  equipment,
  source,
  and procurement preference.
- **Pass condition:**
  The teacher identifies the missing calibration,
  unit,
  and access path;
  obtains or checks the particulars needed for the actual recipe;
  then supplies usable instructions with a specific deliverable product link or an opened
  in-person map,
  matched to the learner's available tools,
  location,
  and preference.
- **Near miss:**
  Precise quantities and an explicit temperature unit still fail if the
  method or setting is unverified.
  A genuine product link fails if the product cannot
  be delivered to the learner or the promised map has not been opened.
- **Countercase:**
  A definition or conceptual explanation that asks the learner to perform
  no measured step need not manufacture quantities.
  The rule is about action boundaries.

### Probe: establish the need before naming the new abstraction

- **Input:**
  A learner can count aligned groups of apples with addition;
  the Promise learner can build a greeting page but has not established callback knowledge.
- **Fail:**
  A multiplication table appears first and the apples merely decorate it.
  Or a Promise API list appears before a meaningful callback coordination or failure problem.
- **Pass condition:**
  The apple arrangement keeps equal rows visible as repeated addition;
  increasing the same arrangement makes counting unwieldy before multiplication and its table
  are introduced.
  The Promise example teaches the relevant callback mechanism,
  develops a real coordination or failure problem with it,
  and compares Promise-based
  handling of that same problem before `Promise.withResolvers()` is taught.
- **Near miss:**
  Merely placing grouped apples next to an unexplained table,
  or callbacks next to a Promise API inventory,
  preserves the early abstraction failure.
- **Countercase:**
  No novice must fail at seasoning,
  shopping,
  or a hazardous task to learn
  a fact.
  Failure-driven scaling applies when a new abstraction answers an existing method's limit.

### Probe: the learner's role determines the causal model

- **Input:**
  A developer learner can independently build a single-file greeting page
  and is learning to make a browser chat interface respond to Send.
  This capability evidence does not mean every learner wrote one particular greeting page.
- **Fail:**
  “The browser calls your function when clicked” replaces listener registration,
  or the teacher asks the novice to specify an AI vendor and credentials as if that were
  required to learn the event or Promise mechanism.
- **Pass condition:**
  Teach the listener-registration and handler relationship first.
  When the lesson reaches reply coordination,
  show the send operation and its Promise;
  choose the offline reply fixture for that application-building exercise.
  Keep the destination visible without making production integration a prerequisite.
- **Near miss:**
  Showing the listener,
  send,
  Promise,
  and fixture in one untaught block
  names the pieces but still skips the learner's prerequisite sequence.
- **Countercases:**
  A listener-only step need not introduce a Promise yet.
  The manual `Promise.withResolvers()` experiment supplies an outcome without a service request.
  A novice cook needs a usable ingredient and equipment path,
  not listener registration.
  The shared rule selects the learner's causal responsibility,
  not identical subject matter.

### Probe: records update when their named event happens

- **Input:**
  In the preserved before-belt lesson,
  the Promise observer reports rejection
  while an “observed outcomes” belt is empty;
  another Create later reveals that result.
- **Fail:**
  A teacher describes the belt as current observation even though Create causes
  the update,
  or adds a receipt at the resolver call before the observer reports.
- **Pass condition:**
  Before the observer runs,
  the live and print lists gain no receipt.
  When it reports,
  add exactly one for the current Promise,
  even if no later Create occurs.
  Ignored calls and later Create leave both ordered lists unchanged.
- **Countercases:**
  A clearly labeled historical snapshot need not update live.
  The cooking critique supplies no need for a timeline or belt.
  These are transfer boundaries,
  not evidence of a cooking-timeline failure.

### Probe: a format promise does not override the teaching goal

- **Input:**
  The user requests an interactive and printable lesson.
- **Fail:**
  “Printable” is used to remove useful browser interactions,
  or an attractive browser artifact prints without its relevant examples and explanations.
- **Pass condition:**
  Keep interactions that expose the concept and supply complete static
  counterparts in print.
  Choose subject-appropriate visuals rather than copying the
  Promise factory,
  code highlighting,
  or controls into cooking by default.
- **Countercase:**
  A quick Markdown alternative requested by the user need not acquire
  artificial browser controls.
  The confirmed default medium is conditional on the request.

### Probe: verification claims do not outrun evidence

- **Input:**
  The Promise lesson passes browser and PDF checks and the user likes the result.
- **Fail:**
  “The novice can now build the chat application independently” is inferred
  from working reference code,
  artifact tests,
  or the user's design feedback.
- **Pass condition:**
  Report the artifact's tested behavior and the user's feedback;
  withhold claims about independent learner performance unless it was actually observed.
- **Countercase:**
  Lack of a learner study need not prevent using or improving the artifact;
  the user already declined that as an approval gate.

## Dry run against the recorded cooking fragments

The only recorded learner prompt is “Teach me to make rotated chicken”.
The recorded assistant fragments include “A little bit of salt”,
“Get raw chicken from grocery store”,
and “Set oven to 300degrees”.
The quote is not a full recipe,
so this dry run scores only the faults it exposes.
[The rotisserie definition][rotisserie-definition] makes that term a plausible interpretation:
it names an appliance that rotates food on a spit before or over heat.
It does not establish the dish the learner intended or their equipment.

- **Observed-fragment verdict:**
  The salt instruction lacks usable calibration,
  the oven setting lacks a scale,
  and the shopping instruction offers no actual access path.
  Proceeding as though “rotated chicken” names a settled dish leaves the target unresolved.
  There is no evidence here about a printed artifact,
  learner performance,
  or a cooking timeline.
- **Bounded first-turn repair candidate:**
  “I suspect you mean rotisserie chicken,
  but I don't want to give you steps for the wrong preparation.
  Is rotisserie chicken the dish you want to make?”
  This demonstrates a reasoned interpretation and a consequential clarification,
  not a complete cooking lesson or a verified recipe.
- **Verdict on that candidate:**
  It passes the target-disambiguation move and does not
  invent equipment,
  quantities,
  food-safety facts,
  or procurement access.
  It cannot yet pass the actionability probe:
  the intended dish,
  learner's equipment,
  recipe facts,
  and delivery-versus-trip preference still need to be established or checked.
- **Near-miss control:**
  Filling the original response with precise-looking amounts,
  a temperature scale,
  and a genuine but unusable product link would not repair an
  unverified dish or inaccessible shopping route.
  A rule that rewarded numbers or links
  alone would incorrectly pass that response.
- **Promise contrast:**
  The preserved before-belt scene fails the event-record probe;
  the corrected lesson passes the observed-receipt guard in browser,
  Firefox,
  and print.
  Neither result supplies evidence that cooking needs an event log.

This is a dry run of proposed checks against recorded excerpts,
not a test of a completed skill or proof that the candidate reply teaches the learner.

## Current interview frontier

The original request already covers live teaching and authored materials.
The HTML default,
optional Markdown alternative,
print completeness,
and learner-observation
veto are settled;
do not ask the user to ratify them again.
No full cooking lesson has been requested or verified,
and these working probes have not been run against a finalized skill.
The broad Q1 that asked the user to invent a failure escaping this comparison was rejected.
[The primary-source research note](teaching-skill-research-feedback.md) now tests a
possible specificity gap:
reference output and raw errors do not necessarily tell the
learner how to check their own result,
interpret a mismatch,
and recover or stop.
Its cooking and Promise contrasts are proposed cases,
not observed learner failures.
The cited evidence supports making this a bounded candidate refinement in the
acceptance draft;
no preference question is needed to decide that drafting step.
Do not ask for a generic falsifying example,
checklist approval,
or a choice between more questions and implementation.

[rotisserie-definition]: https://www.merriam-webster.com/dictionary/rotisserie

# Proposed teaching-skill acceptance checks

## Status

This is a synthesis for shared-understanding review,
not the finished skill or an accepted policy.
The [discovery record](teaching-skill.md) retains the user's examples,
corrections,
and rejected questions.
The local Promises page is a source of counterexamples,
not an accepted teaching design.
`AGENTS.md` remains outside this change.
The user marked the independent review's `Concerns that require real learner observation` section as fine.
Do not turn that section into a learner-study requirement or an approval gate.
The [implemented review corrections](../handover/promises-review-corrections.md) supply additional concrete evidence.
A [planning-only decision outline](teaching-skill-decision-outline.md) and
[static counterexample replay](teaching-skill-outline-replay.md) exercise this draft;
neither is an implemented agent skill or final confirmation.

## Learner framing and prerequisite sequencing

### Establish capabilities without inventing a biography

Passing behavior:

- Describe the starting profile in terms of what the learner can do and explain.
- Treat a supplied example of those capabilities as evidence of the profile,
  not a claim that every reader wrote that exact program.
- Make the lesson's starting example self-contained for readers with that profile.
- Frame the destination through an understandable capability or problem,
  then introduce the subject's unfamiliar name when it has meaning.

Counterexamples this must catch:

- Calling a particular `Hello, name` program "your existing page" without establishing shared context.
- Opening with "Learn Promise" as if the unfamiliar abstraction explains the learner's destination.
- Describing a "chat" or "chat service" without establishing what participants and programs actually do.

### Introduce prerequisites before relying on them

Passing behavior:

- Account for the concepts and operations each explanation requires.
  Each is supported by the learner profile or an explanation the learner has already encountered.
- Show the concrete operation behind a term such as "define" or "display";
  a familiar-looking word does not establish the relevant programming concept.
- Place the object explanation before explanations that require understanding objects.
- Keep necessary foundations in the teaching path.
  Optional background can be folded away without making the main explanation depend on hidden knowledge.
- Bridge a changed state model before requiring a combined application:
  a single-operation cancellation exercise does not itself teach ownership of overlapping sends.
- State which supplied source is teaching machinery and which the learner must construct.
  Extra implementation details are optional only when the independent task has a taught alternative.

Counterexamples this must catch:

- Requiring the reader to understand objects in order to reach the paragraph explaining objects.
- Introducing undefined terms through headings and goal statements,
  then defining them much later.
- Replacing a missing prerequisite with additional jargon or a link that leaves its role unexplained.

### Teach the mental model appropriate to the intended role

Passing behavior:

- Identify what the learner must construct,
  control,
  diagnose,
  or explain to achieve the destination.
- Make those causal relationships visible rather than substituting an observer's account of the outcome.
- Match explanatory depth to that role and goal;
  this is not a requirement to teach every underlying implementation detail.

The click-listener counterexample:

- A browser user's account is "click the button and something happens".
- The developer needs the connection they create:
  attach a listener for the button's click event and supply the function it runs.
- "Let the browser call a function on a click" skips that developer model.
  Merely adding an event-loop footnote does not repair the missing relationship.

### Build from existing knowledge without making it a ceiling

Passing behavior:

- Introduce a useful new tool or concept when it serves the learning objective,
  showing its purpose and teaching the knowledge needed to use it.
- Choose the teaching environment for its contribution to the objective,
  rather than promising that the learner will never encounter anything new.
- Explain environment requirements where they support an actionable step,
  not as a substitute for an understandable learning destination.

The apples example establishes the sequence:

- Begin with concrete arrangements whose count the learner can establish using addition.
- Preserve the visible equal-row structure while increasing the problem's scale.
- Use the demonstrated limit of repeated counting to motivate multiplication and then the table.
- Do not lead with the table and assume that examples afterward establish why it exists.

### Make physical and setup instructions actionable

Passing behavior:

- Resolve an ambiguous intended dish rather than building a lesson around an unexamined name.
- Give a novice a usable quantity and the relevant measurement unit.
- Connect procurement to an actual usable product or location,
  using the learner's available tools and genuine delivery or travel preferences.
- Inspect available evidence before asking the learner to supply discoverable facts.

Counterexamples this must catch:

- "A little bit of salt" without calibration.
- "300 degrees" without a temperature scale.
- "Get raw chicken from the grocery store" without a usable procurement path.

### Ask from a reasoned understanding

Passing behavior:

- Investigate the subject and workspace before asking about facts or diagnosing a reported defect.
- Form an interpretation from the evidence,
  then ask about meaningful uncertainty that remains.
- Ask questions the learner is equipped to answer about their experience,
  goals,
  tools,
  and constraints.
- Choose incidental teaching fixtures rather than turning a learning destination into product specification.

Required rejected questions:

- Q5 asks the novice to choose an AI API,
  service contract,
  product audience,
  and access setup.
  The learner is not equipped to choose unexplained service or credential architecture;
  an answer such as "maybe A" would manufacture an unreliable requirement.
  An interview demonstration does not imply a production application.
  Its repair is the already-selected offline,
  semi-deterministic random fixture,
  with the lesson focused on Promises rather than production AI integration.
  Pedagogically relevant questions about the learner or learning audience remain appropriate;
  these are different from specifying the application's customers and service contract.
- Q9 asks what is misleading about the click-listener heading without first recognizing its role mismatch.
  Its repair is thoughtful analysis before a clarification question,
  not a prohibition on questions.

## Observable problems and causal explanations

### Establish the need before introducing its solution

Passing behavior:

- When an abstraction addresses the limitations of an existing method,
  let the learner use a concrete version of that method they understand.
- In that case,
  increase the same problem's demands when the limitation has not yet become apparent.
- Make the limitation observable in the learner's attempt or a demonstration they can inspect.
- Explain what the replacement changes and why that addresses the demonstrated problem.
- Other introductions can build directly from concrete observations or operations;
  learner failure is not a universal prerequisite for teaching any new fact.

Promise-specific acceptance example:

- Show the callback-based approaches actually used before Promises were specified and shipped,
  rather than inventing a predecessor solely to make Promises look preferable.
- Develop a meaningful coordination or failure-handling problem using that predecessor mechanism.
- Compare how the Promise-based approach handles the same problem.
- Distinguish this pre-Promise history from the established Promise constructor pattern.
- Explain what `Promise.withResolvers()` provides relative to that pattern
  after the underlying need and model are established.

Scope of the generalization:

- This requires a motivated abstraction,
  not an exhaustive chronology for every topic.
- Scaling should reveal a relevant method limit,
  not manufacture failure through unrelated difficulty.
- Use safe demonstrations or simulations where exposing a real-world failure could cause harm.
  The user's scaling example does not authorize exhausting hardware or creating physical hazards.

### Explain the operation the demonstration actually performs

Passing behavior:

- Distinguish an operation from the representation used to observe its eventual outcome.
- Explain which participant or piece of code starts work and which produces its result.
- Label learner-controlled settlement as a model of supplying an outcome,
  rather than presenting the learner's button press as an actual service request.
- Define waiting,
  failure,
  retry,
  and cancellation in terms of the specific operation and outcome.
- Separate chosen demonstration policies from guarantees of the language abstraction.

Counterexamples this must catch:

- Calling the manual settlement experiment "a service request" without distinguishing their mechanisms.
- Saying "stop" without identifying what stops and what may continue.
- Teaching the fixture's attempt count or timing policy as if it were Promise semantics.

### Use causal and temporal precision without losing the teaching sequence

Passing behavior:

- Replace unqualified "now" and "later" with the relevant boundary in a visible execution sequence.
- Distinguish creating a function,
  registering it,
  invoking it,
  returning,
  and continuing delayed work
  when the learner needs those distinctions for the current problem.
- Establish the relevant JavaScript execution and thread model before relying on claims about blocking.
- Connect code execution to the observed page behavior instead of treating an unchanged or responsive page
  as a self-explanatory account of the mechanism.
- Verify the actual execution paths used in a demonstration against current sources.

Counterexamples this must catch:

- "Stopping the whole page" without an established explanation of what occupies the execution thread.
- "Promise object now" without identifying the return boundary and the work that remains incomplete.
- Correct trace output accompanied by an explanation that skips the mechanism the learner must understand.

### Make conceptual distinctions inspectable

Passing behavior:

- Pair a distinction with a concrete example in which the difference matters.
- Let the learner relate the source operation,
  observed state,
  and resulting behavior.
- Use actual language behavior for claims about execution rather than an animation that merely asserts it.
- Show useful representations where structured values are introduced,
  returned,
  received,
  or changed.
  An English label alone can hide the object,
  collection,
  function,
  or Promise the learner needs to understand.
- Distinguish the original value,
  serialized or inspected text,
  and the program's observed outcome.
  Explain the representation tool before relying on it and state what it omits.
- Label snapshots and recorded examples as such;
  neither is live inspection of a value that has since changed.

Value-representation counterexamples:

- A Promise is represented only by “Promise object”,
  missing the opportunity to inspect it beside its result.
- JSON's empty object is treated as proof that a Promise has no outcome or accessible behavior.
- Showify's unknown-state marker is confused with the pending state.
- A formatted function name is taken to expose its closed-over variables.
- Debugger text is presented as JSON or executable source.
- A helper's formatting hooks are assumed to be universally side-effect-free.

Control-affordance counterexamples:

- A button whose click would neither invoke a useful operation nor produce an observable trace
  stays enabled and visually unchanged.
- A click leaves no trace on the control,
  in a record,
  or in the drawn scene.
- A fresh numbered button is disabled after settlement even though it could call the resolver
  and visibly log the ignored attempt that teaches first-settlement-wins.
- A repeatable action reuses one button whose label cannot tell the first call from the fifth.
- Every numbered control repeats a full call-history sentence,
  obscuring the ready control;
  the current Promise status appears only after that crowded control surface.
- An interactive analogy inverts its source:
  the falling object of the source game becomes a
  stationary agent while the line drops agents instead.
- A factory analogy claims game fidelity while omitting the source's supplied normal/defective
  cues,
  delaying a caught pudding's conveyor placement,
  sending a destroyed defective pudding
  onto the belt,
  or treating a performance-grade face as a Promise-outcome indicator.
- One drawn object is simultaneously the processed material,
  the durable Promise identity,
  and the public-looking outcome marking without explaining the distinct lifetimes.
- A belt labeled as observed outcomes stays empty after the current Promise is observed settled;
  it records that outcome only when Create makes a later Promise,
  implying the wrong cause.
- A value snapshot present in the lesson is dropped from a redesign of the same section.

The Promise lesson uses literal JSON beside live Showify snapshots,
with an explicitly recorded native Node inspector comparison.
Those are tools used by this example,
not a prescription that every subject needs these same formatters.

Promise acceptance example:

- Show a Promise resolved with another still-pending Promise.
- Make the unresolved outcome visible,
  then show how the eventual fulfillment or rejection is adopted.
- Use this to explain why "resolved" does not mean "fulfilled",
  rather than adding that sentence as an unsupported terminology warning.

The semantic basis was checked against MDN's
[resolve function documentation][resolve-function].
The local lesson now includes and exercises this adoption experiment.
The correction handover records its retained browser verification;
that evidence does not constitute approval of a general teaching-skill policy.

## HTML behavior and repository reuse

### Use the medium to expose the concept

Passing behavior:

- Build visual and interactive representations around the relationship being taught.
- Let relevant changes reveal their consequences:
  for example,
  increase equal rows,
  change code inputs,
  or observe a state transition beside its source.
- Keep code and the effect it explains close enough for the learner to connect them.
- Provide syntax highlighting and legible code presentation using relevant existing capabilities.
- Make controls behave as their presentation suggests.
  Distinguish read-only indicators from actions rather than adding arbitrary actions to every card.
- Remove required timing races rather than merely making their windows longer.
  Manual outcomes and prearranged action sequences are distinct ways to expose the actual mechanism.
- Inspect the usable reading space around controls and results,
  not only whether a programmatic click or horizontal-overflow assertion passes.

Counterexamples this must catch:

- A long explanation accompanied by buttons that do not help expose its underlying relationships.
- Pending,
  Fulfilled,
  and Rejected cards that look clickable but are not.
- State descriptions without the corresponding code when the representation could explain both together.
- Ignoring repository highlighting support and presenting all code as unhighlighted text.

Repository evidence:

- `package/ssg/aquati.cat/src/lib/rehype-highlight.ts` computes token-offset attributes.
- `package/ssg/aquati.cat/src/lib/markdown.ts` uses that plugin in the content pipeline.
- `package/ssg/aquati.cat/src/client/index.ts`,
  `highlightAllCodeBlocks`,
  reads attributes and registers DOM ranges with the CSS Custom Highlight API.
- `package/ssg/aquati.cat/src/style/highlight.ts` supplies the corresponding styles;
  `package/ssg/aquati.cat/src/style/base.ts` consumes them.

The repository lookup alone established a relevant implementation,
not a verified standalone integration.
The lesson subsequently reused its grammars and tag mapping with a separate native-editor presentation.
The correction handover records checks of changed code,
themes,
and print output.
Any other lesson must still exercise its own chosen integration.
The acceptance checks do not prescribe copying the whole site client into a lesson.

### Preserve both teaching substance and browser interactivity

Passing behavior:

- Default to visual,
  interactive,
  self-contained HTML with automatic theme adaptation and print support.
- Offer Markdown when the user wants that alternative.
- Keep the browser's useful interactivity even though the material also prints.
- Preserve full teaching substance in both browser HTML and printout.
- Supply usable static counterparts for the printout's interactive mechanisms:
  relevant code,
  examples,
  states,
  explanations,
  and exercises remain available.
- Check printed content coverage,
  not just whether a PDF was generated or fits on paper.

Required rejected behavior:

- "Printable" becomes an excuse to omit interactivity everywhere.
- The browser remains complete but the printout loses substantive material.
- Both outputs are simplified to avoid designing static counterparts.

The scope of print adaptation is interactive mechanics,
not conceptual depth or content.

## Verification and proposed scope

### Distinguish a functioning artifact from successful teaching

Passing behavior:

- Verify controls,
  displayed outcomes,
  examples,
  exports,
  offline behavior,
  and print behavior as applicable.
- Execute every worked comparison in its advertised editor or file context,
  including the promised controls and failure paths.
  Counting or printing an exercise is not execution coverage.
- Test the proposed repair,
  not only the original defect:
  deleting a duplicate declaration can leave missing dependencies and unwired controls.
- Separately review prerequisites,
  definitions,
  causal models,
  examples,
  and learner actions.
- Provide opportunities to build or perform independently and explain the result,
  progressively withdrawing supplied answers and scaffolding.
- Report the evidence actually obtained rather than equating a supplied working solution with learner ability.
- Distinguish evidence levels:
  browser checks establish artifact behavior;
  content review checks for the presence and structure of teaching elements;
  learner performance,
  explanation,
  or critique is needed to assess teaching effectiveness.
- Require demonstrated learner performance and explanation to claim mastery.
  Including an independent exercise does not establish that the learner completed it.
- Use learner critique to revise the teaching,
  not merely its superficial wording.

Counterexamples this must catch:

- Browser tests pass,
  so the lesson is declared pedagogically adequate.
- The reference chat application works,
  so the learner is declared capable of building one.
- The learner is said to have understood or noticed something without supporting evidence.
- A capstone appears in the dropdown and printout but is absent from the worked-comparison execution loop.
- A helper is described as Promise-returning while a stated input condition throws before the call returns.
- A label says backoff has started,
  but a Stop probe runs before the actual wait exists.

These distinctions govern what a report can claim.
They do not reopen the learner-observation concerns that the user accepted as fine.

### Candidate refinement: check the learner's own result

This is a proposed addition,
not a user-confirmed requirement or a claim that the
current Promise lesson lacks feedback.
[The primary-source research note](teaching-skill-research-feedback.md) distinguishes
school feedback and metacognition guidance from the food-safety criterion owned by
University of Minnesota Extension.
Transfer to adult or AI teaching remains an inference.

Passing behavior at a consequential learner action or inference:

- Give an observable criterion for the learner's own result using an available,
  appropriate check.
  Do not assume a displayed reference result proves their attempt.
- Distinguish a mismatch from its possible causes.
  Explain what the observed signal can
  and cannot establish;
  do not diagnose an underlying state from a label alone.
- Offer a bounded next action,
  a safe stop boundary,
  or what to report for help when
  the evidence cannot identify a cause.
  Give a chance to act on useful feedback.

Counterexamples this candidate should reject:

- A recipe with quantities,
  units,
  and a time instruction tells a novice to serve poultry
  without a verified food-safety check.
  Existing source-check requirements already count
  against this;
  the proposed refinement makes the learner's check explicit.
- A working Promise reference or raw preview error is treated as a diagnosis of the
  learner's own result.
  “Try again” does not distinguish operation start,
  settlement,
  and observation when the preview still says Waiting.
- Identical Waiting text and an empty error record are taken as proof that a Promise
  remains pending.
  Disposable learner-code probes produced those same visible signals
  both without a resolver call and after a literal value was resolved but never rendered.

Do not add an artificial checkpoint after every sentence,
force deliberately erroneous examples into every subject,
or make a learner study a prerequisite for using or improving material.
The current Promise lesson's rejection starter explicitly names the unhandled failure,
provides a `try`/`catch` hint,
and its worked comparison shows the error reason.
An instrumented disposable variation recorded handler and resolver-call boundaries;
this is a demonstration of how to gather evidence,
not a published lesson change.
No learner has been observed struggling in either case.

### Scope proposed for confirmation

The skill would guide both teaching conversations and authored teaching materials across subjects.
The cooking,
apples,
and Promise cases are acceptance examples,
not limits on its domain.
Its process would cover discovery,
learner and goal modeling,
instructional sequencing,
medium-specific construction,
and distinct operational and pedagogical review.

Required counterexamples belong in material the agent must consult at the relevant decision:
product-specification questions posed to a novice,
print support used to suppress browser interactivity,
and clarification without an initial reasoned diagnosis.

The final skill and any lesson reconstruction remain separate from this requirements proposal.
The implemented lesson repair is evidence for these counterexamples,
not confirmation of this proposal or completion of the teaching skill.

[resolve-function]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/Promise

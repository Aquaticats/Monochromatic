# Teaching-skill research: learner-facing checks and recovery

Status:
planning hypothesis,
not an accepted skill,
lesson revision,
or request for a learner study.
This report tests a possible omission in [the acceptance checks](teaching-skill-acceptance.md)
and [the transfer probe](teaching-skill-transfer-probe.md).
It does not establish that the quoted cooking learner or the Promise learner encountered this omission.

## Candidate failure

A teacher can name the correct target,
supply a usable action and prerequisites,
explain the mechanism,
provide an accurate demonstration,
and verify the artifact,
yet leave a novice unable to tell whether **their own attempt** is on track or what to do when it is not.
The missing teaching decision is not merely to display an outcome.
At a consequential procedural step,
specify an observable check,
how to interpret a mismatch,
and a safe next move or stop boundary that does not presume the novice can diagnose the cause.
This is a proposed extension of the existing action-boundary check,
not an observed failure in either lesson.

The existing checks already require observable Promise states,
honest controls,
worked-comparison execution,
actionable quantities,
and chances to perform independently
([acceptance checks](teaching-skill-acceptance.md),
[transfer probe](teaching-skill-transfer-probe.md)).
They do not explicitly require the learner to compare **their result** with a criterion at the step,
or teach the response to a mismatch before proceeding.
The observed-outcome belt tests when a reference scene records an event;
it does not establish a learner's ability to diagnose a different outcome.
A reference file that succeeds and an independent exercise that exists can therefore pass those checks
while the exercise's first wrong result has no interpretation or recovery path.
This is a reading of the current documents,
not an empirical verdict on the lesson.

## What primary sources establish

- **General instructional guidance,
  bounded to school settings:**
  The Education Endowment Foundation (EEF) describes effective feedback as information relative
  to learning goals that gives specific ways to improve,
  including feedback when work is correct.
  Its implementation guidance calls for assessing understanding,
  explaining what succeeded or failed
  and why,
  and giving pupils an opportunity to act on feedback
  ([EEF feedback toolkit][eef-feedback]).
  Its teacher-feedback guide covers teacher-delivered feedback for ages 5 to 18 in school subjects,
  so applying it to an adult,
  an AI tutor,
  or a recipe is a design inference,
  not a tested effect
  ([EEF teacher-feedback guidance][eef-feedback-guide]).
- **General strategy,
  not a universal script:**
  EEF advises explicitly modelling and scaffolding how pupils plan,
  monitor,
  and evaluate
  their work,
  using subject-specific self-questioning or success criteria rather than a detached
  thinking-skills lesson.
  It cautions that a task should challenge pupils without preventing them
  from applying the strategy
  ([EEF metacognition toolkit][eef-metacognition],
  [EEF metacognition guidance][eef-metacognition-guide]).
  The proposed step-local check and recovery route are one application of that advice;
  EEF does not prescribe an identical checkpoint after every step.
- **Original,
  domain-specific comparison:**
  McLaren,
  Adams,
  and Mayer randomly assigned middle-school pupils to an interactive decimals
  lesson using erroneous examples or supported problem solving.
  Both conditions received correctness feedback;
  the erroneous-example condition asked pupils to identify,
  explain,
  and fix mistakes,
  then solve practice problems.
  Among the 390 analysed pupils,
  the groups did not differ significantly on the immediate test,
  while delayed-test gains favoured erroneous examples (reported effect size `d = .33`).
  The erroneous-example condition took longer and included more explanation prompts;
  the study did not isolate which component caused the difference.
  Its misconception-awareness analysis did not show a general condition advantage,
  so it does **not** prove that the intervention taught pupils to detect their own mistakes
  ([authors' published study][decimals-study],
  Methods,
  Results,
  and Limitations).
  This supports testing a supported find,
  explain,
  correct opportunity,
  not mandating deliberately wrong examples in every subject.
- **Physical-domain criterion,
  not a generic sensory cue:**
  University of Minnesota Extension says colour and texture alone cannot establish whether
  cooked food is safe;
  a food thermometer checks internal temperature.
  It describes placement in the thickest part of whole-muscle poultry without touching bone
  ([Extension thermometer guidance][thermometer]).
  This shows why a recipe's oven setting and elapsed time are not themselves the learner's
  criterion for a safely finished chicken.
  It does not identify the intended dish,
  appliance,
  recipe,
  or particular safe endpoint
  for the recorded “rotated chicken” request.

## Falsifiable contrasts for the proposed check

- **Cooking near miss:**
  After the dish and equipment are clarified,
  a hypothetical recipe
  supplies sourced ingredients,
  measured seasoning,
  an oven unit,
  and a valid shopping route.
  It then says “cook for the stated time and serve” without a check of the food itself.
  A surface scan for quantity,
  unit,
  access,
  and target might miss this;
  the existing requirement to check food-safety sources already counts against it.
  The proposed refinement makes the learner's own safety observation explicit because
  time or appearance does not supply the cited criterion
  ([Extension thermometer guidance][thermometer]).
  A contrasting lesson identifies the appropriate sourced internal-temperature criterion,
  teaches how the available thermometer is used and what to do if the reading is not yet adequate,
  and does not authorize serving on an unverified cue.
  This is a design test,
  **not** a replacement recipe or an invented temperature.
  If no thermometer or applicable safety source is available,
  the teacher cannot claim
  that appearance proves safe completion;
  the task needs a verified safe path first.
- **Promise near miss:**
  A novice follows an executable reference showing a successful send,
  but their independent preview stays at “Waiting…” or reports an unhandled rejection.
  Neither display alone proves whether the underlying operation is still pending.
  A status label and correct reference trace do not explain the learner's own different result;
  “try again” without distinguishing operation start,
  settlement,
  and UI observation fails
  this candidate check.
  A contrasting exercise gives a fixture-specific expected observation,
  guides inspection of those separate boundaries,
  and offers a bounded next step or a request
  for help when the signal cannot diagnose the cause.
  This is a proposed teaching case,
  not a claim that the current Promise lesson lacks those aids.
- **Countercase:**
  A short explanation of what an unfamiliar term means,
  with no learner
  action or consequential decision,
  needs no fabricated sensor reading,
  error tree,
  or retry.
  A self-check becomes useful when it bears on a real action or inference;
  forcing one after every sentence would not follow the EEF's subject-embedded strategy
  ([EEF metacognition toolkit][eef-metacognition]).

## Applying the contrast without inventing a learner failure

The generated Promise lesson already has per-exercise hints and a preview that reports
JavaScript errors and unhandled rejections (`doc/planning/promises-teaching.local.html`,
`exercises` and `installPreviewDiagnostics`).
Those are real supports;
their existence does not establish that every wrong result is
explained,
nor that a particular learner was stuck.
The proposed skill distinction is narrower:
a reference trace,
raw diagnostic,
and generic
hint can coexist with an independent task whose unexpected result lacks a usable decision path.

- **Hypothetical cooking completion A:**
  A clarified recipe names ingredients,
  units,
  access,
  and cooking duration,
  then instructs the novice to serve when time elapses.
  That may look complete under a surface scan for specifics,
  but it conflicts with
  [Extension thermometer guidance][thermometer] and the existing transfer probe's
  requirement to check food-safety sources.
  The refinement makes a learner-usable
  criterion for their own result explicit,
  not a new permission to ignore safety.
- **Hypothetical cooking completion B:**
  The same recipe adds a sourced internal-temperature
  criterion,
  thermometer placement for the applicable food,
  and a safe next step when its
  reading has not met that criterion.
  No numerical endpoint is supplied in this comparison.
- **Hypothetical Promise completion A:**
  “The reference worked;
  if your preview still says
  Waiting or shows an error,
  try again.”
  It treats display text as an explanation of the
  underlying Promise and supplies no comparison with the learner's attempt.
- **Hypothetical Promise completion B:**
  Give the exercise's expected observation;
  if its display still says Waiting,
  inspect whether Send started an operation,
  whether it
  settled,
  and whether the observer rendered the outcome.
  For an unhandled rejection,
  inspect the reason and catch boundary.
  None of these display symptoms alone identifies
  the cause.
  If evidence is inconclusive,
  identify what to report for help.

These paired completions test whether the proposed checks notice a usable criterion,
interpretation,
and next move.
They are not a new recipe,
a verified repair to the
existing Promise lesson,
or evidence of learner performance.

## Local probe of the existing Promise workshop

`mise run probe:learner-feedback` exercised the generated local lesson in an owned,
disposable browser session;
it did not reload the user's presented tab.
The authored “Handle rejection” starter leaves the result at `Waiting…` and reports
`Unhandled rejection: Permission denied. Retrying unchanged inputs will not repair this.`
The actual exercise task names that mismatch and asks for a local error handler.
Its hint says to put the awaited call inside `try` and show `error.message` in `catch`.
The worked comparison instead displays `Could not reply: Permission denied...`
with no preview error.
This is a positive control for feedback already present;
the research must not claim that the lesson provides no recovery guidance.

Two separate disposable learner-code variations then produced identical preview text,
`Waiting…`,
and an empty error record.
In one,
`Promise.withResolvers()` was never resolved;
in the other,
`resolve("Hello Ada")` had run but no observer updated the result element.
The probe itself tracked the resolver call and awaited the second Promise to confirm
its value.
It proves that this display alone does not identify the underlying state;
it does not prove a real learner wrote either variation or that the lesson needs a new UI.
A candidate teaching check can require evidence gathering across operation start,
settlement,
and display observation when a learner's own result differs from the worked case.

A positive control repeated both disposable variations with a learner-visible `#output`
record while leaving the result at `Waiting…` and the error record empty.
The unresolved variation recorded `handler ran`;
the resolved-but-unrendered one also
recorded `resolver called`.
The probe separately observed `"Hello Ada"` from that
second Promise.
This trace narrows the next investigation but does not itself prove
that every resolver call immediately fulfills a Promise or that the display observer ran.
The temporary learner code,
not the published lesson,
generated these records.

A direct falsifier of **newness** would be a reading of the current acceptance checks showing
that they already require the learner's own step-local criterion,
mismatch interpretation,
and recovery or stop action,
not only a working artifact,
observable demo,
or independent task.
A falsifier of **usefulness** would be a concrete procedure where these added elements neither
change the learner's next decision nor guard a consequential mistake.
Neither has been demonstrated by the existing cooking excerpt or browser tests.

## Scope of the finding

EEF synthesises school research and offers instructional guidance;
it is not a trial of this repository's adult or AI-authored materials.
The decimals study tests a bundled mathematics intervention,
not food safety or Promise instruction.
The Extension page owns the food-domain criterion,
not a general pedagogy finding.
The report's cross-domain step-local feedback loop is therefore an inference to probe,
not a confirmed universal rule or a claim of learner mastery.
Artifact checks,
content review,
and observed learner performance remain separate evidence levels
([acceptance checks](teaching-skill-acceptance.md)).
No learner study is proposed as an approval gate.

[eef-feedback]: https://educationendowmentfoundation.org.uk/education-evidence/teaching-learning-toolkit/feedback
[eef-feedback-guide]: https://educationendowmentfoundation.org.uk/education-evidence/guidance-reports/feedback
[eef-metacognition]: https://educationendowmentfoundation.org.uk/education-evidence/teaching-learning-toolkit/metacognition-and-self-regulation
[eef-metacognition-guide]: https://educationendowmentfoundation.org.uk/education-evidence/guidance-reports/metacognition
[decimals-study]: https://www.cs.cmu.edu/~bmclaren/pubs/McLarenAdamsMayer-DelayedLearningWithErrEx-IJAIED2015.pdf
[thermometer]: https://extension.umn.edu/preserving-and-preparing/food-thermometers

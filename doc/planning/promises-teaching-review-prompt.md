# Independent review of the Promise teaching lesson

I have quickly scanned this lesson and have no major complaints,
but I am concerned that I missed important problems.
Give me an independent opinion,
not reassurance or a quota of criticisms.
You have filesystem access but not the conversation that produced the lesson.

## What to review

Repository:
`/var/home/user/Monochromatic`.

Current artifact:
`/var/home/user/Monochromatic/doc/planning/promises-teaching.local.html`.

Review whether it can teach its intended learner to build and explain the intended application,
not merely whether the supplied application works.
This lesson is also a test case for a future teaching skill;
that skill has not been approved or implemented.
Your task is review,
not implementation or final skill approval.

### Learner and destination

The established starting profile is someone who can independently make a JavaScript hello-world program
and a single-file HTML page that prompts for a name and displays a greeting in the DOM.
Nothing beyond that is established.
This is a skill profile,
not a claim that every learner wrote one particular greeting program.

The destination is independently building and explaining an interview-demo browser chat with
rate limiting,
cancellation,
exponential retries,
API-fault tolerance,
and multiple conversations.
The learner's application should require no dependencies.
The lesson's bundled highlighting tools are a separate concern.

The subject is JavaScript Promises in that building path,
not production AI infrastructure.
A supplied local simulated API is appropriate.
Judge whether the learner can write and explain the coordination and interface,
not whether they can reproduce the simulator internals or obtain provider credentials.
Necessary prerequisites belong in the teaching path;
optional background may be disclosed separately.

### Required artifact behavior

- The opening should begin with a meaningful human activity,
  not unrelated controls with a chat label.
- Sending another message must remain possible before replies arrive,
  including within the same conversation.
  An unfinished draft is an option,
  not forced turn-taking.
- Fixture policies must be distinguished from general chat behavior and from JavaScript semantics.
- Intended demonstrations must be observable without a typing or clicking race.
- The HTML should be self-contained,
  interactive,
  automatically themed,
  and usable with native editing controls.
- Main long-form reading backgrounds must have zero saturation/chroma in both themes.
  Bounded purposeful accents may retain color.
- Printing must preserve the teaching substance through useful static counterparts,
  without sacrificing browser interactivity or hiding necessary explanations.

## Review method

1.  Read applicable repository instructions,
    then examine the current artifact before reading our planning notes.
    Record your initial findings independently.
    Cover the complete learner-facing sequence,
    exercises,
    hints,
    worked solutions,
    reference chat,
    and print path.
    Use rendered content or extracted blocks when helpful;
    a truncated raw file read is not complete coverage.
    Bundled third-party highlighting internals need investigation only if relevant to a finding.

2.  Trace the prerequisite chain from the stated starting profile.
    Identify unsupported conceptual jumps,
    undefined operations,
    misleading explanations,
    unnecessary cognitive burden,
    and exercises that reward copying without understanding.
    Check whether interactions reveal the claimed mechanism and whether the learner can transfer it to a new build.
    Distinguish helpful scaffolding from giving away the work the learner needs to learn.

3.  Audit the JavaScript and application model.
    Inspect definition versus invocation,
    event-listener wiring,
    return values versus later callbacks,
    Promise construction and observation,
    resolution versus fulfillment,
    adoption,
    scheduling and continuation order,
    and fair callback/Promise comparisons.
    Check that retries,
    limits,
    cancellation,
    and UI policies are attributed to application code rather than Promises.
    Verify disputed semantics with primary sources or bounded runtime probes.
    Evaluate instructional code for correctness and teaching value;
    production-style refactoring is not a substitute for this review.

4.  Exercise the interactions in your own disposable browser session if your tools permit.
    Include counterexamples,
    overlapping operations,
    failures,
    cancellation,
    reset,
    draft preservation,
    and reply ownership,
    rather than only the prescribed happy paths.
    Review layout,
    keyboard use,
    editing,
    light/dark rendering,
    and actual print output.
    Separate observed defects from source-backed concerns and untested pedagogical judgments.
    An expert successfully following the lesson is not evidence that a novice can independently build the result.

5.  After your independent pass,
    consult these records and challenge their conclusions:
    - `/var/home/user/Monochromatic/doc/planning/teaching-skill.md`
    - `/var/home/user/Monochromatic/doc/planning/teaching-skill-acceptance.md`

    They contain chronological history,
    prior corrections,
    and provisional acceptance hypotheses,
    not a complete specification of quality or proof of acceptance.
    Check historical complaints against the current artifact before reporting them as current defects.
    Review test assertions too:
    passing tests establish their checked behavior,
    not the soundness of the teaching model.
    Look beyond the failure categories already recorded.

## Supporting files and boundaries

Authoring fragments and verification helpers:
`/var/home/user/temp/agent/promises-revision/`.
Compare relevant fragments with the delivered artifact rather than assuming they match.

Generated print output:
`/var/home/user/temp/agent/promises-revision/revised-lesson-print.pdf`.
The sibling `revised-lesson-print.txt` and inventory files assist content checks,
but cannot establish visual readability or complete teaching coverage by themselves.

Inspect the scratch `mise.toml` before using existing checks.
`test:all` includes a build that overwrites the artifact;
some other checks rely on fixtures whose previous processes have been stopped.
Use disposable copies and resources for experiments,
not the supplied authoring files or existing user sessions.
Browser-tool troubleshooting records are under the repository's `doc/troubleshooting/agent-browser-*.md`.
Distinguish a tool or harness failure from a lesson defect.

Keep the lesson,
authoring sources,
`AGENTS.md`,
and existing browser tabs/drafts unchanged.
Use generated PDFs or disposable print previews,
not physical print jobs.
Close only resources you created.
If an interaction cannot be exercised,
continue through available evidence
and mark that coverage limitation explicitly.

## What to return

- Your overall assessment,
  separately addressing semantic correctness,
  teachability,
  independent-build preparation,
  and browser/print usability.
- Findings ranked by consequence for this learner.
  For each,
  give an exact location (section,
  element/script ID,
  source line,
  or PDF page),
  the quote or reproducible interaction,
  the likely mistaken inference or blocked learner action,
  supporting evidence and uncertainty,
  and a concrete repair direction with a way to check it.
- What is working and should be preserved.
- A coverage statement identifying the artifact hash,
  sections and activities inspected,
  interactions actually exercised,
  browser(s) used,
  print pages visually checked,
  and remaining gaps.
- Which concerns require real learner observation rather than further source inspection.

Prioritize consequential findings over cosmetic preferences.
State when something is an alternative teaching choice rather than a demonstrated defect.
Investigate discoverable facts yourself;
reserve questions for genuinely missing human context.
If the reviewed material holds up,
say so with the same evidence discipline.

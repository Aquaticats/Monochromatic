# Post-redesign direction: bound the production consolidation, measure on a four-entry set

## Status

Decided 2026-09-01.
The owner delegated the direction ("I don't really trust Sol at this point.
You decide.")
and chose the completion set themselves ("One small,
one medium,
one large, + Carena").
This record supersedes the implementation stop in
[`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md)
for legacy-pipeline work only.

## Context

The finite redesign (Candidates A through M) is terminal:
no candidate satisfied the normal-run publication contract,
and the failure report froze all implementation pending an owner decision.
Production still runs the slice pipeline on branch `translation-repair-rebased`,
generations 12 through 16.

The production pipeline's measured wedge is the consolidation correction machinery.
The 2026-08-30 Carena run bought 186 roster rounds over 16,659 seconds,
86 percent of summed round time was post-quorum grace,
40 of 46 naturalness reviews rejected the standing candidate,
and the exact-repetition detector never fired because wording and findings vary between rounds.
Continuous correction has no finite quality ceiling,
and its bounded terminal is a visible `INCOMPLETE` pause,
which the owner's normal-return requirement forbids:
a normal run must return one complete document,
and quality machinery may never withhold output.

The same pipeline's shape can publish:
the Toka_ls grace-300 rerun was read publishable by strict and independent readings
(`doc/audit/translation-repair-output-reading-20260826.md`).
That page predates generations 12 through 16,
so no existing page is readiness evidence for current HEAD.

## The decision

1.  The redesign stays closed.
    No Candidate N,
    no spent-prompt reuse,
    no changed route or ceiling for a spent prompt,
    and the `prototype/translation-repair-finite-pipelines` branch stays unmerged.
    The spent-prompt boundary in the failure report remains in force.
2.  Production remediation is authorized:
    bound the consolidation correction machinery so a normal run always terminates with a published page.
    Correction gets a statically known round budget;
    at exhaustion the best standing candidate ships and reviewer rejections demote to recorded findings;
    quality machinery never pauses an entry `INCOMPLETE`.
    Bounded errors remain only for infrastructure failure,
    per the normal-return requirement in
    [`translation-repair-pipeline-redesign.md`](../planning/translation-repair-pipeline-redesign.md).
3.  Production-ready quantifies over four entries:
    one small,
    one medium,
    one large,
    picked by measured source size and recorded before the run,
    plus Carena.
4.  After the bound lands,
    one fresh pass on current HEAD over that set,
    then the actual-output reading,
    and only a passing reading re-opens the readiness signal.

## Owner refinement, 2026-09-01, later the same day

After instructing a full reading of the failed candidates
(recorded in
[`translation-repair-redesign-insights.md`](../planning/translation-repair-redesign-insights.md)),
the owner refined the direction:

"I would say loops are discouraged altogether.
The models Hyper and Synthetic provide has become much more advanced than when this project started.
We shouldn't even need loops if we play it smart."

A follow-up clarification the same day:
"Discouraged,
not a ban.
You can overrule on this if you provide enough evidence and get an approval."

Together these supersede item 2's bound-the-loops shape:
stages default to fixed-depth graphs with no correction loops,
and the design leans on current-generation models instead of compensating machinery.
A loop may stay only where evidence shows it is needed and the owner approves that specific loop.
The model-advancement premise is measured against the live provider catalogs before the design is fixed.
Item 1 (redesign stays closed),
item 3 (the four-entry completion set),
and item 4 (one pass,
then the reading,
then the readiness signal) stand.

## Why this direction

- It keeps the only architecture that has produced a page a reading accepted.
- It targets the one measured non-termination,
  which is localized in consolidation,
  not in the translate or repair lanes.
- It brings production into compliance with the owner's own corrected requirements:
  finite work,
  normal run returns output,
  naturalness is not a measurement,
  no generic gate with withholding authority.
- The alternatives re-enter the space where thirteen whole-document candidates failed
  for reasons not confined to the roster,
  or reverse a standing policy to fix only truncation.

The known cost:
at round exhaustion the pipeline ships text its reviewers rejected.
The owner's "naturalness is not a measurement" decision says those verdicts are unstable labels,
and the Toka_ls contest that split 4 to 4 yet read publishable is one measured case in point.
Whether exhaustion-shipped text is acceptable is exactly what the four-entry reading measures.

## What this does not decide

- The exact round budget.
  That is designed and justified during implementation,
  from the retained round accounting of real runs.
- Which small,
  medium,
  and large entries are picked.
  That is measured from the corpus and recorded in `doc/planning/`.
- Whether roster research happens later.
  Only a failing four-entry reading reopens that question.

## Verification owed before the build

The owner distrusts the takeover sessions,
so the load-bearing claims are re-verified independently before code changes:

- re-derive the Carena wedge numbers from the retained analysis artifacts,
  not from the takeover prose;
- re-run the whole-package suite,
  lint,
  and type check at current HEAD;
- read the consolidation loop modules directly and confirm the mechanism the docs describe.

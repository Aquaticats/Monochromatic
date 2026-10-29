# Superlinear cost dismissed as acceptable: gate research

## Purpose

State of the research into how to stop agents from waving away superlinear cost with "inputs are short today".
Written 2026-10-29 for the next session.
The owner paused the work, so everything under "Open" is unstarted or half-started.
No `AGENTS.md` edit, decision doc, or code change came out of this work.

## Trigger

The owner found a thinking block in an earlier session (`73f0639e`) that read:
a mild quadratic cost in the gap-finding logic is acceptable because lines are short in practice,
despite not being strictly linear for unbounded input.
A second thinking block in the same stretch called the cost negligible for short CLI rows.
The owner's reply: lines might be short now, but they can grow.
The owner then asked for a way to make future agents less likely to do this,
and rejected a wording-only change for this one case as a band-aid.

## The code behind it

The sentence concerns `readBinRow` in `package/file-enforcer/core/src/toolchain/providers.parse.ts`.
It tested every column gap with a `.some()` over all gaps, inside a `.find()` over the same gaps.
The fix is commit `58d2bb414d`.
Its message records a crafted 20,000-gap row taking 2383 ms before and 4.5 ms after.
That the sentence concerns this function is an inference from the code and the commit,
because the earlier transcript was not read in full.

## Findings

### The existing rules already cover the failure in text

`RG2` requires a regex replacement to be proven O(n) for unbounded input.
The dismissal quoted that criterion and overrode it.
`doc/philosophy/agents.md` says `AGENTS.md` grows only when no rule covers the failure.
`doc/audit/chain-flatten-skewed-tree.md` reached the same conclusion for a different bug:
correct in text, failed in application, same trap twice.
Its follow-up added sharper text in three places, and the class recurred.
The `PXQ` section of `doc/philosophy/agents.md` records a rule that stayed in `AGENTS.md` and still failed for a month.

### The dismissal lived only in thinking

In `73f0639e` the sentence appears only in thinking blocks, not in a message or a tool input.
The stop hook sends no thinking blocks to its detectors (`package/claude-code-plugin/stop-reminder/README.md`, the Luna section),
and no document I read gives a reason for that exclusion.
The first block also leaned on "passes fuzz testing" as reassurance,
which short random inputs cannot justify.

### Prior art

No published false-positive rate exists for a lint rule of this kind.
The closest is `react-doctor/js-set-map-lookups`, an oxlint rule that flags `includes` and `indexOf` inside loops.
Its page says a match proves only the modeled source pattern.
A Sentry experiment found one real quadratic reduce callback and reported no totals.
`doc/planning/wall-clock-bound-test-remedies-review.md` measured representative quadratic string work at 40 ms to 200 ms
against a 250 ms bound, so an absolute wall-clock bound does not catch it.
That review recommends raising the input size or adding a two-size ratio check with a positive control.

## Rejected: a nested-scan lint rule

I prototyped a detector in scratch and measured it.
It parses production TypeScript with Node's type stripping plus `acorn`
and reports a search call (`indexOf`, `includes`, `some`, `find`, and similar) inside an iteration callback or loop body.

- Scope: 5,404 files selected, 5,390 parsed, 14 failed to parse.
- Counts by cumulative tier: 1,923 for any scan nested in a scan or loop,
  1,282 for a search call, 653 when the searched collection is declared outside the nest,
  and 511 when that search also has no start-position argument.
- False positives: I classified every 12th hit of the 490 tightest-tier hits not on a module-level constant, 41 samples.
  Seven were test fixtures or fuzz packages that the file filter missed, leaving 34.
  About 19 were harmless (constant character sets and rosters, small fixed domains, string searches, short CLI dedupe).
  About 10 looked like real superlinear work on document-scale data,
  and about 5 were unclear.
- Recall: on the pre-fix code of 8 historical commits that fixed non-regex quadratic work,
  the tightest tier flagged the changed hunks in 3, a looser tier in 1, and missed the rest,
  including the accumulator-copy fixes.
- Cost: about 10 of 34 real-looking hits extrapolates to roughly 140 existing sites.
  Each suppression needs a justified disable comment and a doc (`LN3`), so the rule would demand hundreds of docs.

Limits of this evidence: one rater (me), no measurement of the data sizes behind any hit,
and a file filter that leaked tests and fuzz packages.
Conclusion: a lint rule is not viable as a gate at a tolerable false-positive rate.
The owner called the proposal laughable without a measured false-positive rate, and the measurement agrees.

## Open

The owner chose three directions after the lint rule failed.
None is finished.

### Ratio-test gate

The idea is a helper that runs a function at sizes n and 2n on adversarial input and fails when the time ratio is far above 2,
with a self-test proving a synthetic quadratic trips it.
A prototype exists in scratch (`ratio-gate.ts`) but was never run.
It times the exact parsers from before and after `58d2bb414d` on three input generators:
the typical row repeated, the typical row with a longer path column, and an adversarial row of many short columns.
What it should answer:
whether the ratio is about 2 for the fixed parser and about 4 for the old one,
the run-to-run spread on identical input (the prototype measures this),
and whether growing a typical sample row exposes the bug at all.
My prediction, unmeasured: repeating valid rows does not expose it,
because the first matching gap returns early, so the gate needs an adversarial shape that an agent must author.

### Separate-reviewer gate

Test whether an independent review (the `project-code-review` or `code-review` skills) flags the pre-fix `readBinRow` diff
when shown only the code and not the author's reasoning.
`doc/agent/self-review.md` says same-session review is not independent review.
Not started.

### Luna

Luna is `openai/gpt-6-luna-decisions`, the model the `ccsr` Stop hook uses to label the situation of the final message
(`doc/handover/stop-reminder-luna.md`).
Findings so far:
the hook sends the final message only, with no thinking blocks, and runs at the end of the turn,
long after a mid-turn dismissal.
The handover records that a Luna label for uncertainty is not built and needs its own calibration under #1772.
Thinking blocks are readable in the session transcript, so a detector could read them.
An experiment would replay thinking blocks through Luna, with the two dismissal blocks as positives and a sample of other blocks as negatives.
That sends reasoning text to an external provider, which the hook does not do today,
so it needs the owner's authorization first.
Not started.

## Scratch artifacts

All in `/var/home/user/temp/agent/`, uncommitted, and safe to delete:
`nested-scan-detect.ts` (detector and repo scan), `sample-hits.ts` and `sample-12.txt` (the classified sample),
`recall.ts` (historical fix-commit test), `tier3-hits.json` (the 511 hits),
`parse-before.ts` and `parse-after.ts` (the two parser versions), `ratio-gate.ts` (unrun), and `bin-list-scaling.ts` (unrun).
`nested-scan-detect.ts` needs Node's experimental `stripTypeScriptTypes` and the `acorn` copy in `node_modules/.pnpm`.

## Decisions on record

- Rejected by the owner: a wording-only change for this single case.
- Rejected by measurement: the nested-scan lint rule.
- Not decided: which gate to build, and whether to authorize the Luna experiment.

## Next action

Run `ratio-gate.ts` from the scratch directory with `node ratio-gate.ts` and read the output.
Then decide with the owner among the three directions, and ask before sending any thinking text to Luna.

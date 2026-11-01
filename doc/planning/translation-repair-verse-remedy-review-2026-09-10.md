# Reassessing the verse remedy

## Correction

The owner wrote:
"No,
the best way to fix that defect isn't red/green guards and a publication test."
This does not select another mechanism.
The claim that the class-twenty-nine guard was the right remedy is withdrawn.
Tests demonstrate behavior;
they do not establish that the behavior is the right intervention.

`Mio11` was stopped by sending SIGTERM to its verified pass pid `1653981` at 02:18 UTC.
The supervisor exited after 150 seconds.
No further source edits or paid launches followed the correction.
The guard remains committed but provisional,
not an accepted shipping remedy.
No automatic reversal is claimed or performed.

## What the evidence establishes

- Mio10's source-only closing poem has five explicit Markdown breaks.
  Its accepted translation and consolidated text have zero.
- The stored reply survey includes a break-preserving poem proposal as well as flat proposals.
  A producer was capable of preserving the structure;
  the existence of that reply alone does not establish its stage or eligibility.
- The artifact's slice-17 consolidation has `rewrapped: false`.
  Its base,
  proposed and shipped polish texts are the same flat text.
  The final wrapper is not where this selected text lost its breaks.
- The isolated source poem returns false from `isLineStructured`.
  That predicate requires five blank-separated blocks.
  The effective production flag also inherits from the enclosing chunk;
  that flag is not recorded in the settled artifact and must not be inferred from the isolated test.
- `translate-wire.ts`,
  `translate-selection-sheet.ts` and `consolidate-wire.ts` make verse instructions conditional on that flag.
  Their verse language refers to output lines,
  not explicitly to rendered Markdown breaks.
- The new `source-only-breaks.ts` rejects missing breaks after generation.
  It leaves those upstream instructions and their source representation unchanged.

Evidence:
`~/temp/agent/Mio10-poem-replies-20260910.out`,
`~/temp/agent/Mio10-poem-decisions-20260910.out`,
`~/temp/agent/Mio10-20260910/artifacts/Mio.json`,
and the source files under `package/module/translation-repair/src/` (moved on 2026-10-18 to
`package/translation-repair/*/src/`).
Raw provider payloads remain private.
The settled run's `slice-cache/Mio` directory is absent;
that is not evidence of an absent earlier stage or an absent inherited flag.

## Delegated remedy

The owner subsequently delegated the changes:
"Do whatever changes you think would be the best."
The selected remedy is a shared generation-and-selection contract for explicit rendered line structure,
not a replacement for prose generation and not another standalone refusal rule.

The production evidence gap is now closed.
`trace-Mio10-structure-20260910.mjs` rebuilt preparation through frozen `dc51b02d9` using every recorded pairing.
It reproduced 18 slices,
with slice 17's exact source bytes,
empty archive text and effective `lineStructured: false`.
The writer sheet lacked the verse rule and any hard-break instruction.
The recorded translate slate has a five-break GLM-flash candidate at index 3;
the winning Mercury candidate at index 2 has zero.
Its voters preferred wording and tone without addressing rendered structure.
Both choices existed before the wrapper.
Evidence:
`~/temp/agent/Mio10-structure-trace-20260910.out`.

Implementation direction:

- Supply parsed per-block explicit-break facts before initial writing,
  including single-block passages the existing verse heuristic does not govern.
- Make the shared contract distinguish rendered breaks from physical newlines.
  Recommend visible `<br/>` spelling while accepting real Markdown hard breaks.
- Show judges counts for their actual anonymous candidate ordering.
  A source-only shortfall cannot win merely for wording;
  counts do not excuse misplaced breaks or semantic defects.
- Carry the contract into author repair,
  lane comparison,
  consolidation production and final selection.
- Preserve raw source fences,
  source offsets,
  archive-backed layout authority and ordinary-passage prompt identity.
- Keep the existing source-only floor as a backstop,
  not evidence that generation is repaired.
- Run bounded writer and fixed-slate judge probes on this passage before another full-entry pass.

The existing `isLineStructured` heuristic remains a distinct blank-separated verse-unit rule;
its boolean will not acquire incompatible rendered-break semantics.
Structured translation units and source canonicalization are deferred:
the measured slate already contains a faithful break-preserving representation,
so generation and selection are the demonstrated seams to repair first.

## Implementation and bounded verification

`1a6ebbf62` implements the shared rendered-break prompt.
`553d0bcab` adds fact-boundary and real anonymous-slate tests.
`8699ffd61` centralizes the translated slate's criteria and preserves optional syntax correctly;
`d2e102b12` applies the final formatting correction.
The production-sheet guard `2f1c9a0e3` failed before the implementation.
Author repair preserves the original system contract through its existing conversation,
verified through its real builder rather than duplicating the rule.
Final polish does not rewrite these render-bearing blocks:
the built eligibility reader excludes blockquotes as `not-a-paragraph`,
Markdown hard-break paragraphs as `hard-break`,
and intrinsic-`br` paragraphs as `carries-markup`.
All were invoked directly to confirm the seam.
Its post-rewrite structural check uses the approved base,
not the original archive;
that check is not claimed as another source-only floor.

The helper's archive input is the actual archival wording,
never an eligibility-filtered fallback or generated standing.
Its activation means no archival wording supplies a competing layout,
not that an empty string decides the existing stage's fallback/absence policy.
That separate `incumbentKind` policy is unchanged.
Whitespace-only and other nonempty archive inputs are excluded explicitly.

Build,
types and oxlint pass with zero warnings.
The full suite ends `unit exit 0` in `~/temp/agent/rendered-contract-verified-unit-20260910.out`.
A comparison against frozen `dc51b02d9` proves byte-identical writer,
consolidation,
lane and gate messages for ordinary source-only prose,
archive-backed explicit lines and metadata:
`~/temp/agent/rendered-prompt-parity-20260910.out`.

The live probe uses frozen `d2e102b12` and the reconstructed 132-character source slice.
It compares baseline and new writer instructions on Mercury and GLM-flash,
then baseline and new judging of the same recorded five-candidate slate on the original four responding judges.
Both arms omit pictures and neighboring text to hold that window fixed;
this is not a rerun of the full production context or a timing calibration.
New consolidation instructions then receive the selected wording on the same writer seats.
Nominal requests:
four initial writer calls,
eight judge calls and two consolidation calls,
with a 720000 ms whole-probe deadline and existing model completion caps.
No source floor repairs the direct writer replies before their break counts are measured.
Outputs are compiled with the installed MDX compiler without executing generated code.

The first scratch launch failed before generation because the meter method needed `{ signal }`.
That script call was corrected,
not pipeline source.
The active probe is `proc_f94d` (`translation-repair-rendered-verse-probe-r1`),
with log `~/temp/agent/rendered-verse-probe-20260910-r1.log`
and report `~/temp/agent/rendered-verse-probe-20260910-r1/report.json`.
No source changes or full-entry pass are planned until the terminal report is read.

## Live results and selected source presentation

The matched contract probe completed in 378 seconds.
It did not establish that the initial writing problem was solved:
Mercury produced zero breaks under both contracts,
while GLM-flash produced five under both and adopted `<br/>` under the new contract.
All direct writer outputs compiled.

Selection did change on the fixed slate.
The baseline split its votes and raised `TranslateAbsenceError`;
the new contract selected the unique five-break rendering with three of four judges,
including explicit rendered-structure reasoning.
New consolidation outputs preserved the boundaries,
but Mercury added a sixth trailing break while GLM kept five.
That trailing break is not described as clean compliance.
Probe spend:
0.010003172 USD on OpenRouter,
0.00114931 USD on Bedrock,
and five Synthetic calls without a per-call price.

The planned supporting source-presentation experiment was therefore run,
without requiring every writer to become reliable on every instruction.
Only parser-confirmed Markdown break syntax in the existing model-facing source block changed to `<br/>`.
No second source view,
source-file edit,
structural repair,
new judge call or changed cap entered that experiment.
The control and treatment system messages were byte-identical;
the user-message diff was exactly the five source-break spellings.
Source display length changed from 132 to 147 characters.
The canonical source-file SHA-256 was identical before and after.

Both treatment writers produced exactly five breaks,
with no trailing `<br/>` after attribution,
and both outputs compiled.
Mercury now preserved the visible boundaries;
GLM continued to do so.
This is a passage-specific positive result,
not a general compliance-rate claim.
The report is `~/temp/agent/visible-source-break-probe-20260910/report.json`.
The treatment spent 0.00049012 USD on OpenRouter and one Synthetic call without a per-call price.
The daily-cost helper ran after both probes.

`b3113164a` implements that presentation in initial translation,
with explicit absent-incumbent provenance forwarded by the composed stage and existing probe callers.
Unknown presence defaults to retaining the original view;
nonempty archive wording and front matter are never transformed.
`parseSliceBody` shares offset-preserving comment and lone-container masking with structural admission.
`sourceBreakDisplay` rewrites only actual Markdown break-node spans,
keeps LF,
CRLF and CR endings,
and preserves code,
comments,
JSX attributes and all other characters.
A strict parser refusal logs why and keeps the original view.
Judges and canonical validation still receive the raw source.

Guard `1d026b754` failed six cases before implementation.
`79e82a368` verifies the actual composed stage:
its first writer requests receive visible source breaks,
its judge receives canonical raw source,
and the rendering needs no structural send-back.
Formatting and authority-check separation end at `ff6d288bc`.
The final build,
type check and oxlint pass with zero warnings.
The full suite ends `unit exit 0` in `~/temp/agent/source-display-verified-unit-20260910.out`.
The integrated production source view exactly matches the measured treatment;
the system-message comparison differs only by replacing a relative-position phrase with "reported per-block counts".
That comparison is `~/temp/agent/source-display-treatment-parity-20260910.out`.
A separate reviewer supports proceeding to the required full page reading after those checks,
not buying another prelaunch experiment.
The next reading must still inspect consolidation's break placement and trailing-break behavior.

## Recommended direction

Repair the existing generation and judging contract around rendered structure.
Parsed explicit breaks should be source facts visible to writers and judges even inside one block.
The contract must distinguish a rendered break from a physical newline.
Consolidation must receive the same fact rather than reinterpreting the source from another rule.
The guard and publication test can verify this remedy,
but cannot substitute for it.

Before implementation,
recover the effective source-structure fact and trace the break-preserving proposal to its stage and ballots.
Then compare a targeted contract change against the current behavior on the same passage,
without buying another full-entry pass merely to test a rejection rule.

### Improve the existing rendered-structure contract

- Pros:
  changes what writers produce and judges select;
  uses the existing verse-handling path.
- Cons:
  remains model-mediated;
  requires measurement through selection rather than a prompt-string assertion.

### Make explicit breaks visible in the model-facing representation

- Pros:
  a canonical visible spelling such as `<br/>` avoids relying on trailing spaces.
- Cons:
  requires careful scoping and renderer verification;
  changing the source presentation alone does not ensure judges preserve it.
- Relationship:
  a possible supporting part of the recommended contract repair,
  not an alternative to it.

### Move syntax ownership into structured translation units

- Pros:
  keeps authored syntax outside unrestricted prose generation.
- Cons:
  changes the translation interface and handling of legitimate English expansion;
  the current incident does not yet justify that scope.

Ranking for intervention priority:
repair the existing contract first,
measure visible break representation as a supporting change next,
and consider structured units only if that evidence establishes the need.
The first precedes representation changes because both writers and selectors currently need the rendered distinction.
Representation precedes structured units because it can retain the existing translation interface.
These are complementary interventions,
not mutually exclusive user choices.

## Independent review

A fresh Advisor review on `openai-codex/gpt-5.6-terra` agrees that the guard does not establish the best remedy.
The earlier review answered whether a narrowly scoped guard fit existing authority,
not whether it was the best intervention.
Its agreement was incorrectly promoted into confidence about remedy selection.
The inherited production flag remains an evidence gap despite that review's initial inference.

## Proposed instruction amendment

Replace `TC2` in the worktree's `AGENTS.md`,
rather than add a duplicate testing rule:

> TC2:
> Passing tests prove neither completeness nor remedy choice.
> Prefer preventing the failure to rejecting its output;
> compare test names with implementation branches.

This is a proposal,
not an applied instruction change.
The existing guard-demonstration requirement in `GFP` remains unchanged.

## Cost and next action

Stopped Mio11 logged 0.003006791 USD on OpenRouter,
0.00038817 USD on Bedrock,
and six Synthetic calls with no per-call price.
These are logged amounts,
not a post-cancellation balance reconciliation;
in-flight usage may not have been reported.
The daily helper ran into `~/temp/agent/Mio11-costs-stopped-20260910.out`.

Both bounded probes are complete and their outputs and costs have been read.
The integrated source-display checks pass.
Mio12 launched at 03:45 UTC on frozen `ff6d288bc`,
pass pid `1713424`,
terminal-only supervisor `proc_6a7b`.
Its page and log live under `~/temp/agent/Mio12-20260910` and the corresponding `.log` file.
The page was read in full after settlement at 07:02 UTC.
Its poem carries five visible breaks,
with soft continuation wraps within each original row and no trailing break after attribution.
Translation and publication use the same poem;
consolidation kept that standing text.
This closes the original verse-flattening defect at the published-page boundary.

The queue is still blocked by a separate initial-attempt artifact-validation error and recorded wording concerns,
not by missing evidence for the break remedy.
The full Mio12 reading is in the OpenRouter planning log.

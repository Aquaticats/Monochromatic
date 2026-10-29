# Translation repair

Multi-model translation critique and conservative repair.
The family lives in `package/translation-repair/`,
one directory for each of its 36 packages.

Takes an original text plus its translation,
returns a structured issue list anchored to an immutable document model,
and a repaired candidate translation.

## The packages

Until 2026-10-18 the pipeline was one package,
`package/module/translation-repair`.
It was split into the packages that follow because its lint took too long as one unit,
and the owner asked for packages of a manageable size.
The measurements and the map of every moved file are in
[the split plan](../../doc/planning/translation-repair-package-split.md).
Each package has its own `README.md` that names what it exports and what it depends on.
`markup` is the lowest layer and imports no other family package;
every other package with production code lists it in `dependencies`,
except `consolidate`,
which lists it in `devDependencies` only.
`source-scan` and `test-fixture` hold no production code and have no build.

- [`adjudication`](adjudication/README.md):
  the critic and panel prompts,
  candidate selection,
  the ballot tallies and the screen for defects a repair introduced.
- [`archive`](archive/README.md):
  the reviews,
  disputes,
  revision checks,
  coverage verdicts and screens over the translation a page already carries,
  and the preparation of its sections and blocks.
- [`artifact`](artifact/README.md):
  the two-lane settled artifact built from a run,
  read back with strict checks,
  and rebuilt into the preparation the run settled with.
- [`artifact-pool`](artifact-pool/README.md):
  the pool of settled artifacts,
  its eligibility policy and the comparison of the two lanes' rows.
- [`assembly`](assembly/README.md):
  assembly integrity and repetition checks and the repair contract and record shapes.
- [`bench`](bench/README.md):
  calibration benches and probes that run from outside the pipeline.
- [`consistency`](consistency/README.md):
  the page-level pass that makes the same wording read the same across a page.
- [`consolidate`](consolidate/README.md):
  the step that decides whether a new rendering replaces the standing text of a contested slice.
- [`corpus-pass`](corpus-pass/README.md):
  the corpus pass that runs the whole pipeline over every complete pair and settles an artifact.
- [`corpus-source`](corpus-source/README.md):
  the pinned corpus reader and the preparation of an entry for the pipeline.
- [`coverage-census`](coverage-census/README.md):
  finds the family code no unit test ran,
  by running every package's unit suite under one V8 coverage directory.
- [`document-structure`](document-structure/README.md):
  the MDX and Markdown parse and the heading and block alignment of original and translation.
- [`glossary`](glossary/README.md):
  the community glossary and the rules that read a page against it.
- [`house-rule`](house-rule/README.md):
  the house policy text and the identity and name context of a page.
- [`insertion`](insertion/README.md):
  checks for carried insertions and container halves and the corpus-run bookkeeping beside them.
- [`lane-contest`](lane-contest/README.md):
  the contest between the lanes over a contested slice,
  the checks that follow it,
  and the seeded-error and sampling tools that measure the pipeline.
- [`markup`](markup/README.md):
  the lowest layer with the character classes and markup scanners every other package reads text through.
- [`page-check`](page-check/README.md):
  published-page checks and the page-assembly passes.
- [`page-pass`](page-pass/README.md):
  passes that restore or splice text around the model lanes.
- [`probe`](probe/README.md):
  probes and the reports that read graded sheets and artifacts.
- [`provider`](provider/README.md):
  routing of a call to the provider that serves its model and the provider clients.
- [`provider-stream`](provider-stream/README.md):
  the wire-level half of the model clients with credential masking and stream watching.
- [`reference`](reference/README.md):
  lookups of the cited pages and work titles that the critics use.
- [`rendering-audit`](rendering-audit/README.md):
  the audit that asks a roster whether a rendering says what its original says.
- [`repair`](repair/README.md):
  the repair lane and the refine lane with the editor ensemble they share.
- [`run`](run/README.md):
  run configuration and the reports that read a run back.
- [`shared-contract`](shared-contract/README.md):
  small shared shapes such as model cards and roster ids that many layers read.
- [`slice-cache`](slice-cache/README.md):
  the slice and consolidation caches that make a pass resumable and the audits over them.
- [`slice-reading`](slice-reading/README.md):
  reads a document into slices and pictures and holds the image reading path.
- [`source-scan`](source-scan/README.md):
  the package-wide source scans of the family,
  with no production code.
- [`stage`](stage/README.md):
  the panel-stage machinery of rounds and quorum that every model-backed step runs on.
- [`test-fixture`](test-fixture/README.md):
  test fixtures that import no production code and serve every package.
- [`translate-floor`](translate-floor/README.md):
  the floors and pieces a translated slice answer is built from.
- [`translate-lane`](translate-lane/README.md):
  the translate lane that translates every prepared slice afresh and assembles the result.
- [`translate-stage`](translate-stage/README.md):
  the stage in which several models translate a slice independently and a judge roster chooses.
- [`typography`](typography/README.md):
  prose typography with quote normalization and semantic line wrapping through the pinned lint binary.

## Where the docs live

- This file carries the design:
  the standing constraints,
  the English variety rule,
  how a corpus pass is operated,
  and the contract of `repairTranslation`.
- `doc/` beside this file holds the docs about the family as a whole,
  listed under "Where the rest lives".
- A doc about one package sits in that package's `doc/`:
  [`slice-reading/doc/`](slice-reading/doc/pictures.md) and
  [`typography/doc/`](typography/doc/semantic-wrap-sync-spawn.md) each hold one.
- The runbooks,
  decisions,
  troubleshooting notes,
  handovers and plans sit in `doc/` at the repository root;
  [the document map](../../doc/handover/translation-repair-document-map.md) says which to read first.

## Standing redesign constraints

These constraints were written for the finite redesign that was to replace the slice pipeline,
and they supersede historical architecture notes in this file.
That redesign stopped after Candidate M on 2026-09-01 and stays closed under that day's direction decision
(`doc/decision/translation-repair-post-redesign-direction.md` at the repository root),
so no replacement is in progress.
The shipped slice pipeline and the corpus runbook predate the constraints and were not rebuilt to them;
the two bounded errors they name,
`ProductionUnavailableError` and `PublicationUnavailableError`,
exist nowhere in `src`.
Every section after `Standing redesign constraints` describes current or historical implementation
and is non-normative for replacement architecture.
A normal run with working model and publication infrastructure must return one good complete document.
It must not leave its caller with a suspended translation because an auditor,
reviewer,
reviser,
or generic quality Gate timed out,
refused,
or disliked wording.
A node that does not produce usable text cannot receive authority to withhold usable text.

Every model invocation belongs to a statically finite manifest.
No finding,
text change,
round,
nonce,
cache event,
or failed review may add work.
No hidden scheduler may keep creating manifests.
One model plus one canonical substantive prompt may produce at most one provider payload.
Caller cancellation throws exact abort identity and deterministic restart reuses completed nodes.

Model nodes have two kinds.
Preparation-evidence node may produce brief or specification before authorship;
its unusable response contributes nothing and cannot withhold producer work.
Candidate-producer node returns complete candidate under one concrete responsibility,
or it has no effect and prior complete candidate survives byte-for-byte.
Every model stage after preparation is candidate producer.
First adopted producer owns full concrete quality contract,
including fidelity,
completeness,
identity,
grammar,
clear references,
consistent tense,
paragraph relations,
and appropriate register.
Later stages are targeted improvements,
not required rescue for deficient baseline.
Before first candidate exists,
the manifest may try statically named fallback producers once each.
Operational run begins after provider preflight and finite manifest persistence.
First candidate is adopted only after one response yields complete document passing deterministic obligations.
If every planned producer has transport failure or unusable response before first adoption,
command throws bounded `ProductionUnavailableError` with exhausted node identities.
It does not suspend,
wait for human continuation,
automatically create another manifest,
or publish archive text as though repaired.
`ProductionUnavailableError` concerns exhausted candidate producers only;
failed preparation-evidence node never causes it.
This physical infrastructure or model-output failure is only no-output exception.

Runtime quality responsibility belongs to these producing stages,
not a final generic model Gate.
Deterministic adoption checks protect source coverage,
identities,
structure,
links,
media,
formatting,
and publication bytes.
A later producer supplies complete candidate plus exact edit transaction anchored to prior bytes and source evidence.
Deterministic adoption recomputes candidate from transaction,
rejects undeclared changes,
and preserves prior text when transaction or candidate fails.
These checks bound change authority but cannot prove semantic judgment.
Every prototype comparison and every readiness claim must read complete actual output,
not only artifact,
status,
or tally.
Any repeatable concrete defect means producer contract is not ready.

`naturalness`,
`absolute naturalness`,
and similar aggregate labels are not measurements,
thresholds,
floors,
or verdicts in new work.
Use concrete located defect classes such as wrong meaning,
omission,
addition,
identity change,
grammar,
unclear reference,
tense inconsistency,
broken paragraph relation,
and register mismatch.
One wet provider remains normal operation.
Manifest planning builds complete role and fallback roster reachable through whichever provider is wet;
no cross-provider response is correctness dependency.
Finite manifest does not imply serial execution.
Replacement scheduler must dispatch dependency-independent nodes concurrently
up to dated live-measured per-provider and per-model limits.
Dependency edges still serialize work whose prompt consumes prior output.
Concurrency and request-rate limits are separate constraints and must be measured separately.
Synthetic has no local concurrency ceiling since 2026-10-19
(`SYNTHETIC_PER_MODEL_CONCURRENCY` in `package/translation-repair/provider/src/synthetic-client.ts` is unbounded,
like Hyper's):
its rate-limit page of that day (`https://synthetic.new/rate-limits`) describes a weekly credit limit alone
and names no concurrency limit and no five-hour request limit.
Until then production used 5 slots per active model,
measured on 2026-08-30 with four active models,
and a width-10 gpt-oss arm returned 3 HTTP 429 responses.
No arm has measured Synthetic's width since the page changed,
so a refusal at the wire is the signal.
A run routes two roster models to Synthetic,
`hf:Qwen/Qwen3.8-27B` and `hf:moonshotai/Kimi-K3`:
`hf:zai-org/GLM-5.3-Flash` is withheld from Synthetic (`synthetic-withheld` on its card)
and `hf:openai/gpt-oss-120b` left every role (`owner-culled`),
both on 2026-09-24.
Hyper has no local concurrency ceiling;
a live width-64 arm completed 64 of 64 structured calls.
Its 1,000 requests-per-hour account limit remains a separate rate budget.
OpenRouter,
the third provider since 2026-09-03,
has no local ceiling either:
a width-32 chat-completions arm completed 32 of 32 on two models with no refusal,
and the provider states no request-rate limit for paid models.
Amazon Bedrock,
the fourth provider since 2026-09-07,
publishes no requests-per-minute quota and no token quota for the four models whose cards name a Bedrock id
(three in use since `hf:openai/gpt-oss-120b` left every role),
asks for retry with backoff on throttling and a gradual ramp,
and has no local ceiling either.
NeuralWatt,
the fifth provider since 2026-10-20,
serves DeepSeek V4.1 Flash and GLM-5.3-Flash on a free daily energy allowance the package meters itself
(`doc/decision/translation-repair-neuralwatt-provider.md`);
it has no local concurrency ceiling either.
DeepSeek and Xiaomi,
the sixth and seventh providers since 2026-10-22,
are called on the owner's own keys and each for exactly one model
(DeepSeek V4.1 Flash and MiMo V2.6 Flash);
a key left unset leaves that provider unconfigured
(`doc/decision/translation-repair-direct-providers.md`).
Routing walks `PROVIDER_ORDER` (NeuralWatt,
Synthetic,
Bedrock,
DeepSeek,
Xiaomi,
Hyper,
OpenRouter;
NeuralWatt first by the owner's decision of 2026-10-20 so the free energy is spent before anything else,
Bedrock ahead of Hyper by the owner's decision of 2026-09-07,
so the seats both serve spend the expiring credits before the balance the Hyper-only seats run on,
and the two direct routes ahead of Hyper and OpenRouter,
an order open to the owner's veto):
the first provider that serves the model and has budget takes the call,
a saturated provider overflows to the next usable one,
and a dry provider passes the call down the order.
See `doc/troubleshooting/translation-repair-provider-concurrency.md`.
Candidate A2's 432-second intentionally serial run
(`doc/planning/translation-repair-interface-candidates-a-d.md` at the repository root)
measures its implementation,
not provider concurrency capacity.
Measured arms may explicitly require any subset of the providers.
Each node prompt digest binds exact source,
archive,
brief,
prior-candidate bytes or explicit absence marker,
role,
and response contract so restart cannot change fallback into different revision task.
Restart requires same manifest digest and checkpoint and may execute pending nodes only.
Completed,
failed,
unusable,
aborted,
or indeterminate nodes are spent.
Indeterminate transmission may reuse recorded payload but may never resend canonical prompt.
Caller abort bypasses fallback immediately and throws exact `signal.reason`.

Assembly,
atomic write,
or readback failure after candidate exists throws bounded `PublicationUnavailableError`.
It does not suspend,
automatically requeue,
publish partial bytes,
publish archive fallback,
or become quality outcome.

## English variety

The page is written in Canadian English (en_CA),
in spelling and in vocabulary
(owner,
2026-09-25:
"The convention is and should be en_CA.").
Spelling follows the Canadian Press Stylebook and the Canadian Oxford:
colour,
centre,
travelled,
counsellor,
realize,
analyze,
cheque,
program,
advisor.
Vocabulary takes the Canadian word where varieties differ:
gas,
not petrol;
Grade 12,
not Year 12;
math,
not maths.
Dates are written month first (April 29,
March 13,
2024),
never day first.
The rule lives in `HOUSE_POLICY_BLOCK` (`package/translation-repair/house-rule/src/house-policy.ts`),
which every writing and judging sheet carries,
and the rendering glossaries (`rendering-glossary*.ts` in the `rendering-audit` package,
with their tests in
`translate-floor`) seed Canadian forms only;
`package/translation-repair/adjudication/src/canadian-english-policy.unit.test.ts` and
`package/translation-repair/translate-floor/src/canadian-forms-policy.unit.test.ts` guard both.
The sheets reach only slices a lane rewrites,
so a page-assembly pass (`canadianizePage`,
`package/translation-repair/page-check/src/corpus-run/canadian-forms.ts`)
also reads every slice as the page will carry it,
the untouched ones included,
writes every date month first
(day first,
year first,
a range sharing one month,
an abbreviated month),
with the month in full,
no ordinal suffix and the year set off by commas on both sides
("On 4th May 2024 the cat" becomes "On May 4,
2024,
the cat"),
and respells a closed list of words with no second sense
(the -our and -ise families with their inflections,
and pairs such as centre,
counsellor,
enrolment,
grey,
licorice,
judgment and program;
`package/translation-repair/page-check/src/corpus-run/canadian-spelling-words.ts`),
outside front matter,
tags,
attributes,
link destinations,
code,
comments,
identifiers,
paths and spans sealed as the English original;
each change prints `canadian-form-rewritten`.
Where Canadian sources disagree,
McGill's language guidelines decide
(counsellor,
enrolment,
program),
and fulfill and skillful,
on which McGill is silent,
stand as written.
Names,
titles and quoted English keep their own spelling:
a capital mid-sentence,
after a title such as "Mr.",
before another capital,
in a sentence-case heading,
in emphasis or in capitals throughout is kept,
while a capital that opens a sentence or styles a title-case heading is respelled with its capital
("## The Cat’s Favor" becomes "## The Cat’s Favour"),
and a word the slice's original itself writes in English keeps its spelling.
"Mum" becomes "mom" after a possessive and as a capitalised form of address.
The convention covers translated pages only,
not the repository's own prose
(owner,
2026-09-25:
"Translated pages only").

## Operating a corpus pass

This file describes the design.
To run the pipeline over the corpus,
follow [the corpus pass runbook](../../doc/runbook/translation-repair-corpus-pass.md),
which carries the environment,
the launch,
what to watch while it runs,
and how to read the output back once it has exited.

The semantic wrap of shipped text runs in the repository's pinned Rust linter,
`monochromatic-lint`,
which `mise run toolchain:sync` installs as a cargo tool from `toolchain.lock.json`.
A pass started under `mise run` finds it on the PATH,
in the tree's `.cache/toolchain-bin`;
without it,
or with another version,
the first wrap stops the run with a message naming the install commands
(`package/translation-repair/typography/src/semantic-wrap-binary.ts`).

Read-back tools,
none of which spends quota or calls a model:

- `verify-published` reads the published tree back against the artifacts that produced it,
  prints every page that disagrees with what its artifact promised and every artifact with no page,
  and exits 0,
  since a run always ships;
  it exits 2 only for a run it could not read at all.
  The next pass started in the runs directory rewrites each such page from its artifact.
- `meter-report` says what each provider was doing while the run was asking,
  which is availability at the moments the run asked rather than availability in general.
- `run-timing-report` says where the wall clock went,
  splitting each round into work and straggler waiting,
  and reports achieved rather than configured concurrency.
- `spend-report` prices the metered seats against a rate table carrying its date,
  and counts subscription seats without pricing them;
  a call written as a reckoning rather than reported by the wire is named per seat (ledger P14).
- `cap-census` re-reads the completion cap rule over pass-run logs before a launch,
  per seat and provider,
  with how many capped calls ran to the cap and whether they carried content,
  and flags a card still on the pooled placeholder (ledger P10).
- `ledger-report` says who produced each candidate and how often judges chose it.
  Its `--model` view prints corpus wording,
  so it must not be pasted anywhere.

The runbook carries the exact invocation and the expected output for each,
including what each one prints when the run recorded nothing for it,
which is never the same as the run having done nothing.

The pass also prints,
beside each settled entry's `TALLY` line,
`DESTINATIONS <id> at=<time> attempt=<n> source=N page=M dropped=K`
(the time the line was rendered and the attempt it belongs to,
which every per-entry line of the pass carries,
`doc/configuration.md`,
"Bounding one run"):
how many distinct web addresses the source page links to,
how many the published page carries,
and how many of the source's the page lacks.
A source destination the archive rendered another way is not lacking when the page carries the archive's rendering
(`doc/decision/translation-repair-rewritten-destination.md`):
the line then ends with `destinations-archive-rendering`,
and with `destinations-both-renderings` when the page carries the original's and the archive's for one reference.
The addresses themselves go to the run log at warn,
one line per address reading `entry <id>: page drops source destination <address>`
with the slices that carry it in the original,
the archive and the shipped text,
never to stdout.
A dropped destination from a wording both deciders approved is a finding,
not a late publish rewrite.

A page that ships with a defect also prints `DEFECTS <id> at=<time> attempt=<n> checks=<a>,<b>`
after its `DESTINATIONS` line.
The page ships regardless (`package/translation-repair/doc/design-commitments.md`),
and each check names what it found:
`archive-original`,
`contributor-names`,
`destinations`,
`front-matter` and `headings`
are the publish-time content checks,
`no-valid-wording` names the slices where the archive's own wording stayed and says why for each:
no wording passed the deterministic rule,
the final round could not seat two judges and no wording two judges chose passed it,
or the slate seated two or more judges and declined every wording
(a record written before artifact generation 22 says it records no reason),
and `page-agreement` names a page that does not carry what its artifact says ships,
which is a defect in page assembly,
not in the text.
Each defect's message is in the run log at warn,
under `publish: shipping with defect <check>:`,
and names ids,
counts and slice indices only.
A page that does not parse still refuses.
The neutral pronoun the sources write as `TA`,
`Ta` or `ta` renders as singular they:
the declared-identity pronoun line counts all three spellings (`identity-context.ts`),
the house rule says what English makes of it,
and a translation that leaves it standing fails the deterministic slice rule (`translate-neutral-pronoun.ts`),
which names the spelling and its count to the model that wrote it.
Measured over the pinned corpus,
that rule flags one archive (a rewrite keeping `TA`) and 15 sources.
A different rule protects a source-only passage:
whole-document coverage must call it absent,
then page shortfall or a destination missing from target admits translation.
A source-only passage the coverage round finds carried inside the neighbouring paired slice's archive span
is folded into that neighbour at the admission (`corpus-run/insertion-carried-fold.ts`;
where the archive spread it over both neighbours,
into the one holding the larger share),
so both lanes write it as part of the slice that carries it and the publish guard has nothing to find.

A source passage still unfilled after the single translation round and its one follow-up
ships its source text under a note,
not as a refusal and no longer as a silent gap
(owner ruling of 2026-10-23,
"Source text + note",
[the commitment](doc/design-commitments-shipping-and-the-archive.md)):
the artifact records it (`lanes.translate.unfilled`),
the log names each one (`corpus-run/pass-entry.ts`),
and the page carries the original text of the passage after one block-quote note per contiguous run,
`> This passage has not been translated yet. The original text follows.`
The text is each slice's original as the artifact records it,
the slices of a run joined by one blank line;
text between slices is not carried.
Where the composed page does not parse as MDX with the original as recorded,
the publisher writes the runs that do not parse as literal text,
then every run,
and last ships the page without the original under the defect `page-source-unparseable`,
with a warn line for each step (ledger B832).
A run written as literal text is compared with its source,
as a whole,
and block by block where a footnote that the two readings end at different places reaches a block,
and the page ships a defect that names the slices and no text:
`source-reads-differently` for a block that shows other text than the source,
`source-reading-undecided` for a block the comparison cannot decide (a footnote in a block quote or a list item
that the two readings end at different places),
and `source-reading-not-compared` for a run the comparison could not be made for.
A run always ships,
so none of them stops a page
(the owner,
2026-10-25,
"Add the comparison";
`page-check/README.md`).
A later run replaces the text once a wording is backed.
Under the no-loop design of 2026-09-01 an insertion is recovered supplementary content,
so the entry still ships rather than failing for want of it.
A passage admitted for translation when a provider outage leaves every translator unheard is different:
the entry stops `INCOMPLETE` and keeps its slice cache for a later run.
A stored prompt payload the run cannot read or write stops the entry `INCOMPLETE` too,
its tally line naming what refused;
`PromptPayloadStoreError` (`package/translation-repair/stage/src/prompt-payload-store.ts`) says what to do about one.

## Contract

The core export is the batch driver over pure stage functions,
declared in the `repair` package:

```ts
import { repairTranslation, } from '@monochromatic-dev/translation-repair-repair';

const result = await repairTranslation({
  client,
  sourceText,
  targetText,
  models: {
    criticModelIds,
    panelModelIds,
    editorModelIds,
    judgeModelIds,
    checkerModelIds,
  },
  signal,
  perCallTimeoutMs,
  stageSettings,
},);
```

- `client` is an injected model client (`createSyntheticClient` or any
  `ModelClient` implementation);
  the library performs no IO of its own.
- `stageSettings` is the value `readStageSettings` (declared in the `stage` package) returns;
  `repairTranslation` requires it.
  The optional `sliceCharBudget`,
  `sliceCache` and `overlap` complete the parameters.
- `perCallTimeoutMs` is the deadline for each model exchange.
  It has no default:
  every caller in the package states its own.
- `models` names the role roster:
  critic fan-out,
  provenance-blind adjudication panel,
  editors,
  selection judges,
  and resolution checkers.
  A stage that gathers voices (`gatherStageVoices` in `package/translation-repair/stage/src/stage-quorum.ts`)
  needs a quorum of at least half its roster,
  rounded up;
  when the router has refused seats for want of a wet provider and fewer than that remain,
  the quorum is half the reachable seats,
  rounded up,
  and at least two (`package/translation-repair/stage/src/stage-reachable-quorum.ts`).
  Each round asks a window of what quorum still needs plus one spare seat,
  from a bench rotated by the prompt (`package/translation-repair/stage/src/stage-fanout-window.ts`),
  and a window seat the router refuses for want of a wet provider
  hands its place in that round to the next seat not yet asked (`runGatherRound` in
  `package/translation-repair/stage/src/stage-round.ts`);
  up to three retry rounds (`STAGE_RETRY_ROUNDS`) ask the seats not yet asked before the ones lost,
  and never re-ask a seat the router refused,
  nor a seat whose answer arrived but could not be read,
  since the same prompt returns the same bytes.
  Once the rounds end,
  one recovery round re-asks every seat whose answer could not be read,
  whichever round it came in (ledger P2),
  with a note naming what happened:
  a reply the length limit cut,
  or one off the shape asked for
  (`package/translation-repair/stage/src/recovery-nudge.ts`,
  ledger P10).
  A stage still short of quorum proceeds on what it heard and records the shortfall as findings.
  The six stages that record every seat's own outcome
  take the same window and retry rounds through `runWindowedRounds`
  (`package/translation-repair/stage/src/stage-windowed-rounds.ts`),
  with no recovery round.
  An optional `editorRuleAddendum` splices one extra machine-enforced
  rule line into the editor prompt for calibration experiments.
- No single model decides the repaired text.
  Every editor in `editorModelIds` rewrites the chunk independently,
  each proposal passes the same deterministic apply gate,
  and judges drawn from `judgeModelIds` choose what ships.
  Selection seats the whole judge roster,
  producers included,
  and counts a judge's ballot for its own candidate at half weight;
  every other ballot it casts carries full weight,
  including one for another producer's candidate.
  A winner needs two ballots,
  whatever their weight (the owner's rulings of 2026-10-25 removed every weight floor),
  so one judge alone never selects a candidate and the half weight of a self-vote only ranks candidates.
  `assertJudgeableEditorRoster` (`package/translation-repair/assembly/src/repair-contract.ts`) no longer requires judges
  outside the editor roster:
  the ruling of 2026-08-15 allows self-judging at reduced weight instead.
  It refuses a roster that repeats an editor or a judge,
  seats no editor,
  or seats fewer than two judges,
  since such a roster could not name a winner however it voted.
  `checkerModelIds` should exclude every editor,
  so nothing certifies text it wrote.
- Judging runs at two granularities.
  Per envelope,
  the best fix for each individual issue can win even when the model that wrote it botched the rest of the chunk;
  the winners are assembled into a composite candidate.
  Per chunk,
  whole candidates compete,
  including that composite,
  which is the only level at which coherence across envelopes is visible.
  The composite has to win on its merits rather than being adopted by construction.
  When judges decline,
  the strongest editor patch ships anyway:
  falling back to the untouched translation would turn a disagreement about wording into a lost repair.
- The result is never an unqualified "corrected translation":
  `repairedText` ships with a completion status (`repaired`,
  `unchanged`,
  or `blocked-non-translation`),
  every adjudicated issue with its resolution fate,
  and the alignment and stage findings in scorecard-stable wording.
  When no candidate demonstrably beats the input,
  the input is returned unchanged with its unresolved issues.
- Every issue record also carries **what was written** for it,
  so repair quality can be judged apart from whether the issue was real.
  `repairRegions` records replaced regions rather than per-issue repairs,
  because envelopes merge overlapping and touching evidence,
  so one replacement can serve several accepted issues and fix only some;
  each region names every issue it serves.
  `repairDisposition` says what became of that repair in the returned document (`shipped`,
  `not-selected`,
  `withdrawn`,
  or `no-region`),
  and is decided after document-level blocking and the naturalness lane,
  neither of which any single slice can see.
  `refined` marks a slice the naturalness lane rewrote afterwards,
  which is where the recorded replacement stops being the returned wording,
  and `finalSliceText` carries that wording exactly there.
- Translation policy files (register,
  terminology,
  tense discipline) are deliberately open;
  the system functions without them using conservative defaults.

## Where the rest lives

The docs about the family as a whole sit beside this file in `doc/`,
one file per subject,
except the first,
which sits in the `slice-reading` package:

- [Reading the pictures a document shows](slice-reading/doc/pictures.md):
  the deterministic reader,
  the model readers (six when every provider is wet),
  what is sent and what is not.
- [Evidence beside a slice](doc/slice-context.md):
  the neighbouring passages,
  the pages the original cites,
  what is folded out of candidate and archive text at intake.
- [Repetition and the site's grammar](doc/repetition.md):
  the repetition the pipeline introduced and how it is caught,
  and why the site's grammar is not this family's.
- [Design commitments](doc/design-commitments.md):
  what the pipeline promises and refuses,
  with the measurements behind each.
- [Configuration](doc/configuration.md):
  credentials,
  where a run writes,
  bounding,
  pacing and choosing a run,
  probe tools,
  kill and relaunch,
  which corpus a run reads,
  pooling artifacts,
  schema generations.
- [Current roster changes](doc/roster-changes.md):
  the one-card workflow and every seating and unseating with its date.
- [Provider availability, measured](doc/provider-availability.md):
  latency,
  running out of budget,
  how much of the time each provider was there.
- [Deciding who fills a seat](doc/seats-and-calibration.md):
  calibrations,
  standings,
  editor and refiner credit.
- [Status](doc/status.md):
  the milestone and generation history since 2026-07.
- [Redaction timing](doc/redaction-timing.md):
  when redaction happens.
- [The audit ledger](doc/audit-ledger.md):
  every finding of the whole-package audit of 2026-09-27 and after,
  with its measurement,
  its fix and its guard,
  and every mistake made while fixing.
- [Preventing this family's mistakes](doc/mistake-prevention.md):
  the ledger's mistakes grouped by family,
  the rule that prevents each and what enforces it,
  and the checklist before a run launches.
- [Why the semantic wrap starts its linter synchronously](typography/doc/semantic-wrap-sync-spawn.md):
  the one synchronous process start in the family and the lint suppression that covers it.

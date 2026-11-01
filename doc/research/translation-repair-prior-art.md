# Translation repair prior art

## Scope and conclusion

This note compares the current `package/module/translation-repair` (moved on 2026-10-18 to the packages under
`package/translation-repair`) worktree with primary sources:
standards,
 published papers,
 and first-party project documentation.
It is an architecture survey,
 not a patent search or proof of novelty.
A missing match means only that no match appeared in the sources reviewed here.

The package is not a new task category.
Automatic post-editing,
 error-guided LLM refinement,
 fine-grained translation error detection,
multi-agent evaluation,
 conservative candidate selection,
 and perturbation-based evaluation all have direct precedent.
A particularly close published precedent is [MQM-APE](https://aclanthology.org/2025.coling-main.374/),
which assigns LLMs evaluator,
 post-editor,
 and pairwise-verifier roles to retain only error annotations whose repairs
improve a translation.
[TEaR](https://aclanthology.org/2025.findings-naacl.218/) is another close match through its
translate,
 estimate,
 refine,
 and improvement-selection stages.

What appears unusual is the package's conservative composition:
provenance-blind critic adjudication feeds mechanically derived editable envelopes;
the editor returns patches bound to envelope hashes;
deterministic code rejects stale or out-of-scope changes;
separate checkers assess fixes and regressions;
and the unchanged input wins a tie.
The surveyed translation systems contain parts of this design,
but none combines all of these controls.

## Package architecture examined

The comparison follows the contract in `package/module/translation-repair/README.md` (moved on 2026-10-18 to
`package/translation-repair/README.md`) and the current pipeline in:

- `package/module/translation-repair/src/repair-stages.ts` (moved on 2026-10-18 to
  `package/translation-repair/repair/src/repair-stages.ts`) and
  `package/module/translation-repair/src/adjudicate-model.ts` (moved on 2026-10-18 to
  `package/translation-repair/shared-contract/src/adjudicate-model.ts`) for critic aggregation and panel decisions.
- `package/module/translation-repair/src/patch-model.ts` (moved on 2026-10-18 to
  `package/translation-repair/shared-contract/src/patch-model.ts`),
  `package/module/translation-repair/src/edit-wire.ts` (moved on 2026-10-18 to
  `package/translation-repair/repair/src/edit-wire.ts`),
  and `package/module/translation-repair/src/apply-patch.ts` (moved on 2026-10-18 to
  `package/translation-repair/page-pass/src/apply-patch.ts`) for envelopes and guarded application.
- `package/module/translation-repair/src/repair-edit-stages.ts` (moved on 2026-10-18 to
  `package/translation-repair/repair/src/repair-edit-stages.ts`) and
  `package/module/translation-repair/src/select-candidate.ts` (moved on 2026-10-18 to
  `package/translation-repair/adjudication/src/select-candidate.ts`) for checking and candidate selection.
- `package/module/translation-repair/src/non-translation-evidence.ts` (moved on 2026-10-18 to
  `package/translation-repair/archive/src/non-translation-evidence.ts`) and
  `package/module/translation-repair/src/repair-translation.ts` (moved on 2026-10-18 to
  `package/translation-repair/repair/src/repair-translation.ts`) for local degradation and document blocking.
- `package/module/translation-repair/src/derive-seeds.ts` (moved on 2026-10-18 to
  `package/translation-repair/repair/src/derive-seeds.ts`),
  `package/module/translation-repair/src/seeded-error.ts` (moved on 2026-10-18 to
  `package/translation-repair/lane-contest/src/seeded-error.ts`),
  and `package/module/translation-repair/src/repair-benchmark.ts` (moved on 2026-10-18 to
  `package/translation-repair/repair/src/repair-benchmark.ts`) for planted-error evaluation.

The central distinction is ownership of decisions.
Models propose evidence,
 judgments,
 edits,
 and verdicts.
Code validates anchors,
 derives edit scope,
 applies patches,
 and selects according to fixed ordering rules.

## Direct translation precedents

### Automatic post-editing and error-guided refinement

Automatic post-editing is established as the task of correcting output from a black-box MT system.
The [WMT 2022 findings](https://aclanthology.org/2022.wmt-1.5/) describe the eighth round of its shared task,
so the package's source-plus-translation-to-post-edit contract is direct prior art.

Several LLM systems also use the same critique-to-repair shape:

- [Ki and Carpuat](https://aclanthology.org/2024.findings-naacl.265/) prompt and fine-tune LLMs to post-edit MT
  from MQM error annotations.
  Their results also caution that fine-grained feedback is not automatically useful under prompting alone.
- [LLMRefine](https://aclanthology.org/2024.findings-naacl.92/) uses a learned feedback model to pinpoint defects,
  then searches iteratively over LLM-proposed edits,
   including for machine translation.
- [TEaR](https://aclanthology.org/2025.findings-naacl.218/) separates translation,
   estimation,
   refinement,
  and autonomous improvement selection.
- [MQM-APE](https://aclanthology.org/2025.coling-main.374/) is the closest role-level precedent:
  an evaluator proposes error annotations,
   a post-editor repairs from each annotation,
  and a pairwise verifier filters errors that do not produce an improvement.

These precedents make it unsafe to claim that critique,
 post-edit,
 and verification are new.
The package differs in treating accepted evidence as authority for a hard edit boundary,
rather than supplying feedback to an otherwise free-form generator.

### Earlier systems named translation repair

[TransRepair](https://doi.org/10.1145/3377811.3380420) combines mutation and metamorphic testing to find
translation inconsistencies,
 then performs black-box or grey-box repairs from translations of related mutants.
Its candidate ranking includes the original translation and leaves it untouched when it outranks repair sources.
Its local replacement procedure also uses word alignment,
 numeric-type checks,
 parsing checks,
and a post-repair consistency test.
This is direct precedent for the name,
 test-and-repair framing,
 unchanged candidate,
 and constrained local repair.
It targets context-similar lexical inconsistency,
 not general MQM-like document repair.

Document repair also predates this package.
[Context-Aware Monolingual Repair](https://aclanthology.org/D19-1081/) post-edits sequences of sentence translations
for contextual consistency,
while [Lexical Translation Inconsistency-Aware Document-Level Translation Repair][ltidtr]
locates inconsistent items,
 supplies candidates,
 and repairs document translations.
Their repair scope is narrower and their controls are learned rather than mechanically evidence-bounded.

## Adjacent component precedents

### Error taxonomies and anchored findings

The [MQM error typology](https://themqm.org/error-types-2/typology/) organizes translation defects into hierarchical
dimensions including accuracy,
 terminology,
 linguistic conventions,
 style,
 locale conventions,
audience appropriateness,
 and design or markup.
The package's taxonomy is MQM-derived and adds pipeline states rather than reproducing MQM exactly.

Error localization by models is also established.
[GEMBA-MQM](https://aclanthology.org/2023.wmt-1.64/) prompts GPT-4 to mark reference-free translation error spans,
and [xCOMET](https://aclanthology.org/2024.tacl-1.54/) combines sentence scoring with categorized error-span detection.
The package goes further operationally:
quotes must resolve to immutable node IDs and offsets,
and invalid claims are discarded before adjudication or editing.
The span idea is direct prior art;
the evidence object as an enforced patch capability is not present in these metrics.

### Multi-model adjudication and checking

[M-MAD](https://aclanthology.org/2025.acl-long.351/) decomposes MQM-style evaluation into dimensions,
uses multi-agent debate,
 and synthesizes dimension-specific results into a final MT judgment.
This is direct precedent for multi-agent translation evaluation and aggregation.
MQM-APE supplies a closer precedent for a separate verifier after editing.

The package's fixed panel,
 removal of critic provenance before panel review,
quorum retry behavior,
 and separate checker roster appear to be implementation-specific choices.
Different model families and separate calls do not by themselves prove statistically independent errors,
so the architecture should be described as ensemble-checked rather than independently verified.

### Minimal editing and unchanged-baseline selection

Over-correction is a documented APE failure mode.
[Effort-Aware Neural Automatic Post-Editing](https://aclanthology.org/W19-5416/) conditions repair on predicted
required effort,
[Quality Estimation-Assisted Automatic Post-Editing](https://aclanthology.org/2023.findings-emnlp.115/) integrates
word-level quality estimation to reduce unnecessary changes,
and [QE-assisted constrained decoding](https://aclanthology.org/2025.naacl-short.77/) uses word-level QE during
decoding to preserve minimal editing.

Candidate ranking is also standard in MT.
[Neural-metric minimum Bayes risk decoding](https://aclanthology.org/2022.tacl-1.47/) selects hypotheses by
estimated quality rather than model probability.
TEaR selects refinements while preserving a quality baseline,
and TransRepair explicitly lets the original translation defeat mutant-derived repairs.
The package's unusual part is therefore not retaining a no-op candidate.
It is the exact deterministic policy:
integrity first,
 then confirmed high-severity fixes,
 regression count,
 preservation,
and finally an unchanged-wins-ties rule.

### Non-translation, hallucination, and derivability

Rejecting unrelated bilingual pairs has precedent in bitext filtering.
[Margin-based parallel-corpus mining](https://aclanthology.org/P19-1309/) scores whether sentence pairs belong
together using multilingual embeddings and relative margins.
MT hallucination work similarly treats bad outputs as detached from their sources:
[Dale et al.](https://aclanthology.org/2023.acl-long.3/) use source contribution and cross-lingual similarity,
while [HalOmi](https://aclanthology.org/2023.emnlp-main.42/) supplies human annotations for partial and full
hallucinations and omissions at sentence and word level.

The current worktree applies this concern at repair time.
Per-slice non-translation votes can degrade locally;
standing votes block the whole document only when they dominate its target characters;
and anchored content critique can deterministically contradict a block vote.
The reviewed sources establish pair validity and source-grounding detection,
but not this combination of ensemble voting,
 contradiction screening,
 and local-versus-document control flow.

The benchmark's derivability probe is adjacent to source-grounding work but serves a different purpose.
It attributes a missed planted restoration to underivable source information instead of an editor failure.
That is an evaluation-accounting rule,
 not a hallucination detector,
and no direct counterpart appeared in the reviewed repair systems.

### Seeded errors and challenge sets

Controlled perturbation is established for translation evaluation.
[DEMETR](https://aclanthology.org/2022.emnlp-main.649/) builds minimal pairs with linguistic perturbations to test
metric sensitivity,
and [ACES](https://aclanthology.org/2022.wmt-1.44/) tests MT metrics against accuracy phenomena ranging from
character changes to discourse and real-world knowledge.
TransRepair likewise generates mutations to test and repair consistency.

The package's seeded omissions therefore have direct methodological precedent.
Its narrower distinction is end-to-end repair accounting:
it knows the planted region,
 tests detection and restoration,
and records where the critic,
 adjudicator,
 editor,
 or derivability decision lost the seed.
Seeded performance establishes behavior on the chosen corruption distribution;
it does not establish precision or safety on naturally occurring errors without human-graded evaluation.

### CAT and translation quality assurance

Established CAT and translation QA tooling already combines deterministic and configurable checks.
[Okapi CheckMate](https://www.okapiframework.org/wiki/index.php/CheckMate) checks bilingual documents for missing
translations,
 code differences,
 length anomalies,
 whitespace,
 patterns,
 corrupted characters,
 and repeated words.
Its [configuration](https://okapiframework.org/wiki/index.php/CheckMate_-_Quality_Check_Configuration) also supports
terminology,
 blacklists,
 severity,
 false-warning handling,
 and reports.

Translate Toolkit's [pofilter][pofilter]
runs checks over PO,
 XLIFF,
 and TMX files,
 emits failing units for correction,
and can apply automatic corrections where supported.
Its [test catalogue](https://docs.translatehouse.org/projects/translate-toolkit/en/latest/commands/pofilter_tests.html)
includes variables,
 format strings,
 tags,
 numbers,
 whitespace,
 unchanged text,
 untranslated text,
 and terminology-like
must-translate or must-not-translate checks.
These tools are direct precedent for deterministic integrity gates and issue reports,
but not for open-ended semantic repair by model ensembles.

## Architectural analogues outside translation

The strongest analogues for the package's guarded-edit machinery come from annotation and patch standards:

- The [W3C Web Annotation Data Model](https://www.w3.org/TR/annotation-model/) defines text quote selectors with
  exact text plus prefix and suffix,
   text position selectors with start and end offsets,
  and resource states to disambiguate changing targets.
  This closely parallels redundant evidence anchors against a particular document state.
- [JSON Patch](https://www.rfc-editor.org/rfc/rfc6902.html) defines location-addressed operations,
  a `test` precondition,
   termination on a failed operation,
   and atomic HTTP PATCH behavior.
  Its own example combines a patch with `If-Match`,
   making it a direct analogue for rejecting stale edits.
- [SARIF 2.1.0][sarif] represents findings with physical
  regions and snippets,
   artifact hashes,
   stable fingerprints,
   and fixes containing deleted regions and inserted content.
  SARIF records these fields but does not require a consumer to enforce the package's hash-and-envelope gate.

These standards mean exact anchors,
 hashes,
 regions,
 and guarded patches are established engineering techniques.
What appears unusual in translation repair is binding them together so model evidence determines edit authority
and deterministic application code revalidates that authority.

## Established versus unusual

Established:

- Automatic post-editing and document-level repair.
- MQM-like error categories and model-produced error spans.
- Error-guided LLM refinement and evaluator,
   editor,
   verifier role separation.
- Multi-agent translation evaluation and candidate ranking.
- Preserving correct text,
   controlling over-correction,
   and allowing no edit.
- Hallucination,
   omission,
   and bilingual-pair validity detection.
- Perturbation and seeded-error evaluation.
- Deterministic CAT integrity checks and limited autocorrection.

Unusual as a combined architecture:

- Provenance-blind adjudication between critic fan-out and editing.
- Deterministically validated evidence that becomes the sole source of editable envelopes.
- Patch operations bound to immutable base text and hashes,
   with stale or out-of-envelope edits rejected.
- A separate resolution and regression check followed by a fixed lexicographic selector.
- An unchanged candidate that wins exact ties and retains unresolved issue provenance.
- Non-translation blocking that can be contradicted by anchored content evidence and otherwise degrades locally.
- Seed-level failure attribution that distinguishes underivable source content from editor failure.

The defensible positioning is:
**a conservative composition of established MT error annotation,
 automatic post-editing,
 ensemble evaluation,
and perturbation benchmarking,
 augmented with software-style evidence-bound patch application and an explicit
unchanged-baseline safety policy.
**
Claims that it is the first translation-repair system,
 that model-family diversity provides independence,
or that seeded benchmarks prove real-world safety would exceed the evidence reviewed here.

[ltidtr]: https://aclanthology.org/2023.findings-acl.791/
[pofilter]: https://docs.translatehouse.org/projects/translate-toolkit/en/latest/commands/pofilter.html
[sarif]: https://docs.oasis-open.org/sarif/sarif/v2.1.0/os/sarif-v2.1.0-os.html

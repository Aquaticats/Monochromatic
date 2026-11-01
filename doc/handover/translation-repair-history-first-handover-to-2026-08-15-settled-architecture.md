# Translation repair history, first handover document: the settled architecture

The design of the pipeline as settled after the adversarial second-model critique:
stages as pure functions,
the fixed adjudication panel,
patch operations inside editable envelopes,
and candidate selection that always lets the unchanged text compete.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Settled architecture

Deterministic core plus model stages,
revised after an adversarial second-model critique (pi,
gpt-5.6-sol):

- Stages are pure `(state, responses) -> newState`;
  drivers are the impure shell (functional core,
  imperative shell).
  Batch driver = `repairTranslation`;
  interactive driver adds human steering as typed operations
  (approve/strike issue,
  correct alignment,
  lock wording,
  force verdict)
  applied to serialized checkpoints at stage boundaries.
  Mid-stage steering = abort in-flight calls (`AbortSignal` on every client call),
  edit checkpoint,
  rerun stage.
- Critic fan-out across vendor families;
  refusals handled reactively (schema failure or refusal-shaped valid output -> reroute to another family),
  never predicted or pre-annotated.
- Adjudication by a fixed provenance-blind panel with vote states
  (`supported`/`unsupported`/`ambiguous`/`source-defect`/`abstain`) and quorum.
  Never a variable electorate of non-proposers (selection defect:
  consensus shrinks the electorate).
- Canaries (planted errors) are calibration probes feeding routing and vote weights only;
  never hard gates;
  corroborated findings survive a proposer's canary miss.
- Issues keep atomic claims;
  clustering only proposes merges,
  an adjudicator disposes.
- Editors return patch operations against base hashes inside declared editable envelopes;
  never whole rewritten chunks.
  Deterministic guards check region change;
  a semantic adjudication call checks issue resolution
  (region changed does not mean issue resolved).
- The unchanged translation always competes in candidate selection;
  selection is lexicographic (integrity,
  high-severity resolution,
  no regressions,
  preservation)
  with pairwise preference only as tie-breaker;
  original returned with unresolved issues when nothing demonstrably beats it.
- Output contract:
  repaired candidate plus accepted,
  rejected,
  and unresolved issues plus completion status;
  never an unqualified "corrected translation".
- Issue states beyond MQM:
  `suspected-source-error` (blocks correction,
  preserves safer translation),
  `interpretive-ambiguity`,
  `alignment-error`,
  `footnote-conflict`,
  and a `policy` family
  (editing-guide violations such as suicide-method detail).
- Document dossier (facts only,
  no instructions):
  entities,
  recurring term renderings,
  quotations,
  footnote graph.
  Translation policy files are static,
  human-written,
  optional inputs;
  never generated per passage (content sensitivity does not correlate with document class).
- Verdicts are chunk-level with a document rollup (translation quality varies within one document).
- Scorecard harness gates everything:
  per-model per-role recall/precision/refusal/schema-compliance on seeded errors decides
  panel sizes,
  weights,
  and routing.
  Ensemble recall ceiling is the go/no-go number.

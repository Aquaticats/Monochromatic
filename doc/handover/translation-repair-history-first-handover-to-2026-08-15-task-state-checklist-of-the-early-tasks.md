# Translation repair history, first handover document: the task state checklist of the early tasks

The checklist of the early tasks with what each delivered:
scaffold,
document model,
issue model,
model client,
benchmark harness,
corpus reads,
chunking,
tolerant parsing,
and the repair-phase tasks as they stood when milestone two began.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Task state

1. Scaffold package:
   done.
2. Document model and segmentation core:
   done
   (`parse-document.ts`,
   `document-node.ts`,
   `footnote-graph.ts`,
   `footnote-model.ts`,
   `front-matter.ts`,
   `parse-mdx.ts`;
   20 tests;
   zero lint findings).
3. Issue model and span/anchor validation:
   done
   (`issue-taxonomy.ts`:
   closed MQM-derived category union with `policy` and
   `extension` families,
   severities including `neutral`,
   runtime guards;
   `issue-model.ts`:
   `SpanAnchor` with side/nodeId/nodeHash/absolute offsets/exact
   quote,
   zero-width insertion anchors,
   atomic multi-span `IssueClaim`,
   deterministic `computeIssueClaimId` over canonical serialization;
   `validate-issue.ts`:
   rejection-as-data `validateIssueClaim` with kinds
   anchorless-issue,
   malformed-offset,
   inverted-span,
   unknown-node,
   stale-node-hash,
   span-outside-node,
   quote-mismatch;
   fail-fast per span,
   all spans reported independently;
   adversarial tests over parsed cat-themed fixtures).
   Claims carry no proposer provenance;
   the shell tracks that outside the claim.
4. Synthetic model client:
   done
   (`synthetic-catalog.ts` verified catalog and request-weight estimator;
   `synthetic-transport.ts` injectable transport seam,
   fetch receives only
   locally owned values plus a dependent signal;
   `synthetic-quota.ts` typed `/v2/quotas` snapshots;
   `completion-shape.ts` protocol parsing with first-class `refusal` field;
   `refusal.ts` deterministic opening-window marker scan;
   `chat-contract.ts` request/outcome types;
   `model-content.ts` fence and think-block handling,
   tolerant JSON parse;
   `synthetic-client.ts` `createSyntheticClient` with per-model `p-limit(1)`,
   mandatory `AbortSignal`,
   outcome-as-data `chatJson`;
   boundary-verified against the live API twice).
5. Seeded-error benchmark harness and scorecard (exit criterion of milestone one):
   code landed (`4cd25ae95`),
   first real run in progress.
   `seeded-error.ts` (deterministic planting,
   region tracking,
   hit tolerance 30),
   `derive-seeds.ts` (runtime omission derivation from longest unique sentences,
   min 40 chars,
   so no UNLICENSED content is committed),
   `critic-wire.ts` (quote-based wire format,
   `CRITIC_RESPONSE_FORMAT` JSON schema,
   quote-to-anchor resolution failing closed on absent/ambiguous/cross-block
   quotes,
   final `validateIssueClaim` gate),
   `critic-prompt.ts` (strict system prompt with closed vocabularies),
   `scorecard.ts` (pure aggregation;
   per-model schema-ok,
   refusal,
   effective
   seeded recall;
   `ensembleRecall` over the entry+seed universe is the go/no-go
   number;
   precision is deliberately not graded against seeded truth because the
   MT-seeded corpus carries genuine errors),
   `benchmark.ts` (`runCriticBenchmark`:
   entries and models parallel,
   HTTP failures as attempt data,
   aborts propagate,
   and one
   fresh-deadline second attempt per transient-shaped failure via
   `attempt-retry.ts`).
   First real run:
   entries Huasheng and DarlinChit,
   2 derived omission seeds each,
   all 7 models;
   driver script `run-benchmark.ts` in the session scratchpad;
   results land in `benchmark-result.json` there and must be copied into this doc.
6. Pinned-SHA corpus reads from the user's local clone:
   done (`corpus-source.ts`;
   see corpus facts).
7. Section chunking with total automatic alignment:
   done
   (`chunk-document.ts`,
   commit `2f9b2c8af`).
8. Tolerant parsing:
   done (commit `5762f4748`).
9. Xu_Yushu polish loop:
   done (see status narrative).
10. Truncation-shaped retry:
    done (`attempt-retry.ts`).
11. Claim aggregation into merge-proposal clusters:
    done
    (`aggregate-claims.ts`,
    commit `8818f27fd`:
    dedupe by
    `computeIssueClaimId`,
    transitive same-family span-overlap clustering
    via work-stack walk,
    zero-width anchors expand by
    `CLUSTER_ANCHOR_TOLERANCE` 30,
    deterministic cluster ids over sorted
    member ids,
    clusters in document order;
    nine unit tests,
    lint 0/0.
    `format:oxlint` is the auto-fixer for the vertical stylistic rules;
    run it instead of hand-splitting arguments).
12. Adjudication panel with vote states and quorum:
    blocked by 11.
13. Patch-operation model,
    envelopes,
    deterministic apply:
    parallel-ready.
14. Editor stage (patch-op wire):
    blocked by 12 and 13.
15. Resolution check,
    no-regression gate,
    candidate selection:
    blocked by 14.
16. `repairTranslation` end-to-end:
    blocked by 15.
17. Milestone-two benchmark (seeded repair rate):
    blocked by 16.

# Translation repair history, first handover document: the questions left open in the early design

The questions the early design left open:
the output consumer,
the policy files,
the benchmark focus,
unrelated text pairs,
model-driven input fixing and document chunking,
each with the evidence that deferred or settled it.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Deliberately open

- Output consumer (wiki PRs,
  files,
  UI):
  deferred until after the pure fn proves itself.
- User possesses the policy files;
  the system must function without them.
- Benchmark focuses on memorial texts first (user decision).
- Unrelated text pairs (user probe:
  zh cat story vs "Meow meow meow"):
  `accuracy/non-translation` is in the taxonomy (commit `d4dabc283`) and
  the critic prompt reports one critical instance for wholly unrelated
  pairs.
  Verified live on the invented pair:
  GLM-5.2,
  gpt-oss-120b,
  and
  Qwen3.6-27B each emitted exactly one accuracy/non-translation/critical
  issue.
  Qwen's failed to ANCHOR (degenerate repetitive gibberish makes
  every short quote ambiguous),
  so the future rollup must treat
  non-translation as a document-level verdict where wire-level ensemble
  agreement suffices and anchoring is best-effort;
  ensemble-agreed
  critical non-translation blocks repair and returns the input unchanged.
- Model-driven input fixing (user:
  parse phase may "optionally" use LLMs to
  fix source and translations before continuing):
  deferred by evidence,
  not
  rejected.
  Deterministic tolerance (comment masking + markdown fallback)
  covers every document in the pinned corpus with zero fallbacks needed;
  `RepairDocument.parseFindings` is the designed trigger seam.
  Build the
  LLM fixer only when a corpus commit produces a document whose findings
  show real damage (an `mdx-downgraded` finding is the signal to watch),
  and gate any fixed text on a strict re-parse plus a content-preservation
  check before it replaces the input.
- Document chunking (zh-to-en aligned sections) LANDED
  (commit `2f9b2c8af`:
  `chunk-document.ts`,
  `alignDocumentSections`
  pairing mirrored structures by index,
  degrading to proportional
  monotone merging with findings,
  never refusing).
  Motivation stands:
  run 4 proved whole large documents exceed the 65_536-token output
  ceiling on thinking models,
  and small units complete in ~30 s.

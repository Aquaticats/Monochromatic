# OpenRouter and the corpus passes: XingZ6019 and XingZ6020 on build 1d46232457 (2026-10-22)

What was read on 2026-10-22 of the two passes of entry `XingZ60` that ran on the build after `XingZ6018`:
`XingZ6019`,
which ended with no page after 19.2 min because its writing benches were short while NeuralWatt was dry,
and `XingZ6020`,
which was relaunched on the same build and was still running when this was filed.

The facts are the lead's read of 2026-10-22.
The filer did not read the log of either pass,
and no second reader checked them,
so the figures are the lead's and carry no log lines.
The one check the filer made is the ancestry of the build commit,
in "The build".

Note of 2026-10-25 (UTC):
`XingZ6020` has since settled and its read is filed,
as checked by a second reader,
in
[the run of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md),
[its stages](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-stages.md),
[its evidence](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-evidence.md)
and [its defects](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md).
The sentences of this record about `XingZ6020` are kept as written on 2026-10-22.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the earlier passes of the rotation and the build before this one are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).
The run before this one is read in
[the run of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-run.md).

## The build

- `XingZ6019` and `XingZ6020` both ran on the frozen build `sha256-tree-v1:44ce3051...`,
  built from commit `1d46232457`.
- That commit was made at 18:52 UTC on 2026-10-22,
  after the direct DeepSeek and Xiaomi routes landed in it
  (`b960c49c64` and `d2f7683469` are its ancestors).
- It was made before the fixes of the `XingZ6018` read were merged.
  By `git merge-base --is-ancestor`,
  none of `bae1bb310e` (ledger B818),
  `38db033837` (B819) or `c265e8df95` (B820) is an ancestor of `1d46232457`,
  so neither pass ran with the run-ending refusal,
  the corrected NeuralWatt time in its text,
  or the cache gate that reads a prefixed quorum finding.
  The branch that holds them and the build were joined by the merge `ecbbae7fa3`.
  This does not bear on the stop of `XingZ6019`,
  which was a writing-bench floor and not an every-provider-dry refusal.

## XingZ6019

- Launched at 18:54 UTC on 2026-10-22 on the frozen build `sha256-tree-v1:44ce3051...`,
  entry `XingZ60`.
- Ended `DONE processed=0`,
  `artifacts=0/1` at 19:14:20 UTC,
  after 19.2 min.
  No artifact and no page were written.
- The `JUDGE SEATS` lines counted the writing benches at the lanes phase:
  editors 1 of 3 reachable and refiners 1 of 3 reachable,
  against a floor of 2.
  `TALLY` printed `status=ERROR` twice,
  and then the entry was `STALLED` with 16 cache records.
- NeuralWatt was dry for the whole run.
  Its 550 Wh day had been spent by the earlier runs,
  and the account's day restarted at 19:15:08 UTC,
  48 s after the stop.
  The dryness was not a defect in itself;
  the owner had ruled on 2026-10-22 that launches need not be aligned to NeuralWatt's day
  ("You don't need to align launches to NeuralWatt starts.").
- The editors were `hf:zai-org/GLM-5.3-Flash`,
  `glm-5.3` and `deepseek-v4.1-flash`:
  - `hf:zai-org/GLM-5.3-Flash` is served by NeuralWatt only,
    so it was out.
  - `glm-5.3` is served by Hyper,
    whose balance was empty,
    so it was out.
  - `deepseek-v4.1-flash` was served directly and wet,
    so it was the one reachable editor.
- `xiaomi/mimo-v2.6-flash` is a checker only (measured) and was never reached.

## Defect E2: NeuralWatt's known return was not counted as a named return

Recorded as ledger B826.
Open:
a fix is in progress in the lead's briefs labelled D4 and E2,
and has not landed.

Note of 2026-10-25 (UTC):
ledger B826 is fixed;
the merge `9eea0009d1` of the fix rounds 1 to 9 landed on 2026-10-23 at 08:05 UTC and is an ancestor of `main` and
of the build `bcff23b8ce`.
The fix has not yet been exercised by a run (see
[the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md)).

NeuralWatt's dryness has a known return:
the next day starts from the anchor the run decided at its start (19:15:08 UTC),
48 s after the stop in this pass.
The seat reader nevertheless logged `no provider has named its return`,
and the writing-bench floor stopped the entry instead of waiting for it.
Read from the source at the branch tip (not run):
`readDrynessPastShortBench` in `run/src/corpus-run/run-seats-read.ts` takes the shortest of `client.providerHolds()`,
and when that is zero and a writing bench is below its floor in a phase that stops,
it logs `writing bench unreachable ... stopping the entry` and raises `WritingBenchUnreachableError`.
A provider that is dry by its meter alone has no hold there.

The owner's ruling of 2026-09-08 stops the entry only when a writing bench has no reachable seat
"and no provider has named its return"
(`doc/decision/translation-repair-writing-bench-floor.md`),
so a return the run itself knows belongs on the waiting side of that ruling.

## XingZ6020

- Relaunched at 19:32:29 UTC on 2026-10-22 on the same frozen build,
  entry `XingZ60`,
  with a fresh runs directory,
  so it does not reuse the cache records of `XingZ6019`.
- It was still running when this record was filed,
  and its read is owed when it settles.
- Note of 2026-10-25 (UTC):
  it settled on its second attempt at 06:05:33 UTC on 2026-10-23 after 10 h 32 min 51 s,
  and its read is filed in
  [the run of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md).

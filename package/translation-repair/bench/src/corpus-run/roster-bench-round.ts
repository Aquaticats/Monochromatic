import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { runTranslateStage, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { BenchSlice, } from './bench-sample.ts';
import { recordingClient, } from './bench-record.ts';
import type { BenchRow, } from './roster-bench-row.ts';
import { RUN_PER_CALL_TIMEOUT_MS, } from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Roster bench round
// ONE SLICE AT ONE WIDTH, run and recorded.
//
// SPLIT OUT OF `roster-bench.ts` when that file became wiring only. The
// client, the roster and the clock are handed in, so the round buys nothing
// the caller did not give it and a case scripts every reply and every
// instant.

/**
 Runs one slice at one width and records what it cost.

 @param slice - slice to translate

 @param width - producers to seat, taken from the head of the roster

 @param pass - which pass over this width

 @param roster - models the widths are cut from, which also judge

 @param client - client this row's exchanges go through, built by the caller
 so a case supplies one that answers from a script

 @param clock - source of the instants the row's duration is read from

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Row for the report

 @example
 ```ts
 const row = await runBenchRow({ slice, width: 3, pass: 1, roster, client, clock: performance, stageSettings, },);
 ```
 */
export async function runBenchRow(
  {
    slice,
    width,
    pass,
    roster,
    client,
    clock,
    stageSettings,
  }: {
    readonly slice: BenchSlice;
    readonly width: number;
    readonly pass: number;
    readonly roster: readonly RosterModelId[];
    readonly client: ModelClient;
    readonly clock: { readonly now: () => number; };
    readonly stageSettings: StageSettings;
  },
): Promise<BenchRow> {
  /**
   Logger tagged for this run.
   */
  const l = tagged({ tag: `bench-w${String(width,)}p${String(pass,)}`, },);

  /**
   Client recording every exchange this row makes.
   */
  const recorder = recordingClient({ inner: client, },);

  /**
   Seats for this width, from the head of the roster so the sequence is
   nested: width 3 is width 2 plus one model.
   */
  const translators = roster.slice(
    0,
    width,
  );

  /**
   Start of the stage call.
   */
  const began = clock.now();

  /**
   What the stage decided for this slice.
   */
  const result = await runTranslateStage({
    client: recorder.client,
    translatorModelIds: translators,
    judgeModelIds: roster,
    sourceText: slice.sourceText,
    incumbentText: slice.incumbentText,
    // Every benched slice is drawn from a pair the archive HAS translated, so
    // there is always something to fall back on. A bench over anchored slices
    // would say `absent` and have to be ready for the stage to refuse.
    incumbentKind: 'present',
    lineStructured: slice.lineStructured,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  return {
    width,
    pass,
    entryId: slice.entryId,
    index: slice.index,
    sourceChars: slice.sourceText
      .length,
    incumbentChars: slice.incumbentText
      .length,
    translators: [...translators,],
    decision: result.decision,
    keptIncumbent: result.origin === 'incumbent',
    voteWeight: result.voteWeight,
    judgesAvailable: result.tally
      .judgesAvailable,
    ballots: result.tally
      .ballots,
    abstentions: result.tally
      .abstentions,
    selfVotes: result.tally
      .selfVotes,
    round: {
      producers: result.slate
        .map(function toProducer(entry,) {
          return entry.producer;
        },),
      ballots: result.ballots,
    },
    candidateCount: result.candidateCount,
    heardTranslators: result.heardTranslators,
    findings: result.findings,
    calls: recorder.calls,
    ms: clock.now() - began,
  };
}

//endregion Roster bench round

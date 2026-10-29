import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  exchangeFailureLogText,
  exchangeFailureText,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { RepeatGroup, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ArtifactConsistencyRepeat,
  ArtifactConsistencyUnplaced,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import { askConsistencyBench, } from './pass-consistency-ask.ts';
import type { ConsistencyCacheValue, } from './pass-consistency-cache.ts';
import {
  type ConsistencyPlace,
  occurrenceViewOf,
  repeatOf,
} from './pass-consistency-group.ts';
import { askRunOff, } from './pass-consistency-run-off.ts';
import { runOffResultLine, } from './pass-consistency-run-off-line.ts';
import {
  countAsked,
  runOffReasonOf,
  settleAsked,
} from './pass-consistency-settle.ts';
import type {
  DecidedGroup,
  SettledGroup,
} from './pass-consistency-types.ts';
import {
  settleFailed,
  unplacedOf,
} from './pass-consistency-unasked.ts';
import {
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Pass consistency bench decide
// ASKING THE JUDGE BENCH ABOUT ONE GROUP AND DECIDING IT: the first round's
// ballots, a run-off among the finalists, and the counting that
// names the chosen place and the places the chosen rendering is still to reach.
// The carry of the chosen rendering is a later step (`pass-consistency-bench.ts`),
// which waits until every group is decided so each carry can be told the located
// words of the other groups.

/**
 A group with the places a slice holds whole.

 @example
 ```ts
 const prepared: PreparedGroup = { group, places, };
 ```
 */
export type PreparedGroup = {
  /**
   The group the detector found.
   */
  readonly group: RepeatGroup;

  /**
   The places a slice holds whole.
   */
  readonly places: readonly ConsistencyPlace[];
};

/**
 A group the bench has decided, or one that failed before it was decided.

 @example
 ```ts
 const decision: AskedDecision = { kind: 'failed', settled, };
 ```
 */
export type AskedDecision =
  | {
    /**
     The bench was asked and counted and the group is decided.
     */
    readonly kind: 'decided';

    /**
     The repeated wording or frame.
     */
    readonly repeat: ArtifactConsistencyRepeat;

    /**
     The places no slice holds whole.
     */
    readonly unplaced: readonly ArtifactConsistencyUnplaced[];

    /**
     The group decided, with the places the chosen rendering is still to reach.
     */
    readonly decided: DecidedGroup;
  }
    | {
    /**
     The bench could not be asked or counted, and the places stay as they stand.
     */
      readonly kind: 'failed';

    /**
     The group left as it stands, with its failure said.
     */
      readonly settled: SettledGroup;
    };

/**
 Asks the bench about one group and decides it, containing whatever goes wrong.
 The carry of the chosen rendering waits until every group is decided, so each
 carry can see the words every other group located.

 @param client - provider client

 @param modelIds - judge bench asked

 @param text - whole original

 @param prepared - group and its places

 @param cache - store an earlier answer is read from

 @param identityContext - declared identity lines of the page, which the sheet shows

 @param referenceContext - lines of the pages the original cites, which the sheet shows

 @param lifeEnded - whether the entry records a death, which adds the tense block to the sheet

 @param subjectNames - names the translation declares for its subject

 @param signal - entry deadline and caller abort

 @param l - logger the pass writes to

 @param stageSettings - straggler windows the entry file read for this launch

 @returns Group decided, or left as it stands with its failure said

 @throws The abort reason when the caller stopped the work

 @example
 ```ts
 const decision = await decideAskedGroup({ client, modelIds, text, prepared, cache, signal, l, stageSettings, },);
 ```
 */
export async function decideAskedGroup(
  {
    client,
    modelIds,
    text,
    prepared,
    cache,
    identityContext,
    referenceContext,
    lifeEnded,
    subjectNames,
    signal,
    l,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly modelIds: readonly RosterModelId[];
    readonly text: string;
    readonly prepared: PreparedGroup;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<AskedDecision> {
  /**
   Logger pre-tagged with this function's name.
   */
  const gl = tagged({
    tag: decideAskedGroup.name,
    l,
  },);
  /**
   The repeated wording or frame.
   */
  const repeat = repeatOf({
    group: prepared.group,
    text,
  },);
  /**
   The places no slice holds whole.
   */
  const unplaced = unplacedOf({ group: prepared.group, },);
  try {
    /**
     The sheet the first round and the run-off show.
     */
    const subject = {
      repeat,
      occurrences: prepared.places
        .map(function viewOf(place,) {
          return occurrenceViewOf({
            place,
            text,
          },);
        },),
      ...((identityContext === undefined) ? {} : { identityContext, }),
      ...((referenceContext === undefined) ? {} : { referenceContext, }),
      ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
      ...((subjectNames === undefined) ? {} : { subjectNames, }),
    };
    /**
     The first round's ballots.
     */
    const asked = await askConsistencyBench({
      client,
      modelIds,
      subject,
      cache,
      signal,
      exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
      l: gl,
      stageSettings,
    },);
    /**
     What they come to, and the finalists when no place has a strict majority.
     */
    const count = countAsked({
      repeat,
      places: prepared.places,
      asked,
    },);
    if (count.finalists
      .length
      > 0) {
      gl.info(`asking the bench once more: ${runOffReasonOf({
        count,
        asked,
      },)}`,);
    }
    /**
     The bench asked once more among the finalists, absent when there are none.
     */
    const runOff = (count.finalists
      .length
      === 0)
      ? undefined
      : await askRunOff({
        client,
        modelIds,
        subject,
        finalists: count.finalists,
        renderings: count.renderings,
        cache,
        signal,
        exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
        l: gl,
        stageSettings,
      },);
    /**
     The group, counted and settled.
     */
    const decided = settleAsked({
      repeat,
      places: prepared.places,
      unplaced,
      asked,
      ...((runOff === undefined) ? {} : { runOff, }),
    },);
    /**
     What the bench decided for the group.
     */
    const { verdict, } = decided;
    // THE RUN-OFF'S FINAL COUNT IN ONE LINE: usable of heard, the quorum, and the winner or the reason for none.
    if ((verdict.kind === 'asked') && (verdict.runOff !== undefined)) {
      gl.info(runOffResultLine({
        runOff: verdict.runOff,
        winner: verdict.winner,
      },),);
    }
    return {
      kind: 'decided',
      repeat,
      unplaced,
      decided,
    };
  }
  catch (error) {
    if (signal.aborted)
      throw error;
    gl.error(`the bench could not be asked or counted (${exchangeFailureLogText({ error, },)}); the places stay as they stand`,);
    return {
      kind: 'failed',
      settled: settleFailed({
        repeat,
        places: prepared.places,
        unplaced,
        detail: exchangeFailureText({ error, },),
      },),
    };
  }
}

//endregion Pass consistency bench decide

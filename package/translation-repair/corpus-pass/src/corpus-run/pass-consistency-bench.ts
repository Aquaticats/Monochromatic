import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import {
  exchangeFailureLogText,
  exchangeFailureText,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  mapOverlapped,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type AskedDecision,
  decideAskedGroup,
  type PreparedGroup,
} from './pass-consistency-bench-decide.ts';
import {
  type ConsistencyCacheValue,
  openConsistencyCache,
} from './pass-consistency-cache.ts';
import { carryAll, } from './pass-consistency-carry.ts';
import { keptWordsOf, } from './pass-consistency-carry-units.ts';
import { settleCarried, } from './pass-consistency-settle.ts';
import type {
  CarryWriters,
  DecidedGroup,
  SettledGroup,
} from './pass-consistency-types.ts';
import { settleFailed, } from './pass-consistency-unasked.ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  readPhaseSeats,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run/ts';

export {
  type AskedDecision,
  decideAskedGroup,
  type PreparedGroup,
} from './pass-consistency-bench-decide.ts';

//region Pass consistency bench
// ASKING THE JUDGE BENCH ABOUT EVERY GROUP THAT NEEDS IT, side by side, and
// settling what it says: the first round's ballots, a run-off among the finalists
// of a split with no strict majority, and the carry of the chosen rendering to each receiving
// place by the writer bench and a choice among candidates. EVERY GROUP IS DECIDED
// BEFORE ANY IS CARRIED, so each carry is told the located words of the other
// groups it must not rewrite away. The seats are read
// when the first group needs them and not before, so a page that repeats nothing
// the bench must judge reads no meter and waits out no hold.

/**
 Carries the chosen rendering of a decided group to its receiving places and
 settles the group, containing whatever goes wrong.

 @param client - provider client

 @param modelIds - judge bench asked

 @param writers - whether the translate stage's writer bench carries the chosen rendering, and which seats

 @param text - whole original

 @param decision - the group decided

 @param others - the page's other decided groups, whose located words the carry must leave as they are

 @param cache - store an earlier answer is read from

 @param identityContext - declared identity lines of the page, which the sheet shows

 @param referenceContext - lines of the pages the original cites, which the sheet shows

 @param lifeEnded - whether the entry records a death, which adds the tense block to the sheet

 @param subjectNames - names the translation declares for its subject

 @param declared - name pairs the front matter declares, which the floors read

 @param declaredNames - declared name forms, which the floors keep

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @param signal - entry deadline and caller abort

 @param l - logger the pass writes to

 @param stageSettings - straggler windows the entry file read for this launch

 @returns The group settled, or left as it stands with its failure said

 @throws The abort reason when the caller stopped the work

 @example
 ```ts
 const settled = await settleAskedGroup({ client, modelIds, writers, text, decision, others: [], cache, declared: [], declaredNames: [], signal, l, stageSettings, },);
 ```
 */
async function settleAskedGroup(
  {
    client,
    modelIds,
    writers,
    text,
    decision,
    others,
    cache,
    identityContext,
    referenceContext,
    lifeEnded,
    subjectNames,
    declared,
    declaredNames,
    declaredSubjectForms,
    signal,
    l,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly modelIds: readonly RosterModelId[];
    readonly writers: CarryWriters;
    readonly text: string;
    readonly decision: AskedDecision;
    readonly others: readonly DecidedGroup[];
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<SettledGroup> {
  if (decision.kind === 'failed')
    return decision.settled;
  /**
   Logger pre-tagged with this function's name.
   */
  const gl = tagged({
    tag: settleAskedGroup.name,
    l,
  },);
  /**
   The group decided, with the places the chosen rendering is still to reach.
   */
  const { decided, } = decision;
  try {
    /**
     The chosen place, none when no place won.
     */
    const [chosen,] = decided.chosen;
    return settleCarried({
      decided,
      carried: (chosen === undefined)
        ? []
        : await carryAll({
          context: {
            client,
            writers,
            judgeModelIds: modelIds,
            cache,
            text,
            ...((identityContext === undefined) ? {} : { identityContext, }),
            ...((referenceContext === undefined) ? {} : { referenceContext, }),
            ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
            ...((subjectNames === undefined) ? {} : { subjectNames, }),
            declared,
            declaredNames,
            ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
            signal,
            l: gl,
            stageSettings,
          },
          repeat: decision.repeat,
          chosen,
          receivers: decided.receivers,
          kept: keptWordsOf({
            decided,
            others,
          },),
        },),
    },);
  }
  catch (error) {
    if (signal.aborted)
      throw error;
    gl.error(`the carry could not be made (${exchangeFailureLogText({ error, },)}); the places stay as they stand`,);
    return settleFailed({
      repeat: decision.repeat,
      places: decided.places,
      unplaced: decision.unplaced,
      detail: exchangeFailureText({ error, },),
    },);
  }
}

/**
 Names a group asked, for the line that reports a failure left unthrown.

 @param position - position of the group among those asked

 @returns Name of the group

 @example
 ```ts
 const name = groupAskedName({ item, position: 0, },); // 'group 1 asked'
 ```
 */
function groupAskedName(
  {
    position,
  }: {
    readonly item: PreparedGroup;
    readonly position: number;
  },
): string {
  return `group ${String(position + 1,)} asked`;
}

/**
 Names a group carried, for the line that reports a failure left unthrown.

 @param position - position of the group among those carried

 @returns Name of the group

 @example
 ```ts
 const name = groupAskedCarriedName({ item, position: 0, },); // 'group 1 carried'
 ```
 */
function groupAskedCarriedName(
  {
    position,
  }: {
    readonly item: AskedDecision;
    readonly position: number;
  },
): string {
  return `group ${String(position + 1,)} carried`;
}

/**
 Whether the translate stage's writer bench carries the chosen rendering, and
 which seats, or why the writers are not asked.

 @param translators - the writer bench as the seat reading left it

 @param unreachableWriters - clauses naming each writing bench below its floor that no provider has named the return of

 @returns The writers to ask, or why they are not asked

 @example
 ```ts
 const writers = writersOf({ translators, unreachableWriters: [], },);
 ```
 */
export function writersOf(
  {
    translators,
    unreachableWriters,
  }: {
    readonly translators: readonly RosterModelId[];
    readonly unreachableWriters: readonly string[];
  },
): CarryWriters {
  if (unreachableWriters.length > 0) {
    return {
      kind: 'not-asked',
      why: `${unreachableWriters.join('; ',)}; no provider has named a return the pass can wait for`,
    };
  }
  if (translators.length === 0) {
    return {
      kind: 'not-asked',
      why: 'the translators bench seats no model',
    };
  }
  return {
    kind: 'asked',
    modelIds: translators,
  };
}

/**
 Puts every group the bench is asked about to the bench, side by side.

 @param client - provider client

 @param text - whole original

 @param toAsk - groups to put to the bench, in page order

 @param identityContext - declared identity lines of the page, which each sheet shows

 @param referenceContext - lines of the pages the original cites, which each sheet shows

 @param lifeEnded - whether the entry records a death, which adds the tense block to each sheet

 @param subjectNames - names the translation declares for its subject

 @param declared - name pairs the front matter declares, which the floors read

 @param declaredNames - declared name forms, which the floors keep

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @param entryCacheDir - per-entry cache root

 @param pipelineDigest - generation stamp for the cache

 @param signal - entry cancellation

 @param overlap - most groups in flight

 @param l - logger the pass writes to

 @param stageSettings - straggler windows the entry file read for this launch

 @returns One settled group per group asked, in page order

 @throws The abort reason when the caller stopped the work, and whatever the
 seat reading and the cache opening throw, which the pass contains

 @example
 ```ts
 const settled = await askAll({ client, text, toAsk, declared: [], declaredNames: [], entryCacheDir, pipelineDigest, signal, overlap, l, stageSettings, },);
 ```
 */
export async function askAll(
  {
    client,
    text,
    toAsk,
    identityContext,
    referenceContext,
    lifeEnded,
    subjectNames,
    declared,
    declaredNames,
    declaredSubjectForms,
    entryCacheDir,
    pipelineDigest,
    signal,
    overlap,
    l,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly text: string;
    readonly toAsk: readonly PreparedGroup[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly entryCacheDir: string;
    readonly pipelineDigest: PipelineDigest;
    readonly signal: AbortSignal;
    readonly overlap: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<readonly SettledGroup[]> {
  if (toAsk.length === 0)
    return [];
  /**
   This phase's benches, read now and only when someone is to be asked, and the
   writing benches below their floor that the phase goes on without.
   */
  const {
    seats,
    unreachableWriters,
  } = await readPhaseSeats({
    client,
    phase: 'page consistency',
    signal,
    l,
  },);
  /**
   Whether the writers are asked, and which seats.
   */
  const writers = writersOf({
    translators: seats.translators,
    unreachableWriters,
  },);
  /**
   Store earlier answers are read from.
   */
  const cache = await openConsistencyCache({
    dir: entryCacheDir,
    generation: pipelineDigest,
  },);
  // EVERY GROUP IS DECIDED BEFORE ANY GROUP IS CARRIED. A block rewrite for one group can cover the words another
  // group located, and what that group located is known only once its ballots are counted, so the carries wait for the
  // decisions and each is told the located words of every other group, whichever order the groups finished in.
  /**
   Every group decided, or failed, in page order.
   */
  const decisions = await mapOverlapped({
    items: toAsk,
    overlap,
    oneItem: async function decideOne({ item, },): Promise<AskedDecision> {
      return await decideAskedGroup({
        client,
        modelIds: seats.lateJudges,
        text,
        prepared: item,
        cache,
        ...((identityContext === undefined) ? {} : { identityContext, }),
        ...((referenceContext === undefined) ? {} : { referenceContext, }),
        ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
        ...((subjectNames === undefined) ? {} : { subjectNames, }),
        signal,
        l,
        stageSettings,
      },);
    },
    signal,
    l,
    nameOf: groupAskedName,
  },);
  /**
   The groups that were decided, in page order, which every carry reads the others of.
   */
  const decidedGroups = decisions.flatMap(function decidedOf(decision,): readonly DecidedGroup[] {
    return (decision.kind === 'decided') ? [decision.decided,] : [];
  },);
  return await mapOverlapped({
    items: decisions,
    overlap,
    oneItem: async function carryOne({ item, },): Promise<SettledGroup> {
      return await settleAskedGroup({
        client,
        modelIds: seats.lateJudges,
        writers,
        text,
        decision: item,
        others: decidedGroups.filter(function isAnother(group,): boolean {
          return (item.kind === 'decided') && (group !== item.decided);
        },),
        cache,
        ...((identityContext === undefined) ? {} : { identityContext, }),
        ...((referenceContext === undefined) ? {} : { referenceContext, }),
        ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
        ...((subjectNames === undefined) ? {} : { subjectNames, }),
        declared,
        declaredNames,
        ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
        signal,
        l,
        stageSettings,
      },);
    },
    signal,
    l,
    nameOf: groupAskedCarriedName,
  },);
}

//endregion Pass consistency bench

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  assertCheckerIndependence,
  assertCheckerQuorumReachable,
  type RepairModels,
  type RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Checker bench at the stage
// THE CHECKER STAGE READS THE SEATING AGAIN AT THE STAGE ITSELF (class one
// hundred nine, zheermao9, 2026-09-24). The class one hundred three re-seat
// lands where a chunk starts, but the repair lane runs eight chunks at once,
// so the chunks already in flight when Synthetic's weekly allowance ran out
// kept the bench read at their start: seven of eight checker rounds asked
// the two Synthetic-only seats a minute after the dry-out and heard one
// voice, short of quorum, while five re-seat lines had already named a
// bench with two reachable seats for chunks not yet started. A chunk's
// checker stage runs after its critics, panel, editor and select judges,
// which is minutes on a live bench, so it asks the hook again here and runs
// on whatever the reading seats now. It costs nothing while nothing is
// held: the hook returns the seating it last read.

/**
 Refuses a checker bench the contract would refuse against the chunk's own
 writers: the editors and refiners the chunk was seated with wrote the text
 the bench checks, whichever bench is asked.

 LEDGER B29. The chunk check left the refiners out, while its TSDoc said it
 refused a checker who refines and every other caller of the independence
 check passes them; and the bench read at the stage was read for quorum
 alone, so a re-seat naming this chunk's editor would have had it grade its
 own rewrite. Production seats derived benches against the static editors
 and refiners (`corpus-run/run-seats.ts`), so neither reached a run.

 @param models - roster the chunk was seated with, whose writers the bench
 must not hold

 @param checkerModelIds - bench to be asked

 @throws {@link CheckerIndependenceError} when a checker also edits or
 refines, unless the chunk permits self-certification

 @throws {@link CheckerQuorumError} when the bench is below the floor

 @example
 ```ts
 assertBenchAgainstWriters({ models, checkerModelIds: fresh, },);
 ```
 */
function assertBenchAgainstWriters(
  {
    models,
    checkerModelIds,
  }: {
    readonly models: RepairModels;
    readonly checkerModelIds: readonly RosterModelId[];
  },
): void {
  assertCheckerIndependence({
    editorModelIds: models.editorModelIds,
    refinerModelIds: models.refinerModelIds ?? [],
    checkerModelIds,
    selfCertificationPermitted: models.checkerSelfCertificationPermitted ?? false,
  },);
  assertCheckerQuorumReachable({ checkerModelIds, },);
}

/**
 Refuses a chunk roster whose checker bench the contract would refuse: a
 checker who also edits or refines, or fewer checkers than a split verdict
 can be read from. Runs once per chunk before any stage buys anything.

 @param models - roster the chunk is seated with

 @throws {@link CheckerIndependenceError} when a checker also edits or
 refines

 @throws {@link CheckerQuorumError} when the bench is below the floor

 @example
 ```ts
 assertCheckerBench({ models, },);
 ```
 */
export function assertCheckerBench(
  { models, }: { readonly models: RepairModels; },
): void {
  assertBenchAgainstWriters({
    models,
    checkerModelIds: models.checkerModelIds,
  },);
}

/**
 Seating for a driver given no hook: the roster it was handed stands.

 @returns Empty seating

 @example
 ```ts
 const seating = await standingSeating();
 ```
 */
export function standingSeating(): Promise<RepairSliceSeating> {
  return Promise.resolve({},);
}

/**
 Whether two benches name the same seats in the same order.

 @param left - one bench

 @param right - other bench

 @returns True when neither seat nor order differs

 @example
 ```ts
 const unchanged = sameBench({ left: seated, right: fresh, },);
 ```
 */
function sameBench(
  {
    left,
    right,
  }: {
    readonly left: readonly RosterModelId[];
    readonly right: readonly RosterModelId[];
  },
): boolean {
  if (left.length !== right.length)
    return false;
  return left.every(function sameSeat(
    seat,
    index,
  ): boolean {
    return seat === right[index];
  },);
}

/**
 Checkers the stage runs on: the bench the seating read now names where a
 hook is given and that bench differs, else the chunk's own.

 @param models - roster the chunk was seated with

 @param reseat - reads the seating as of now; `standingSeating` where the
 roster the driver was given stands for the whole run

 @param l - repair-lane logger

 @returns Checker ids for this chunk's checker stage and its probe

 @throws {@link CheckerQuorumError} when the bench read now is below the
 checker floor, since a stage the contract refuses must not start

 @throws {@link CheckerIndependenceError} when the bench read now holds a
 model that edited or refines this chunk, for the same reason

 @example
 ```ts
 const stageCheckers = await checkerBenchAtStage({ models, reseat, l, },);
 ```
 */
export async function checkerBenchAtStage(
  {
    models,
    reseat,
    l,
  }: {
    readonly models: RepairModels;
    readonly reseat: () => Promise<RepairSliceSeating>;
    readonly l: Logger;
  },
): Promise<readonly RosterModelId[]> {
  /**
   Seating as of now.
   */
  const seating = await reseat();
  /**
   Roster the reading seats, absent while the given one stands.
   */
  const reseated = seating.repairModels;
  if (reseated === undefined)
    return models.checkerModelIds;
  /**
   Bench the reading seats.
   */
  const fresh = reseated.checkerModelIds;
  if (sameBench({
    left: models.checkerModelIds,
    right: fresh,
  },))
    return models.checkerModelIds;
  assertBenchAgainstWriters({
    models,
    checkerModelIds: fresh,
  },);
  l.info(`checker stage re-seated: ${fresh.join(',',)} (the chunk was seated with ${
    models.checkerModelIds
      .join(',',)
  })`,);
  return fresh;
}

//endregion Checker bench at the stage

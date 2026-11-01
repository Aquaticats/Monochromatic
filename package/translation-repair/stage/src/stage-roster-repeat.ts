import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stage roster repeat
// A STAGE ROSTER IS A SET OF SEATS, and every stage that gathers voices keys
// what it reads back by model id (the panel's ballots, the checkers' ballots,
// the critics' attributions, the page title bench's rows). A roster that seats
// one model twice would count that model's replies as separate voices toward
// quorum while those readers collapse them into one, so a quorum could be met
// on voices that are one model. The editors and the checkers were refused a
// repeated id where their rosters are composed; the panel, the critics and the
// page title bench never were. `gatherStageVoices` is the one place every
// role's round goes through, so the refusal lives there.

/**
 Thrown when a stage roster seats one model more than once.

 @example
 ```ts
 throw new StageRosterRepeatError({ stage: 'panel', duplicated, },);
 ```
 */
export class StageRosterRepeatError extends Error {
  /**
   Declares this message safe to forward: it names a stage label and model ids, and never anything a model wrote.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the report from the stage and the ids it repeated.

   @param stage - label the calling stage gave its gather, so the report names the role that was seated wrongly

   @param duplicated - ids the roster listed more than once, each once
   */
  constructor(
    {
      stage,
      duplicated,
    }: {
      readonly stage: string;
      readonly duplicated: readonly RosterModelId[];
    },
  ) {
    super(
      `the ${stage} roster seats these models more than once: [${duplicated.join(', ',)}]; a repeated id is `
        + `one model counted as several voices, which meets quorum on fewer independent voices than the `
        + `roster size promises, and the ballots keyed by model id would keep only one of its replies`,
    );
    this.name = 'StageRosterRepeatError';
  }
}

/**
 Reads which ids a roster lists more than once, in the order each first repeats.

 @param modelIds - roster a stage is about to ask

 @returns Each repeated id once, empty when the roster seats every model once

 @example
 ```ts
 const duplicated = repeatedRosterIds({ modelIds, },);
 ```
 */
export function repeatedRosterIds(
  { modelIds, }: { readonly modelIds: readonly RosterModelId[]; },
): readonly RosterModelId[] {
  /**
   Ids met so far.
   */
  const met = new Set<RosterModelId>();

  /**
   Ids met a second time, each once.
   */
  const repeated = new Set<RosterModelId>();
  for (const modelId of modelIds) {
    if (met.has(modelId,))
      repeated.add(modelId,);
    met.add(modelId,);
  }
  return [...repeated,];
}

//endregion Stage roster repeat

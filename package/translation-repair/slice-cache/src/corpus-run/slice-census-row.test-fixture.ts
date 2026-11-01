import type { EntryCensus, } from '../../dist/final/node/index.mjs';

//region Slice census row fixture
// ONE MEASURED ENTRY, invented, for the cases of the census's line builders:
// every measure empty unless the case names it.

/**
 The measures a case may set on a row, each optional because a row that does
 not name one carries none of it.

 @example
 ```ts
 const measures: CensusRowMeasures = { targetOnlyBlocks: 1, };
 ```
 */
type CensusRowMeasures = {
  /**
   Source characters of every slice.
   */
  readonly sliceSourceChars?: readonly number[];

  /**
   Target characters of every slice.
   */
  readonly sliceTargetChars?: readonly number[];

  /**
   Source sections the aligner would not pair.
   */
  readonly unpairedSourceSections?: number;

  /**
   Characters in those source sections.
   */
  readonly unpairedSourceChars?: number;

  /**
   Translation sections no source section partnered.
   */
  readonly unpairedTargetSections?: number;

  /**
   Characters in those translation sections.
   */
  readonly unpairedTargetChars?: number;

  /**
   Blocks only the translation carries.
   */
  readonly targetOnlyBlocks?: number;

  /**
   Characters in those blocks.
   */
  readonly targetOnlyChars?: number;

  /**
   Size of every such block.
   */
  readonly targetOnlyBlockChars?: readonly number[];

  /**
   Which carve the sizes describe.
   */
  readonly carve?: EntryCensus['carve'];
};

/**
 Builds one measured entry.

 @param entryId - corpus id the row names

 @param measures - the measures the case sets; every other one stays empty

 @returns A row that carries no slice, no unpaired section and no
 target-only block beyond what the case names

 @example
 ```ts
 const row = censusRowOf({ entryId: 'mochi', measures: { carve: 'settled-moved', }, },);
 ```
 */
export function censusRowOf(
  {
    entryId,
    measures = {},
  }: {
    readonly entryId: string;
    readonly measures?: CensusRowMeasures;
  },
): EntryCensus {
  return {
    entryId,
    sliceSourceChars: [],
    sliceTargetChars: [],
    unpairedSourceSections: 0,
    unpairedSourceChars: 0,
    unpairedTargetSections: 0,
    unpairedTargetChars: 0,
    targetOnlyBlocks: 0,
    targetOnlyChars: 0,
    targetOnlyBlockChars: [],
    carve: 'deterministic',
    pairingRefusal: '',
    ...measures,
  };
}

//endregion Slice census row fixture

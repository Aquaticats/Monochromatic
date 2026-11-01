import {
  type CorpusPin,
  isMissingCorpusObject,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { PairingRecipe, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  censusEntry,
  type EntryCensus,
} from './slice-census-entry.ts';

//region Slice census gather
// Measures every listed entry that carries both sides, and sets the others
// aside by name, reading each entry's settled recipe on the way.

/**
 What a settled artifact says about one entry's recipe, as far as the census
 reads it. `readSettledRecipe` answers this shape and more.

 @example
 ```ts
 const reading: SliceCensusRecipeReading = { kind: 'unsettled', };
 ```
 */
export type SliceCensusRecipeReading = {
  /**
   A two-lane artifact records this entry, and here is its recipe.
   */
  readonly kind: 'settled';

  /**
   Corpus commit the artifact was settled against.
   */
  readonly corpusSha: string;

  /**
   Pairing recipe the artifact records.
   */
  readonly recipe: PairingRecipe;
} | {
  /**
   An artifact exists but predates the recipe.
   */
  readonly kind: 'legacy';
} | {
  /**
   No artifact records this entry.
   */
  readonly kind: 'unsettled';
};

/**
 What the gatherer found over every entry it was given.

 @example
 ```ts
 const gathered: SliceCensusGathered = { rows: [], incomplete: [], legacy: [], };
 ```
 */
export type SliceCensusGathered = {
  /**
   Entries that carry both sides, measured.
   */
  readonly rows: readonly EntryCensus[];

  /**
   Ids missing one side, which is ordinary rather than a fault.
   */
  readonly incomplete: readonly string[];

  /**
   Entries whose artifact predates the recipe, carved deterministically,
   including any of them found incomplete.
   */
  readonly legacy: readonly string[];
};

/**
 Measures each listed entry, one at a time.

 @param pin - corpus clone and commit the entries are read at, which an entry
 with a settled artifact reads at the artifact's own commit instead

 @param entryIds - corpus ids to measure, in the order to measure them

 @param readRecipe - reads one entry's settled recipe from the runs
 directory, passed in because it reads files

 @returns Measured rows, incomplete ids and legacy ids, each in the order
 the entries were given

 @throws Whatever the read of a recipe throws, and whatever measuring an
 entry throws except the absence of one side, which makes the entry
 incomplete

 @example
 ```ts
 const { rows, incomplete, legacy, } = await gatherSliceCensus({ pin, entryIds, readRecipe, },);
 ```
 */
export async function gatherSliceCensus(
  {
    pin,
    entryIds,
    readRecipe,
  }: {
    readonly pin: CorpusPin;
    readonly entryIds: readonly string[];
    readonly readRecipe: (input: { readonly entryId: string; },) => Promise<SliceCensusRecipeReading>;
  },
): Promise<SliceCensusGathered> {
  /**
   Entries that carry both sides, measured.
   */
  const rows: EntryCensus[] = [];

  /**
   Ids missing one side, which is ordinary rather than a fault.
   */
  const incomplete: string[] = [];

  /**
   Entries whose artifact predates the recipe, carved deterministically.
   */
  const legacy: string[] = [];
  for (const entryId of entryIds) {
    /* oxlint-disable no-await-in-loop -- sequential by design: this reads git at a pinned commit and a fan-out would only contend for the same object store */
    /**
     Recipe the entry's settled artifact records, if any.
     */
    const settled = await readRecipe({ entryId, },);
    /* oxlint-enable no-await-in-loop */
    if (settled.kind === 'legacy')
      legacy.push(entryId,);
    try {
      /* oxlint-disable-next-line no-await-in-loop -- sequential by design: this reads git at a pinned commit and a fan-out would only contend for the same object store */
      rows.push(await censusEntry({
        entryId,
        // The commit the settled artifact read, or the run's own for an entry
        // whose artifact predates the recipe.
        ...((settled.kind === 'settled')
          ? {
            pin: {
              ...pin,
              commitSha: settled.corpusSha,
            },
            recipe: settled.recipe,
          }
          : { pin, }),
      },),);
    }
    catch (error) {
      // A missing translation is the ordinary shape of an incomplete pair, and
      // the corpus has them. Anything else keeps propagating.
      if (!isMissingCorpusObject(error,))
        throw error;
      incomplete.push(entryId,);
    }
  }
  return {
    rows,
    incomplete,
    legacy,
  };
}

//endregion Slice census gather

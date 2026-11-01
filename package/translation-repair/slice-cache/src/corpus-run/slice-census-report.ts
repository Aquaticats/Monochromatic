import {
  type CorpusPin,
  listCorpusPeople,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { describeSpread, } from './census-spread.ts';
import { sliceCensusCarveLine, } from './slice-census-carve.ts';
import {
  gatherSliceCensus,
  type SliceCensusRecipeReading,
} from './slice-census-gather.ts';
import { sliceCensusTargetOnlyLines, } from './slice-census-target-only.ts';
import { sliceCensusUnpairedLines, } from './slice-census-unpaired.ts';
import { sliceCensusWidestLines, } from './slice-census-widest.ts';

//region Slice census report
// What the corpus looks like AFTER slicing, measured rather than assumed, and
// spending no quota.
//
// Three questions need this and none of them could be answered from the code:
//
//   How many calls a translate lane costs, which is slices times producers, and
//   how large the largest of them is. `RUN_PER_CALL_TIMEOUT_MS` was tuned
//   against repair envelopes averaging 72 characters, and this lane sends whole
//   slices.
//
//   How much of the corpus reaches the lane as a section only ONE side carries,
//   which subdivision returns whole. Those are the calls that time out.
//
//   How much text sits in TARGET-ONLY blocks, the class where the English
//   carries something the Chinese markdown does not, letters held as images
//   being the known case. A translator working from the source has no source for
//   it, so whether it can be detected deterministically decides whether it can
//   be protected.
//
// The command is `slice-census.ts`; one entry's measure is
// `slice-census-entry.ts`; the gathering is `slice-census-gather.ts` and the
// lines are `slice-census-carve.ts`, `slice-census-unpaired.ts`,
// `slice-census-target-only.ts` and `slice-census-widest.ts`.

/**
 Measures every complete pair at the pin and prints the census.

 THE RUNS DIRECTORY IS RESOLVED AFTER THE CORPUS IS LISTED, as the command
 always did, so a clone that cannot be listed is the refusal an operator
 meets even where no runs directory resolves either.

 @param pin - corpus clone and commit to read

 @param resolveRuns - finds the runs directory whose settled artifacts carry
 each entry's recipe, passed in because the environment decides it

 @param readRecipe - reads one entry's settled recipe from that directory,
 passed in because it reads files

 @throws Whatever listing the corpus, resolving the runs directory, reading a
 recipe or measuring an entry throws, except the absence of one side of an
 entry

 @example
 ```ts
 await reportSliceCensus({ pin, resolveRuns: resolveRunsDir, readRecipe: readSettledRecipe, },);
 ```
 */
export async function reportSliceCensus(
  {
    pin,
    resolveRuns,
    readRecipe,
  }: {
    readonly pin: CorpusPin;
    readonly resolveRuns: () => Promise<string>;
    readonly readRecipe: (
      input: {
        readonly entryId: string;
        readonly runsDir: string;
      },
    ) => Promise<SliceCensusRecipeReading>;
  },
): Promise<void> {
  /**
   Every corpus id at the pinned commit.
   */
  const entryIds = await listCorpusPeople({ pin, },);

  /**
   Runs directory whose settled artifacts carry each entry's recipe.
   */
  const runsDir = await resolveRuns();

  /**
   Entries measured, set aside as incomplete, and found carved
   deterministically.
   */
  const {
    rows,
    incomplete,
    legacy,
  } = await gatherSliceCensus({
    pin,
    entryIds,
    readRecipe: async function readRecipeIn({ entryId, },): Promise<SliceCensusRecipeReading> {
      return await readRecipe({
        entryId,
        runsDir,
      },);
    },
  },);

  /**
   Every slice's source characters, corpus-wide.
   */
  const sourceChars = rows.flatMap(function toSourceChars(row,) {
    return [...row.sliceSourceChars,];
  },);

  /**
   Every slice's target characters.
   */
  const targetChars = rows.flatMap(function toTargetChars(row,) {
    return [...row.sliceTargetChars,];
  },);

  /**
   Every line the census prints, in order.
   */
  const lines = [
    `CENSUS complete pairs: ${String(rows.length,)}, incomplete: ${
      String(incomplete.length,)
    }, slices: ${String(sourceChars.length,)}`,
    sliceCensusCarveLine({
      rows,
      legacyCount: legacy.length,
    },),
    describeSpread({
      label: 'CENSUS slice source chars',
      values: sourceChars,
    },),
    describeSpread({
      label: 'CENSUS slice target chars',
      values: targetChars,
    },),
    ...sliceCensusUnpairedLines({ rows, },),
    ...sliceCensusTargetOnlyLines({ rows, },),
    ...sliceCensusWidestLines({ rows, },),
  ];
  for (const line of lines)
    console.log(line,);
}

//endregion Slice census report

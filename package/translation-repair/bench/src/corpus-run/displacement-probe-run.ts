import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { reportLines, } from './displacement-probe-report.ts';
import {
  type EntryDisplacement,
  readEntry,
} from './displacement-probe-row.ts';
import {
  recipeLabel,
  type SettledCarve,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Displacement probe run
// Relocation: where the corpus carries a passage the translator MOVED across a
// section boundary, which a per-slice judge cannot tell from a fabrication,
// and what ELSE the same size reading turns up along the way.
//
// COSTS NOTHING AND DECIDES NOTHING. It reads two files per entry, counts
// characters, and prints. No model is asked, no artifact is written, and no lane
// reads its output.
//
// WHY IT REPORTS FOUR THINGS. Its first version reported one number, 44 moved
// pairs, and that number was a mixture: relocation, sections nobody translated,
// content that exists only in English, and arithmetic on slices too short to
// mean anything. Each wants a different remedy and a different ticket, so each
// is counted apart.

/**
 One entry beside its carve, or the reason it has none.

 @example
 ```ts
 const entry: EntryCarve = { entryId: 'whiskers', carve: { kind: 'unsettled', }, };
 ```
 */
type EntryCarve = {
  /**
   Corpus entry.
   */
  readonly entryId: string;

  /**
   Its carve through the settled recipe, or why there is none.
   */
  readonly carve: SettledCarve;
};

/**
 Walks the settled entries and reports what their size anomalies look like.

 OVER SETTLED ENTRIES ONLY, carved through the recipe each artifact records,
 since the displacement this reads is a property of the slicing the lanes
 judged. Its first versions carved the whole corpus with the deterministic
 aligner and counted that aligner's own slides as translation displacement.

 @param log - logger every report line goes to

 @param listEntryIds - entries whose settled artifacts name the population,
 read from the runs directory

 @param carve - one entry's slicing through its settled recipe, or why there
 is none

 @param writeOut - where the rows document goes, the process's standard
 output in a run

 @example
 ```ts
 await probeDisplacement({ log, listEntryIds, carve, writeOut, },);
 ```
 */
export async function probeDisplacement(
  {
    log,
    listEntryIds,
    carve,
    writeOut,
  }: {
    readonly log: Logger;
    readonly listEntryIds: () => Promise<readonly string[]>;
    readonly carve: (entryId: string,) => Promise<SettledCarve>;
    readonly writeOut: (text: string,) => void;
  },
): Promise<void> {
  /**
   Every settled entry.
   */
  const entryIds = await listEntryIds();

  /**
   Each entry carved through its recipe, or the reason it could not be.
   */
  const carves = await allInInputOrder({
    members: entryIds.map(async function toCarve(entryId,): Promise<EntryCarve> {
      return {
        entryId,
        carve: await carve(entryId,),
      };
    },),
  },);

  // SAID IN THE ORDER THE ENTRIES ARE LISTED, once every carve has ended, so
  // two walks over one archive log the same lines; until 2026-10-10 each
  // carve logged as it ended, in whichever order that was.
  for (
    const {
      entryId,
      carve: entryCarve,
    } of carves
  ) {
    if (entryCarve.kind !== 'settled')
      log.info(`${entryId}: skipped, ${entryCarve.kind} artifact records no recipe`,);
    // A CARVE THAT MOVED IS STILL MEASURED, and said to be: the readings then
    // describe slices the run did not see.
    if (entryCarve.kind === 'settled') {
      /**
       Whether the re-carve is the run's own.
       */
      const { reproduction, } = entryCarve;
      if (reproduction.kind === 'moved')
        log.warn(`${entryId}: re-carve is not the run's (${reproduction.detail}); its readings measure other slices`,);
    }
  }

  /**
   Readings for every settled entry.
   */
  const rows = carves.flatMap(function toRow({
    entryId,
    carve: entryCarve,
  },): readonly EntryDisplacement[] {
    if (entryCarve.kind !== 'settled')
      return [];
    return [readEntry({
      entryId,
      prepared: entryCarve.prepared,
    },),];
  },);

  /**
   Settled entries whose recipe had a defaulted half, each with the label
   naming the halves.
   */
  const defaulted = carves.flatMap(function toDefaulted({
    entryId,
    carve: entryCarve,
  },): readonly {
    readonly entryId: string;
    readonly label: string;
  }[] {
    if (entryCarve.kind !== 'settled')
      return [];

    /**
     Halves this recipe lacks.
     */
    const { unrecorded, } = entryCarve.recipe;
    if (unrecorded.length === 0)
      return [];
    return [{
      entryId,
      label: recipeLabel({ recipe: entryCarve.recipe, },),
    },];
  },);
  for (
    const line of reportLines({
      rows,
      artifactCount: entryIds.length,
      defaulted,
    },)
  )
    log.info(line,);
  writeOut(`${
    JSON.stringify(
      { rows, },
      undefined,
      2,
    )
  }\n`,);
}

//endregion Displacement probe run

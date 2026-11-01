import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ArtifactDeliveryRow,
  type ArtifactFileName,
  entryIdOfArtifact,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { parseSettledTwoLaneArtifact, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  pageRelationOf,
  type SettledPageRelation,
} from './rendering-audit-settled-relation.ts';
import {
  type WouldShipReading,
  wouldShipTextPerSlice,
} from '@monochromatic-dev/translation-repair-page-check/ts';

//region Damage regions, version 2
// EVERY REGION WHERE THIS PIPELINE SHIPPED REPLACEMENT TEXT, read out of the
// version 2 delivery ledger.
//
// WHY THIS EXISTS: `damage-sample.ts` drew from a top-level `issues[]`, which
// version 2 artifacts do not have, so the draw failed outright on the pool
// settled 2026-08-17 with
// `artifact parse failed at artifact Acheron.issues: expected an array`.
//
// MOVING THE FIELD READ WOULD HAVE BEEN THE WRONG FIX, and quietly so. Version 2
// runs two lanes and BOTH of them ship replacement text. On `Acheron` the repair
// lane ships 3 replacements and the translate lane ships 4, so a draw reading
// only the repair lane's issues would cover 3 of the 7 regions where text was
// actually replaced, while describing itself as a draw over the shipped regions.
// Nothing in its output would show the shortfall.
//
// THE POPULATION IS DETERMINED BY THE QUESTION rather than chosen. The damage
// question is whether an edit damaged the text, and it is answerable exactly
// where a row carries both what was there before and what shipped instead. Every
// `replacement-shipped` row carries both, in either lane, so both lanes belong
// and the lane is recorded per row so a reader can still separate them.
//
// THE ARTIFACT'S OWN TEXT IS USED, never a fresh slicing of the corpus, for the
// reason the settled audit established: re-sliced text is a different input from the one the
// judges saw, so a sheet built from it would ask about something the pipeline
// never produced. It also removes the old draw's unplaceable-region failure,
// where a region survived the draw and then could not be located again.

/**
 Raised when a slice that shipped is named by no comparison row.

 @example
 ```ts
 throw new DamageRegionError({ message: 'slice 4 shipped in the repair lane and is named by no comparison row', },);
 ```
 */
export class DamageRegionError extends Error {
  /**
   Builds refusal carrying what could not hold.

   @param message - which slice and lane no row accounts for

   @example
   ```ts
   throw new DamageRegionError({ message: 'slice 4 shipped in the repair lane and is named by no comparison row', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'DamageRegionError';
  }
}

/**
 Lanes a damage draw ranges over, in the order a pass runs them.
 */
export const DAMAGE_LANES = [
  'repair',
  'translate',
] as const;

/**
 Lane one shipped region came from.
 */
export type DamageLane = typeof DAMAGE_LANES[number];

/**
 One region where a lane shipped replacement text.

 @example
 ```ts
 const region: ShippedRegion = {
   entryId: 'Tabby',
   lane: 'repair',
   sliceIndex: 2,
   regionId: 'repair#2',
   sourceText: '猫',
   incumbentText: 'the cat',
   shippedText: 'the tabby',
 };
 ```
 */
export type ShippedRegion = {
  /**
   Corpus entry the region belongs to.
   */
  readonly entryId: string;

  /**
   Lane that shipped it, kept because the two answer different questions and
   pooling them without a label would make them impossible to separate later.
   */
  readonly lane: DamageLane;

  /**
   Slice within the preparation both lanes ran over.
   */
  readonly sliceIndex: number;

  /**
   Stable identity within an entry, which the draw digests and the sheet
   shows. Version 1 had an envelope id here; version 2 addresses a slice by
   lane and index instead, and one slice can be shipped by both lanes.
   */
  readonly regionId: string;

  /**
   Original passage, as the judges were shown it.
   */
  readonly sourceText: string;

  /**
   Wording that was there before, which is what an edit could have damaged.
   */
  readonly incumbentText: string;

  /**
   Wording that shipped instead.
   */
  readonly shippedText: string;

  /**
   Whether a later stage overruled that wording.

   BESIDE THE LANE MEANING, never instead of it, and the measurement is why.
   A region exists where a lane REPLACED an incumbent, which stays true
   however the contest and the consolidation later ruled. Rebuilding this
   population from what would ship instead would cut it from 378 regions to
   30 and delete every region from 33 of 47 artifacts, since on an entry
   nobody has decided the archive stands and nothing replaces anything. The
   draw would not be corrected, it would be emptied.

   A DISPLACED REGION IS STILL A REAL EDIT. The lane made it, and asking
   whether it damaged the text is still answerable; what the annotation adds
   is that no reader of a document would meet the result.
   */
  readonly pageRelation: SettledPageRelation;
};

/**
 What one pool of artifacts held.

 THE SKIPPED COUNT IS REPORTED RATHER THAN DROPPED. A slice whose incumbent is
 absent was filled where the archive had no English at all, so nothing was
 replaced and no edit could have damaged anything: the honest question there is
 whether the rendering is correct, which is a different sheet. Counting them
 keeps that visible instead of letting the pool look smaller than the run.

 @example
 ```ts
 const census: ShippedRegionCensus = { regions: [], filledWithoutIncumbent: 0, };
 ```
 */
export type ShippedRegionCensus = {
  /**
   Regions the damage question can be asked about.
   */
  readonly regions: readonly ShippedRegion[];

  /**
   Rows that shipped into a passage with no incumbent wording.
   */
  readonly filledWithoutIncumbent: number;
};

/**
 Builds a region's identity within its entry.

 @param lane - lane that shipped it

 @param sliceIndex - slice index

 @returns Identity, stable across runs of one pipeline

 @example
 ```ts
 const id = regionIdOf({ lane: 'translate', sliceIndex: 4, },);
 ```
 */
export function regionIdOf(
  {
    lane,
    sliceIndex,
  }: {
    readonly lane: DamageLane;
    readonly sliceIndex: number;
  },
): string {
  return `${lane}#${String(sliceIndex,)}`;
}

/**
 Turns one lane's delivery ledger into the regions a damage draw can use.

 EXPORTED FOR ITS TESTS, per `XPT`. Every judgement this module makes lives
 here: which rows count, which are set aside, and what identifies one. The
 reader around it only lists files and hands them to a parser that has its own
 tests, so testing it through the filesystem would exercise that parser again
 and this decision once.

 @param entryId - corpus entry

 @param lane - lane the rows came from

 @param rows - that lane's delivery ledger

 @param laneSelection - whether any stage decided this entry

 @param readings - what would stand at each slice, by chunk index

 @returns Regions, and how many rows had no incumbent to damage

 @throws {@link Error} when a shipped row is named by no comparison row,
 which is a contradiction inside one artifact rather than a missing answer

 @example
 ```ts
 const found = regionsOfLane({ entryId, lane: 'repair', rows, laneSelection, readings, },);
 ```
 */
export function regionsOfLane(
  {
    entryId,
    lane,
    rows,
    laneSelection,
    readings,
  }: {
    readonly entryId: string;
    readonly lane: DamageLane;
    readonly rows: readonly ArtifactDeliveryRow[];
    readonly laneSelection: ArtifactLaneSelection;
    readonly readings: ReadonlyMap<number, WouldShipReading>;
  },
): ShippedRegionCensus {
  /**
   Rows where this lane's wording replaced something.
   */
  const shipped = rows.filter(function wasShipped(row,): boolean {
    /**
     What the lane did with this slice.
     */
    const { delivery, } = row;
    return delivery.kind === 'replacement-shipped';
  },);

  /**
   Of those, the ones that replaced actual wording rather than filling a gap.
   */
  const replaced = shipped.filter(function hadIncumbent(row,): boolean {
    return row.incumbentKind === 'present';
  },);

  return {
    regions: replaced.map(function toRegion(row,): ShippedRegion {
      /**
       What would stand at this slice.

       THROWN ON RATHER THAN SKIPPED, since the comparison is derived from
       the same slicing this ledger delivered: a shipped row no comparison
       row names is a contradiction, and dropping it would shrink the draw
       population for a reason nobody would see.
       */
      const reading = readings.get(row.sliceIndex,);
      if (reading === undefined)
        throw new DamageRegionError({
          message: `slice ${row.sliceIndex} shipped in the ${lane} lane and is named by no comparison row`,
        },);

      return {
        entryId,
        lane,
        sliceIndex: row.sliceIndex,
        regionId: regionIdOf({
          lane,
          sliceIndex: row.sliceIndex,
        },),
        sourceText: row.sourceText,
        incumbentText: row.incumbentText,
        shippedText: row.shippedText,
        pageRelation: pageRelationOf({
          laneSelection,
          reading,
          candidateText: row.shippedText,
        },),
      };
    },),
    filledWithoutIncumbent: shipped.length - replaced.length,
  };
}

/**
 Reads every shipped region out of a pool of version 2 artifacts.

 ONE FILE AT A TIME, on purpose: an artifact runs to hundreds of kilobytes and
 holding a whole pool open at once buys nothing a sequential read does not.

 @param artifactsDir - directory the settled artifacts sit in

 @param files - artifact file names already filtered to the eligible pool

 @returns Every region the damage question can be asked about, entry order

 @throws {@link ArtifactParseError} when an artifact is malformed

 @example
 ```ts
 const census = await collectTwoLaneShippedRegions({ artifactsDir, files, },);
 ```
 */
export async function collectTwoLaneShippedRegions(
  {
    artifactsDir,
    files,
  }: {
    readonly artifactsDir: string;
    readonly files: readonly ArtifactFileName[];
  },
): Promise<ShippedRegionCensus> {
  /**
   Regions gathered so far.
   */
  const found: ShippedRegion[] = [];

  /**
   Rows that filled a passage having no incumbent wording.
   */
  const skipped = { count: 0, };

  /* oxlint-disable no-await-in-loop -- sequential on purpose: one artifact at a time keeps peak memory flat across a pool */
  for (const file of files) {
    /**
     Entry id, which is the artifact's own file name.
     */
    const entryId = entryIdOfArtifact({ name: file, },);

    /**
     Artifact, parsed by version 2's own reader so its invariants are checked
     here rather than assumed.
     */
    const artifact = parseSettledTwoLaneArtifact({
      value: await readRunJson({ path: `${artifactsDir}/${file}`, },),
    },);

    /**
     What would stand at each slice, derived once for both lanes.
     */
    const readings = new Map(
      wouldShipTextPerSlice({ artifact, },)
        .map(function pair(slice,): readonly [
          number,
          WouldShipReading,
        ] {
          return [
            slice.sliceIndex,
            slice.reading,
          ];
        },),
    );

    DAMAGE_LANES.forEach(function perLane(lane,): void {
      /**
       This lane as the parser read it.
       */
      const laneRead = artifact.lanes[lane];

      /**
       What this lane delivered.
       */
      const census = regionsOfLane({
        entryId,
        lane,
        rows: laneRead.delivery,
        laneSelection: artifact.laneSelection,
        readings,
      },);
      found.push(...census.regions,);
      skipped.count += census.filledWithoutIncumbent;
    },);
  }
  /* oxlint-enable no-await-in-loop */

  return {
    regions: found,
    filledWithoutIncumbent: skipped.count,
  };
}

//endregion Damage regions, version 2

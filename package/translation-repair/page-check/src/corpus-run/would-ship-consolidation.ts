import type { ArtifactConsolidateSlice, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { WouldShipSource, } from './would-ship-text.ts';

//region Would-ship consolidation
// WHAT THE THIRD RENDERING AND THE FINAL POLISH CONTRIBUTE AT ONE SLICE, split from `would-ship-text.ts` at its line cap
// when the interim page's reading joined it. The reader there walks the deciders; these two answer for the consolidation.

/**
 What the third rendering contributes at one slice.

 A NAMED ABSENCE RATHER THAN `undefined`, for the reason the union it feeds
 carries: this stage contributing nothing is a state to read, not a value
 missing. `ArtifactConsolidateShipped` makes the same choice one level
 below, and collapsing it here would put the trap back one call deeper.

 @example
 ```ts
 const contribution: ConsolidationContribution = { kind: 'replaced-nothing', };
 ```
 */
export type ConsolidationContribution =
  | {
    /**
     This stage settled wording here, and it replaces whatever stood.
     */
    readonly kind: 'wording';

    /**
     Exactly what it settled, wrapped as the stage left it.
     */
    readonly text: string;
  }
    | {
    /**
     No wording passed the deterministic rule here, so the archive keeps the
     slice over whatever the lane contest chose (owner, 2026-09-27, "Keep
     archive, ship").
     */
      readonly kind: 'archive-kept';
    }
    | {
    /**
     This stage replaced nothing here, so whatever stood still stands.
     */
      readonly kind: 'replaced-nothing';
    };

/**
 Reads final body polish wording where generation six stage replaced base.

 @param artifact - parsed artifact carrying optional polish records

 @param sliceIndex - slice to answer

 @returns Polished wording or named absence

 @example
 ```ts
 const text = polishedWordingAt({ artifact, sliceIndex: 0, });
 ```
 */
export function polishedWordingAt(
  {
    artifact,
    sliceIndex,
  }: {
    readonly artifact: WouldShipSource;
    readonly sliceIndex: number;
  },
): ConsolidationContribution {
  /**
   Consolidation stage carrying optional polish records.
   */
  const { consolidation, } = artifact;
  if (consolidation.kind !== 'settled')
    return { kind: 'replaced-nothing', };
  /**
   Consolidation record for requested slice.
   */
  const slice = consolidation.slices
    .find(function namesIt(candidate: ArtifactConsolidateSlice,): boolean {
      return candidate.sliceIndex === sliceIndex;
    },);
  if (slice === undefined)
    return { kind: 'replaced-nothing', };
  /**
   Polish decision for requested slice, absent before generation six.
   */
  const { polish, } = slice;
  if ((polish === undefined) || (polish.kind !== 'settled'))
    return { kind: 'replaced-nothing', };
  if (!polish.changed)
    return { kind: 'replaced-nothing', };
  return {
    kind: 'wording',
    text: polish.text,
  };
}

/**
 Names consolidation wording at one slice, or that it contributed none.

 @param artifact - parsed artifact whose consolidation is read

 @param sliceIndex - slice to answer

 @returns Consolidated wording or named absence

 @example
 ```ts
 const text = consolidatedWordingAt({ artifact, sliceIndex: 0, });
 ```
 */
export function consolidatedWordingAt(
  {
    artifact,
    sliceIndex,
  }: {
    readonly artifact: WouldShipSource;
    readonly sliceIndex: number;
  },
): ConsolidationContribution {
  /**
   What this artifact says about its third rendering, across all three states.
   */
  const { consolidation, } = artifact;
  if (consolidation.kind !== 'settled')
    return { kind: 'replaced-nothing', };

  /**
   Record this stage left for the slice, absent where it was never eligible.
   */
  const slice = consolidation
    .slices
    .find(function namesIt(candidate: ArtifactConsolidateSlice,): boolean {
      return candidate.sliceIndex === sliceIndex;
    },);
  if (slice === undefined)
    return { kind: 'replaced-nothing', };

  /**
   Wording this slice contributes, or a named absence saying it contributes none.

   THE INCUMBENT THAT STOOD IN IS WORDING TOO (owner, 2026-09-09): the lane
   contest left a standing the gate refused, so what the contest chose is
   not what ships, and only this record says so.
   */
  const { shipped, } = slice;
  if (shipped.kind === 'unchanged')
    return { kind: 'replaced-nothing', };
  if (shipped.kind === 'archive')
    return { kind: 'archive-kept', };

  return {
    kind: 'wording',
    text: shipped.text,
  };
}

//endregion Would-ship consolidation

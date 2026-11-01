import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { sameAddress, } from './destination-renderings.ts';
import {
  collectDestinations,
  type DestinationCheck,
} from './dropped-destinations.ts';

//region Destination completeness
// THE REFUSAL TRACES WHAT IT REFUSES (ledger E1). It used to say only how many
// destinations a page would drop, after every stage had spent; XingZ6011 lost
// 96 minutes on 2026-09-26 to a refusal its log could not identify. Each
// dropped address is traced to the slices whose original carries it, whose
// archive span carries it, and whose shipped text carries it: the address
// itself is corpus content and goes to the run log, the slice indices go into
// the message.
//
// A SLICE CARRIES A DESTINATION WHERE IT READS AS ONE: the readers that found
// the drop (`collectDestinations`) read the slice's own text, and the slice
// carries the address when one of the destinations they read is that address,
// a trailing slash aside, as `sameAddress` keys it. A search for the address
// as a substring traced a dropped `https://example.org/a` to a slice holding
// only `https://example.org/ab`, and a dropped `.` to every slice holding a
// full stop, the shipped text included, which then read as keeping what the
// page dropped.

/**
 Where one dropped destination sits among the slices.

 @example
 ```ts
 const trace: DroppedDestinationTrace = { sourceSlices: [4,], archiveSlices: [], shippedSlices: [], };
 ```
 */
export type DroppedDestinationTrace = Readonly<{
  /**
   Slices whose original carries it; empty when it sits outside every slice.
   */
  sourceSlices: readonly number[];

  /**
   Slices whose archive span carries it.
   */
  archiveSlices: readonly number[];

  /**
   Slices whose shipped text carries it, empty since the page drops it.
   */
  shippedSlices: readonly number[];
}>;

/**
 One side of one slice: its index and the text that side carries.

 @example
 ```ts
 const side: SliceText = { sliceIndex: 4, text: 'She naps.', };
 ```
 */
type SliceText = Readonly<{
  /**
   Slice the text belongs to.
   */
  sliceIndex: number;

  /**
   What that side carries there.
   */
  text: string;
}>;

/**
 One side of one slice: its index and the keys of the destinations its text
 carries.

 @example
 ```ts
 const side: SliceDestinations = { sliceIndex: 4, keys: new Set(['https://example.org/tabby',],), };
 ```
 */
type SliceDestinations = Readonly<{
  /**
   Slice the text belongs to.
   */
  sliceIndex: number;

  /**
   Destinations that side carries there, each as `sameAddress` keys it.
   */
  keys: ReadonlySet<string>;
}>;

/**
 Reads every slice's text on one side for the destinations it carries, by
 the readers the check reads whole pages with.

 @param texts - every slice's text on that side

 @param side - which side, for the readers

 @returns Each slice's destination keys, in slice order

 @example
 ```ts
 const originals = destinationsOfSide({ texts: originalTexts, side: 'source', },);
 ```
 */
function destinationsOfSide(
  {
    texts,
    side,
  }: {
    readonly texts: readonly SliceText[];
    readonly side: 'source' | 'page' | 'archive';
  },
): readonly SliceDestinations[] {
  return texts.map(function readSlice({
    sliceIndex,
    text,
  },): SliceDestinations {
    return {
      sliceIndex,
      keys: new Set(
        collectDestinations({
          text,
          side,
        },)
          .urls
          .map(function keyOf(url,): string {
            return sameAddress({ url, },);
          },),
      ),
    };
  },);
}

/**
 Slices whose text on one side carries an address as a destination of its
 own.

 @param side - every slice's destinations on that side

 @param address - destination to look for

 @returns Slice indices, in slice order

 @example
 ```ts
 const sources = slicesCarrying({ side: originals, address, },);
 ```
 */
function slicesCarrying(
  {
    side,
    address,
  }: {
    readonly side: readonly SliceDestinations[];
    readonly address: string;
  },
): readonly number[] {
  /**
   The address as the readers' destinations are keyed.
   */
  const key = sameAddress({ url: address, },);
  return side
    .filter(function carriesIt({ keys, },): boolean {
      return keys.has(key,);
    },)
    .map(function indexOf({ sliceIndex, },): number {
      return sliceIndex;
    },);
}

/**
 Traces every dropped destination to the slices that carry it on each side.

 @param dropped - destinations the page does not carry

 @param slices - preparation the page was spliced over

 @param replacements - rows the page writes, by slice

 @returns One trace per dropped destination, in the order given, none
 without reading a slice when nothing was dropped

 @example
 ```ts
 const traces = traceDroppedDestinations({ dropped: destinations.dropped, slices, replacements, },);
 ```
 */
export function traceDroppedDestinations(
  {
    dropped,
    slices,
    replacements,
  }: {
    readonly dropped: readonly string[];
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): readonly DroppedDestinationTrace[] {
  // EVERY PUBLISH TRACES, and almost every page drops nothing; reading every
  // slice's three texts for destinations is then work nobody reads.
  if (dropped.length === 0)
    return [];
  /**
   Every slice's original.
   */
  const originals = slices.map(function originalOf({
    source,
    target,
  },): SliceText {
    return {
      sliceIndex: target.sliceIndex,
      text: source.text,
    };
  },);
  /**
   Every slice's archive span.
   */
  const archives = slices.map(function archiveOf({ target, },): SliceText {
    return {
      sliceIndex: target.sliceIndex,
      text: target.text,
    };
  },);
  /**
   What every slice ships: its row where one is written, else its archive span.
   */
  const shipped = archives.map(function shippedOf(archive,): SliceText {
    /**
     The row written at this slice, if any.
     */
    const row = replacements.find(function writesHere({ sliceIndex, },): boolean {
      return sliceIndex === archive.sliceIndex;
    },);
    return (row === undefined)
      ? archive
      : {
        sliceIndex: archive.sliceIndex,
        text: row.replacementText,
      };
  },);
  /**
   Destinations every slice's original carries.
   */
  const originalDestinations = destinationsOfSide({
    texts: originals,
    side: 'source',
  },);
  /**
   Destinations every slice's archive span carries.
   */
  const archiveDestinations = destinationsOfSide({
    texts: archives,
    side: 'archive',
  },);
  /**
   Destinations every slice's shipped text carries.
   */
  const shippedDestinations = destinationsOfSide({
    texts: shipped,
    side: 'page',
  },);
  return dropped.map(function trace(address,): DroppedDestinationTrace {
    return {
      sourceSlices: slicesCarrying({
        side: originalDestinations,
        address,
      },),
      archiveSlices: slicesCarrying({
        side: archiveDestinations,
        address,
      },),
      shippedSlices: slicesCarrying({
        side: shippedDestinations,
        address,
      },),
    };
  },);
}

/**
 Says where the original carries the dropped destinations, for a message.

 @param traces - one trace per dropped destination

 @returns A clause naming the slices, or saying they sit outside every slice

 @example
 ```ts
 whereCarried({ traces, },); // 'carried by the original in slice 4'
 ```
 */
function whereCarried(
  { traces, }: { readonly traces: readonly DroppedDestinationTrace[]; },
): string {
  /**
   Every slice whose original carries one, once each, in order.
   */
  const slices = [...new Set(traces.flatMap(function sourcesOf({ sourceSlices, },): readonly number[] {
    return sourceSlices;
  },),),].toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
  return (slices.length === 0)
    ? 'carried by the original outside every slice'
    : `carried by the original in ${
      wordForCount({
        count: slices.length,
        one: 'slice',
        many: 'slices',
      },)
    } ${
      slices.map(String,)
        .join(', ',)
    }`;
}

/**
 Defensive invariant when would-ship page loses source destinations.

 @example
 ```ts
 throw new DroppedDestinationError({ entryId: 'Cat', droppedCount: 1, traces, });
 ```
 */
export class DroppedDestinationError extends Error {
  /**
   Message contains operation names, counts and slice indices only.
   */
  readonly messageNamesOnly: true = true;
  /**
   Entry whose page failed invariant.
   */
  readonly entryId: string;
  /**
   Source destinations absent from would-ship page.
   */
  readonly droppedCount: number;
  /**
   Where each dropped destination sits among the slices.
   */
  readonly traces: readonly DroppedDestinationTrace[];

  /**
   @param entryId - affected entry

   @param droppedCount - missing destination count

   @param traces - where each dropped destination sits
   */
  public constructor(
    {
      entryId,
      droppedCount,
      traces,
    }: {
      readonly entryId: string;
      readonly droppedCount: number;
      readonly traces: readonly DroppedDestinationTrace[];
    },
  ) {
    super(
      `entry ${entryId} would drop ${String(droppedCount,)} source ${
        wordForCount({
          count: droppedCount,
          one: 'destination',
          many: 'destinations',
        },)
      }, ${whereCarried({ traces, },)}`,
    );
    this.name = 'DroppedDestinationError';
    this.entryId = entryId;
    this.droppedCount = droppedCount;
    this.traces = traces;
  }
}

/**
 Refuses persistence when deterministic source floor failed upstream.

 @param entryId - entry about to publish

 @param destinations - source and would-ship destination comparison

 @param traces - where each dropped destination sits, for the refusal

 @throws {@link DroppedDestinationError} when any source destination is absent

 @example
 ```ts
 assertDestinationsComplete({ entryId: 'Cat', destinations, traces, });
 ```
 */
export function assertDestinationsComplete(
  {
    entryId,
    destinations,
    traces,
  }: {
    readonly entryId: string;
    readonly destinations: DestinationCheck;
    readonly traces: readonly DroppedDestinationTrace[];
  },
): void {
  if (destinations.dropped
    .length
    === 0)
    return;
  throw new DroppedDestinationError({
    entryId,
    droppedCount: destinations.dropped
      .length,
    traces,
  },);
}

//endregion Destination completeness

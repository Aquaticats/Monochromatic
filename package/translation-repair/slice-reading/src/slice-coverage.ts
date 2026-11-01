import type { ChunkPair, } from './chunk-document.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type DocumentNode,
  UnhandledMemberInvariantError,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Slice coverage
// The one thing slicing cannot prove about itself: that carving a chunk pair
// into slices kept every block of it.
//
// `assertSpanContiguity` checks a slice against the range it claims, which
// catches a slice carrying fewer nodes than its own offsets cover. It cannot
// see a block that reached NO slice, because no range covers it and so no range
// disagrees with itself. That is a different failure and it is silent: the
// block leaves the document between alignment and the lanes, and every later
// reader works from the slices.
//
// WHAT IT COST ON 2026-08-20: `lintong`s closing paragraph, a friend's last
// message and its date, reached no slice while its English sat in a slice's
// incumbent. The repair lane, shown English with no original behind it, deleted
// the rendering of one clause and left a bare blockquote marker in the shipped
// text. Nothing reported anything; the entry settled with zero alignment
// findings.
//
// CHECKED PER SIDE AND IN ORDER, because the three failures are distinct and
// each is worth naming: a block placed nowhere, a block placed twice, which
// inflates a run past its budget, and blocks placed out of document order,
// which gathers text from two places into one slice.
//
// ONE TRANSLATION BLOCK MAY LEGITIMATELY REACH NO SLICE, and only one kind:
// one the roster's pairing accounted for nowhere. `declined-target-runs.ts`
// states why such a block has to leave every slice span rather than ride inside
// a neighbouring one, and a letter was once nearly deleted
// because it did not.
//
// THIS DOES NOT WEAKEN THE RULE, it sharpens it. The enemy here was never
// absence, it was SILENCE: the 2026-08-20 incident was a block vanishing with
// nothing recording it. So a target block must now be in a slice OR in the
// declined list, a block in NEITHER still fails, and a declined block that
// turns up in a slice anyway fails too, because then two parts of the
// preparation disagree about where it went.

/**
 How one side's blocks failed to reach the carved slices.

 BLOCK IDS ARE POSITIONS: `document-node.ts` stamps every block `block/N`
 from its place in top-level order, so a list of them names positions and
 never wording, and the sentence may print them.

 @example
 ```ts
 const placement: BlockPlacementFault = { kind: 'missing', missing: ['block/6',], expected: 7, };
 ```
 */
export type BlockPlacementFault = {
  /**
   Blocks the side was given that no slice placed.
   */
  readonly kind: 'missing';

  /**
   Ids of the blocks that reached no slice.
   */
  readonly missing: readonly string[];

  /**
   Blocks the side was given.
   */
  readonly expected: number;
} | {
  /**
   Blocks placed in more than one slice.
   */
  readonly kind: 'repeated';

  /**
   Ids of the blocks placed again.
   */
  readonly repeated: readonly string[];
} | {
  /**
   Every block placed once, out of document order.
   */
  readonly kind: 'out-of-order';

  /**
   Ids in the order the slices placed them.
   */
  readonly placed: readonly string[];
};

/**
 Why a chunk's carved slices do not cover it.

 @example
 ```ts
 const fault: SliceCoverageFault = { kind: 'declined-reached', sliceIndex: 3, contradicted: ['block/2',], };
 ```
 */
export type SliceCoverageFault = {
  /**
   Target blocks the pairing declined reached a slice anyway.
   */
  readonly kind: 'declined-reached';

  /**
   Chunk being sliced.
   */
  readonly sliceIndex: number;

  /**
   Ids of the declined blocks a slice placed.
   */
  readonly contradicted: readonly string[];
} | {
  /**
   Blocks the archive's note sealed reached a slice anyway.
   */
  readonly kind: 'sealed-reached';

  /**
   Chunk being sliced.
   */
  readonly sliceIndex: number;

  /**
   Side whose sealed blocks a slice placed.
   */
  readonly side: 'source' | 'target';

  /**
   Ids of the sealed blocks a slice placed.
   */
  readonly contradicted: readonly string[];
} | {
  /**
   One side's blocks were not placed exactly once, in order.
   */
  readonly kind: 'placement';

  /**
   Chunk being sliced.
   */
  readonly sliceIndex: number;

  /**
   Side whose blocks went astray.
   */
  readonly side: 'source' | 'target';

  /**
   How they went astray.
   */
  readonly placement: BlockPlacementFault;
};

/**
 Words a placement fault.

 @param placement - how one side's blocks went astray

 @returns Sentence composed from counts and block ids

 @example
 ```ts
 const sentence = blockPlacementSentence({ placement: { kind: 'repeated', repeated: ['block/2',], }, },);
 ```
 */
export function blockPlacementSentence(
  { placement, }: { readonly placement: BlockPlacementFault; },
): string {
  if (placement.kind === 'missing') {
    /**
     Ids that reached no slice, and how many blocks the side was given.
     */
    const {
      missing,
      expected,
    } = placement;
    return `${String(missing.length,)} of ${String(expected,)} ${
      wordForCount({
        count: expected,
        one: 'block',
        many: 'blocks',
      },)
    } reached no slice: ${missing.join(', ',)}`;
  }
  if (placement.kind === 'repeated') {
    /**
     Ids placed more than once.
     */
    const { repeated, } = placement;
    return `${String(repeated.length,)} ${
      wordForCount({
        count: repeated.length,
        one: 'block',
        many: 'blocks',
      },)
    } ${
      wordForCount({
        count: repeated.length,
        one: 'was',
        many: 'were',
      },)
    } placed more than once: ${repeated.join(', ',)}`;
  }

  /**
   Ids in the order the slices placed them.
   */
  const { placed, } = placement;
  return `blocks were placed out of document order: ${placed.join(', ',)}`;
}

/**
 Words a coverage fault, after the chunk the class prefixes.

 @param fault - why the carved slices do not cover the chunk

 @returns Sentence composed from a side name, counts and block ids

 @throws {@link UnhandledMemberInvariantError} when a member of `SliceCoverageFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = coverageSentence({ fault, },);
 ```
 */
export function coverageSentence({ fault, }: { readonly fault: SliceCoverageFault; },): string {
  if (fault.kind === 'declined-reached') {
    /**
     Ids of the declined blocks a slice placed.
     */
    const { contradicted, } = fault;
    return `target ${String(contradicted.length,)} declined ${
      wordForCount({
        count: contradicted.length,
        one: 'block',
        many: 'blocks',
      },)
    } reached a slice: ${contradicted.join(', ',)}`;
  }
  if (fault.kind === 'sealed-reached') {
    /**
     Ids of the sealed blocks a slice placed.
     */
    const { contradicted, } = fault;
    return `${fault.side} ${String(contradicted.length,)} sealed ${
      wordForCount({
        count: contradicted.length,
        one: 'block',
        many: 'blocks',
      },)
    } reached a slice: ${contradicted.join(', ',)}`;
  }
  if (fault.kind === 'placement')
    return `${fault.side} ${blockPlacementSentence({ placement: fault.placement, },)}`;
  throw new UnhandledMemberInvariantError({
    what: 'slice coverage fault',
    member: fault,
  },);
}

/**
 Failure of slice coverage: a side's blocks went missing, repeated or moved.

 MARKED: its message is a chunk index and the sentence `coverageSentence`
 writes from a side name, counts and positional block ids.

 @example
 ```ts
 throw new SliceCoverageError({ fault: { kind: 'declined-reached', sliceIndex: 3, contradicted: ['block/2',], }, },);
 ```
 */
export class SliceCoverageError extends Error {
  /**
   Declares this message safe to forward: an index, a side name, counts and
   positional block ids, in a sentence written here.
   */
  readonly messageNamesOnly: true = true;

  /**
   Why the carved slices do not cover the chunk.
   */
  readonly fault: SliceCoverageFault;

  /**
   @param fault - why the carved slices do not cover the chunk
   */
  public constructor({ fault, }: { readonly fault: SliceCoverageFault; },) {
    super(`slicing chunk ${String(fault.sliceIndex,)}: ${coverageSentence({ fault, },)}`,);
    this.name = 'SliceCoverageError';
    this.fault = fault;
  }
}

/**
 Reads the ids of node runs, in the order they were placed.

 @param runs - node runs, one per slice

 @returns Ids in placement order

 @example
 ```ts
 const placed = idsOf({ runs: [pair.source.nodes,], },);
 ```
 */
function idsOf(
  { runs, }: { readonly runs: readonly (readonly DocumentNode[])[]; },
): readonly string[] {
  return runs.flatMap(function toIds(run,): readonly string[] {
    return run.map(function toId(node,): string {
      return node.id;
    },);
  },);
}

/**
 Names how a side's placement departs from the blocks it was given.

 @param expected - ids the chunk pair carried, in document order

 @param placed - ids the slices carry, in placement order

 @returns Every way this side departed, empty when placement is exact

 @example
 ```ts
 const faults = describePlacement({ expected: ['block/0',], placed: [], },);
 ```
 */
function describePlacement(
  {
    expected,
    placed,
  }: {
    readonly expected: readonly string[];
    readonly placed: readonly string[];
  },
): readonly BlockPlacementFault[] {
  /**
   Ids the slices never carried.
   */
  const missing = expected.filter(function unplaced(id,): boolean {
    return !placed.includes(id,);
  },);
  if (missing.length > 0)
    return [
      {
        kind: 'missing',
        missing,
        expected: expected.length,
      },
    ];

  /**
   Ids carried by more than one slice, or twice by one.
   */
  const repeated = placed.filter(function isRepeat(
    id,
    at,
  ): boolean {
    return placed.indexOf(id,) !== at;
  },);
  if (repeated.length > 0)
    return [
      {
        kind: 'repeated',
        repeated,
      },
    ];
  if (placed.join(' ',) !== expected.join(' ',))
    return [
      {
        kind: 'out-of-order',
        placed,
      },
    ];
  return [];
}

/**
 Asserts that slices carved from a chunk pair carry its blocks exactly once.

 @param pair - chunk pair that went in

 @param carved - slices it was carved into

 @param declined - ids of translation blocks the pairing accounted for
 nowhere, which reach no slice by decision

 @param sealed - ids, per side, of the blocks the archive's note seals,
 which reach no slice by the owner's rule of 2026-09-08

 @throws SliceCoverageError when either side loses, repeats or reorders a block

 @example
 ```ts
 assertSliceCoverage({ pair, carved, },);
 ```
 */
export function assertSliceCoverage(
  {
    pair,
    carved,
    declined = new Set<string>(),
    sealed = {
      source: new Set<string>(),
      target: new Set<string>(),
    },
  }: {
    readonly pair: ChunkPair;
    readonly carved: readonly ChunkPair[];
    readonly declined?: ReadonlySet<string>;
    readonly sealed?: {
      readonly source: ReadonlySet<string>;
      readonly target: ReadonlySet<string>;
    };
  },
): void {
  /**
   Translation blocks the slices carry, needed before the sides are built so a
   declined block turning up in one can be named as its own fault.
   */
  const placedTargets = idsOf({
    runs: carved.map(function toRun(slice,): readonly DocumentNode[] {
      return slice.target
        .nodes;
    },),
  },);

  /**
   Original blocks the slices carry.
   */
  const placedSources = idsOf({
    runs: carved.map(function toRun(slice,): readonly DocumentNode[] {
      return slice.source
        .nodes;
    },),
  },);

  /**
   Declined blocks a slice carries anyway, which contradicts the decline.
   */
  const contradicted = placedTargets.filter(function isDeclined(id,): boolean {
    return declined.has(id,);
  },);
  if (contradicted.length > 0)
    throw new SliceCoverageError({
      fault: {
        kind: 'declined-reached',
        sliceIndex: pair.source
          .sliceIndex,
        contradicted,
      },
    },);
  /**
   Sealed blocks a slice carries anyway, per side, which contradicts the seal.
   */
  const sealedReached = [
    {
      side: 'source' as const,
      contradicted: placedSources.filter(function isSealed(id,): boolean {
        return sealed.source
          .has(id,);
      },),
    },
    {
      side: 'target' as const,
      contradicted: placedTargets.filter(function isSealed(id,): boolean {
        return sealed.target
          .has(id,);
      },),
    },
  ];
  for (const reached of sealedReached) {
    if (reached.contradicted
      .length
      > 0)
      throw new SliceCoverageError({
        fault: {
          kind: 'sealed-reached',
          sliceIndex: pair.source
            .sliceIndex,
          side: reached.side,
          contradicted: reached.contradicted,
        },
      },);
  }
  /**
   Both sides, each with the blocks it was given and the blocks it placed.
   */
  const sides = [
    {
      name: 'source' as const,
      // Sealed originals are the back-translation of sealed blocks and are
      // not expected in a slice either.
      expected: idsOf({
        runs: [ pair.source
          .nodes, ],
      },)
        .filter(function isNotSealed(id,): boolean {
          return !sealed.source
            .has(id,);
        },),
      placed: placedSources,
    },
    {
      name: 'target' as const,
      // Declined and sealed blocks are not expected in a slice, so they drop
      // out of the comparison entirely rather than reading as losses.
      expected: idsOf({
        runs: [ pair.target
          .nodes, ],
      },)
        .filter(function isNotDeclinedOrSealed(id,): boolean {
          return (!declined.has(id,)) && (!sealed.target
            .has(id,));
        },),
      placed: placedTargets,
    },
  ];
  for (const side of sides) {
    /**
     How this side departed from its blocks, empty when it did not.
     */
    const faults = describePlacement({
      expected: side.expected,
      placed: side.placed,
    },);
    for (const placement of faults)
      throw new SliceCoverageError({
        fault: {
          kind: 'placement',
          sliceIndex: pair.source
            .sliceIndex,
          side: side.name,
          placement,
        },
      },);
  }
}

//endregion Slice coverage

import { mapOverlapped, } from '@monochromatic-dev/translation-repair-stage/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { FidelityReferenceSpec, } from '../fidelity-reference-model.ts';
import {
  isPlainEntryId,
  seededEntryOrder,
} from '../editor-fidelity-pool-rule.ts';
import {
  drawPoolEntry,
  type DrawnEntry,
} from './editor-fidelity-pool-draw-entry.ts';

//region Editor fidelity pool draw
// THE DRAW OF THE WHOLE POOL: every entry of the pinned corpus that could be drawn is evaluated, the entries are put in the
// order a seed gives (a sample of the eligible entries, not an alphabetical prefix), and the draw takes entries in that
// order until the pool, reviewed trials counted, holds the trials wanted. Evaluating every entry first is what lets the
// draw say how many entries could have been drawn, which is the population the sample is taken from.

/**
 Logger the overlapped map writes a failed entry's line to.
 */
const l = contextRoot({ tag: 'editor-fidelity-pool-draw', },);

/**
 What reading an entry gives: its two pages, or that it cannot be read.

 @example
 ```ts
 const pages: EntryPages = { zh: '', en: '', };
 ```
 */
export type EntryPages =
  | {
    /**
     Original page.
     */
    readonly zh: string;

    /**
     Archive page.
     */
    readonly en: string;
  }
    | {
    /**
     The entry has no readable pair of pages.
     */
      readonly unreadable: true;
    };

/**
 What a draw took, and the population it was taken from.

 @example
 ```ts
 const draw: PoolDraw = await drawPool({ entryIds, readEntry, seed, want, reviewedTrials, reviewedEntryIds, corpusSha, reviewedOn, },);
 ```
 */
export type PoolDraw = {
  /**
   Specifications taken, in the order of the draw.
   */
  readonly specs: readonly FidelityReferenceSpec[];

  /**
   Entries taken, in the order of the draw.
   */
  readonly drawnEntries: readonly string[];

  /**
   Entries that could have been drawn: plain ids, readable pages of equal shape, at least one scoreable trial, and not a
   reviewed entry.
   */
  readonly eligibleEntries: number;

  /**
   Trials taken, not counting the reviewed ones.
   */
  readonly trials: number;

  /**
   Trials taken of each kind.
   */
  readonly kinds: {
    readonly insertion: number;
    readonly alteration: number
  };

  /**
   Why entries and sections were left out, each reason with how many times.
   */
  readonly skipped: Readonly<Record<string, number>>;

  /**
   Characters of the sheet of each trial taken.
   */
  readonly sheetChars: readonly number[];
};

/**
 One entry's draw with its id.
 */
type EntryDraw = DrawnEntry & {
  /**
   The entry.
   */
  readonly entryId: string;
};

/**
 Counts how many times each reason occurs.

 @param reasons - reasons, one for each thing left out

 @returns Each reason with its count

 @example
 ```ts
 const counts = countReasons({ reasons: ['a', 'a', 'b'], },);
 ```
 */
function countReasons({ reasons, }: { readonly reasons: readonly string[]; },): Readonly<Record<string, number>> {
  return Object.fromEntries(new Set(reasons,)
    .values()
    .map(function counted(reason,): readonly [
      string,
      number
    ] {
      return [
        reason,
        reasons.filter(function is(other,): boolean {
          return other === reason;
        },)
          .length,
      ];
    },),);
}

/**
 Counts the trials of one kind in specifications.

 @param specs - specifications counted

 @param kind - kind counted

 @returns How many damages of that kind they hold

 @example
 ```ts
 const insertions = countKind({ specs, kind: 'insertion', },);
 ```
 */
function countKind(
  {
    specs,
    kind,
  }: {
    readonly specs: readonly FidelityReferenceSpec[];
    readonly kind: 'insertion' | 'alteration';
  },
): number {
  return specs.flatMap(function damagesOf(spec,) {
    return spec.damages;
  },)
    .filter(function isKind(damage,): boolean {
      return damage.kind === kind;
    },)
    .length;
}

/**
 Names an entry on the line written for a failed one.

 @param item - the row of the entry

 @returns The entry's name

 @example
 ```ts
 const name = entryName({ item: 'tabby', },);
 ```
 */
function entryName({ item, }: { readonly item: string; },): string {
  return `entry ${item}`;
}

/**
 Reads and draws one entry.

 @param entryId - the entry

 @param readEntry - reads the two pages of an entry

 @param corpusSha - commit the pages are read at

 @param reviewedOn - date of the freeze

 @returns The entry's draw, with the reason when its pages cannot be read

 @example
 ```ts
 const entry = await evaluatedEntry({ entryId, readEntry, corpusSha, reviewedOn, },);
 ```
 */
async function evaluatedEntry(
  {
    entryId,
    readEntry,
    corpusSha,
    reviewedOn,
  }: {
    readonly entryId: string;
    readonly readEntry: (input: { readonly entryId: string; },) => Promise<EntryPages>;
    readonly corpusSha: string;
    readonly reviewedOn: string;
  },
): Promise<EntryDraw> {
  /**
   The pages of this entry.
   */
  const pages = await readEntry({ entryId, },);
  if ('unreadable' in pages) {
    return {
      entryId,
      specs: [],
      skipped: ['entry unreadable',],
      sheetChars: [],
    };
  }
  return {
    entryId,
    ...await drawPoolEntry({
      entryId,
      zh: pages.zh,
      en: pages.en,
      corpusSha,
      reviewedOn,
    },),
  };
}

/**
 Takes the leading entries until the pool, reviewed trials counted, holds the trials wanted.

 @param eligible - entries that gave trials, in the seed's order

 @param reviewedTrials - trials the reviewed references already hold

 @param want - trials the pool should hold

 @returns The entries up to and including the one that reaches the trials wanted, or all of them when none does

 @example
 ```ts
 const taken = leadingEntries({ eligible, reviewedTrials: 4, want: 40, },);
 ```
 */
function leadingEntries(
  {
    eligible,
    reviewedTrials,
    want,
  }: {
    readonly eligible: readonly EntryDraw[];
    readonly reviewedTrials: number;
    readonly want: number;
  },
): readonly EntryDraw[] {
  if (reviewedTrials >= want)
    return [];
  /**
   Position of the entry that reaches the trials wanted, none (minus one) when the entries do not.
   */
  const reachedAt = eligible.findIndex(function reaches(
    _entry,
    position,
  ): boolean {
    return (reviewedTrials
      + eligible.slice(
        0,
        position + 1,
      )
        .reduce(
        function sum(
          total: number,
          other: EntryDraw,
        ): number {
          return total + trialsIn({ specs: other.specs, },);
        },
        0,
        )) >= want;
  },);
  return (reachedAt === (-1)) ? eligible : eligible.slice(
    0,
    reachedAt + 1,
  );
}

/**
 Counts the trials of specifications, one for each damage.

 @param specs - specifications counted

 @returns How many damages they hold

 @example
 ```ts
 const trials = trialsIn({ specs, },);
 ```
 */
function trialsIn({ specs, }: { readonly specs: readonly FidelityReferenceSpec[]; },): number {
  return specs.reduce(
    function sum(
      total: number,
      spec: FidelityReferenceSpec,
    ): number {
      return total
        + spec.damages
          .length;
    },
    0,
  );
}

/**
 Draws the pool.

 @param entryIds - every entry of the corpus, in any order

 @param readEntry - reads the two pages of an entry

 @param seed - seed of the draw

 @param want - trials the pool should hold, reviewed ones counted

 @param reviewedTrials - trials the reviewed references already hold

 @param reviewedEntryIds - entries the reviewed references come from, which the draw leaves out

 @param corpusSha - commit the pages are read at

 @param reviewedOn - date of the freeze

 @returns The specifications taken, the entries they came from and the population they were taken from

 @example
 ```ts
 const draw = await drawPool({ entryIds, readEntry, seed: 'round-4', want: 40, reviewedTrials: 4, reviewedEntryIds, corpusSha, reviewedOn, },);
 ```
 */
export async function drawPool(
  {
    entryIds,
    readEntry,
    seed,
    want,
    reviewedTrials,
    reviewedEntryIds,
    corpusSha,
    reviewedOn,
  }: {
    readonly entryIds: readonly string[];
    readonly readEntry: (input: { readonly entryId: string; },) => Promise<EntryPages>;
    readonly seed: string;
    readonly want: number;
    readonly reviewedTrials: number;
    readonly reviewedEntryIds: readonly string[];
    readonly corpusSha: string;
    readonly reviewedOn: string;
  },
): Promise<PoolDraw> {
  /**
   Entries with a plain id, and how many had none.
   */
  const plain = entryIds.filter(function isPlain(entryId,): boolean {
    return isPlainEntryId({ entryId, },);
  },);
  /**
   The entries the seed visits, in its order, reviewed ones left out.
   */
  const ordered = seededEntryOrder({
    entryIds: plain.filter(function notReviewed(entryId,): boolean {
      return !reviewedEntryIds.includes(entryId,);
    },),
    seed,
  },);
  /**
   Every entry evaluated, one after the other: a clone is read through git one entry at a time, and the order of the
   results is the seed's order.
   */
  const evaluated = await mapOverlapped({
    items: ordered,
    overlap: 1,
    signal: new AbortController().signal,
    l,
    nameOf: entryName,
    oneItem: function evaluate({ item: entryId, },): Promise<EntryDraw> {
      return evaluatedEntry({
        entryId,
        readEntry,
        corpusSha,
        reviewedOn,
      },);
    },
  },);
  /**
   Entries that gave at least one trial, in the seed's order.
   */
  const eligible = evaluated.filter(function gaveTrials(entry,): boolean {
    return entry.specs
      .length
      > 0;
  },);
  /**
   Entries taken: the leading ones, up to and including the one that reaches the trials wanted.
   */
  const taken = leadingEntries({
    eligible,
    reviewedTrials,
    want,
  },);
  /**
   Specifications of the entries taken.
   */
  const specs = taken.flatMap(function specsOf(entry,): readonly FidelityReferenceSpec[] {
    return entry.specs;
  },);
  /**
   Kinds of the trials taken.
   */
  const kinds = {
    insertion: countKind({
      specs,
      kind: 'insertion',
    },),
    alteration: countKind({
      specs,
      kind: 'alteration',
    },),
  };
  return {
    specs,
    drawnEntries: taken.map(function idOf(entry,): string {
      return entry.entryId;
    },),
    eligibleEntries: eligible.length,
    trials: kinds.insertion + kinds.alteration,
    kinds,
    skipped: countReasons({
      reasons: [
        ...Array.from(
          { length: entryIds.length - plain.length, },
          function notPlain(): string {
            return 'entry id not plain';
          },
        ),
        ...evaluated.flatMap(function reasonsOf(entry,): readonly string[] {
          return entry.skipped;
        },),
      ],
    },),
    sheetChars: taken.flatMap(function charsOf(entry,): readonly number[] {
      return entry.sheetChars;
    },),
  };
}

//endregion Editor fidelity pool draw

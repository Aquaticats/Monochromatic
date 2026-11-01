import { howOften, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ProtectedAtom, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  describeAtom,
  mergeAtoms,
} from './translate-atom-floor.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Rewritten renderings
// WHERE THE PAGE RENDERS A REFERENCE ANOTHER WAY, a candidate owes one
// rendering, not both. Decided by the owner on 2026-09-04 ("Either
// rendering") after the luxuanwen3 pass of that day stopped at its first
// paragraph: the original links twitter.com, the archive's page links x.com,
// and the per-key maximum of `mergeAtoms` owed a candidate both, so the
// archive's own paragraph was ineligible and no proposal could pass. Measured
// over the pinned corpus: eight of ninety-three entries carry a link the
// archive rewrote (a moved domain, a trailing slash, a same-language
// Wikipedia article); footnotes never diverge both ways.
//
// A REWRITE IS READ OFF THE SET DIFFERENCE, per atom kind: atoms only the
// original carries and atoms only the page carries, of the same kind, form
// one pool, and the candidate owes exactly the larger side's count from it.
// That is the block rule's arithmetic (the page is a floor, the original
// sets the ceiling with it) applied to references. A kind that diverges in
// one direction only is an addition or a drop and stays owed as before;
// carrying both renderings is refused, since a link the original carries
// once and the page carries once is one link on the page.

/**
 Atoms of one kind the original and the page render differently.
 */
export type AtomRenderingPool = {
  /**
   Atom kind every member shares.
   */
  readonly kind: ProtectedAtom['kind'];

  /**
   Keys only the original carries, repeated per copy.
   */
  readonly fromSource: readonly string[];

  /**
   Keys only the page carries, repeated per copy.
   */
  readonly fromPage: readonly string[];

  /**
   Copies a candidate must draw from the pool, the larger side's count.
   */
  readonly owed: number;
};

/**
 One atom by kind and exact key.
 */
type KeyedAtom = {
  /**
   Atom kind.
   */
  readonly kind: ProtectedAtom['kind'];

  /**
   Exact key from {@link describeAtom}.
   */
  readonly key: string;
};

/**
 Keys one side carries and the other does not, keeping copies.

 @param atoms - atoms of the side being read

 @param other - atoms of the side compared against

 @returns Keys absent from the other side, one per copy

 @example
 ```ts
 const onlyHere = keysAbsentFrom({ atoms: source, other: page, },);
 ```
 */
function keysAbsentFrom(
  {
    atoms,
    other,
  }: {
    readonly atoms: readonly ProtectedAtom[];
    readonly other: readonly ProtectedAtom[];
  },
): readonly KeyedAtom[] {
  /**
   Keys the other side carries at all.
   */
  const otherKeys = new Set(other.map(function toKey(atom,): string {
    return describeAtom(atom,);
  },),);
  return atoms
    .map(function toKeyed(atom,): KeyedAtom {
      return {
        kind: atom.kind,
        key: describeAtom(atom,),
      };
    },)
    .filter(function isAbsent(keyed,): boolean {
      return !otherKeys.has(keyed.key,);
    },);
}

/**
 Pools of atoms the original and the page render differently, one per
 kind that diverges in both directions.

 @param page - atoms the text being replaced carries

 @param source - atoms the original carries

 @returns One pool per kind with members on both sides, in kind order of
 first appearance in the original

 @example
 ```ts
 const pools = renderingPoolsOf({ page: page.atoms, source: expected.atoms, },);
 ```
 */
export function renderingPoolsOf(
  {
    page,
    source,
  }: {
    readonly page: readonly ProtectedAtom[];
    readonly source: readonly ProtectedAtom[];
  },
): readonly AtomRenderingPool[] {
  /**
   Original's members the page lacks.
   */
  const sourceOnly = keysAbsentFrom({
    atoms: source,
    other: page,
  },);
  /**
   Page's members the original lacks.
   */
  const pageOnly = keysAbsentFrom({
    atoms: page,
    other: source,
  },);
  /**
   Kinds that diverge in both directions, in original order.
   */
  const kinds = [...new Set(sourceOnly.map(function toKind(keyed,): ProtectedAtom['kind'] {
    return keyed.kind;
  },),),].filter(function divergesBothWays(kind,): boolean {
    return pageOnly.some(function isKind(keyed,): boolean {
      return keyed.kind === kind;
    },);
  },);
  return kinds.map(function toPool(kind,): AtomRenderingPool {
    /**
     Original's keys of this kind.
     */
    const fromSource = sourceOnly
      .filter(function isKind(keyed,): boolean {
        return keyed.kind === kind;
      },)
      .map(function toKey(keyed,): string {
        return keyed.key;
      },);
    /**
     Page's keys of this kind.
     */
    const fromPage = pageOnly
      .filter(function isKind(keyed,): boolean {
        return keyed.kind === kind;
      },)
      .map(function toKey(keyed,): string {
        return keyed.key;
      },);
    return {
      kind,
      fromSource,
      fromPage,
      owed: Math.max(
        fromSource.length,
        fromPage.length,
      ),
    };
  },);
}

/**
 How many times each atom appears, keyed by exact description in order of
 first appearance.

 @param atoms - atoms one side carries

 @returns Copies per key

 @example
 ```ts
 const carried = copiesByKey({ atoms: candidate, },);
 ```
 */
function copiesByKey({ atoms, }: { readonly atoms: readonly ProtectedAtom[]; },): ReadonlyMap<string, number> {
  /**
   Copies counted so far.
   */
  const counted = new Map<string, number>();
  for (const atom of atoms) {
    /**
     Key identifying this atom exactly.
     */
    const key = describeAtom(atom,);
    counted.set(
      key,
      (counted.get(key,) ?? 0) + 1,
    );
  }
  return counted;
}

/**
 Copies of each pool member a candidate carries, counted against the copies
 the pool lists, so a member carried beyond its own count draws no more.

 @param pool - renderings the references disagree on

 @param carried - copies the candidate carries per key

 @returns Copies drawn from the pool

 @example
 ```ts
 const drawn = drawnFrom({ pool, carried, },);
 ```
 */
function drawnFrom(
  {
    pool,
    carried,
  }: {
    readonly pool: AtomRenderingPool;
    readonly carried: ReadonlyMap<string, number>;
  },
): number {
  /**
   Members of both sides, repeated per copy.
   */
  const members = [
    ...pool.fromSource,
    ...pool.fromPage,
  ];
  return [...new Set(members,),].reduce(
    function addMember(
      sum,
      key,
    ): number {
      /**
       Copies the pool lists of this member.
       */
      const listed = members
        .filter(function isMember(member,): boolean {
          return member === key;
        },)
        .length;
      return sum + Math.min(
        listed,
        carried.get(key,) ?? 0,
      );
    },
    0,
  );
}

// LEDGER B37 (2026-09-30). The floor pushed one sentence per missing copy and
// one per atom carried too often, each saying the other side carried none: a
// candidate carrying a link once where the original carries it twice was told
// "your translation does not" carry it, and one carrying a link three times
// where the original carries it once was told "the ORIGINAL does not". A model
// told a side lacks an atom it has can remove every copy and be refused again
// for the drop. Each atom now draws one sentence naming both counts.

/**
 Findings for atoms the candidate owes and did not carry, invented, or
 drew from a rendering pool in the wrong number.

 Compared as a MULTISET rather than in order, because a translation
 reorders clauses legitimately and a link moving within a sentence is not
 damage. What is damage is a reference that stopped existing, one that
 appeared from nowhere, or both renderings of one reference side by side.

 @param page - atoms the text being replaced carries

 @param source - atoms the original carries

 @param candidate - atoms the candidate carries

 @param referenceName - what a finding calls the merged reference

 @returns One finding per atom carried fewer or more times than the
 references carry it, naming both counts, and one per pool drawn from in the
 wrong number

 @example
 ```ts
 const findings = atomFindings({ page, source, candidate, referenceName: 'ORIGINAL', },);
 ```
 */
export function atomFindings(
  {
    page,
    source,
    candidate,
    referenceName,
  }: {
    readonly page: readonly ProtectedAtom[];
    readonly source: readonly ProtectedAtom[];
    readonly candidate: readonly ProtectedAtom[];
    readonly referenceName: string;
  },
): readonly string[] {
  /**
   Pools of renderings the two references disagree on.
   */
  const pools = renderingPoolsOf({
    page,
    source,
  },);
  /**
   Pool each pooled key belongs to.
   */
  const poolOfKey = new Map<string, AtomRenderingPool>(
    pools.flatMap(function toEntries(pool,): readonly (readonly [
      string,
      AtomRenderingPool,
    ])[] {
      return [
        ...pool.fromSource,
        ...pool.fromPage,
      ].map(function toEntry(key,): readonly [
        string,
        AtomRenderingPool,
      ] {
        return [
          key,
          pool,
        ];
      },);
    },),
  );
  /**
   How many times the candidate carries each atom, in the candidate's order.
   */
  const carried = copiesByKey({ atoms: candidate, },);
  /**
   How many times the references carry each atom, the larger side's count,
   in the references' order.
   */
  const owed = copiesByKey({
    atoms: mergeAtoms({
      page,
      source,
    },),
  },);
  /**
   Atoms the references carry more often than the candidate, outside the
   pools, which are counted whole.
   */
  const missing = [...owed.entries(),]
    .filter(function isShort([key, count,]: ReadonlyEntry<string, number>,): boolean {
      return (!poolOfKey.has(key,)) && ((carried.get(key,) ?? 0) < count);
    },)
    .map(function toFinding([key, count,]: ReadonlyEntry<string, number>,): string {
      /**
       Copies the candidate carries.
       */
      const has = carried.get(key,) ?? 0;
      return (has === 0)
        ? `The ${referenceName} carries ${key}${
          (count === 1) ? '' : ` ${howOften({ count, },)}`
        } and your translation does not.`
        : `The ${referenceName} carries ${key} ${howOften({ count, },)} and your translation carries it ${
          howOften({ count: has, },)
        }.`;
    },);
  /**
   Pools drawn from in the wrong number.
   */
  const misdrawn = pools.flatMap(function toFinding(pool,): readonly string[] {
    /**
     Copies the candidate drew from the pool.
     */
    const drawn = drawnFrom({
      pool,
      carried,
    },);
    if (drawn === pool.owed)
      return [];
    /**
     Original's renderings, listed.
     */
    const fromSource = pool.fromSource
      .join(', ',);
    /**
     Page's renderings, listed.
     */
    const fromPage = pool.fromPage
      .join(', ',);
    return [
      `The ORIGINAL carries ${fromSource} where the PAGE AS IT STANDS carries ${fromPage}: the page rendered the original's reference another way, and your translation must carry exactly ${
        String(pool.owed,)
      } of these, taken from either side; it carries ${String(drawn,)}.`,
    ];
  },);
  /**
   Atoms the candidate carries more often than the references.
   */
  const surplus = [...carried.entries(),]
    .filter(function isSurplus([key, count,]: ReadonlyEntry<string, number>,): boolean {
      return count > (owed.get(key,) ?? 0);
    },)
    .map(function toFinding([key, count,]: ReadonlyEntry<string, number>,): string {
      /**
       Copies the references carry.
       */
      const wanted = owed.get(key,) ?? 0;
      return (wanted === 0)
        ? `Your translation carries ${key}${
          (count === 1) ? '' : ` ${howOften({ count, },)}`
        } and the ${referenceName} does not.`
        : `Your translation carries ${key} ${howOften({ count, },)} and the ${referenceName} carries it ${
          howOften({ count: wanted, },)
        }.`;
    },);
  return [
    ...missing,
    ...misdrawn,
    ...surplus,
  ];
}

//endregion Rewritten renderings

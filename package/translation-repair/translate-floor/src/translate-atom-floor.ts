import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ProtectedAtom, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Translation atom floor

/**
 Renders one atom for finding and multiset key.

 @param atom - atom to describe

 @returns Kind and value

 @example
 ```ts
 const label = describeAtom({ kind: 'footnote', value: '1', },);
 ```
 */
export function describeAtom(atom: ProtectedAtom,): string {
  return `${atom.kind} ${atom.value}`;
}

/**
 One atom and count in reference.
 */
type AtomTally = {
  /**
   Atom being counted.
   */
  readonly atom: ProtectedAtom;

  /**
   Copies carried.
   */
  readonly count: number;
};

/**
 How many times each atom appears,
 keyed by exact description.

 @param atoms - atoms one side carries

 @returns One entry per distinct atom and count

 @example
 ```ts
 const counted = countAtoms({ atoms, });
 ```
 */
function countAtoms(
  { atoms, }: { readonly atoms: readonly ProtectedAtom[]; },
): ReadonlyMap<string, AtomTally> {
  return atoms.reduce(
    function tally(
      seen: Map<string, AtomTally>,
      atom: ProtectedAtom,
    ): Map<string, AtomTally> {
      /**
       Exact atom key.
       */
      const key = describeAtom(atom,);
      /**
       Existing tally when seen before.
       */
      const counted = seen.get(key,);
      /**
       Copies before current atom.
       */
      const before = (counted === undefined) ? 0 : counted.count;
      seen.set(
        key,
        {
          atom,
          count: before + 1,
        },
      );
      return seen;
    },
    new Map<string, AtomTally>(),
  );
}

/**
 Atoms candidate owes,
 taking whichever reference asks for more copies.

 @param page - atoms text being replaced carries

 @param source - atoms original carries

 @returns Union with higher count per atom

 @example
 ```ts
 const owed = mergeAtoms({ page, source, });
 ```
 */
export function mergeAtoms(
  {
    page,
    source,
  }: {
    readonly page: readonly ProtectedAtom[];
    readonly source: readonly ProtectedAtom[];
  },
): readonly ProtectedAtom[] {
  /**
   Counts requested by page.
   */
  const fromPage = countAtoms({ atoms: page, },);
  /**
   Counts requested by source.
   */
  const fromSource = countAtoms({ atoms: source, },);
  /**
   Distinct keys either reference carries.
   */
  const keys = [
    ...new Set([
      ...fromPage.keys(),
      ...fromSource.keys(),
    ],),
  ];
  return keys.flatMap(function expand(key,): readonly ProtectedAtom[] {
    /**
     Page tally for key.
     */
    const byPage = fromPage.get(key,);
    /**
     Source tally for key.
     */
    const bySource = fromSource.get(key,);
    /**
     Preferred record describing atom; one of the two tallies holds every
     key, since the keys are theirs (ledger T8: the throw that stood here for
     a key in neither could not run, and went with its class).
     */
    const held = nonNullishOrThrow(bySource ?? byPage,);
    /**
     Copies page requests.
     */
    const wantedByPage = (byPage === undefined) ? 0 : byPage.count;
    /**
     Copies source requests.
     */
    const wantedBySource = (bySource === undefined) ? 0 : bySource.count;
    /**
     Larger reference count candidate owes.
     */
    const owed = Math.max(
      wantedByPage,
      wantedBySource,
    );
    return Array.from(
      { length: owed, },
      function copy(): ProtectedAtom {
        return held.atom;
      },
    );
  },);
}

//endregion Translation atom floor

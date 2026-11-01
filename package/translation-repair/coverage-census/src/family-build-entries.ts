import { nodeEntries as benchEntries, } from '@monochromatic-dev/translation-repair-bench/ts';
import { nodeEntries as corpusPassEntries, } from '@monochromatic-dev/translation-repair-corpus-pass/ts';
import { nodeEntries as probeEntries, } from '@monochromatic-dev/translation-repair-probe/ts';
import { nodeEntries as runEntries, } from '@monochromatic-dev/translation-repair-run/ts';
import { nodeEntries as sliceCacheEntries, } from '@monochromatic-dev/translation-repair-slice-cache/ts';
import { nodeEntries as ownEntries, } from './build-entries.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Family build entries
// The census tells a runner's command-line main from library source by the
// build entry lists (ledger T8). Before the translation-repair package was
// split, one list named every runner. Each package that owns runners now lists
// its own in `src/build-entries.ts`, so the census reads the union of those
// lists, or it would take every runner another package owns for library
// source. `family-build-entries.unit.test.ts` holds `FAMILY_ENTRY_LISTS` to the
// packages that have a list, so a package that gains one cannot be missed.

/**
 Names a build entry two packages list at two paths.

 @example
 ```ts
 throw new FamilyEntryConflictError({ entry: 'purr', firstOwner: 'tabby', firstPath: './src/purr.ts', secondOwner: 'calico', secondPath: './src/other.ts', },);
 ```
 */
export class FamilyEntryConflictError extends Error {
  /**
   Declares this message safe to forward: it names a build entry and the
   family packages and paths whose lists carry it, all written in the
   packages' own `src/build-entries.ts`.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the entry and both lists.

   @param entry - entry name both lists carry

   @param firstOwner - package whose list was read first

   @param firstPath - path that list gives the entry

   @param secondOwner - package whose list conflicts

   @param secondPath - path that list gives the entry

   @example
   ```ts
   new FamilyEntryConflictError({ entry: 'purr', firstOwner: 'tabby', firstPath: './src/purr.ts', secondOwner: 'calico', secondPath: './src/other.ts', },);
   ```
   */
  constructor(
    {
      entry,
      firstOwner,
      firstPath,
      secondOwner,
      secondPath,
    }: {
      readonly entry: string;
      readonly firstOwner: string;
      readonly firstPath: string;
      readonly secondOwner: string;
      readonly secondPath: string;
    },
  ) {
    super(
      `build entry ${entry} is listed at ${firstPath} by ${firstOwner} and at ${secondPath} by ${secondOwner}, `
        + 'so the census cannot tell which runner it names',
    );
    this.name = 'FamilyEntryConflictError';
  }
}

/**
 Every package's own list, by the package's directory name in the family.

 @example
 ```ts
 const lists = FAMILY_ENTRY_LISTS.get('slice-cache',); // the entries `slice-cache` builds
 ```
 */
export const FAMILY_ENTRY_LISTS: ReadonlyMap<string, ReadonlyMap<string, string>> = new Map([
  [
    'bench',
    benchEntries,
  ],
  [
    'corpus-pass',
    corpusPassEntries,
  ],
  [
    'coverage-census',
    ownEntries,
  ],
  [
    'probe',
    probeEntries,
  ],
  [
    'run',
    runEntries,
  ],
  [
    'slice-cache',
    sliceCacheEntries,
  ],
],);

/**
 The package whose list gave an entry, and the path that list gave it.
 */
type EntryOwner = {
  readonly owner: string;
  readonly path: string;
};

/**
 Merges the packages' lists into one, in the order given, an entry two lists
 carry at one path kept once.

 @param lists - each package's list by the package's name

 @returns The union, as one list named every runner before the split

 @throws FamilyEntryConflictError where two lists name one entry at two paths

 @example
 ```ts
 const entries = familyEntriesOf({ lists: FAMILY_ENTRY_LISTS, },);
 ```
 */
export function familyEntriesOf(
  { lists, }: { readonly lists: ReadonlyMap<string, ReadonlyMap<string, string>>; },
): ReadonlyMap<string, string> {
  /**
   Entries merged so far, and the package whose list gave each.
   */
  const merged = new Map<string, EntryOwner>();
  for (const [owner, list,] of lists) {
    for (const [entry, path,] of list) {
      /**
       The list that gave this entry before, if one did.
       */
      const first = merged.get(entry,);
      if (first === undefined) {
        merged.set(
          entry,
          {
            owner,
            path,
          },
        );
        continue;
      }
      if (first.path !== path)
        throw new FamilyEntryConflictError({
          entry,
          firstOwner: first.owner,
          firstPath: first.path,
          secondOwner: owner,
          secondPath: path,
        },);
    }
  }
  return new Map([...merged,].map(function pathOf([entry, { path, },]: ReadonlyEntry<string, EntryOwner>,): readonly [
    string,
    string
  ] {
    return [
      entry,
      path,
    ];
  },),);
}

/**
 Named entries of every package's normal build and coverage build, each a path
 from its package directory: what one list named before the split. A map, as
 every table keyed by a name is (ledger B77).

 @example
 ```ts
 const runner = nodeEntries.get('slice-census',); // './src/corpus-run/slice-census.ts'
 ```
 */
export const nodeEntries: ReadonlyMap<string, string> = familyEntriesOf({ lists: FAMILY_ENTRY_LISTS, },);

/**
 Source files of the runner entries of every package, as the census names a
 source inside its package: from the package directory, with no leading `./`.
 The library index is left out, since what it reaches is library source. Every
 entry is written from `./src/`, which each package's list is held to, so the
 prefix is cut without asking.

 @returns Runner entry sources

 @example
 ```ts
 runnerEntrySources().has('src/corpus-run/corpus-pass.ts',); // true
 ```
 */
export function runnerEntrySources(): ReadonlySet<string> {
  return new Set(
    [...nodeEntries,]
      .filter(function isRunner([name,]: ReadonlyEntry<string, string>,): boolean {
        return name !== 'index';
      },)
      .map(function sourceOf([, path,]: ReadonlyEntry<string, string>,): string {
        return path.slice('./'.length,);
      },),
  );
}

//endregion Family build entries

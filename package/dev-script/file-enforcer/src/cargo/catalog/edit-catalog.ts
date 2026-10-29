/**
 Text splices for `rust-catalog.toml`: adopted entries, raised floors, and pruned entries.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/module-rust-catalog/ts';

import {
  childPairs,
  type LocatedDocument,
  pathKey,
} from './locate.ts';
import {
  renderEntryValue,
  renderString,
} from './render.ts';
import {
  leadingCommentStart,
  lineEndInclusive,
  type Splice,
} from './text-splice.ts';

//region Inserts

/**
 Offset just past the line holding `end - 1`.

 @param source - catalog text

 @param end - exclusive end of a node

 @returns start of the next line

 @example
 ```ts
 nextLine({ source, end: node.range[1], });
 ```
 */
function nextLine({ source, end, }: { readonly source: string; readonly end: number; },): number {
  return lineEndInclusive({ source, offset: Math.max(end - 1, 0,), },);
}

/**
 Splices that add new `[catalog]` entries, each before the first existing entry that sorts after it.

 @param source - catalog text

 @param located - indexed catalog

 @param additions - entries to add

 @returns splices; entries sharing an insertion point are merged into one splice in name order

 @example
 ```ts
 additionSplices({ source, located, additions, });
 ```
 */
function additionSplices(
  { source, located, additions, }: { readonly source: string; readonly located: LocatedDocument; readonly additions: ReadonlyMap<string, RustCatalogEntry>; },
): readonly Splice[] {
  if (additions.size === 0)
    return [];
  /**
   The default catalog table.
   */
  const table = located.tables.get(pathKey(['catalog',],),);
  /**
   Added lines in name order.
   */
  const lines = [...additions.entries(),]
    .toSorted(function byName([left,], [right,],): number {
      return left < right ? -1 : (left > right ? 1 : 0);
    },)
    .map(function toLine([name, entry,],): readonly [string, string] {
      return [name, `${name} = ${renderEntryValue(entry,)}\n`,];
    },);
  if (table === undefined) {
    /**
     Separator so the new table starts after one blank line.
     */
    const separator = source.length === 0 ? '' : (source.endsWith('\n',) ? '\n' : '\n\n');
    return [{ start: source.length, end: source.length, text: `${separator}[catalog]\n${lines.map(function toText([, line,],): string {
      return line;
    },).join('',)}`, },];
  }
  /**
   Existing single-key entries in source order.
   */
  const existing = childPairs({ located, prefix: ['catalog',], },);
  /**
   Lines grouped by insertion offset.
   */
  const byOffset = new Map<number, string[]>();
  for (const [name, line,] of lines) {
    /**
     First existing entry sorting after the new name.
     */
    const following = existing.find(function sortsAfter(pair,): boolean {
      return (pair.path[1] ?? '') > name;
    },);
    /**
     Last existing entry, after which unmatched names go.
     */
    const last = existing.at(-1,);
    /**
     Where this line goes.
     */
    const offset = following === undefined
      ? (last === undefined ? nextLine({ source, end: table.key.range[1], },) : nextLine({ source, end: last.node.range[1], },))
      : leadingCommentStart({ source, offset: following.node.range[0], },);
    byOffset.set(offset, [...(byOffset.get(offset,) ?? []), line,],);
  }
  return [...byOffset.entries(),].map(function toSplice([offset, group,],): Splice {
    return { start: offset, end: offset, text: group.join('',), };
  },);
}

//endregion Inserts

//region Changes

/**
 Splices that raise existing `[catalog]` floors.

 @param located - indexed catalog

 @param floorRaises - dependency name to new `>=` requirement

 @returns one splice per raised entry

 @example
 ```ts
 floorRaiseSplices({ located, floorRaises: new Map([['tokio', '>=1.50']]), });
 ```
 */
function floorRaiseSplices(
  { located, floorRaises, }: { readonly located: LocatedDocument; readonly floorRaises: ReadonlyMap<string, string>; },
): readonly Splice[] {
  return [...floorRaises.entries(),].flatMap(function toSplices([name, requirement,],): readonly Splice[] {
    /**
     The entry and, for table entries, its `version` member.
     */
    const candidates = located.pairs.filter(function isTarget(pair,): boolean {
      /**
       Joined path of this pair.
       */
      const key = pathKey(pair.path,);
      return (key === pathKey(['catalog', name,],)) || (key === pathKey(['catalog', name, 'version',],));
    },);
    /**
     The string-valued pair to rewrite.
     */
    const target = candidates.find(function isString(pair,): boolean {
      return pair.node.value.type === 'TOMLValue';
    },);
    if (target === undefined)
      return [];
    return [{ start: target.node.value.range[0], end: target.node.value.range[1], text: renderString(requirement,), },];
  },);
}

/**
 Splices that delete unused entries and the comment lines directly above them.

 @param source - catalog text

 @param located - indexed catalog

 @param prunes - entry paths such as `['catalog', 'anyhow']` or `['catalogs', 'pinned', 'salsa']`

 @returns one splice per pruned entry

 @example
 ```ts
 pruneSplices({ source, located, prunes: [['catalog', 'gone',],], });
 ```
 */
function pruneSplices(
  { source, located, prunes, }: { readonly source: string; readonly located: LocatedDocument; readonly prunes: readonly (readonly string[])[]; },
): readonly Splice[] {
  /**
   Joined paths to prune.
   */
  const targets = new Set(prunes.map(pathKey,),);
  return located.pairs
    .filter(function isPruned(pair,): boolean {
      return targets.has(pathKey(pair.path,),);
    },)
    .map(function toSplice(pair,): Splice {
      return {
        start: leadingCommentStart({ source, offset: pair.node.range[0], },),
        end: nextLine({ source, end: pair.node.range[1], },),
        text: '',
      };
    },);
}

/**
 Every splice the catalog file needs after a sync.

 @param source - catalog text

 @param located - indexed catalog

 @param additions - new default entries

 @param floorRaises - raised default floors

 @param prunes - unused entry paths to delete

 @returns splices to apply to `source`

 @example
 ```ts
 catalogSplices({ source, located, additions, floorRaises, prunes: [], });
 ```
 */
export function catalogSplices(
  {
    source,
    located,
    additions,
    floorRaises,
    prunes,
  }: {
    readonly source: string;
    readonly located: LocatedDocument;
    readonly additions: ReadonlyMap<string, RustCatalogEntry>;
    readonly floorRaises: ReadonlyMap<string, string>;
    readonly prunes: readonly (readonly string[])[];
  },
): readonly Splice[] {
  return [
    ...additionSplices({ source, located, additions, },),
    ...floorRaiseSplices({ located, floorRaises, },),
    ...pruneSplices({ source, located, prunes, },),
  ];
}

//endregion Changes

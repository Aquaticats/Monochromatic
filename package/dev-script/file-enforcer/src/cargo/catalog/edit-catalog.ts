/**
 Text splices for `rust-catalog.toml`: adopted entries, raised floors, and pruned entries.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/module-rust-catalog/ts';

import {
  childPairs,
  type LocatedDocument,
  type LocatedPair,
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

//region Shared

/**
 One added catalog line with the name it sorts by.
 */
type AddedLine = {
  readonly name: string;
  readonly line: string;
};

/**
 Offset just past the line holding `end - 1`.

 @param source - catalog text

 @param end - exclusive end of a node

 @returns start of the next line

 @example
 ```ts
 nextLine({ source, end: pair.range.end, });
 ```
 */
function nextLine({
  source,
  end,
}: {
  readonly source: string;
  readonly end: number
},): number {
  return lineEndInclusive({
    source,
    offset: Math.max(
      end - 1,
      0,
    ),
  },);
}

/**
 Code-point order of two names, for deterministic catalog ordering.

 @param left - name

 @param right - name

 @returns negative, zero, or positive

 @example
 ```ts
 compareNames({ left: 'a', right: 'b', }); // -1
 ```
 */
function compareNames({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string
},): number {
  return left < right ? -1 : (left > right ? 1 : 0);
}

//endregion Shared

//region Inserts

/**
 Offset where a new `[catalog]` entry named `name` goes: before the first entry sorting after it, else after the last.

 @param source - catalog text

 @param headerEnd - end of the `[catalog]` header

 @param existing - existing entries in source order

 @param name - new entry name

 @returns insertion offset

 @example
 ```ts
 insertionOffset({ source, headerEnd, existing, name: 'clap', });
 ```
 */
function insertionOffset(
  {
    source,
    headerEnd,
    existing,
    name,
  }: {
    readonly source: string;
    readonly headerEnd: number;
    readonly existing: readonly LocatedPair[];
    readonly name: string
  },
): number {
  /**
   First existing entry sorting after the new name.
   */
  const following = existing.find(function sortsAfter(pair: LocatedPair,): boolean {
    return compareNames({
      left: pair.path[1] ?? '',
      right: name,
    },) > 0;
  },);
  if (following !== undefined)
    return leadingCommentStart({
      source,
      offset: following.range
        .start,
    },);
  /**
   Last existing entry.
   */
  const last = existing.at(-1,);
  return last === undefined ? nextLine({
    source,
    end: headerEnd,
  },) : nextLine({
    source,
    end: last.range
      .end,
  },);
}

/**
 Splices that add new `[catalog]` entries in name order.

 @param source - catalog text

 @param located - projected catalog

 @param additions - entries to add

 @returns splices; entries sharing an insertion point are merged into one splice in name order

 @example
 ```ts
 additionSplices({ source, located, additions, });
 ```
 */
function additionSplices(
  {
    source,
    located,
    additions,
  }: {
    readonly source: string;
    readonly located: LocatedDocument;
    readonly additions: ReadonlyMap<string, RustCatalogEntry>
  },
): readonly Splice[] {
  if (additions.size === 0)
    return [];
  /**
   Added lines in name order.
   */
  const lines: AddedLine[] = [];
  for (const [name, entry,] of additions)
    lines.push({
      name,
      line: `${name} = ${renderEntryValue(entry,)}\n`,
    },);
  lines.sort(function byName(
    left: AddedLine,
    right: AddedLine,
  ): number {
    return compareNames({
      left: left.name,
      right: right.name,
    },);
  },);
  /**
   The default catalog table.
   */
  const table = located.tables
    .get(pathKey(['catalog',],),);
  if (table === undefined) {
    /**
     Separator so the new table starts after one blank line.
     */
    const separator = source.length === 0 ? '' : (source.endsWith('\n',) ? '\n' : '\n\n');
    return [{
      start: source.length,
      end: source.length,
      text: `${separator}[catalog]\n${lines.map(function toText(added: AddedLine,): string {
      return added.line;
    },)
        .join('',)}`,
    },];
  }
  /**
   Existing entries in source order.
   */
  const existing = childPairs({
    located,
    prefix: ['catalog',],
  },);
  /**
   Lines grouped by insertion offset, kept in name order.
   */
  const byOffset = new Map<number, string>();
  for (const added of lines) {
    /**
     Where this line goes.
     */
    const offset = insertionOffset({
      source,
      headerEnd: table.headerEnd,
      existing,
      name: added.name,
    },);
    byOffset.set(
      offset,
      `${byOffset.get(offset,) ?? ''}${added.line}`,
    );
  }
  /**
   One splice per insertion point.
   */
  const splices: Splice[] = [];
  for (const [offset, text,] of byOffset)
    splices.push({
      start: offset,
      end: offset,
      text,
    },);
  return splices;
}

//endregion Inserts

//region Changes

/**
 Splices that raise existing `[catalog]` floors.

 @param located - projected catalog

 @param floorRaises - dependency name to new `>=` requirement

 @returns one splice per raised entry

 @example
 ```ts
 floorRaiseSplices({ located, floorRaises: new Map([['tokio', '>=1.50']]), });
 ```
 */
function floorRaiseSplices(
  {
    located,
    floorRaises,
  }: {
    readonly located: LocatedDocument;
    readonly floorRaises: ReadonlyMap<string, string>
  },
): readonly Splice[] {
  /**
   Splices collected per raised entry.
   */
  const splices: Splice[] = [];
  for (const [name, requirement,] of floorRaises) {
    /**
     Paths that hold the version string: the entry itself or its `version` member.
     */
    const targets = new Set([
      pathKey([
        'catalog',
        name,
      ],),
      pathKey([
        'catalog',
        name,
        'version',
      ],),
    ],);
    /**
     The string-valued pair to rewrite.
     */
    const target = located.pairs
      .find(function isTarget(pair: LocatedPair,): boolean {
        return targets.has(pathKey(pair.path,),) && (pair.valueKind === 'string');
      },);
    if (target !== undefined)
      splices.push({
        start: target.valueRange
          .start,
        end: target.valueRange
          .end,
        text: renderString(requirement,),
      },);
  }
  return splices;
}

/**
 Splices that delete unused entries and the comment lines directly above them.

 @param source - catalog text

 @param located - projected catalog

 @param prunes - entry paths such as `['catalog', 'anyhow']` or `['catalogs', 'pinned', 'salsa']`

 @returns one splice per pruned entry

 @example
 ```ts
 pruneSplices({ source, located, prunes: [['catalog', 'gone',],], });
 ```
 */
function pruneSplices(
  {
    source,
    located,
    prunes,
  }: {
    readonly source: string;
    readonly located: LocatedDocument;
    readonly prunes: readonly (readonly string[])[]
  },
): readonly Splice[] {
  /**
   Joined paths to prune.
   */
  const targets = new Set<string>();
  for (const path of prunes)
    targets.add(pathKey(path,),);
  return located.pairs
    .filter(function isPruned(pair: LocatedPair,): boolean {
      return targets.has(pathKey(pair.path,),);
    },)
    .map(function toSplice(pair: LocatedPair,): Splice {
      return {
        start: leadingCommentStart({
          source,
          offset: pair.range
            .start,
        },),
        end: nextLine({
          source,
          end: pair.range
            .end,
        },),
        text: '',
      };
    },);
}

/**
 Every splice the catalog file needs after a sync.

 @param source - catalog text

 @param located - projected catalog

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
    ...additionSplices({
      source,
      located,
      additions,
    },),
    ...floorRaiseSplices({
      located,
      floorRaises,
    },),
    ...pruneSplices({
      source,
      located,
      prunes,
    },),
  ];
}

//endregion Changes

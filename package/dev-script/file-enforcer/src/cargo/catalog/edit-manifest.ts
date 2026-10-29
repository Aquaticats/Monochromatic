/**
 Text splices that bring one crate manifest in line with the catalog.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/module-rust-catalog/ts';

import {
  childPairs,
  type LocatedDocument,
  type LocatedPair,
  pairAt,
  pathKey,
} from './locate.ts';
import {
  GENERATED_TABLE_COMMENT,
  renderGeneratedTable,
  renderString,
} from './render.ts';
import {
  leadingCommentStart,
  lineEndInclusive,
  lineStart,
  type Splice,
} from './text-splice.ts';
import type { UseSite, } from './types.ts';

//region Shared

/**
 Keys a literal carries that move into the catalog when the site becomes `workspace = true`.
 */
const MOVED_KEYS: ReadonlySet<string> = new Set([
  'git',
  'package',
  'rev',
  'version',
],);

/**
 Offset just past the line holding `end - 1`.

 @param source - full text

 @param end - exclusive end offset of a node

 @returns offset after that line's newline

 @example
 ```ts
 afterLine({ source, end: pair.range.end, });
 ```
 */
function afterLine({
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
 Last segment of a pair's path.

 @param pair - projected pair

 @returns key name, e.g. `version`

 @example
 ```ts
 lastKey(pair); // 'version'
 ```
 */
function lastKey(pair: LocatedPair,): string {
  return pair.path
    .at(-1,)
    ?? '';
}

/**
 Members of an inline-table entry.

 @param located - projected manifest

 @param site - use site

 @returns pairs inside `name = { ... }`, in source order

 @example
 ```ts
 inlineMembers({ located, site, });
 ```
 */
function inlineMembers({
  located,
  site,
}: {
  readonly located: LocatedDocument;
  readonly site: UseSite
},): readonly LocatedPair[] {
  return childPairs({
    located,
    prefix: [
      ...site.table,
      site.name,
    ],
  },)
    .filter(function isInline(pair: LocatedPair,): boolean {
      return pair.inInlineTable;
    },);
}

/**
 Sub-key pairs that spell out one dependency: a `[table.name]` section or dotted `name.key` pairs.

 @param located - projected manifest

 @param site - use site

 @returns sub-key pairs in source order

 @example
 ```ts
 subKeyPairs({ located, site, });
 ```
 */
function subKeyPairs({
  located,
  site,
}: {
  readonly located: LocatedDocument;
  readonly site: UseSite
},): readonly LocatedPair[] {
  return childPairs({
    located,
    prefix: [
      ...site.table,
      site.name,
    ],
  },)
    .filter(function isSubKey(pair: LocatedPair,): boolean {
      return !pair.inInlineTable;
    },);
}

/**
 Text a sub-key pair needs before its last segment: `name.` for dotted pairs, empty inside a section.

 @param pair - sub-key pair

 @param site - use site

 @returns key prefix to keep when replacing the pair

 @example
 ```ts
 subKeyPrefix({ pair, site, }); // 'smithay.' or ''
 ```
 */
function subKeyPrefix({
  pair,
  site,
}: {
  readonly pair: LocatedPair;
  readonly site: UseSite
},): string {
  return pair.keySegments > 1 ? `${site.name}.` : '';
}

//endregion Shared

//region Use sites

/**
 Splices that turn a literal site into `workspace = true`, keeping its features, optional, and default-features text.

 @param source - manifest text

 @param located - projected manifest

 @param site - literal use site

 @returns splices for this site

 @example
 ```ts
 inheritSplices({ source, located, site, });
 ```
 */
export function inheritSplices(
  {
    source,
    located,
    site,
  }: {
    readonly source: string;
    readonly located: LocatedDocument;
    readonly site: UseSite
  },
): readonly Splice[] {
  /**
   Inline form: `name = "1"` or `name = { ... }`.
   */
  const [inline,] = pairAt({
    located,
    path: [
      ...site.table,
      site.name,
    ],
  },);
  if (inline !== undefined) {
    /**
     Kept member texts, verbatim.
     */
    const kept = inline.valueKind === 'inline-table'
      ? inlineMembers({
        located,
        site,
      },)
        .filter(function isKept(pair: LocatedPair,): boolean {
          return !MOVED_KEYS.has(lastKey(pair,),);
        },)
        .map(function toText(pair: LocatedPair,): string {
          return source.slice(
            pair.range
              .start,
            pair.range
              .end,
          );
        },)
      : [];
    return [{
      start: inline.valueRange
        .start,
      end: inline.valueRange
        .end,
      text: `{ ${[
        'workspace = true',
        ...kept,
      ].join(', ',)} }`,
    },];
  }

  /**
   Section or dotted sub-keys to move out.
   */
  const moved = subKeyPairs({
    located,
    site,
  },)
    .filter(function isMoved(pair: LocatedPair,): boolean {
      return MOVED_KEYS.has(lastKey(pair,),);
    },);
  return moved.map(function toSplice(
    pair: LocatedPair,
    index: number,
  ): Splice {
    if (index === 0)
      return {
        start: pair.range
          .start,
        end: pair.range
          .end,
        text: `${subKeyPrefix({
          pair,
          site,
        },)}workspace = true`,
      };
    return {
      start: lineStart({
        source,
        offset: pair.range
          .start,
      },),
      end: afterLine({
        source,
        end: pair.range
          .end,
      },),
      text: '',
    };
  },);
}

/**
 Splices that set an internal path dependency's `version` to the exact target version.

 @param source - manifest text

 @param located - projected manifest

 @param site - path use site

 @param version - target crate's `package.version`

 @returns splices, empty when the version already matches

 @example
 ```ts
 pathVersionSplices({ source, located, site, version: '0.1.1', });
 ```
 */
export function pathVersionSplices(
  {
    source,
    located,
    site,
    version,
  }: {
    readonly source: string;
    readonly located: LocatedDocument;
    readonly site: UseSite;
    readonly version: string
  },
): readonly Splice[] {
  /**
   Requirement the path entry should carry (the `workspace:*` counterpart).
   */
  const wanted = renderString(`=${version}`,);
  /**
   Inline-table form, when the entry is written that way.
   */
  const [inline,] = pairAt({
    located,
    path: [
      ...site.table,
      site.name,
    ],
  },);
  /**
   Pairs to search for `version`: inline members or sub-keys.
   */
  const members = inline === undefined ? subKeyPairs({
    located,
    site,
  },) : inlineMembers({
    located,
    site,
  },);
  /**
   Existing `version` member.
   */
  const current = members.find(function isVersion(pair: LocatedPair,): boolean {
    return lastKey(pair,) === 'version';
  },);
  if (current !== undefined) {
    if (source.slice(
      current.valueRange
        .start,
      current.valueRange
        .end,
    ) === wanted)
      return [];
    return [{
      start: current.valueRange
        .start,
      end: current.valueRange
        .end,
      text: wanted,
    },];
  }
  /**
   Last member, after which `version` goes.
   */
  const last = members.at(-1,);
  if (last === undefined)
    return [];
  if (inline !== undefined)
    return [{
      start: last.range
        .end,
      end: last.range
        .end,
      text: `, version = ${wanted}`,
    },];
  /**
   Line after the last sub-key.
   */
  const insertAt = afterLine({
    source,
    end: last.range
      .end,
  },);
  return [{
    start: insertAt,
    end: insertAt,
    text: `${subKeyPrefix({
      pair: last,
      site,
    },)}version = ${wanted}\n`,
  },];
}

//endregion Use sites

//region Generated table

/**
 Splice that makes the generated `[workspace.dependencies]` table hold exactly `entries`.

 @param source - manifest text

 @param located - projected manifest

 @param entries - dependency name to resolved catalog entry

 @returns zero or one splice

 @example
 ```ts
 generatedTableSplices({ source, located, entries, });
 ```
 */
export function generatedTableSplices(
  {
    source,
    located,
    entries,
  }: {
    readonly source: string;
    readonly located: LocatedDocument;
    readonly entries: ReadonlyMap<string, RustCatalogEntry>
  },
): readonly Splice[] {
  /**
   Existing generated table.
   */
  const table = located.tables
    .get(pathKey([
      'workspace',
      'dependencies',
    ],),);
  /**
   Wanted table text.
   */
  const body = renderGeneratedTable(entries,);
  if (table === undefined) {
    if (entries.size === 0)
      return [];
    /**
     Newlines so the block starts after one blank line.
     */
    const separator = source.endsWith('\n',) ? '\n' : '\n\n';
    return [{
      start: source.length,
      end: source.length,
      text: `${separator}${GENERATED_TABLE_COMMENT}\n${body}`,
    },];
  }
  /**
   End of the table's last line.
   */
  const end = afterLine({
    source,
    end: table.range
      .end,
  },);
  if (entries.size === 0)
    return [{
      start: leadingCommentStart({
        source,
        offset: table.range
          .start,
      },),
      end,
      text: '',
    },];
  if (source.slice(
    table.range
      .start,
    end,
  ) === body)
    return [];
  return [{
    start: table.range
      .start,
    end,
    text: body,
  },];
}

//endregion Generated table

/**
 Text splices that bring one crate manifest in line with the catalog.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/module-rust-catalog/ts';
import type { AST, } from 'toml-eslint-parser';

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
const MOVED_KEYS: ReadonlySet<string> = new Set(['git', 'package', 'rev', 'version',],);

/**
 Name of a pair's last key segment.

 @param kv - key-value node

 @returns last segment, unquoted

 @example
 ```ts
 lastKeyName(kv); // 'version' for `smithay.version = "1"`
 ```
 */
function lastKeyName(kv: AST.TOMLKeyValue,): string {
  /**
   Last key segment.
   */
  const segment = kv.key.keys.at(-1,);
  if (segment === undefined)
    return '';
  return segment.type === 'TOMLBare' ? segment.name : segment.value;
}

/**
 Offset just past the line ending at `end`.

 @param source - full text

 @param end - exclusive end offset of a node

 @returns offset after that line's newline

 @example
 ```ts
 afterLine({ source, end: node.range[1], });
 ```
 */
function afterLine({ source, end, }: { readonly source: string; readonly end: number; },): number {
  return lineEndInclusive({ source, offset: Math.max(end - 1, 0,), },);
}

/**
 Key-value pairs that spell out one dependency as sub-keys: a `[table.name]` section or dotted `name.key` pairs.

 @param located - indexed manifest

 @param site - use site

 @returns sub-key pairs in source order, empty when the entry is a single inline value

 @example
 ```ts
 subKeyPairs({ located, site, });
 ```
 */
function subKeyPairs({ located, site, }: { readonly located: LocatedDocument; readonly site: UseSite; },): readonly LocatedPair[] {
  return childPairs({ located, prefix: [...site.table, site.name,], },).filter(function isOutsideInline(pair,): boolean {
    return pair.node.parent.type !== 'TOMLInlineTable';
  },);
}

/**
 Text a sub-key pair needs before its last key segment (`name.` for dotted pairs, empty inside a section).

 @param pair - sub-key pair

 @param site - use site

 @returns prefix to keep when replacing the pair

 @example
 ```ts
 subKeyPrefix({ pair, site, }); // 'smithay.' or ''
 ```
 */
function subKeyPrefix({ pair, site, }: { readonly pair: LocatedPair; readonly site: UseSite; },): string {
  return pair.node.key.keys.length > 1 ? `${site.name}.` : '';
}

//endregion Shared

//region Use sites

/**
 Splices that turn a literal site into `workspace = true`, keeping its features, optional, and default-features text.

 @param source - manifest text

 @param located - indexed manifest

 @param site - literal use site

 @returns splices for this site

 @example
 ```ts
 inheritSplices({ source, located, site, });
 ```
 */
export function inheritSplices(
  { source, located, site, }: { readonly source: string; readonly located: LocatedDocument; readonly site: UseSite; },
): readonly Splice[] {
  /**
   Inline form: `name = "1"` or `name = { ... }`.
   */
  const [inline,] = pairAt({ located, path: [...site.table, site.name,], },);
  if (inline !== undefined) {
    /**
     Entry value node.
     */
    const value = inline.node.value;
    /**
     Kept key-value texts, verbatim.
     */
    const kept = value.type === 'TOMLInlineTable'
      ? value.body.filter(function isKept(kv,): boolean {
        return !MOVED_KEYS.has(lastKeyName(kv,),);
      },).map(function toText(kv,): string {
        return source.slice(kv.range[0], kv.range[1],);
      },)
      : [];
    return [{
      start: value.range[0],
      end: value.range[1],
      text: `{ ${['workspace = true', ...kept,].join(', ',)} }`,
    },];
  }

  /**
   Section or dotted sub-keys to move out.
   */
  const moved = subKeyPairs({ located, site, },).filter(function isMoved(pair,): boolean {
    return MOVED_KEYS.has(pair.path.at(-1,) ?? '',);
  },);
  return moved.map(function toSplice(pair, index,): Splice {
    if (index === 0)
      return { start: pair.node.range[0], end: pair.node.range[1], text: `${subKeyPrefix({ pair, site, },)}workspace = true`, };
    return { start: lineStart({ source, offset: pair.node.range[0], },), end: afterLine({ source, end: pair.node.range[1], },), text: '', };
  },);
}

/**
 Splices that set an internal path dependency's `version` to the exact target version.

 @param source - manifest text

 @param located - indexed manifest

 @param site - path use site

 @param version - target crate's `package.version`

 @returns splices, empty when the version already matches

 @example
 ```ts
 pathVersionSplices({ source, located, site, version: '0.1.1', });
 ```
 */
export function pathVersionSplices(
  { source, located, site, version, }: { readonly source: string; readonly located: LocatedDocument; readonly site: UseSite; readonly version: string; },
): readonly Splice[] {
  /**
   Requirement the path entry should carry (the `workspace:*` counterpart).
   */
  const wanted = renderString(`=${version}`,);
  /**
   Inline-table form.
   */
  const [inline,] = pairAt({ located, path: [...site.table, site.name,], },);
  /**
   Pairs to search for `version`: inline-table members or sub-keys.
   */
  const members: readonly AST.TOMLKeyValue[] = inline !== undefined && inline.node.value.type === 'TOMLInlineTable'
    ? inline.node.value.body
    : subKeyPairs({ located, site, },).map(function toNode(pair,): AST.TOMLKeyValue {
      return pair.node;
    },);
  /**
   Existing `version` member.
   */
  const current = members.find(function isVersion(kv,): boolean {
    return lastKeyName(kv,) === 'version';
  },);
  if (current !== undefined) {
    if (source.slice(current.value.range[0], current.value.range[1],) === wanted)
      return [];
    return [{ start: current.value.range[0], end: current.value.range[1], text: wanted, },];
  }
  /**
   Last member, after which `version` is inserted.
   */
  const last = members.at(-1,);
  if (last === undefined)
    return [];
  if (inline !== undefined)
    return [{ start: last.range[1], end: last.range[1], text: `, version = ${wanted}`, },];
  /**
   Dotted prefix when the sub-keys are dotted pairs.
   */
  const prefix = last.key.keys.length > 1 ? `${site.name}.` : '';
  /**
   Line after the last sub-key.
   */
  const insertAt = afterLine({ source, end: last.range[1], },);
  return [{ start: insertAt, end: insertAt, text: `${prefix}version = ${wanted}\n`, },];
}

//endregion Use sites

//region Generated table

/**
 Splice that makes the generated `[workspace.dependencies]` table hold exactly `entries`.

 @param source - manifest text

 @param located - indexed manifest

 @param entries - dependency name to resolved catalog entry

 @returns zero or one splice

 @example
 ```ts
 generatedTableSplices({ source, located, entries, });
 ```
 */
export function generatedTableSplices(
  { source, located, entries, }: { readonly source: string; readonly located: LocatedDocument; readonly entries: ReadonlyMap<string, RustCatalogEntry>; },
): readonly Splice[] {
  /**
   Existing generated table.
   */
  const table = located.tables.get(pathKey(['workspace', 'dependencies',],),);
  /**
   Wanted table text.
   */
  const body = renderGeneratedTable(entries,);
  if (table === undefined) {
    if (entries.size === 0)
      return [];
    /**
     Newlines needed so the block starts after one blank line.
     */
    const separator = source.endsWith('\n',) ? '\n' : '\n\n';
    return [{ start: source.length, end: source.length, text: `${separator}${GENERATED_TABLE_COMMENT}\n${body}`, },];
  }
  /**
   End of the table's last line.
   */
  const end = afterLine({ source, end: table.range[1], },);
  if (entries.size === 0) {
    /**
     Start of the table, including the generated comment above it.
     */
    const start = leadingCommentStart({ source, offset: table.range[0], },);
    return [{ start, end, text: '', },];
  }
  if (source.slice(table.range[0], end,) === body)
    return [];
  return [{ start: table.range[0], end, text: body, },];
}

//endregion Generated table

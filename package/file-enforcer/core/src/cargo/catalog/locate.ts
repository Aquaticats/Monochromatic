/**
 A project-owned, deeply readonly projection of a TOML document's tables and key-value pairs.

 This module is the only place that touches the mutable `toml-eslint-parser`
 AST: it parses once and keeps just the source ranges, paths, and kinds the
 catalog editors need, so every edit can splice exact text into the source.

 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  type AST,
  getStaticTOMLValue,
  parseTOML,
} from 'toml-eslint-parser';

//region Types

/**
 Half-open source range `[start, end)`.

 @example
 ```ts
 const range: SourceRange = { start: 0, end: 7, };
 ```
 */
export type SourceRange = {
  readonly start: number;
  readonly end: number;
};

/**
 What a pair's value is, as far as the editors care.

 @example
 ```ts
 const kind: ValueKind = 'inline-table';
 ```
 */
export type ValueKind = 'inline-table' | 'other' | 'string';

/**
 One key-value pair with its full dotted path from the document root.

 @example
 ```ts
 const pair: LocatedPair = {
   path: ['dependencies', 'anyhow',],
   range: { start: 20, end: 32, },
   valueRange: { start: 29, end: 32, },
   valueKind: 'string',
   keySegments: 1,
   inInlineTable: false,
 };
 ```
 */
export type LocatedPair = {
  readonly path: readonly string[];
  readonly range: SourceRange;
  readonly valueRange: SourceRange;
  readonly valueKind: ValueKind;
  /**
   Segments in the pair's own key: 1 for `version = ...`, 2 for `smithay.version = ...`.
   */
  readonly keySegments: number;
  readonly inInlineTable: boolean;
};

/**
 One standard `[table]` with its header and full range.

 @example
 ```ts
 const table: LocatedTable = { range: { start: 0, end: 40, }, headerEnd: 12, };
 ```
 */
export type LocatedTable = {
  readonly range: SourceRange;
  readonly headerEnd: number;
};

/**
 Every table and key-value pair of a document, plus its plain value.

 @example
 ```ts
 const located = locateSource('[catalog]\nanyhow = ">=1"\n');
 located.pairs.length; // 1
 ```
 */
export type LocatedDocument = {
  readonly tables: ReadonlyMap<string, LocatedTable>;
  readonly pairs: readonly LocatedPair[];
  readonly value: unknown;
};

//endregion Types

//region Paths

/**
 Separator for joined path keys; NUL cannot appear in a key Cargo or the catalog accepts.
 */
const PATH_SEPARATOR = '\u0000';

/**
 Joins a key path into a map key.

 @param path - key segments

 @returns joined key

 @example
 ```ts
 pathKey(['workspace', 'dependencies',]);
 ```
 */
export function pathKey(path: readonly string[],): string {
  return path.join(PATH_SEPARATOR,);
}

//endregion Paths

//region Projection

/**
 Unquoted names of a parsed key's segments.

 @param key - parsed key from the parser boundary

 @returns segment names

 @example
 ```ts
 keyNames(kv.key); // ['smithay', 'version']
 ```
 */
function keyNames(key: ForeignBorrowed<AST.TOMLKey>,): readonly string[] {
  /**
   Names collected in order.
   */
  const names: string[] = [];
  for (const segment of key.keys)
    names.push(segment.type === 'TOMLBare' ? segment.name : segment.value,);
  return names;
}

/**
 Projects one parsed pair (and the members of an inline-table value) into readonly records.

 @param prefix - path of the enclosing table or inline table

 @param kv - parsed pair from the parser boundary

 @param inInlineTable - whether `kv` sits inside an inline table

 @param into - collector the projected pairs are appended to, in source order

 @example
 ```ts
 projectPair({ prefix: ['dependencies',], kv, inInlineTable: false, into: pairs, });
 ```
 */
function projectPair(
  {
    prefix,
    kv,
    inInlineTable,
    into,
  }: {
    readonly prefix: readonly string[];
    readonly kv: ForeignBorrowed<AST.TOMLKeyValue>;
    readonly inInlineTable: boolean;
    readonly into: LocatedPair[];
  },
): void {
  /**
   Segments of the pair's own key.
   */
  const names = keyNames(kv.key,);
  /**
   Full path of this pair.
   */
  const path = [
    ...prefix,
    ...names,
  ];
  /**
   Value kind the editors branch on.
   */
  const valueKind: ValueKind = kv.value
    .type
    === 'TOMLInlineTable'
    ? 'inline-table'
    : ((kv.value
      .type
      === 'TOMLValue') && (kv.value
        .kind
        === 'string') ? 'string' : 'other');
  into.push({
    path,
    range: {
      start: kv.range[0],
      end: kv.range[1],
    },
    valueRange: {
      start: kv.value
        .range[0],
      end: kv.value
        .range[1],
    },
    valueKind,
    keySegments: names.length,
    inInlineTable,
  },);
  if (kv.value
    .type
    === 'TOMLInlineTable') {
    for (const member of kv.value
      .body)
      projectPair({
        prefix: path,
        kv: member,
        inInlineTable: true,
        into,
      },);
  }
}

/**
 Parses TOML text and projects its tables and pairs.

 @param source - TOML text

 @returns readonly projection plus the document's plain value

 @throws Error from `toml-eslint-parser` when `source` is not valid TOML

 @example
 ```ts
 const located = locateSource(source);
 ```
 */
export function locateSource(source: string,): LocatedDocument {
  /**
   Parsed document; only this function holds the parser's mutable AST.
   */
  const program: ForeignBorrowed<AST.TOMLProgram> = parseTOML(source,);
  /**
   Standard tables by joined path.
   */
  const tables = new Map<string, LocatedTable>();
  /**
   Pairs in source order.
   */
  const pairs: LocatedPair[] = [];
  for (const item of program.body[0]
    .body) {
    if (item.type === 'TOMLKeyValue') {
      projectPair({
        prefix: [],
        kv: item,
        inInlineTable: false,
        into: pairs,
      },);
      continue;
    }
    /**
     Table path as Cargo sees it.
     */
    const tablePath: string[] = [];
    for (const segment of item.resolvedKey)
      tablePath.push(String(segment,),);
    if (item.kind === 'standard')
      tables.set(
        pathKey(tablePath,),
        {
          range: {
            start: item.range[0],
            end: item.range[1],
          },
          headerEnd: item.key
            .range[1],
        },
      );
    for (const kv of item.body)
      projectPair({
        prefix: tablePath,
        kv,
        inInlineTable: false,
        into: pairs,
      },);
  }
  return {
    tables,
    pairs,
    value: getStaticTOMLValue(program,),
  };
}

//endregion Projection

//region Queries

/**
 Whether `path` starts with every segment of `prefix` and has exactly one more.

 @param path - candidate path

 @param prefix - parent path

 @returns whether `path` is a direct child of `prefix`

 @example
 ```ts
 isDirectChild({ path: ['a', 'b',], prefix: ['a',], }); // true
 ```
 */
function isDirectChild({
  path,
  prefix,
}: {
  readonly path: readonly string[];
  readonly prefix: readonly string[]
},): boolean {
  if (path.length !== (prefix.length
    + 1))
    return false;
  for (let index = 0; index < prefix.length; index += 1) {
    if (path[index] !== prefix[index])
      return false;
  }
  return true;
}

/**
 Pairs one level below `prefix`, in source order.

 @param located - projected document

 @param prefix - parent path

 @returns direct children of `prefix`

 @example
 ```ts
 childPairs({ located, prefix: ['dependencies', 'smithay',], });
 ```
 */
export function childPairs({
  located,
  prefix,
}: {
  readonly located: LocatedDocument;
  readonly prefix: readonly string[]
},): readonly LocatedPair[] {
  return located.pairs
    .filter(function isChild(pair: LocatedPair,): boolean {
      return isDirectChild({
        path: pair.path,
        prefix,
      },);
    },);
}

/**
 Pairs at exactly `path`.

 @param located - projected document

 @param path - full key path

 @returns zero or one pair

 @example
 ```ts
 const [pair] = pairAt({ located, path: ['dependencies', 'anyhow',], });
 ```
 */
export function pairAt({
  located,
  path,
}: {
  readonly located: LocatedDocument;
  readonly path: readonly string[]
},): readonly LocatedPair[] {
  /**
   Joined target path.
   */
  const target = pathKey(path,);
  return located.pairs
    .filter(function isTarget(pair: LocatedPair,): boolean {
      return pathKey(pair.path,) === target;
    },);
}

//endregion Queries

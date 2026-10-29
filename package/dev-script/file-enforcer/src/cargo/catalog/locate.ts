/**
 Source locations of tables and key-value pairs in a parsed TOML document.

 Every node keeps its parse-time `range`, so catalog edits can splice exact
 text into the original source.

 @module
 */

import type { AST, } from 'toml-eslint-parser';

//region Types

/**
 One key-value pair with its full dotted path from the document root.

 @example
 ```ts
 const pair: LocatedPair = { path: ['dependencies', 'anyhow',], node, };
 ```
 */
export type LocatedPair = {
  readonly path: readonly string[];
  readonly node: AST.TOMLKeyValue;
};

/**
 Every table and key-value pair of a document, by full dotted path.

 @example
 ```ts
 const located = locateDocument(parseTOML(source));
 located.tables.get('workspace\u0000dependencies');
 ```
 */
export type LocatedDocument = {
  readonly tables: ReadonlyMap<string, AST.TOMLTable>;
  readonly pairs: readonly LocatedPair[];
};

//endregion Types

//region Keys

/**
 Separator for joined path keys; NUL cannot appear in a TOML key that Cargo or the catalog accepts.
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

/**
 Names of a TOML key's segments.

 @param key - parsed key

 @returns segment names, unquoted

 @example
 ```ts
 keyNames(pair.key); // ['anyhow', 'workspace']
 ```
 */
function keyNames(key: AST.TOMLKey,): readonly string[] {
  return key.keys.map(function toName(segment,): string {
    return segment.type === 'TOMLBare' ? segment.name : segment.value;
  },);
}

//endregion Keys

//region Locate

/**
 Indexes every standard table and key-value pair (including those inside inline tables) by full path.

 @param program - parsed document

 @returns tables keyed by {@link pathKey} and pairs in source order

 @example
 ```ts
 const located = locateDocument(parseTOML(source));
 ```
 */
export function locateDocument(program: AST.TOMLProgram,): LocatedDocument {
  /**
   Standard tables by joined path.
   */
  const tables = new Map<string, AST.TOMLTable>();
  /**
   Pairs in source order.
   */
  const pairs: LocatedPair[] = [];

  /**
   Records `kv` under `prefix` and descends into inline tables.

   @param prefix - path of the enclosing table

   @param kv - pair to record
   */
  function visit({ prefix, kv, }: { readonly prefix: readonly string[]; readonly kv: AST.TOMLKeyValue; },): void {
    /**
     Full path of this pair.
     */
    const path = [...prefix, ...keyNames(kv.key,),];
    pairs.push({ path, node: kv, },);
    if (kv.value.type === 'TOMLInlineTable') {
      for (const inner of kv.value.body)
        visit({ prefix: path, kv: inner, },);
    }
  }

  /**
   Root table holding top-level pairs and every table header.
   */
  const root = program.body[0];
  for (const item of root.body) {
    if (item.type === 'TOMLKeyValue') {
      visit({ prefix: [], kv: item, },);
      continue;
    }
    /**
     Table path as Cargo sees it.
     */
    const tablePath = item.resolvedKey.map(String,);
    if (item.kind === 'standard')
      tables.set(pathKey(tablePath,), item,);
    for (const kv of item.body)
      visit({ prefix: tablePath, kv, },);
  }
  return { tables, pairs, };
}

/**
 Pairs whose path starts with `prefix` and has exactly one more segment.

 @param located - indexed document

 @param prefix - parent path

 @returns direct children of `prefix`, in source order

 @example
 ```ts
 childPairs({ located, prefix: ['dependencies', 'smithay',], }); // version, features, ...
 ```
 */
export function childPairs({ located, prefix, }: { readonly located: LocatedDocument; readonly prefix: readonly string[]; },): readonly LocatedPair[] {
  return located.pairs.filter(function isChild(pair,): boolean {
    return (pair.path.length === prefix.length + 1) && prefix.every(function matches(segment, index,): boolean {
      return pair.path[index] === segment;
    },);
  },);
}

/**
 Pair at exactly `path`, or none.

 @param located - indexed document

 @param path - full key path

 @returns the pair, or an empty list when the document has none at that path

 @example
 ```ts
 const [pair] = pairAt({ located, path: ['dependencies', 'anyhow',], });
 ```
 */
export function pairAt({ located, path, }: { readonly located: LocatedDocument; readonly path: readonly string[]; },): readonly LocatedPair[] {
  /**
   Joined target path.
   */
  const target = pathKey(path,);
  return located.pairs.filter(function isTarget(pair,): boolean {
    return pathKey(pair.path,) === target;
  },);
}

//endregion Locate

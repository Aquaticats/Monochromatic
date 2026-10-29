/**
 Locate the entry block (key-value or table) a path names, for comment ops.
 
 Unlike {@link locateValueNode}, this returns the enclosing block so callers
 can read its line span and attached comments.
 
 @module
 */

import type {
  Block,
  KeyValueNode,
} from './document.ts';
import { segmentsEqual, } from './path-prefix.ts';
import {
  matchTableSection,
  NOT_LOCATED,
  type TableSectionHit,
} from './resolve-document.ts';
import type { TomlPath, } from './types.ts';

/**
 A located entry block.
 */
export type LocatedBlock =
  | {
    readonly kind: 'kv';
    readonly kv: KeyValueNode
  }
    | TableSectionHit;

/**
 Locate the entry block named by `path`.
 
 @returns A {@link LocatedBlock}, or {@link NOT_LOCATED}.
 
 @example
 ```ts
 locateBlock({ blocks: edit.blocks, path: ['tools'], },);
 ```
 */
export function locateBlock(
  {
    blocks,
    path,
  }: {
    readonly blocks: readonly Block[];
    readonly path: TomlPath;
  },
): LocatedBlock | typeof NOT_LOCATED {
  /**
   Exact key-value hit takes priority over any table scan.
   */
  const kv = blocks.find(function isKv(b,): b is KeyValueNode {
    return (b.kind
      === 'keyvalue')
      && segmentsEqual({
        left: b.keySegments,
        right: path,
      },);
  },);
  if (kv !== undefined)
    return {
      kind: 'kv',
      kv,
    };
  /**
   Section scan shared with {@link locateValueNode}; descent stays block-shaped.
   */
  const section = matchTableSection({
    blocks,
    path,
  },);
  if (section === NOT_LOCATED)
    return NOT_LOCATED;
  if (section.kind
    === 'descend')
    return locateBlock(section,);
  return section;
}

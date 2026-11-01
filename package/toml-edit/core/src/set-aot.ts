/**
 Array-of-tables wholesale replace for {@link tomlSet}.
 
 Given the existing `[[foo]]` instances at a path, replace them with one
 synthetic `[[foo]]` section per element of the array value. Passing `[]`
 clears every instance.
 
 @module
 */

import type {
  Block,
  TableNode,
} from './document.ts';
import { TomlTypeError, } from './errors.ts';
import { formatPath, } from './path.ts';
import { makeKeyValue, } from './set-create.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';
import { isPlainObject, } from './value-encoders.ts';

/**
 Replace the array-of-tables instances at `path` with `value`'s elements.
 
 @returns Fresh {@link TomlEditState}.
 
 @throws {@link TomlTypeError} when `value` is not an array, or an element is not
         a plain object.
 
 @example
 ```ts
 doAotReplace({ edit, path: ['fruits'], value: [{ name: 'apple' }], },);
 ```
 */
export function doAotReplace(
  {
    edit,
    path,
    value,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
    readonly value: unknown;
  },
): TomlEditState {
  if (!Array.isArray(value,)) {
    throw new TomlTypeError(
      `tomlSet on an array-of-tables at ${
        formatPath({ path, },)
      } requires an array value; pass [] to clear all instances`,
    );
  }
  /**
   Header segments as strings so each new instance shares one header spelling.
   */
  const header = path.map(function segmentText(segment,): string {
    return `${segment}`;
  },);
  /**
   One synthetic `[[header]]` block per array element.
   */
  const newTables: readonly TableNode[] = value.map(function each(
    el,
    i,
  ) {
    if (!isPlainObject(el,)) {
      throw new TomlTypeError(
        `tomlSet on an array-of-tables at ${
          formatPath({ path, },)
        } requires every element to be a plain object; index ${i} is not`,
      );
    }
    return {
      kind: 'table',
      tableKind: 'array',
      headerSegments: header,
      aotIndex: i,
      headerOrigin: { kind: 'synthetic', },
      body: Object.entries(el,)
        .map(function eachEntry([key, v,],) {
          return makeKeyValue({
            segments: [key,],
            value: v,
            options: edit.canonical,
          },);
        },),
      commentsBefore: [],
    };
  },);
  return {
    ...edit,
    blocks: spliceInstances({
      blocks: edit.blocks,
      header,
      newTables,
    },),
  };
}

/**
 Remove existing `[[header]]` instances and splice the new ones at the first
 removed position (or at end when none existed).
 
 @returns Fresh block list.
 */
function spliceInstances(
  {
    blocks,
    header,
    newTables,
  }: {
    readonly blocks: readonly Block[];
    readonly header: readonly string[];
    readonly newTables: readonly TableNode[];
  },
): readonly Block[] {
  /**
   True when a block is an existing array instance at the header path.
   
   @param block - Block tested for being an `[[header]]` instance.
   
   @returns Whether `block` is an array table whose header matches.
   
   @example
   ```ts
   isInstance(blocks[0],);
   ```
   */
  function isInstance(block: Block,): boolean {
    return (block.kind
      === 'table')
      && (block.tableKind
        === 'array')
      && (block.headerSegments
        .length
        === header.length)
      && block.headerSegments
        .every(function eq(
          seg,
          i,
        ) {
          return seg === header[i];
        },);
  }
  /**
   First existing-instance index so the replacements land in the same place.
   */
  const firstIdx = blocks.findIndex(function atInstance(b,) {
    return isInstance(b,);
  },);
  /**
   Blocks with the old instances removed.
   */
  const kept = blocks.filter(function keep(b,) {
    return !isInstance(b,);
  },);
  /**
   Insertion index within `kept`: everything before the first old instance is kept.
   */
  const insertAt = firstIdx === (-1) ? kept.length : firstIdx;
  return [
    ...kept.slice(
      0,
      insertAt,
    ),
    ...newTables,
    ...kept.slice(insertAt,),
  ];
}

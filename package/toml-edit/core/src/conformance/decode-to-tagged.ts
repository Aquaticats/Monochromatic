/**
 Walk a parsed TOML program into a toml-test tagged tree.
 
 Reconstructs the nested table / array structure from the parser's flat
 document body, using each table's absolute `resolvedKey` (which carries
 numeric array-of-tables indices) so the walk is a flat path-insert rather
 than a stateful re-implementation of table resolution. The mutable tree is
 captured in a builder closure, so no helper takes a mutable container as a
 parameter.
 
 @module
 */

import type { AST, } from 'toml-eslint-parser';

import { keysOf as keyPath, } from '../path.ts';

import { leafToTagged, } from './decode-leaf.ts';
import type {
  TaggedTree,
  TaggedValue,
} from './tagged-types.ts';

//region Builder model

/**
 Mutable table built during the walk; assignable to the readonly {@link TaggedTree}.
 */
type BuildTable = { [key: string]: BuildNode; };

/**
 Mutable tree node mirroring {@link TaggedTree} while the structure is assembled.
 */
type BuildNode = TaggedValue | BuildNode[] | BuildTable;

/**
 Container node a path step can descend into.
 */
type BuildContainer = BuildTable | BuildNode[];

/**
 Test whether a tree node is a tagged scalar leaf rather than a container.
 
 Typed over the deeply-readonly {@link TaggedTree} so the mutable builder union stays
 out of a parameter position; a leaf carries a string `type`, while a table's
 `type` entry (if any) holds a nested node, making the check decisive.
 
 @param node - Tree node to classify.
 
 @returns True when `node` is a tagged scalar.
 
 @example
 ```ts
 isLeaf(node); // false for a table
 ```
 */
function isLeaf(node: TaggedTree,): node is TaggedValue {
  return ('type' in node) && ((typeof node.type) === 'string');
}

/**
 Path of string table/key segments and numeric array-of-tables indices.
 */
type AbsolutePath = readonly (string | number)[];

//endregion Builder model

//region Document builder

/**
 Creates document root for direct insertion helpers.
 
 @returns Empty mutable build table.
 
 @example
 ```ts
 const root = createBuilder();
 addKeyValue({ root, path: ['a'], content: valueNode, });
 ```
 */
function createBuilder(): BuildTable {
  return {};
}

  /**
   Descend to the container holding `path`'s final segment, creating missing
   intermediate tables and arrays; a numeric next segment makes an array.
   
   @param path - Absolute path whose parent container is wanted.
   
   @returns Container holding the final segment.
   
   @throws Error when a prefix descends through a scalar.
   
   @example
   ```ts
   descend(['a', 'b']); // container at `a`
   ```
   */
function descend({
  root,
  path,
}: {
  readonly root: BuildTable;
  readonly path: AbsolutePath;
},): BuildContainer {
  return path
    .slice(
    0,
    -1,
    )
    .reduce<BuildContainer>(
        /**
         Descends one path segment and creates missing child container.
         
         @param current - Mutable builder container at current path prefix.
         
         @param seg - Current path segment.
         
         @param index - Segment offset used to inspect next segment kind.
         
         @returns Child container for next step.
         
         @mutates current - Stores newly created child container when path prefix is absent.
         */
    function step(
      current,
      seg,
      index,
    ) {
          /**
           Existing child at this step, if the prefix was already built.
           */
      const existing = Array.isArray(current,)
        ? (((typeof seg) === 'number') ? current[seg] : undefined)
        : (((typeof seg) === 'string') ? current[seg] : undefined);
          /**
           Child to descend into, created to match the next segment's kind.
           */
      const child: BuildNode = existing ?? (((typeof path[index + 1]) === 'number') ? [] : {});
      if (existing === undefined) {
        if (Array.isArray(current,)) {
          if ((typeof seg) === 'number') current[seg] = child;
        }
        else if ((typeof seg) === 'string') current[seg] = child;
      }
      if (isLeaf(child,))
        throw new Error('conformance decode: path descends through a scalar',);
      return child;
    },
    root,
    );
}

  /**
   Insert one key-value's tagged content at its absolute path.
   
   @param path - Absolute insertion path.
   
   @param content - Parsed content node to tag and insert.
   
   @example
   ```ts
   addKeyValue({ path: ['a'], content: valueNode, });
   ```
   */
function addKeyValue({
  root,
  path,
  content,
}: {
  readonly root: BuildTable;
  readonly path: AbsolutePath;
  readonly content: AST.TOMLNode
},): void {
    /**
     Tagged value to place, computed before navigation.
     */
  const value = contentToTagged({ node: content, },);
    /**
     Container holding the final segment.
     */
  const parent = descend({
    root,
    path,
  },);
    /**
     Final path segment.
     */
  const last = path.at(-1,) ?? '';
  if (Array.isArray(parent,)) {
    if ((typeof last) === 'number') parent[last] = value;
    return;
  }
  if ((typeof last) === 'string') parent[last] = value;
}

  /**
   Ensure an empty table exists at `resolvedKey`, then insert its entries.
   
   @param resolvedKey - Absolute table path.
   
   @param entries - Key-values declared under the table header.
   
   @example
   ```ts
   addTable({ resolvedKey: ['a'], entries: [], });
   ```
   */
function addTable(
  {
    root,
    resolvedKey,
    entries,
  }: {
    readonly root: BuildTable;
    readonly resolvedKey: AbsolutePath;
    readonly entries: readonly AST.TOMLKeyValue[]
  },
): void {
    /**
     Container holding the table's final segment.
     */
  const parent = descend({
    root,
    path: resolvedKey,
  },);
    /**
     Final segment naming the table within its parent.
     */
  const last = resolvedKey.at(-1,) ?? '';
  if (Array.isArray(parent,)) {
    if (((typeof last) === 'number') && (parent[last] === undefined)) parent[last] = {};
  }
  else if (((typeof last) === 'string') && (parent[last] === undefined)) parent[last] = {};
  for (const entry of entries)
    addKeyValue({
      root,
      path: [
        ...resolvedKey,
        ...keyPath({ key: entry.key, },),
      ],
      content: entry.value,
    },);
}

//endregion Document builder

//region Content and document walk

/**
 Convert a content node (scalar, array, or inline table) to a tagged tree,
 delegating scalar leaves to {@link leafToTagged}.
 
 @param node - Parsed content node, typed as the broad `TOMLNode` alias so the
               mutable parser union stays out of a narrower parameter position.
 
 @returns Tagged scalar, tagged array, or tagged table.
 
 @throws Error when the node is not a value, array, or inline table.
 
 @example
 ```ts
 contentToTagged({ node, }); // { type: 'string', value: 'x' }
 ```
 */
function contentToTagged({ node, }: { readonly node: AST.TOMLNode; },): BuildNode {
  if (node.type === 'TOMLValue')
    return leafToTagged({ node, },);
  if (node.type === 'TOMLArray')
    return node.elements
      .map(function each(element: AST.TOMLNode,) {
        return contentToTagged({ node: element, },);
      },);
  if (node.type === 'TOMLInlineTable') {
    /**
     Nested builder so dotted keys inside the inline table nest correctly.
     */
    const inline = createBuilder();
    for (const entry of node.body)
      addKeyValue({
        root: inline,
        path: keyPath({ key: entry.key, },),
        content: entry.value,
      },);
    return inline;
  }
  throw new Error('conformance decode: unexpected content node',);
}

/**
 Walk a parsed program into the toml-test tagged tree for its document.
 
 @param program - Parsed TOML program.
 
 @returns Tagged tree rooted at an object, ready to serialize as JSON.
 
 @example
 ```ts
 documentToTagged({ program, }); // { a: { type: 'integer', value: '42' } }
 ```
 */
export function documentToTagged({
  program,
}: {
  readonly program: AST.TOMLProgram;
},): TaggedTree {
  /**
   Builder accumulating every key-value and table.
   */
  const builder = createBuilder();
  for (const item of program.body[0]
    .body) {
    if (item.type === 'TOMLKeyValue') {
      addKeyValue({
        root: builder,
        path: keyPath({ key: item.key, },),
        content: item.value,
      },);
      continue;
    }
    addTable({
      root: builder,
      resolvedKey: item.resolvedKey,
      entries: item.body,
    },);
  }
  return builder;
}

//endregion Content and document walk

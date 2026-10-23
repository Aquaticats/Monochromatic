import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  startsAny,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships module line
// THE LINES MDX READS AS AN ES MODULE: a flow block at the first column of the document's own level that starts with
// `import ` or `export ` is read by acorn as JavaScript, and prose that starts so is refused. A backslash cannot
// make it plain; the first letter written as a character reference does.

/**
 What a flow block starts with when MDX reads it as an ES module: `import` or `export` and one space.
 */
const ES_MODULE_STARTS: readonly string[] = [
  'import ',
  'export ',
];

/**
 Whether MDX reads the node as an ES module: a paragraph or a setext heading at the first column of the document's own
 level that starts with `import ` or `export `.

 @param source - the text

 @param node - node of the parse

 @param isTopLevel - whether the node's parent is the root

 @param range - where the node stands

 @returns Whether the first word of the node must be written as a character reference

 @example
 ```ts
 const esm = startsEsModule({ source, node, isTopLevel: true, range, },);
 ```
 */
export function startsEsModule(
  {
    source,
    node,
    isTopLevel,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly isTopLevel: boolean;
    readonly range: TextRange;
  },
): boolean {
  // An ATX heading starts with its marker, so only a setext heading, whose first line is the module line, can match.
  if ((node.type !== 'paragraph') && (node.type !== 'heading'))
    return false;
  if (!isTopLevel)
    return false;
  /**
   Column the node starts at, where a node that follows indentation does not start at the first.
   */
  const column = node.position
    ?.start
    .column;
  if (column !== 1)
    return false;
  return startsAny({
    text: source,
    at: range.start,
    candidates: ES_MODULE_STARTS,
  },);
}

//endregion Source ships module line

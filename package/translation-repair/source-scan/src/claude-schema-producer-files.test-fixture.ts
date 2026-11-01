import {
  childNodes,
  identifierName,
  isTreeNode,
  literalText,
  nodesUnder,
  parseSource,
  type TreeNode,
} from './source-scan.test-fixture.ts';

//region Claude schema producer files
// THE FILES OF THE PRODUCER LIST, READ OFF ITS SOURCE. The list is `CLAUDE_SCHEMA_PRODUCERS` in the corpus-pass
// package's `claude-schema-producers.ts`, a runtime value of the formats each producer builds. The scan of the family's
// source (`claude-schema-source-search.unit.test.ts`) needs only the file of each entry, and a scan cannot import the
// corpus-pass package without declaring it, so it reads the file column off the text. The corpus-pass package holds
// this reading to the list it runs, so the two cannot drift.

/**
 Name of the constant that holds the producer list.
 */
const PRODUCERS_CONSTANT = 'CLAUDE_SCHEMA_PRODUCERS';

/**
 The file of a producer written as an object literal.

 @param entry - object node, one element of the list

 @returns Text of its `file` property

 @throws {@link Error} when the entry has no `file` written as a string literal, since a producer the scan cannot place
 would leave its file out of the comparison

 @example
 ```ts
 const file = fileOfEntry({ entry, },); // 'adjudication/src/adjudicate-wire.ts'
 ```
 */
function fileOfEntry({ entry, }: { readonly entry: TreeNode; }): string {
  /**
   The property named `file`.
   */
  const property = childNodes({ node: entry, },)
    .find(function isFile(node,): boolean {
      return (node.type === 'Property') && (identifierName({ node: node.key, },) === 'file');
    },);
  /**
   Its string value.
   */
  const file = isTreeNode(property,) ? literalText({ node: property.value, },) : '';
  if (file === '')
    throw new Error('a producer of the list writes no file as a string literal',);
  return file;
}

/**
 The files the producer list names, read off the text that declares it.

 @param text - text of `claude-schema-producers.ts`

 @returns File of each producer, in the order written

 @throws {@link Error} when the text declares no array under the list's name, or an entry names no file

 @example
 ```ts
 const files = producerFilesOf({ text, },); // ['adjudication/src/adjudicate-wire.ts', ...]
 ```
 */
export function producerFilesOf({ text, }: { readonly text: string; }): readonly string[] {
  /**
   The declaration of the list.
   */
  const declaration = nodesUnder({
    root: parseSource({
      file: {
        path: 'corpus-pass/claude-schema-producers.ts',
        text,
        isTest: false,
      },
    },)
      .program,
  },)
    .find(function declaresList(node,): boolean {
      return (node.type === 'VariableDeclarator') && (identifierName({ node: node.id, },) === PRODUCERS_CONSTANT)
        && isTreeNode(node.init,)
        && (node.init
          .type
          === 'ArrayExpression');
    },);
  if (declaration === undefined)
    throw new Error(`the text declares no array under the name ${PRODUCERS_CONSTANT}`,);
  /**
   The array the list is.
   */
  const list = declaration.init;
  if (!isTreeNode(list,))
    throw new Error(`the text declares no array under the name ${PRODUCERS_CONSTANT}`,);
  return childNodes({ node: list, },)
    .filter(function isEntry(node,): boolean {
      return node.type === 'ObjectExpression';
    },)
    .map(function fileOf(entry,): string {
      return fileOfEntry({ entry, },);
    },);
}

//endregion Claude schema producer files

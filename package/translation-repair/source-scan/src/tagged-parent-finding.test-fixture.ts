import {
  localNamesOf,
  nodeListOf,
  TAGGED,
} from './entry-logger-finding.test-fixture.ts';
import {
  identifierName,
  nodesUnder,
  parseSource,
  type SourceText,
} from './source-scan.test-fixture.ts';

//region Tagged parent finding
// WHAT THE LOGGER-ROOT SCAN READS (ledger B1110), kept apart from the scan so the finding can be run over any text.
// `tagged` writes through the default logger of logger-core when it is given no `l`, and a pass keeps its records in a
// run folder by installing a run log that only a logger rooted on `runLogger` (or on a `contextRoot`, which is one)
// reaches. A call of `tagged`, under whatever name the file binds it (an import with another local name, a const
// alias), is found when the options it is given carry no `l` property and no spread, which could carry one.

/**
 Whether a property node is a spread, which may bring an `l` the scan cannot see.

 @param property - property or spread of an options object

 @returns Whether it is a spread

 @example
 ```ts
 const spread = isSpread({ property, },);
 ```
 */
function isSpread({ property, }: { readonly property: { readonly type: string; }; },): boolean {
  return property.type === 'SpreadElement';
}

/**
 Calls of `tagged` in one file that are given no parent logger, as the line each call begins on.

 @param file - file read

 @returns Line numbers of the calls, ascending

 @example
 ```ts
 taggedWithoutParentIn({ file: { path: 'corpus-pass/a.ts', text: "const l = tagged({ tag: 'cat', },);", isTest: false, }, },); // [1]
 ```
 */
export function taggedWithoutParentIn({ file, }: { readonly file: SourceText; },): readonly number[] {
  /**
   Text of the file, which a file that never names `tagged` cannot call it by, or by an alias of it.
   */
  const { text, } = file;
  if (!text.includes(TAGGED,))
    return [];

  /**
   The file's tree.
   */
  const { program, } = parseSource({ file, },);

  /**
   Every node of the file.
   */
  const nodes = nodesUnder({ root: program, },);

  /**
   Names the file calls `tagged` by.
   */
  const taggedNames = localNamesOf({
    nodes,
    imported: TAGGED,
  },);
  return nodes
    .filter(function isTaggedCall(node,): boolean {
      return (node.type === 'CallExpression') && taggedNames.has(identifierName({ node: node.callee, },),);
    },)
    .filter(function hasNoParent(call,): boolean {
      /**
       Properties of every options object the call is given.
       */
      const properties = nodeListOf({ value: call.arguments, },)
        .filter(function isOptions(argument,): boolean {
          return argument.type === 'ObjectExpression';
        },)
        .flatMap(function propertiesOf(options,) {
          return nodeListOf({ value: options.properties, },);
        },);
      return !properties.some(function carriesParent(property,): boolean {
        return isSpread({ property, },)
          || ((property.type === 'Property') && (identifierName({ node: property.key, },) === 'l'));
      },);
    },)
    .map(function lineOf(call,): number {
      /**
       Text before the call, whose lines are the lines above it.
       */
      const before = text.slice(
        0,
        call.start,
      );
      return before.split('\n',)
        .length;
    },)
    .toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },);
}

//endregion Tagged parent finding

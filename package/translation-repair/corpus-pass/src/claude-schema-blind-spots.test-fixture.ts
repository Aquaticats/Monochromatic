import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  identifierName,
  nodesUnder,
  parentsOf,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  bearingFilesOf,
  carryingObjectsOf,
} from './claude-schema-blind-bearing.test-fixture.ts';
import {
  builderShapesIn,
  chainVisitsIn,
} from './claude-schema-blind-builders.test-fixture.ts';
import {
  loadShapesIn,
  namesBearingModule,
} from './claude-schema-blind-loads.test-fixture.ts';
import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import { mayLeadToBuilder, } from './claude-schema-blind-standing.test-fixture.ts';
import {
  namesUnreadableCode,
  unreadableShapesIn,
} from './claude-schema-blind-unreadable.test-fixture.ts';
import {
  buildersIn,
  familyOf,
  type SourceFile,
} from './claude-schema-sources.test-fixture.ts';
import { moduleLoadOf, } from './claude-schema-tree.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region Places the schema search reads nothing of, found in the family's sources
// The check behind the header of `claude-schema-sources.test-fixture.ts`: the search that keeps the producer list
// honest follows a fixed set of positions, and a place it does not follow can hide a file that sends an answer schema.
// `blindSpotsOf` reads the same sources and holds three totals. Every string that names a module which carries a builder
// must sit in a position the search follows, and every `import()` and `require` must have a path that is one literal,
// and no code may build a loader or run text (`claude-schema-blind-unreadable.test-fixture.ts`); every chain of names
// that stands for a builder, or for a module that carries one, must sit where the search reads it; and the value of a
// builder's call must end in a sender, a wrapper or a constant that does (`claude-schema-blind-flow.test-fixture.ts`).
// Anything else is reported with the file, the line and the position, so a place the search cannot read never turns up
// in the family without a failure that names it. The check is a net, not a second search: it reports every place it
// does not understand, whether or not a producer hides there, and a place it reports is answered by reading the place
// in the search or by naming the place in the test.

/**
 The shapes the check reports, each as the name a report gives it.
 */
type BlindShape =
  | 'builder-in-typescript-value-construct'
    | 'builder-outside-a-read-position'
    | 'builder-result-outside-a-destination'
    | 'builder-result-returned-outside-a-wrapper'
    | 'code-the-search-cannot-read'
    | 'computed-load-path'
    | 'format-in-undeclared-function'
    | 'load-in-unfollowed-form'
    | 'module-binding-outside-a-read-position'
    | 'module-path-outside-a-load';

/**
 One place in a source file that the search does not read.
 */
export type BlindSpot = {
  /**
   Path of the file under `package/translation-repair`.
   */
  readonly file: string;

  /**
   Line of the place, counted from one.
   */
  readonly line: number;

  /**
   The shape the place holds.
   */
  readonly shape: BlindShape;

  /**
   Where the place sits: the kind of the node that holds it and the field, or what is wrong with it.
   */
  readonly position: string;
};

/**
 A shape found in a file and the node that holds it, before the node is turned into a line.
 */
export type FoundShape = {
  /**
   The shape.
   */
  readonly shape: BlindShape;

  /**
   The node the place starts at.
   */
  readonly node: TreeNode;

  /**
   Where the place sits.
   */
  readonly position: string;
};

/**
 The line a node starts on.

 @param text - file's text

 @param node - node read

 @returns Line, counted from one

 @example
 ```ts
 const line = lineOf({ text: 'a\nb', node: { type: 'Identifier', start: 2, end: 3, }, },);
 // => 2
 ```
 */
function lineOf(
  {
    text,
    node,
  }: {
    readonly text: string;
    readonly node: TreeNode;
  },
): number {
  return text.slice(
    0,
    node.start,
  )
    .split('\n',)
    .length;
}

/**
 The places in the sources that the schema search does not read.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema, which the check
 does not read for a function that holds a format

 @returns One entry per place, in the order of the sources and then of the lines

 @throws {@link Error} when a file does not parse

 @example
 ```ts
 const spots = blindSpotsOf({ sources: await familySources(), notStageSchemas: new Set(), },);
 ```
 */
export function blindSpotsOf(
  {
    sources,
    notStageSchemas,
  }: {
    readonly sources: readonly SourceFile[];
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly BlindSpot[] {
  /**
   The modules of the sources.
   */
  const family = familyOf({ sources, },);
  /**
   The builders and wrappers of the sources.
   */
  const builders = buildersIn({
    family,
    notStageSchemas,
  },);
  /**
   The builders and wrappers, by where each sits and by name.
   */
  const known = {
    keys: new Set(builders.map(function keyOfBuilder(builder,): string {
      return keyOf(builder,);
    },),),
    names: new Set(builders.map(function nameOfBuilder({ name, },): string {
      return name;
    },),),
  };
  /**
   The modules that carry a builder.
   */
  const bearing = bearingFilesOf({
    family,
    builders,
  },);
  /**
   The objects that hold a builder as a member.
   */
  const carriers = carryingObjectsOf({ builders, },);
  return sources.flatMap(function spotsOfSource({
    file,
    text,
  },): readonly BlindSpot[] {
    /**
     What the file declares, binds and refers to, and its syntax tree.
     */
    const facts = nonNullishOrThrow(family.get(file,),);
    /**
     Every node of the file.
     */
    const nodes = nodesUnder({ root: facts.program, },);
    /**
     What the names of the file are read against, before the tree around them is known.
     */
    const base = {
      family,
      file,
      facts,
      builders: known,
      bearing,
      carriers,
      recognized: new Set(facts.declared
        .filter(function isKnown({ name, },): boolean {
          return known.keys
            .has(keyOf({
              file,
              name,
            },),);
        },)
        .map(function fnOf({ fn, },): TreeNode {
          return fn;
        },),),
      parents: new Map<TreeNode, TreeNode>(),
      references: [],
    } satisfies BlindContext;
    /**
     Whether to look for a function that holds a format and that no declared function holds, which a file named as no
     stage schema is not searched for.
     */
    const checksUndeclared = facts.holdsFormat && (!notStageSchemas.has(file,));
    /**
     Whether the file holds a string that names a module carrying a builder, a load, or a chain of names that could lead
     to a builder, which is the only reason to read its tree further.
     */
    const isRelevant = nodes.some(function isLoadOrPath(node,): boolean {
      return namesBearingModule({
        context: base,
        node,
      },)
        || (node.type === 'ImportExpression')
        || namesUnreadableCode({ node, })
        || ((node.type === 'CallExpression')
          && moduleLoadOf({ node, },)
            .isLoad);
    },)
      || [
        ...new Set(nodes.map(function nameOf(node,): string {
          return identifierName({ node, },);
        },),),
      ].some(function leadsToBuilder(name,): boolean {
        return (name !== '') && mayLeadToBuilder({
          context: base,
          name,
        },);
      },);
    if ((!isRelevant) && (!checksUndeclared))
      return [];
    /**
     What the names of the file are read against.
     */
    const context: BlindContext = {
      ...base,
      parents: parentsOf({ program: facts.program, },),
      references: chainVisitsIn({ program: facts.program, },),
    };
    return spotsOf({
      text,
      found: [
        ...loadShapesIn({
          context,
          nodes,
        },),
        ...unreadableShapesIn({
          context,
          nodes,
        },),
        ...builderShapesIn({
          context,
          nodes,
          declared: facts.declared
            .map(function holderOf({ holder, },): TreeNode {
              return holder;
            },),
          checksUndeclared,
        },),
      ],
      file,
    },);
  },);
}

/**
 The places a file's shapes are at, each place once, in the order of the lines.

 @param text - file's text

 @param found - shapes found, a place that holds the same shape twice listed twice

 @param file - path of the file

 @returns One entry per line and shape

 @example
 ```ts
 const spots = spotsOf({ text, found, file: 'a/src/a.ts', },);
 ```
 */
function spotsOf(
  {
    text,
    found,
    file,
  }: {
    readonly text: string;
    readonly found: readonly FoundShape[];
    readonly file: string;
  },
): readonly BlindSpot[] {
  return [
    ...new Map(found.map(function keyedSpot({
      shape,
      node,
      position,
    },): readonly [
      string,
      BlindSpot,
    ] {
      /**
       The line the place starts on.
       */
      const line = lineOf({
        text,
        node,
      },);
      return [
        `${String(line,)} ${shape}`,
        {
          file,
          line,
          shape,
          position,
        },
      ];
    },),)
      .values(),
  ].toSorted(function byLine(
    first,
    second,
  ): number {
    return first.line - second.line;
  },);
}

//endregion Places the schema search reads nothing of, found in the family's sources

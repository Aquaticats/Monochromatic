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
import { constantReadersOf, } from './claude-schema-blind-constants.test-fixture.ts';
import { baseContextOf, } from './claude-schema-blind-context.test-fixture.ts';
import {
  forwardersIn,
  knownIn,
  undeclaredForwardersIn,
  type UndeclaredForwarder,
} from './claude-schema-blind-forwarders.test-fixture.ts';
import type {
  BlindContext,
  ConstantReaders,
} from './claude-schema-blind-positions.test-fixture.ts';
import { mayLeadToBuilder, } from './claude-schema-blind-standing.test-fixture.ts';
import { unresolvedSpecifiersIn, } from './claude-schema-blind-specifiers.test-fixture.ts';
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
import {
  keyOf,
  usagesIn,
} from './claude-schema-usage.test-fixture.ts';

//region Places the schema search reads nothing of, found in the family's sources
// The check behind the header of `claude-schema-sources.test-fixture.ts`: the search that keeps the producer list
// honest follows a fixed set of positions, and a place it does not follow can hide a file that sends an answer schema.
// `blindSpotsOf` reads the same sources and holds three totals. Every string that names a module which carries a
// builder must sit in a position the search follows, every `import()` and `require` must have a path that is one
// literal, every specifier must name something the check can resolve
// (`claude-schema-blind-specifiers.test-fixture.ts`), and no code may build a loader or run text
// (`claude-schema-blind-unreadable.test-fixture.ts`); every chain of names that stands for a builder, or for a module
// that carries one, must sit where the search reads it; and the value of a builder's call must end in a sender, a
// wrapper or a constant that does (`claude-schema-blind-flow.test-fixture.ts`); and no function that hands a sender a
// builder call made from its own parameters may be one that no declaration names, since the files that call it cannot
// be listed (`claude-schema-blind-forwarders.test-fixture.ts`).
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
    | 'forwarder-in-undeclared-function'
    | 'forwarder-outside-a-read-position'
    | 'format-in-undeclared-function'
    | 'load-in-unfollowed-form'
    | 'module-binding-outside-a-read-position'
    | 'module-path-outside-a-load'
    | 'unresolved-module-specifier';

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

  /**
   The name of the function the place holds, for a shape that holds one.
   */
  readonly held?: string;
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

  /**
   The name of the function the place holds, for a shape that holds one.
   */
  readonly held?: string;
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
 The places a file's shapes are at, each place once, in the order of the lines. A shape is kept once for each node, since
 two reports of one shape on a line are two places, and a name that stood for the first would hide the second.

 @param text - file's text

 @param found - shapes found, a place that holds the same shape twice listed twice

 @param file - path of the file

 @returns One entry per node and shape

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
      held,
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
      /**
       The place once for each node and shape. A start offset tells the line, and it does not tell two nodes apart: the
       name `Function` and the chain `Function.prototype.constructor` start at one offset.
       */
      const key = `${shape} ${node.type} ${String(node.start,)} ${String(node.end,)}`;
      return (held === undefined)
        ? [
          key,
          {
            file,
            line,
            shape,
            position,
          },
        ]
        : [
          key,
          {
            file,
            line,
            shape,
            position,
            held,
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
   What the references of each file stand for.
   */
  const usages = usagesIn({ family, },);
  /**
   The functions a declaration names that hand a sender a builder call made from their own parameters, or pass a name
   made from them on.
   */
  const forwarders = forwardersIn({
    family,
    builders,
    usages,
  },);
  /**
   The forwarders, by where each sits and by name.
   */
  const knownForwarders = knownIn({ functions: forwarders, },);
  /**
   The builders, the wrappers and the forwarders, which a file reaches through a module in the same ways.
   */
  const carrying = [
    ...builders,
    ...forwarders,
  ];
  /**
   The modules that carry a builder or a forwarder.
   */
  const bearing = bearingFilesOf({
    family,
    builders: carrying,
  },);
  /**
   The objects that hold a builder or a forwarder as a member.
   */
  const carriers = carryingObjectsOf({ builders: carrying, },);
  /**
   The functions of that kind that no declaration names, by the file that writes each.
   */
  const undeclaredForwarders = Map.groupBy(
    undeclaredForwardersIn({
      family,
      builders,
      forwarders,
      usages,
    },),
    function fileOf({ file, },): string {
      return file;
    },
  );
  /**
   The contexts of the files whose tree has been read, so a file that another file's constant leads to is read once.
   */
  const contexts = new Map<string, BlindContext>();
  /**
   What the names of a file are read against, with the tree around them, made once for each file.

   @param file - path of the file

   @returns Context of the file
   */
  function contextOfFile({ file, }: { readonly file: string; },): BlindContext {
    /**
     The context made before, absent when the file has not been read.
     */
    const made = contexts.get(file,);
    if (made !== undefined)
      return made;
    /**
     What the file declares, binds and refers to, and its syntax tree.
     */
    const { program, } = nonNullishOrThrow(family.get(file,),);
    /**
     What the names of the file are read against, with the tree around them.
     */
    const read = {
      ...baseContextOf({
        family,
        file,
        known,
        forwarding: knownForwarders,
        bearing,
        carriers,
        constants,
      },),
      parents: parentsOf({ program, },),
      references: chainVisitsIn({ program, },),
    };
    contexts.set(
      file,
      read,
    );
    return read;
  }
  /**
   Where the other files read a constant a file exports.
   */
  const constants: ConstantReaders = constantReadersOf({
    family,
    contextOf: contextOfFile,
  },);
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
    const base = baseContextOf({
      family,
      file,
      known,
      forwarding: knownForwarders,
      bearing,
      carriers,
      constants,
    },);
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
    /**
     The specifiers of the file that the check cannot resolve, which are read whether or not the file is relevant
     otherwise, since an alias can name a module the search does not know.
     */
    const unresolved = unresolvedSpecifiersIn({
      family,
      file,
      nodes,
    },);
    /**
     The functions the file writes that hand a sender a builder call made from their own parameters and that no
     declaration names, as places to report.
     */
    const forwarding = (undeclaredForwarders.get(file,) ?? [])
      .map(function shapeOf({ node, }: UndeclaredForwarder,): FoundShape {
        return {
          shape: 'forwarder-in-undeclared-function',
          node,
          position: node.type,
        };
      },);
    /**
     Whether the file is read no further: nothing in it leads to a builder, it holds no undeclared format to look for,
     it writes no specifier the check cannot resolve and no function that forwards without a declaration.
     */
    const isIdle = (!isRelevant) && (!checksUndeclared);
    /**
     Whether the file writes nothing to report apart from what its tree holds.
     */
    const reportsNothingElse = (unresolved.length === 0) && (forwarding.length === 0);
    if (isIdle && reportsNothingElse)
      return [];
    /**
     What the names of the file are read against.
     */
    const context: BlindContext = contextOfFile({ file, },);
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
        ...unresolved,
        ...forwarding,
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

//endregion Places the schema search reads nothing of, found in the family's sources

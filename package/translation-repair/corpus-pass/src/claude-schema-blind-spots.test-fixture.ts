import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  nodesUnder,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { builderShapesIn, } from './claude-schema-blind-builders.test-fixture.ts';
import { loadShapesIn, } from './claude-schema-blind-loads.test-fixture.ts';
import {
  buildersIn,
  familyOf,
  type SourceFile,
} from './claude-schema-sources.test-fixture.ts';
import {
  keyOf,
  standsForKnown,
} from './claude-schema-usage.test-fixture.ts';

//region Shapes the schema search does not read, found in the family's sources
// The check behind the header of `claude-schema-sources.test-fixture.ts`: the search that keeps the producer list
// honest reads a fixed set of shapes and leaves others unread, and a shape it leaves unread can hide a file that
// sends an answer schema. `blindSpotsOf` reads the same sources and reports each place that holds one of those
// shapes, with the file and the line, so a shape the search cannot read never turns up in the family without a
// failure that names it. The check is a net, not a second search: it reports every place with the shape, whether or
// not a producer hides there, and a place it reports is answered by reading the shape in the search or by naming the
// place in the test.

/**
 The shapes the schema search leaves unread, each as the name a report gives it.
 */
type BlindShape =
  | 'builder-handed-to-a-call-held-by-a-constant'
    | 'builder-in-namespace-enum-or-parameter-property'
    | 'builder-in-typescript-value-construct'
    | 'builder-outside-a-read-position'
    | 'builder-result-returned-outside-a-wrapper'
    | 'builder-stored-in-object'
    | 'computed-load-path'
    | 'format-in-undeclared-function'
    | 'load-in-unfollowed-form'
    | 'module-binding-outside-a-read-position'
    | 'module-path-outside-a-load'
    | 'result-returned-through-a-local';

/**
 One place in a source file that holds a shape the search leaves unread.
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
   Where the place sits, as the check of the next commit reports it: the kind of node that holds it and the field, or
   what is wrong with it. This check leaves it unset.
   */
  readonly position?: string;
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
 The places in the sources that hold a shape the schema search leaves unread.

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
     What the names of the file are read against.
     */
    const context = {
      family,
      file,
      builders: known,
    };
    /**
     Every place the shapes are found at, a place that holds the same shape twice listed twice.
     */
    const found = [
      ...loadShapesIn({
        program: facts.program,
        nodes,
      },),
      ...builderShapesIn({
        context,
        nodes,
        refersToBuilder: facts.references
          .some(function isBuilder(path,): boolean {
            return standsForKnown({
              family,
              file,
              path,
              known,
            },);
          },),
        declared: facts.declared
          .map(function holderOf({ holder, },): TreeNode {
            return holder;
          },),
        checksUndeclared: facts.holdsFormat && (!notStageSchemas.has(file,)),
      },),
    ]
      .map(function spotOf({
        shape,
        node,
      },): BlindSpot {
        return {
          file,
          line: lineOf({
            text,
            node,
          },),
          shape,
        };
      },);
    return [
      ...new Map(found.map(function keyedSpot(spot,): readonly [
        string,
        BlindSpot,
      ] {
        return [
          `${String(spot.line,)} ${spot.shape}`,
          spot,
        ];
      },),)
        .values(),
    ].toSorted(function byLine(
      first,
      second,
    ): number {
      return first.line - second.line;
    },);
  },);
}

//endregion Shapes the schema search does not read, found in the family's sources

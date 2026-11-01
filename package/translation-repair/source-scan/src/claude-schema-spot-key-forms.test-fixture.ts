import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';

//region What makes two reports of the check one place
// The check keeps a report once for each node and shape, and tells a node by its type and its range: two nodes of one
// shape that differ in the end of their range (the nested members of a chain start at one offset), in their type (the
// name `Function` and the chain read off its prototype start at one offset too) or in the start of their range (two
// nodes that end at one offset) are two places, and one node found twice is one. Every part of the key is held by a
// case on nodes written for the purpose, since no source the check reads makes two reports of one shape that differ in
// one part only (a probe over the adversarial inputs and the real sources found none).

/**
 One case of the key: the nodes found and how many places they make.
 */
export type SpotKeyCase = {
  /**
   What the case holds.
   */
  readonly name: string;

  /**
   The shapes found, one entry per node and shape.
   */
  readonly found: readonly FoundShape[];

  /**
   How many places the shapes make.
   */
  readonly places: number;
};

/**
 The shape every case but one reports.
 */
const SHAPE = 'module-binding-outside-a-read-position';

/**
 Cases of the key of a report.
 */
export const SPOT_KEY_CASES: readonly SpotKeyCase[] = [
  {
    name: 'KEEPS TWO NODES OF ONE SHAPE AND ONE TYPE THAT START AT ONE OFFSET AND END AT TWO, the nested members of a '
      + 'chain',
    found: [
      {
        shape: SHAPE,
        node: {
          type: 'MemberExpression',
          start: 4,
          end: 9,
        },
        position: 'inner member',
      },
      {
        shape: SHAPE,
        node: {
          type: 'MemberExpression',
          start: 4,
          end: 12,
        },
        position: 'outer member',
      },
    ],
    places: 2,
  },
  {
    name: 'KEEPS TWO NODES OF ONE SHAPE THAT HAVE ONE RANGE AND TWO TYPES',
    found: [
      {
        shape: SHAPE,
        node: {
          type: 'Identifier',
          start: 4,
          end: 9,
        },
        position: 'a name',
      },
      {
        shape: SHAPE,
        node: {
          type: 'ExpressionStatement',
          start: 4,
          end: 9,
        },
        position: 'a statement',
      },
    ],
    places: 2,
  },
  {
    name: 'KEEPS TWO NODES OF ONE SHAPE AND ONE TYPE THAT END AT ONE OFFSET AND START AT TWO',
    found: [
      {
        shape: SHAPE,
        node: {
          type: 'MemberExpression',
          start: 4,
          end: 12,
        },
        position: 'long member',
      },
      {
        shape: SHAPE,
        node: {
          type: 'MemberExpression',
          start: 8,
          end: 12,
        },
        position: 'short member',
      },
    ],
    places: 2,
  },
  {
    name: 'KEEPS TWO SHAPES FOUND AT ONE NODE AS TWO PLACES',
    found: [
      {
        shape: SHAPE,
        node: {
          type: 'Identifier',
          start: 4,
          end: 9,
        },
        position: 'as a namespace',
      },
      {
        shape: 'forwarder-outside-a-read-position',
        node: {
          type: 'Identifier',
          start: 4,
          end: 9,
        },
        position: 'as a forwarder',
      },
    ],
    places: 2,
  },
  {
    name: 'KEEPS ONE NODE FOUND TWICE AS ONE PLACE, whatever position the second finding gives',
    found: [
      {
        shape: SHAPE,
        node: {
          type: 'Identifier',
          start: 4,
          end: 9,
        },
        position: 'first',
      },
      {
        shape: SHAPE,
        node: {
          type: 'Identifier',
          start: 4,
          end: 9,
        },
        position: 'second',
      },
    ],
    places: 1,
  },
];

//endregion What makes two reports of the check one place

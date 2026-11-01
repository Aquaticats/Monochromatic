/**
 fast-check generators for whole `deepmerge` argument lists, built on the
 tree generators in `./arbitraries.ts`: records most of the time, any trees
 otherwise, plus the edge shapes the historical-recall audit showed the model
 reached only by hand examples.

 @module
 */

import {
  array,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
  type Arbitrary,
} from 'fast-check';

import {
  CONTAINER_MAX_LENGTH,
  treeArbitraries,
  type TreeOptions,
} from './arbitraries.ts';

/**
 Switches for {@link mergeArgumentsArbitrary} beyond the tree switches.
 */
export type ArgumentOptions = TreeOptions & {
  /**
   Whether argument lists include the historical-recall edge shapes
   (`doc/audit/deepmerge-ts-recall-2026-09-24.md`): zero arguments, and
   records sharing one key whose middle value is `undefined`. Off only where
   a fixed-seed draw must stay reproducible against an earlier report.
   */
  readonly edgeCalls?: boolean;
};

/**
 Argument lists where every input is a record holding `key`, and one value
 strictly between the first and the last is `undefined`: the shape of
 upstream's `#460`, which the model reached only by chance.

 @param key - Shared key.

 @param values - At least three values for that key, in argument order.

 @param hole - Index to replace with `undefined`, reduced into the middle.

 @returns One record per value.

 @example
 ```ts
 undefinedBetween({ key: 'a', values: [{ x: 1, }, 2, { y: 3, },], hole: 0, });
 // [{ a: { x: 1 } }, { a: undefined }, { a: { y: 3 } }]
 ```
 */
function undefinedBetween(
  {
    key,
    values,
    hole,
  }: {
    readonly key: string;
    readonly values: readonly unknown[];
    readonly hole: number;
  },
): readonly unknown[] {
  /**
   Middle index: never the first or the last value.
   */
  const middle = 1 + (hole % (values.length - 2));
  return values.map(function wrap(
    value,
    index,
  ) {
    return { [key]: (index === middle) ? undefined : value, };
  },);
}

/**
 Least number of values {@link undefinedBetween} needs for a middle.
 */
const MIN_BETWEEN_VALUES = 3;

/**
 Argument lists for `deepmerge`: records most of the time, since that is the
 dominant use, otherwise any trees, plus the {@link ArgumentOptions} edge
 shapes.

 @param exotic - Forwarded to {@link treeArbitraries}.

 @param objectLeaves - Forwarded to {@link treeArbitraries}.

 @param protoKey - Forwarded to {@link treeArbitraries}.

 @param undefinedLeaves - Forwarded to {@link treeArbitraries}; off also drops the `undefined`-between shape.

 @param edgeCalls - See {@link ArgumentOptions}.

 @returns Generator of zero to four merge arguments.

 @example
 ```ts
 const inputs = mergeArgumentsArbitrary({ exotic: true, });
 ```
 */
export function mergeArgumentsArbitrary({
  exotic,
  objectLeaves = true,
  protoKey = true,
  undefinedLeaves = true,
  edgeCalls = true,
}: ArgumentOptions,): Arbitrary<readonly unknown[]> {
  /**
   Generators shared by both argument shapes.
   */
  const {
    record: recordTree,
    tree,
  } = treeArbitraries({
    exotic,
    objectLeaves,
    protoKey,
    undefinedLeaves,
  },);
  /**
   Shapes every caller gets.
   */
  const common = [
    {
      weight: 12,
      arbitrary: array(
        recordTree,
        {
          minLength: 1,
          maxLength: CONTAINER_MAX_LENGTH,
        },
      ),
    },
    {
      weight: 4,
      arbitrary: array(
        tree,
        {
          minLength: 1,
          maxLength: CONTAINER_MAX_LENGTH,
        },
      ),
    },
  ];
  /**
   Historical-recall edge shapes, when enabled.
   */
  const edges = edgeCalls
    ? [
      {
        weight: 1,
        arbitrary: constant([],),
      },
      ...(undefinedLeaves
        ? [{
          weight: 2,
          arbitrary: record({
            hole: integer({
              min: 0,
              max: CONTAINER_MAX_LENGTH,
            },),
            key: constantFrom(
              'a',
              'b',
            ),
            values: array(
              oneof(
                recordTree,
                tree,
              ),
              {
                minLength: MIN_BETWEEN_VALUES,
                maxLength: CONTAINER_MAX_LENGTH + 1,
              },
            ),
          },)
            .map(function shape(plan,) {
              return undefinedBetween(plan,);
            },),
        },]
        : []),
    ]
    : [];
  return oneof(
    ...common,
    ...edges,
  );
}

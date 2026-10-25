import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { destinationsOf, } from './claude-schema-blind-flow.test-fixture.ts';
import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import { isNoNode, } from './claude-schema-tree.test-fixture.ts';

//region Where the value of a builder's call goes
// The part of the second total of the check behind the schema search (`claude-schema-blind-chains.test-fixture.ts`) that
// reads a call of a builder: its value goes out of a function that the search does not know as a builder, or to a place
// that no sender, wrapper or constant answers for, or to code the search cannot read.

/**
 The shapes a call of a builder holds: its value goes out of a function that the search does not know as a builder, or
 to a place that no sender, wrapper or constant answers for.

 @param context - what the file's names are read against

 @param call - call, `new` or tagged template of a builder

 @returns Shapes, none where the value stays or goes to a wrapper, a sender or a constant that does the same

 @example
 ```ts
 const found = callShapesAt({ context, call, },);
 ```
 */
export function callShapesAt(
  {
    context,
    call,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
  },
): readonly FoundShape[] {
  /**
   The function the value is returned from, `NO_NODE` when it is not, and the other places the value goes.
   */
  const {
    leaving,
    leavingIn,
    strays,
    unreadable,
  } = destinationsOf({
    context,
    call,
  },);
  return [
    ...(isNoNode({ node: leaving, },)
      ? []
      : [
        {
          shape: 'builder-result-returned-outside-a-wrapper',
          node: call,
          position: `${leaving.type} returns it${(leavingIn === '') ? '' : ` in ${leavingIn}`}`,
        } satisfies FoundShape,
      ]),
    ...((strays.length === 0)
      ? []
      : [
        {
          shape: 'builder-result-outside-a-destination',
          node: call,
          position: strays.join(', ',),
        } satisfies FoundShape,
      ]),
    ...((unreadable.length === 0)
      ? []
      : [
        {
          shape: 'code-the-search-cannot-read',
          node: call,
          position: unreadable.join(', ',),
        } satisfies FoundShape,
      ]),
  ];
}

//endregion Where the value of a builder's call goes

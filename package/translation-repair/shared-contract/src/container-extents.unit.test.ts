/**
 Tests for handing a dissolved container's tags to the blocks beside them.

 The defect this exists for is that a container's opening and closing tags
 belong to no block once the container is dissolved, while every range in this
 package is minted from block offsets. A boundary could therefore fall between
 an opener and its closer, and assembly, which replaces a range and copies the
 rest through, would delete one tag and keep the other.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type BlockExtent,
  type ContainerSpan,
  widenExtentsToContainers,
} from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';
import {
  nodeOwnsCloser,
  nodeOwnsOpener,
} from '@monochromatic-dev/translation-repair-document-structure/ts/container-tag-ownership.test-fixture.ts';

/**
 Builds a container span from its two tag ranges.

 @param name - element name reported on the span

 @param opener - opening tag range

 @param closer - closing tag range

 @returns Span in the shape the parse reports

 @example
 ```ts
 const container = spanOf({ name: 'details', opener: [0, 10,], closer: [40, 52,], },);
 ```
 */
function spanOf(
  {
    name,
    opener,
    closer,
  }: {
    readonly name: string;
    readonly opener: readonly [number, number,];
    readonly closer: readonly [number, number,];
  },
): ContainerSpan {
  return {
    name,
    openerStartOffset: opener[0],
    openerEndOffset: opener[1],
    closerStartOffset: closer[0],
    closerEndOffset: closer[1],
  };
}

/**
 Reads a widened extent pair back as plain numbers for comparison.

 @param extent - extent to read

 @returns Start and end as a pair

 @example
 ```ts
 const [start, end,] = pairOf({ extent, },);
 ```
 */
function pairOf({ extent, }: { readonly extent: BlockExtent; },): readonly [number, number,] {
  return [
    extent.startOffset,
    extent.endOffset,
  ];
}

/**
 Translation packaging two of its four blocks inside a disclosure element.
 */
const SPLIT_TEXT = `The cat sleeps on the windowsill.

<details>

She watches the birds outside.

Later she eats.

</details>

She purrs at nothing in particular.
`;

await describe({
  name: widenExtentsToContainers.name,
  children: [
    it({
      name: 'ACCEPTS a container spanning several blocks by giving its opening tag to the first and '
        + 'its closing tag to the last, leaving the blocks between them untouched',
      fn: async () => {
        /** Three blocks, the outer two of which will take a tag. */
        const widened = widenExtentsToContainers({
          extents: [
            {
              startOffset: 20,
              endOffset: 30,
            },
            {
              startOffset: 32,
              endOffset: 44,
            },
            {
              startOffset: 46,
              endOffset: 60,
            },
          ],
          containers: [
            spanOf({
              name: 'details',
              opener: [
                10,
                20,
              ],
              closer: [
                60,
                72,
              ],
            },),
          ],
        },);
        expect(pairOf({ extent: nonNullishOrThrow(widened[0],), },),).toEqual([
          10,
          30,
        ],);
        expect(pairOf({ extent: nonNullishOrThrow(widened[1],), },),).toEqual([
          32,
          44,
        ],);
        expect(pairOf({ extent: nonNullishOrThrow(widened[2],), },),).toEqual([
          46,
          72,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS a container holding a SINGLE block, which takes both tags itself rather than '
        + 'one of them, since first and last are the same block',
      fn: async () => {
        /** One block carrying an element on each side of it. */
        const widened = widenExtentsToContainers({
          extents: [
            {
              startOffset: 12,
              endOffset: 40,
            },
          ],
          containers: [
            spanOf({
              name: 'details',
              opener: [
                2,
                12,
              ],
              closer: [
                40,
                52,
              ],
            },),
          ],
        },);
        expect(pairOf({ extent: nonNullishOrThrow(widened[0],), },),).toEqual([
          2,
          52,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS NESTED containers by taking the outermost opening offset and the outermost '
        + 'closing one, so an inner element does not strand its outer element half owned',
      fn: async () => {
        /** One block wrapped twice over, as the corpus entry carrying nesting does. */
        const widened = widenExtentsToContainers({
          extents: [
            {
              startOffset: 24,
              endOffset: 40,
            },
          ],
          containers: [
            spanOf({
              name: 'details',
              opener: [
                2,
                12,
              ],
              closer: [
                52,
                64,
              ],
            },),
            spanOf({
              name: 'Hexagon',
              opener: [
                12,
                24,
              ],
              closer: [
                40,
                52,
              ],
            },),
          ],
        },);
        expect(pairOf({ extent: nonNullishOrThrow(widened[0],), },),).toEqual([
          2,
          64,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS a container whose closing tag ends the DOCUMENT, which is the shape whose tag '
        + 'used to fall past the last chunk boundary and outside every range',
      fn: async () => {
        /** A block whose element closes at the final character. */
        const widened = widenExtentsToContainers({
          extents: [
            {
              startOffset: 18,
              endOffset: 44,
            },
          ],
          containers: [
            spanOf({
              name: 'p',
              opener: [
                0,
                18,
              ],
              closer: [
                44,
                49,
              ],
            },),
          ],
        },);
        expect(pairOf({ extent: nonNullishOrThrow(widened[0],), },),).toEqual([
          0,
          49,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS an EMPTY container by widening nothing, since its tags have no block to ride '
        + 'in and no range can reach them either',
      fn: async () => {
        /** A block sitting outside the empty element entirely. */
        const widened = widenExtentsToContainers({
          extents: [
            {
              startOffset: 40,
              endOffset: 60,
            },
          ],
          containers: [
            spanOf({
              name: 'details',
              opener: [
                2,
                12,
              ],
              closer: [
                12,
                24,
              ],
            },),
          ],
        },);
        expect(pairOf({ extent: nonNullishOrThrow(widened[0],), },),).toEqual([
          40,
          60,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS a page carrying NO container by returning its extents unchanged, which is what '
        + 'keeps the fix a no-op on the pages it was not written for',
      fn: async () => {
        /** Two ordinary blocks with nothing wrapped around them. */
        const widened = widenExtentsToContainers({
          extents: [
            {
              startOffset: 0,
              endOffset: 10,
            },
            {
              startOffset: 12,
              endOffset: 30,
            },
          ],
          containers: [],
        },);
        expect(pairOf({ extent: nonNullishOrThrow(widened[0],), },),).toEqual([
          0,
          10,
        ],);
        expect(pairOf({ extent: nonNullishOrThrow(widened[1],), },),).toEqual([
          12,
          30,
        ],);
      },
    },),
    it({
      name: 'FORWARDS the widening through parseDocument, so a parsed page reports blocks that '
        + 'already own their container tags rather than blocks a later caller must widen',
      fn: async () => {
        /** Parsed fixture whose disclosure element wraps two of its four blocks. */
        const parsed = parseDocument({ text: SPLIT_TEXT, },);

        /** Its single container, whose tags the parse must have handed out. */
        const [container,] = parsed.containers;
        if (container === undefined)
          throw new Error('fixture reported no container, so this case would pass for the wrong reason',);
        expect(parsed.nodes
          .some(function ownsOpener(node,): boolean {
            return nodeOwnsOpener({ node, container, },);
          },),).toBe(true,);
        expect(parsed.nodes
          .some(function ownsCloser(node,): boolean {
            return nodeOwnsCloser({ node, container, },);
          },),).toBe(true,);
      },
    },),
  ],
},);

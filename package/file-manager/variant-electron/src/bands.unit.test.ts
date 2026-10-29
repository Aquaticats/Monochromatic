/**
 Unit tests for the sticky rail-band geometry.
 
 The decisive invariant is structural non-overlap: within every column, flow
 margins between rail wrappers are never negative, so sibling rails can
 never intersect no matter how the strip scrolls. Tests import from built
 `dist/app` so they verify the artifact the renderer consumes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  computeColumnLayouts,
  railHeightPx,
  rowY,
} from '../dist/app/bands.js';
import {
  createStrip,
  directoryLocation,
  openRoot,
  paneById,
  spawnChild,
  type Strip,
} from '../dist/app/strip.js';

/** Pane height mirrored from constants for readable expectations. */
const paneHeight = 520;

/** Row stride (pane height plus gap) mirrored for readable expectations. */
const rowStride = 532;

/**
 Builds the strip used by most cases: a root with three children, the middle
 child grown into a two-leaf subtree, mirroring the Rust model test
 `a_later_sibling_is_pushed_below_a_grown_subtree`.
 */
function grownStrip(): Strip {
  const root = openRoot({
    location: directoryLocation({ path: '/home', },),
    strip: createStrip(),
  },);
  const a = spawnChild({
    forceDuplicate: false,
    location: directoryLocation({ path: '/home/a', },),
    parent: root.id,
    strip: root.strip,
  },);
  const b = spawnChild({
    forceDuplicate: false,
    location: directoryLocation({ path: '/home/b', },),
    parent: root.id,
    strip: a.strip,
  },);
  const c = spawnChild({
    forceDuplicate: false,
    location: directoryLocation({ path: '/home/c', },),
    parent: root.id,
    strip: b.strip,
  },);
  const bx = spawnChild({
    forceDuplicate: false,
    location: directoryLocation({ path: '/home/b/x', },),
    parent: b.id,
    strip: c.strip,
  },);
  const by = spawnChild({
    forceDuplicate: false,
    location: directoryLocation({ path: '/home/b/y', },),
    parent: b.id,
    strip: bx.strip,
  },);
  return by.strip;
}

await describe({
  name: '',
  children: [
    describe({
      name: rowY.name,
      children: [
        it({
          name: 'tiles rows at the fixed stride',
          fn: async () => {
            expect(rowY({ row: 0, },),).toBe(0,);
            expect(rowY({ row: 2, },),).toBe(2 * rowStride,);
          },
        },),
      ],
    },),
    describe({
      name: railHeightPx.name,
      children: [
        it({
          name: 'is one pane tall for a leaf',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const pane = paneById({
              id: root.id,
              strip: root.strip,
            },);
            expect((typeof pane) === 'symbol',).toBe(false,);
            if ((typeof pane) === 'symbol')
              return;

            expect(railHeightPx({
              pane,
              panes: root.strip.panes,
            },),).toBe(paneHeight,);
          },
        },),
        it({
          name: 'stretches a parent rail to its deepest direct child bottom',
          fn: async () => {
            const strip = grownStrip();
            /**
             Root pane at row 0 whose direct children sit at rows 0, 1, 3.
             */
            const root = strip.panes
              .find(function isRoot(pane,): boolean {
                return pane.parent === undefined;
              },);
            expect(root === undefined,).toBe(false,);
            if (root === undefined)
              return;

            // Deepest direct child (c) is at row 3: rail spans rows 0..3 plus
            // one pane height.
            expect(railHeightPx({
              pane: root,
              panes: strip.panes,
            },),).toBe((3 * rowStride) + paneHeight,);
          },
        },),
        it({
          name: 'ignores grandchildren when sizing a rail',
          fn: async () => {
            const strip = grownStrip();
            /**
             Pane b at row 1 whose direct children x, y sit at rows 1, 2;
             b's rail must not include c's row even though c sits below.
             */
            const b = strip.panes
              .find(function isB(pane,): boolean {
                return pane.location.path === '/home/b';
              },);
            expect(b === undefined,).toBe(false,);
            if (b === undefined)
              return;

            expect(railHeightPx({
              pane: b,
              panes: strip.panes,
            },),).toBe(rowStride + paneHeight,);
          },
        },),
      ],
    },),
    describe({
      name: computeColumnLayouts.name,
      children: [
        it({
          name: 'is empty for an empty strip',
          fn: async () => {
            expect(computeColumnLayouts({ panes: [], },).length,).toBe(0,);
          },
        },),
        it({
          name: 'places bands at their global grid offsets via margins',
          fn: async () => {
            const strip = grownStrip();
            // Columns: root, root's children, and b's grandchildren.
            const layouts = computeColumnLayouts({ panes: strip.panes, },);
            expect(layouts.length,).toBe(3,);

            /**
             Column 1 rails: a (row 0), b (row 1), c (row 3).
             */
            const [, columnOne,] = layouts;
            expect(columnOne === undefined,).toBe(false,);
            if (columnOne === undefined)
              return;

            expect(columnOne.rails.length,).toBe(3,);
            // a: leaf at row 0 with no flow gap above.
            expect(columnOne.rails[0]?.marginTopPx,).toBe(0,);
            expect(columnOne.rails[0]?.railHeightPx,).toBe(paneHeight,);
            // b: starts at row 1; the gap above is one row stride minus a's
            // rail height.
            expect(columnOne.rails[1]?.marginTopPx,).toBe(rowStride - paneHeight,);
            expect(columnOne.rails[1]?.railHeightPx,).toBe(rowStride + paneHeight,);
            // c: starts at row 3, below b's two-row rail.
            expect(columnOne.rails[2]?.marginTopPx,).toBe(
              (3 * rowStride) - (rowStride + (rowStride + paneHeight)),
            );
          },
        },),
        it({
          name: 'never produces a negative margin (structural non-overlap)',
          fn: async () => {
            const strip = grownStrip();
            const layouts = computeColumnLayouts({ panes: strip.panes, },);
            const margins = layouts.flatMap(function railMargins(layout,): readonly number[] {
              return layout.rails
                .map(function marginOf(rail,): number {
                  return rail.marginTopPx;
                },);
            },);
            expect(margins.every(function nonNegative(margin,): boolean {
              return margin >= 0;
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

/**
 Unit tests for the pane-strip state machine.
 
 Mirrors `package/file-manager/app/src/model_tests.rs` case for
 case (for the ported surface) so the TS port and the Rust original stay
 behaviorally identical. Tests import from built `dist/app` so they verify
 the artifact the Electron renderer consumes, not sibling source.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  closePane,
  columnCount,
  createStrip,
  directoryLocation,
  firstPaneInColumn,
  focusPane,
  openRoot,
  paneById,
  previewLocation,
  spawnChild,
  type Pane,
  type PaneId,
  type Strip,
} from '../dist/app/strip.js';

/**
 Looks a pane up and fails the calling test when it is not live.
 */
function mustFind(
  {
    id,
    strip,
  }: {
    readonly id: PaneId;
    readonly strip: Strip;
  },
): Pane {
  const pane = paneById({
    id,
    strip,
  },);
  if ((typeof pane) === 'symbol')
    throw new Error(`Pane ${id} is not live`,);

  return pane;
}

await describe({
  name: '',
  children: [
    describe({
      name: openRoot.name,
      children: [
        it({
          name: 'places pane in column zero and focuses it',
          fn: async () => {
            const opened = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            expect(opened.strip.panes.length,).toBe(1,);
            expect(opened.strip.active,).toBe(opened.id,);
            const pane = mustFind({
              id: opened.id,
              strip: opened.strip,
            },);
            expect(pane.column,).toBe(0,);
            expect(pane.row,).toBe(0,);
            const top = firstPaneInColumn({
              column: 0,
              strip: opened.strip,
            },);
            expect((typeof top) === 'symbol',).toBe(false,);
            if ((typeof top) === 'symbol')
              return;

            expect(top.id,).toBe(opened.id,);
          },
        },),
        it({
          name: 'dedups a reopened root location and focuses the existing pane',
          fn: async () => {
            const first = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const again = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: first.strip,
            },);
            expect(again.id,).toBe(first.id,);
            expect(again.strip.panes.length,).toBe(1,);
          },
        },),
      ],
    },),
    describe({
      name: spawnChild.name,
      children: [
        it({
          name: 'opens next column aligned to the parent row',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const child = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: root.strip,
            },);
            expect(child.id === root.id,).toBe(false,);
            const pane = mustFind({
              id: child.id,
              strip: child.strip,
            },);
            expect(pane.column,).toBe(1,);
            expect(pane.row,).toBe(0,);
            expect(child.strip.active,).toBe(child.id,);
            expect(child.strip.panes.length,).toBe(2,);
          },
        },),
        it({
          name: 'aligns child with parent row while siblings stack downward',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const first = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/a', },),
              parent: root.id,
              strip: root.strip,
            },);
            const second = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/b', },),
              parent: root.id,
              strip: first.strip,
            },);
            expect(mustFind({
              id: first.id,
              strip: second.strip,
            },).row,).toBe(0,);
            expect(mustFind({
              id: second.id,
              strip: second.strip,
            },).row,).toBe(1,);
            const grandchild = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/b/c', },),
              parent: second.id,
              strip: second.strip,
            },);
            const pane = mustFind({
              id: grandchild.id,
              strip: grandchild.strip,
            },);
            expect(pane.column,).toBe(2,);
            expect(pane.row,).toBe(1,);
          },
        },),
        it({
          name: 'pushes a later sibling below a grown subtree',
          fn: async () => {
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
            expect(mustFind({
              id: a.id,
              strip: by.strip,
            },).row,).toBe(0,);
            expect(mustFind({
              id: b.id,
              strip: by.strip,
            },).row,).toBe(1,);
            expect(mustFind({
              id: c.id,
              strip: by.strip,
            },).row,).toBe(3,);
          },
        },),
        it({
          name: 'dedups a revisited location and focuses the existing pane',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const first = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: root.strip,
            },);
            const refocused = focusPane({
              id: root.id,
              strip: first.strip,
            },);
            const again = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: refocused,
            },);
            expect(again.id,).toBe(first.id,);
            expect(again.strip.panes.length,).toBe(2,);
            expect(again.strip.active,).toBe(first.id,);
          },
        },),
        it({
          name: 'forces a duplicate pane when requested',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const first = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: root.strip,
            },);
            const dup = spawnChild({
              forceDuplicate: true,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: first.strip,
            },);
            expect(dup.id === first.id,).toBe(false,);
            expect(dup.strip.panes.length,).toBe(3,);
          },
        },),
        it({
          name: 'treats preview and directory locations as distinct dedup keys',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const listing = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/x', },),
              parent: root.id,
              strip: root.strip,
            },);
            const preview = spawnChild({
              forceDuplicate: false,
              location: previewLocation({ path: '/home/x', },),
              parent: root.id,
              strip: listing.strip,
            },);
            expect(preview.id === listing.id,).toBe(false,);
            expect(preview.strip.panes.length,).toBe(3,);
          },
        },),
      ],
    },),
    describe({
      name: closePane.name,
      children: [
        it({
          name: 'removes the pane and clears its dedup registration',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const child = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: root.strip,
            },);
            const closed = closePane({
              id: child.id,
              strip: child.strip,
            },);
            expect(closed.panes.length,).toBe(1,);
            expect(typeof paneById({
              id: child.id,
              strip: closed,
            },),).toBe('symbol',);
            const respawned = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: closed,
            },);
            expect(respawned.id === child.id,).toBe(false,);
            expect(respawned.strip.panes.length,).toBe(2,);
          },
        },),
        it({
          name: 'orphans children of a closed parent into roots',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const child = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: root.strip,
            },);
            const closed = closePane({
              id: root.id,
              strip: child.strip,
            },);
            const orphan = mustFind({
              id: child.id,
              strip: closed,
            },);
            expect(orphan.parent,).toBe(undefined,);
            expect(orphan.row,).toBe(0,);
            expect(closed.panes.length,).toBe(1,);
          },
        },),
        it({
          name: 'clears focus when the focused pane closes',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const closed = closePane({
              id: root.id,
              strip: root.strip,
            },);
            expect(closed.active,).toBe(undefined,);
          },
        },),
      ],
    },),
    describe({
      name: columnCount.name,
      children: [
        it({
          name: 'is zero for an empty strip and one past the deepest column',
          fn: async () => {
            expect(columnCount({ strip: createStrip(), },),).toBe(0,);
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const child = spawnChild({
              forceDuplicate: false,
              location: directoryLocation({ path: '/home/docs', },),
              parent: root.id,
              strip: root.strip,
            },);
            expect(columnCount({ strip: child.strip, },),).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: focusPane.name,
      children: [
        it({
          name: 'ignores a stale pane id',
          fn: async () => {
            const root = openRoot({
              location: directoryLocation({ path: '/home', },),
              strip: createStrip(),
            },);
            const closed = closePane({
              id: root.id,
              strip: root.strip,
            },);
            const refocused = focusPane({
              id: root.id,
              strip: closed,
            },);
            expect(refocused.active,).toBe(undefined,);
          },
        },),
      ],
    },),
  ],
},);

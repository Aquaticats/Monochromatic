/**
 Tests for the log scans: shortcut uses and the nested Wayland display name.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  countShortcutUses,
  isDisplayName,
  waylandDisplayOf,
} from './log-scan.ts';

await describe({
  name: countShortcutUses.name,
  children: [
    it({
      name: 'counts every shortcut use when no action is named',
      fn: async ({ expect, }) => {
        const lines = ['INFO shortcut used action="reveal"', 'noise', 'INFO shortcut used action="row-details"',];
        expect(countShortcutUses({ lines, },),).toBe(2,);
      },
    },),
    it({
      name: 'counts only the named action',
      fn: async ({ expect, }) => {
        const lines = ['INFO shortcut used action="reveal"', 'INFO shortcut used action="row-details"',];
        expect(countShortcutUses({ lines, action: 'reveal', },),).toBe(1,);
        expect(countShortcutUses({ lines, action: 'missing', },),).toBe(0,);
      },
    },),
    it({
      name: 'does not count a named action that is not a use line',
      fn: async ({ expect, }) => {
        expect(countShortcutUses({ lines: ['action="reveal" only',], action: 'reveal', },),).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: isDisplayName.name,
  children: [
    it({
      name: 'accepts wayland-N with digits only',
      fn: async ({ expect, }) => {
        expect(isDisplayName('wayland-0',),).toBe(true,);
        expect(isDisplayName('wayland-12',),).toBe(true,);
      },
    },),
    it({
      name: 'rejects other names',
      fn: async ({ expect, }) => {
        expect(isDisplayName('wayland-',),).toBe(false,);
        expect(isDisplayName('wayland-1a',),).toBe(false,);
        expect(isDisplayName('x11-1',),).toBe(false,);
        expect(isDisplayName('',),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: waylandDisplayOf.name,
  children: [
    it({
      name: 'finds the display in the socket line',
      fn: async ({ expect, }) => {
        const lines = ['noise', 'INFO smithay: Created new socket name=Some("wayland-3")', 'later',];
        expect(waylandDisplayOf(lines,),).toBe('wayland-3',);
      },
    },),
    it({
      name: 'accepts several spaces before the name',
      fn: async ({ expect, }) => {
        expect(waylandDisplayOf(['Created new socket   name=Some("wayland-1")',],),).toBe('wayland-1',);
      },
    },),
    it({
      name: 'gives the empty string when no line matches',
      fn: async ({ expect, }) => {
        expect(waylandDisplayOf([],),).toBe('',);
        expect(waylandDisplayOf(['Created new socket name=Some("wayland-1',],),).toBe('',);
        expect(waylandDisplayOf(['Created new socket name=Some("other-1")',],),).toBe('',);
        expect(waylandDisplayOf(['Created new socketname=Some("wayland-1")',],),).toBe('',);
        expect(waylandDisplayOf(['Created new socket name=None',],),).toBe('',);
        expect(waylandDisplayOf(['unrelated',],),).toBe('',);
      },
    },),
    it({
      name: 'skips a non-matching line and takes the next matching one',
      fn: async ({ expect, }) => {
        const lines = ['Created new socket name=None', 'Created new socket name=Some("wayland-2")',];
        expect(waylandDisplayOf(lines,),).toBe('wayland-2',);
      },
    },),
  ],
},);

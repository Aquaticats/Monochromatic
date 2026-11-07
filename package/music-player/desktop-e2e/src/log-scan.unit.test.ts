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
  describeStartStage,
  isDisplayName,
  startStageOf,
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

/**
 Lines of a session log as the compositor and the app write them, from a headless run measured on 2026-11-07.
 */
const COMPOSITOR_FIRST = 'INFO nested_wayland_session::backend_select: selected the render backend choice=BackendChoice';
const SOCKET_LINE = 'INFO nested_wayland_session::control_socket: control socket listening at /s/wide.sock';
const SPAWNED_LINE = 'INFO nested_wayland_session::child: spawned hosted client /app/music-player on "wayland-2"';
const APP_LINE = 'DEBUG music_player::ui_theme: no accent override, following the system accent';
const STATE_LINE = 'INFO music_player::ui_state_log: ui_state {"seq":1}';

await describe({
  name: startStageOf.name,
  children: [
    it({
      name: 'says the compositor wrote nothing for an empty log',
      fn: async ({ expect, }) => {
        expect(startStageOf({ lines: [], },),).toBe('compositor-silent',);
        expect(startStageOf({ lines: ['',], },),).toBe('compositor-silent',);
      },
    },),
    it({
      name: 'walks through the stages in the order a start passes them',
      fn: async ({ expect, }) => {
        expect(startStageOf({ lines: [COMPOSITOR_FIRST,], },),).toBe('compositor-starting',);
        expect(startStageOf({ lines: [COMPOSITOR_FIRST, SOCKET_LINE,], },),).toBe('socket-listening',);
        expect(startStageOf({ lines: [COMPOSITOR_FIRST, SOCKET_LINE, SPAWNED_LINE,], },),).toBe('app-spawned',);
        expect(startStageOf({ lines: [COMPOSITOR_FIRST, SOCKET_LINE, SPAWNED_LINE, APP_LINE,], },),).toBe('app-running',);
        expect(startStageOf({ lines: [COMPOSITOR_FIRST, SOCKET_LINE, SPAWNED_LINE, APP_LINE, STATE_LINE,], },),).toBe(
          'state-line',
        );
      },
    },),
    it({
      name: 'reads the furthest stage reached, whatever order the lines are in',
      fn: async ({ expect, }) => {
        expect(startStageOf({ lines: [STATE_LINE, COMPOSITOR_FIRST,], },),).toBe('state-line',);
      },
    },),
  ],
},);

await describe({
  name: describeStartStage.name,
  children: [
    it({
      name: 'names the last point the start reached in plain words, one text per stage',
      fn: async ({ expect, }) => {
        const stages = [
          'compositor-silent',
          'compositor-starting',
          'socket-listening',
          'app-spawned',
          'app-running',
          'state-line',
        ] as const;
        const texts = stages.map((stage,) => describeStartStage(stage,),);
        expect(new Set(texts,).size,).toBe(stages.length,);
        expect(describeStartStage('app-spawned',),).toContain('app',);
        expect(describeStartStage('compositor-silent',),).toContain('compositor',);
      },
    },),
  ],
},);

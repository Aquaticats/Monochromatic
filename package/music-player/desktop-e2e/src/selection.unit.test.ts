/**
 Tests for the filter: which sessions run for which text, and which checks pass it.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isSelected,
  SESSION_RULES,
  sessionsFor,
} from './selection.ts';

await describe({
  name: sessionsFor.name,
  children: [
    it({
      name: 'runs every session in order for an empty filter',
      fn: async ({ expect, }) => {
        expect(sessionsFor({ only: '', },),).toEqual(['wide', 'narrow', 'dialog', 'auto-play', 'text-size', 'quit', 'states', 'visual',],);
      },
    },),
    it({
      name: 'runs the wide session for the group prefixes of its checks',
      fn: async ({ expect, }) => {
        for (const only of ['start', 'focus/ring-moves', 'seek', 'row/delete', 'badges', 'double-shift', 'audio', 'tooltip', 'menu/opens-at-pointer',])
          expect(sessionsFor({ only, },),).toEqual(['wide',],);
      },
    },),
    it({
      name: 'runs the narrow, quit and visual sessions for their own prefixes',
      fn: async ({ expect, }) => {
        expect(sessionsFor({ only: 'folded/tab-order', },),).toEqual(['narrow',],);
        expect(sessionsFor({ only: 'editor-page', },),).toEqual(['narrow',],);
        expect(sessionsFor({ only: 'click-through/search-narrow', },),).toEqual(['wide', 'narrow',],);
        expect(sessionsFor({ only: 'click-through/search-wide', },),).toEqual(['wide', 'narrow',],);
        expect(sessionsFor({ only: 'quit', },),).toEqual(['quit',],);
        expect(sessionsFor({ only: 'text-size/keys-step-a-size', },),).toEqual(['text-size',],);
        expect(sessionsFor({ only: 'auto-play/on-click-selects-and-plays', },),).toEqual(['auto-play',],);
        expect(sessionsFor({ only: 'states/in-order-stops', },),).toEqual(['states',],);
        expect(sessionsFor({ only: 'visual/dark', },),).toEqual(['visual',],);
      },
    },),
    it({
      name: 'runs the dialog session for its prefix and for its check id, which also brings the wide session',
      fn: async ({ expect, }) => {
        expect(sessionsFor({ only: 'dialog', },),).toEqual(['dialog',],);
        expect(sessionsFor({ only: 'browse/ctrl-shift-o', },),).toEqual(['wide', 'dialog',],);
        expect(sessionsFor({ only: 'browse', },),).toEqual(['wide',],);
      },
    },),
    it({
      name: 'runs no session for a filter nothing starts with',
      fn: async ({ expect, }) => {
        expect(sessionsFor({ only: 'nothing', },),).toEqual([],);
      },
    },),
    it({
      name: 'names every session once',
      fn: async ({ expect, }) => {
        expect(new Set(SESSION_RULES.map((rule,) => rule.name),).size,).toBe(SESSION_RULES.length,);
      },
    },),
  ],
},);

await describe({
  name: isSelected.name,
  children: [
    it({
      name: 'selects everything for an empty filter',
      fn: async ({ expect, }) => {
        expect(isSelected({ only: '', id: 'a/b', group: 'g', },),).toBe(true,);
      },
    },),
    it({
      name: 'selects a check whose id contains the filter',
      fn: async ({ expect, }) => {
        expect(isSelected({ only: 'alt-left', id: 'seek/alt-left', group: 'playback', },),).toBe(true,);
      },
    },),
    it({
      name: 'selects a check whose group contains the filter',
      fn: async ({ expect, }) => {
        expect(isSelected({ only: 'playback', id: 'seek/alt-left', group: 'playback', },),).toBe(true,);
      },
    },),
    it({
      name: 'rejects a check that matches neither',
      fn: async ({ expect, }) => {
        expect(isSelected({ only: 'badges', id: 'seek/alt-left', group: 'playback', },),).toBe(false,);
      },
    },),
  ],
},);

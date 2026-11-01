import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isGhosttyHelixCombination, } from '../dist/final/node/index.mjs';

await describe({
  name: isGhosttyHelixCombination.name,
  children: [
    it({
      name: 'detects Ghostty desktop entry with hx',
      fn: async () => {
        expect(isGhosttyHelixCombination({
          terminalEntryId: 'com.mitchellh.ghostty.desktop',
          editorCommand: ['hx',],
        },),)
          .toBe(true,);
      },
    },),
    it({
      name: 'detects case-insensitive Ghostty with absolute Helix executable',
      fn: async () => {
        expect(isGhosttyHelixCombination({
          terminalEntryId: 'GHOSTTY',
          editorCommand: ['/usr/bin/helix',],
        },),)
          .toBe(true,);
      },
    },),
    it({
      name: 'accepts Ghostty with nano',
      fn: async () => {
        expect(isGhosttyHelixCombination({
          terminalEntryId: 'com.mitchellh.ghostty.desktop',
          editorCommand: ['nano',],
        },),)
          .toBe(false,);
      },
    },),
    it({
      name: 'accepts Helix with another terminal',
      fn: async () => {
        expect(isGhosttyHelixCombination({
          terminalEntryId: 'org.kde.konsole.desktop',
          editorCommand: ['hx',],
        },),)
          .toBe(false,);
      },
    },),
  ],
},);

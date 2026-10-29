/**
 Tests for the window title condition of the states checks.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isCleanedFirstTrackTitle, } from './states-title.ts';

await describe({
  name: isCleanedFirstTrackTitle.name,
  children: [
    it({
      name: 'accepts the title of the first track without its folder or extension',
      fn: async ({ expect, }) => {
        expect(isCleanedFirstTrackTitle('ok One',),).toBe(true,);
      },
    },),
    it({
      name: 'rejects the idle title, which the window shows until the track is playing',
      fn: async ({ expect, }) => {
        expect(isCleanedFirstTrackTitle('ok Music Player',),).toBe(false,);
        expect(isCleanedFirstTrackTitle('ok',),).toBe(false,);
      },
    },),
    it({
      name: 'rejects a title that keeps the extension or the folder',
      fn: async ({ expect, }) => {
        expect(isCleanedFirstTrackTitle('ok Aaa Folder - One.wav',),).toBe(false,);
        expect(isCleanedFirstTrackTitle('ok One.wav',),).toBe(false,);
        expect(isCleanedFirstTrackTitle('ok Aaa Folder One',),).toBe(false,);
      },
    },),
    it({
      name: 'rejects the title of the second track',
      fn: async ({ expect, }) => {
        expect(isCleanedFirstTrackTitle('ok Two',),).toBe(false,);
      },
    },),
  ],
},);

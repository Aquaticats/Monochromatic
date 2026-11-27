/**
 Guards for the two questions a driver asks about a pulled arm log: whether a word begins one of its lines, and which of
 the markers a passing arm prints the log lacks.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hasLineStartingWith,
  missingMarkers,
} from './guest-route-log-lines.ts';

await describe({
  name: '',
  children: [
    describe({
      name: hasLineStartingWith.name,
      children: [
        it({
          name: 'finds the word that begins the first line',
          fn: async () => {
            expect(hasLineStartingWith({ text: 'SKIPPED: no ZFS here\nok', prefix: 'SKIPPED', },),).toBe(true,);
          },
        },),
        it({
          name: 'finds the word that begins a line after a newline',
          fn: async () => {
            expect(hasLineStartingWith({ text: 'ok\nSKIP\nmore', prefix: 'SKIP', },),).toBe(true,);
          },
        },),
        it({
          name: 'finds the word that begins a last line with no newline after it',
          fn: async () => {
            expect(hasLineStartingWith({ text: 'ok\nSKIP', prefix: 'SKIP', },),).toBe(true,);
          },
        },),
        it({
          name: 'does not count a word that only stands inside a line',
          fn: async () => {
            expect(hasLineStartingWith({ text: 'the arm printed SKIPPED once\nok', prefix: 'SKIPPED', },),).toBe(false,);
          },
        },),
        it({
          name: 'does not count the word in the middle of a line that begins with another',
          fn: async () => {
            expect(hasLineStartingWith({ text: 'DOCTOR-GUEST SKIP inside\n', prefix: 'SKIP', },),).toBe(false,);
          },
        },),
        it({
          name: 'does not count an indented word, as a line beginning with a space begins with that space',
          fn: async () => {
            expect(hasLineStartingWith({ text: '  SKIPPED\n', prefix: 'SKIPPED', },),).toBe(false,);
          },
        },),
        it({
          name: 'finds nothing in an empty log',
          fn: async () => {
            expect(hasLineStartingWith({ text: '', prefix: 'SKIPPED', },),).toBe(false,);
          },
        },),
        it({
          name: 'reads a prefix that is longer than the line as no match',
          fn: async () => {
            expect(hasLineStartingWith({ text: 'SKIP\n', prefix: 'SKIPPED', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: missingMarkers.name,
      children: [
        it({
          name: 'answers nothing when the log holds every marker',
          fn: async () => {
            expect(missingMarkers({
              text: 'FORCEDACT phase=path-holder-asked\nFORCEDACT phase=arm-finished\n',
              markers: ['FORCEDACT phase=path-holder-asked', 'FORCEDACT phase=arm-finished',],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'names the markers the log lacks, in the order the driver listed them',
          fn: async () => {
            expect(missingMarkers({
              text: 'FORCEDACT phase=arm-finished\n',
              markers: ['FORCEDACT phase=path-holder-asked', 'FORCEDACT phase=arm-finished', 'FORCEDACT phase=none',],
            },),).toEqual(['FORCEDACT phase=path-holder-asked', 'FORCEDACT phase=none',],);
          },
        },),
        it({
          name: 'names every marker of an empty log',
          fn: async () => {
            expect(missingMarkers({
              text: '',
              markers: ['SLOT-STATE done', 'REPAIR pruned registration=gone',],
            },),).toEqual(['SLOT-STATE done', 'REPAIR pruned registration=gone',],);
          },
        },),
        it({
          name: 'reads a marker inside a longer line as printed, as the drivers have always read it',
          fn: async () => {
            expect(missingMarkers({
              text: '     1  DEFERRED done\n',
              markers: ['DEFERRED done',],
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

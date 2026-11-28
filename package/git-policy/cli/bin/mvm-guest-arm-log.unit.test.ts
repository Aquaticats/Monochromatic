/**
 Guards for the two questions a driver asks about a pulled arm log: whether a word begins one of its lines, which of the
 markers a passing arm prints the log lacks, and the refusal a log that lacks one earns.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  holdsLineStartingWith,
  missingMarkers,
  requireMarkers,
} from './mvm-guest-arm-log.ts';

/**
 The markers a test log is asked for.

 @returns two lines a passing log carries
 */
function markers(): readonly string[] {
  return [
    'ARM started',
    'ARM done',
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: holdsLineStartingWith.name,
      children: [
        it({
          name: 'finds the word that begins the first line',
          fn: async () => {
            expect(holdsLineStartingWith({ text: 'SKIPPED: no ZFS here\nok', word: 'SKIPPED', },),).toBe(true,);
          },
        },),
        it({
          name: 'finds the word that begins a line after a newline',
          fn: async () => {
            expect(holdsLineStartingWith({ text: 'ok\nSKIP\nmore', word: 'SKIP', },),).toBe(true,);
          },
        },),
        it({
          name: 'finds the word that begins a last line with no newline after it',
          fn: async () => {
            expect(holdsLineStartingWith({ text: 'ok\nSKIP', word: 'SKIP', },),).toBe(true,);
          },
        },),
        it({
          name: 'does not count a word that only stands inside a line',
          fn: async () => {
            expect(holdsLineStartingWith({ text: 'the arm printed SKIPPED once\nok', word: 'SKIPPED', },),).toBe(false,);
          },
        },),
        it({
          name: 'does not count the word in the middle of a line that begins with another',
          fn: async () => {
            expect(holdsLineStartingWith({ text: 'DOCTOR-GUEST SKIP inside\n', word: 'SKIP', },),).toBe(false,);
          },
        },),
        it({
          name: 'does not count an indented word, as a line beginning with a space begins with that space',
          fn: async () => {
            expect(holdsLineStartingWith({ text: '  SKIPPED\n', word: 'SKIPPED', },),).toBe(false,);
          },
        },),
        it({
          name: 'finds nothing in an empty log',
          fn: async () => {
            expect(holdsLineStartingWith({ text: '', word: 'SKIPPED', },),).toBe(false,);
          },
        },),
        it({
          name: 'reads a word that is longer than the line as no match',
          fn: async () => {
            expect(holdsLineStartingWith({ text: 'SKIP\n', word: 'SKIPPED', },),).toBe(false,);
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
    describe({
      name: requireMarkers.name,
      children: [
        it({
          name: 'passes a log that carries every marker',
          fn: async () => {
            expect(function checks(): void {
              requireMarkers({
                text: 'ARM started\nARM done\n',
                markers: markers(),
                subject: 'the arm log',
                because: 'the arm did not run',
              },);
            },).not.toThrow();
          },
        },),
        it({
          name: 'names the first marker the log lacks and why a missing marker fails',
          fn: async () => {
            expect(function checks(): void {
              requireMarkers({
                text: 'ARM started\n',
                markers: markers(),
                subject: 'the arm log',
                because: 'the arm did not run',
              },);
            },).toThrow('the arm log lacks ARM done; the arm did not run');
          },
        },),
        it({
          name: 'refuses with the guest drivers own error class, so a log names what refused it',
          fn: async () => {
            expect(function checks(): void {
              requireMarkers({
                text: '',
                markers: markers(),
                subject: 'the pulled lock log',
                because: 'the guest ran the arm without performing it',
              },);
            },).toThrow('the pulled lock log lacks ARM started; the guest ran the arm without performing it');
          },
        },),
      ],
    },),
  ],
},);

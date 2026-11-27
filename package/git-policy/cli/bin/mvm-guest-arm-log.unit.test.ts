import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  holdsWordLine,
  missingMarkerRefusal,
  missingMarkers,
} from './mvm-guest-arm-log.ts';

/**
 The three lines one arm's log holds, in the order the driver asks for them.
 */
const claimed = [
  'LOCK released=yes',
  'LOCK installed-lock-kept-the-registration=true',
  'LOCK done',
] as const;

await describe({
  name: '',
  children: [
    describe({
      name: missingMarkers.name,
      children: [
        it({
          name: 'answers nothing when the log holds every line',
          fn: async () => {
            expect(missingMarkers({
              text: 'work\nLOCK released=yes\nLOCK installed-lock-kept-the-registration=true\nLOCK done\n',
              markers: claimed,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'answers every line the log lacks, in the order they were asked for',
          fn: async () => {
            expect(missingMarkers({
              text: 'work\nLOCK done\n',
              markers: claimed,
            },),).toEqual([
              'LOCK released=yes',
              'LOCK installed-lock-kept-the-registration=true',
            ],);
          },
        },),
        it({
          name: 'finds a marker inside a longer line, which the guest s own grep is what rules out',
          fn: async () => {
            expect(missingMarkers({
              text: 'LOCK done=not-yet\n',
              markers: ['LOCK done',],
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: holdsWordLine.name,
      children: [
        it({
          name: 'reads a word a line begins with',
          fn: async () => {
            expect(holdsWordLine({
              text: 'building\nSKIP the arm did not run\n',
              word: 'SKIP',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'reads a word only in the middle of a line as absent',
          fn: async () => {
            expect(holdsWordLine({
              text: 'the SKIP line was not printed\n',
              word: 'SKIP',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'reads a log with no such line as absent',
          fn: async () => {
            expect(holdsWordLine({
              text: 'building\nDONE=0\n',
              word: 'SKIP',
            },),).toBe(false,);
            expect(holdsWordLine({
              text: '',
              word: 'SKIP',
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: missingMarkerRefusal.name,
      children: [
        it({
          name: 'names the line the log lacked, and says the arm ran without performing it',
          fn: async () => {
            /**
             What the driver refuses with.
             */
            const refusal = missingMarkerRefusal({ marker: 'LOCK done', },);
            expect(refusal.name,).toBe('GuestDriverError',);
            expect(refusal.message,)
              .toBe('the pulled arm log lacks LOCK done; the guest ran the arm without performing it',);
          },
        },),
      ],
    },),
  ],
},);

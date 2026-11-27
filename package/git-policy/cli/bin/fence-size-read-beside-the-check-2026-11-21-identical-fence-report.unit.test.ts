import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type FenceReading,
  fenceOf,
  identicalFenceReport,
} from './fence-size-read-beside-the-check-2026-11-21-identical-fence-report.ts';

/**
 One arm's add as the control read it, with the fence decision and the destination the test gives it.

 @param fence - the line the add printed its fence decision on

 @param route - the route the add's record named, or nothing when it named none

 @returns one arm's reading
 */
function reading({ fence, route, }: { readonly fence: string; readonly route?: string; }): FenceReading {
  return {
    fence,
    status: '',
    ...(route === undefined ? {} : { route, }),
    verbose: [fence,],
    carried: {},
  };
}

/**
 The two files one destination of the fixture holds, as path and bytes.

 @param bytes - the second file's bytes, which is what a differing destination changes

 @returns one destination's entries
 */
function destination({ bytes, }: { readonly bytes: string; }): readonly (readonly [string, string])[] {
  return [
    ['.gitattributes', 'Ki50eHQgdGV4dCBlb2w9bGY=',],
    ['plain.txt', bytes,],
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: fenceOf.name,
      children: [
        it({
          name: 'read the decision from the marker onwards, without what the add printed before it',
          fn: async () => {
            expect(fenceOf({ line: 'some notice: the fence carries 40 paths itself', },),).toBe('the fence carries 40 paths itself',);
          },
        }),
        it({
          name: 'read the whole line when it carries no decision, which is what an arm that printed none says',
          fn: async () => {
            expect(fenceOf({ line: '  no fence line  ', },),).toBe('no fence line',);
          },
        }),
      ],
    },),
    describe({
      name: identicalFenceReport.name,
      children: [
        it({
          name: 'answer that the two arms agreed when their decisions and their destinations are the same',
          fn: async () => {
            expect(identicalFenceReport({
              serial: reading({ fence: 'the fence carries 40 paths itself', route: 'clone', },),
              overlap: reading({ fence: 'notice: the fence carries 40 paths itself', route: 'clone', },),
              serialTree: destination({ bytes: 'b25l', },),
              overlapTree: destination({ bytes: 'b25l', },),
            },),).toEqual({
              serial: {
                fence: 'the fence carries 40 paths itself',
                status: '',
                route: 'clone',
                verbose: ['the fence carries 40 paths itself',],
              },
              overlap: {
                fence: 'the fence carries 40 paths itself',
                status: '',
                route: 'clone',
                verbose: ['notice: the fence carries 40 paths itself',],
              },
              sameTrees: true,
              files: 2,
              sameDecision: true,
            },);
          },
        }),
        it({
          name: 'answer that the arms differ when one decision differs',
          fn: async () => {
            expect(identicalFenceReport({
              serial: reading({ fence: 'the fence carries 40 paths itself', },),
              overlap: reading({ fence: 'the fence carries nothing', },),
              serialTree: destination({ bytes: 'b25l', },),
              overlapTree: destination({ bytes: 'b25l', },),
            },).sameDecision,).toBe(false,);
          },
        }),
        it({
          name: 'answer that the destinations differ when one byte of one file differs',
          fn: async () => {
            expect(identicalFenceReport({
              serial: reading({ fence: 'the fence carries 40 paths itself', },),
              overlap: reading({ fence: 'the fence carries 40 paths itself', },),
              serialTree: destination({ bytes: 'b25l', },),
              overlapTree: destination({ bytes: 'dHdv', },),
            },),).toMatchObject({
              sameTrees: false,
              sameDecision: true,
            },);
          },
        }),
        it({
          name: 'write the route an arm took not at all as null, which is what the control printed',
          fn: async () => {
            expect(identicalFenceReport({
              serial: reading({ fence: 'the fence carries 40 paths itself', },),
              overlap: reading({ fence: 'the fence carries 40 paths itself', },),
              serialTree: destination({ bytes: 'b25l', },),
              overlapTree: destination({ bytes: 'b25l', },),
            },).serial,).toEqual({
              fence: 'the fence carries 40 paths itself',
              status: '',
              route: null,
              verbose: ['the fence carries 40 paths itself',],
            },);
          },
        }),
      ],
    },),
  ],
},);

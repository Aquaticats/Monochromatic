/**
 How much room the volume that holds the repository has, and the floor a new run needs before it starts. The reading here is
 handed a volume's answer by the test, so no test depends on how full this machine's disk is.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BYTES_IN_A_GIBIBYTE,
  type FreeSpace,
  FREE_SPACE_FLOOR_BYTES,
  freeSpaceFloorTextOf,
  freeSpaceIsUnderTheFloor,
  freeSpaceOnVolume,
  freeSpaceTextOf,
  wholeGibibytesOf,
} from '../dist/final/node/index.mjs';

/**
 The volume every room in this file is on.

 @returns the volume's path

 @example
 ```ts
 const volume = aVolumePath();
 ```
 */
function aVolumePath(): string {
  return '/a-volume-of-the-test';
}

await describe({
  name: '',
  children: [
    describe({
      name: freeSpaceOnVolume.name,
      children: [
        it({
          name: 'asks the volume for its room once and counts it in bytes, keeping the volume the room is on',
          fn: async () => {
            /**
             The paths the reading was asked about.
             */
            const asked: string[] = [];
            /**
             The room a volume answers with, as many blocks of its own size as the test says.
             */
            const freeBlocks = 7;
            /**
             The room the reading answers with.
             */
            const freeSpace: FreeSpace = freeSpaceOnVolume({
              volume: aVolumePath(),
              statfs: function answersAboutRoom(path,) {
                asked.push(path,);
                return {
                  bavail: freeBlocks,
                  bsize: BYTES_IN_A_GIBIBYTE,
                };
              },
            },);
            expect(asked,).toEqual([aVolumePath(),],);
            expect(freeSpace,).toEqual({
              volume: aVolumePath(),
              freeBytes: freeBlocks * BYTES_IN_A_GIBIBYTE,
            },);
          },
        },),
      ],
    },),
    describe({
      name: wholeGibibytesOf.name,
      children: [
        it({
          name: 'answers the whole gibibytes of a room, dropping the part of one that is left',
          fn: async () => {
            /**
             How many whole gibibytes the room this test measures holds.
             */
            const gibibytes = 3;
            expect(wholeGibibytesOf(gibibytes * BYTES_IN_A_GIBIBYTE,),).toBe(gibibytes,);
            expect(wholeGibibytesOf((gibibytes * BYTES_IN_A_GIBIBYTE) + (BYTES_IN_A_GIBIBYTE - 1),),).toBe(gibibytes,);
          },
        },),
      ],
    },),
    describe({
      name: freeSpaceTextOf.name,
      children: [
        it({
          name: 'says the room in whole gibibytes, naming the volume it is on',
          fn: async () => {
            expect(freeSpaceTextOf({
              volume: aVolumePath(),
              freeBytes: FREE_SPACE_FLOOR_BYTES,
            },),).toBe(`${String(wholeGibibytesOf(FREE_SPACE_FLOOR_BYTES,),)} GiB free on ${aVolumePath()}`,);
          },
        },),
      ],
    },),
    describe({
      name: freeSpaceFloorTextOf.name,
      children: [
        it({
          name: 'says the floor the constant holds, in the same whole gibibytes the counts line does',
          fn: async () => {
            expect(freeSpaceFloorTextOf(),).toBe(`${String(wholeGibibytesOf(FREE_SPACE_FLOOR_BYTES,),)} GiB`,);
          },
        },),
      ],
    },),
    describe({
      name: freeSpaceIsUnderTheFloor.name,
      children: [
        it({
          name: 'says a room is under the floor only when it is smaller than the floor, so a room exactly at it still starts a run',
          fn: async () => {
            expect(freeSpaceIsUnderTheFloor({
              volume: aVolumePath(),
              freeBytes: FREE_SPACE_FLOOR_BYTES - 1,
            },),).toBe(true,);
            expect(freeSpaceIsUnderTheFloor({
              volume: aVolumePath(),
              freeBytes: FREE_SPACE_FLOOR_BYTES,
            },),).toBe(false,);
            expect(freeSpaceIsUnderTheFloor({
              volume: aVolumePath(),
              freeBytes: FREE_SPACE_FLOOR_BYTES + 1,
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

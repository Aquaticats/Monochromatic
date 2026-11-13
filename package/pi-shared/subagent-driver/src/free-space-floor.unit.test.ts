/**
 How much room each of the volumes a new run cannot work without has, the floor a new run needs on each, and how two needs on one
 volume are kept once at the strictest floor. Every room here is handed to the reading by the test, so no test depends on how full
 this machine's disk is.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BUILD_VOLUME,
  BUILD_VOLUME_FLOOR_BYTES,
  BYTES_IN_A_GIBIBYTE,
  type FreeSpace,
  type FreeSpaceAgainstItsFloor,
  RUNS_FOLDER_FLOOR_BYTES,
  SESSION_FILES_FLOOR_BYTES,
  type VolumeAgainstItsFloor,
  freeSpaceFloorTextOf,
  freeSpaceIsUnderTheFloor,
  freeSpaceOnVolume,
  freeSpaceTextOf,
  freeSpaceUnderTheFloorTextOf,
  strictestFloorPerVolume,
  wholeGibibytesOf,
} from '../dist/final/node/index.mjs';

/**
 The volume every room in the parts of this file that ask about one volume is on.

 @returns the volume's path

 @example
 ```ts
 const volume = aVolumePath();
 ```
 */
function aVolumePath(): string {
  return '/a-volume-of-the-test';
}

/**
 A volume's answer about its room, in whole gibibytes.

 @param gibibytes - how many gibibytes the volume has free

 @returns the room a `statfs` answers with

 @example
 ```ts
 const room = roomOfGibibytes(7);
 ```
 */
function roomOfGibibytes(gibibytes: number,): {
  readonly bavail: number;
  readonly bsize: number;
} {
  return {
    bavail: gibibytes,
    bsize: BYTES_IN_A_GIBIBYTE,
  };
}

/**
 Room on a volume, with the floor a new run needs on it, as the refusal asks about one volume.

 @param freeBytes - the room the volume has

 @param floorBytes - the floor a new run needs on that volume

 @returns the room and the floor

 @example
 ```ts
 const room = roomAgainstItsFloor(SESSION_FILES_FLOOR_BYTES - 1, SESSION_FILES_FLOOR_BYTES);
 ```
 */
function roomAgainstItsFloor(freeBytes: number, floorBytes: number,): FreeSpaceAgainstItsFloor {
  return {
    space: {
      volume: aVolumePath(),
      freeBytes,
    },
    floorBytes,
  };
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
                return roomOfGibibytes(freeBlocks,);
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
      name: strictestFloorPerVolume.name,
      children: [
        it({
          name: 'keeps each distinct volume once, in the order its needs were named, with the strictest floor set for it',
          fn: async () => {
            /**
             The volume of the repository and the builds, whose floor is the strictest of the three.
             */
            const buildVolume = '/a-build-volume-of-the-test';
            /**
             The volume that holds Pi\'s session files.
             */
            const sessionVolume = '/a-session-volume-of-the-test';
            /**
             One entry per need, two of them on the build volume at different floors.
             */
            const volumes: readonly VolumeAgainstItsFloor[] = [
              { volume: buildVolume, floorBytes: BUILD_VOLUME_FLOOR_BYTES, },
              { volume: sessionVolume, floorBytes: SESSION_FILES_FLOOR_BYTES, },
              { volume: buildVolume, floorBytes: RUNS_FOLDER_FLOOR_BYTES, },
            ];
            expect(strictestFloorPerVolume({ volumes, },),).toEqual([
              { volume: buildVolume, floorBytes: BUILD_VOLUME_FLOOR_BYTES, },
              { volume: sessionVolume, floorBytes: SESSION_FILES_FLOOR_BYTES, },
            ],);
          },
        },),
        it({
          name: 'takes the later floor when it is the stricter one, so the order the needs are named in decides nothing',
          fn: async () => {
            /**
             A floor the test writes down, which is stricter than the one named before it.
             */
            const strictest = BUILD_VOLUME_FLOOR_BYTES + BYTES_IN_A_GIBIBYTE;
            expect(strictestFloorPerVolume({
              volumes: [
                { volume: aVolumePath(), floorBytes: RUNS_FOLDER_FLOOR_BYTES, },
                { volume: aVolumePath(), floorBytes: strictest, },
              ],
            },),).toEqual([{ volume: aVolumePath(), floorBytes: strictest, },],);
          },
        },),
        it({
          name: 'answers every volume as it was given when no two needs share one',
          fn: async () => {
            /**
             One need per volume, all of them distinct.
             */
            const volumes: readonly VolumeAgainstItsFloor[] = [
              { volume: BUILD_VOLUME, floorBytes: BUILD_VOLUME_FLOOR_BYTES, },
              { volume: '/a-runs-volume-of-the-test', floorBytes: RUNS_FOLDER_FLOOR_BYTES, },
              { volume: '/a-session-volume-of-the-test', floorBytes: SESSION_FILES_FLOOR_BYTES, },
            ];
            expect(strictestFloorPerVolume({ volumes, },),).toEqual(volumes,);
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
              freeBytes: BUILD_VOLUME_FLOOR_BYTES,
            },),).toBe(`${String(wholeGibibytesOf(BUILD_VOLUME_FLOOR_BYTES,),)} GiB free on ${aVolumePath()}`,);
          },
        },),
      ],
    },),
    describe({
      name: freeSpaceFloorTextOf.name,
      children: [
        it({
          name: 'says the floor it is given, in the same whole gibibytes the counts line does, for each volume a new run fills',
          fn: async () => {
            /**
             How many whole gibibytes a floor the test writes down holds, which is no shipped floor.
             */
            const gibibytes = 3;
            /**
             That floor, in bytes, with the part of a gibibyte a room may hold left over.
             */
            const floorBytes = (gibibytes * BYTES_IN_A_GIBIBYTE) + 1;
            expect(freeSpaceFloorTextOf(floorBytes,),).toBe(`${String(gibibytes,)} GiB`,);
            for (const floor of [BUILD_VOLUME_FLOOR_BYTES, RUNS_FOLDER_FLOOR_BYTES, SESSION_FILES_FLOOR_BYTES,]) {
              expect(freeSpaceFloorTextOf(floor,),).toBe(`${String(wholeGibibytesOf(floor,),)} GiB`,);
            }
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
            expect(freeSpaceIsUnderTheFloor(
              roomAgainstItsFloor(SESSION_FILES_FLOOR_BYTES - 1, SESSION_FILES_FLOOR_BYTES,),
            ),).toBe(true,);
            expect(freeSpaceIsUnderTheFloor(
              roomAgainstItsFloor(SESSION_FILES_FLOOR_BYTES, SESSION_FILES_FLOOR_BYTES,),
            ),).toBe(false,);
            expect(freeSpaceIsUnderTheFloor(
              roomAgainstItsFloor(SESSION_FILES_FLOOR_BYTES + 1, SESSION_FILES_FLOOR_BYTES,),
            ),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: freeSpaceUnderTheFloorTextOf.name,
      children: [
        it({
          name: 'says the room, the volume and the floor, so a refusal names the drive to free and how far under it is',
          fn: async () => {
            /**
             A room on a volume under its floor.
             */
            const room = roomAgainstItsFloor(SESSION_FILES_FLOOR_BYTES - 1, SESSION_FILES_FLOOR_BYTES,);
            expect(freeSpaceUnderTheFloorTextOf(room,),).toBe(
              `${freeSpaceTextOf(room.space,)}, under the floor of ${freeSpaceFloorTextOf(room.floorBytes,)}`,
            );
          },
        },),
      ],
    },),
  ],
},);

/**
 The volume a folder is on, found from the folder itself: its real path with every symbolic link resolved, walked up to the folder
 whose parent is on another device. Every folder here is answered by the test, so no test depends on this machine's links or on
 which drive holds what.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type AnswerAboutAFolder,
  BYTES_IN_A_GIBIBYTE,
  freeSpaceOfTheFolder,
  volumeOfFolder,
} from '../dist/final/node/index.mjs';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 What one folder of the test's file system answers about itself: the path it really is, and the device whose volume holds it.
 */
type FolderAnswer = {
  readonly realPath: string;
  readonly device: number;
};

/**
 A folder of the test's file system, as the map of a file system names it.

 @param folder - the folder's path, which is the path it really is unless the test says otherwise

 @param device - the number of the device whose volume holds the folder, so two folders on one volume share it

 @param realPath - the path the folder really is, when the test reaches it through a symbolic link

 @returns the entry the file system's map holds

 @example
 ```ts
 const entry = folderOn({ folder: '/var/home', device: 2, });
 ```
 */
function folderOn({
  folder,
  device,
  realPath = folder,
}: {
  readonly folder: string;
  readonly device: number;
  readonly realPath?: string;
}): readonly [string, FolderAnswer] {
  return [
    folder,
    {
      realPath,
      device,
    },
  ];
}

/**
 Reads a folder's answer out of the test's file system, the way the command asks the machine's own folders.

 @param folders - every folder of the test's file system, by its path

 @returns what a folder answers, and a refusal of the code the machine gives for a folder that is not there

 @example
 ```ts
 const answerAbout = answerAboutIn({ folders: new Map([folderOn({ folder: '/', device: 1 })]), });
 ```
 */
function answerAboutIn({ folders, }: { readonly folders: ReadonlyMap<string, FolderAnswer> },): AnswerAboutAFolder {
  return function answerAbout(folder,) {
    /**
     What the test's file system holds under this path.
     */
    const found = folders.get(folder,);
    if (found === undefined) {
      throw Object.assign(
        new Error(`the test's file system holds no folder ${folder}`,),
        { code: MISSING_PATH_CODE, },
      );
    }
    return found;
  };
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

await describe({
  name: '',
  children: [
    describe({
      name: volumeOfFolder.name,
      children: [
        it({
          name: 'names the volume a folder is on by that volume\'s own path rather than by the folder inside it',
          fn: async () => {
            /**
             The volume the folder is on, which is the folder whose parent is on another device.
             */
            const volume = '/var/home';
            expect(volumeOfFolder({
              folder: `${volume}/user/temp/agent/pi-runs`,
              answerAbout: answerAboutIn({
                folders: new Map([
                  folderOn({ folder: '/', device: 1, }),
                  folderOn({ folder: '/var', device: 1, }),
                  folderOn({ folder: volume, device: 2, }),
                  folderOn({ folder: `${volume}/user`, device: 2, }),
                  folderOn({ folder: `${volume}/user/temp`, device: 2, }),
                  folderOn({ folder: `${volume}/user/temp/agent`, device: 2, }),
                  folderOn({ folder: `${volume}/user/temp/agent/pi-runs`, device: 2, }),
                ],),
              },),
            },),).toBe(volume,);
          },
        },),
        it({
          name: 'reads the nearest folder above one that has not been made yet, because that folder is on the same volume',
          fn: async () => {
            /**
             The volume the newest folder that is there sits on.
             */
            const volume = '/a-volume-of-the-test';
            expect(volumeOfFolder({
              folder: `${volume}/agent/temp/pi-runs`,
              answerAbout: answerAboutIn({
                folders: new Map([
                  folderOn({ folder: '/', device: 1, }),
                  folderOn({ folder: volume, device: 2, }),
                  folderOn({ folder: `${volume}/agent`, device: 2, }),
                  folderOn({ folder: `${volume}/agent/temp`, device: 2, }),
                ],),
              },),
            },),).toBe(volume,);
          },
        },),
        it({
          name: 'names the folder it really is when the folder is reached through a symbolic link, so the volume named is the drive that fills',
          fn: async () => {
            /**
             The path the caller was given, whose `temp` is a symbolic link to another drive.
             */
            const throughTheLink = '/home/a/temp/agent/pi-runs';
            /**
             The drive the link points at, which is the volume the folder's writes fill.
             */
            const really = '/mnt/encrypted/temp/agent/pi-runs';
            expect(volumeOfFolder({
              folder: throughTheLink,
              answerAbout: answerAboutIn({
                folders: new Map([
                  folderOn({ folder: '/', device: 1, }),
                  folderOn({ folder: '/home', device: 1, }),
                  folderOn({ folder: '/home/a', device: 1, }),
                  folderOn({
                    folder: throughTheLink,
                    device: 3,
                    realPath: really,
                  },),
                  folderOn({ folder: '/mnt', device: 1, }),
                  folderOn({ folder: '/mnt/encrypted', device: 3, }),
                  folderOn({ folder: '/mnt/encrypted/temp', device: 3, }),
                  folderOn({ folder: '/mnt/encrypted/temp/agent', device: 3, }),
                ],),
              },),
            },),).toBe('/mnt/encrypted',);
          },
        },),
        it({
          name: 'names the root itself as the volume when every folder above a folder is on one device, which is a machine with no other mount',
          fn: async () => {
            expect(volumeOfFolder({
              folder: '/runs/pi-runs',
              answerAbout: answerAboutIn({
                folders: new Map([
                  folderOn({ folder: '/', device: 1, }),
                  folderOn({ folder: '/runs', device: 1, }),
                  folderOn({ folder: '/runs/pi-runs', device: 1, }),
                ],),
              },),
            },),).toBe('/',);
          },
        },),
        it({
          name: 'answers a folder it cannot place at all, which is one no folder above it on the way to the root is there',
          fn: async () => {
            try {
              volumeOfFolder({
                folder: '/nowhere/at/all',
                answerAbout: answerAboutIn({ folders: new Map(), },),
              },);
            }
            catch (error) {
              expect((error as { readonly code?: string }).code,).toBe(MISSING_PATH_CODE,);
              return;
            }
            throw new Error('a folder with no folder above it answered instead of refusing',);
          },
        },),
        it({
          name: 'answers a folder whose read failed for a reason other than not being there, rather than walking above it',
          fn: async () => {
            /**
             The folder whose read the machine refused.
             */
            const folder = '/a-folder-of-the-test/inside';
            /**
             The error the machine gave for that read.
             */
            const refused = Object.assign(
              new Error('the machine refused this read',),
              { code: 'EACCES', },
            );
            /**
             What the folders above it answer, which a walk must never reach.
             */
            const above = answerAboutIn({
              folders: new Map([
                folderOn({ folder: '/', device: 1, }),
                folderOn({ folder: '/a-folder-of-the-test', device: 1, }),
              ],),
            },);
            try {
              volumeOfFolder({
                folder,
                answerAbout: function answerAbout(asked,) {
                  if (asked === folder) {
                    throw refused;
                  }
                  return above(asked,);
                },
              },);
            }
            catch (error) {
              expect(error,).toBe(refused,);
              return;
            }
            throw new Error('a folder the machine refused was walked above instead of answered',);
          },
        },),
      ],
    },),
    describe({
      name: freeSpaceOfTheFolder.name,
      children: [
        it({
          name: 'reads the room of the volume a folder is on once, and names that volume',
          fn: async () => {
            /**
             The folders the reading asked about.
             */
            const asked: string[] = [];
            /**
             The room the volume has, as the test hands it over.
             */
            const room = roomOfGibibytes(7,);
            /**
             The volume the folder is on.
             */
            const volume = '/var/home';
            /**
             What the reading answered for a folder several levels into that volume.
             */
            const freeSpace = freeSpaceOfTheFolder({
              folder: `${volume}/user/temp/agent/pi-runs`,
              answerAbout: answerAboutIn({
                folders: new Map([
                  folderOn({ folder: '/', device: 1, }),
                  folderOn({ folder: '/var', device: 1, }),
                  folderOn({ folder: volume, device: 2, }),
                  folderOn({ folder: `${volume}/user`, device: 2, }),
                  folderOn({ folder: `${volume}/user/temp`, device: 2, }),
                  folderOn({ folder: `${volume}/user/temp/agent`, device: 2, }),
                  folderOn({ folder: `${volume}/user/temp/agent/pi-runs`, device: 2, }),
                ],),
              },),
              statfs: function answersAboutRoom(path,) {
                asked.push(path,);
                return room;
              },
            },);
            expect(freeSpace,).toEqual({
              volume,
              freeBytes: room.bavail * room.bsize,
            },);
            expect(asked,).toEqual([volume,],);
          },
        },),
      ],
    },),
  ],
},);

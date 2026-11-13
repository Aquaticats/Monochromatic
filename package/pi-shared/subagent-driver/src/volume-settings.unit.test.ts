/**
 The two settings a caller gives instead of letting the gate read this machine's disks and links: each volume's room, and which
 volume each folder is on. Nothing here reads a disk or a link of this machine.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BYTES_IN_A_GIBIBYTE,
  FOLDER_VOLUMES_VARIABLE,
  VOLUME_FREE_SPACE_BYTES_VARIABLE,
  VolumeSettingsError,
  givenFolderVolumes,
  givenVolumeFreeSpaceBytes,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: givenVolumeFreeSpaceBytes.name,
      children: [
        it({
          name: 'reads the room each volume was given, by the volume\'s path, and nothing at all when the caller set none',
          fn: async () => {
            expect(givenVolumeFreeSpaceBytes({ listed: '', },),).toEqual(new Map(),);
            /**
             The room the test gives the first volume.
             */
            const firstRoom = BYTES_IN_A_GIBIBYTE;
            /**
             The room the test gives the second volume, written as a caller would with spaces around a pair.
             */
            const secondRoom = BYTES_IN_A_GIBIBYTE * 2;
            expect(givenVolumeFreeSpaceBytes({
              listed: ` /a-volume=${String(firstRoom,)},/another-volume=${String(secondRoom,)} , `,
            },),).toEqual(new Map([
              ['/a-volume', firstRoom,],
              ['/another-volume', secondRoom,],
            ],),);
          },
        },),
        it({
          name: 'refuses a pair that gives a volume no whole number of bytes, naming the variable, the volume and the value',
          fn: async () => {
            /**
             A pair whose room is no whole number of bytes, as a mistyped setting gives it.
             */
            const listed = '/a-volume=half';
            try {
              givenVolumeFreeSpaceBytes({ listed, },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(VolumeSettingsError,);
              expect(String(error,),).toContain(VOLUME_FREE_SPACE_BYTES_VARIABLE,);
              expect(String(error,),).toContain(listed,);
              return;
            }
            throw new Error('a room that is no whole number of bytes was read instead of refused',);
          },
        },),
        it({
          name: 'refuses a pair that is no <volume>=<bytes> at all, rather than dropping it and reading the machine\'s own disk',
          fn: async () => {
            for (const listed of ['/a-volume', '=1024', '/a-volume=',]) {
              try {
                givenVolumeFreeSpaceBytes({ listed, },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(VolumeSettingsError,);
                expect(String(error,),).toContain(VOLUME_FREE_SPACE_BYTES_VARIABLE,);
                continue;
              }
              throw new Error(`the setting read "${listed}" instead of refusing it`,);
            }
          },
        },),
      ],
    },),
    describe({
      name: givenFolderVolumes.name,
      children: [
        it({
          name: 'reads the volume each folder was given, by the folder\'s path, and nothing at all when the caller set none',
          fn: async () => {
            expect(givenFolderVolumes({ listed: '', },),).toEqual(new Map(),);
            expect(givenFolderVolumes({
              listed: '/home/a/temp/agent/pi-runs=/mnt/encrypted,/home/a/.pi/agent/sessions=/var/home',
            },),).toEqual(new Map([
              ['/home/a/temp/agent/pi-runs', '/mnt/encrypted',],
              ['/home/a/.pi/agent/sessions', '/var/home',],
            ],),);
          },
        },),
        it({
          name: 'refuses a pair that is no <folder>=<volume>, naming the variable and the pair',
          fn: async () => {
            /**
             A pair with no volume in it.
             */
            const listed = '/home/a/temp/agent/pi-runs';
            try {
              givenFolderVolumes({ listed, },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(VolumeSettingsError,);
              expect(String(error,),).toContain(FOLDER_VOLUMES_VARIABLE,);
              expect(String(error,),).toContain(listed,);
              return;
            }
            throw new Error('a folder with no volume in its pair was read instead of refused',);
          },
        },),
      ],
    },),
  ],
},);

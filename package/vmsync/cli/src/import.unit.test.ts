import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { nameFromPath, } from './import.ts';

//region nameFromPath: derives VM names from image file paths

await describe({
  name: nameFromPath.name,
  children: [
    it({
      name: 'strips file extension',
      fn: async () => {
        expect(nameFromPath('/tmp/alpine.qcow2',),).toBe('alpine',);
      },
    },),

    it({
      name: 'strips only the last extension',
      fn: async () => {
        expect(nameFromPath('/tmp/disk.backup.qcow2',),).toBe('disk-backup',);
      },
    },),

    it({
      name: 'replaces dots with hyphens',
      fn: async () => {
        expect(nameFromPath('/tmp/alpine-3.21-cloud.qcow2',),).toBe('alpine-3-21-cloud',);
      },
    },),

    it({
      name: 'uses only the basename, not directory parts',
      fn: async () => {
        expect(nameFromPath('/home/user/images/fedora.raw',),).toBe('fedora',);
      },
    },),

    it({
      name: 'replaces spaces with hyphens',
      fn: async () => {
        expect(nameFromPath('/tmp/my disk.qcow2',),).toBe('my-disk',);
      },
    },),

    it({
      name: 'preserves hyphens and underscores',
      fn: async () => {
        expect(nameFromPath('/tmp/my-vm_01.qcow2',),).toBe('my-vm_01',);
      },
    },),

    it({
      name: 'handles vhdx extension',
      fn: async () => {
        expect(nameFromPath('/data/windows.vhdx',),).toBe('windows',);
      },
    },),

    it({
      name: 'handles raw extension',
      fn: async () => {
        expect(nameFromPath('/data/disk.raw',),).toBe('disk',);
      },
    },),

    it({
      name: 'replaces consecutive special chars with hyphens',
      fn: async () => {
        expect(nameFromPath('/tmp/a@b#c.qcow2',),).toBe('a-b-c',);
      },
    },),
  ],
},);

//endregion nameFromPath

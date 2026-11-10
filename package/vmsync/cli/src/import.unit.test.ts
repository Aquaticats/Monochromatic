import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nameFromPath, } from '../dist/final/node/testing.mjs';

/**
 First printable ASCII code point, `!`; below it lie control characters only.
 */
const FIRST_PRINTABLE_CODE_POINT = 0x21;

/**
 Last printable ASCII code point, `~`.
 */
const LAST_PRINTABLE_CODE_POINT = 0x7E;

/**
 Every printable ASCII character, in code-point order.
 */
const PRINTABLE_ASCII: readonly string[] = Array.from(
  { length: (LAST_PRINTABLE_CODE_POINT - FIRST_PRINTABLE_CODE_POINT) + 1, },
  function codePointAt(_, offset,) {
    return String.fromCodePoint(FIRST_PRINTABLE_CODE_POINT + offset,);
  },
);

/**
 The VM-name character contract: lowercase and uppercase ASCII letters,
 digits, underscore, and hyphen. Written out here so a divergence on the
 sanitising side fails against an expectation independent of
 `validateName` and `isVmNameChar`.
 */
const ALLOWED_NAME_CHARACTERS: ReadonlySet<string> = new Set(
  Array.from('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-',),
);

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

    it({
      name: 'preserves exactly the characters the name contract allows',
      fn: async () => {
        for (const c of PRINTABLE_ASCII) {
          // A slash is a path separator, so it never reaches the basename.
          if (c === '/')
            continue;
          const expected = ALLOWED_NAME_CHARACTERS.has(c,)
            ? `x${c}`
            : 'x-';
          expect(nameFromPath(`/tmp/x${c}.img`,),).toBe(expected,);
        }
      },
    },),
  ],
},);

//endregion nameFromPath

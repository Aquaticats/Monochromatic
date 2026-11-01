/**
 Tests for `keysToEvdev`, the combo-to-evdev translation that is the injection
 security boundary: known combos map to the exact press/release token order,
 and any unknown or hostile token throws instead of reaching ydotool.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { UnknownKeyError } from './errors.ts';
import { keysToEvdev } from './keys.ts';

await describe({
  name: keysToEvdev.name,
  children: [
    it({
      name: 'presses left-to-right then releases right-to-left',
      fn: async () => {
        expect(keysToEvdev('ctrl+w')).toEqual(['29:1', '17:1', '17:0', '29:0']);
      },
    }),
    it({
      name: 'handles a single key',
      fn: async () => {
        expect(keysToEvdev('w')).toEqual(['17:1', '17:0']);
      },
    }),
    it({
      name: 'is case-insensitive',
      fn: async () => {
        expect(keysToEvdev('CTRL+W')).toEqual(keysToEvdev('ctrl+w'));
      },
    }),
    it({
      name: 'nests releases for a three-key combo',
      fn: async () => {
        expect(keysToEvdev('ctrl+shift+t')).toEqual([
          '29:1',
          '42:1',
          '20:1',
          '20:0',
          '42:0',
          '29:0',
        ]);
      },
    }),
    it({
      name: 'throws UnknownKeyError on an unmapped token',
      fn: async () => {
        expect(() => keysToEvdev('ctrl+hyper')).toThrow(UnknownKeyError);
      },
    }),
    it({
      name: 'throws on an empty combo',
      fn: async () => {
        expect(() => keysToEvdev('')).toThrow(UnknownKeyError);
      },
    }),
    it({
      name: 'rejects a shell-injection-shaped token instead of forwarding it',
      fn: async () => {
        expect(() => keysToEvdev('ctrl; rm -rf /')).toThrow(UnknownKeyError);
      },
    }),
  ],
});

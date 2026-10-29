/**
 Key-combo to evdev translation and ydotool injection.
 
 `keysToEvdev` is a pure translation from a `+`-joined combo (e.g. `ctrl+w`)
 into the press/release evdev code sequence ydotool expects; `sendKeys` runs
 it through the `ydotool` CLI. The translation is the security boundary: it
 only ever emits numeric `code:value` tokens, so an unknown or hostile token
 throws instead of reaching the shell as anything but a rejected key name.
 
 @module
 */

import { execFile } from 'node:child_process';
import { once } from 'node:events';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { UnknownKeyError } from './errors.ts';

/**
 Lowercased key token to Linux evdev key code.
 
 Only the keys the KWin script actually injects are listed; extend this table
 (and rebuild) to support new injections. Numeric values are evdev codes from
 `linux/input-event-codes.h`.
 
 @example
 ```ts
 EVDEV_KEYS.ctrl // 29
 ```
 */
export const EVDEV_KEYS: Record<string, number> = {
  ctrl: 29,
  shift: 42,
  alt: 56,
  meta: 125,
  a: 30,
  b: 48,
  c: 46,
  d: 32,
  e: 18,
  f: 33,
  g: 34,
  h: 35,
  i: 23,
  j: 36,
  k: 37,
  l: 38,
  m: 50,
  n: 49,
  o: 24,
  p: 25,
  q: 16,
  r: 19,
  s: 31,
  t: 20,
  u: 22,
  v: 47,
  w: 17,
  x: 45,
  y: 21,
  z: 44,
  f4: 62,
  f5: 63,
  tab: 15,
  enter: 28,
  esc: 1,
  space: 57,
  backspace: 14,
  delete: 111,
};

/**
 Translate a `+`-joined key combo into the ydotool press-then-release token
 sequence: every key is pressed left-to-right, then released right-to-left.
 
 @param combo - Case-insensitive combo such as `ctrl+w` or `ctrl+shift+t`
 
 @returns ydotool `key` arguments, e.g. `['29:1','17:1','17:0','29:0']`
 
 @throws {@link UnknownKeyError} when a token has no {@link EVDEV_KEYS} entry
 
 @example
 ```ts
 keysToEvdev('ctrl+w'); // ['29:1','17:1','17:0','29:0']
 ```
 */
export function keysToEvdev(combo: string): readonly string[] {
  /**
   Lowercased tokens split on `+`, in press order.
   */
  const parts = combo.toLowerCase()
    .split('+');
  /**
   Evdev codes for each token; throws on any unmapped token.
   */
  const codes = parts.map(function toCode(part: string): number {
    /**
     Evdev code for one token, or `undefined` when unmapped.
     */
    const code = EVDEV_KEYS[part];
    if (code === undefined) {
      throw new UnknownKeyError(part);
    }
    return code;
  });
  /**
   Press events (`code:1`) in left-to-right order.
   */
  const presses = codes.map(function toPress(code: number): string {
    return `${code}:1`;
  });
  /**
   Release events (`code:0`) in reverse order, so keys release inside-out.
   */
  const releases = codes.toReversed()
    .map(function toRelease(code: number): string {
      return `${code}:0`;
    });
  return [
    ...presses,
    ...releases
  ];
}

/**
 Inject a key combo by shelling out to `ydotool key`, logging (never throwing)
 on translation or process failure so a bad combo cannot crash the daemon.
 
 @param keys - Combo string forwarded to {@link keysToEvdev}
 
 @example
 ```ts
 sendKeys('ctrl+w');
 ```
 */
export async function sendKeys(keys: string): Promise<void> {
  try {
    /**
     Press/release token sequence for the requested combo.
     */
    const evdev = keysToEvdev(keys);
    /**
     Spawned ydotool process, awaited via its `close` event.
     */
    const child = execFile(
      'ydotool',
      [
        'key',
        ...evdev
      ]
    );
    await once(
      child,
      'close'
    );
  } catch (error) {
    /**
     Best-effort message extracted from a thrown value of unknown type.
     */
    const message = caughtValueText(error,);
    console.error(`[key-helper] ydotool/key error: ${message}`);
  }
}

/**
 Device enumeration: list what `adb devices` reports as structured entries.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { runAdb, } from './adb.ts';
import {
  type Device,
  parseDevices,
} from './parse.ts';

/**
 Module-level tagged logger; each function wraps it with its own name.
 */
const l = tagged({ tag: 'devices', },);

/**
 List every device adb currently sees, with its connection state, by
 running `adb devices` via {@link runAdb} and parsing the output with
 {@link parseDevices}.
 
 Callers filter to {@link ./constants.ts CONNECTED_STATE} before use; this
 returns the raw set so an `unauthorized` device can be surfaced to the user.
 
 @returns One Device per line adb reports.
 
 @example
 ```ts
 const devices = await listDevices();
 const ready = devices.filter((device,) => device.state === 'device',);
 ```
 */
export async function listDevices(): Promise<readonly Device[]> {
  /**
   Tagged logger for this call.
   */
  const fl = tagged({
    tag: listDevices.name,
    l,
  },);
  /**
   Captured stdout from `adb devices`.
   */
  const stdout = await runAdb({ args: ['devices',], },);
  /**
   Parsed device entries.
   */
  const devices = parseDevices({ stdout, },);
  fl.debug(`parsed ${String(devices.length,)} device line(s)`,);
  return devices;
}

/**
 key-helper daemon entry point.
 
 Claims the {@link DBUS_SERVICE} bus name, exports {@link keyHelperInterface}
 for the KWin script to drive, and monitors every Shift-capable input device
 for double-shift. Double-shift trips F20 in Neovim only while Neovide is
 focused, so the tap stays passive everywhere else (e.g. VSCodium keeps its own
 double-shift).
 
 This half does the actuation KWin scripts cannot: spawning processes, injecting
 keys via ydotool, and talking to Neovim over RPC.
 
 @module
 */

import { promisify } from 'node:util';

import { sessionBus } from '@homebridge/dbus-native';
import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';

import {
  DBUS_PATH,
  DBUS_SERVICE,
  NEOVIDE_CLASS,
} from './constants.ts';
import {
  keyHelperInterface,
  keyHelperInterfaceDescriptor,
} from './dbus-iface.ts';
import {
  findShiftDevices,
  startEvdevMonitor,
} from './evdev.ts';
import { sendNvimInput } from './nvim.ts';
import { getActiveWindowClass } from './state.ts';

/**
 `RequestName` reply code meaning this connection became primary owner of the
 requested well-known name.
 */
const DBUS_NAME_PRIMARY_OWNER = 1;

/**
 `RequestName` flags value requesting the name with no special queueing.
 */
const DBUS_NAME_NO_FLAGS = 0;

/**
 Fire F20 on double-shift, but only while Neovide is focused.
 
 @example
 ```ts
 await startEvdevMonitor({ devicePath, onDoubleShift });
 ```
 */
function onDoubleShift(): void {
  if (getActiveWindowClass() === NEOVIDE_CLASS) {
    console.log('[key-helper] double-shift, sending F20');
    void sendNvimInput('<F20>');
  }
}

/**
 Start the D-Bus service and every evdev monitor.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Session bus connection.
   */
  const bus = sessionBus();
  bus.exportInterface(
    keyHelperInterface,
    DBUS_PATH,
    keyHelperInterfaceDescriptor
  );
  /**
   Promise-returning `RequestName`, resolving to its numeric reply code.
   */
  const requestName: (
    name: string,
    flags: number
  ) => Promise<number> = promisify(bus.requestName
    .bind(bus));
  /**
   `RequestName` reply code for {@link DBUS_SERVICE}.
   */
  const retCode = await requestName(
    DBUS_SERVICE,
    DBUS_NAME_NO_FLAGS
  );
  if (retCode !== DBUS_NAME_PRIMARY_OWNER) {
    throw new Error(`[key-helper] failed to own ${DBUS_SERVICE}: RequestName returned ${retCode}`);
  }
  console.log(`[key-helper] D-Bus service registered: ${DBUS_SERVICE}`);

  /**
   Every readable Shift-capable input device.
   */
  const devices = await findShiftDevices();
  if (devices.length === 0) {
    console.error('[key-helper] no readable input devices found -- double-shift disabled');
  }
  await Promise.all(devices.map(async function monitorDevice(device): Promise<void> {
    console.log(`[key-helper] monitoring: ${device.path} (${device.name})`);
    await startEvdevMonitor({
      devicePath: device.path,
      onDoubleShift
    });
  }));
}

try {
  await main();
} catch (error) {
  /**
   Stack or message for a fatal startup failure.
   */
  const message = caughtValueStack(error,);
  console.error(`[key-helper] fatal: ${message}`);
  process.exitCode = 1;
}

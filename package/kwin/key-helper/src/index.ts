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
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  DBUS_PATH,
  DBUS_SERVICE,
  NEOVIDE_CLASS,
} from './constants.ts';
import {
  keyHelperInterface,
  keyHelperInterfaceDescriptor,
} from './dbus-iface.ts';
import { watchShiftDevices, } from './evdev.ts';
import { sendNvimInput } from './nvim.ts';
import { getActiveWindowClass } from './state.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.

 The daemon runs from the session bus, so stderr reaches whatever started it and
 the default file sink keeps a record beside it.
 */
const keyHelperLogger = tagged({ tag: 'kwin-key-helper-index', },);

//endregion Logging

//region Constants

/**
 `RequestName` reply code meaning this connection became primary owner of the
 requested well-known name.
 */
const DBUS_NAME_PRIMARY_OWNER = 1;

/**
 `RequestName` flags value requesting the name with no special queueing.
 */
const DBUS_NAME_NO_FLAGS = 0;

//endregion Constants

//region Daemon

/**
 Fire F20 on double-shift, but only while Neovide is focused.
 
 @example
 ```ts
 await startEvdevMonitor({ devicePath, onDoubleShift });
 ```
 */
function onDoubleShift(): void {
  if (getActiveWindowClass() === NEOVIDE_CLASS) {
    tagged({
      tag: onDoubleShift.name,
      l: keyHelperLogger,
    },)
      .debug('double-shift, sending F20',);
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
  tagged({
    tag: main.name,
    l: keyHelperLogger,
  },)
    .info(`D-Bus service registered: ${DBUS_SERVICE}`,);
  // The watcher runs for the life of the daemon: it rescans so a replugged or
  // newly attached keyboard is monitored again, and keeps a monitor on every
  // device it finds.
  await watchShiftDevices({ onDoubleShift, });
}

// A startup failure ends the process through the throw, so Node prints the message and the stack, and the status is one.
await main();

//endregion Daemon

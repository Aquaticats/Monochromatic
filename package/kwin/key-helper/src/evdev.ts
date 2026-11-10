/**
 evdev device discovery and monitoring (the I/O half of double-shift).
 
 All readable input devices that expose a Shift key are monitored, rather than
 trying to tell keyboards from mice: gaming mice (e.g. Razer Naga) expose a
 keyboard interface with programmable keys that may be bound to combos we care
 about. Each device stream is decoded into fixed-size evdev records and fed to
 the pure {@link reduceShiftEvent} detector.
 
 @module
 */

import { constants } from 'node:fs';
import {
  access,
  open,
  readdir,
  readFile,
} from 'node:fs/promises';
import type { Readable, } from 'node:stream';

import { wait, } from '@monochromatic-dev/async-time-core';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  INITIAL_SHIFT_STATE,
  parseBitmap,
  reduceShiftEvent,
  type ShiftState,
} from './evdev-parse.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.
 */
const evdevLogger = tagged({ tag: 'kwin-key-helper-evdev', },);

//endregion Logging

//region Constants

/**
 evdev `type` value for a key event.
 */
const EV_KEY = 1;

/**
 evdev code for the left Shift key.
 */
const KEY_LEFTSHIFT = 42;

/**
 evdev code for the right Shift key.
 */
const KEY_RIGHTSHIFT = 54;

/**
 Size of one `struct input_event` on 64-bit Linux: `timeval`(16) + type(2) +
 code(2) + value(4).
 */
const EVENT_SIZE = 24;

/**
 Byte offset of the `type` field within an event record.
 */
const TYPE_OFFSET = 16;

/**
 Byte offset of the `code` field within an event record.
 */
const CODE_OFFSET = 18;

/**
 Byte offset of the `value` field within an event record.
 */
const VALUE_OFFSET = 20;

/**
 Directory of evdev character devices.
 */
const INPUT_DIR = '/dev/input';

/**
 sysfs directory describing each input device.
 */
const SYS_INPUT_DIR = '/sys/class/input';

/**
 Milliseconds between device scans, in the daemon.

 A scan is one directory listing plus a few small sysfs reads, so five seconds
 keeps its cost negligible while a keyboard that comes back after a suspend or a
 replug is monitored again within five seconds.
 */
const DEVICE_RESCAN_MS = 5_000;

//endregion Constants

//region Types

/**
 A monitorable input device.
 
 @example
 ```ts
 const [first] = await findShiftDevices();
 ```
 */
export type Device = {
  /**
   Absolute path of the evdev character device.
   */
  readonly path: string;
  /**
   Human-readable device name from sysfs.
   */
  readonly name: string;
};

/**
 Decoded fields of one evdev event record.
 */
type DecodedEvent = {
  /**
   evdev event type.
   */
  readonly type: number;
  /**
   evdev key code.
   */
  readonly code: number;
  /**
   evdev value: 1 press, 0 release, 2 autorepeat.
   */
  readonly value: number;
};

/**
 An open device's byte stream and the way to release it.
 
 @example
 ```ts
 const device: DeviceHandle = { stream, close: async () => {} };
 ```
 */
export type DeviceHandle = {
  /**
   Stream of raw evdev records.
   */
  readonly stream: Readable;
  /**
   Release the device.
   */
  readonly close: () => Promise<void>;
};

/**
 Opens one device for reading, so a test can supply a stream instead of a real
 `/dev/input` character device.
 
 @example
 ```ts
 const openDevice: DeviceOpener = async (devicePath) => openDeviceStream(devicePath);
 ```
 */
export type DeviceOpener = (devicePath: string) => Promise<DeviceHandle>;

/**
 One running device monitor.
 
 @example
 ```ts
 const monitor: DeviceMonitor = await startEvdevMonitor({ devicePath, onDoubleShift });
 ```
 */
export type DeviceMonitor = {
  /**
   Absolute path of the device being read.
   */
  readonly devicePath: string;
  /**
   Resolves when the stream ended, failed, or the monitor was stopped; never
   rejects.
   */
  readonly settled: Promise<void>;
  /**
   Stop reading and release the device.
   */
  readonly stop: () => Promise<void>;
};

/**
 Starts one device's monitor.
 
 @example
 ```ts
 const pair: DeviceMonitorStarter = async ({ devicePath, onDoubleShift }) => startEvdevMonitor({ devicePath, onDoubleShift });
 ```
 */
export type DeviceMonitorStarter = (request: {
  readonly devicePath: string;
  readonly onDoubleShift: () => void;
}) => Promise<DeviceMonitor>;

/**
 Mutable reassembly state for one device's byte stream, a `const` holder rather
 than function-root `let` bindings.
 */
type MonitorState = {
  /**
   Detector state carried across events.
   */
  state: ShiftState;
  /**
   Bytes currently buffered in the record.
   */
  filled: number;
};

//endregion Types

//region Devices

/**
 Read a small sysfs text node into a trimmed string.
 
 @param filePath - Absolute path of the sysfs node
 
 @returns Trimmed node contents
 
 @example
 ```ts
 await readSysfs('/sys/class/input/event3/device/name');
 ```
 */
async function readSysfs(filePath: string): Promise<string> {
  return (await readFile(
    filePath,
    'utf8'
  )).trim();
}

/**
 Discover every readable input device that exposes a Shift key.
 
 @returns Devices safe to open and worth monitoring for double-shift
 
 @example
 ```ts
 const devices = await findShiftDevices();
 ```
 */
export async function findShiftDevices(): Promise<readonly Device[]> {
  /**
   All entries under {@link INPUT_DIR}.
   */
  const entries = await readdir(INPUT_DIR);
  /**
   The `eventN` character-device entries.
   */
  const eventEntries = entries.filter(function isEvent(name: string): boolean {
    return name.startsWith('event');
  });
  /**
   Probe result per entry, each a zero- or one-element array so unreadable or
   non-Shift devices drop out on flatten.
   */
  const probed = await Promise.all(
    eventEntries.map(async function probeDevice(dev: string): Promise<readonly Device[]> {
      /**
       Absolute path of this candidate device.
       */
      const path = `${INPUT_DIR}/${dev}`;
      try {
        /**
         Key-capability bitmap for this device.
         */
        const bitmap = parseBitmap(await readSysfs(`${SYS_INPUT_DIR}/${dev}/device/capabilities/key`));
        if ((!bitmap.has(KEY_LEFTSHIFT)) && (!bitmap.has(KEY_RIGHTSHIFT))) {
          return [];
        }
        // Confirm the seat ACL actually lets us read the device before listing it.
        await access(
          path,
          constants.R_OK
        );
        return [{
          path,
          name: await readSysfs(`${SYS_INPUT_DIR}/${dev}/device/name`)
        }];
      } catch (error) {
        /**
         Message for an unreadable or malformed device, which is expected.
         */
        const message = caughtValueText(error,);
        tagged({
          tag: findShiftDevices.name,
          l: evdevLogger,
        },)
          .debug(`skipping input device ${path}: ${message}`,);
        return [];
      }
    }),
  );
  return probed.flat();
}

/**
 Extract the evdev `type`, `code`, and `value` from one raw event record.
 
 @param record - Exactly {@link EVENT_SIZE} bytes of one `input_event`
 
 @returns Decoded fields needed by the double-shift detector
 
 @example
 ```ts
 const { type, code, value } = decodeEvent(buf);
 ```
 */
function decodeEvent(record: Buffer): DecodedEvent {
  return {
    type: record.readUInt16LE(TYPE_OFFSET),
    code: record.readUInt16LE(CODE_OFFSET),
    value: record.readInt32LE(VALUE_OFFSET),
  };
}

/**
 Open one device for reading, keeping the socket-like byte stream alive until
 the monitor closes it.
 
 @param devicePath - Absolute path of the evdev character device
 
 @returns The device's byte stream and the way to release it
 
 @example
 ```ts
 await openDeviceStream('/dev/input/event3');
 ```
 */
async function openDeviceStream(devicePath: string): Promise<DeviceHandle> {
  /**
   Open handle for the device.
   */
  const handle = await open(
    devicePath,
    'r'
  );
  return {
    stream: handle.createReadStream({ autoClose: false, },),
    close: async function closeDevice(): Promise<void> {
      await handle.close();
    },
  };
}

/**
 Monitor one device for double-shift, invoking `onDoubleShift` each time two
 clean Shift taps land within the detection window.

 The returned monitor reports when its stream ends or fails, so a supervisor can
 restart it, and closing it releases the device: a monitor whose read failed used
 to leave its handle open and its device unmonitored for the rest of the daemon's
 life.
 
 @param devicePath - Absolute path of the evdev device to read
 
 @param onDoubleShift - Called (fire-and-forget) when a double-shift completes
 
 @param openDevice - How to open the device, the real one by default
 
 @returns The running monitor
 
 @example
 ```ts
 await startEvdevMonitor({ devicePath: '/dev/input/event3', onDoubleShift() {} });
 ```
 */
export async function startEvdevMonitor({
  devicePath,
  onDoubleShift,
  openDevice = openDeviceStream,
}: {
  readonly devicePath: string;
  readonly onDoubleShift: () => void;
  readonly openDevice?: DeviceOpener;
}): Promise<DeviceMonitor> {
  /**
   Opened device, streamed below.
   */
  const device = await openDevice(devicePath);
  /**
   Resolves when this monitor has stopped, whatever ended it.
   */
  const {
    promise: settled,
    resolve: settle,
  } = Promise.withResolvers<void>();
  /**
   Whether a stop is already running, so the three end paths below stop once.
   */
  const stopping: { started: boolean; } = { started: false };
  /**
   Reassembly buffer for one event straddling chunk boundaries.
   */
  const record = Buffer.alloc(EVENT_SIZE);
  /**
   Mutable reassembly and detector state carried across chunks.
   */
  const monitor: MonitorState = {
    state: INITIAL_SHIFT_STATE,
    filled: 0
  };

  /**
   Stop reading and release the device.
   
   @example
   ```ts
   await stop();
   ```
   */
  async function stop(): Promise<void> {
    if (stopping.started) {
      await settled;
      return;
    }
    stopping.started = true;
    device.stream
      .destroy();
    try {
      await device.close();
    } catch (error) {
      /**
       Message for a close that failed, which happens when the stream closed the
       handle first.
       */
      const message = caughtValueText(error,);
      tagged({
        tag: stop.name,
        l: evdevLogger,
      },)
        .debug(`closing ${devicePath} failed: ${message}`,);
    }
    settle(undefined,);
  }

  device.stream
    .on(
    'data',
    function onData(chunk: Buffer): void {
      for (let offset = 0; offset < chunk.length;) {
        /**
         Bytes copied from this chunk into the record on this step.
         */
        const copied = Math.min(
          EVENT_SIZE - monitor.filled,
          chunk.length - offset
        );
        chunk.copy(
          record,
          monitor.filled,
          offset,
          offset + copied
        );
        monitor.filled += copied;
        offset += copied;
        if (monitor.filled !== EVENT_SIZE) {
          continue;
        }
        monitor.filled = 0;
        /**
         Decoded fields of the completed record.
         */
        const {
          type,
          code,
          value
        } = decodeEvent(record);
        if (type !== EV_KEY) {
          continue;
        }
        /**
         Result of folding this key event into the detector.
         */
        const {
          state: nextState,
          doubleShift
        } = reduceShiftEvent({
          state: monitor.state,
          event: (code === KEY_LEFTSHIFT) || (code === KEY_RIGHTSHIFT)
            ? {
              type: 'shift',
              key: code === KEY_LEFTSHIFT ? 'left' : 'right',
              value,
              now: performance.now()
            }
            : {
              type: 'other',
              value
            },
        });
        monitor.state = nextState;
        if (doubleShift) {
          onDoubleShift();
        }
      }
    }
    );

  device.stream
    .on(
    'error',
    function onError(error: ForeignBorrowed<Error>): void {
      tagged({
        tag: startEvdevMonitor.name,
        l: evdevLogger,
      },)
        .error(`evdev read error on ${devicePath}: ${error.message}`);
      // A failed read leaves the device unreadable until it is reopened, so
      // this monitor ends and the watcher starts a fresh one on its next scan.
      void stop();
    }
    );

  device.stream
    .on(
    'end',
    function onEnd(): void {
      tagged({
        tag: startEvdevMonitor.name,
        l: evdevLogger,
      },)
        .debug(`${devicePath} stopped returning events`,);
      void stop();
    }
    );

  device.stream
    .on(
    'close',
    function onClose(): void {
      void stop();
    }
    );

  return {
    devicePath,
    settled,
    stop,
  };
}

/**
 Forget a monitor once its stream ended, so the next scan starts a fresh one.

 The device is unplugged, or its read failed, or it was stopped: either way the
 device is no longer being read, and a later scan that still finds it must
 attach a new monitor.
 
 @param monitor - Monitor that was started
 
 @param devicePath - Path it was started for
 
 @param monitors - The watcher's live monitors, mutated here
 
 @example
 ```ts
 void forgetWhenSettled({ monitor, devicePath, monitors });
 ```
 */
async function forgetWhenSettled({
  monitor,
  devicePath,
  monitors,
}: {
  readonly monitor: DeviceMonitor;
  readonly devicePath: string;
  readonly monitors: Map<string, DeviceMonitor>;
}): Promise<void> {
  await monitor.settled;
  if (monitors.get(devicePath) === monitor) {
    monitors.delete(devicePath);
  }
}

/**
 Wait one scan interval before looking for devices again.
 
 @example
 ```ts
 await waitForDeviceRescan();
 ```
 */
async function waitForDeviceRescan(): Promise<void> {
  await wait(DEVICE_RESCAN_MS,);
}

//endregion Devices

//region Watching

/**
 Keep a monitor on every readable Shift-capable device, scanning again whenever
 the set of devices may have changed.

 The scan is what makes a replugged keyboard or a newly attached device work
 without restarting the daemon: `/dev/input` is re-listed on every pass, a
 device that is gone stops being scanned, and a device whose monitor ended is
 started again. A device that vanishes between the scan and the open is skipped
 for that pass and picked up by a later one.
 
 @param onDoubleShift - Called when any monitored device reports a double-shift
 
 @param discover - Lists the devices worth monitoring, {@link findShiftDevices} by default
 
 @param startMonitor - Starts one device's monitor, {@link startEvdevMonitor} by default
 
 @param waitForRescan - Waits one scan interval, the daemon's fixed delay by default
 
 @param signal - Stops the watcher and every monitor it started
 
 @example
 ```ts
 await watchShiftDevices({ onDoubleShift });
 ```
 */
export async function watchShiftDevices({
  onDoubleShift,
  discover = findShiftDevices,
  startMonitor = startEvdevMonitor,
  waitForRescan = waitForDeviceRescan,
  signal,
}: {
  readonly onDoubleShift: () => void;
  readonly discover?: () => Promise<readonly Device[]>;
  readonly startMonitor?: DeviceMonitorStarter;
  readonly waitForRescan?: () => Promise<void>;
  readonly signal?: AbortSignal;
}): Promise<void> {
  /**
   Monitors currently reading, by device path.
   */
  const monitors = new Map<string, DeviceMonitor>();
  /**
   Whether the empty-scan warning has been logged, so it is not repeated every
   five seconds on a machine with no readable keyboards.
   */
  const warned: { empty: boolean; } = { empty: false };

  for (;;) {
    if (signal?.aborted ?? false) {
      break;
    }
    /**
     Devices the current scan considers worth monitoring.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- a scan must finish before the devices it found are started, so no device is started twice
    const devices = await discover();
    if ((devices.length === 0) && (!warned.empty)) {
      warned.empty = true;
      tagged({
        tag: watchShiftDevices.name,
        l: evdevLogger,
      },)
        .warn('no readable input devices found -- double-shift disabled until one appears',);
    }
    if (devices.length > 0) {
      warned.empty = false;
    }
    for (const device of devices) {
      if (monitors.has(device.path)) {
        continue;
      }
      try {
        /**
         Monitor just attached to this device.
         */
        // oxlint-disable-next-line eslint/no-await-in-loop -- one monitor at a time, so a device that cannot be opened does not abort the others in this scan
        const monitor = await startMonitor({
          devicePath: device.path,
          onDoubleShift
        });
        monitors.set(
          device.path,
          monitor,
        );
        tagged({
          tag: watchShiftDevices.name,
          l: evdevLogger,
        },)
          .info(`monitoring: ${device.path} (${device.name})`,);
        void forgetWhenSettled({
          monitor,
          devicePath: device.path,
          monitors,
        });
      } catch (error) {
        /**
         Message for a device that could not be opened, e.g. because it was
         unplugged after being listed.
         */
        const message = caughtValueText(error,);
        tagged({
          tag: watchShiftDevices.name,
          l: evdevLogger,
        },)
          .debug(`could not monitor ${device.path}: ${message}`,);
      }
    }
    if (signal?.aborted ?? false) {
      break;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the interval is the point: scanning again sooner would only spend reads on an unchanged device tree
    await waitForRescan();
  }

  await Promise.all([...monitors.values()].map(async function stopMonitor(monitor): Promise<void> {
    await monitor.stop();
  }));
}

//endregion Watching

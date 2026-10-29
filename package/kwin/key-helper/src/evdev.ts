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

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  INITIAL_SHIFT_STATE,
  parseBitmap,
  reduceShiftEvent,
  type ShiftState,
} from './evdev-parse.ts';

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
        console.error(`[key-helper] skipping input device ${path}: ${message}`);
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
 Monitor one device for double-shift, invoking `onDoubleShift` each time two
 clean Shift taps land within the detection window.
 
 @param devicePath - Absolute path of the evdev device to read
 
 @param onDoubleShift - Called (fire-and-forget) when a double-shift completes
 
 @example
 ```ts
 await startEvdevMonitor({ devicePath: '/dev/input/event3', onDoubleShift() {} });
 ```
 */
export async function startEvdevMonitor({
  devicePath,
  onDoubleShift,
}: {
  readonly devicePath: string;
  readonly onDoubleShift: () => void;
}): Promise<void> {
  /**
   Open handle for the device, streamed below.
   */
  const fh = await open(
    devicePath,
    'r'
  );
  /**
   Byte stream of raw evdev records.
   */
  const stream = fh.createReadStream();
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

  stream.on(
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
          event: {
            isShift: (code === KEY_LEFTSHIFT) || (code === KEY_RIGHTSHIFT),
            value,
            now: performance.now(),
          },
        });
        monitor.state = nextState;
        if (doubleShift) {
          onDoubleShift();
        }
      }
    }
  );

  stream.on(
    'error',
    function onError(error: ForeignBorrowed<Error>): void {
      console.error(`[key-helper] evdev read error: ${error.message}`);
    }
  );

  console.log(`[key-helper] monitoring ${devicePath} for double-shift`);
}

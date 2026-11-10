/**
 Tests for the evdev monitor and the watcher that keeps one per device.

 The monitor is driven with an in-memory stream carrying evdev records, so no
 `/dev/input` device is opened and nothing the owner types is read. The watcher
 is driven with fakes for discovery, monitor startup and the scan interval, so
 a replug, a device that vanished before it could be opened, and a shutdown all
 run without touching the real device tree or waiting for a timer.

 @module
 */

import { PassThrough, } from 'node:stream';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type Device,
  type DeviceHandle,
  type DeviceMonitor,
  flushLogs,
  startEvdevMonitor,
  watchShiftDevices,
} from '../dist/final/node/testing.mjs';

/**
 evdev `type` value for a key event.
 */
const EV_KEY = 1;

/**
 evdev code for the left Shift key.
 */
const KEY_LEFTSHIFT = 42;

/**
 Size of one `struct input_event` on 64-bit Linux.
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
 evdev `value` for a key press.
 */
const KEY_PRESS = 1;

/**
 evdev `value` for a key release.
 */
const KEY_RELEASE = 0;

/**
 Path the monitor cases claim to read, never opened.
 */
const FAKE_DEVICE_PATH = '/dev/input/event-memory';

/**
 Build one raw evdev key record.
 
 @param code - evdev key code
 @param value - evdev value of the event
 @returns The record's bytes, exactly {@link EVENT_SIZE} long
 
 @example
 ```ts
 keyRecord({ code: KEY_LEFTSHIFT, value: KEY_PRESS });
 ```
 */
function keyRecord({ code, value }: {
  readonly code: number;
  readonly value: number;
}): Buffer {
  /**
   Record bytes, zero-filled for the fields this test does not set.
   */
  const record = Buffer.alloc(EVENT_SIZE);
  record.writeUInt16LE(
    EV_KEY,
    TYPE_OFFSET,
  );
  record.writeUInt16LE(
    code,
    CODE_OFFSET,
  );
  record.writeInt32LE(
    value,
    VALUE_OFFSET,
  );
  return record;
}

/**
 A stream-backed stand-in device, with the close recorded.
 
 @example
 ```ts
 const fake = makeDevice();
 await fake.close();
 ```
 */
type FakeDevice = {
  /**
   The device as the monitor sees it.
   */
  readonly handle: DeviceHandle;
  /**
   The stream the test writes records into.
   */
  readonly stream: PassThrough;
  /**
   How many times the monitor closed the device.
   */
  readonly closeCount: () => number;
};

/**
 Build a stand-in device whose stream the test drives.
 
 @returns The device handle, its stream, and the close counter
 
 @example
 ```ts
 const fake = makeDevice();
 fake.stream.write(keyRecord({ code: KEY_LEFTSHIFT, value: KEY_PRESS }));
 ```
 */
function makeDevice(): FakeDevice {
  /**
   Stream standing in for the device's reads.
   */
  const stream = new PassThrough();
  /**
   Closes recorded so far.
   */
  const closes: number[] = [];
  return {
    handle: {
      stream,
      close: async function closeDevice(): Promise<void> {
        closes.push(closes.length);
      },
    },
    stream,
    closeCount: function closeCount(): number {
      return closes.length;
    },
  };
}

/**
 A stand-in monitor whose settling and stopping the test controls.
 
 @example
 ```ts
 const fake = makeMonitor({ devicePath: FAKE_DEVICE_PATH });
 fake.settle();
 ```
 */
type FakeMonitor = {
  /**
   The monitor as the watcher sees it.
   */
  readonly monitor: DeviceMonitor;
  /**
   Report that this monitor's stream ended.
   */
  readonly settle: () => void;
  /**
   How many times the watcher stopped it.
   */
  readonly stopCount: () => number;
};

/**
 Build a stand-in monitor.
 
 @param devicePath - Device it claims to read
 @returns The monitor, plus its settle and stop counters
 
 @example
 ```ts
 const fake = makeMonitor({ devicePath: FAKE_DEVICE_PATH });
 ```
 */
function makeMonitor({ devicePath }: { readonly devicePath: string; }): FakeMonitor {
  /**
   Resolves when this monitor ends.
   */
  const settled = Promise.withResolvers<void>();
  /**
   Stops recorded so far.
   */
  const stops: number[] = [];
  return {
    monitor: {
      devicePath,
      settled: settled.promise,
      stop: async function stopMonitor(): Promise<void> {
        stops.push(stops.length);
        settled.resolve(undefined);
      },
    },
    settle: function settle(): void {
      settled.resolve(undefined);
    },
    stopCount: function stopCount(): number {
      return stops.length;
    },
  };
}

/**
 A discovered device that is safe to fake.
 
 @param path - Its device path
 @returns The device the watcher's `discover` returns
 
 @example
 ```ts
 fakeDevice('/dev/input/event1');
 ```
 */
function fakeDevice(path: string): Device {
  return {
    path,
    name: 'stand-in keyboard',
  };
}

/**
 Stop a monitor when the surrounding scope ends.
 
 @param monitor - Monitor a case started
 
 @returns A disposer that stops it
 
 @example
 ```ts
 try (running = disposingMonitor({ monitor })) {}
 ```
 */
function disposingMonitor({ monitor }: { readonly monitor: DeviceMonitor; }): {
  readonly [Symbol.asyncDispose]: () => Promise<void>;
} {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await monitor.stop();
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: startEvdevMonitor.name,
      children: [
        it({
          name: 'fires onDoubleShift for two clean Shift taps',
          fn: async () => {
            const fake = makeDevice();
            /** Double-shifts the monitor reported. */
            const fired: number[] = [];
            const monitor = await startEvdevMonitor({
              devicePath: FAKE_DEVICE_PATH,
              onDoubleShift: function onDoubleShift(): void {
                fired.push(fired.length);
              },
              openDevice: async function openDevice(): Promise<DeviceHandle> {
                return fake.handle;
              },
            });
            await using _running = disposingMonitor({ monitor, });
            fake.stream.write(Buffer.concat([
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_PRESS, }),
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_RELEASE, }),
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_PRESS, }),
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_RELEASE, }),
            ],));
            expect(fired).toHaveLength(1);
          },
        }),
        it({
          name: 'reassembles a record split across chunks',
          fn: async () => {
            const fake = makeDevice();
            /** Double-shifts the monitor reported. */
            const fired: number[] = [];
            const monitor = await startEvdevMonitor({
              devicePath: FAKE_DEVICE_PATH,
              onDoubleShift: function onDoubleShift(): void {
                fired.push(fired.length);
              },
              openDevice: async function openDevice(): Promise<DeviceHandle> {
                return fake.handle;
              },
            });
            await using _running = disposingMonitor({ monitor, });
            /**
             Release record delivered in two pieces, the second completing it.
             */
            const release = keyRecord({ code: KEY_LEFTSHIFT, value: KEY_RELEASE, });
            /**
             Byte count of the first piece, chosen to split the record.
             */
            const splitAt = Math.floor(EVENT_SIZE / 2);
            fake.stream.write(Buffer.concat([
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_PRESS, }),
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_RELEASE, }),
              keyRecord({ code: KEY_LEFTSHIFT, value: KEY_PRESS, }),
              release.subarray(0, splitAt),
            ],));
            fake.stream.write(release.subarray(splitAt,));
            expect(fired).toHaveLength(1);
          },
        }),
        it({
          name: 'closes the device and settles when the stream fails',
          fn: async () => {
            const fake = makeDevice();
            const monitor = await startEvdevMonitor({
              devicePath: FAKE_DEVICE_PATH,
              onDoubleShift: function onDoubleShift(): void {
                // Not reached: no records are written.
              },
              openDevice: async function openDevice(): Promise<DeviceHandle> {
                return fake.handle;
              },
            });
            fake.stream.destroy(new Error('Input/output error',));
            await monitor.settled;
            expect(fake.closeCount()).toBe(1);
          },
        }),
        it({
          name: 'closes the device and settles when the stream ends',
          fn: async () => {
            const fake = makeDevice();
            const monitor = await startEvdevMonitor({
              devicePath: FAKE_DEVICE_PATH,
              onDoubleShift: function onDoubleShift(): void {
                // Not reached: no records are written.
              },
              openDevice: async function openDevice(): Promise<DeviceHandle> {
                return fake.handle;
              },
            });
            fake.stream.end();
            await monitor.settled;
            expect(fake.closeCount()).toBe(1);
          },
        }),
        it({
          name: 'releases the device once however often it is stopped',
          fn: async () => {
            const fake = makeDevice();
            const monitor = await startEvdevMonitor({
              devicePath: FAKE_DEVICE_PATH,
              onDoubleShift: function onDoubleShift(): void {
                // Not reached: no records are written.
              },
              openDevice: async function openDevice(): Promise<DeviceHandle> {
                return fake.handle;
              },
            });
            await monitor.stop();
            await monitor.stop();
            expect(fake.closeCount()).toBe(1);
          },
        }),
      ],
    }),
    describe({
      name: watchShiftDevices.name,
      // One at a time: a case that finds no device logs a warning, and the case
      // that counts warnings reads the same process-wide stream.
      concurrency: 1,
      children: [
        it({
          name: 'monitors every discovered device once',
          fn: async () => {
            /** Devices the fake discovery claims to see. */
            const devices: readonly Device[] = [
              fakeDevice('/dev/input/event1'),
              fakeDevice('/dev/input/event2'),
            ];
            /** Paths the watcher started a monitor for. */
            const started: string[] = [];
            /** Monitors the watcher started. */
            const monitors: FakeMonitor[] = [];
            /** Stops the watcher once the second scan has run. */
            const controller = new AbortController();
            /** Scans the fake discovery answered. */
            const scans: number[] = [];
            await watchShiftDevices({
              onDoubleShift: function onDoubleShift(): void {
                // The devices in these cases never report a tap.
              },
              discover: async function discover(): Promise<readonly Device[]> {
                scans.push(scans.length);
                if (scans.length >= 2) {
                  controller.abort();
                }
                return devices;
              },
              startMonitor: async function startMonitor({ devicePath }): Promise<DeviceMonitor> {
                started.push(devicePath);
                /**
                 Stand-in monitor for this device.
                 */
                const fake = makeMonitor({ devicePath, });
                monitors.push(fake);
                return fake.monitor;
              },
              waitForRescan: async function waitForRescan(): Promise<void> {
                // The next scan starts as soon as the current pass ends.
              },
              signal: controller.signal,
            });
            expect(started).toEqual(['/dev/input/event1', '/dev/input/event2',]);
            expect(monitors.map(function stopCount(fake: FakeMonitor): number {
              return fake.stopCount();
            })).toEqual([1, 1,]);
          },
        }),
        it({
          name: 'monitors a device again when its monitor ended',
          fn: async () => {
            /** Paths the watcher started a monitor for. */
            const started: string[] = [];
            /** Monitors the watcher started, in order. */
            const monitors: FakeMonitor[] = [];
            /** Stops the watcher once the third scan has run. */
            const controller = new AbortController();
            /** Scans the fake discovery answered. */
            const scans: number[] = [];
            await watchShiftDevices({
              onDoubleShift: function onDoubleShift(): void {
                // The device in this case never reports a tap.
              },
              discover: async function discover(): Promise<readonly Device[]> {
                scans.push(scans.length);
                if (scans.length >= 3) {
                  controller.abort();
                }
                return [fakeDevice('/dev/input/event1'),];
              },
              startMonitor: async function startMonitor({ devicePath }): Promise<DeviceMonitor> {
                started.push(devicePath);
                /**
                 Stand-in monitor for this device.
                 */
                const fake = makeMonitor({ devicePath, });
                monitors.push(fake);
                if (started.length === 1) {
                  // The keyboard was unplugged: its reads end straight away.
                  fake.settle();
                }
                return fake.monitor;
              },
              waitForRescan: async function waitForRescan(): Promise<void> {
                // The next scan starts as soon as the current pass ends.
              },
              signal: controller.signal,
            });
            expect(started).toEqual(['/dev/input/event1', '/dev/input/event1',]);
          },
        }),
        it({
          name: 'stops monitoring a device that disappeared',
          fn: async () => {
            /** Paths the watcher started a monitor for. */
            const started: string[] = [];
            /** Stops the watcher once the third scan has run. */
            const controller = new AbortController();
            /** Scans the fake discovery answered. */
            const scans: number[] = [];
            await watchShiftDevices({
              onDoubleShift: function onDoubleShift(): void {
                // The device in this case never reports a tap.
              },
              discover: async function discover(): Promise<readonly Device[]> {
                scans.push(scans.length);
                if (scans.length >= 3) {
                  controller.abort();
                }
                return scans.length === 1 ? [fakeDevice('/dev/input/event1'),] : [];
              },
              startMonitor: async function startMonitor({ devicePath }): Promise<DeviceMonitor> {
                started.push(devicePath);
                /**
                 Stand-in monitor for the device that is about to vanish.
                 */
                const fake = makeMonitor({ devicePath, });
                fake.settle();
                return fake.monitor;
              },
              waitForRescan: async function waitForRescan(): Promise<void> {
                // The next scan starts as soon as the current pass ends.
              },
              signal: controller.signal,
            });
            expect(started).toEqual(['/dev/input/event1',]);
          },
        }),
        it({
          name: 'keeps watching when a device cannot be opened',
          fn: async () => {
            /** Devices the fake discovery claims to see, the broken one first. */
            const devices: readonly Device[] = [
              fakeDevice('/dev/input/event-broken'),
              fakeDevice('/dev/input/event1'),
            ];
            /** Paths the watcher started a monitor for. */
            const started: string[] = [];
            /** Stops the watcher once the second scan has run. */
            const controller = new AbortController();
            /** Scans the fake discovery answered. */
            const scans: number[] = [];
            await watchShiftDevices({
              onDoubleShift: function onDoubleShift(): void {
                // The device in this case never reports a tap.
              },
              discover: async function discover(): Promise<readonly Device[]> {
                scans.push(scans.length);
                if (scans.length >= 2) {
                  controller.abort();
                }
                return devices;
              },
              startMonitor: async function startMonitor({ devicePath }): Promise<DeviceMonitor> {
                if (devicePath === '/dev/input/event-broken') {
                  throw new Error('device vanished before it could be opened',);
                }
                started.push(devicePath);
                return makeMonitor({ devicePath, }).monitor;
              },
              waitForRescan: async function waitForRescan(): Promise<void> {
                // The next scan starts as soon as the current pass ends.
              },
              signal: controller.signal,
            });
            expect(started).toEqual(['/dev/input/event1',]);
          },
        }),
        it({
          name: 'warns once when no device is readable',
          fn: async ctx => {
            /** Stops the watcher after two empty scans. */
            const controller = new AbortController();
            /** Scans the fake discovery answered. */
            const scans: number[] = [];
            /** Recorder over the daemon's log destination. */
            const standardError = ctx.sinon.spy(process.stderr, 'write');
            await watchShiftDevices({
              onDoubleShift: function onDoubleShift(): void {
                // No device, so no tap.
              },
              discover: async function discover(): Promise<readonly Device[]> {
                scans.push(scans.length);
                if (scans.length >= 2) {
                  controller.abort();
                }
                return [];
              },
              startMonitor: async function startMonitor(): Promise<DeviceMonitor> {
                throw new Error('no device may be started in this case',);
              },
              waitForRescan: async function waitForRescan(): Promise<void> {
                // The next scan starts as soon as the current pass ends.
              },
              signal: controller.signal,
            });
            await flushLogs();
            /** Log lines warning that nothing is readable. */
            const warnings = standardError.args
              .map(function lineText(call: readonly unknown[]): string {
                return String(call[0]);
              })
              .filter(function namesMissingDevices(line: string): boolean {
                return line.includes('no readable input devices found');
              });
            expect(warnings).toHaveLength(1);
          },
        }),
      ],
    }),
  ],
});

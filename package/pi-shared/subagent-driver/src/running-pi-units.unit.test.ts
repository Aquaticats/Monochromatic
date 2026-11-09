/**
 The Pi runs that are going now, read from the text systemd would have printed and from a stand-in for the command that asks it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  PiUnitsUnreadableError,
  parseRunningPiUnits,
  type RunCommand,
  startRunningPiUnits,
  unitNameOfGiven,
} from '../dist/final/node/index.mjs';

/**
 Microseconds in one minute, which is the unit systemd counts the start of a unit in.
 */
const MICROSECONDS_PER_MINUTE = 60_000_000;

/**
 Two units systemd printed, one of them running and one of them finished, followed by a unit that is still starting.
 */
const SHOWN = [
  'Id=pi-alpha.service',
  'ActiveState=active',
  'ActiveEnterTimestampMonotonic=1000000',
  'MainPID=4242',
  '',
  'Id=pi-finished.service',
  'ActiveState=inactive',
  'ActiveEnterTimestampMonotonic=2000000',
  'MainPID=0',
  '',
  'Id=pi-starting.service',
  'ActiveState=activating',
  'ActiveEnterTimestampMonotonic=0',
  'MainPID=0',
  '',
].join(
  '\n',
  );

await describe({
  name: parseRunningPiUnits.name,
  children: [
    it({
      name: 'reads the runs that are going, with their age on the clock systemd stamps, and leaves the rest out',
      fn: async () => {
        /**
         The units systemd would have named as running.
         */
        const units = parseRunningPiUnits({
          shown: SHOWN,
          nowMicroseconds: 1_000_000 + (5 * MICROSECONDS_PER_MINUTE),
        },);
        expect(units,).toEqual([
          {
            name: 'alpha',
            unitName: 'pi-alpha.service',
            elapsedMilliseconds: 5 * 60_000,
            mainPid: 4242,
          },
        ],);
      },
    },),
    it({
      name: 'reads the last block of a listing the output did not close with a blank line',
      fn: async () => {
        /**
         A finished unit, then the running one with nothing after it.
         */
        const units = parseRunningPiUnits({
          shown: [
            'Id=pi-finished.service',
            'ActiveState=inactive',
            'ActiveEnterTimestampMonotonic=0',
            'MainPID=0',
            '',
            'Id=pi-beta.service',
            'ActiveState=active',
            'ActiveEnterTimestampMonotonic=0',
            'MainPID=7',
          ].join(
            '\n',
            ),
          nowMicroseconds: 1_000,
        },);
        expect(units).toHaveLength(1,);
        expect(units[0]?.name,).toBe('beta',);
      },
    },),
    it({
      name: 'reads an empty answer as no run at all',
      fn: async () => {
        expect(parseRunningPiUnits({
          shown: '',
          nowMicroseconds: 0,
        },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: startRunningPiUnits.name,
  children: [
    it({
      name: 'asks systemd for the running Pi units and reads what it printed',
      fn: async () => {
        /**
         What the command was asked.
         */
        const asked: { command: string; words: readonly string[] }[] = [];
        /**
         A stand-in for systemd.
         */
        const run: RunCommand = async function standInSystemctl(request,) {
          asked.push(request,);
          return {
            status: 0,
            stdout: SHOWN,
            stderr: '',
          };
        };
        /**
         Reads the units.
         */
        const units = startRunningPiUnits({
          run,
          nowMicroseconds: function standInClock() {
            return 1_000_000 + (2 * MICROSECONDS_PER_MINUTE);
          },
        },);
        expect(await units(),).toEqual([
          {
            name: 'alpha',
            unitName: 'pi-alpha.service',
            elapsedMilliseconds: 2 * 60_000,
            mainPid: 4242,
          },
        ],);
        expect(asked,).toHaveLength(1,);
        expect(asked[0]?.command,).toBe('systemctl',);
        expect(asked[0]?.words,).toContain('pi-*.service',);
      },
    },),
    it({
      name: 'refuses when systemd could not be asked, naming what it said',
      fn: async () => {
        /**
         A stand-in for a systemd that is not running for this user.
         */
        const run: RunCommand = async function refuse() {
          return {
            status: 1,
            stdout: '',
            stderr: 'Failed to connect to bus',
          };
        };
        try {
          await startRunningPiUnits({ run, },)();
        }
        catch (error) {
          expect(error,).toBeInstanceOf(PiUnitsUnreadableError,);
          expect(String(error,),).toContain('Failed to connect to bus',);
          return;
        }
        throw new Error('a refused question was read as no units',);
      },
    },),
  ],
},);

await describe({
  name: unitNameOfGiven.name,
  children: [
    it({
      name: 'gives the unit name of a run name, which is what a line to act on prints',
      fn: async () => {
        expect(unitNameOfGiven({ name: 'copy-route', },),).toBe('pi-copy-route.service',);
      },
    },),
    it({
      name: 'keeps a unit name that is already whole, and adds only what it is missing',
      fn: async () => {
        expect(unitNameOfGiven({ name: 'pi-copy-route.service', },),).toBe('pi-copy-route.service',);
        expect(unitNameOfGiven({ name: 'pi-copy-route', },),).toBe('pi-copy-route.service',);
        expect(unitNameOfGiven({ name: '  copy-route.service  ', },),).toBe('pi-copy-route.service',);
      },
    },),
  ],
},);

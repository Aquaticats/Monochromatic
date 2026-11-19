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
  type CommandOutput,
  PiUnitsUnreadableError,
  parseRunningPiUnits,
  type RunCommand,
  startRunningPiUnits,
  unitNameOfGiven,
} from '../dist/final/node/index.mjs';

/**
 What one question of a stand-in command was.
 */
type Question = {
  /**
   The program asked.
   */
  readonly command: string;

  /**
   Its arguments.
   */
  readonly words: readonly string[];
};

/**
 Microseconds in one minute, the elapsed interval between the fixture's start stamps and reading.
 */
const MICROSECONDS_PER_MINUTE = 60_000_000;

/**
 Microseconds systemd counts in each millisecond of reported age.
 */
const MICROSECONDS_PER_MILLISECOND = 1_000;

/**
 A numbered active unit, an ended unit, and an unnumbered unit activating again after having been active.
 */
const SHOWN = [
  'Id=pi-alpha@1.service',
  'ActiveState=active',
  `ActiveEnterTimestampMonotonic=${String(MICROSECONDS_PER_MINUTE,)}`,
  'MainPID=4242',
  '',
  'Id=pi-finished.service',
  'ActiveState=inactive',
  'ActiveEnterTimestampMonotonic=2000000',
  'MainPID=0',
  '',
  'Id=pi-starting.service',
  'ActiveState=activating',
  `ActiveEnterTimestampMonotonic=${String(MICROSECONDS_PER_MINUTE,)}`,
  'MainPID=0',
  '',
].join(
  '\n',
);

/**
 Systemd's clock when the fixture is read, one minute after its active-enter stamps.
 */
const READING_MICROSECONDS = MICROSECONDS_PER_MINUTE + MICROSECONDS_PER_MINUTE;

/**
 Elapsed time since either recorded active-enter stamp.
 */
const FIXTURE_AGE_MILLISECONDS = MICROSECONDS_PER_MINUTE / MICROSECONDS_PER_MILLISECOND;

/**
 Every unfinished unit from SHOWN, retaining its original unit name and omitting a number for legacy units.
 */
const UNFINISHED_UNITS = [
  {
    name: 'alpha',
    unitName: 'pi-alpha@1.service',
    runNumber: 1,
    elapsedMilliseconds: FIXTURE_AGE_MILLISECONDS,
    mainPid: 4_242,
    state: 'active',
  },
  {
    name: 'starting',
    unitName: 'pi-starting.service',
    elapsedMilliseconds: FIXTURE_AGE_MILLISECONDS,
    mainPid: 0,
    state: 'activating',
  },
];

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
          nowMicroseconds: READING_MICROSECONDS,
        },);
        expect(units,).toEqual(UNFINISHED_UNITS,);
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
      name: 'reads a unit that is starting or stopping as a run that has not ended, and one that ended as no run',
      fn: async () => {
        expect(parseRunningPiUnits({
          shown: [
            'Id=pi-alpha@2.service',
            'ActiveState=activating',
            'ActiveEnterTimestampMonotonic=0',
            'MainPID=9',
            '',
            'Id=pi-alpha@1.service',
            'ActiveState=deactivating',
            'ActiveEnterTimestampMonotonic=0',
            'MainPID=8',
            '',
            'Id=pi-alpha.service',
            'ActiveState=inactive',
            'ActiveEnterTimestampMonotonic=0',
            'MainPID=0',
          ].join(
            '\n',
          ),
          nowMicroseconds: 1_000,
        },).map(function nameOf(unit,) {
          return [unit.name, unit.runNumber, unit.state,];
        },),).toEqual([
          ['alpha', 2, 'activating',],
          ['alpha', 1, 'deactivating',],
        ],);
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
        const asked: Question[] = [];
        /**
         Answers systemd's question with the text systemd would have printed.

         @param request - the program and its arguments

         @returns what systemd would have printed

         @example
         ```ts
         const shown = await standInSystemctl({ command: 'systemctl', words: [], },);
         ```
         */
        async function standInSystemctl(request: Question,): Promise<CommandOutput> {
          asked.push(request,);
          return {
            status: 0,
            stdout: SHOWN,
            stderr: '',
          };
        }

        /**
         A stand-in for systemd.
         */
        const run: RunCommand = standInSystemctl;
        /**
         Reads the units.
         */
        const units = startRunningPiUnits({
          run,
          nowMicroseconds: function standInClock() {
            return READING_MICROSECONDS;
          },
        },);
        expect(await units(),).toEqual(UNFINISHED_UNITS,);
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

         @returns what a systemd with no bus to talk to prints

         @example
         ```ts
         const shown = await refuse({ command: 'systemctl', words: [], },);
         ```
         */
        async function refuse(): Promise<CommandOutput> {
          return {
            status: 1,
            stdout: '',
            stderr: 'Failed to connect to bus',
          };
        }

        /**
         A stand-in for a systemd that refuses.
         */
        const run: RunCommand = refuse;
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
        expect(unitNameOfGiven({ name: '  copy-route.service  ', },),).toBe('pi-copy-route.service',);
      },
    },),
    it({
      name: 'preserves a run name starting with pi- instead of mistaking it for a unit name',
      fn: async () => {
        const runNumber = 2;
        expect(unitNameOfGiven({ name: 'pi-copy-route', runNumber, },),)
          .toBe(`pi-pi-copy-route@${String(runNumber,)}.service`,);
      },
    },),
  ],
},);

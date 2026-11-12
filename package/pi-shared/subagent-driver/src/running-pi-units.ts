/**
 The Pi runs that are going now, as systemd reports them: each unit's name, how long it has been up, and the process it leads.

 The question to systemd is the seam: a test hands over the text a question would have printed and never needs a real unit.

 @module
 */

import { hrtime, } from 'node:process';
import type { RunCommand, } from './run-command.ts';

/**
 Nanoseconds in one microsecond, which is the unit of the timestamps systemd prints.
 */
const NANOSECONDS_PER_MICROSECOND = 1_000n;

/**
 Microseconds in one millisecond.
 */
const MICROSECONDS_PER_MILLISECOND = 1_000;

/**
 The properties asked of every unit.

 `ActiveEnterTimestampMonotonic` is the start of the unit on the same clock `process.hrtime.bigint` reads, so the age of a unit
 needs no wall clock and cannot be moved by one.
 */
const UNIT_PROPERTIES = 'Id,ActiveState,ActiveEnterTimestampMonotonic,MainPID';

/**
 The start of every Pi run's unit name, as the recipe in `.agents/skills/pi-as-subagent` writes it.
 */
const PI_UNIT_PREFIX = 'pi-';

/**
 The end of every systemd unit name.
 */
const UNIT_SUFFIX = '.service';

/**
 A Pi run that is going now.
 */
export type RunningPiUnit = {
  /**
   The run's name, which is also the start of its run folder's name.
   */
  readonly name: string;

  /**
   The unit's name as systemd prints it.
   */
  readonly unitName: string;

  /**
   How long the unit has been up.
   */
  readonly elapsedMilliseconds: number;

  /**
   The process the unit leads, which is the driver.
   */
  readonly mainPid: number;
};

/**
 systemd's answer could not be read, so no run's age is known.
 */
export class PiUnitsUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PiUnitsUnreadableError';
}

/**
 What one block of unit properties says, when it describes a run that is going.

 A block describing a unit that is not running answers {@link NOT_RUNNING}, which is a symbol so that no unit's name can be
 mistaken for it.
 */
export const NOT_RUNNING: unique symbol = Symbol('systemd block describes no running Pi unit',);

/**
 Reads the clock systemd stamps a unit's start with.

 @returns microseconds of `CLOCK_MONOTONIC`, which is the clock `process.hrtime.bigint` reads

 @example
 ```ts
 const now = monotonicMicrosecondsNow();
 ```
 */
export function monotonicMicrosecondsNow(): number {
  return Number(hrtime.bigint() / NANOSECONDS_PER_MICROSECOND,);
}

/**
 Reads the unit name a person or a log line named.

 @param name - a systemd unit name, or a run's name

 @returns the unit name, with the prefix and suffix the recipe uses

 @example
 ```ts
 const unitName = unitNameOfGiven({ name: 'foo', },);
 ```
 */
export function unitNameOfGiven({
  name,
}: {
  readonly name: string;
},): string {
  /**
   The name without the unit suffix systemd adds.
   */
  const withoutSuffix = name.trim()
    .replace(
    UNIT_SUFFIX,
    '',
    );
  return (withoutSuffix.startsWith(
    PI_UNIT_PREFIX,
  ) ? withoutSuffix : `${PI_UNIT_PREFIX}${withoutSuffix}`) + UNIT_SUFFIX;
}

/**
 Reads the run name a unit's name carries.

 @param unitName - the unit's name as systemd prints it

 @returns the run's name, or the unit's name when it does not follow the recipe

 @example
 ```ts
 const runName = runNameOfUnit({ unitName: 'pi-foo.service', },);
 ```
 */
export function runNameOfUnit({
  unitName,
}: {
  readonly unitName: string;
},): string {
  /**
   The name without its prefix and suffix, when it has both.
   */
  const bare = unitName.endsWith(UNIT_SUFFIX,)
    ? unitName.slice(
      0,
      -UNIT_SUFFIX.length,
    )
    : unitName;
  return bare.startsWith(PI_UNIT_PREFIX,)
    ? bare.slice(PI_UNIT_PREFIX.length,)
    : bare;
}

/**
 Reads the properties systemd printed for one unit.

 @param block - one unit's lines, each `Name=Value`

 @param nowMicroseconds - the reading of the same clock the start timestamp is on

 @returns the running unit, or {@link NOT_RUNNING}

 @example
 ```ts
 const unit = runningUnitOfBlock({ block: 'Id=pi-a.service', nowMicroseconds: 1_000, },);
 ```
 */
function runningUnitOfBlock({
  block,
  nowMicroseconds,
}: {
  readonly block: string;
  readonly nowMicroseconds: number;
},): RunningPiUnit | typeof NOT_RUNNING {
  /**
   The block's properties, by name.
   */
  const fields = new Map(block
    .split(
    '\n',
    )
    .filter(function isProperty(line,) {
      return line.includes('=',);
    },)
    .map(function asEntry(line,) {
      /**
       Where the property's name ends.
       */
      const split = line.indexOf('=',);
      return [
        line.slice(
          0,
          split,
        ),
        line.slice(split + 1,),
      ] as const;
    },));
  /**
   The unit's name.
   */
  const unitName = fields.get('Id',);
  /**
   Where the unit's start is on the monotonic clock.
   */
  const started = fields.get('ActiveEnterTimestampMonotonic',);
  if (
    (unitName === undefined)
      || (started === undefined)
      || (fields.get('ActiveState',) !== 'active')
  ) {
    return NOT_RUNNING;
  }
  return {
    name: runNameOfUnit({ unitName, },),
    unitName,
    elapsedMilliseconds: (nowMicroseconds - Number(started,)) / MICROSECONDS_PER_MILLISECOND,
    mainPid: Number(fields.get('MainPID',) ?? 0,),
  };
}

/**
 Reads every running Pi unit systemd printed.

 @param shown - what `systemctl show` printed, one block of properties per unit

 @param nowMicroseconds - the reading of the clock the start timestamps are on

 @returns the running units, in the order systemd printed them

 @example
 ```ts
 const units = parseRunningPiUnits({ shown: 'Id=pi-a.service\nActiveState=active\n', nowMicroseconds: 0, },);
 ```
 */
export function parseRunningPiUnits({
  shown,
  nowMicroseconds,
}: {
  readonly shown: string;
  readonly nowMicroseconds: number;
},): readonly RunningPiUnit[] {
  /**
   The running units found so far.
   */
  const units: RunningPiUnit[] = [];
  /**
   The properties of the block being read.
   */
  let block: string[] = [];
  /**
   Adds the block just finished to the units, when it described a running one.
   */
  function takeBlock(): void {
    /**
     The unit the finished block described, when it described a running one.
     */
    const unit = runningUnitOfBlock({
      block: block.join('\n',),
      nowMicroseconds,
    },);
    if ((typeof unit) !== 'symbol') {
      units.push(unit,);
    }
  }

  for (const line of shown.split(
    '\n',
  )) {
    if (line.trim() !== '') {
      block.push(line,);
      continue;
    }
    takeBlock();
    block = [];
  }
  takeBlock();
  return units;
}

/**
 Reads the Pi units that are running now, asking systemd.

 @param run - how a command is run, which a test replaces

 @param nowMicroseconds - where the age of a unit is measured from

 @returns what to await for the running units at that moment

 @throws PiUnitsUnreadableError when systemd could not be asked, or refused

 @example
 ```ts
 const units = startRunningPiUnits({ run: runCommandWithChildProcess, },);
 const running = await units();
 ```
 */
export function startRunningPiUnits({
  run,
  nowMicroseconds = monotonicMicrosecondsNow,
}: {
  readonly run: RunCommand;
  readonly nowMicroseconds?: () => number;
},): () => Promise<readonly RunningPiUnit[]> {
  return async function runningPiUnits(): Promise<readonly RunningPiUnit[]> {
    /**
     What systemd said about the Pi units.
     */
    const shown = await run({
      command: 'systemctl',
      words: [
        '--user',
        'show',
        `--property=${UNIT_PROPERTIES}`,
        'pi-*.service',
      ],
    },);
    if (shown.status !== 0) {
      throw new PiUnitsUnreadableError(
        `systemctl could not list the Pi units: ${shown.problem
          ?? shown.stderr
            .trim()}`,
      );
    }
    return parseRunningPiUnits({
      shown: shown.stdout,
      nowMicroseconds: nowMicroseconds(),
    },);
  };
}

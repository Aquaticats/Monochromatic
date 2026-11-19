/**
 The Pi runs that are going now, as systemd reports them: each unit's name, how long it has been up, and the process it leads.

 A run's unit is named by the run's name and the number of the run folder its start claimed: `pi-<name>@<number>.service`. Two
 starts of one run name claim different folders, so they are two units and neither can be refused for asking for the other's name.
 The part before `@` is the run's name, which is what a reader that looks a run up by name needs, and the part after it is the
 number of the folder that run writes into.

 The question to systemd is the seam: a test hands over the text a question would have printed and never needs a real unit.

 @module
 */

import { hrtime, } from 'node:process';
import {
  firstWordOf,
  isDigitsOnly,
} from './text-of-an-option.ts';
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
 The character between a run's name and the number of the folder its start claimed, which no run's name may carry, so a reader can
 always tell the two apart. Systemd accepts it in a transient unit's name and refuses a second unit of the same name, which is what
 makes the unit name a name one start holds alone.
 */
export const PI_UNIT_NUMBER_SEPARATOR = '@';

/**
 The states a unit is in before it has ended, which are the states in which a run it carries is going: a unit being started and a unit
 being stopped both still hold their run's work, and `wait-for-a-unit-to-end` and the worktree removal read the same set.

 A state of its own, such as `inactive` or `failed`, is a unit that has ended; empty text names no state at all, which is not a state
 a run has ended in either.
 */
export const UNIT_STATES_NOT_ENDED: ReadonlySet<string> = new Set<string>([
  'active',
  'activating',
  'reloading',
  'refreshing',
  'deactivating',
]);

/**
 What a unit's name says: the run's name, and the number of the run folder its start claimed when the name carries one.
 */
export type UnitNameParts = {
  /**
   The run's name, which is everything between the prefix and the number.
   */
  readonly name: string;

  /**
   The number of the run folder its start claimed, when the name carries one.
   */
  readonly runNumber?: number;
};

/**
 Reads what a unit's name says, in one place, so every reader of a unit's name agrees about the form it takes.

 A unit is `pi-<name>.service` or `pi-<name>@<number>.service`: the first form is one a launcher of an earlier version made or a
 person started by the recipe, and the second is every start this launcher makes. The `pi-` prefix and the `.service` suffix are the
 unit's own, so a run's name is never taken to begin with `pi-` or to end with `.service`.

 @param unitName - the unit's name as systemd prints it

 @returns the run's name, and the number the name carries when it carries one

 @example
 ```ts
 const parts = unitNameParts({ unitName: 'pi-copy-route@3.service', },);
 ```
 */
export function unitNameParts({ unitName, }: { readonly unitName: string },): UnitNameParts {
  /**
   The name without the unit's own prefix and suffix.
   */
  const withoutSuffix = unitName.trim()
    .replace(
    UNIT_SUFFIX,
    '',
    );
  /**
   The name without the unit's own prefix, which a run's name never begins with.
   */
  const bare = withoutSuffix.startsWith(PI_UNIT_PREFIX,)
    ? withoutSuffix.slice(PI_UNIT_PREFIX.length,)
    : withoutSuffix;
  /**
   Where the separator stands, or minus one when the name carries no number.
   */
  const separator = bare.indexOf(PI_UNIT_NUMBER_SEPARATOR,);
  if (separator === (-1)) {
    return { name: bare, };
  }
  /**
   The text after the separator, which is the number when it is whole.
   */
  const written = bare.slice(separator + PI_UNIT_NUMBER_SEPARATOR.length,);
  return {
    name: bare.slice(
      0,
      separator,
    ),
    ...(isDigitsOnly(written,) ? { runNumber: Number(written,), } : {}),
  };
}

/**
 Says whether a state systemd reported is one a run has not ended in.

 @param state - the state systemd answered, or empty text when it answered nothing

 @returns whether the run of a unit in that state is going

 @example
 ```ts
 const going = unitStateIsNotEnded({ state: 'deactivating', },);
 ```
 */
export function unitStateIsNotEnded({ state, }: { readonly state: string },): boolean {
  return UNIT_STATES_NOT_ENDED.has(state,);
}

/**
 The end of every systemd unit name.
 */
export const UNIT_SUFFIX = '.service';

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
   The number of the run folder the start that made this unit claimed, when its name carries one.
   */
  readonly runNumber?: number;

  /**
   The state the manager reports for the unit, which is a state its run has not ended in.
   */
  readonly state: string;

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
 Reads the unit name of one start of a run.

 @param name - a run's name, or the unit name of a start that claimed no folder number

 @param runNumber - the number of the run folder that start claimed, which names its unit and no other start's

 @returns the unit name, with the prefix, the number and the suffix the recipe uses

 @example
 ```ts
 const unitName = unitNameOfGiven({ name: 'foo', runNumber: 3, },);
 ```
 */
export function unitNameOfGiven({
  name,
  runNumber,
}: {
  readonly name: string;
  readonly runNumber?: number;
},): string {
  /**
   The run's name as the caller gave it, without a unit suffix the caller may have added.

   A run's name is never read out of a unit's name here: `unitNameParts` does that, and every reader of a unit's name uses it. So a
   run named `pi-shared-x` gives the unit `pi-pi-shared-x@2.service`, which is the unit that start makes.
   */
  const bareName = name.trim()
    .replace(
    UNIT_SUFFIX,
    '',
    );
  return `${PI_UNIT_PREFIX}${bareName}${(runNumber === undefined) ? '' : `${PI_UNIT_NUMBER_SEPARATOR}${String(runNumber,)}`}${UNIT_SUFFIX}`;
}

/**
 Reads the run name a unit's name carries.

 @param unitName - the unit's name as systemd prints it

 @returns the run's name

 @example
 ```ts
 const runName = runNameOfUnit({ unitName: 'pi-foo@3.service', },);
 ```
 */
export function runNameOfUnit({
  unitName,
}: {
  readonly unitName: string;
},): string {
  return unitNameParts({ unitName, },)
    .name;
}

/**
 Reads the number of the run folder the start that made a unit claimed.

 @param unitName - the unit's name as systemd prints it

 @returns the number, or nothing when the unit's name carries none

 @example
 ```ts
 const number = runNumberOfUnit({ unitName: 'pi-foo@3.service', },);
 ```
 */
export function runNumberOfUnit({
  unitName,
}: {
  readonly unitName: string;
},): { readonly number?: number } {
  /**
   What the unit's name says, by the one parse every reader uses.
   */
  const parts = unitNameParts({ unitName, },);
  return ((parts.runNumber === undefined) ? {} : { number: parts.runNumber, });
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
  /**
   The state the manager reports for the unit.
   */
  const state = fields.get('ActiveState',) ?? '';
  if (
    (unitName === undefined)
      || (started === undefined)
      || (!unitStateIsNotEnded({ state, },))
  ) {
    return NOT_RUNNING;
  }
  /**
   The number of the run folder that unit's start claimed, when its name carries one.
   */
  const { number: runNumber, } = runNumberOfUnit({ unitName, },);
  return {
    name: runNameOfUnit({ unitName, },),
    unitName,
    elapsedMilliseconds: (nowMicroseconds - Number(started,)) / MICROSECONDS_PER_MILLISECOND,
    mainPid: Number(fields.get('MainPID',) ?? 0,),
    state,
    ...((runNumber === undefined) ? {} : { runNumber, }),
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

/**
 Reads every unit the manager holds for one run's name, with the state it reports for each.

 The manager is asked for every Pi unit at once, `pi-*.service`, and each answer is read by `unitNameParts`, so the question carries no
 run's name into a glob: a name that begins with `pi-` and a name that carries `@` are both read as the run's own name, and a unit
 started without a number is found beside the numbered ones. A state in {@link UNIT_STATES_NOT_ENDED} is a unit whose run is going,
 which is the answer `checks-before-a-start.ts` and `remove-finished-worktrees.ts` both ask for.

 @param run - how a command is run, which a test replaces

 @param runName - the run's name

 @returns each unit of that name the manager holds, with its state, in the order it printed them

 @example
 ```ts
 const units = await unitsOfARunName({ run: runCommandWithChildProcess, runName: 'copy-route', },);
 ```
 */
export async function unitsOfARunName({
  run,
  runName,
}: {
  readonly run: RunCommand;
  readonly runName: string;
}): Promise<readonly {
  readonly unitName: string;
  readonly state: string
}[]> {
  /**
   What the manager printed for every Pi unit it holds, ended or not.
   */
  const shown = await run({
    command: 'systemctl',
    words: [
      '--user',
      'list-units',
      '--all',
      '--no-legend',
      '--plain',
      `${PI_UNIT_PREFIX}*${UNIT_SUFFIX}`,
    ],
  },);
  // A manager that could not answer names no unit, which is what an empty answer gives as well: refusing a run because a question
  // failed would end work that could have gone ahead.
  if (shown.status !== 0) {
    return [];
  }
  /**
   Every unit whose run name is this run's, with its state.
   */
  const units: {
    unitName: string;
    state: string
  }[] = [];
  for (const line of shown.stdout
    .split(
    '\n',
    )) {
    /**
     The unit's name, which is the line's first word.
     */
    const unitName = firstWordOf(line,);
    if ((unitName === '') || (unitNameParts({ unitName, },)
      .name
      !== runName)) {
      continue;
    }
    /**
     The state the manager reports, which is the line's third column.
     */
    const state = line.trim()
      .split(' ',)
      .filter(function isAWord(word,) {
        return word !== '';
      },)[2]
      ?? '';
    units.push({
      unitName,
      state,
    },);
  }
  return units;
}

/**
 Reads the run names whose unit is in a state its run has not ended in, from one listing of every Pi unit.

 This is the answer `remove-finished-worktrees.ts` asks once for the whole machine, so a worktree is kept while any unit of its run's
 name is starting, running or stopping.

 @param run - how a command is run, which a test replaces

 @returns the run names whose unit has not ended, each once

 @example
 ```ts
 const going = await runNamesWithAUnitNotEnded({ run: runCommandWithChildProcess, },);
 ```
 */
export async function runNamesWithAUnitNotEnded({
  run,
}: {
  readonly run: RunCommand;
}): Promise<ReadonlySet<string>> {
  /**
   What the manager printed for every Pi unit it holds.
   */
  const shown = await run({
    command: 'systemctl',
    words: [
      '--user',
      'list-units',
      '--all',
      '--no-legend',
      '--plain',
      `${PI_UNIT_PREFIX}*${UNIT_SUFFIX}`,
    ],
  },);
  if (shown.status !== 0) {
    // A manager that cannot answer leaves every worktree alone rather than reading a live run as an ended one.
    throw new PiUnitsUnreadableError(
      `systemctl could not list the Pi units: ${shown.problem
        ?? shown.stderr
          .trim()}`,
    );
  }
  /**
   The run names with a unit whose run has not ended.
   */
  const names = new Set<string>();
  for (const line of shown.stdout
    .split(
    '\n',
    )) {
    /**
     The unit's name and the state the manager reports for it.
     */
    const unitName = firstWordOf(line,);
    /**
     The state the manager reports, which is the line's third column.
     */
    const state = line.trim()
      .split(' ',)
      .filter(function isAWord(word,) {
        return word !== '';
      },)[2]
      ?? '';
    if ((unitName !== '') && (!unitStateIsNotEnded({ state, },))) {
      continue;
    }
    if (unitName !== '') {
      names.add(unitNameParts({ unitName, },)
        .name,);
    }
  }
  return names;
}

/**
 Says whether any unit of one run's name is in a state its run has not ended in.

 @param run - how a command is run

 @param runName - the run's name

 @returns whether a run of that name is going

 @example
 ```ts
 const going = await aRunOfThatNameIsGoing({ run, runName: 'copy-route', },);
 ```
 */
export async function aRunOfThatNameIsGoing({
  run,
  runName,
}: {
  readonly run: RunCommand;
  readonly runName: string;
}): Promise<boolean> {
  /**
   Every unit of that run's name, with its state.
   */
  const units = await unitsOfARunName({
    run,
    runName,
  },);
  return units.some(function isNotEnded(unit,) {
    return unitStateIsNotEnded({ state: unit.state, },);
  },);
}

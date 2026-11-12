/**
 The launchers that are still running although the unit they started has ended: they wait on a waiter that never hears the unit
 end, they make the pool gate answer `ALREADY RUNNING` for that run's name, and they hold that run's slot until someone ends them.

 A launcher is a process whose command line names one of this package's launcher entries, and its unit is `pi-<the name it was
 given>`. The process table, the unit's state and the signal are the seams: a test hands over what `ps` would have printed, what
 systemd would have said, and records what it would have signalled, so no test signals a process of the machine it runs on.

 @module
 */

import { basename, } from 'node:path';
import type { ProcessUnderARun, } from './processes-under-a-pi-run.ts';
import { SECONDS_PER_MINUTE, } from './overdue-constants.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';

/**
 The signal a launcher past its unit is ended with: the polite one, so a launcher holding a file open still closes it.
 */
export const STOP_SIGNAL = 'SIGTERM';

/**
 Seconds a launcher must have been running before it counts as one that outlived its unit, when the caller names no age. A
 launcher makes a worktree and links dependencies before its unit exists, which takes minutes, so an hour is a bound rather than a
 measurement: below it, a launcher whose unit has not been made yet reads as one whose unit has ended.
 */
export const LAUNCHER_MINIMUM_AGE_SECONDS = 3_600;

/**
 One launcher process, as the process table describes it.
 */
export type LauncherProcess = {
  /**
   The name of the run it started.
   */
  readonly name: string;

  /**
   The launcher's process number.
   */
  readonly pid: number;

  /**
   The process number of the process that started it.
   */
  readonly parentPid: number;

  /**
   How long it has been running, in seconds.
   */
  readonly elapsedSeconds: number;

  /**
   Its command line.
   */
  readonly command: string;
};

/**
 Reads the words of a command line, which `ps` prints joined by single spaces.

 @param command - the command line, as `ps` printed it

 @returns the words it holds, empty for a blank line

 @example
 ```ts
 const words = wordsInCommand({ command: 'node /runs/launcher.ts --name=a', },);
 ```
 */
export function wordsInCommand({ command, }: { readonly command: string },): readonly string[] {
  return command.split(' ',)
    .filter(function isAWord(word,) {
      return word !== '';
    },);
}

/**
 Reads the value of one option out of the words of a command line.

 @param words - the command line's words

 @param option - the option, with its equals sign

 @returns the value, or empty text when the words do not carry that option

 @example
 ```ts
 const name = optionWordOf({ words: ['node', 'launcher.ts', '--name=a',], option: '--name=', },);
 ```
 */
export function optionWordOf({
  words,
  option,
}: {
  readonly words: readonly string[];
  readonly option: string;
},): string {
  return words.find(function startsWithOption(word,) {
    return word.startsWith(option,);
  },)
    ?.slice(option.length,)
    ?? '';
}

/**
 Reads every launcher process the table holds.

 @param table - the process table, as `ps` printed it

 @param launcherEntryNames - the basenames a launcher is run as

 @returns one entry per launcher, in the order the table listed them

 @example
 ```ts
 const launchers = launchersIn({ table, launcherEntryNames: ['start-pi-run-in-a-worktree.ts'], },);
 ```
 */
export function launchersIn({
  table,
  launcherEntryNames,
}: {
  readonly table: readonly ProcessUnderARun[];
  readonly launcherEntryNames: readonly string[];
},): readonly LauncherProcess[] {
  return table.flatMap(function ofProcess(process,) {
    /**
     The words of its command line.
     */
    const words = wordsInCommand({ command: process.command, },);
    // A launcher's entry is a path, and its basename is what names it: `ps` prints the whole word the kernel holds.
    if (!words.some(function runsALauncher(word,) {
      return launcherEntryNames.includes(basename(word,),);
    },)) {
      return [];
    }
    /**
     The name of the run it started, which a launcher without one started nothing.
     */
    const name = optionWordOf({
      words,
      option: '--name=',
    },);
    return (name === '')
      ? []
      : [{
        name,
        pid: process.pid,
        parentPid: process.parentPid,
        elapsedSeconds: process.elapsedSeconds,
        command: process.command,
      },];
  },);
}

/**
 Reads the processes that are children of a launcher.

 @param table - the process table, as `ps` printed it

 @param parentPid - the launcher's process number

 @returns its children, in the order the table listed them

 @example
 ```ts
 const children = childrenOf({ table, parentPid: 12, },);
 ```
 */
export function childrenOf({
  table,
  parentPid,
}: {
  readonly table: readonly ProcessUnderARun[];
  readonly parentPid: number;
},): readonly ProcessUnderARun[] {
  return table.filter(function isChild(process,) {
    return process.parentPid === parentPid;
  },);
}

/**
 One launcher whose unit has ended.
 */
export type LauncherPastItsUnit = {
  /**
   The launcher.
   */
  readonly launcher: LauncherProcess;

  /**
   The state systemd answered for its unit, or empty text when it answered nothing.
   */
  readonly unitState: string;

  /**
   The processes it started, which are its waiter and that waiter's own children.
   */
  readonly children: readonly ProcessUnderARun[];
};

/**
 Reads the launchers whose unit has ended, among those old enough to have made one.

 An answer that names no state is not an end: a manager that could not be asked says nothing about the unit, and a launcher it
 could not describe is not one this command may end.

 @param launchers - the launcher processes

 @param minimumAgeSeconds - how long a launcher must have been running to count

 @param unitStateOf - asks systemd what state a unit is in, which a test replaces

 @param stateIsRunning - says whether a state means the unit is still running

 @param childrenOfParent - reads the processes a launcher started

 @returns one entry per launcher whose unit has ended, and none for a launcher whose state could not be read

 @example
 ```ts
 const past = await launchersPastTheirUnits({ launchers, minimumAgeSeconds: 3_600, unitStateOf, stateIsRunning, childrenOfParent, },);
 ```
 */
export async function launchersPastTheirUnits({
  launchers,
  minimumAgeSeconds,
  unitStateOf,
  stateIsRunning,
  childrenOfParent,
}: {
  readonly launchers: readonly LauncherProcess[];
  readonly minimumAgeSeconds: number;
  readonly unitStateOf: (unitName: string) => Promise<string>;
  readonly stateIsRunning: (state: string) => boolean;
  readonly childrenOfParent: (pid: number) => readonly ProcessUnderARun[];
},): Promise<readonly LauncherPastItsUnit[]> {
  /**
   The launchers old enough to have made a unit.
   */
  const old = launchers.filter(function isOldEnough(launcher,) {
    return launcher.elapsedSeconds >= minimumAgeSeconds;
  },);
  /**
   The state of each of their units, asked all at once: one question is about one unit and waits on nothing else.
   */
  const states = await Promise.all(old.map(function ofLauncher(launcher,) {
    return unitStateOf(unitNameOfGiven({ name: launcher.name, },),);
  },));
  return old.flatMap(function ofState(
    launcher,
    index,
  ) {
    /**
     The state systemd answered for its unit.
     */
    const unitState = states[index] ?? '';
    return ((unitState === '') || stateIsRunning(unitState,))
      ? []
      : [{
        launcher,
        unitState,
        children: childrenOfParent(launcher.pid,),
      },];
  },);
}

/**
 Says one launcher past its unit as the line a reader acts on.

 @param launcher - the launcher process

 @param unitState - the state systemd answered for its unit

 @param children - the processes it started

 @returns the line: the name, the unit's state, the launcher's age and its children

 @example
 ```ts
 const line = launcherLineOf({ launcher, unitState: 'inactive', children: [], },);
 ```
 */
export function launcherLineOf({
  launcher,
  unitState,
  children,
}: LauncherPastItsUnit,): string {
  /**
   What each child runs, as `pid:program`.
   */
  const childText = children.map(function described(child,) {
    /**
     The words of the child's command line.
     */
    const words = wordsInCommand({ command: child.command, },);
    return `${String(child.pid,)}:${words[0]
      ?.split('/')
      .at(-1,)
      ?? '?'}`;
  },)
    .join(' ',);
  return `${launcher.name}: unit ${unitState}, launcher ${String(launcher.pid,)} started ${
    String(Math.round(launcher.elapsedSeconds / SECONDS_PER_MINUTE,),)
  } min ago, children ${(childText === '') ? 'none' : childText}`;
}

/**
 Reads the processes one launcher past its unit is ended by, children first.

 A launcher's waiter follows the unit, and ending the launcher alone can leave the waiter holding the unit's name in its parent's
 place, so each child is signalled before the launcher itself.

 @param launcher - the launcher process

 @param children - the processes it started

 @returns the process numbers to signal, in that order

 @example
 ```ts
 const targets = stopTargetsOf({ launcher, unitState: 'inactive', children: [], },);
 ```
 */
export function stopTargetsOf({
  launcher,
  children,
}: LauncherPastItsUnit,): readonly number[] {
  return [
    ...children.map(function pidOf(child,) {
      return child.pid;
    },),
    launcher.pid,
  ];
}

/**
 Ends one process, whichever way the caller signals.

 @param pid - the process number

 @param signal - the signal's name

 @example
 ```ts
 signalProcess({ pid: 12, signal: STOP_SIGNAL, });
 ```
 */
export type SignalProcess = (request: {
  readonly pid: number;
  readonly signal: string;
}) => void;

/**
 Says what a signal that failed to reach a process reports.

 @param pid - the process number

 @param caught - what the signal threw

 @returns the line to print

 @example
 ```ts
 const line = signalFailureLineOf({ pid: 12, caught: new Error('no such process'), },);
 ```
 */
export function signalFailureLineOf({
  pid,
  caught,
}: {
  readonly pid: number;
  readonly caught: unknown;
},): string {
  return `  could not signal ${String(pid,)}: ${String(caught,)}`;
}

/**
 Ends every launcher past its unit, children first, and says what could not be signalled.

 @param past - the launchers to end

 @param signal - how one process is signalled, which a test replaces

 @returns one line per process that could not be signalled

 @example
 ```ts
 const failures = stopLaunchers({ past, signal: signalProcess, },);
 ```
 */
export function stopLaunchers({
  past,
  signal,
}: {
  readonly past: readonly LauncherPastItsUnit[];
  readonly signal: SignalProcess;
},): readonly string[] {
  /**
   The lines about what could not be signalled.
   */
  const failures: string[] = [];
  for (const launcher of past) {
    for (const pid of stopTargetsOf(launcher,)) {
      try {
        signal({
          pid,
          signal: STOP_SIGNAL,
        },);
      }
      catch (caught) {
        failures.push(signalFailureLineOf({
          pid,
          caught,
        },),);
      }
    }
  }
  return failures;
}

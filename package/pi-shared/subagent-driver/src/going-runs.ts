/**
 The Pi runs that are going now: the launchers that have not made their unit yet, and the units systemd lists, with the pool
 each holds a slot in.

 A launcher takes minutes to make its unit, so a count of units alone once allowed a twenty-first run. A unit whose launcher has
 gone still names its provider and model on its own command line, so it is read from there; counting every such unit in one
 fallback pool, as the scratch launcher did before 2026-11-11, let runs sit on standard Neuralwatt while it read empty.

 @module
 */

import {
  basename,
  join,
} from 'node:path';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import type { RunCommand, } from './run-command.ts';
import {
  PiUnitsUnreadableError,
} from './running-pi-units.ts';
import {
  type PoolMembership,
  membershipOf,
  poolNamed,
} from './pool-limits.ts';
import { optionIn, } from './option-in.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  firstWordOf,
  isDigitsOnly,
  valueOfOption,
} from './text-of-an-option.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The option that names the provider a run asks for.
 */
const PROVIDER_OPTION = '--provider=';

/**
 The option that names the model a run asks for.
 */
const MODEL_OPTION = '--model=';

/**
 The option that names the run.
 */
const NAME_OPTION = '--name=';

/**
 The unit properties a Pi run's unit is read for, which is where a unit whose launcher has gone names its pool.
 */
const EXEC_START_PROPERTY = '--property=ExecStart';

/**
 The end of every systemd unit name.
 */
const UNIT_SUFFIX = '.service';

/**
 The start of every Pi run's unit name.
 */
const PI_UNIT_PREFIX = 'pi-';

/**
 The basenames this package's worktree launcher is run as: its TypeScript entry, and the built entry of the same name.
 */
export const LAUNCHER_ENTRY_NAMES: readonly string[] = [
  'start-pi-run-in-a-worktree.ts',
  'start-pi-run-in-a-worktree.mjs',
];

/**
 The basename of the scratch launcher this command replaced, still counted so that a run the coordinator started before the
 switch holds its slot. Remove it once no launcher of that name is running.
 */
export const REPLACED_LAUNCHER_ENTRY_NAMES: readonly string[] = [
  'start-pi-run-in-worktree.ts',
];

/**
 One Pi run that is going now, and the pool it holds a slot in.
 */
export type GoingRun = {
  /**
   The run's name, which is also the start of its run folder's name.
   */
  readonly name: string;

  /**
   The pool this run holds a slot in; a membership that is in no pool claims no slot anywhere.
   */
  readonly membership: PoolMembership;
};

/**
 Reads the pool a launcher's own command line names.

 @param commandLine - the launcher's arguments, as its process was given them

 @returns which pool the run it started holds a slot in

 @example
 ```ts
 const membership = membershipOfCommandLine({ commandLine: ['node', 'launcher.ts', '--name=a',], },);
 ```
 */
function membershipOfCommandLine(commandLine: readonly string[],): PoolMembership {
  /**
   The pool the launcher's own options name.
   */
  const pool = poolNamed({
    provider: optionIn({
      commandLine,
      option: PROVIDER_OPTION,
    },),
    model: optionIn({
      commandLine,
      option: MODEL_OPTION,
    },),
  },);
  return membershipOf({
    runName: optionIn({
      commandLine,
      option: NAME_OPTION,
    },),
    ...(pool.holds === 'a-pool' ? { pool, } : {}),
  },);
}

/**
 Reads one process's command line, or nothing when the process ended between the listing and the read.

 @param path - the process's own command line file

 @returns the words the kernel split the line into, or nothing at all

 @throws Error when the read failed for any reason other than a process that is gone

 @example
 ```ts
 const words = await commandLineOrNothingIn('/proc/123/cmdline',);
 ```
 */
async function commandLineOrNothingIn(path: string,): Promise<readonly string[]> {
  try {
    return (await readFile(
      path,
      'utf8',
    )).split('\0',);
  }
  catch (error) {
    /**
     The code the operating system gave the read.
     */
    const code = errorCodeOf(error,);
    if ((code === MISSING_PATH_CODE) || (code === 'ESRCH')) {
      return [];
    }
    throw error;
  }
}

/**
 Reads the Pi runs whose launcher is still running, before their unit exists.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param launcherEntryNames - the basenames a launcher is run as

 @returns one entry per launcher that names a run

 @example
 ```ts
 const launchers = await launcherRunsIn({ processFolder: '/proc', launcherEntryNames: LAUNCHER_ENTRY_NAMES, },);
 ```
 */
export async function launcherRunsIn({
  processFolder,
  launcherEntryNames,
}: {
  readonly processFolder: string;
  readonly launcherEntryNames: readonly string[];
}): Promise<readonly GoingRun[]> {
  /**
   The entries of the process folder, which is one per process number.
   */
  const entries = await readdir(processFolder,);
  /**
   The launchers found so far.
   */
  const runs: GoingRun[] = [];
  for (const entry of entries) {
    if (!isDigitsOnly(entry,)) {
      continue;
    }
    /**
     The launcher's command line, split where the kernel splits it, or nothing when the process ended between the listing and
     the read.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one process after another, and a launcher's line is only readable while its process lives; see doc/lint/sequential-awaits.md
    const commandLine = await commandLineOrNothingIn(join(
      processFolder,
      entry,
      'cmdline',
    ),);
    if (commandLine.length === 0) {
      continue;
    }
    /**
     The entry point the process runs, which is the second word Node and a shebang script both leave in place.
     */
    const [, entryPoint,] = commandLine;
    if ((entryPoint === undefined) || (!launcherEntryNames.includes(basename(entryPoint,),))) {
      continue;
    }
    /**
     The run this launcher names.
     */
    const name = optionIn({
      commandLine,
      option: NAME_OPTION,
    },);
    if (name === '') {
      continue;
    }
    runs.push({
      name,
      membership: membershipOfCommandLine(commandLine,),
    },);
  }
  return runs;
}

/**
 Reads the run names systemd lists as going now.

 @param shown - what `systemctl list-units` printed

 @returns the run names, in the order systemd printed them

 @example
 ```ts
 const names = unitRunNamesIn({ shown: 'pi-a.service loaded active running x', },);
 ```
 */
export function unitRunNamesIn({ shown, }: { readonly shown: string },): readonly string[] {
  return shown.split('\n',)
    .map(function wordOf(line,) {
      return firstWordOf(line,);
    },)
    .filter(function isPiUnit(unit,) {
      return unit.startsWith(PI_UNIT_PREFIX,) && unit.endsWith(UNIT_SUFFIX,);
    },)
    .map(function nameOf(unit,) {
      return unit.slice(
        PI_UNIT_PREFIX.length,
        -UNIT_SUFFIX.length,
      );
    },);
}

/**
 Reads the pool a unit's own command line names.

 @param shown - what `systemctl show --property=ExecStart --value` printed for the unit

 @returns which pool the run holds a slot in

 @example
 ```ts
 const membership = poolOfExecStart({ shown: '{ path=node ; argv[]=node x.ts --provider=neuralwatt --model=deepseek-v4.1-flash ; }', },);
 ```
 */
export function poolOfExecStart({ shown, }: { readonly shown: string },): PoolMembership {
  /**
   The provider named on the unit's command line.
   */
  const provider = valueOfOption({
    text: shown,
    option: '--provider',
  },);
  /**
   The model named on the unit's command line.
   */
  const model = valueOfOption({
    text: shown,
    option: '--model',
  },);
  if ((provider === '') || (model === '')) {
    return { holds: 'no-pool', };
  }
  return poolNamed({
    provider,
    model,
  },);
}

/**
 Reads every Pi run going now, from the launchers and from systemd.

 A unit whose own command line could not be read holds a slot in every pool its name could be running on, which is what the
 caller counts: a guessed single pool read that one pool full and the others empty.

 @param run - how a command is run, which a test replaces

 @param processFolder - the folder holding one entry per process

 @param launcherEntryNames - the basenames a launcher is run as

 @param onProblem - told about a unit whose own command line could not be read

 @returns one entry per run going now

 @throws PiUnitsUnreadableError when systemd could not list the units

 @example
 ```ts
 const runs = await goingRunsNow({ run, processFolder: '/proc', launcherEntryNames: LAUNCHER_ENTRY_NAMES, onProblem, },);
 ```
 */
export async function goingRunsNow({
  run,
  processFolder,
  launcherEntryNames,
  onProblem,
}: {
  readonly run: RunCommand;
  readonly processFolder: string;
  readonly launcherEntryNames: readonly string[];
  readonly onProblem: (problem: string) => void;
}): Promise<readonly GoingRun[]> {
  /**
   The launchers that have not made their unit yet.
   */
  const launchers = await launcherRunsIn({
    processFolder,
    launcherEntryNames,
  },);
  /**
   The runs the launchers already account for.
   */
  const runs = new Map(launchers.map(function keyed(launcher,) {
    return [
      launcher.name,
      launcher,
    ] as const;
  },),);
  /**
   What systemd said about the Pi units.
   */
  const listed = await run({
    command: 'systemctl',
    words: [
      '--user',
      'list-units',
      '--state=active,activating',
      '--no-legend',
      '--plain',
      `${PI_UNIT_PREFIX}*${UNIT_SUFFIX}`,
    ],
  },);
  if (listed.status !== 0) {
    throw new PiUnitsUnreadableError(
      `systemctl could not list the Pi units: ${listed.problem
        ?? listed.stderr
          .trim()}`,
    );
  }
  for (const name of unitRunNamesIn({ shown: listed.stdout, },)) {
    if (runs.has(name,)) {
      continue;
    }
    /**
     What the unit's own command line says about its pool.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one unit after another: each question is about a different unit, and asking them all at once would leave the counts of a listing nobody can tie to a moment; see doc/lint/sequential-awaits.md
    const shown = await run({
      command: 'systemctl',
      words: [
        '--user',
        'show',
        `${PI_UNIT_PREFIX}${name}${UNIT_SUFFIX}`,
        EXEC_START_PROPERTY,
        '--value',
      ],
    },);
    if (shown.status !== 0) {
      onProblem(`pool of unit ${PI_UNIT_PREFIX}${name}${UNIT_SUFFIX} not read: ${shown.problem
        ?? shown.stderr
          .trim()}`);
    }
    runs.set(
      name,
      {
        name,
        membership: (shown.status === 0)
          ? poolOfExecStart({ shown: shown.stdout, },)
          : membershipOf({ runName: name, },),
      },
    );
  }
  return [...runs.values(),];
}

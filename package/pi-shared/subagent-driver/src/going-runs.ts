/**
 The Pi runs that are going now: the launchers that have not made their unit yet, and the units systemd lists, with the pool
 each holds a slot in.

 A launcher takes minutes to make its unit, so a count of units alone once allowed a twenty-first run. A unit whose launcher has
 gone still names its provider and model on its own command line, so it is read from there; counting every such unit in one
 fallback pool, as the scratch launcher did before 2026-11-11, let runs sit on standard Neuralwatt while it read empty.

 The entries are one per run name, which is what a pool counts and what a second start of one name is refused against; each
 entry also says how many agents the name stands for, because the machine's cap counts agents and two runs that share a name are
 two agents. A launcher and a unit are joined as one run only when the launcher's own `systemd-run` client is still waiting on
 that very unit: two runs may share a name, because an audit name may go twice, and a unit whose launcher has gone beside a
 launcher of the same name is two agents, not one.

 @module
 */

import { basename, } from 'node:path';
import type { RunCommand, } from './run-command.ts';
import {
  NO_PROCESS,
  type ProcessGoing,
  parentPidOf,
  processesGoingIn,
} from './processes-of-the-table.ts';
import {
  PiUnitsUnreadableError,
  unitNameOfGiven,
} from './running-pi-units.ts';
import {
  type PoolMembership,
  type ProviderPool,
  PROVIDER_POOLS,
  membershipOf,
  poolNamed,
} from './pool-limits.ts';
import { optionIn, } from './option-in.ts';
import {
  firstWordOf,
  valueOfOption,
} from './text-of-an-option.ts';

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
 The program that starts a unit and waits on it, as a launcher runs it: a process of that name whose parent is the launcher and
 whose command line names the unit is what proves the unit is that launcher's own run.
 */
const UNIT_WAITER_PROGRAM = 'systemd-run';

/**
 The option that names the unit a waiting `systemd-run` client started.
 */
const UNIT_OPTION = '--unit=';

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
 One Pi run that is going now, the pool it holds a slot in, and how many agents it stands for.
 */
export type GoingRun = {
  /**
   The run's name, which is also the start of its run folder's name.
   */
  readonly name: string;

  /**
   How many agents this run name stands for: one for a launcher or for a unit, one when a launcher and the unit it made share
   the name, and two when two runs are going under one name, which an audit name may do and a race can leave for a moment.
   A launcher counts for the unit only when its own `systemd-run` client is still waiting on that unit; a unit no launcher proves
   is another agent. A pool counts the entry once; the machine's cap counts these agents, so two runs sharing a name are never one.
   */
  readonly agents: number;

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
function membershipOfCommandLine({
  commandLine,
  pools = PROVIDER_POOLS,
}: {
  readonly commandLine: readonly string[];
  readonly pools?: readonly ProviderPool[];
},): PoolMembership {
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
    pools,
    ...(pool.holds === 'a-pool' ? { pool, } : {}),
  },);
}

/**
 One launcher going now: the run it names, and its process number, which is what proves a unit it waits on.
 */
type LauncherGoing = {
  /**
   The run the launcher names.
   */
  readonly run: GoingRun;

  /**
   The launcher's process number.
   */
  readonly pid: number;
};

/**
 Reads the unit a `systemd-run` client is waiting on.

 @param words - the words of one process's command line

 @returns the unit's name, or empty text when the process is no client waiting on a unit

 @example
 ```ts
 const unitName = unitAwaitedBy({ words: ['systemd-run', '--unit=pi-a.service',], },);
 ```
 */
function unitAwaitedBy({ words }: { readonly words: readonly string[] }): string {
  /**
   The program the process runs, which a waiting client spells as the launcher started it.
   */
  const [program,] = words;
  if ((program === undefined) || (basename(program,) !== UNIT_WAITER_PROGRAM)) {
    return '';
  }
  return optionIn({
    commandLine: words,
    option: UNIT_OPTION,
  },);
}

/**
 Reads the unit each launcher is waiting on, which is what proves a listed unit is that launcher's own run.

 The launcher starts its unit as `systemd-run --user --wait --unit=pi-<name>.service` and stays that client's parent until the
 unit ends, so a live client naming a unit whose parent is a launcher of that name says the unit is that launcher's run.

 Only a unit that exactly one live client names can be that client's: a second client naming an existing unit was refused by the
 manager (`Unit pi-<name>.service was already loaded or has a fragment file`), and while it lives it names the unit and has the
 second launcher as its parent without having made anything. With two clients naming one unit, the one that made it cannot be told
 from the one that was refused, so the unit is counted as another agent for the moment both live: an overcount refuses one start,
 where joining them would let one through. A unit no launcher proves is another agent, because a unit of one run and a launcher of
 another may share a name: an audit name may go twice.

 @param processes - every process of the folder

 @param launchers - the launchers going now, with their process numbers

 @param processFolder - the folder holding one entry per process

 @returns the names of the units a launcher proved to be its own run

 @example
 ```ts
 const proved = await unitsLaunchersWaitOn({ processes, launchers, processFolder: '/proc', },);
 ```
 */
async function unitsLaunchersWaitOn({
  processes,
  launchers,
  processFolder,
}: {
  readonly processes: readonly ProcessGoing[];
  readonly launchers: readonly LauncherGoing[];
  readonly processFolder: string;
},): Promise<ReadonlySet<string>> {
  /**
   Every live `systemd-run` client: the unit it waits on, and the process that started it.
   */
  const clients: {
    readonly unitName: string;
    readonly parent: number;
  }[] = [];
  for (const process of processes) {
    /**
     The unit this process waits on, when it is a `systemd-run` client.
     */
    const unitName = unitAwaitedBy({ words: process.words, },);
    if (unitName === '') {
      continue;
    }
    /**
     The process that started this client, or {@link NO_PROCESS} when that cannot be read.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one waiting client after another, and a parent is read only while the client lives; see doc/lint/sequential-awaits.md
    const parent = await parentPidOf({
      processFolder,
      pid: process.pid,
    },);
    clients.push({
      unitName,
      parent,
    },);
  }
  /**
   The units proved to be a launcher's own run.
   */
  const proved = new Set<string>();
  for (const unitName of new Set(clients.map(function unitOf(client,) {
    return client.unitName;
  },))) {
    /**
     The processes that started the clients naming this unit.
     */
    const parents = clients.filter(function waitsOn(client,) {
      return client.unitName === unitName;
    },)
      .map(function startedBy(client,) {
        return client.parent;
      },);
    if (parents.length !== 1) {
      // Two clients name one unit: one of them was refused, and which made the unit cannot be told from here.
      continue;
    }
    /**
     The process that started the one client naming this unit.
     */
    const [parent,] = parents;
    /**
     The launcher that started it, when one of that name did.
     */
    const launcher = launchers.find(function startedIt(candidate,) {
      /**
       The run the candidate launcher names, and the unit its name makes.
       */
      const { name: candidateName, } = candidate.run;
      return (candidate.pid === parent)
        && (unitNameOfGiven({ name: candidateName, },) === unitName);
    },);
    if (launcher !== undefined) {
      proved.add(unitName,);
    }
  }
  return proved;
}

/**
 Reads the launcher processes among the processes of the folder: one entry per process that names a run.

 @param processes - every process of the folder

 @param launcherEntryNames - the basenames a launcher is run as

 @param pools - the pools to read a launcher's own options against

 @returns one entry per launcher that names a run

 @example
 ```ts
 const launchers = launchersAmong({ processes, launcherEntryNames: LAUNCHER_ENTRY_NAMES, },);
 ```
 */
function launchersAmong({
  processes,
  launcherEntryNames,
  pools = PROVIDER_POOLS,
}: {
  readonly processes: readonly ProcessGoing[];
  readonly launcherEntryNames: readonly string[];
  readonly pools?: readonly ProviderPool[];
}): readonly LauncherGoing[] {
  /**
   The launchers found so far.
   */
  const launchers: LauncherGoing[] = [];
  for (const process of processes) {
    /**
     The entry point the process runs, which is the second word Node and a shebang script both leave in place.
     */
    const [, entryPoint,] = process.words;
    if ((entryPoint === undefined) || (!launcherEntryNames.includes(basename(entryPoint,),))) {
      continue;
    }
    /**
     The run this launcher names.
     */
    const name = optionIn({
      commandLine: process.words,
      option: NAME_OPTION,
    },);
    if (name === '') {
      continue;
    }
    launchers.push({
      pid: process.pid,
      run: {
        name,
        agents: 1,
        membership: membershipOfCommandLine({
          commandLine: process.words,
          pools,
        },),
      },
    },);
  }
  return launchers;
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
  pools = PROVIDER_POOLS,
}: {
  readonly processFolder: string;
  readonly launcherEntryNames: readonly string[];
  readonly pools?: readonly ProviderPool[];
}): Promise<readonly GoingRun[]> {
  return launchersAmong({
    processes: await processesGoingIn({ processFolder, },),
    launcherEntryNames,
    pools,
  },)
    .map(function runOf(launcher,) {
      return launcher.run;
    },);
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

 @returns one entry per run name going now, each saying how many agents it stands for

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
  pools = PROVIDER_POOLS,
  onProblem,
}: {
  readonly run: RunCommand;
  readonly processFolder: string;
  readonly launcherEntryNames: readonly string[];
  readonly pools?: readonly ProviderPool[];
  readonly onProblem: (problem: string) => void;
}): Promise<readonly GoingRun[]> {
  /**
   Every process whose command line could be read, which is where the launchers and the clients waiting on their units are found.
   */
  const processes = await processesGoingIn({ processFolder, },);
  /**
   The launchers that have not made their unit yet.
   */
  const launchers = launchersAmong({
    processes,
    launcherEntryNames,
    pools,
  },);
  /**
   The runs the launchers account for, one entry per name, each with the agents going under that name: two launchers of one name
   are two agents. A unit is folded into its launcher's entry only when that launcher is proved to be waiting on it.
   */
  const runs = new Map<string, GoingRun>();
  for (const launcher of launchers) {
    /**
     The run this launcher names, which is the entry it joins.
     */
    const { name, } = launcher.run;
    /**
     The entry this name already holds, when a launcher of it was seen.
     */
    const held = runs.get(name,);
    runs.set(
      name,
      (held === undefined)
        ? launcher.run
        : {
          ...held,
          agents: held.agents + 1,
        },
    );
  }
  /**
   The units a launcher proved to be its own run, by the client it is still waiting with.
   */
  const provedUnits = await unitsLaunchersWaitOn({
    processes,
    launchers,
    processFolder,
  },);
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
    /**
     The unit's own name, which is what a launcher's waiting client names.
     */
    const unitName = `${PI_UNIT_PREFIX}${name}${UNIT_SUFFIX}`;
    /**
     The entry this run name already holds, when a launcher of it is going.
     */
    const held = runs.get(name,);
    if ((held !== undefined) && provedUnits.has(unitName,)) {
      // The launcher is proved to be waiting on this very unit, so it is that one run, already counted.
      continue;
    }
    if (held !== undefined) {
      // A launcher of this name is going and no launcher proves this unit, so the two are two runs: one more agent under this
      // name, in the one entry a pool counts for it. The entry keeps the launcher's own pool membership.
      runs.set(
        name,
        {
          ...held,
          agents: held.agents + 1,
        },
      );
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
        unitName,
        EXEC_START_PROPERTY,
        '--value',
      ],
    },);
    if (shown.status !== 0) {
      onProblem(`pool of unit ${unitName} not read: ${shown.problem
        ?? shown.stderr
          .trim()}`);
    }
    runs.set(
      name,
      {
        name,
        agents: 1,
        membership: (shown.status === 0)
          ? poolOfExecStart({ shown: shown.stdout, },)
          : membershipOf({
            runName: name,
            pools,
          },),
      },
    );
  }
  return [...runs.values(),];
}

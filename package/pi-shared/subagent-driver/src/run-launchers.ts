/**
 The starts of Pi runs still in flight, read from the command lines of the processes on this machine: a run whose unit systemd
 has not listed yet already holds its slot.

 A start is counted from the moment its command exists, which is the behaviour the scratch scripts had before there was a
 package for it: a count of units alone once allowed a twenty-first run on a pool of twenty, because a new run's unit takes
 minutes to appear. The command line is read; nothing else about the process is, and nothing read is ever printed.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  basename,
  extname,
  join,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isWholeNumberText, } from './whole-number-text.ts';
import {
  type NO_POOL,
  type SlotPool,
  poolNamed
} from './pools-of-pi-runs.ts';

/**
 An entry of the process table that is no process with a command line, as a symbol so no process can be mistaken for it.
 */
const NOT_A_PROCESS: unique symbol = Symbol('the entry is no process with a command line',);

/**
 What a command line is read as when the process is not a start of a pi run.
 */
export const NOT_A_LAUNCHER: unique symbol = Symbol('the command line is not a start of a pi run',);

/**
 The name of the command that starts a run, without its folder or extension. A process running it with a `--name=` argument is
 one start in flight.
 */
export const STARTER_COMMAND_NAME = 'start-pi-run-in-a-free-slot';

/**
 The prefix of the option that names the run a start will perform.
 */
const NAME_OPTION = '--name=';

/**
 The prefix of the option that names the provider a start will use.
 */
const PROVIDER_OPTION = '--provider=';

/**
 The prefix of the option that names the model a start will use.
 */
const MODEL_OPTION = '--model=';

/**
 The name of the file a process's command line is kept in, inside the folder of its process number.
 */
const COMMAND_LINE_FILE = 'cmdline';

/**
 The character a process's arguments are joined with, and the byte its command line ends with.
 */
const ARGUMENT_SEPARATOR = '\u{0}';

/**
 One process on this machine and the arguments it was started with.
 */
export type ProcessCommandLine = {
  /**
   The process's number, as its folder is named.
   */
  readonly processId: string;

  /**
   The arguments, the program included.
   */
  readonly commandLine: readonly string[];
};

/**
 A start of a run that is in flight.
 */
export type RunLauncher = {
  /**
   The run it will start.
   */
  readonly name: string;

  /**
   The pool its command line names, or {@link NO_POOL}.
   */
  readonly pool: SlotPool | typeof NO_POOL;
};

/**
 Reads the value of one option, which a command line writes as `--option=value`.

 @param commandLine - the process's arguments

 @param option - the option's prefix, with its equals sign

 @returns the value, or empty text when the command line does not carry the option

 @example
 ```ts
 const name = valueOfOption({ commandLine: ['node', 'a.ts', '--name=x',], option: '--name=', });
 ```
 */
function valueOfOption({
  commandLine,
  option,
}: {
  readonly commandLine: readonly string[];
  readonly option: string;
},): string {
  return commandLine.find(function startsWithTheOption(argument,) {
    return argument.startsWith(option,);
  },)
    ?.slice(option.length,)
    ?? '';
}

/**
 Names the command a process is running, by its file name without its extension.

 @param commandLine - the process's arguments

 @returns the command's name, or empty text when the command line is empty

 @example
 ```ts
 const command = commandNameOf({ commandLine: ['node', '/a/start-pi-run-in-a-free-slot.ts',], });
 ```
 */
export function commandNameOf({
  commandLine,
}: {
  readonly commandLine: readonly string[];
},): string {
  /**
   The program the process runs, which is `node` for every command of this package.
   */
  const program = commandLine[1] ?? '';
  return basename(
    program,
    extname(program,),
  );
}

/**
 Reads one process as a start of a run in flight.

 @param process - the process's number and arguments

 @param commandName - the name of the command that starts a run

 @returns the run it will start and the pool it names, or {@link NOT_A_LAUNCHER}

 @example
 ```ts
 const launcher = launcherIn({ process: { processId: '7', commandLine: ['node', 'x', '--name=a',], }, commandName: 'x', });
 ```
 */
export function launcherIn({
  process,
  commandName,
}: {
  readonly process: ProcessCommandLine;
  readonly commandName: string;
},): RunLauncher | typeof NOT_A_LAUNCHER {
  if (commandNameOf({ commandLine: process.commandLine, },) !== commandName) {
    return NOT_A_LAUNCHER;
  }
  /**
   The run the start will perform.
   */
  const name = valueOfOption({
    commandLine: process.commandLine,
    option: NAME_OPTION,
  },);
  if (name === '') {
    return NOT_A_LAUNCHER;
  }
  return {
    name,
    pool: poolNamed({
      provider: valueOfOption({
        commandLine: process.commandLine,
        option: PROVIDER_OPTION,
      },),
      model: valueOfOption({
        commandLine: process.commandLine,
        option: MODEL_OPTION,
      },),
    },),
  };
}

/**
 Reads every start of a run in flight, leaving this process out so a start never counts itself.

 @param processes - every process on this machine

 @param commandName - the name of the command that starts a run

 @param ownProcessId - this process's number

 @returns what each run in flight will be started with, by the run's name

 @example
 ```ts
 const launchers = launchersOf({ processes: [], commandName: 'x', ownProcessId: '1', });
 ```
 */
export function launchersOf({
  processes,
  commandName,
  ownProcessId,
}: {
  readonly processes: readonly ProcessCommandLine[];
  readonly commandName: string;
  readonly ownProcessId: string;
},): ReadonlyMap<string, SlotPool | typeof NO_POOL> {
  /**
   The starts found so far.
   */
  const launchers = new Map<string, SlotPool | typeof NO_POOL>();
  for (const process of processes) {
    if (process.processId === ownProcessId) {
      continue;
    }
    /**
     What this process is, when it is a start of a run at all.
     */
    const launcher = launcherIn({
      process,
      commandName,
    },);
    if ((typeof launcher) !== 'symbol') {
      launchers.set(
        launcher.name,
        launcher.pool,
      );
    }
  }
  return launchers;
}

/**
 Reads the processes of one process table.

 A process that ended between the listing and the read is left out: the operating system keeps its folder until whoever started
 it collects it, so a listing can name one that no longer has a command line.

 @param folder - the folder that holds one folder per process number

 @param ownProcessId - this process's number

 @returns every command line it could read

 @throws the error a read gave for a folder that is not a process's, rather than treating it as a process that ended

 @example
 ```ts
 const processes = await commandLinesUnderProcessFolder({ folder: '/proc', ownProcessId: '1', });
 ```
 */
export async function commandLinesUnderProcessFolder({
  folder,
  ownProcessId,
}: {
  readonly folder: string;
  readonly ownProcessId: string;
},): Promise<readonly ProcessCommandLine[]> {
  /**
   The entries of the process table.
   */
  const entries = await readdir(
    folder,
    { withFileTypes: true, },
  );
  /**
   Every process of the table that has a command line to read.
   */
  const processes = (await Promise.all(entries.map(async function commandLineOf(entry,) {
    if ((!entry.isDirectory()) || (!isWholeNumberText({ text: entry.name, }))) {
      return NOT_A_PROCESS;
    }
    try {
      /**
       What this process was started with.
       */
      const commandLine = await readFile(
        join(
          folder,
          entry.name,
          COMMAND_LINE_FILE,
        ),
        { encoding: 'utf8', },
      );
      return {
        processId: entry.name,
        commandLine: commandLine.split(ARGUMENT_SEPARATOR,),
      };
    }
    catch (error) {
      /**
       The operating system's own reason for the reading failing.
       */
      const code = errorCodeOf(error,);
      // A process that ended while the table was being read has no command line left to read, which is no failure of ours.
      if ((code === 'ENOENT') || (code === 'ESRCH')) {
        return NOT_A_PROCESS;
      }
      throw error;
    }
  },),))
    .filter(function isAProcess(process: ProcessCommandLine | typeof NOT_A_PROCESS,): process is ProcessCommandLine {
      return (typeof process) !== 'symbol';
    },);
  return processes.filter(function isAnotherProcess(process: ProcessCommandLine,) {
    return process.processId !== ownProcessId;
  },);
}

/**
 Reads every start of a run in flight now, from this machine's process table.

 @param commandName - the name of the command that starts a run

 @param ownProcessId - this process's number, which is left out

 @returns what each run in flight will be started with, by the run's name

 @example
 ```ts
 const launchers = await readRunLaunchers({ commandName: STARTER_COMMAND_NAME, ownProcessId: '1', });
 ```
 */
export async function readRunLaunchers({
  commandName,
  ownProcessId,
}: {
  readonly commandName: string;
  readonly ownProcessId: string;
},): Promise<ReadonlyMap<string, SlotPool | typeof NO_POOL>> {
  return launchersOf({
    processes: await commandLinesUnderProcessFolder({
      folder: '/proc',
      ownProcessId,
    },),
    commandName,
    ownProcessId,
  },);
}

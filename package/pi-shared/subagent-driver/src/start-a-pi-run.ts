/**
 Starts one run when its provider pool has a slot free: counts what is already going, chooses the pool, makes the run's worktree
 and run folder, sizes its turn cap, and starts its unit, answering with the unit's own status.

 The questions about the machine and the unit itself are handed in, so a test starts a run without a unit, a repository or a
 process table.

 @module
 */

import {
  mkdir,
  readFile,
  rmdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  inboxOfARun,
  runFolderPathOf,
} from './pi-runs-folders.ts';
import { PiRunNotStartedError, } from './pi-run-not-started-error.ts';
import { releaseFinishedWorktree, } from './release-finished-worktree.ts';
import {
  dependenciesIn,
  workingFolderIn,
} from './worktree-of-a-pi-run.ts';
import {
  ALREADY_RUNNING_EXIT_STATUS,
  countsOfPools,
  countsSummaryOf,
  hasRoom,
  NO_LAUNCHER,
  NO_POOL,
  NO_SLOT_EXIT_STATUS,
  poolNamed,
  poolWithRoom,
  runIsAlreadyGoing,
  type SlotPool,
  wallSecondsInPool,
} from './pools-of-pi-runs.ts';
import type {
  CommandOutput,
  RunCommand,
} from './run-command.ts';
import type { RunToTheEnd, } from './run-command-to-the-end.ts';
import type { RunningPiUnit, } from './running-pi-units.ts';
import {
  NO_WORKTREE_BRANCH,
  type StartPiRunOptions,
} from './start-pi-run-options.ts';
import type { SizedTurnCap, } from './turn-caps-from-finished-runs.ts';
import {
  unitEnvironmentOf,
  unitWordsOf,
} from './unit-of-a-pi-run.ts';

export { PiRunNotStartedError, } from './pi-run-not-started-error.ts';

/**
 What a start answers with, which is what its command prints and the status it exits with.
 */
export type StartOutcome = {
  /**
   The status to exit with: the unit's own, or the one that says why no unit was started.
   */
  readonly status: number;

  /**
   The lines to print, in the order they happen.
   */
  readonly lines: readonly string[];
};

/**
 Sizes a run's turn cap from what finished runs used, which a test replaces so it needs no history.
 */
export type SizeATurnCap = (request: {
  readonly runsFolder: string;
  readonly runName: string;
}) => Promise<SizedTurnCap>;

/**
 Everything a start needs from outside itself.
 */
export type StartAPiRunRequest = {
  /**
   What the caller asked for.
   */
  readonly options: StartPiRunOptions;

  /**
   How a short question of the machine is asked: git, mise, systemd's own listing.
   */
  readonly run: RunCommand;

  /**
   How the run's unit is started and waited for.
   */
  readonly runToTheEnd: RunToTheEnd;

  /**
   Every start of a run that is in flight now.
   */
  readonly launchers: () => Promise<ReadonlyMap<string, SlotPool | typeof NO_POOL>>;

  /**
   Every Pi unit systemd is running now.
   */
  readonly runningUnits: () => Promise<readonly RunningPiUnit[]>;

  /**
   How the turn cap is sized.
   */
  readonly sizeATurnCap: SizeATurnCap;

  /**
   The home directory the unit is given, which is where this machine's Pi and its configuration live.
   */
  readonly home: string;
};

/**
 The summary a finished attempt leaves last, which names the session a resume reads.
 */
const SUMMARY_FILE = 'summary.json';

/**
 Reads what one attempt left last.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param sessionOf - the run folder of the attempt to read

 @returns the summary's text

 @throws PiRunNotStartedError when that attempt left no summary

 @example
 ```ts
 const text = await readSummaryOf({ runsFolder: '/runs', sessionOf: 'a-1', });
 ```
 */
async function readSummaryOf({
  runsFolder,
  sessionOf,
}: {
  readonly runsFolder: string;
  readonly sessionOf: string;
},): Promise<string> {
  try {
    return await readFile(
      join(
        runsFolder,
        sessionOf,
        SUMMARY_FILE,
      ),
      { encoding: 'utf8', },
    );
  }
  catch (error) {
    throw new PiRunNotStartedError(
      `--session-of=${sessionOf} names no finished run under ${runsFolder}: ${String(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Reads the session file a resume continues.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param sessionOf - the run folder of the attempt whose session is continued

 @returns the session file's path

 @throws PiRunNotStartedError when that attempt left no summary naming a session

 @example
 ```ts
 const session = await sessionFileOf({ runsFolder: '/runs', sessionOf: 'a-1', });
 ```
 */
async function sessionFileOf({
  runsFolder,
  sessionOf,
}: {
  readonly runsFolder: string;
  readonly sessionOf: string;
},): Promise<string> {
  /**
   What that attempt left last, or why there is nothing to read.
   */
  const text = await readSummaryOf({
    runsFolder,
    sessionOf,
  },);
  /**
   The session file the attempt's statistics name, when it names one.
   */
  const sessionFile = (JSON.parse(text,) as {
    readonly sessionStats?: { readonly sessionFile?: unknown };
  }).sessionStats
    ?.sessionFile;
  if ((typeof sessionFile) !== 'string') {
    throw new PiRunNotStartedError(
      `--session-of=${sessionOf} left no session file in its statistics, so there is nothing to resume`,
    );
  }
  return sessionFile;
}

/**
 Reads every run that is going, with the pool each one's start named.

 @param launchers - every start of a run that is in flight

 @param units - every Pi unit systemd is running

 @returns one entry per run, by name

 @example
 ```ts
 const going = runsGoing({ launchers: new Map(), units: [], });
 ```
 */
function runsGoing({
  launchers,
  units,
}: {
  readonly launchers: ReadonlyMap<string, SlotPool | typeof NO_POOL>;
  readonly units: readonly RunningPiUnit[];
},): readonly {
  readonly name: string;
  readonly launcherPool: SlotPool | typeof NO_POOL | typeof NO_LAUNCHER;
}[] {
  /**
   Every run that is going, by name, which is the union of the units and the starts in flight.
   */
  const names = new Set<string>([
    ...launchers.keys(),
    ...units.map(function nameOf(unit,) {
      return unit.name;
    },),
  ],);
  return [...names,].map(function withItsPool(name,) {
    return {
      name,
      launcherPool: launchers.get(name,) ?? NO_LAUNCHER,
    };
  },);
}

/**
 Starts one run when its pool has a slot free.

 @param request - what the run was asked for, and everything the start asks the machine

 @returns the lines to print and the status to exit with

 @throws PiRunNotStartedError when the worktree, its dependencies or the session could not be prepared

 @example
 ```ts
 const outcome = await startAPiRun({ options, run: runCommandWithChildProcess, runToTheEnd, launchers, runningUnits, sizeATurnCap, home: '/home/a', });
 ```
 */
export async function startAPiRun(
  request: StartAPiRunRequest,
): Promise<StartOutcome> {
  /**
   Everything this start asks the machine, as the caller handed it over.
   */
  const {
    options,
    run,
    runToTheEnd,
    launchers,
    runningUnits,
    sizeATurnCap,
    home,
  } = request;
  /**
   Every run that is going now.
   */
  const going = runsGoing({
    launchers: await launchers(),
    units: await runningUnits(),
  },);
  if (runIsAlreadyGoing({
    name: options.name,
    going: new Set(going.map(function nameOf(running,) {
      return running.name;
    },),),
  },)) {
    return {
      status: ALREADY_RUNNING_EXIT_STATUS,
      lines: [`ALREADY RUNNING: ${options.name}`,],
    };
  }
  /**
   How many runs each pool holds.
   */
  const counts = countsOfPools({ runs: going, },);
  /**
   What the counts say, which every line about slots carries.
   */
  const summary = countsSummaryOf({ counts, },);
  /**
   The pool the caller's provider and model name.
   */
  const requested = poolNamed({
    provider: options.provider,
    model: options.model,
  },);
  /**
   The pool this run goes to, or {@link NO_POOL} for a run outside the pools of this table.
   */
  const chosen = poolWithRoom({
    requested,
    resumes: options.sessionOf !== undefined,
    counts,
  },);
  /**
   Whether the pool this run would go to has room for it.
   */
  const room = (chosen !== NO_POOL) && hasRoom({
    pool: chosen,
    counts,
  },);
  if ((requested !== NO_POOL) && (!room)) {
    return {
      status: NO_SLOT_EXIT_STATUS,
      lines: [`NO SLOT: ${summary}, not started: ${options.name}`,],
    };
  }
  /**
   The worktree the run works in, and what preparing it printed.
   */
  const prepared = await workingFolderIn({
    options,
    run,
  },);
  /**
   The folder this attempt writes into, whose inbox is what a coordinator steers the run through.
   */
  const runFolder = runFolderPathOf({
    runsFolder: options.runsFolder,
    name: options.name,
    run: options.run,
  },);
  await mkdir(
    runFolder,
    { recursive: true, },
  );
  await mkdir(inboxOfARun({ runFolder, },),);
  /**
   Whether the pool this run went to is one a default run may be moved into, which is what multiplies its wall clock.
   */
  const movedToAPool = (chosen !== NO_POOL) && (chosen !== requested);
  /**
   What the turn cap is read from, or what the caller named.
   */
  const sized = (options.turnCap === undefined)
    ? await sizeATurnCap({
      runsFolder: options.runsFolder,
      runName: options.name,
    },)
    : undefined;
  /**
   How many turns the run may start.
   */
  const turnCap = options.turnCap ?? (sized?.cap ?? 0);
  /**
   The wall clock the run is given, which a slower pool doubles.
   */
  const wallSeconds = movedToAPool
    ? wallSecondsInPool({
      pool: chosen,
      wallSeconds: options.wallSeconds,
    },)
    : options.wallSeconds;
  /**
   The session this run continues, when it continues one.
   */
  const session = (options.sessionOf === undefined)
    ? options.session
    : await sessionFileOf({
      runsFolder: options.runsFolder,
      sessionOf: options.sessionOf,
    },);
  /**
   What the unit needs from this machine's own environment.
   */
  const environment = await unitEnvironmentOf({ run, },);
  /**
   How the unit ended.
   */
  const ended: CommandOutput = await runToTheEnd({
    command: 'systemd-run',
    words: unitWordsOf({
      options,
      runFolder,
      workingFolder: prepared.folder,
      wallSeconds,
      turnCap,
      home,
      environment,
      ...((session === undefined) ? {} : { session, }),
    },),
  },);
  /**
   What became of the worktree this run worked in, which a run that finished its work gives back to the disk.
   */
  const release = (options.branch === NO_WORKTREE_BRANCH)
    ? {
      released: false,
      lines: [] as readonly string[],
    }
    : await releaseFinishedWorktree({
      run,
      repository: options.repository,
      worktree: prepared.folder,
      runFolder,
    },);
  return {
    status: ended.status,
    lines: [
      `slot free: ${summary}; ${options.name} goes on ${(chosen === NO_POOL) ? 'its own provider' : chosen.name}`,
      ...prepared.lines,
      ...((options.branch === NO_WORKTREE_BRANCH)
        ? []
        : [await dependenciesIn({
          folder: prepared.folder,
          run,
        },),]),
      ...((sized === undefined) ? [`turn cap ${String(turnCap,)}: named with --turn-cap`,] : [sized.line,]),
      `${options.name}: unit ended with status ${String(ended.status,)}${(ended.problem === undefined) ? '' : ` (${ended.problem})`}; run folder ${runFolder}; worktree ${prepared.folder}`,
      ...release.lines
        .map(function asLineOfThisRun(line,) {
          return `${options.name}: ${line}`;
        },),
    ],
  };
}

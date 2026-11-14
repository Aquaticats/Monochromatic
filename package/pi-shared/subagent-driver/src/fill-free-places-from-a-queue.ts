#!/usr/bin/env node
/**
 `fill-free-places-from-a-queue`: fills the places free under a level from a queue of runs, round after round, without a session
 being awake. See `README.md` for the options and the entry shapes.

 Each round first puts the reviews the provider's rate limit ended back at the head of the queue, then fills what the level leaves
 free, then says on the desktop whether the filling has stalled. A round that throws is reported and the filling goes on: one entry
 the filling cannot read, or one command that failed, must not stop the queue behind it.

 Nothing started here outlives this command in a way a session must tend: every entry is handed to its own unit, which the filling
 never waits for.

 @module
 */

import { statfsSync, } from 'node:fs';
import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import {
  BUILD_VOLUME,
  type FreeSpace,
  freeSpaceOnVolume,
} from './free-space-floor.ts';
import { fillFreePlacesFromAQueue, } from './filling-free-places-from-a-queue.ts';
import { placesAskedFor, } from './asked-for-places.ts';
import { gitRunnerOf, } from './git-command.ts';
import {
  goingRunsNow,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
} from './going-runs.ts';
import {
  keepFailedOutput,
  reportOnStandardError,
} from './kept-start-failure.ts';
import {
  FILLING_USAGE,
  parseFillingOptions,
} from './parse-filling-options.ts';
import {
  limitsIn,
  POOL_LIMITS_VARIABLE,
  poolsWithTheseLimits,
} from './pool-limits.ts';
import {
  putBackSince,
  putEndedReviewsBack,
} from './putting-ended-reviews-back.ts';
import type { QueueRecord, } from './queue-of-runs.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import {
  setAsideAnEntryOfAQueue,
} from './set-aside-entry.ts';
import {
  type DesktopNotice,
  type DesktopNoticeOutcome,
  noticeOnTheDesktop,
} from './desktop-notice.ts';
import { reportOnStandardOutput, } from './standard-output-of-a-command.ts';
import { noticeIfTheFillingStalls, } from './stalled-filling-notice.ts';
import {
  startAnEntryOfAQueue,
  type StartOfAnEntry,
} from './starting-an-entry-of-a-queue.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: FILLING_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The folder the process table is read from, which is where a Linux machine keeps it. A test points this at a folder of its own,
 so no test counts the processes of the machine it runs on.
 */
const PROCESS_FOLDER = env.PI_RUNS_PROCESS_FOLDER ?? '/proc';

/**
 The variable that names the room the build volume has, in bytes, instead of reading the volume.
 */
const GIVEN_FREE_SPACE_BYTES_VARIABLE = 'PI_RUNS_FREE_SPACE_BYTES';

/**
 What this run of the command was asked for.
 */
const options = parseFillingOptions({ commandLine: argv.slice(2,), },);

/**
 The pools this run counts against: the shipped table, with the limits a caller set in the environment overriding them.
 */
const pools = poolsWithTheseLimits({ limits: limitsIn({ listed: env[POOL_LIMITS_VARIABLE] ?? '', },), },);

/**
 How the package asks git a question.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 Reads the room the volume that holds the repository has, which a test may give instead of the machine reading its own disk.

 @returns the volume and the room it has
 */
function freeSpaceNow(): FreeSpace {
  /**
   The room the caller gave, in bytes.
   */
  const given = env[GIVEN_FREE_SPACE_BYTES_VARIABLE] ?? '';
  return (given === '')
    ? freeSpaceOnVolume({
      volume: BUILD_VOLUME,
      statfs: statfsSync,
    },)
    : {
      volume: BUILD_VOLUME,
      freeBytes: Number(given,),
    };
}

/**
 Reads every run going now, every launcher that has not made its unit yet, and every place asked for whose run has not appeared.

 A place asked for counts against the level from the moment its batch was asked for, which is what keeps a filling from starting an
 entry into a place another batch is already promised.

 @returns what systemd, the process table and the batch lists beside the runs say
 */
async function runsGoing(): Promise<Awaited<ReturnType<typeof goingRunsNow>>> {
  /**
   The runs and launchers a count of the machine finds.
   */
  const going = await goingRunsNow({
    run: runCommandWithChildProcess,
    processFolder: PROCESS_FOLDER,
    launcherEntryNames: [
      ...LAUNCHER_ENTRY_NAMES,
      ...REPLACED_LAUNCHER_ENTRY_NAMES,
    ],
    pools,
    onProblem: reportOnStandardError,
  },);
  /**
   The places asked for by a batch whose run has not appeared.
   */
  const askedFor = await placesAskedFor({
    runsFolder: options.runsFolder,
    run: runCommandWithChildProcess,
    goingNames: new Set(going.map(function nameOf(run,) {
      return run.name;
    },),),
    onProblem: reportOnStandardError,
  },);
  return [
    ...going,
    ...askedFor,
  ];
}

/**
 Starts one entry and keeps the whole of what its failed start printed, beside the runs.

 @param record - the entry to start

 @returns what the start answered, and where the whole of its output went when it failed
 */
async function startAndKeep({ record, }: { readonly record: QueueRecord },): Promise<StartOfAnEntry> {
  /**
   What starting the entry answered.
   */
  const started = await startAnEntryOfAQueue({
    request: {
      record,
      runsFolder: options.runsFolder,
      reviewsFolder: options.reviewsFolder,
      checkoutFolder: options.checkoutFolder,
      home: env.HOME ?? '',
      now: Date.now,
      deadlineSeconds: options.deadlineSeconds,
    },
    seams: { git, },
  },);
  if (started.started) {
    return started;
  }
  /**
   Where the whole of what the failed start printed was kept.
   */
  const kept = await keepFailedOutput({
    runsFolder: options.runsFolder,
    name: record.name,
    output: started.failure
      .output,
    report: reportOnStandardError,
  },);
  return {
    started: false,
    failure: {
      ...started.failure,
      ...(kept.keptAt === undefined ? {} : { keptOutputAt: kept.keptAt, }),
    },
  };
}

/**
 Puts the reviews the provider's rate limit ended back at the head of the queue, and says one line for each.
 */
async function putEndedReviewsBackThisRound(): Promise<void> {
  try {
    /**
     The runs going now, which a review whose unit is up is not put back for.
     */
    const going = await runsGoing();
    /**
     What putting the ended reviews back answered.
     */
    const putBack = await putEndedReviewsBack({
      queueFile: options.queueFile,
      runsFolder: options.runsFolder,
      reviewsFolder: options.reviewsFolder,
      since: putBackSince({ now: Date.now(), },),
      going: new Set(going.map(function nameOf(run,) {
        return run.name;
      },),),
      now: Date.now,
    },);
    for (const line of putBack.lines) {
      reportOnStandardOutput(`${line}\n`,);
    }
  }
  catch (error) {
    // One review whose record could not be read must not end the filling, which is what the queue behind it waits on.
    reportOnStandardError(`${new Date().toISOString()} putting ended reviews back failed and the filling goes on: ${String(error,)}\n`,);
  }
}

/**
 What this filling has last started and last said, which each round keeps for the next.
 */
const fillingSoFar: {
  lastStartedAt: number;
  lastNoticedAt: number;
} = {
  lastStartedAt: Date.now(),
  lastNoticedAt: 0,
};

/**
 Shows one desktop notification on the machine's own notification path.

 @param notice - the title and body to show

 @returns whether the desktop accepted it, and why not when it did not

 @example
 ```ts
 const shown = await notifyOnTheDesktop({ title: 'x', body: 'y' });
 ```
 */
function notifyOnTheDesktop(notice: DesktopNotice,): Promise<DesktopNoticeOutcome> {
  return noticeOnTheDesktop({ notice, },);
}

/**
 When this filling stops asking for places, as milliseconds since the epoch.
 */
const stopAskingAt = Date.now() + (options.secondsAtMost * MILLISECONDS_PER_SECOND);

/* oxlint-disable eslint/no-await-in-loop -- one round after another: each round counts the runs going and fills the places the
   level leaves, and a second count inside a round reads a run just started as a free place. See doc/lint/sequential-awaits.md. */
for (;;) {
  if (options.putBackEndedReviews) {
    await putEndedReviewsBackThisRound();
  }
  try {
    /**
     What one round of filling answered.
     */
    const outcome = await fillFreePlacesFromAQueue({
      request: {
        queueFile: options.queueFile,
        runsFolder: options.runsFolder,
        level: options.level,
        startsAtMost: options.startsAtMost,
      },
      seams: {
        runsGoing,
        freeSpace: freeSpaceNow,
        start: startAndKeep,
        setAside: function setAside({
          record,
          failure,
        },) {
          return setAsideAnEntryOfAQueue({
            queueFile: options.queueFile,
            runsFolder: options.runsFolder,
            record,
            reason: failure.reason,
            ...(failure.keptOutputAt === undefined ? {} : { keptOutputAt: failure.keptOutputAt, }),
            now: Date.now,
          },);
        },
        wait,
        report: reportOnStandardOutput,
      },
    },);
    /**
     The entries the round started, in the order it started them.
     */
    const { started: startedThisRound, } = outcome;
    if (startedThisRound.length > 0) {
      fillingSoFar.lastStartedAt = Date.now();
    }
    /**
     What the look for a stalled filling answered.
     */
    const noticed = await noticeIfTheFillingStalls({
      look: {
        lastStartedAt: fillingSoFar.lastStartedAt,
        now: Date.now(),
        entriesStanding: outcome.left,
        placesFree: outcome.placesFreeAtTheStart,
        queueFile: options.queueFile,
        ...(outcome.stoppedBecause === undefined ? {} : { stoppedBecause: outcome.stoppedBecause, }),
        ...((fillingSoFar.lastNoticedAt === 0) ? {} : { lastNoticedAt: fillingSoFar.lastNoticedAt, }),
        notify: notifyOnTheDesktop,
        report: reportOnStandardOutput,
      },
    },);
    /**
     When the filling last said on the desktop that it had stalled, which zero means it has not.
     */
    const { lastNoticedAt, } = noticed;
    fillingSoFar.lastNoticedAt = lastNoticedAt ?? 0;
  }
  catch (error) {
    // One failed round must not end the filling: the round after it asks again.
    reportOnStandardError(`${new Date().toISOString()} a round failed and the filling goes on: ${String(error,)}\n`,);
  }
  if (options.once || ((Date.now() + options.roundMilliseconds) > stopAskingAt)) {
    break;
  }
  await wait(options.roundMilliseconds,);
}
/* oxlint-enable eslint/no-await-in-loop */

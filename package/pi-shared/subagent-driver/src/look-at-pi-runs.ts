/**
 One look at every Pi run going now: which are past a limit the finished runs set, and where each one stands.

 The look reads the runs folder and nothing else: the units it judges are handed to it, so a test decides which runs exist and
 how long each has been up.

 @module
 */

import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { CheckedRuns, } from './checked-runs.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  MILLISECONDS_PER_MINUTE,
  OPEN_CALL_PERCENTILE,
  OVERDUE_PERCENTILE,
} from './overdue-constants.ts';
import {
  minutesOf,
  percentTextOf,
} from './overdue-line-text.ts';
import {
  type FinishedRunEvidence,
  evidenceFromRunsFolder,
} from './overdue-limits.ts';
import {
  type NO_RUN_FOLDER,
  latestRunFolderOf,
} from './run-folder-of-a-unit.ts';
import type { RunningPiUnit, } from './running-pi-units.ts';
import {
  type OpenToolCall,
  toolCallTimesOfFeed,
} from './tool-call-times-of-a-feed.ts';

/**
 The code the operating system gave a missing path.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 A limit that found nothing to report, as a symbol so no line can be mistaken for it.
 */
const NO_FINDING: unique symbol = Symbol('unit sits inside its limit, so there is nothing to report',);

/**
 What a run's feed said at the moment of the look.
 */
type FeedState = {
  /**
   The calls the run has started and not ended.
   */
  readonly openCalls: readonly OpenToolCall[];

  /**
   How long the feed has been written to nothing.
   */
  readonly quietMilliseconds?: number;
};

/**
 What one look found.
 */
export type OverdueLook = {
  /**
   The lines to act on, one per finding. The command that makes the look prints these and nothing else, so every line answers a
   question the caller can do something about.
   */
  readonly lines: readonly string[];

  /**
   Whether any Pi run is going at all, which says whether a later look could find anything.
   */
  readonly anyRunning: boolean;

  /**
   When each finding was reported, to be kept until the next look.
   */
  readonly reported: CheckedRuns;

  /**
   What the finished runs said, so a look that found nothing can say what it compared against.
   */
  readonly evidence: FinishedRunEvidence;
};

/**
 What one look needs.
 */
export type OverdueLookRequest = {
  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   The Pi runs going now.
   */
  readonly units: readonly RunningPiUnit[];

  /**
   When each finding was reported before.
   */
  readonly checked: CheckedRuns;

  /**
   The time the look reads, as milliseconds since the epoch.
   */
  readonly now: number;

  /**
   How long a reported finding is left alone before it is reported again.
   */
  readonly recheckMilliseconds: number;

  /**
   A limit in minutes that replaces both computed limits, for a caller who wants every run older than it reported now. It also
   passes the recheck interval, because a person asking now is not a rotation to keep quiet.
   */
  readonly limitMinutes?: number;
};

/**
 What looking at one unit found.
 */
type UnitLook = {
  readonly lines: readonly string[];
  readonly keys: readonly string[];
};

/**
 Reads what a run's feed says at the moment of the look.

 The quiet time is the feed's last write, read straight after the feed itself, so a run that has written nothing for a while is
 told apart from one whose call simply takes long.

 @param feedFile - the run's feed

 @param now - the time the look reads

 @returns the open calls and the quiet time, and no open call at all for a run that has written no feed yet

 @example
 ```ts
 const feed = await feedStateOf({ feedFile: '/runs/a-1/feed.jsonl', now: Date.now(), },);
 ```
 */
async function feedStateOf({
  feedFile,
  now,
}: {
  readonly feedFile: string;
  readonly now: number;
},): Promise<FeedState> {
  try {
    /**
     The run's calls.
     */
    const times = await toolCallTimesOfFeed({ feedFile, },);
    /**
     When the feed was last written.
     */
    const feed = await stat(feedFile,);
    return {
      openCalls: times.openCalls,
      quietMilliseconds: now - feed.mtimeMs,
    };
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return { openCalls: [], };
    }
    throw error;
  }
}

/**
 Says whether a finding is due to be reported, which it is when it was never reported or its recheck interval has passed.

 @param checked - when each finding was reported before

 @param key - the finding's key

 @param now - the time the look reads

 @param recheckMilliseconds - how long a finding is left alone after it is reported

 @returns whether to report it again

 @example
 ```ts
 const due = isDue({ checked: {}, key: 'pi-a', now: 1_000, recheckMilliseconds: 60_000, },);
 ```
 */
function isDue({
  checked,
  key,
  now,
  recheckMilliseconds,
}: {
  readonly checked: CheckedRuns;
  readonly key: string;
  readonly now: number;
  readonly recheckMilliseconds: number;
},): boolean {
  /**
   When this finding was reported before.
   */
  const reportedAt = checked[key];
  if ((reportedAt === undefined) || (reportedAt > now)) {
    return true;
  }
  return (now - reportedAt) >= recheckMilliseconds;
}

/**
 Describes a call that is open, with how long it has been open.

 The age is the run's own elapsed time minus the moment the feed recorded for the call's start, so it needs no clock of its
 own.

 @param openCall - the call

 @param elapsedMilliseconds - how long the run has been up

 @returns the tool's name and the call's age

 @example
 ```ts
 const shown = openCallTextOf({ openCall, elapsedMilliseconds: 600_000, },);
 ```
 */
function openCallTextOf({
  openCall,
  elapsedMilliseconds,
}: {
  readonly openCall: OpenToolCall;
  readonly elapsedMilliseconds: number;
},): string {
  return `${openCall.tool} open ${minutesOf(elapsedMilliseconds - openCall.startedMilliseconds,)} min`;
}

/**
 Describes where a run stands, as every line about it ends.

 @param folder - the run's folder, or {@link NO_RUN_FOLDER}

 @param unit - the run going now

 @param runsFolder - folder the driver writes one run folder per run into

 @returns where to read the run, or why there is nowhere to read

 @example
 ```ts
 const shown = whereTextOf({ folder: '/runs/a-1', unit, runsFolder: '/runs', },);
 ```
 */
function whereTextOf({
  folder,
  unit,
  runsFolder,
}: {
  readonly folder: string | typeof NO_RUN_FOLDER;
  readonly unit: RunningPiUnit;
  readonly runsFolder: string;
},): string {
  return ((typeof folder) === 'symbol')
    ? `no run folder named ${unit.name}-<number> under ${runsFolder}`
    : folder;
}

/**
 Describes a run whose open call has been open for longer than the finished calls say one takes.

 @param unit - the run going now

 @param openCall - the call that is open

 @param limitMilliseconds - how long a call may stay open

 @param where - where the run's folder is, or why it has none

 @param toolCalls - how many finished calls the limit came from

 @returns the line to act on, or {@link NO_FINDING} while the call is within the limit

 @example
 ```ts
 const shown = stuckLineOf({ unit, openCall, limitMilliseconds: 1_200_000, where: '/runs/a-1', toolCalls: 24, },);
 ```
 */
function stuckLineOf({
  unit,
  openCall,
  limitMilliseconds,
  where,
  toolCalls,
}: {
  readonly unit: RunningPiUnit;
  readonly openCall: OpenToolCall;
  readonly limitMilliseconds: number;
  readonly where: string;
  readonly toolCalls: number;
},): string | typeof NO_FINDING {
  /**
   How long the call has been open.
   */
  const openMilliseconds = unit.elapsedMilliseconds - openCall.startedMilliseconds;
  if (openMilliseconds <= limitMilliseconds) {
    return NO_FINDING;
  }
  /**
   What the call is, and how long it has been that way.
   */
  const call = openCallTextOf({
    openCall,
    elapsedMilliseconds: unit.elapsedMilliseconds,
  },);
  return `STUCK? ${unit.unitName}: ${call}, and ${percentTextOf(OPEN_CALL_PERCENTILE,)} of ${String(toolCalls,)} finished calls ended within ${minutesOf(limitMilliseconds,)} min; ${where}`;
}

/**
 Describes a run that has been up for longer than the finished runs say one takes.

 @param unit - the run going now

 @param limitMilliseconds - how long a run may last

 @param quietMilliseconds - how long the run's feed has been quiet

 @param openCall - the call the run has open, when it has one

 @param where - where the run's folder is, or why it has none

 @param finishedRuns - how many finished runs the limit came from

 @returns the line to act on, or {@link NO_FINDING} while the run is within the limit

 @example
 ```ts
 const shown = overdueLineOf({ unit, limitMilliseconds: 5_100_000, where: '/runs/a-1', finishedRuns: 120, },);
 ```
 */
function overdueLineOf({
  unit,
  limitMilliseconds,
  quietMilliseconds,
  openCall,
  where,
  finishedRuns,
}: {
  readonly unit: RunningPiUnit;
  readonly limitMilliseconds: number;
  readonly quietMilliseconds?: number;
  readonly openCall?: OpenToolCall;
  readonly where: string;
  readonly finishedRuns: number;
},): string | typeof NO_FINDING {
  if (unit.elapsedMilliseconds <= limitMilliseconds) {
    return NO_FINDING;
  }
  /**
   How long the run has been quiet.
   */
  const quiet = (quietMilliseconds === undefined)
    ? 'no feed yet'
    : `feed quiet ${minutesOf(quietMilliseconds,)} min`;
  /**
   The call the run has open, when it has one.
   */
  const call = (openCall === undefined)
    ? ''
    : `, ${openCallTextOf({
      openCall,
      elapsedMilliseconds: unit.elapsedMilliseconds,
    },)}`;
  return `OVERDUE ${unit.unitName}: up ${minutesOf(unit.elapsedMilliseconds,)} min, and ${percentTextOf(OVERDUE_PERCENTILE,)} of ${String(finishedRuns,)} finished runs were done within ${minutesOf(limitMilliseconds,)} min; ${quiet}${call}; ${where}`;
}

/**
 Looks at one Pi run.

 @param unit - the run going now

 @param request - what the look needs

 @param evidence - what the finished runs said

 @returns the lines to act on and the keys they answer

 @example
 ```ts
 const found = await lookAtOneUnit({ unit, request, evidence, },);
 ```
 */
async function lookAtOneUnit({
  unit,
  request,
  evidence,
}: {
  readonly unit: RunningPiUnit;
  readonly request: OverdueLookRequest;
  readonly evidence: FinishedRunEvidence;
},): Promise<UnitLook> {
  /**
   The folder this run writes to.
   */
  const folder = await latestRunFolderOf({
    runsFolder: request.runsFolder,
    runName: unit.name,
  },);
  /**
   What the run's feed says, and nothing about a call for a run that has written none.
   */
  const feed: FeedState = ((typeof folder) === 'symbol')
    ? { openCalls: [], }
    : await feedStateOf({
      feedFile: join(
        folder,
        'feed.jsonl',
      ),
      now: request.now,
    },);
  /**
   A limit the caller set, in milliseconds, which replaces the one the finished runs give.
   */
  const forcedMilliseconds = (request.limitMinutes === undefined)
    ? undefined
    : request.limitMinutes * MILLISECONDS_PER_MINUTE;
  /**
   How long a run may last before it is reported.
   */
  const overdueMilliseconds = forcedMilliseconds ?? evidence.overdueMilliseconds;
  /**
   How long one call may stay open before it is reported.
   */
  const openCallMilliseconds = forcedMilliseconds ?? evidence.openCallMilliseconds;
  /**
   Where the run is, as every line about it ends.
   */
  const where = whereTextOf({
    folder,
    unit,
    runsFolder: request.runsFolder,
  },);
  /**
   The run's oldest open call, which is the one that decides whether the run is stuck.
   */
  const [oldestCall,] = feed.openCalls;
  /**
   Says whether a finding about this unit is past the recheck interval, which a caller's own limit always is.

   @param key - the finding's key

   @returns whether to report it again

   @example
   ```ts
   const due = isDueFor('pi-a.service',);
   ```
   */
  function isDueFor(key: string,): boolean {
    return (forcedMilliseconds !== undefined) || isDue({
      checked: request.checked,
      key,
      now: request.now,
      recheckMilliseconds: request.recheckMilliseconds,
    },);
  }

  /**
   The line about the open call, when there is one and it is past its limit and due.
   */
  const stuck = ((openCallMilliseconds === undefined) || (oldestCall === undefined)
    || (!isDueFor(`open-call:${unit.unitName}`,)))
    ? NO_FINDING
    : stuckLineOf({
      unit,
      openCall: oldestCall,
      limitMilliseconds: openCallMilliseconds,
      where,
      toolCalls: evidence.toolCalls,
    },);
  /**
   The line about the run itself, when it is past its limit and due.
   */
  const overdue = ((overdueMilliseconds === undefined) || (!isDueFor(unit.unitName,)))
    ? NO_FINDING
    : overdueLineOf({
      unit,
      limitMilliseconds: overdueMilliseconds,
      ...(feed.quietMilliseconds === undefined ? {} : { quietMilliseconds: feed.quietMilliseconds, }),
      ...(oldestCall === undefined ? {} : { openCall: oldestCall, }),
      where,
      finishedRuns: evidence.finishedRuns,
    },);
  return {
    lines: [
      stuck,
      overdue,
    ].filter(function isFound(line,): line is string {
      return (typeof line) !== 'symbol';
    },),
    keys: [
      ...((typeof stuck) === 'symbol' ? [] : [`open-call:${unit.unitName}`,]),
      ...((typeof overdue) === 'symbol' ? [] : [unit.unitName,]),
    ],
  };
}

/**
 Looks at every Pi run going now.

 @param runsFolder - folder the driver writes one run folder per run into

 @param units - the Pi runs going now

 @param checked - when each finding was reported before

 @param now - the time the look reads

 @param recheckMilliseconds - how long a reported finding is left alone

 @param limitMinutes - a limit that replaces both computed limits, for a caller who wants every run older than it reported now

 @returns the lines to act on, and when each finding was reported

 @example
 ```ts
 const look = await lookAtPiRuns({ runsFolder: '/runs', units: [], checked: {}, now: Date.now(), recheckMilliseconds: 1_800_000, },);
 ```
 */
export async function lookAtPiRuns({
  runsFolder,
  units,
  checked,
  now,
  recheckMilliseconds,
  limitMinutes,
}: {
  readonly runsFolder: string;
  readonly units: readonly RunningPiUnit[];
  readonly checked: CheckedRuns;
  readonly now: number;
  readonly recheckMilliseconds: number;
  readonly limitMinutes?: number;
},): Promise<OverdueLook> {
  /**
   What the finished runs say a run should have done by now.
   */
  const evidence = await evidenceFromRunsFolder({ runsFolder, },);
  /**
   What every run going now says.
   */
  const looks = await Promise.all(units.map(function lookAtUnit(unit,) {
    return lookAtOneUnit({
      unit,
      request: {
        runsFolder,
        units,
        checked,
        now,
        recheckMilliseconds,
        ...(limitMinutes === undefined ? {} : { limitMinutes, }),
      },
      evidence,
    },);
  },));
  /**
   When each finding was reported: this moment for the findings this look produced, and what it was before for the rest. A
   look asked to report again writes nothing, so a forced limit leaves the recheck interval as it was.
   */
  const reported: Record<string, number> = { ...checked, };
  if (limitMinutes === undefined) {
    for (const key of looks.flatMap(function keysOf(look,) {
      return look.keys;
    },)) {
      reported[key] = now;
    }
  }
  return {
    lines: looks.flatMap(function linesOf(look,) {
      return look.lines;
    },),
    anyRunning: units.length > 0,
    reported,
    evidence,
  };
}

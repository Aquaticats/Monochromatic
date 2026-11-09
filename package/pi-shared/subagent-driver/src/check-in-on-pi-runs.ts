#!/usr/bin/env node
/**
 `check-in-on-pi-runs`: describes the Pi runs named on its command line, so a quiet feed can be told from a stuck run. For each
 one it prints the tool call the run has open, how long the feed has been quiet, and the processes under the agent with their
 age, state and the kernel call each is waiting in.

 It reads each process's command line, never its environment.

 @module
 */

import { join, } from 'node:path';
import { argv, } from 'node:process';
import { stat, } from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { SECONDS_PER_MINUTE, } from './overdue-constants.ts';
import { minutesOf, } from './overdue-line-text.ts';
import { parseCheckInOptions, } from './parse-check-in-options.ts';
import {
  type ProcessUnderARun,
  processesUnder,
  startProcessTable,
} from './processes-under-a-pi-run.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { latestRunFolderOf, } from './run-folder-of-a-unit.ts';
import {
  type RunningPiUnit,
  startRunningPiUnits,
} from './running-pi-units.ts';
import { toolCallTimesOfFeed, } from './tool-call-times-of-a-feed.ts';

/**
 The code the operating system gave a missing path.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How many of a run's processes are printed: enough to name what is working, few enough that one line stays one line.
 */
const PROCESSES_SHOWN = 8;

/**
 How much of a process's command line is printed. A command line can hold a whole document, and the head names the work.
 */
const COMMAND_CHARACTERS = 170;

/**
 How much of the open call's arguments are printed, which the feed already keeps short.
 */
const ARGUMENTS_CHARACTERS = 300;

/**
 What this run of the command was asked for.
 */
const options = parseCheckInOptions(argv.slice(2,),);

/**
 Reads the Pi runs going now.
 */
const runningPiUnits = startRunningPiUnits({ run: runCommandWithChildProcess, },);

/**
 Reads the process table.
 */
const processTable = startProcessTable({ run: runCommandWithChildProcess, },);

/**
 The Pi runs going now, read once so every named run is described at the same moment.
 */
const units = await runningPiUnits();

/**
 The process table, read once for the same reason.
 */
const table = await processTable();

/**
 Cuts text to what a line prints.

 @param text - the text to print

 @param characters - how much of it to keep

 @returns the text, cut to that length

 @example
 ```ts
 const shown = cutTo({ text: 'a long command line', characters: 8, },);
 ```
 */
function cutTo({
  text,
  characters,
}: {
  readonly text: string;
  readonly characters: number;
},): string {
  return (text.length <= characters) ? text : `${text.slice(
    0,
    characters,
  )}...`;
}

/**
 Describes one process.

 @param process - the process to describe

 @returns the process's number, parent, age, state, kernel wait and command

 @example
 ```ts
 const shown = processLineOf(process,);
 ```
 */
function processLineOf(process: ProcessUnderARun,): string {
  return `  pid ${String(process.pid,)} ppid ${String(process.parentPid,)} ${String(Math.round(process.elapsedSeconds / SECONDS_PER_MINUTE,),)}m ${process.state} wait=${process.waitingIn} ${cutTo({
    text: process.command,
    characters: COMMAND_CHARACTERS,
  },)}`;
}

/**
 Describes the processes under one run.

 @param unit - the run going now

 @returns one line per process, oldest first

 @example
 ```ts
 const lines = processLinesOf(unit,);
 ```
 */
function processLinesOf(unit: RunningPiUnit,): readonly string[] {
  /**
   Every process under the run, oldest first.
   */
  const under = processesUnder({
    table,
    rootPid: unit.mainPid,
  },)
    .toSorted(function oldestFirst(
      left,
      right,
    ) {
      return right.elapsedSeconds - left.elapsedSeconds;
    },);
  return [
    `processes under the agent: ${String(under.length,)}; the ${String(Math.min(
      under.length,
      PROCESSES_SHOWN,
    ),)} oldest:`,
    ...under.slice(
      0,
      PROCESSES_SHOWN,
    )
      .map(processLineOf,),
  ];
}

/**
 Describes what a run's feed says: the call it has open and how long the feed has been quiet.

 @param feedFile - the run's feed

 @returns the lines to print about the feed

 @example
 ```ts
 const lines = await feedLinesOf('/runs/a-1/feed.jsonl',);
 ```
 */
async function feedLinesOf(feedFile: string,): Promise<readonly string[]> {
  try {
    /**
     The run's calls.
     */
    const times = await toolCallTimesOfFeed({ feedFile, },);
    /**
     How long the feed has been written to nothing.
     */
    const quiet = Date.now() - (await stat(feedFile,)).mtimeMs;
    /**
     The call the run has open, when it has one.
     */
    const [openCall,] = times.openCalls;
    if (openCall === undefined) {
      return [`feed quiet ${minutesOf(quiet,)} min; no tool call open`,];
    }
    /**
     What the call runs, and how much of its arguments the feed kept.
     */
    const argumentsShown = cutTo({
      text: openCall.argumentsPreview,
      characters: ARGUMENTS_CHARACTERS,
    },);
    return [`feed quiet ${minutesOf(quiet,)} min; last call open: ${openCall.tool}, started ${minutesOf(openCall.startedMilliseconds,)} min into the run: ${argumentsShown}`,];
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return ['the run has written no feed yet',];
    }
    throw error;
  }
}

/**
 Describes one named run.

 @param unitName - the unit's name as systemd prints it

 @returns every line about that run

 @example
 ```ts
 const lines = await reportOf('pi-a.service',);
 ```
 */
async function reportOf(unitName: string,): Promise<readonly string[]> {
  /**
   The run named, when it is going now.
   */
  const unit = units.find(function isNamed(running,) {
    return running.unitName === unitName;
  },);
  if (unit === undefined) {
    return [
      `=== ${unitName}`,
      'not running now',
    ];
  }
  /**
   The folder this run writes to.
   */
  const folder = await latestRunFolderOf({
    runsFolder: options.runsFolder,
    runName: unit.name,
  },);
  if ((typeof folder) === 'symbol') {
    return [
      `=== ${unitName}`,
      `up ${minutesOf(unit.elapsedMilliseconds,)} min; main pid ${String(unit.mainPid,)}; no run folder named ${unit.name}-<number> under ${options.runsFolder}`,
      ...processLinesOf(unit,),
    ];
  }
  /**
   The run's feed.
   */
  const feedFile = join(
    folder,
    'feed.jsonl',
  );
  return [
    `=== ${unitName}`,
    `up ${minutesOf(unit.elapsedMilliseconds,)} min; main pid ${String(unit.mainPid,)}; ${feedFile}`,
    ...await feedLinesOf(feedFile,),
    ...processLinesOf(unit,),
  ];
}

/**
 The lines to print, for every run named.
 */
const lines = (await Promise.all(options.unitNames
  .map(reportOf,))).flat();

console.log(lines.join('\n',),);

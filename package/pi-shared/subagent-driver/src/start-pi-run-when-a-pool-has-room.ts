#!/usr/bin/env node
/**
 `start-pi-run-when-a-pool-has-room`: starts one Pi run when the provider pool its provider and model share has room, and answers
 why it started none otherwise. See `README.md` for the options and the pools.

 The run itself is the worktree launcher's work: this command only counts, chooses the pool, and hands the launcher the words the
 run needs.

 @module
 */

import { statfsSync, } from 'node:fs';
import {
  realpath,
  stat,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  argv,
  env,
} from 'node:process';
import {
  BUILD_VOLUME,
  BUILD_VOLUME_FLOOR_BYTES,
  type FreeSpace,
  type FreeSpaceAgainstItsFloor,
  RUNS_FOLDER_FLOOR_BYTES,
  SESSION_FILES_FLOOR_BYTES,
  type VolumeAgainstItsFloor,
  freeSpaceOnVolume,
  strictestFloorPerVolume,
} from './free-space-floor.ts';
import {
  goingRunsNow,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
} from './going-runs.ts';
import {
  MACHINE_CAP_VARIABLE,
  type MachineCap,
  OTHER_AGENTS_GOING_VARIABLE,
  machineCapIn,
  otherAgentsGoingIn,
} from './machine-cap.ts';
import {
  POOL_GATE_USAGE,
  parsePoolGateOptions,
} from './parse-pool-gate-options.ts';
import {
  SESSION_FOLDER_VARIABLE,
  sessionFolderOf,
} from './pi-session-folder.ts';
import {
  limitsIn,
  POOL_LIMITS_VARIABLE,
  poolsWithTheseLimits,
} from './pool-limits.ts';
import { gateOutcomeOf, } from './pool-gate.ts';
import { talliesOf, } from './pool-tallies.ts';
import {
  runCommandShowingItsOutput,
  runCommandWithChildProcess,
} from './run-command.ts';
import { sessionFileOfRun, } from './session-file-of-a-run.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';
import {
  type AnswerAboutAFolder,
  volumeOfFolder,
} from './volume-of-a-folder.ts';
import {
  FOLDER_VOLUMES_VARIABLE,
  VOLUME_FREE_SPACE_BYTES_VARIABLE,
  givenFolderVolumes,
  givenVolumeFreeSpaceBytes,
} from './volume-settings.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: POOL_GATE_USAGE,
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
 The map of the pairs each volume setting holds, which a test writes instead of naming this machine's disks and links.
 */
const givenRooms = givenVolumeFreeSpaceBytes({ listed: env[VOLUME_FREE_SPACE_BYTES_VARIABLE] ?? '', },);

/**
 The volume each folder was given, which a test writes instead of resolving this machine's symbolic links and devices.
 */
const givenVolumesOfFolders = givenFolderVolumes({ listed: env[FOLDER_VOLUMES_VARIABLE] ?? '', },);

/**
 What a folder answers about itself, which is how the volume holding a folder of a run is found: the path it really is, with every
 symbolic link resolved, and the device whose volume it is on.

 A caller reaches these folders through symbolic links, here as much as anywhere: `~/temp` is a link to another drive on this
 machine, and a volume named by the link would name neither drive.

 @param folder - the folder to ask about

 @returns the folder it really is, and the device whose volume holds it

 @example
 ```ts
 const answer = await answerAboutAFolder('/home/a/temp/agent/pi-runs',);
 ```
 */
async function answerAboutAFolder(folder: string,): Promise<Awaited<ReturnType<AnswerAboutAFolder>>> {
  /**
   The folder with every symbolic link on the way to it resolved.
   */
  const realPath = await realpath(folder,);
  return {
    realPath,
    device: (await stat(realPath,)).dev,
  };
}

/**
 Reads the volume a folder is on, from the setting a caller gave for that folder or from the folder itself.

 @param folder - the folder whose volume is read

 @returns the volume's own path

 @example
 ```ts
 const volume = await volumeOf('/home/a/temp/agent/pi-runs',);
 ```
 */
async function volumeOf(folder: string,): Promise<string> {
  return givenVolumesOfFolders.get(folder,)
    ?? await volumeOfFolder({
      folder,
      answerAbout: answerAboutAFolder,
    },);
}

/**
 Reads how much room a volume has, from the setting a caller gave for that volume or from the disk itself.

 @param volume - the volume whose room is read

 @returns the room the volume has, and the volume it is on

 @example
 ```ts
 const room = roomOf(BUILD_VOLUME,);
 ```
 */
function roomOf(volume: string,): FreeSpace {
  /**
   The room a caller gave this volume, when it gave one.
   */
  const given = givenRooms.get(volume,);
  return (given === undefined)
    ? freeSpaceOnVolume({
      volume,
      statfs: statfsSync,
    },)
    : {
      volume,
      freeBytes: given,
    };
}

/**
 What this run of the command was asked for.
 */
const options = parsePoolGateOptions(argv.slice(2,),);

/**
 The folder Pi keeps its session files in, whose drive filled on 2026-11-12 while every floor read free.
 */
const sessionFolder = sessionFolderOf({
  home: env.HOME ?? homedir(),
  sessionFolder: env[SESSION_FOLDER_VARIABLE] ?? '',
},);

/**
 Each volume a new run cannot work without, one entry per need, and the floor that need sets. A volume two needs share is read and
 named once, at the strictest of those floors.
 */
const needs: readonly VolumeAgainstItsFloor[] = [
  {
    volume: BUILD_VOLUME,
    floorBytes: BUILD_VOLUME_FLOOR_BYTES,
  },
  {
    volume: await volumeOf(options.runsFolder,),
    floorBytes: RUNS_FOLDER_FLOOR_BYTES,
  },
  {
    volume: await volumeOf(sessionFolder,),
    floorBytes: SESSION_FILES_FLOOR_BYTES,
  },
];

/**
 The room of each distinct volume, with the strictest floor set for it.
 */
const volumes: readonly FreeSpaceAgainstItsFloor[] = strictestFloorPerVolume({ volumes: needs, },)
  .map(function withItsRoom(volume,) {
    return {
      space: roomOf(volume.volume,),
      floorBytes: volume.floorBytes,
    };
  },);

/**
 How many agents the machine may carry at once, and how many of those this gate cannot see.
 */
const machineCap: MachineCap = {
  cap: machineCapIn({ listed: env[MACHINE_CAP_VARIABLE] ?? '', },),
  otherAgentsGoing: otherAgentsGoingIn({ listed: env[OTHER_AGENTS_GOING_VARIABLE] ?? '', },),
};

/**
 The pools this run counts against: the shipped table, with the limits a caller set in the environment overriding them, so a pool
 whose provider has a weekly or monthly quota is closed and reopened without a code change.
 */
const pools = poolsWithTheseLimits({ limits: limitsIn({ listed: env[POOL_LIMITS_VARIABLE] ?? '', },), },);

/**
 The runs going now, and every launcher that has not made its unit yet.
 */
const runs = await goingRunsNow({
  run: runCommandWithChildProcess,
  processFolder: PROCESS_FOLDER,
  launcherEntryNames: [
    ...LAUNCHER_ENTRY_NAMES,
    ...REPLACED_LAUNCHER_ENTRY_NAMES,
  ],
  pools,
  onProblem: function onProblem(problem,) {
    // The pools are counted from what could be read; a unit whose own command line could not be read still holds the fallback
    // pool, and saying so is what tells a reader which run that was.
    process.stderr
      .write(`${problem}\n`,);
  },
},);

/**
 What the gate answers.
 */
const outcome = await gateOutcomeOf({
  options,
  runs,
  tallies: talliesOf({
    runs,
    pools,
  },),
  volumes,
  machineCap,
  sessionFileOf: function sessionFileOf(runFolder,) {
    return sessionFileOfRun({
      runsFolder: options.runsFolder,
      runFolder,
    },);
  },
},);

for (const line of outcome.lines) {
  console.log(line,);
}

if (outcome.action !== 'start') {
  process.exit((outcome.action === 'counts') ? 0 : outcome.status,);
}
if (outcome.action === 'start') {
  /**
   The launcher this package ships beside this command.
   */
  const launcher = await siblingCommandEntry({
    folder: import.meta.dirname,
    command: 'start-pi-run-in-a-worktree',
  },);
  /**
   What the launcher's process ended with, which this command answers with.
   */
  const launched = await runCommandShowingItsOutput({
    command: process.execPath,
    words: [
      launcher,
      ...outcome.arguments
        .words,
    ],
    environment: env,
  },);
  if (launched.startProblem !== undefined) {
    throw new Error(`the launcher could not be started: ${launched.startProblem}`,);
  }
  process.exitCode = launched.status ?? 1;
}

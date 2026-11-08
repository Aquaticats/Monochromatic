/**
 The files a failing check keeps beside its failure file: the log the session's unit wrote, and the CPU and disk
 accounting the kernel kept for that unit.

 A failure file holds what the app said it did, which is `ui_state` lines only. It cannot say what the app was waiting
 for, nor whether the unit never ran because it ran out of its quota, was throttled on the disk, or was simply not
 scheduled. Those answers sit in the unit's own output and in its control group, and both disappear with the scratch
 directory and the unit, so they are copied at the moment a check fails.

 @example
 ```ts
 import { sessionEvidenceFiles, } from './session-evidence.ts';

 const files = await sessionEvidenceFiles({ logPath: '/work/run/wide.log', unit: 'music-player-e2e-4242-wide', },);
 ```

 @packageDocumentation
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { runCommand, } from './command.ts';
import type { EvidenceFile, } from './failure-evidence.ts';

/**
 Mount point of the unified cgroup hierarchy, under which the kernel keeps every unit's accounting files.
 */
const CGROUP_ROOT = '/sys/fs/cgroup';

/**
 Name the unit's log is kept under beside a failure file. It holds the compositor's lines and the app's, because
 `StandardOutput` and `StandardError` of the unit both go to it.
 */
export const UNIT_LOG_NAME = 'unit.log';

/**
 The accounting files kept for the unit's control group, each under the name the kernel gives it. `cpu.stat` says
 whether the unit was throttled (`nr_throttled`, `throttled_usec`) and `io.pressure` says how long its tasks waited
 for the disk, which is what a cold start and a late answer under load both show.
 */
export const UNIT_ACCOUNTING_FILES: readonly string[] = [
  'cpu.stat',
  'io.pressure',
];

/**
 Where the control group of a session's unit is, or why it cannot be named.
 */
type UnitControlGroup = {
  /**
   Directory holding the unit's `cpu.stat` and `io.pressure`.
   */
  readonly directory: string;
} | {
  /**
   Why the unit has no control group, such as the unit having already stopped.
   */
  readonly reason: string;
};

/**
 Asks systemd where the control group of a session's unit is. The unit is the only handle the runner keeps on the
 processes of a session, so its name is what these figures are read by.
 
 @param unit - unit name without the `.service` suffix
 
 @returns the directory of the unit's control group, or why there is none
 
 @example
 ```ts
 await unitControlGroup({ unit: 'music-player-e2e-4242-wide', },);
 // { directory: '/sys/fs/cgroup/user.slice/user-1000.slice/.../music-player-e2e-4242-wide.service' }
 ```
 */
async function unitControlGroup({ unit, }: { readonly unit: string; },): Promise<UnitControlGroup> {
  /**
   Tagged logger naming the lookup.
   */
  const rl = tagged({
    tag: unitControlGroup.name,
    l: logger,
  },);
  /**
   How `systemctl show` ended.
   */
  const {
    status,
    stderr,
    stdout,
  } = await runCommand({
    command: 'systemctl',
    args: [
      '--user',
      'show',
      '--property=ControlGroup',
      '--value',
      `${unit}.service`,
    ],
  },);
  /**
   The cgroup path systemd printed, empty when the unit has none.
   */
  const relative = stdout.trim();
  rl.debug(`control group of ${unit}: "${relative}" (status ${status})`,);
  if ((status !== 0) || (relative === ''))
    return {
      reason: status === 0
        ? 'systemd names no control group for the unit'
        : `systemctl show exited with status ${status}: ${stderr.trim()}`,
    };
  return { directory: join(
    CGROUP_ROOT,
    relative,
  ), };
}

/**
 Reads one file to keep.

 @param name - name it is kept under beside the failure file

 @param source - path its content is read from

 @returns the content, or why there is none

 @example
 ```ts
 await readEvidenceFile({ name: 'unit.log', source: '/work/run/wide.log', },);
 // { kind: 'kept', name: 'unit.log', source: '/work/run/wide.log', text: '...' }
 ```
 */
export async function readEvidenceFile(
  {
    name,
    source,
  }: {
    readonly name: string;
    readonly source: string;
  },
): Promise<EvidenceFile> {
  /**
   Tagged logger naming the read.
   */
  const rl = tagged({
    tag: readEvidenceFile.name,
    l: logger,
  },);
  try {
    return {
      kind: 'kept',
      name,
      source,
      text: await readFile(
        source,
        'utf8',
      ),
    };
  }
  catch (error: unknown) {
    /**
     Why the file could not be read.
     */
    const reason = caughtValueText(error,);
    rl.warn(`could not read ${source} to keep beside the failure file: ${reason}`,);
    return {
      kind: 'missing',
      name,
      source,
      reason,
    };
  }
}

/**
 Reads the files a session keeps beside a failure file: the unit's log, then the accounting of its control group.

 The read happens while the session runs, because the log goes away with the scratch directory and the control group
 goes away with the unit.

 @param logPath - file the unit wrote its output to

 @param unit - unit name without the `.service` suffix

 @returns one file per name, each holding what was read or why nothing could be

 @example
 ```ts
 const files = await sessionEvidenceFiles({ logPath: '/work/run/wide.log', unit: 'music-player-e2e-4242-wide', },);
 ```
 */
export async function sessionEvidenceFiles(
  {
    logPath,
    unit,
  }: {
    readonly logPath: string;
    readonly unit: string;
  },
): Promise<readonly EvidenceFile[]> {
  /**
   The log of the unit, which the unit keeps writing until it stops.
   */
  const log = await readEvidenceFile({
    name: UNIT_LOG_NAME,
    source: logPath,
  },);
  /**
   Where the unit's accounting files are, or why they cannot be named.
   */
  const group = await unitControlGroup({ unit, },);
  if ('directory' in group) {
    /**
     The accounting files, read from the unit's own control group.
     */
    const accounting = await Promise.all(UNIT_ACCOUNTING_FILES.map(function accountingFile(name,): Promise<EvidenceFile> {
      return readEvidenceFile({
        name,
        source: join(
          group.directory,
          name,
        ),
      },);
    },));
    return [
      log,
      ...accounting,
    ];
  }
  /**
   Why the unit has no accounting to keep.
   */
  const { reason, } = group;
  return [
    log,
    ...UNIT_ACCOUNTING_FILES.map(function missing(name,): EvidenceFile {
      return {
        kind: 'missing',
        name,
        source: `${unit}.service`,
        reason,
      };
    },),
  ];
}

import { writeFile, } from 'node:fs/promises';
import {
  basename,
  dirname,
} from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  ABSENT,
  IO_BFQ_WEIGHT_MAX,
  IO_WEIGHT_MAX,
  IO_WEIGHT_MIN,
  NOT_MEASURED,
} from './constants.ts';
import {
  pathExists,
  readTextOrEmpty,
} from './files.ts';
import { IoWeightError, } from './io-weight-error.ts';

/**
 Start of the weight line that holds the cgroup-wide weight; per-device lines start with `major:minor` instead.
 */
const DEFAULT_LINE_PREFIX = 'default ';

/**
 One weight file of the cgroup and the largest weight the kernel accepts in it.
 */
type WeightTarget = Readonly<{
  /**
   Path of the weight file.
   */
  file: string;
  /**
   Largest accepted weight; the smallest is {@link IO_WEIGHT_MIN} for both files.
   */
  max: number;
}>;

/**
 What a pass did to one weight file: found no such file, wrote it, found it already right, or could not read or
 write it.
 */
type TargetOutcome = 'absent' | 'applied' | 'failed' | 'unchanged';

/**
 What a pass found out about one weight file, kept so the other file is still handled and every failure is reported
 together.
 */
type TargetReport = Readonly<{
  /**
   Path of the weight file.
   */
  file: string;
  /**
   What was done to the file.
   */
  outcome: TargetOutcome;
  /**
   Plain-language diagnostic naming the file and the system error; empty unless the outcome is `failed`.
   */
  problem: string;
  /**
   The caught error; undefined unless the outcome is `failed`.
   */
  cause: unknown;
}>;

/**
 Reads the cgroup-wide weight from the text of a weight file (`io.weight` and `io.bfq.weight` share the format).

 @param text - contents of the weight file

 @returns the weight, or {@link NOT_MEASURED} when there is no `default` line or it holds no number

 @example
 ```ts
 parseDefaultWeight('default 100\n259:0 50\n',); // 100
 ```
 */
export function parseDefaultWeight(text: string,): number {
  /**
   The line that starts with `default`, absent when the file lists only per-device weights.
   */
  const line = text.split('\n',)
    .find(function isDefaultLine(candidate,) {
      return candidate.startsWith(DEFAULT_LINE_PREFIX,);
    },);
  if (line === undefined) return NOT_MEASURED;
  return Number(line.slice(DEFAULT_LINE_PREFIX.length,)
    .trim(),);
}

/**
 Moves a wanted weight into the range a weight file accepts, because the kernel rejects a write outside it
 (`io.bfq.weight` with `ERANGE`, `io.weight` with `EINVAL`).

 @param weight - wanted weight

 @param max - largest weight the file accepts

 @returns the weight, raised to {@link IO_WEIGHT_MIN} or lowered to `max` when it lies outside

 @example
 ```ts
 clampWeight({ weight: 5_000, max: 1_000, },); // 1000
 ```
 */
export function clampWeight({
  weight,
  max,
}: Readonly<{
  weight: number;
  max: number;
}>,): number {
  return Math.min(
    max,
    Math.max(
      IO_WEIGHT_MIN,
      weight,
    ),
  );
}

/**
 Writes the wanted default weight into one weight file when it differs.
 A failure is reported in the result instead of thrown, so the other weight file is still handled.

 @param target - weight file and its range

 @param weight - wanted default weight, before clamping

 @param l - parent logger

 @returns what was found and done, with the diagnostic and the caught error when the file could not be read or written

 @example
 ```ts
 await reconcileTarget({ target: { file: '/sys/fs/cgroup/x/io.weight', max: 10_000, }, weight: 10, l, },);
 // { file: '/sys/fs/cgroup/x/io.weight', outcome: 'applied', problem: '', cause: undefined, }
 ```
 */
async function reconcileTarget({
  target,
  weight,
  l,
}: Readonly<{
  target: WeightTarget;
  weight: number;
  l: Logger;
}>,): Promise<TargetReport> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: reconcileTarget.name,
    l,
  },);
  /**
   Weight this file can hold.
   */
  const wanted = clampWeight({
    weight,
    max: target.max,
  },);
  try {
    /**
     Text now in the cgroup file, empty when the kernel does not offer this file here.
     */
    const text = (await readTextOrEmpty(target.file,)).trim();
    if (text === ABSENT) return {
      file: target.file,
      outcome: 'absent',
      problem: '',
      cause: undefined,
    };
    /**
     Default weight now in the file.
     */
    const current = parseDefaultWeight(text,);
    if (current === wanted) return {
      file: target.file,
      outcome: 'unchanged',
      problem: '',
      cause: undefined,
    };
    if (wanted !== weight) {
      rl.warn(
        `IO-WEIGHT-CLAMPED wanted default weight ${weight} is outside the range ${IO_WEIGHT_MIN} to ${target.max} `
          + `of ${target.file}; using ${wanted}`,
      );
    }
    await writeFile(
      target.file,
      `${DEFAULT_LINE_PREFIX}${wanted}`,
    );
    rl.info(`IO-WEIGHT-APPLIED claude-code-bash ${basename(target.file,)} default ${current} -> ${wanted}`,);
    return {
      file: target.file,
      outcome: 'applied',
      problem: '',
      cause: undefined,
    };
  } catch (error) {
    return {
      file: target.file,
      outcome: 'failed',
      problem: `could not set the default weight ${wanted} in ${target.file}: ${caughtValueText(error,)}`,
      cause: error,
    };
  }
}

/**
 Writes the wanted default weight into every weight file the cgroup offers, `io.bfq.weight` (read for disks that use
 the BFQ scheduler) and `io.weight` (read by the iocost controller), each clamped to its own range.
 Claude Code recreates the cgroup in every boot, so this runs on every pass like the CPU cap.
 A weight is a share against sibling cgroups, so it costs nothing while a disk is idle; per-device lines in the
 files are left as they are. A file that does not exist is skipped, and a failing file does not stop the other one.

 @param ioWeightFile - cgroup's `io.weight`; its directory is the cgroup directory

 @param ioBfqWeightFile - cgroup's `io.bfq.weight`

 @param weight - wanted default weight

 @param l - parent logger

 @returns whether any weight file was rewritten, false too when the cgroup does not exist yet

 @throws IoWeightError with reason `no-weight-file` when the cgroup directory exists but holds neither file
 (its io controller is off), or `file-access-failed` when a file could not be read or written

 @example
 ```ts
 await reconcileIoWeight({ ioWeightFile, ioBfqWeightFile, weight: 10, l, },); // true
 ```
 */
export async function reconcileIoWeight({
  ioWeightFile,
  ioBfqWeightFile,
  weight,
  l,
}: Readonly<{
  ioWeightFile: string;
  ioBfqWeightFile: string;
  weight: number;
  l: Logger;
}>,): Promise<boolean> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: reconcileIoWeight.name,
    l,
  },);
  /**
   Weight files in the order they are written.
   */
  const targets: readonly WeightTarget[] = [
    {
      file: ioBfqWeightFile,
      max: IO_BFQ_WEIGHT_MAX,
    },
    {
      file: ioWeightFile,
      max: IO_WEIGHT_MAX,
    },
  ];
  /**
   What was found and done for each file, in target order.
   */
  const reports = await Promise.all(targets.map(function reconcileOne(target,) {
    return reconcileTarget({
      target,
      weight,
      l: rl,
    },);
  },),);
  /**
   Reports of the files that could not be read or written.
   */
  const failures = reports.filter(function hasFailed(report,) {
    return report.outcome === 'failed';
  },);
  if (failures.length > 0) {
    throw new IoWeightError({
      reason: 'file-access-failed',
      files: failures.map(function fileOf(failure,) {
        return failure.file;
      },),
      message: `${
        failures.map(function problemOf(failure,) {
          return failure.problem;
        },)
          .join('; ',)
      }. The other weight file was still tried. Check that the user running claude-cpu-watch owns the cgroup, `
        + `and that the file accepts the weight (io.bfq.weight takes ${IO_WEIGHT_MIN} to ${IO_BFQ_WEIGHT_MAX}, `
        + `io.weight ${IO_WEIGHT_MIN} to ${IO_WEIGHT_MAX})`,
      cause: new AggregateError(
        failures.map(function causeOf(failure,) {
          return failure.cause;
        },),
        'errors of the weight files that failed',
      ),
    },);
  }
  /**
   Distinct outcomes of this pass; no file failed when this is reached.
   */
  const outcomes: ReadonlySet<TargetOutcome> = new Set(reports.map(function outcomeOf(report,) {
    return report.outcome;
  },),);
  if (outcomes.has('applied',)) return true;
  if (outcomes.has('unchanged',)) return false;
  // Both files are absent. Before Claude Code creates the cgroup that is normal; with the cgroup present it is not.
  if (!(await pathExists(dirname(ioWeightFile,),))) {
    rl.debug(`cgroup ${dirname(ioWeightFile,)} absent; nothing to weight yet`,);
    return false;
  }
  throw new IoWeightError({
    reason: 'no-weight-file',
    files: [
      ioBfqWeightFile,
      ioWeightFile,
    ],
    message: `neither ${ioBfqWeightFile} nor ${ioWeightFile} exists although the cgroup directory `
      + `${dirname(ioWeightFile,)} does, so the IO weight cannot be set. The io controller is probably off for this `
      + 'cgroup: read cgroup.controllers and cgroup.subtree_control in its parent directory, and check that the '
      + 'kernel was built with BFQ cgroup support (CONFIG_BFQ_GROUP_IOSCHED) or iocost (CONFIG_BLK_CGROUP_IOCOST)',
  },);
}

/**
 Runs {@link reconcileIoWeight} for the watcher pass: an {@link IoWeightError} is logged and swallowed, so a missing
 or refusing weight file never stops the CPU cap and the alerts of the same pass.

 @param ioWeightFile - cgroup's `io.weight`

 @param ioBfqWeightFile - cgroup's `io.bfq.weight`

 @param weight - wanted default weight

 @param l - parent logger

 @returns whether any weight file was rewritten, false after a logged failure

 @throws on any failure that is not an {@link IoWeightError}

 @example
 ```ts
 await reconcileIoWeightOrLog({ ioWeightFile, ioBfqWeightFile, weight: 10, l, },); // false when a write failed
 ```
 */
export async function reconcileIoWeightOrLog({
  ioWeightFile,
  ioBfqWeightFile,
  weight,
  l,
}: Readonly<{
  ioWeightFile: string;
  ioBfqWeightFile: string;
  weight: number;
  l: Logger;
}>,): Promise<boolean> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: reconcileIoWeightOrLog.name,
    l,
  },);
  try {
    return await reconcileIoWeight({
      ioWeightFile,
      ioBfqWeightFile,
      weight,
      l: rl,
    },);
  } catch (error) {
    if (!(error instanceof IoWeightError)) throw error;
    if (error.reason === 'no-weight-file') {
      rl.warn(`IO-WEIGHT-UNAVAILABLE ${error.message}`,);
      return false;
    }
    rl.error(`IO-WEIGHT-FAILED ${error.message}`,);
    return false;
  }
}

import { writeFile, } from 'node:fs/promises';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  ABSENT,
  NOT_MEASURED,
} from './constants.ts';
import { readTextOrEmpty, } from './files.ts';

/**
 Start of the `io.weight` line that holds the cgroup-wide weight; per-device lines start with `major:minor` instead.
 */
const DEFAULT_LINE_PREFIX = 'default ';

/**
 Reads the cgroup-wide weight from the text of an `io.weight` file.

 @param text - contents of `io.weight`

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
 Writes the wanted default weight into the cgroup's `io.weight` when it differs.
 Claude Code recreates the cgroup in every boot, so this runs on every pass like the CPU cap.
 A weight is a share against sibling cgroups, so it costs nothing while the disk is idle and only
 helps under the BFQ scheduler; per-device lines in the file are left as they are.

 @param ioWeightFile - the cgroup's `io.weight`

 @param weight - wanted default weight

 @param l - parent logger

 @returns whether `io.weight` was rewritten

 @example
 ```ts
 await reconcileIoWeight({ ioWeightFile, weight: 10, l, },); // true
 ```
 */
export async function reconcileIoWeight({
  ioWeightFile,
  weight,
  l,
}: Readonly<{
  ioWeightFile: string;
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
   Text now in the cgroup file, empty when the cgroup does not exist yet.
   */
  const text = (await readTextOrEmpty(ioWeightFile,)).trim();
  if (text === ABSENT) {
    rl.debug(`cgroup ${ioWeightFile} absent; nothing to weight yet`,);
    return false;
  }
  /**
   Default weight now in the cgroup.
   */
  const current = parseDefaultWeight(text,);
  if (current === weight) return false;
  await writeFile(
    ioWeightFile,
    `${DEFAULT_LINE_PREFIX}${weight}`,
  );
  rl.info(`IO-WEIGHT-APPLIED claude-code-bash io.weight default ${current} -> ${weight}`,);
  return true;
}

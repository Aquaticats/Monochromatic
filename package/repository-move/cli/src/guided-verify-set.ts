/**
 The choice of subvolumes to compare by content in the cut-over verification.

 @module
 */

import {
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { GuidedConfig, } from './guided-types.ts';
import { isDigits, } from './text-checks.ts';

/**
 How many of the largest and of the smallest subvolumes the content check covers: the plan's "six largest and six
 smallest", section "Verification of the real copy".
 */
const EXTREME_COUNT = 6;

/**
 One row of the size table: a size in MiB and the subvolume's path.
 */
type SizeEntry = {
  readonly size: number;
  readonly path: string;
};

/**
 Reads the snapshot records of the final pass as a map from copy name to source path.

 @param config - the configuration
 
 @returns rows of name and path, in name order
 */
async function finalMap(config: GuidedConfig,): Promise<readonly (readonly [
  string,
  string
])[]> {
  /**
   The files this step reads.
   */
  const files = (await readdir(config.moveState,)).filter(function isFinalSnapshot(file,) {
    return file.startsWith('f',) && file.endsWith('.snapshot.json',)
      && isDigits(file.slice(
        1,
        -'.snapshot.json'.length,
      ),);
  },)
    .toSorted();
  return await Promise.all(files.map(async function readRow(file,) {
    /**
     The parsed content of a record file.
     */
    const data = JSON.parse(await readFile(
      join(
        config.moveState,
        file,
      ),
      'utf8',
    ),) as { readonly path: string; };
    return [
      file.slice(
        0,
        -'.snapshot.json'.length,
      ),
      data.path,
    ] as const;
  },),);
}

/**
 Chooses the subvolumes to compare by content: the checkout, every worktree with a non-clean baseline row, and the
 six largest and six smallest by the size table taken before the window. Writes `final-map.tsv` and `verify-set.tsv`.

 @param config - the configuration
 
 @returns the line to show
 */
export async function selectVerify(config: GuidedConfig,): Promise<string> {
  /**
   The final-pass copies and the paths they were taken from.
   */
  const map = await finalMap(config,);
  await writeFile(
    join(
      config.wiz,
      'final-map.tsv',
    ),
    map.map(function toRow([name, path,],) {
      return `${name}\t${path}\n`;
    },)
      .join('',),
  );
  /**
   The paths the content comparison covers.
   */
  const wanted = new Set<string>();
  /**
   The first entry.
   */
  const first = map[0];
  if (first !== undefined) {
    wanted.add(first[1],);
  }
  /**
   The rows of the frozen baseline table.
   */
  const baseline = (await readFile(
    join(
      config.wiz,
      'baseline-frozen.tsv',
    ),
    'utf8',
  )).split('\n',)
    .slice(1,);
  for (const row of baseline) {
    /**
     The parts of the value this function uses, taken out of it.
     */
    const [path = '', , changed = '', untracked = '', unfinished = '', indexLock = '',] = row.split('\t',);
    if ((path !== '') && ((Number(changed || '0',) > 0) || (Number(untracked || '0',) > 0)
      || (unfinished !== '')
      || (indexLock === 'true'))) {
      wanted.add(path,);
    }
  }
  /**
   The size table, sorted by size.
   */
  const sizes = (await readFile(
    join(
      config.wiz,
      'subvolume-sizes.txt',
    ),
    'utf8',
  )).split('\n',)
    .filter(function notBlank(line,) {
      return line.trim() !== '';
    },)
    .map(function toSize(line,) {
      /**
       The parts of the value this function uses, taken out of it.
       */
      const [size = '0', ...rest] = line.split('\t',);
      return {
        size: Number(size,),
        path: rest.join('\t',),
      };
    },)
    .toSorted(function bySize(
      left: Readonly<SizeEntry>,
      right: Readonly<SizeEntry>,
    ) {
      return left.size - right.size;
    },);
  for (const entry of [
    ...sizes.slice(
      0,
      EXTREME_COUNT,
    ),
    ...sizes.slice(-EXTREME_COUNT,),
  ]) {
    wanted.add(entry.path,);
  }
  /**
   The entries this step selected.
   */
  const chosen = map.filter(function isWanted([, path,],) {
    return wanted.has(path,);
  },);
  await writeFile(
    join(
      config.wiz,
      'verify-set.tsv',
    ),
    chosen.map(function toRow([name, path,],) {
      return `${name}\t${path}\n`;
    },)
      .join('',),
  );
  return `content checks cover ${String(chosen.length,)} of ${String(map.length,)} subvolumes`;
}

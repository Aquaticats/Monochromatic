/**
 The computations the guided stages run in TypeScript instead of shell: the space rule and the window maximum. Each reads
 files the stages wrote and returns the text the stage's check reads.

 @module
 */

import {
  readFile,
  statfs,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { BYTES_PER_GIB, } from './constants.ts';
import type { GuidedConfig, } from './guided-types.ts';
import { wordsOf, } from './text-checks.ts';

/**
 Seconds in a minute, for the printed window maximum.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Whole GiB in a byte count, as text.

 @param bytes - the bytes
 
 @returns the GiB
 */
function gibibytes(bytes: number,): string {
  return String(Math.floor(bytes / BYTES_PER_GIB,),);
}

/**
 Reads a number from a file in the state folder.

 @param config - the configuration
 
 @param name - the file name
 
 @returns the number
 
 @throws Error when the file is missing or not a number
 */
async function numberFile(
  {
    config,
    name,
  }: {
    readonly config: GuidedConfig;
    readonly name: string;
  },
): Promise<number> {
  /**
   The value this step works on.
   */
  const value = Number((await readFile(
    join(
      config.wiz,
      name,
    ),
    'utf8',
  )).trim(),);
  if (!Number.isFinite(value,)) {
    throw new Error(`${name} in ${config.wiz} is not a number`,);
  }
  return value;
}

/**
 Free bytes available to the user on the filesystem holding a path.

 @param path - the path
 
 @returns bytes
 */
async function freeBytes(path: string,): Promise<number> {
  /**
   The filesystem information read for the path.
   */
  const info = await statfs(path,);
  return info.bavail * info.bsize;
}

/**
 The footprint of the checkout from `btrfs filesystem du --summarize --raw` output: exclusive plus set shared.

 @param text - the output; the last line is `Total Exclusive "Set shared" path`
 
 @returns bytes
 */
export function footprintOf(text: string,): number {
  /**
   The last row of the output.
   */
  const last = wordsOf(text.trim()
    .split('\n',)
    .at(-1,)
    ?? '',);
  return Number(last[1] ?? '0',) + Number(last[2] ?? '0',);
}

/**
 The space rule of the plan's section "Capacity policy": the 860 keeps at least the minimum free after the estimate
 plus a quarter margin, and the NVMe keeps its minimum.

 @param config - the configuration
 
 @returns the lines to show, ending in `pre-flight passed`
 
 @throws Error with the REFUSED line when a rule fails
 */
export async function preflight(config: GuidedConfig,): Promise<string> {
  /**
   The footprint of the nested checkout in bytes.
   */
  const mainBytes = footprintOf(await readFile(
    join(
      config.wiz,
      'main-du.txt',
    ),
    'utf8',
  ),);
  /**
   The size of the external worktrees in bytes.
   */
  const externalBytes = (await readFile(
    join(
      config.wiz,
      'external-du.txt',
    ),
    'utf8',
  )).split('\n',)
    .reduce(
    function sum(
      total,
      line,
    ) {
      return total + Number(wordsOf(line,)[0] ?? '0',);
    },
    0,
    );
  /**
   The estimate in bytes.
   */
  const total = mainBytes + externalBytes;
  /**
   The estimate plus the margin in bytes.
   */
  const need = total + Math.floor((total * config.marginPercent) / 100,);
  /**
   Free bytes on the 860.
   */
  const driveFree = await freeBytes(config.drive,);
  /**
   Free bytes on the NVMe.
   */
  const nvmeFree = await freeBytes(config.homeDir,);
  /**
   The lines of text this step works on.
   */
  const lines = [
    `nested checkout footprint (exclusive + set shared): ${gibibytes(mainBytes,)} GiB`,
    `external worktrees (du, no sharing): ${gibibytes(externalBytes,)} GiB`,
    `estimate: ${gibibytes(total,)} GiB; with the plan margin of ${String(config.marginPercent,)} percent: ${gibibytes(need,)} GiB`,
    `860 free now: ${gibibytes(driveFree,)} GiB; free after the estimate and margin: ${gibibytes(driveFree - need,)} GiB; required: at least ${String(config.minFreeGiB,)} GiB`,
    `NVMe free now: ${gibibytes(nvmeFree,)} GiB; required: at least ${String(config.nvmeMinFreeGiB,)} GiB while source snapshots exist`,
  ];
  if ((driveFree - need) < (config.minFreeGiB * BYTES_PER_GIB)) {
    throw new Error(`${lines.join('\n',)}\nREFUSED: the 860 would be left under ${String(config.minFreeGiB,)} GiB free.`,);
  }
  if (nvmeFree < (config.nvmeMinFreeGiB
    * BYTES_PER_GIB)) {
    throw new Error(`${lines.join('\n',)}\nREFUSED: the NVMe is under ${String(config.nvmeMinFreeGiB,)} GiB free.`,);
  }
  return `${lines.join('\n',)}\npre-flight passed`;
}

/**
 The stated maximum of the cut-over window, from times this runner measured plus one allowance. It may be shortened
 with `MOVE_WINDOW_MAX_SECONDS`, never lengthened. Writes `window-max-seconds`.

 @param config - the configuration
 
 @returns the lines to show
 
 @throws Error when the override is above the derived maximum
 */
export async function windowMaximum(config: GuidedConfig,): Promise<string> {
  /**
   The first entry.
   */
  const first = await numberFile({
    config,
    name: 'first-pass-seconds',
  },);
  /**
   The scan tool command.
   */
  const scan = await numberFile({
    config,
    name: 'scan-seconds',
  },);
  /**
   The footprint measurement time in seconds.
   */
  const du = await numberFile({
    config,
    name: 'du-seconds',
  },);
  /**
   The window maximum derived from the measured times.
   */
  const derived = first + first
    + (2
      * scan)
    + (2
      * du);
  /**
   The lines of text this step works on.
   */
  const lines = [
    `first pass, measured: ${String(first,)} s (the final pass sends only what changed, so a full first pass is its upper bound)`,
    `one scan of every worktree, measured: ${String(scan,)} s (taken twice in the window: frozen baseline and staged table)`,
    `one footprint measurement of the checkout, measured: ${String(du,)} s (taken twice: source and copy)`,
    `content-check allowance, the author's choice: one more first-pass duration, ${String(first,)} s, because fsck, the checksum comparisons and the counts read the copy about once`,
    `derived maximum = first pass + content-check allowance + 2 x scan + 2 x footprint = ${String(derived,)} s`,
  ];
  /**
   The entries this step selected.
   */
  const chosen = config.windowMaxOverrideSeconds === 0 ? derived : config.windowMaxOverrideSeconds;
  if (chosen > derived) {
    throw new Error(`REFUSED: MOVE_WINDOW_MAX_SECONDS=${String(chosen,)} is above the derived maximum ${String(derived,)}; the window may be shortened, never lengthened.`,);
  }
  await writeFile(
    join(
      config.wiz,
      'window-max-seconds',
    ),
    `${String(chosen,)}\n`,
  );
  return `${lines.join('\n',)}\nstated maximum of the window: ${String(chosen,)} s (${String(Math.floor(chosen / SECONDS_PER_MINUTE,),)} minutes)`;
}

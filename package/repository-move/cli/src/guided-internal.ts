/**
 The computations the guided stages run in TypeScript instead of shell: the space rule, the window maximum, the choice
 of subvolumes to compare by content, the holder scan patterns, and the edit of the 860's mount unit. Each reads
 files the stages wrote and returns the text the stage's check reads.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  statfs,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import { BYTES_PER_GIB, } from './constants.ts';
import type { GuidedConfig, } from './guided-types.ts';

/**
 Seconds in a minute, for the printed window maximum.
 */
const SECONDS_PER_MINUTE = 60;

/**
 How many of the largest and of the smallest subvolumes the content check covers: the plan's "six largest and six
 smallest", section "Verification of the real copy".
 */
const EXTREME_COUNT = 6;

/**
 The option the 860 mount gains, the plan's adopted decision.
 */
export const SUBVOLUME_REMOVAL_OPTION = 'user_subvol_rm_allowed';

/**
 The mount options the unit has today, which the edit appends the new option to.
 */
const UNIT_MOUNT_OPTIONS = 'compress=zstd:3,noatime';

/**
 Reads a number from a file in the state folder.

 @param config - the configuration
 @param name - the file name
 @returns the number
 @throws Error when the file is missing or not a number
 */
async function numberFile(
  { config, name, }: {
    readonly config: GuidedConfig;
    readonly name: string;
  },
): Promise<number> {
  const value = Number((await readFile(join(config.wiz, name,), 'utf8',)).trim(),);
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
  const info = await statfs(path,);
  return info.bavail * info.bsize;
}

/**
 The footprint of the checkout from `btrfs filesystem du --summarize --raw` output: exclusive plus set shared.

 @param text - the output; the last line is `Total Exclusive "Set shared" path`
 @returns bytes
 */
export function footprintOf(text: string,): number {
  const last = text.trim().split('\n',).at(-1)?.trim().split(/\s+/v,) ?? [];
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
  const mainBytes = footprintOf(await readFile(join(config.wiz, 'main-du.txt',), 'utf8',),);
  const externalBytes = (await readFile(join(config.wiz, 'external-du.txt',), 'utf8',)).split('\n',).reduce(
    function sum(total, line,) {
      return total + Number(line.trim().split(/\s+/v,)[0] || '0',);
    },
    0,
  );
  const total = mainBytes + externalBytes;
  const need = total + Math.floor(total * config.marginPercent / 100,);
  const driveFree = await freeBytes(config.drive,);
  const nvmeFree = await freeBytes(config.homeDir,);
  const gib = (bytes: number,): string => String(Math.floor(bytes / BYTES_PER_GIB,),);
  const lines = [
    `nested checkout footprint (exclusive + set shared): ${gib(mainBytes,)} GiB`,
    `external worktrees (du, no sharing): ${gib(externalBytes,)} GiB`,
    `estimate: ${gib(total,)} GiB; with the plan margin of ${String(config.marginPercent,)} percent: ${gib(need,)} GiB`,
    `860 free now: ${gib(driveFree,)} GiB; free after the estimate and margin: ${gib(driveFree - need,)} GiB; required: at least ${String(config.minFreeGiB,)} GiB`,
    `NVMe free now: ${gib(nvmeFree,)} GiB; required: at least ${String(config.nvmeMinFreeGiB,)} GiB while source snapshots exist`,
  ];
  if (driveFree - need < config.minFreeGiB * BYTES_PER_GIB) {
    throw new Error(`${lines.join('\n',)}\nREFUSED: the 860 would be left under ${String(config.minFreeGiB,)} GiB free.`,);
  }
  if (nvmeFree < config.nvmeMinFreeGiB * BYTES_PER_GIB) {
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
  const first = await numberFile({ config, name: 'first-pass-seconds', },);
  const scan = await numberFile({ config, name: 'scan-seconds', },);
  const du = await numberFile({ config, name: 'du-seconds', },);
  const derived = first + first + 2 * scan + 2 * du;
  const lines = [
    `first pass, measured: ${String(first,)} s (the final pass sends only what changed, so a full first pass is its upper bound)`,
    `one scan of every worktree, measured: ${String(scan,)} s (taken twice in the window: frozen baseline and staged table)`,
    `one footprint measurement of the checkout, measured: ${String(du,)} s (taken twice: source and copy)`,
    `content-check allowance, the author's choice: one more first-pass duration, ${String(first,)} s, because fsck, the checksum comparisons and the counts read the copy about once`,
    `derived maximum = first pass + content-check allowance + 2 x scan + 2 x footprint = ${String(derived,)} s`,
  ];
  const chosen = config.windowMaxOverrideSeconds === 0 ? derived : config.windowMaxOverrideSeconds;
  if (chosen > derived) {
    throw new Error(`REFUSED: MOVE_WINDOW_MAX_SECONDS=${String(chosen,)} is above the derived maximum ${String(derived,)}; the window may be shortened, never lengthened.`,);
  }
  await writeFile(join(config.wiz, 'window-max-seconds',), `${String(chosen,)}\n`,);
  return `${lines.join('\n',)}\nstated maximum of the window: ${String(chosen,)} s (${String(Math.floor(chosen / SECONDS_PER_MINUTE,),)} minutes)`;
}

/**
 Reads the snapshot records of the final pass as a map from copy name to source path.

 @param config - the configuration
 @returns rows of name and path, in name order
 */
async function finalMap(config: GuidedConfig,): Promise<readonly (readonly [string, string])[]> {
  const files = (await readdir(config.moveState,)).filter(function isFinalSnapshot(file,) {
    return /^f\d+\.snapshot\.json$/v.test(file,);
  },).toSorted();
  return await Promise.all(files.map(async function readRow(file,) {
    const data = JSON.parse(await readFile(join(config.moveState, file,), 'utf8',),) as { readonly path: string; };
    return [file.slice(0, -'.snapshot.json'.length,), data.path,] as const;
  },),);
}

/**
 Chooses the subvolumes to compare by content: the checkout, every worktree with a non-clean baseline row, and the
 six largest and six smallest by the size table taken before the window. Writes `final-map.tsv` and `verify-set.tsv`.

 @param config - the configuration
 @returns the line to show
 */
export async function selectVerify(config: GuidedConfig,): Promise<string> {
  const map = await finalMap(config,);
  await writeFile(join(config.wiz, 'final-map.tsv',), map.map(function toRow([name, path,],) {
    return `${name}\t${path}\n`;
  },).join('',),);
  const wanted = new Set<string>();
  const first = map[0];
  if (first !== undefined) {
    wanted.add(first[1],);
  }
  const baseline = (await readFile(join(config.wiz, 'baseline-frozen.tsv',), 'utf8',)).split('\n',).slice(1,);
  for (const row of baseline) {
    const [path = '', , changed = '', untracked = '', unfinished = '', indexLock = '',] = row.split('\t',);
    if (path !== '' && (Number(changed || '0',) > 0 || Number(untracked || '0',) > 0 || unfinished !== '' || indexLock === 'true')) {
      wanted.add(path,);
    }
  }
  const sizes = (await readFile(join(config.wiz, 'subvolume-sizes.txt',), 'utf8',)).split('\n',).filter(function notBlank(line,) {
    return line.trim() !== '';
  },).map(function toSize(line,) {
    const [size = '0', ...rest] = line.split('\t',);
    return { size: Number(size,), path: rest.join('\t',), };
  },).toSorted(function bySize(left, right,) {
    return left.size - right.size;
  },);
  for (const entry of [...sizes.slice(0, EXTREME_COUNT,), ...sizes.slice(-EXTREME_COUNT,),]) {
    wanted.add(entry.path,);
  }
  const chosen = map.filter(function isWanted([, path,],) {
    return wanted.has(path,);
  },);
  await writeFile(join(config.wiz, 'verify-set.tsv',), chosen.map(function toRow([name, path,],) {
    return `${name}\t${path}\n`;
  },).join('',),);
  return `content checks cover ${String(chosen.length,)} of ${String(map.length,)} subvolumes`;
}

/**
 The folder names directly below the external worktrees folder, sorted.

 @param config - the configuration
 @returns the names
 */
export async function externalNames(config: GuidedConfig,): Promise<readonly string[]> {
  if (!await pathExists(config.externalWorktrees,)) {
    return [];
  }
  return (await readdir(config.externalWorktrees, { withFileTypes: true, },)).filter(function isFolder(entry,) {
    return entry.isDirectory();
  },).map(function nameOf(entry,) {
    return entry.name;
  },).toSorted();
}

/**
 The `-lname` patterns of the holder scan: every place a writer could hold open (the repository, the external
 worktrees, and each registered worktree elsewhere, with its spelling on the 860).

 @param config - the configuration
 @returns one pattern per line, sorted, without duplicates
 */
export async function holderPatterns(config: GuidedConfig,): Promise<readonly string[]> {
  const patterns = new Set<string>([`${config.repo}*`, `${config.externalWorktrees}*`,],);
  const porcelain = join(config.wiz, 'porcelain-start.txt',);
  if (await pathExists(porcelain,)) {
    for (const line of (await readFile(porcelain, 'utf8',)).split('\n',)) {
      const path = line.startsWith('worktree ',) ? line.slice('worktree '.length,) : '';
      if (path !== '' && !path.startsWith(config.repo,) && !path.startsWith(config.externalWorktrees,)) {
        patterns.add(`${path}*`,);
        if (path.startsWith(`${config.homeDir}/temp/`,)) {
          patterns.add(`${config.drive}/temp/${path.slice(`${config.homeDir}/temp/`.length,)}*`,);
        }
      }
    }
  }
  return [...patterns,].toSorted();
}

/**
 Builds the root holder-scan command for an output name.

 @param config - the configuration
 @param name - output name under the state folder
 @returns the shell text
 */
export async function holderScanCommand(
  { config, name, }: {
    readonly config: GuidedConfig;
    readonly name: string;
  },
): Promise<string> {
  const patterns = (await holderPatterns(config,)).map(function toTest(pattern,) {
    return `-lname '${pattern}'`;
  },).join(' -o ',);
  return `sudo /usr/bin/find /proc -maxdepth 3 \\( -name cwd -o -path '/proc/*/fd/*' \\) \\( ${patterns} \\) -printf '%p %l\\n' > ${config.wiz}/${name}.txt 2> ${config.wiz}/${name}.err`;
}

/**
 Edits a copy of the 860 mount unit: appends the removal option to the one `mount -o` line that carries the unit's
 current options, and refuses when that line is not found exactly once. Writes `mount-860.service.before` (the unit as
 it is, kept for the owner) and `mount-860.service.new` in the state folder; changes nothing else.

 @param config - the configuration
 @returns the lines to show, including the changed line
 @throws Error when the unit does not hold the expected line exactly once, or already has the option
 */
export async function prepareUnitEdit(config: GuidedConfig,): Promise<string> {
  const unit = await readFile(config.unitPath, 'utf8',);
  if (unit.includes(SUBVOLUME_REMOVAL_OPTION,)) {
    return `${config.unitPath} already holds ${SUBVOLUME_REMOVAL_OPTION}; the edit is not needed`;
  }
  const marker = `mount -o ${UNIT_MOUNT_OPTIONS} `;
  if (unit.split(marker,).length !== 2) {
    throw new Error(`${config.unitPath} does not hold exactly one '${marker.trim()}'; edit the unit by hand with: sudo systemctl edit --full mount-860.service`,);
  }
  await mkdir(config.wiz, { recursive: true, },);
  await writeFile(join(config.wiz, 'mount-860.service.before',), unit,);
  await writeFile(join(config.wiz, 'mount-860.service.new',), unit.replace(marker, `mount -o ${UNIT_MOUNT_OPTIONS},${SUBVOLUME_REMOVAL_OPTION} `,),);
  return `unit before: ${join(config.wiz, 'mount-860.service.before',)}\nunit after:  ${join(config.wiz, 'mount-860.service.new',)}\nchanged line: ${marker.trim()} -> mount -o ${UNIT_MOUNT_OPTIONS},${SUBVOLUME_REMOVAL_OPTION}`;
}

import { BYTES_PER_KIB, } from '@monochromatic-dev/module-const/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { DEFAULT_MAX_PACK_BYTES, } from './constants.ts';
import {
  isMeasured,
  UNMEASURED,
  type Measured,
} from './measure.ts';
import { objectsDirSize, } from './objects-size.ts';
import { measurePackBytes, } from './pack-bytes.ts';
import { spawnResult, } from './spawn.ts';
import type { Confidence, } from './types.ts';

/**
 Exact (or size-pack proxy) measurement of a complete local repository. Byte
 fields are absent when their measurement could not be obtained: the caller
 omits the corresponding signal rather than recording a fabricated zero. A
 missing `fullBytes` means no full-clone size could be measured at all, so no
 local estimator is contributed and the stream degrades to its prior.
 */
export type LocalExactResult = {
  readonly fullBytes?: number;
  readonly shallowBytes?: number;
  readonly confidence: Confidence;
  readonly basis: string;
  readonly footprintBytes?: number;
};

/**
 Argument vector that lists every object a full clone receives: everything
 reachable from local heads and tags, the ref classes a clone fetches. NOT
 `--all`, which would pull local-only remote-tracking refs a clone never sends.
 */
const FULL_REV_LIST: readonly string[] = [
  'rev-list',
  '--objects',
  '--branches',
  '--tags',
];

/**
 Argument vector listing only the tip snapshot, for the depth-1 shallow side.
 */
const TIP_REV_LIST: readonly string[] = [
  'rev-list',
  '--objects',
  '--max-count=1',
  'HEAD',
];

/**
 Packer argument vector shared by both sides, so the ratio is apples-to-apples
 on one well-packed pack.
 */
const PACK_ARGS: readonly string[] = [
  'pack-objects',
  '--stdout',
  '--delta-base-offset',
];

/**
 Reads the already-packed size from `git count-objects -v` (`size-pack`,
 reported in KiB) as a cheap high-confidence proxy for huge repos.
 
 @param path - repository directory
 
 @returns packed object-store bytes (zero is a valid packed size), or
   {@link UNMEASURED} when `count-objects` could not report it
 
 @example
 ```ts
 const bytes = await countObjectsSizePack({ path: '/repo' });
 ```
 */
export async function countObjectsSizePack({ path, }: { readonly path: string; },): Promise<Measured> {
  /**
   Captured `count-objects -v` report and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      path,
      'count-objects',
      '-v',
    ],
  },);
  if (exitCode !== 0)
    return UNMEASURED;
  /**
   `size-pack: <KiB>` line value, parsed by splitting on the colon.
   */
  const line = stdout
    .split('\n',)
    .find(function isSizePack(text,) {
      return text.startsWith('size-pack:',);
    },);
  if (line === undefined)
    return UNMEASURED;
  /**
   KiB figure after the colon.
   */
  const kib = Math.trunc(Number(
    (line.split(':',)
      .at(1,)
      ?? '').trim(),
  ),);
  return Number.isFinite(kib,) ? kib * BYTES_PER_KIB : UNMEASURED;
}

/**
 Best-effort measure of the shallow tip pack for a local repo. Cheap even on
 huge repos, so it runs on every path. Degrades to {@link UNMEASURED} rather
 than throwing: an empty, unborn, or timing-stalled tip can fail the pack, and
 the never-refuse contract needs a result the caller can fold into a snapshot,
 not a rejection that would crash the whole stream.
 
 @param path - repository directory
 
 @returns tip pack bytes, or {@link UNMEASURED} when the tip cannot be packed
 */
async function measureTip({ path, }: { readonly path: string; },): Promise<Measured> {
  /**
   Tagged logger naming the best-effort tip measurement.
   */
  const rl = tagged({
    tag: measureTip.name,
    l: logger,
  },);
  try {
    return await measurePackBytes({
      cwd: path,
      revListArgs: TIP_REV_LIST,
      packArgs: PACK_ARGS,
    },);
  }
  catch (error: unknown) {
    rl.debug(`tip pack measurement failed (${String(error,)}); shallow tip unmeasured`,);
    return UNMEASURED;
  }
}

/**
 Exactly measures a complete local repository's full and shallow object-store
 sizes by packing the clone-reachable object set with `pack-objects`, the same
 packing a fresh `git clone` produces. Correct by reachability, so alternates,
 `--shared` clones, and linked worktrees are handled without summing the
 alternate store. Above `maxPackBytes`, falls back to the `count-objects`
 size-pack proxy (still high confidence, wider band) to avoid repack-level cost.
 Never runs `git gc`/`git repack`, so the user's repo is never mutated.
 
 @param path - complete local repository directory
 
 @param maxPackBytes - size above which the size-pack proxy is used
 
 @returns full/shallow bytes, confidence, basis label, and storage footprint
 
 @example
 ```ts
 const result = await localExact({ path: '/repo', maxPackBytes: DEFAULT_MAX_PACK_BYTES });
 ```
 */
export async function localExact(
  {
    path,
    maxPackBytes = DEFAULT_MAX_PACK_BYTES,
  }: {
    readonly path: string;
    readonly maxPackBytes?: number
  },
): Promise<LocalExactResult> {
  /**
   Tagged logger naming the local-exact measurement.
   */
  const rl = tagged({
    tag: localExact.name,
    l: logger,
  },);

  /**
   Raw on-disk store footprint, a separate explicitly-labeled secondary metric
   (NOT the full-clone size: it can include unreachable or alternate objects);
   {@link UNMEASURED} when the store cannot be sized.
   */
  const footprint = await objectsDirSize({ repoPath: path, },);

  /**
   Packed size proxy, used to gate the heavy exact pack; {@link UNMEASURED}
   when `count-objects` could not report it.
   */
  const sizePack = await countObjectsSizePack({ path, },);

  /**
   Shallow tip pack, measured once up front (best-effort, never throws) and
   reused by every return path so no fallback re-invokes a call that can fail.
   */
  const tip = await measureTip({ path, },);

  /**
   Secondary sizes carried only when measured, so an absent measurement is
   omitted from the result rather than recorded as a fabricated zero.
   */
  const optionalSizes = {
    ...isMeasured(footprint,) ? { footprintBytes: footprint, } : {},
    ...isMeasured(tip,) ? { shallowBytes: tip, } : {},
  };

  if (isMeasured(sizePack,) && (sizePack > maxPackBytes)) {
    rl.debug(`repo size-pack ${String(sizePack,)}B exceeds cap ${String(maxPackBytes,)}B; size-pack proxy`,);
    return {
      basis: 'local count-objects size-pack (huge-repo fallback)',
      confidence: 'high',
      fullBytes: sizePack,
      ...optionalSizes,
    };
  }

  try {
    /**
     Full clone-reachable pack, near-exact for a fresh clone (the heavy step).
     */
    const fullBytes = await measurePackBytes({
      cwd: path,
      revListArgs: FULL_REV_LIST,
      packArgs: PACK_ARGS,
    },);
    rl.debug(`local exact: full=${String(fullBytes,)}B shallow=${isMeasured(tip,) ? String(tip,) : 'n/a'}B`,);
    return {
      basis: 'local pack-objects (exact)',
      confidence: 'very high',
      fullBytes,
      ...optionalSizes,
    };
  }
  catch (error: unknown) {
    rl.debug(`pack-objects failed (${String(error,)}); falling back to size-pack proxy`,);
    return {
      basis: 'local count-objects size-pack (pack-objects failed)',
      confidence: 'high',
      ...isMeasured(sizePack,) ? { fullBytes: sizePack, } : {},
      ...optionalSizes,
    };
  }
}

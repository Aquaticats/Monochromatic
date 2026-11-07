/**
 What a runner command needs before it starts, checked without changing anything, and the timestamp that names the
 aside folders of the swap and the rollback.

 @module
 */

import {
  lstat,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import {
  EXIT_PRECONDITION,
  GuidedStopError,
} from './guided-stop.ts';
import type {
  GuidedConfig,
  GuidedFlags,
} from './guided-types.ts';

/**
 Mode of the runner's log folder: the owner only.
 */
const LOG_FOLDER_MODE = 0o700;

/**
 The prepare stages that must be recorded as done before the cut-over starts.
 */
const REQUIRED_PREPARE_STAGES: readonly string[] = [
  'p4c-first',
  'p4d-external-first',
  'p4e-sizes',
  'm1b-mount-option',
];

/**
 Stops with the precondition status.

 @param message - why
 
 @throws GuidedStopError always
 */
function refuse(message: string,): never {
  throw new GuidedStopError({
    status: EXIT_PRECONDITION,
    message,
  },);
}

/**
 Tells whether a path is a symbolic link; an absent path is not.

 @param path - the path
 
 @returns true for a symbolic link
 */
async function isSymbolicLink(path: string,): Promise<boolean> {
  try {
    return (await lstat(path,)).isSymbolicLink();
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 A compact UTC timestamp for folder names, such as `20261107T183000Z`.

 @returns the stamp

 @example
 ```ts
 import { compactTimestamp, } from './guided-preconditions.ts';
 
 const result = compactTimestamp();
 ```
 */
export function compactTimestamp(): string {
  return `${new Date().toISOString()
    .split('.',)[0]
    ?? ''}Z`.replaceAll(
    '-',
    '',
    )
    .replaceAll(
    ':',
    '',
    );
}

/**
 The timestamp that names the aside folders of the swap and the rollback: kept in a file so every run of the cut-over,
 and the rollback, use the same one.

 @param config - the configuration
 
 @param create - whether to make and keep one when none exists
 
 @returns the stamp

 @example
 ```ts
 import { swapStamp, } from './guided-preconditions.ts';
 
 const result = await swapStamp({ config, create, },);
 ```
 */
export async function swapStamp(
  {
    config,
    create,
  }: {
    readonly config: GuidedConfig;
    readonly create: boolean;
  },
): Promise<string> {
  /**
   The path this step works on.
   */
  const path = join(
    config.wiz,
    'swap-timestamp',
  );
  if (await pathExists(path,)) {
    return (await readFile(
      path,
      'utf8',
    )).trim();
  }
  /**
   The timestamp that names the aside folders.
   */
  const stamp = compactTimestamp();
  if (create) {
    await mkdir(
      config.wiz,
      { recursive: true, },
    );
    await writeFile(
      path,
      `${stamp}\n`,
    );
  }
  return stamp;
}

/**
 Checks what a command needs before it starts, without changing anything.

 @param config - the configuration
 
 @param flags - the flags
 
 @throws GuidedStopError with the precondition status when a condition is not met

 @example
 ```ts
 import { requirePreconditions, } from './guided-preconditions.ts';
 
 await requirePreconditions({ config, flags, },);
 ```
 */
export async function requirePreconditions(
  {
    config,
    flags,
  }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
  },
): Promise<void> {
  if (process.getuid?.() === 0) {
    refuse('Run this as your own user, not as root: each root step asks sudo itself.',);
  }
  /**
   The current folder.
   */
  const here = process.cwd();
  if ((here === config.repo) || here.startsWith(`${config.repo}/`,)) {
    refuse('The current folder is inside the repository; the holder scans would report this shell. cd out of it first.',);
  }
  for (const tool of [
    config.copyTool,
    config.scanTool,
    config.nodePath,
  ]) {
    if (!await pathExists(tool,)) {
      refuse(`Missing: ${tool}`,);
    }
  }
  if (flags.command === 'cutover') {
    for (const item of REQUIRED_PREPARE_STAGES) {
      if (!await pathExists(join(
        config.wiz,
        'done',
        'prepare',
        item,
      ),)) {
        refuse(`Prepare stage ${item} is not recorded as done. Run prepare first.`,);
      }
    }
  }
  if ((flags.command === 'thaw') && await isSymbolicLink(config.repo,)) {
    refuse(`${config.repo} is already a link: the swap happened. Use the rollback command instead.`,);
  }
  await mkdir(
    join(
      config.wiz,
      'logs',
    ),
    {
      recursive: true,
      mode: LOG_FOLDER_MODE,
    },
  );
}

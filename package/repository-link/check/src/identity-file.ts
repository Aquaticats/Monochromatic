/**
 The identity file the check reads and the cut-over step writes: the plan's `~/.config/monochromatic-dev/repository-link.json`.

 @module
 */

import {
  chmod,
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import {
  IDENTITY_FILE_MODE,
  IDENTITY_RELATIVE_PATH,
} from './constants.ts';
import type { LinkIdentity, } from './link-types.ts';

/**
 What reading the identity file yields: whether it exists, and what it says.
 */
export type IdentityRead = {
  readonly found: boolean;
  readonly identity: LinkIdentity;
};

/**
 An identity with nothing recorded, returned when the file does not exist.
 */
const EMPTY_IDENTITY: LinkIdentity = {
  linkPath: '',
  linkText: '',
  canonicalRoot: '',
  mountPoint: '',
  driveUuid: '',
  nvmeUuid: '',
  rootCommit: '',
  phase: 'before',
};

/**
 Path of the identity file for a home folder.

 @param home - the home folder
 
 @returns the path
 */
export function identityPathFor(home: string,): string {
  return join(
    home,
    IDENTITY_RELATIVE_PATH,
  );
}

/**
 Reads the identity file.

 @param path - the identity file
 
 @returns the identity; `found` is false when the file is absent
 
 @throws Error when the file exists but is not valid JSON
 */
export async function readIdentity(path: string,): Promise<IdentityRead> {
  try {
    /**
     The parsed of this step.
     */
    const parsed = JSON.parse(await readFile(
      path,
      'utf8',
    ),) as Readonly<Record<string, string>>;
    return {
      found: true,
      identity: {
        linkPath: parsed.linkPath ?? '',
        linkText: parsed.linkText ?? '',
        canonicalRoot: parsed.canonicalRoot ?? '',
        mountPoint: parsed.mountPoint ?? '',
        driveUuid: parsed.driveUuid ?? '',
        nvmeUuid: parsed.nvmeUuid ?? '',
        rootCommit: parsed.rootCommit ?? '',
        phase: parsed.phase === 'after' ? 'after' : 'before',
      },
    };
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return {
        found: false,
        identity: EMPTY_IDENTITY,
      };
    }
    throw error;
  }
}

/**
 Writes the identity file through a temporary name and a rename.

 @param path - the identity file
 
 @param identity - what to record
 */
export async function writeIdentity(
  {
    path,
    identity,
  }: {
    readonly path: string;
    readonly identity: LinkIdentity;
  },
): Promise<void> {
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    `${path}.tmp`,
    `${JSON.stringify(
      identity,
      undefined,
      2,
    )}\n`,
  );
  await chmod(
    `${path}.tmp`,
    IDENTITY_FILE_MODE,
  );
  await rename(
    `${path}.tmp`,
    path,
  );
}

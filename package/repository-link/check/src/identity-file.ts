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
import { errorCodeOf, } from './error-code-of.ts';

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

 @example
 ```ts
 import { identityPathFor, } from '\@monochromatic-dev/repository-link-check';
 
 const result = identityPathFor(home,);
 ```
 */
export function identityPathFor(home: string,): string {
  return join(
    home,
    IDENTITY_RELATIVE_PATH,
  );
}

/**
 Builds the identity from parsed JSON of any shape: a field that is missing or not text becomes the empty text.

 @param parsed - the parsed JSON, of any shape

 @returns the identity
 */
function identityFrom(parsed: unknown,): LinkIdentity {
  /**
   The reader of one text field of the parsed value.

   @param key - the field name

   @returns the field when it is text, else the empty text
   */
  function textOf(key: string,): string {
    if (((typeof parsed) !== 'object') || (parsed === null)) {
      return '';
    }
    /**
     The field's value.
     */
    const value: unknown = Reflect.get(
      parsed,
      key,
    );
    return (typeof value) === 'string' ? value : '';
  }
  return {
    linkPath: textOf('linkPath',),
    linkText: textOf('linkText',),
    canonicalRoot: textOf('canonicalRoot',),
    mountPoint: textOf('mountPoint',),
    driveUuid: textOf('driveUuid',),
    nvmeUuid: textOf('nvmeUuid',),
    rootCommit: textOf('rootCommit',),
    phase: textOf('phase',) === 'after' ? 'after' : 'before',
  };
}

/**
 Reads the identity file.

 @param path - the identity file
 
 @returns the identity; `found` is false when the file is absent
 
 @throws Error when the file exists but is not valid JSON

 @example
 ```ts
 import { readIdentity, } from '\@monochromatic-dev/repository-link-check';
 
 const result = await readIdentity(path,);
 ```
 */
export async function readIdentity(path: string,): Promise<IdentityRead> {
  try {
    /**
     The parsed of this step.
     */
    const parsed: unknown = JSON.parse(await readFile(
      path,
      'utf8',
    ),);
    return {
      found: true,
      identity: identityFrom(parsed,),
    };
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
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

 @example
 ```ts
 import { writeIdentity, } from '\@monochromatic-dev/repository-link-check';
 
 await writeIdentity({ path, identity, },);
 ```
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

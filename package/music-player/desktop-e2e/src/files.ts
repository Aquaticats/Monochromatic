/**
 Small file-system probes shared by the runner.
 
 @example
 ```ts
 import { pathExists, } from './files.ts';
 
 await pathExists('/etc/hostname',); // true
 ```
 
 @packageDocumentation
 */

import {
  access,
  mkdir,
} from 'node:fs/promises';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

/**
 Tells whether a path exists and can be reached.
 
 @param path - file or directory path
 
 @returns whether the path can be accessed
 
 @example
 ```ts
 await pathExists('/etc/hostname',); // true
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  /**
   Tagged logger naming the probe.
   */
  const rl = tagged({
    tag: pathExists.name,
    l: logger,
  },);
  try {
    await access(path,);
    return true;
  }
  catch (error: unknown) {
    rl.debug(`${path} is not reachable: ${String(error,)}`,);
    return false;
  }
}

/**
 Creates a directory and any missing parents; an existing directory is fine.
 
 @param path - directory path
 
 @example
 ```ts
 await ensureDirectory('/work/run/shots',);
 ```
 */
export async function ensureDirectory(path: string,): Promise<void> {
  await mkdir(
    path,
    { recursive: true, },
  );
}

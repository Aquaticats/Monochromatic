import {
  directoryNamed,
  findRoot,
  type RootFilesystem,
  RootNotFoundError,
} from '@monochromatic-dev/fs-path-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { resolve, } from 'node:path';

import { l as parentLogger, } from '../logger.ts';

/**
 Tagged logger for dependency-root discovery.
 */
const l = tagged({
  tag: 'stalenessRoot',
  l: parentLogger,
},);

/**
 Directory name searched while locating the workspace dependency root.
 */
export const NODE_MODULES_DIRECTORY_NAME = 'node_modules';

/**
 Marker for the nearest ancestor that owns a `node_modules` directory.
 A `node_modules` that is a file, or a link to something other than a
 directory, does not match; a broken or looping link makes the probe throw,
 which the walk propagates.
 */
const NODE_MODULES_MARKER = directoryNamed(NODE_MODULES_DIRECTORY_NAME,);

/**
 Options for {@link findNodeModulesRoot}.
 */
export type FindNodeModulesRootOptions = {
  /**
   Directory where the upward walk starts; a relative path resolves against
   the process working directory.
   */
  readonly startDirectory: string;
  /**
   Filesystem the walk probes. Omitted, the runtime's real filesystem;
   tests pass an in-memory adapter.
   */
  readonly fs?: RootFilesystem;
};

/**
 Nearest ancestor of `startDirectory` (itself included) that owns a
 `node_modules` directory, or the resolved start directory when no ancestor
 does, so the caller can still place its cache under the start directory.
 Filesystem errors other than a missing entry propagate, so a corrupt
 `node_modules` surfaces instead of being walked past.

 @param startDirectory - where the upward walk starts

 @param fs - filesystem seam, an in-memory adapter in tests

 @returns absolute directory that owns `node_modules`, else the resolved start directory

 @example
 ```ts
 const root = await findNodeModulesRoot({ startDirectory: process.cwd() });
 ```
 */
export async function findNodeModulesRoot({
  startDirectory,
  fs,
}: FindNodeModulesRootOptions,): Promise<string> {
  /**
   Absolute start directory, also the fallback answer.
   */
  const start = resolve(startDirectory,);
  try {
    return await findRoot(
      fs === undefined
        ? {
          cwd: start,
          marker: NODE_MODULES_MARKER,
        }
        : {
          cwd: start,
          fs,
          marker: NODE_MODULES_MARKER,
        },
    );
  }
  catch (error: unknown) {
    if (!(error instanceof RootNotFoundError))
      throw error;
    l.debug(`no ${NODE_MODULES_DIRECTORY_NAME} at or above ${start}; using the start directory`,);
    return start;
  }
}

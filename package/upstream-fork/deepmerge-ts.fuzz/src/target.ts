/**
 The deepmerge-ts build under test.

 By default the installed npm release (`deepmerge-ts` from the workspace
 catalog), which is what consumers run. Setting `DEEPMERGE_FUZZ_TARGET` to a
 module path (for example a local fork build) runs every property and
 known-defect test against that build instead, so a fix can be verified
 before a PR (Q9 in `doc/handover/deepmerge-ts-hardening.md`).

 @module
 */

import {
  isAbsolute,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import type * as DeepmergeTs from 'deepmerge-ts';

/**
 Module shape every target build must provide.
 */
export type DeepmergeTarget = typeof DeepmergeTs;

/**
 Environment variable naming an alternative build to test.
 */
export const TARGET_ENV_NAME = 'DEEPMERGE_FUZZ_TARGET';

/**
 Resolve the import specifier of the build under test.

 @returns Package name for the npm release, or a file URL for an override
   path (relative paths resolve against the working directory).

 @example
 ```ts
 const specifier = targetSpecifier();
 ```
 */
export function targetSpecifier(): string {
  /**
   Override path, when one is configured.
   */
  const override = process.env[TARGET_ENV_NAME];
  if ((override === undefined) || (override === ''))
    return 'deepmerge-ts';
  return pathToFileURL(isAbsolute(override,) ? override : resolve(override,),)
    .href;
}

/**
 Entry points every property and known-defect test calls on the target.
 */
const REQUIRED_EXPORTS = [
  'deepmerge',
  'deepmergeCustom',
  'deepmergeFastUnsafe',
  'deepmergeInto',
  'deepmergeIntoCustom',
  'deepmergeIntoFastUnsafe',
] as const;

/**
 Whether a loaded module provides the entry points the tests call.

 @param loaded - Module namespace from the dynamic import.

 @returns Whether every required export is a function.

 @example
 ```ts
 isDeepmergeTarget(await import('deepmerge-ts')); // true
 ```
 */
function isDeepmergeTarget(loaded: unknown,): loaded is DeepmergeTarget {
  return ((typeof loaded) === 'object')
    && (loaded !== null)
    && REQUIRED_EXPORTS.every(function isExportedFunction(name,) {
      return (typeof Reflect.get(
        loaded,
        name,
      )) === 'function';
    },);
}

/**
 Module namespace of the configured build, before its shape is checked.
 */
const loadedTarget: unknown = await import(targetSpecifier());
if (!isDeepmergeTarget(loadedTarget,))
  throw new Error(`${targetSpecifier()} does not export the deepmerge-ts entry points the tests call`,);

/**
 Loaded build under test, shared by every test file.
 */
export const target: DeepmergeTarget = loadedTarget;

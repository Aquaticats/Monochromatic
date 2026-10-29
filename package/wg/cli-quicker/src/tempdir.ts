import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

/**
 A private temp directory removed automatically when the scope exits.
 
 Implements {@link Symbol.asyncDispose} so it can be bound with `await using`,
 guaranteeing cleanup without an explicit `try...finally`.
 */
export type TempDir = {
  /**
   Absolute path of the created directory.
   */
  readonly path: string;

  /**
   Removes the directory and its contents; invoked by `await using` scope exit.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a root-only temp directory that deletes itself on scope exit.
 
 Used for the transient `wg addconf` config file, which holds a private key
 and must not persist after the interface is configured.
 
 @returns A disposable wrapper around the created directory.
 
 @example
 ```ts
 await using dir = await makeTempDir();
 // use dir.path ...
 ```
 */
export async function makeTempDir(): Promise<TempDir> {
  /**
   Freshly created private temp directory path.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'wg-quicker-',
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

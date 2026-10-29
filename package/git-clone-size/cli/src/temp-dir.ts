import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

/**
 A scratch directory that removes itself when the enclosing `await using`
 scope exits, success or throw alike.
 */
export type DisposableDir = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a uniquely-named temporary directory under the OS temp root, bound to
 `await using` so it is recursively removed on scope exit. Probes clone into
 it; nothing here ever touches the user's own repository.
 
 @param prefix - leading name fragment for the generated directory
 
 @returns disposable handle exposing `path` and an async disposer
 
 @example
 ```ts
 await using dir = await makeTempDir({ prefix: 'gcs-shallow-' });
 // clone into dir.path; removed automatically at scope end
 ```
 */
export async function makeTempDir({ prefix, }: { readonly prefix: string; },): Promise<DisposableDir> {
  /**
   Absolute path of the freshly created scratch directory.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    prefix,
  ),);

  /**
   Tagged logger naming the temp-dir lifecycle.
   */
  const rl = tagged({
    tag: makeTempDir.name,
    l: logger,
  },);
  rl.debug(`created ${path}`,);

  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      rl.debug(`removing ${path}`,);
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

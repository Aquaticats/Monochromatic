import {
  mkdir,
  mkdtemp,
  rm,
  stat,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  findNodeModulesUp,
  NO_NODE_MODULES_FOUND,
} from '@monochromatic-dev/logger-core/ts/sink/file-project.ts';

/**
 Directory that holds this tool's clone scratch: `git-clone-size` under the
 nearest `node_modules/.monochromatic`, or under the user cache directory when
 the working directory is outside any project. Never the system temporary
 directory, whose backing device is unknowable.

 @returns absolute path of the existing scratch parent directory
 */
async function scratchParent(): Promise<string> {
  /**
   Nearest ancestor `node_modules`, or the not-found sentinel outside a project.
   */
  const nodeModules = await findNodeModulesUp({
    cwd: process.cwd(),
    stat,
    dirname,
    join,
  },);
  /**
   Directory under which every monochromatic tool keeps its scratch.
   */
  const base = nodeModules === NO_NODE_MODULES_FOUND
    ? join(process.env['XDG_CACHE_HOME'] ?? join(homedir(), '.cache',), 'monochromatic-dev',)
    : join(nodeModules, '.monochromatic',);
  /**
   This tool's own scratch parent.
   */
  const parent = join(base, 'git-clone-size',);
  await mkdir(parent, { recursive: true, },);
  return parent;
}

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
    await scratchParent(),
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

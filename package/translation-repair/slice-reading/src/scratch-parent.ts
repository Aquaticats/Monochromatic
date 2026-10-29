import {
  mkdir,
  stat,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  findNodeModulesUp,
  NO_NODE_MODULES_FOUND,
} from '@monochromatic-dev/logger-core/ts/sink/file-project.ts';

/**
 Directory that holds this tool's scratch: `translation-repair-slice-reading` under the nearest
 `node_modules/.monochromatic`, or under the user cache directory when the
 working directory is outside any project. Never the system temporary
 directory, whose backing device is unknowable.

 Where the directory lands is read from the working directory, the environment and the home directory, which a
 caller leaves to the process's own and a test hands in as throwaway ones, so no case touches the real ones.

 @param cwd - directory the search for a project starts from; the process's working directory unless a test fixes it

 @param env - environment whose `XDG_CACHE_HOME` moves the user cache directory; the process's own unless a test fixes
 it

 @param home - home directory the cache falls back to when `XDG_CACHE_HOME` is unset; the current user's unless a test
 fixes it

 @returns absolute path of the existing scratch parent directory

 @example
 ```ts
 const directory = await mkdtemp(join(await scratchParent(), 'work-',),);
 ```
 */
export async function scratchParent(
  {
    cwd = process.cwd(),
    env = process.env,
    home = homedir(),
  }: {
    readonly cwd?: string;
    readonly env?: Readonly<NodeJS.ProcessEnv>;
    readonly home?: string;
  } = {},
): Promise<string> {
  /**
   Nearest ancestor `node_modules`, or the not-found sentinel outside a project.
   */
  const nodeModules = await findNodeModulesUp({
    cwd,
    stat,
    dirname,
    join,
  },);
  /**
   Directory under which every monochromatic tool keeps its scratch.
   */
  const base = (nodeModules === NO_NODE_MODULES_FOUND)
    ? join(
      env.XDG_CACHE_HOME ?? join(
        home,
        '.cache',
      ),
      'monochromatic-dev',
    )
    : join(
      nodeModules,
      '.monochromatic',
    );
  /**
   This tool's own scratch parent.
   */
  const parent = join(
    base,
    'translation-repair-slice-reading',
  );
  await mkdir(
    parent,
    { recursive: true, },
  );
  return parent;
}

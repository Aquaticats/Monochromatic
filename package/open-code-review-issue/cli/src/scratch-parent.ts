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
 Directory that holds this tool's scratch: `open-code-review-issue` under the nearest
 `node_modules/.monochromatic`, or under the user cache directory when the
 working directory is outside any project. Never the system temporary
 directory, whose backing device is unknowable.

 @returns absolute path of the existing scratch parent directory

 @example
 ```ts
 const directory = await mkdtemp(join(await scratchParent(), 'work-',),);
 ```
 */
export async function scratchParent(): Promise<string> {
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
   User cache directory, the fallback when no enclosing project names a
   `node_modules`.
   */
  const userCacheDirectory = process.env
    .XDG_CACHE_HOME
    ?? join(
      homedir(),
      '.cache',
    );
  /**
   Directory under which every monochromatic tool keeps its scratch.
   */
  const base = nodeModules === NO_NODE_MODULES_FOUND
    ? join(
      userCacheDirectory,
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
    'open-code-review-issue',
  );
  await mkdir(
    parent,
    { recursive: true, },
  );
  return parent;
}

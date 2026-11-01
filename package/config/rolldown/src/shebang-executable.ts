import { chmod, } from 'node:fs/promises';
import { join, } from 'node:path';

import type {
  Plugin,
} from 'rolldown';

/**
 Owner/group/other read-execute mode applied to shebang entry outputs,
 matching what tsdown set and what direct hook execution
 (`${CLAUDE_PLUGIN_ROOT}/bundle/node/index.mjs`) requires.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Bundle-entry fields the chmod step reads,
 project-owned and deeply readonly so the foreign rolldown bundle
 borrows without exposing mutable structure.
 
 @example
 ```ts
 const entry: ShebangOutputEntry = { code: '#!/usr/bin/env node\nrun();', type: 'chunk' };
 ```
 */
type ShebangOutputEntry = {
  readonly code: string;
  readonly type: 'chunk';
} | {
  readonly fileName: string;
  readonly type: 'asset';
};

/**
 Output-bundle slice the chmod step reads,
 deeply readonly at every level.
 
 @example
 ```ts
 const bundle: ShebangOutputBundle = { 'cli.mjs': { code: '#!\nrun();', type: 'chunk' } };
 ```
 */
type ShebangOutputBundle = Readonly<Record<string, ShebangOutputEntry>>;

/**
 Mark shebang-carrying output chunks executable after write.
 
 Raw rolldown preserves `#!` lines in entry chunks but does not set the
 executable bit tsdown set; committed Claude Code plugin bundles and CLI
 bins are executed directly, so the bit is load-bearing.
 
 @returns Rolldown plugin chmodding shebang outputs in `writeBundle`.
 
 @example
 ```ts
 plugins: [shebangExecutablePlugin(),],
 ```
 */
export function shebangExecutablePlugin(): Plugin {
  return {
    name: 'monochromatic:shebang-executable',
    async writeBundle(
      options,
      bundle: ShebangOutputBundle,
    ): Promise<void> {
      await Promise.all(Object.entries(bundle,)
        .map(
        async function markExecutable([fileName, output,],): Promise<void> {
          if ((output.type === 'chunk')
            && output.code
              .startsWith('#!',)) {
            await chmod(
              join(
                options.dir ?? '.',
                fileName,
              ),
              EXECUTABLE_MODE,
            );
          }
        },
        ),);
    },
  };
}

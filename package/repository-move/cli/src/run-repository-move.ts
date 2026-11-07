/**
 Command-line entry of the guided runner: `run-repository-move prepare | cutover | thaw | rollback | status`, with
 `--dry-run` (show every stage, run and record nothing), `--redo=<stage id>` (offer a completed stage again) and
 `--assert=<stage id>` (state the fact a stage needs: for example that no agent works in any worktree). It runs from
 the repository's TypeScript source with the absolute path of the node that runs it, and passes that same path to every
 root step, because `sudo node` fails when node comes from the version manager.

 @module
 */

import { stdout, argv, execPath, exit, env, } from 'node:process';
import { realpath, } from 'node:fs/promises';
import { createGuidedConfig, } from './guided-config.ts';
import {
  parseGuidedFlags,
  runGuided,
} from './guided-runner.ts';
import { GuidedStop, } from './guided-engine.ts';

try {
  const flags = parseGuidedFlags(argv.slice(2,),);
  const config = await createGuidedConfig({ env, nodePath: await realpath(execPath,), },);
  exit(await runGuided({
    config,
    flags,
    out: function write(text: string,) {
      stdout.write(text,);
    },
  },),);
}
catch (error) {
  if (error instanceof GuidedStop) {
    console.error(error.message,);
    exit(error.status,);
  }
  throw error;
}

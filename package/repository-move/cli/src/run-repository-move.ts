/**
 Command-line entry of the guided runner: `run-repository-move prepare | cutover | thaw | rollback | status`, with
 `--dry-run` (show every stage, run and record nothing), `--redo=<stage id>` (offer a completed stage again) and
 `--assert=<stage id>` (state the fact a stage needs: for example that no agent works in any worktree). It runs from
 the repository's TypeScript source with the absolute path of the node that runs it, and passes that same path to every
 root step, because `sudo node` fails when node comes from the version manager.

 @module
 */

import { realpath, } from 'node:fs/promises';
import {
  argv,
  env,
  execPath,
  exit,
  stdout,
} from 'node:process';
import { createGuidedConfig, } from './guided-config.ts';
import {
  parseGuidedFlags,
  runGuided,
} from './guided-runner.ts';
import { GuidedStopError, } from './guided-stop.ts';

/**
 The environment without its unset entries, as plain text values.
 */
const definedEnvironment: Record<string, string> = {};
for (const name of Object.keys(env,)) {
  /**
   The value of one environment variable.
   */
  const value = env[name];
  if (value !== undefined) {
    definedEnvironment[name] = value;
  }
}

/**
 Writes text to standard output.

 @param text - the text
 */
function write(text: string,): void {
  stdout.write(text,);
}

try {
  /**
   The flags of this invocation.
   */
  const flags = parseGuidedFlags(argv.slice(2,),);
  /**
   The runner's configuration.
   */
  const config = await createGuidedConfig({
    env: definedEnvironment,
    nodePath: await realpath(execPath,),
  },);
  exit(await runGuided({
    config,
    flags,
    out: write,
  },),);
}
catch (error) {
  if (error instanceof GuidedStopError) {
    console.error(error.message,);
    exit(error.status,);
  }
  throw error;
}

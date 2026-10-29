import {
  fileNamed,
  findRoot,
} from '@monochromatic-dev/module-fs-path/ts';
import { join, } from 'node:path';

import {
  driftedPaths,
  enableCheckMode,
} from './check-mode.ts';
import { setActiveConfigPath, } from './context.ts';
import { l, } from './logger.ts';
import { startWatching, } from './watch/watch.ts';

//region CLI entry point: finds and imports file-enforcer.config.ts, optionally watches

/**
 Config file name to search for upward from cwd
 */
const CONFIG_NAME = 'file-enforcer.config.ts';

/**
 Raw CLI arguments after the script path
 */
const args = process.argv
  .slice(2,);

/**
 Whether --watch was passed, enabling continuous file watching
 */
const watchMode = args.includes('--watch',);

/**
 Whether --check was passed: run every generator without writing and fail on drift
 */
const checkMode = args.includes('--check',);

if (checkMode && watchMode)
  throw new Error('file-enforcer: --check and --watch cannot be combined; --check runs once and reports drift.',);
if (checkMode)
  enableCheckMode();

/**
 Positional args with flags stripped, used as an optional config path override
 */
const positionalArgs = args.filter(function isPositional(arg,): boolean {
  return !arg.startsWith('--',);
},);

/**
 Config path from CLI arg, or the config file in the nearest ancestor of cwd that holds one.
 A missing config rejects with `RootNotFoundError`, which names the file and the start directory.
 */
const configPath = positionalArgs[0]
  ?? join(
    await findRoot({ marker: fileNamed(CONFIG_NAME,), },),
    CONFIG_NAME,
  );

l.info(`loading config: ${configPath}`,);
setActiveConfigPath({ configPath, },);

// Importing the config executes it: the config uses top-level await.
await import(configPath);

if (watchMode)
  await startWatching(configPath,);

if (checkMode) {
  /**
   Files a real run would have changed.
   */
  const drifted = driftedPaths();
  if (drifted.length > 0) {
    // CLI output: the drift report is the command's result.
    console.error(
      [
        `file-enforcer --check: ${String(drifted.length,)} generated file(s) differ from what file-enforcer produces:`,
        ...drifted.map(function toLine(dest,): string {
          return `  ${dest}`;
        },),
        'Run `mise run sync:files` (or `node package/dev-script/file-enforcer/src/cli.ts`) and commit the result.',
        'If a file was edited by hand, edit its source instead; generated files are rewritten on every sync.',
      ].join('\n',),
    );
    process.exitCode = 1;
  } else {
    l.info('check: every generated file matches',);
  }
}

//endregion CLI entry point

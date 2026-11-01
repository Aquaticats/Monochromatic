#!/usr/bin/env node
/**
 CLI entry point for vmsync.
 Parses argv, dispatches to the appropriate command handler.
 
 @module
 */

// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import { bootVm, } from './boot.ts';
import { updateConfig, } from './config-cmd.ts';
import { importImage, } from './import.ts';
import { parser, } from './index-parsers.ts';
import { printVmList, } from './list.ts';
import { showStatus, } from './status.ts';
import { syncVm, } from './sync.ts';



//region Verbose flag: stripped before parsing; logger detects it from raw process.argv at import time

/**
 Flags consumed by infrastructure (logger) rather than the argument parser.
 Stripped from tokens before `--` so that guest commands are not affected.
 
 @example
 ```ts
 INFRA_FLAGS.has('--verbose'); // true
 ```
 */
const INFRA_FLAGS: ReadonlySet<string> = new Set(['--verbose',],);

/**
 Raw args after the script name.
 */
const rawArgs = process.argv
  .slice(2,);

/**
 Index of the `--` separator, or end-of-args when absent.
 Infrastructure flags are only stripped before this boundary.
 */
const doubleDashIndex = rawArgs.indexOf('--',);

/**
 Boundary past which tokens must not be touched.
 */
const boundary = (doubleDashIndex === (-1)) ? rawArgs.length : doubleDashIndex;

/**
 TODO: deprecate Optique
 Process argv with infrastructure flags removed only from the vmsync-owned prefix.
 The logger caches its own `process.argv` check at module load time
 (before this runs), so stripping here only affects the \@optique parser.
 */
const filteredArgs = rawArgs.filter(function keepNonInfraArgs(
  arg,
  i,
) {
  return (i >= boundary) || (!INFRA_FLAGS.has(arg,));
},);

//endregion Verbose flag

//region Dispatch: parse argv and route to the appropriate handler

/**
 TODO: deprecate Optique
 Parsed CLI result from process.argv.
 */
const args = runSync(
  parser,
  {
    programName: 'vmsync',
    args: filteredArgs,
    help: 'option',
    aboveError: 'help',
    brief: message`vmsync - multi-hypervisor VM image manager with incremental sync`,
    footer: message`Pass --verbose before the subcommand to enable debug logging.`,
  },
);

if (args.cmd
  === 'import') {
  await importImage(args,);
}
else if (args.cmd
  === 'boot')
  await bootVm(args.name,);
else if (args.cmd
  === 'sync')
  await syncVm(args.name,);
else if (args.cmd
  === 'status')
  await showStatus(args.name,);
else if (args.cmd
  === 'list')
  await printVmList();
else {
  await updateConfig(args,);
}

//endregion Dispatch

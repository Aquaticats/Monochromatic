#!/usr/bin/env node
import { inspect, } from 'node:util';
import type { ReadonlyDeep, } from 'type-fest';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import {
  resolveBackendKind,
  selectBackend,
} from './backend/registry.ts';
import { parser, } from './index-parsers.ts';



//region Infra flags: stripped before parsing (logger reads --verbose from raw argv; --backend selects the backend)

/**
 Valueless flags consumed by infrastructure rather than the argument parser.
 Only stripped from tokens before `--` so that VM commands like
 `mvm exec myvm -- --verbose cmd` preserve `--verbose` for the guest.
 
 @example
 ```ts
 INFRA_FLAGS.has('--verbose'); // true
 ```
 */
const INFRA_FLAGS: ReadonlySet<string> = new Set(['--verbose',],);

/**
 Long flag selecting the backend; consumes the following token as its value.
 */
const BACKEND_FLAG = '--backend';

/**
 Inline form `--backend=value`.
 */
const BACKEND_FLAG_EQ = `${BACKEND_FLAG}=`;

/**
 How many levels of nested causes a --verbose failure report shows.
 */
const CAUSE_CHAIN_DEPTH = 6;

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
 Boundary past which tokens belong to the VM command and must not be touched.
 */
const boundary = (doubleDashIndex === (-1)) ? rawArgs.length : doubleDashIndex;

/**
 TODO: deprecate Optique
 Captured backend value (`''` when `--backend` is absent) and the args handed
 to the optique parser, after stripping infrastructure flags from the
 mvm-owned prefix. Tokens at or past the `--` boundary are preserved verbatim.
 The scan runs in a named IIFE so its cursor/accumulator `let`s do not leak to
 the module body.
 */
const {
  backendValue,
  filteredArgs,
} = (function extractInfra(): {
  readonly backendValue: string;
  readonly filteredArgs: readonly string[];
} {
  /**
   Backend value captured from `--backend`, empty until found.
   */
  let captured = '';
  /**
   Args surviving the infra strip, handed to the optique parser.
   */
  const kept: string[] = [];
  /**
   Scan cursor; advances by two when consuming `--backend value`.
   */
  let idx = 0;
  while (idx < rawArgs.length) {
    /**
     Current token; guarded for the indexed-access undefined case.
     */
    const arg = rawArgs[idx];
    if (arg === undefined) {
      break;
    }
    if (idx >= boundary) {
      kept.push(arg,);
      idx += 1;
      continue;
    }
    if (INFRA_FLAGS.has(arg,)) {
      idx += 1;
      continue;
    }
    if (arg === BACKEND_FLAG) {
      captured = rawArgs[idx + 1] ?? '';
      idx += 2;
      continue;
    }
    if (arg.startsWith(BACKEND_FLAG_EQ,)) {
      captured = arg.slice(BACKEND_FLAG_EQ.length,);
      idx += 1;
      continue;
    }
    kept.push(arg,);
    idx += 1;
  }
  return {
    backendValue: captured,
    filteredArgs: kept,
  };
})();

//endregion Infra flags

//region Dispatch: parse argv, select the backend, and route to the operation

/**
 TODO: deprecate Optique
 Parsed CLI result from process.argv
 */
const args = runSync(
  parser,
  {
    programName: 'mvm',
    args: [...filteredArgs,],
    help: 'option',
    aboveError: 'help',
    brief: message`mvm - ephemeral VM manager`,
    footer:
      message`Pass --verbose before the subcommand to enable debug logging. Pass --backend <libvirt|hetzner> (or set MVM_BACKEND) to choose the backend; defaults to libvirt.`,
  },
);

/**
 Selects the backend and runs the parsed subcommand.

 @throws Error when the backend or the operation fails; the caller reports it

 @example
 ```ts
 await dispatch();
 ```
 */
async function dispatch(): Promise<void> {
  /**
   Selected backend, resolved from `--backend`/`MVM_BACKEND` (default libvirt)
   and guarded against the current platform before any work runs.
   */
  const backend = await selectBackend(resolveBackendKind(backendValue,),);

  if (args.cmd
    === 'create') {
    await (args.from
      !== undefined
      ? backend.clone({
        destination: args.name,
        source: args.from,
      },)
      : backend.create({
        name: args.name,
        ...(args.image !== undefined ? { image: args.image, } : {}),
        ...(args.serverType !== undefined ? { serverType: args.serverType, } : {}),
        ...(args.location !== undefined ? { location: args.location, } : {}),
      },));
  }
  else if (args.cmd
    === 'shell')
    await backend.shell({ name: args.name, },);
  else if (args.cmd
    === 'list') {
    /**
     All managed VMs queried from the selected backend.
     */
    const vms = await backend.list();
    if (vms.length
      === 0)
      console.error('no VMs found',);
    else {
      /**
       Column width for aligned output.
       */
      const NAME_COL_WIDTH = 24;
      vms.forEach(function printVm(vm: ReadonlyDeep<(typeof vms)[number]>,) {
        console.log(`${vm.name
          .padEnd(NAME_COL_WIDTH,)} ${vm.state}`,);
      },);
    }
  }
  else if (args.cmd
    === 'update')
    await backend.update();
  else if (args.cmd
    === 'destroy') {
    if (args.all)
      await backend.destroyAll();
    else if (args.name
      !== undefined)
      await backend.destroy({ name: args.name, },);
    else
      throw new Error('usage: mvm destroy <name> | --all',);
  }
  else if (args.cmd
    === 'exec') {
    /**
     Execution result with stdout, stderr, and exit code.
     */
    const result = await backend.exec({
      command: args.command,
      name: args.name,
    },);
    if (result.stdout
      .length
      > 0)
      process.stdout
        .write(result.stdout,);
    if (result.stderr
      .length
      > 0)
      process.stderr
        .write(result.stderr,);
    if (result.exitCode
      !== 0)
      process.exitCode = result.exitCode;
  }
  else if (args.cmd
    === 'push') {
    /**
     Guest path where the file is accessible inside the VM.
     */
    const guestFilePath = await backend.pushFile({
      name: args.name,
      hostPath: args.hostPath,
      guestPath: args.guestPath,
    },);
    console.log(`pushed ${args.hostPath} -> ${guestFilePath} in VM ${args.name}`,);
  }
  else if (args.cmd
    === 'pull') {
    /**
     File content retrieved from the guest.
     */
    const content = await backend.pullFile({
      name: args.name,
      guestPath: args.guestPath,
    },);
    /**
     Dynamic import keeps the pull-only branch off the cold-start dependency graph.
     */
    const { writeFile, } = await import('node:fs/promises');
    await writeFile(
      args.hostPath,
      content,
    );
    console.log(`pulled ${args.guestPath} -> ${args.hostPath} from VM ${args.name}`,);
  }
  else {
    /**
     Execution result from the ephemeral VM.
     */
    const result = await backend.run({
      command: args.command,
      ...(args.from !== undefined ? { from: args.from, } : {}),
    },);
    if (result.stdout
      .length
      > 0)
      process.stdout
        .write(result.stdout,);
    if (result.stderr
      .length
      > 0)
      process.stderr
        .write(result.stderr,);
    if (result.exitCode
      !== 0)
      process.exitCode = result.exitCode;
  }
}

//endregion Dispatch

//region Failure report: print what went wrong, not where in the bundle it was thrown

try {
  await dispatch();
}
catch (error) {
  // Exact terminal output. An error left uncaught is printed by Node with a line of the minified bundle and a stack through it,
  // which buries the message; the message is the diagnostic. --verbose keeps the stack and the chain of causes.
  console.error(
    rawArgs
      .slice(
      0,
      boundary,
      )
      .includes('--verbose',)
      ? inspect(
        error,
        { depth: CAUSE_CHAIN_DEPTH, },
      )
      : `mvm: ${caughtValueText(error,)}`,
  );
  // Caught here, so the failure exit status has to be set by hand.
  process.exitCode = 1;
}

//endregion Failure report

/**
 The guided runner of the move to the 860 drive: `prepare`, `cutover`, `thaw`, `rollback` and `status`. The coordinator
 runs it (the owner's decision of 2026-11-07); it replaced the guided bash script. Each stage shows its command, runs
 it, checks it, and stops at the first failure with what completed and how to resume. Root steps go through `sudo` with
 the absolute path of node; this module never runs `sudo` itself except through the commands of a real run, and a dry
 run runs nothing.

 @module
 */

import {
  lstat,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import {
  EXIT_PRECONDITION,
  GuidedStop,
  listProgress,
  renameCutoverStateAside,
  runPhase,
  windowRemaining,
  windowRunning,
} from './guided-engine.ts';
import { cutoverStages, } from './guided-cutover-stages.data.ts';
import { prepareStages, } from './guided-prepare-stages.data.ts';
import { rollbackStages, thawStages, } from './guided-undo-stages.data.ts';
import type {
  GuidedCommand,
  GuidedConfig,
  GuidedFlags,
  Stage,
} from './guided-types.ts';

/**
 The commands, for validating the first argument.
 */
const COMMANDS: readonly GuidedCommand[] = ['prepare', 'cutover', 'thaw', 'rollback', 'status',];

/**
 Tells whether a path is a symbolic link; an absent path is not.

 @param path - the path
 @returns true for a symbolic link
 */
async function isSymbolicLink(path: string,): Promise<boolean> {
  try {
    return (await lstat(path,)).isSymbolicLink();
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 Reads the command and flags from the arguments.

 @param argv - arguments after the program name
 @returns the flags
 @throws GuidedStop with the precondition status when the command is unknown
 */
export function parseGuidedFlags(argv: readonly string[],): GuidedFlags {
  const command = COMMANDS.find(function isNamed(name,) {
    return name === argv[0];
  },);
  if (command === undefined) {
    throw new GuidedStop({
      status: EXIT_PRECONDITION,
      message: 'Usage: run-repository-move prepare | cutover | thaw | rollback | status [--dry-run] [--redo=<stage id>] [--assert=<stage id>]',
    },);
  }
  /**
   Collects every `--name=value` among the arguments.

   @param name - the flag name
   @returns the values
   */
  function valuesOf(name: string,): readonly string[] {
    const lead = `--${name}=`;
    return argv.filter(function startsWithLead(argument,) {
      return argument.startsWith(lead,);
    },).map(function valueOf(argument,) {
      return argument.slice(lead.length,);
    },);
  }
  return {
    command,
    dryRun: argv.includes('--dry-run',),
    redo: valuesOf('redo',),
    asserted: valuesOf('assert',),
  };
}

/**
 The timestamp that names the aside folders of the swap and the rollback: kept in a file so every run of the cut-over,
 and the rollback, use the same one.

 @param config - the configuration
 @param create - whether to make and keep one when none exists
 @returns the stamp, or an empty string when none exists and none was asked for
 */
async function swapStamp(
  { config, create, }: {
    readonly config: GuidedConfig;
    readonly create: boolean;
  },
): Promise<string> {
  const path = join(config.wiz, 'swap-timestamp',);
  if (await pathExists(path,)) {
    return (await readFile(path, 'utf8',)).trim();
  }
  const stamp = new Date().toISOString().replaceAll(/[\-:]/gv, '',).replace(/\.\d+Z$/v, 'Z',);
  if (create) {
    await mkdir(config.wiz, { recursive: true, },);
    await writeFile(path, `${stamp}\n`,);
  }
  return stamp;
}

/**
 Prints the progress of one phase.

 @param config - the configuration
 @param phase - the phase
 @param out - the output
 */
async function printProgress(
  { config, phase, out, }: {
    readonly config: GuidedConfig;
    readonly phase: string;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  const lines = await listProgress({ config, phase, },);
  out(lines.length === 0 ? `Nothing in phase ${phase} has completed yet.\n` : `Completed so far (phase ${phase}):\n${lines.map(function indent(line,) {
    return `  - ${line}`;
  },).join('\n',)}\n`,);
}

/**
 Builds the stages of a command.

 @param config - the configuration
 @param flags - the flags
 @returns the stages, or an empty list for `status`
 */
async function stagesFor(
  { config, flags, }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
  },
): Promise<readonly Stage[]> {
  if (flags.command === 'prepare') {
    const fixturePath = join(config.wiz, 'fixture-path',);
    const fixture = await pathExists(fixturePath,) ? (await readFile(fixturePath, 'utf8',)).trim() : `${config.fixtureParent}/send-fixture.DRYRUN`;
    return await prepareStages({ c: config, fixtureFolder: fixture, },);
  }
  if (flags.command === 'cutover') {
    return await cutoverStages({ c: config, stamp: await swapStamp({ config, create: !flags.dryRun, },), },);
  }
  if (flags.command === 'thaw') {
    return thawStages(config,);
  }
  if (flags.command === 'rollback') {
    return [...rollbackStages({ c: config, stamp: await swapStamp({ config, create: false, },), },), ...thawStages(config,),];
  }
  return [];
}

/**
 Checks what a command needs before it starts, without changing anything.

 @param config - the configuration
 @param flags - the flags
 @throws GuidedStop with the precondition status when a condition is not met
 */
async function requirePreconditions(
  { config, flags, }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
  },
): Promise<void> {
  /**
   Stops with the precondition status.

   @param message - why
   @throws GuidedStop always
   */
  function refuse(message: string,): never {
    throw new GuidedStop({ status: EXIT_PRECONDITION, message, },);
  }

  if (process.getuid?.() === 0) {
    refuse('Run this as your own user, not as root: each root step asks sudo itself.',);
  }
  const here = process.cwd();
  if (here === config.repo || here.startsWith(`${config.repo}/`,)) {
    refuse('The current folder is inside the repository; the holder scans would report this shell. cd out of it first.',);
  }
  for (const tool of [config.copyTool, config.scanTool, config.nodePath,]) {
    if (!await pathExists(tool,)) {
      refuse(`Missing: ${tool}`,);
    }
  }
  if (flags.command === 'cutover') {
    for (const item of ['p4c-first', 'p4d-external-first', 'p4e-sizes', 'm1b-mount-option',]) {
      if (!await pathExists(join(config.wiz, 'done', 'prepare', item,),)) {
        refuse(`Prepare stage ${item} is not recorded as done. Run prepare first.`,);
      }
    }
  }
  if (flags.command === 'thaw' && await isSymbolicLink(config.repo,)) {
    refuse(`${config.repo} is already a link: the swap happened. Use the rollback command instead.`,);
  }
  await mkdir(join(config.wiz, 'logs',), { recursive: true, mode: 0o700, },);
}

/**
 Prints what the thaw left behind for the owner to delete before another attempt, and renames the cut-over progress
 and the window clock aside.

 @param config - the configuration
 @param out - the output
 */
async function thawAftermath(
  { config, out, }: {
    readonly config: GuidedConfig;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  const suffix = new Date().toISOString().replaceAll(/[\-:]/gv, '',).replace(/\.\d+Z$/v, 'Z',);
  await renameCutoverStateAside({ config, suffix, },);
  out(`Nothing was deleted. The cut-over progress and window clock were renamed aside (suffix .aborted-${suffix}).\n`
    + 'If a timeout stopped the final pass, its systemd scope may still run until its own deadline: wait for '
    + "'systemctl list-units run-*.scope' to show no btrfs send before deleting a partial receive.\n"
    + 'Before another cut-over attempt, remove the leftovers of this one yourself, because the copy tool refuses to reuse them: '
    + `the f### snapshots in ${config.snapshots}, the f### received copies in ${config.received}, the placed copy ${config.finalPath} `
    + `(nested subvolumes first, then the root: sudo btrfs subvolume delete), and move ${config.moveState}/f*.json aside.\n`,);
}

/**
 Runs one command of the runner.

 @param config - the configuration
 @param flags - the flags
 @param out - the output
 @returns the exit status
 */
export async function runGuided(
  { config, flags, out, }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
    readonly out: (text: string,) => void;
  },
): Promise<number> {
  try {
    if (flags.command === 'status') {
      out(`State folder: ${config.wiz}\n`,);
      for (const phase of ['prepare', 'cutover', 'thaw', 'rollback',]) {
        await printProgress({ config, phase, out, },);
      }
      if (await windowRunning(config,)) {
        out(`window running; remaining ${String(await windowRemaining(config,),)} s\n`,);
      }
      return 0;
    }
    if (!flags.dryRun) {
      await requirePreconditions({ config, flags, },);
    }
    const stages = await stagesFor({ config, flags, },);
    const swapped = flags.command === 'rollback' && !await pathExists(join(config.wiz, 'swap-timestamp',),);
    if (swapped && !flags.dryRun) {
      throw new GuidedStop({ status: EXIT_PRECONDITION, message: 'No swap was recorded. Use the thaw command instead.', },);
    }
    await runPhase({ config, flags, phase: flags.command, stages, out, },);
    if (!flags.dryRun && (flags.command === 'thaw' || flags.command === 'rollback')) {
      await thawAftermath({ config, out, },);
    }
    out(`\n${flags.command} finished${flags.dryRun ? ' (dry run: nothing was executed or recorded)' : ''}.\n`,);
    return 0;
  }
  catch (error) {
    if (error instanceof GuidedStop) {
      out(`\n${error.message}\n`,);
      if (!flags.dryRun && flags.command !== 'status') {
        await printProgress({ config, phase: flags.command, out, },);
        out(error.rollback === '' ? '' : `Rollback for the failed step: ${error.rollback}\n`,);
        out(`To resume: run the ${flags.command} command again. Completed stages are skipped and the failed one is offered again.\n`,);
      }
      return error.status;
    }
    throw error;
  }
}

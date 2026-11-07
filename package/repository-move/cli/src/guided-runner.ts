/**
 The guided runner of the move to the 860 drive: `prepare`, `cutover`, `thaw`, `rollback` and `status`. The coordinator
 runs it (the owner's decision of 2026-11-07); it replaced the guided bash script. Each stage shows its command, runs
 it, checks it, and stops at the first failure with what completed and how to resume. Root steps go through `sudo` with
 the absolute path of node; this module never runs `sudo` itself, and a dry run runs nothing.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import { cutoverStages, } from './guided-cutover-stages.data.ts';
import { requireKnownStages, } from './guided-flags.ts';
import { canonicalPath, } from './canonical-path.ts';
import { runPhase, } from './guided-engine.ts';
import {
  compactTimestamp,
  requirePreconditions,
  swapStamp,
} from './guided-preconditions.ts';
import { prepareStages, } from './guided-prepare-stages.data.ts';
import {
  listProgress,
  renameCutoverStateAside,
  windowRemaining,
  windowRunning,
} from './guided-state.ts';
import {
  EXIT_PRECONDITION,
  GuidedStopError,
} from './guided-stop.ts';
import type {
  GuidedConfig,
  GuidedFlags,
  Stage,
} from './guided-types.ts';
import {
  rollbackStages,
  thawStages,
} from './guided-undo-stages.data.ts';

/**
 Builds the text that says what a phase completed.

 @param config - the configuration

 @param phase - the phase

 @returns the text, ending in a newline
 */
async function progressText(
  {
    config,
    phase,
  }: {
    readonly config: GuidedConfig;
    readonly phase: string;
  },
): Promise<string> {
  /**
   The lines of text this step works on.
   */
  const lines = await listProgress({
    config,
    phase,
  },);
  return lines.length === 0
    ? `Nothing in phase ${phase} has completed yet.\n`
    : `Completed so far (phase ${phase}):\n${
      lines.map(function indent(line,) {
        return `  - ${line}`;
      },)
        .join('\n',)
    }\n`;
}

/**
 Prints what a phase completed.

 @param config - the configuration

 @param phase - the phase

 @param out - the output
 */
async function printProgress(
  {
    config,
    phase,
    out,
  }: {
    readonly config: GuidedConfig;
    readonly phase: string;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  out(await progressText({
    config,
    phase,
  },),);
}

/**
 Builds the stages of a command.

 @param config - the configuration
 
 @param flags - the flags
 
 @returns the stages, or an empty list for `status`
 */
async function stagesFor(
  {
    config,
    flags,
  }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
  },
): Promise<readonly Stage[]> {
  if (flags.command === 'prepare') {
    /**
     The file that names the fixture folder.
     */
    const fixturePath = join(
      config.wiz,
      'fixture-path',
    );
    /**
     The fixture folder in its canonical spelling, so a folder recorded before the stage resolved links still matches the
     mount table and Git's registry.
     */
    const fixture = await pathExists(fixturePath,)
      ? await canonicalPath((await readFile(
        fixturePath,
        'utf8',
      )).trim(),)
      : `${config.fixtureParent}/send-fixture.DRYRUN`;
    return await prepareStages({
      c: config,
      fixtureFolder: fixture,
    },);
  }
  if (flags.command === 'cutover') {
    return await cutoverStages({
      c: config,
      stamp: await swapStamp({
        config,
        create: !flags.dryRun,
      },),
    },);
  }
  if (flags.command === 'thaw') {
    return thawStages(config,);
  }
  if (flags.command === 'rollback') {
    return [
      ...rollbackStages({
        c: config,
        stamp: await swapStamp({
          config,
          create: false,
        },),
      },),
      ...thawStages(config,),
    ];
  }
  return [];
}

/**
 Prints what the thaw left behind for the owner to delete before another attempt, and renames the cut-over progress
 and the window clock aside.

 @param config - the configuration
 
 @param out - the output
 */
async function thawAftermath(
  {
    config,
    out,
  }: {
    readonly config: GuidedConfig;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  /**
   The suffix of the renamed state.
   */
  const suffix = compactTimestamp();
  await renameCutoverStateAside({
    config,
    suffix,
  },);
  out(
    `Nothing was deleted. The cut-over progress and window clock were renamed aside (suffix .aborted-${suffix}).\n`
      + 'If a timeout stopped the final pass, its systemd scope may still run until its own deadline: wait for '
      + '\'systemctl list-units run-*.scope\' to show no btrfs send before deleting a partial receive.\n'
      + 'Before another cut-over attempt, remove the leftovers of this one yourself, because the copy tool refuses to reuse them: '
      + `the f### snapshots in ${config.snapshots}, the f### received copies in ${config.received}, the placed copy ${config.finalPath} `
      + `(nested subvolumes first, then the root: sudo btrfs subvolume delete), and move ${config.moveState}/f*.json aside.\n`,
  );
}

/**
 Says whether the old tree may be deleted: after the swap it stays until the checksum verification of the final copies is
 recorded done.

 @param config - the configuration

 @param out - the output
 */
async function printOldCopyGate(
  {
    config,
    out,
  }: {
    readonly config: GuidedConfig;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  if (!await pathExists(join(
    config.wiz,
    'swap-timestamp',
  ),)) {
    return;
  }
  /**
   Whether the checksum verification of the final copies is recorded done.
   */
  const verified = await pathExists(join(
    config.wiz,
    'done',
    'cutover',
    'c22-verify-final-checksum',
  ),);
  out(
    verified
      ? 'The checksum verification of the final copies is done: the old tree kept for the rollback may be deleted by you.\n'
      : 'DO NOT DELETE the old tree kept for the rollback (the .nvme-before-860 folders): stage c22-verify-final-checksum is not done.\n',
  );
}

/**
 Prints the state of every phase.

 @param config - the configuration
 
 @param out - the output
 */
async function printStatus(
  {
    config,
    out,
  }: {
    readonly config: GuidedConfig;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  out(`State folder: ${config.wiz}\n`,);
  for (
    const text of await Promise.all([
      'prepare',
      'cutover',
      'thaw',
      'rollback',
    ].map(async function textOf(phase,) {
      return await progressText({
        config,
        phase,
      },);
    },),)
  ) {
    out(text,);
  }
  await printOldCopyGate({
    config,
    out,
  },);
  if (await windowRunning(config,)) {
    out(`window running; remaining ${String(await windowRemaining(config,),)} s\n`,);
  }
}

/**
 Prints a stop: its headline, the progress, the rollback of the failed stage and how to resume.

 @param error - the stop
 
 @param config - the configuration
 
 @param flags - the flags
 
 @param out - the output
 */
async function reportStop(
  {
    error,
    config,
    flags,
    out,
  }: {
    readonly error: GuidedStopError;
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
    readonly out: (text: string,) => void;
  },
): Promise<void> {
  out(`\n${error.message}\n`,);
  if ((!flags.dryRun) && (flags.command !== 'status')) {
    await printProgress({
      config,
      phase: flags.command,
      out,
    },);
    out(error.rollback === '' ? '' : `Rollback for the failed step: ${error.rollback}\n`,);
    out(
      `To resume: run the ${flags.command} command again. Completed stages are skipped and the failed one is offered again.\n`,
    );
  }
}

/**
 Runs one command of the runner.

 @param config - the configuration
 
 @param flags - the flags
 
 @param out - the output
 
 @returns the exit status

 @example
 ```ts
 import { runGuided, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = await runGuided({ config, flags, out, },);
 ```
 */
export async function runGuided(
  {
    config,
    flags,
    out,
  }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
    readonly out: (text: string,) => void;
  },
): Promise<number> {
  try {
    if (flags.command === 'status') {
      await printStatus({
        config,
        out,
      },);
      return 0;
    }
    /**
     The stages of the command, built before anything runs so a flag naming a stage that does not exist is refused.
     */
    const stages = await stagesFor({
      config,
      flags,
    },);
    requireKnownStages({
      stages,
      flags,
    },);
    if (!flags.dryRun) {
      await requirePreconditions({
        config,
        flags,
      },);
      if ((flags.command === 'rollback') && (!await pathExists(join(
        config.wiz,
        'swap-timestamp',
      ),))) {
        throw new GuidedStopError({
          status: EXIT_PRECONDITION,
          message: 'No swap was recorded. Use the thaw command instead.',
        },);
      }
    }
    await runPhase({
      config,
      flags,
      phase: flags.command,
      stages,
      restage: async function restage() {
        return await stagesFor({
          config,
          flags,
        },);
      },
      out,
    },);
    if ((!flags.dryRun) && ((flags.command === 'thaw') || (flags.command === 'rollback'))) {
      await thawAftermath({
        config,
        out,
      },);
    }
    out(`\n${flags.command} finished${flags.dryRun ? ' (dry run: nothing was executed or recorded)' : ''}.\n`,);
    return 0;
  }
  catch (error) {
    if (error instanceof GuidedStopError) {
      await reportStop({
        error,
        config,
        flags,
        out,
      },);
      return error.status;
    }
    throw error;
  }
}

/**
 The engine of the guided runner: it runs each stage (show, assert, run, check, record) and stops at the first failure.
 The stage list is data (`guided-*-stages.data.ts`); nothing here knows the move.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import { MILLISECONDS_PER_SECOND, } from './constants.ts';
import { GuidedStopError, } from './guided-stop.ts';
import type { Stage, } from './guided-types.ts';
import {
  recordTimer,
  windowRemaining,
  windowRunning,
} from './guided-state.ts';
import {
  executeCommand,
  runCheck,
} from './guided-stage-execution.ts';
import {
  buildCommand,
  type EngineRun,
  requireAssertion,
  showStage,
  type StageRun,
  startWindowIfAsked,
  windowStop,
} from './guided-stage-steps.ts';

/**
 Runs one stage: show, assert, run, check, record.

 @param listedRun - the stage in its phase, as the first build listed it
 
 @throws GuidedStopError when the stage fails, its check fails, an assertion is missing, or the window is over
 */
async function runStage(listedRun: StageRun,): Promise<void> {
  /**
   The stage in its phase with the stage as it reads now.
   */
  const stageRun: StageRun = {
    ...listedRun,
    stage: await currentStage(listedRun,),
  };
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    config,
    flags,
    phase,
    out,
  } = stageRun.run;
  /**
   The parts of the value this function uses, taken out of it.
   */
  const { stage, } = stageRun;
  /**
   The done file of this step.
   */
  const doneFile = join(
    config.wiz,
    'done',
    phase,
    stage.id,
  );
  if (
    (!flags.dryRun) && (stage.noMark !== true)
      && (!flags.redo
        .includes(stage.id,))
      && await pathExists(doneFile,)
  ) {
    out(`stage ${String(stageRun.index,)}: ${stage.title}: already done at ${(await readFile(
      doneFile,
      'utf8',
    )).trim()}; skipped\n`,);
    return;
  }
  if ((stage.window === true) && await windowRunning(config,)
    && (await windowRemaining(config,) <= 0)) {
    throw windowStop();
  }
  /**
   The parts of the value this function uses, taken out of it.
   */
  const { command, } = await buildCommand(stageRun,);
  showStage({
    stageRun,
    command,
  },);
  if (flags.dryRun) {
    out('dry run: this stage is shown, not executed, and not recorded\n',);
    return;
  }
  requireAssertion(stageRun,);
  await mkdir(
    join(
      config.wiz,
      'done',
      phase,
    ),
    { recursive: true, },
  );
  await mkdir(
    join(
      config.wiz,
      'logs',
    ),
    { recursive: true, },
  );
  await startWindowIfAsked(stageRun,);
  /**
   When the step started, in milliseconds since the epoch.
   */
  const started = Date.now();
  /**
   The file that holds the stage's output.
   */
  const outputFile = await executeCommand({
    stageRun,
    command,
  },);
  await runCheck({
    stageRun,
    outputFile,
  },);
  if (stage.timer !== undefined) {
    await recordTimer({
      config,
      timer: stage.timer,
      seconds: Math.round((Date.now() - started) / MILLISECONDS_PER_SECOND,),
    },);
  }
  if (stage.noMark !== true) {
    await writeFile(
      doneFile,
      `${new Date().toISOString()}\n`,
    );
  }
}

/**
 Gives the stage as it reads now: when the run can build its list again, the stage at the same place in a fresh list,
 so text that names a file an earlier stage of this run wrote is current.

 @param stageRun - the stage in its phase, as the first build listed it
 
 @returns the stage to run
 
 @throws Error when the fresh list has no stage at that place or its id differs, because the order the owner read must not change mid-run
 */
async function currentStage(stageRun: StageRun,): Promise<Stage> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    run,
    stage,
    index,
  } = stageRun;
  /**
   The function that builds the list again, when the run has one.
   */
  const { restage, } = run;
  if (restage === undefined) {
    return stage;
  }
  /**
   The list built again from what is on disk now.
   */
  const fresh = await restage();
  /**
   The stage this list has at the place the first build gave this one.
   */
  const current = fresh[index - 1];
  if (current?.id !== stage.id) {
    throw new Error(
      `The stage list changed while the run was in progress at ${stage.id}; run the command again.`,
    );
  }
  return current;
}

/**
 Runs every stage of a phase in order and stops at the first failure.

 @param run - the phase
 
 @throws GuidedStopError at the first stage that fails

 @example
 ```ts
 import { runPhase, } from '\@monochromatic-dev/repository-move-cli';
 
 await runPhase(run,);
 ```
 */
export async function runPhase(run: EngineRun,): Promise<void> {
  for (const [position, stage,] of run.stages
    .entries()) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- stages of a phase run one after another: each depends on what the earlier one did and stops the run at its first failure; see doc/lint/sequential-awaits.md
    await runStage({
      run,
      stage,
      index: position + 1,
    },);
  }
}

/**
 The run of a stage: its command and internal step, the exit status, and the check against the output.

 @module
 */

import {
  appendFile,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { runShell, } from './guided-shell.ts';
import {
  EXIT_STAGE_FAILED,
  GuidedStopError,
} from './guided-stop.ts';
import {
  logLine,
  windowRemaining,
  windowRunning,
  wizFile,
} from './guided-state.ts';
import type { Stage, } from './guided-types.ts';
import {
  type StageRun,
  windowStop,
} from './guided-stage-steps.ts';

/**
 Runs the command of a stage and its internal step, checks the exit status against the statuses the stage declares, and
 returns the output file and the status that happened.

 @param stageRun - the stage in its phase
 
 @param command - the command text
 
 @returns the output file's path and the exit status
 
 @throws GuidedStopError when the window is over or the command fails

 @example
 ```ts
 import { executeCommand, } from './guided-stage-execution.ts';
 
 const result = await executeCommand({ stageRun, command, },);
 ```
 */
export async function executeCommand(
  {
    stageRun,
    command,
  }: {
    readonly stageRun: StageRun;
    readonly command: string;
  },
): Promise<{
  readonly outputFile: string;
  readonly status: number
}> {
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
   The file that holds the stage's output.
   */
  const outputFile = wizFile({
    config,
    name: `logs/${phase}-${stage.id}.out`,
  },);
  await logLine({
    config,
    flags,
    line: `run ${phase}/${stage.id}: ${command}`,
  },);
  /**
   The output of the stage's internal step.
   */
  const internalText = stage.internal === undefined ? '' : await stage.internal({ config, },);
  out((internalText === '') || internalText.endsWith('\n',) ? internalText : `${internalText}\n`,);
  /**
   Whether the window is running now.
   */
  const live = (stage.window === true) && await windowRunning(config,);
  /**
   The deadline in seconds.
   */
  const seconds = live ? Math.ceil(await windowRemaining(config,),) : stage.timeoutSeconds ?? 0;
  if (live && (seconds <= 0)) {
    throw windowStop();
  }
  await writeFile(
    outputFile,
    internalText,
  );
  /**
   The exit status or result of the call.
   */
  const status = await runShell({
    command,
    outputFile: `${outputFile}.shell`,
    seconds,
    killGrace: config.killGraceSeconds,
    outputFileEnv: outputFile,
    out,
  },);
  await appendFile(
    outputFile,
    await readFile(
      `${outputFile}.shell`,
      'utf8',
    ),
  );
  await logLine({
    config,
    flags,
    line: `exit ${String(status,)} for ${phase}/${stage.id}`,
  },);
  if (!(stage.okCodes ?? [0,]).includes(status,)) {
    throw new GuidedStopError({
      status: EXIT_STAGE_FAILED,
      message: `Stage ${stage.id} (${stage.title}) exited ${String(status,)}; its output is above and in ${outputFile}.`,
      rollback: stage.rollback ?? '',
    },);
  }
  return {
    outputFile,
    status,
  };
}

/**
 Runs the stage's check against its output.

 @param stageRun - the stage in its phase
 
 @param outputFile - the command's output file
 
 @throws GuidedStopError when the check fails

 @example
 ```ts
 import { runCheck, } from './guided-stage-execution.ts';
 
 await runCheck({ stageRun, outputFile, },);
 ```
 */
export async function runCheck(
  {
    stageRun,
    outputFile,
  }: {
    readonly stageRun: StageRun;
    readonly outputFile: string;
  },
): Promise<void> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    stage,
    run,
  } = stageRun;
  if (stage.check === undefined) {
    return;
  }
  /**
   The exit status or result of the call.
   */
  const status = await runShell({
    command: stage.check,
    outputFile: `${outputFile}.check`,
    seconds: 0,
    killGrace: run.config
      .killGraceSeconds,
    outputFileEnv: outputFile,
    out: run.out,
  },);
  if (status !== 0) {
    throw new GuidedStopError({
      status: EXIT_STAGE_FAILED,
      message: `Stage ${stage.id} (${stage.title}) ran, but its check failed: ${stage.checkText ?? stage.check}`,
      rollback: stage.rollback ?? '',
    },);
  }
  run.out(`check passed: ${stage.checkText ?? stage.check}\n`,);
}

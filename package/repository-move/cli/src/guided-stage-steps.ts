/**
 The steps of one stage that come before its run: building and showing its command, the assertion, and the window clock.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import {
  EXIT_STOPPED_AT_ANSWER,
  EXIT_WINDOW_EXCEEDED,
  GuidedStopError,
} from './guided-stop.ts';
import {
  logLine,
  nowSeconds,
  windowRemaining,
  windowRunning,
  wizFile,
} from './guided-state.ts';
import type {
  GuidedConfig,
  GuidedFlags,
  Stage,
} from './guided-types.ts';

/**
 What the engine needs to run one phase.
 */
export type EngineRun = {
  readonly config: GuidedConfig;
  readonly flags: GuidedFlags;
  readonly phase: string;
  readonly stages: readonly Stage[];

  /**
   Builds the same stage list again from what is on disk now. The engine calls it before each stage, so a stage's text
   can depend on a file an earlier stage of the same run wrote (the fixture folder f1a makes). Without it the list
   built at the start is used as it is.
   */
  readonly restage?: () => Promise<readonly Stage[]>;
  readonly out: (text: string,) => void;
};

/**
 One stage in its place in a phase.
 */
export type StageRun = {
  readonly run: EngineRun;
  readonly stage: Stage;
  readonly index: number;
};

/**
 The stop for a window that ran past its maximum.

 @returns the stop
 */
export function windowStop(): GuidedStopError {
  return new GuidedStopError({
    status: EXIT_WINDOW_EXCEEDED,
    message: 'the cut-over window ran past its stated maximum; nothing further runs',
  },);
}

/**
 Builds the stage's command text, with the window's deadlines filled in.

 @param stageRun - the stage in its phase
 
 @returns the command and the window's remaining seconds (zero outside the window)
 */
export async function buildCommand(stageRun: StageRun,): Promise<{
  readonly command: string;
  readonly remaining: number
}> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const { config, } = stageRun.run;
  /**
   Whether the stage runs inside the running window.
   */
  const inWindow = (stageRun.stage
    .window
    === true) && await windowRunning(config,);
  /**
   Seconds left in the window.
   */
  const remaining = inWindow ? await windowRemaining(config,) : 0;
  /**
   The per-step deadline in seconds.
   */
  const deadline = inWindow
    ? Math.max(
      1,
      Math.min(
        remaining,
        config.transferDeadlineSeconds,
      ),
    )
    : config.transferDeadlineSeconds;
  /**
   The text this step works on.
   */
  const text = (typeof stageRun.stage
    .command) === 'string'
    ? stageRun.stage
      .command
    : await stageRun.stage
      .command({ config, },);
  return {
    command: text.replaceAll(
      '@DEADLINE@',
      String(deadline,),
    )
      .replaceAll(
      '@REMAINING@',
      String(Math.max(
        1,
        Math.floor(remaining,),
      ),),
      ),
    remaining,
  };
}

/**
 Shows a stage: what it says, its root use, its command, its check and its rollback.

 @param stageRun - the stage in its phase
 
 @param command - the command text
 */
export function showStage(
  {
    stageRun,
    command,
  }: {
    readonly stageRun: StageRun;
    readonly command: string;
  },
): void {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    out,
    stages,
  } = stageRun.run;
  /**
   The parts of the value this function uses, taken out of it.
   */
  const { stage, } = stageRun;
  out(`\n== stage ${String(stageRun.index,)}/${String(stages.length,)} ${stage.title} [${stage.id}] ==\n${stage.says}\n`,);
  out(`root: ${stage.root}; unprivileged route: ${stage.unprivileged}\n`,);
  out(`command (bash -e -o pipefail -c):\n${
    command.split('\n',)
      .map(function indent(line,) {
      return `    ${line}`;
    },)
      .join('\n',)
  }\n`,);
  out(stage.checkText === undefined ? '' : `check afterwards: ${stage.checkText}\n`,);
  out(stage.rollback === undefined ? '' : `rollback: ${stage.rollback}\n`,);
}

/**
 Refuses a stage whose assertion was not passed.

 @param stageRun - the stage in its phase
 
 @throws GuidedStopError with the answer status when the assertion is missing
 */
export function requireAssertion(stageRun: StageRun,): void {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    stage,
    run,
  } = stageRun;
  if ((stage.assertion !== undefined) && (!run.flags
    .asserted
    .includes(stage.id,))) {
    throw new GuidedStopError({
      status: EXIT_STOPPED_AT_ANSWER,
      message: `stage ${stage.id} states a fact only the coordinator or owner can know: ${stage.assertion} `
        + `Run again with --assert=${stage.id} once it is true; nothing was run for this stage.`,
    },);
  }
}

/**
 Starts the window clock when the stage is the one that starts it.

 @param stageRun - the stage in its phase
 */
export async function startWindowIfAsked(stageRun: StageRun,): Promise<void> {
  /**
   The parts of the value this function uses, taken out of it.
   */
  const {
    config,
    flags,
  } = stageRun.run;
  if ((stageRun.stage
    .windowStart
    === true) && (!await pathExists(wizFile({
      config,
      name: 'window-start',
    },),))) {
    await writeFile(
      wizFile({
        config,
        name: 'window-start',
      },),
      `${String(nowSeconds(),)}\n`,
    );
    await logLine({
      config,
      flags,
      line: 'window start',
    },);
  }
}

/**
 One end-to-end run: preconditions, the scratch library, the private sink, the sessions the filter selects, and the
 report.
 
 The run refuses to start while another run is active, plays only into its own null sink, runs the app and the
 compositor only inside its own systemd units, and keeps all state in scratch directories.
 
 @example
 ```ts
 import { runE2e, } from './run.ts';
 
 const { totals, } = await runE2e({ config, },);
 ```
 
 @packageDocumentation
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { runCommand, } from './command.ts';
import { compositorVersionProblem, } from './compositor-version.ts';
import type { RunConfig, } from './config.ts';
import type { RunContext, } from './context.ts';
import { runAutoPlaySession, } from './auto-play-session.ts';
import { runDialogSession, } from './dialog-session.ts';
import { expect, } from './errors.ts';
import { pathExists, } from './files.ts';
import { buildLibrary, } from './library.ts';
import { runNarrowSession, } from './narrow-session.ts';
import { createNullSink, } from './pipewire.ts';
import { runQuitSession, } from './quit-session.ts';
import { runStatesSession, } from './states-session.ts';
import { runTextSizeSession, } from './text-size-session.ts';
import { createRecorder, } from './recorder.ts';
import {
  type Outcome,
  printLine,
  type Totals,
  writeReport,
} from './report.ts';
import {
  type SessionName,
  sessionsFor,
} from './selection.ts';
import {
  runRefusal,
  UNIT_PREFIX,
} from './session-launch.ts';
import { runVisualSessions, } from './visual-session.ts';
import { runWideSession, } from './wide-session.ts';

/**
 Mode of the scratch root: owner only, so trust checks of other tools accept it.
 */
const SCRATCH_MODE = 0o700;

/**
 What a finished run leaves to the caller: every outcome and the counts.
 */
export type RunResult = {
  readonly outcomes: readonly Outcome[];
  readonly totals: Totals;
};

/**
 The function that runs each session of the suite.
 */
const SESSION_RUNNERS: Readonly<Record<SessionName, (context: RunContext,) => Promise<void>>> = {
  wide: runWideSession,
  narrow: runNarrowSession,
  dialog: runDialogSession,
  'auto-play': runAutoPlaySession,
  'text-size': runTextSizeSession,
  quit: runQuitSession,
  states: runStatesSession,
  visual: runVisualSessions,
};

/**
 Checks what the run needs before it touches anything: the release binary, a compositor that has the controls the
 checks use, a reachable PipeWire, and no other run.
 
 @param config - paths of the run
 
 @throws CheckFailureError when the binary is missing, the compositor is missing or lacks the controls, PipeWire is unreachable, or
 another run is active
 
 @example
 ```ts
 await checkPreconditions(config,);
 ```
 */
export async function checkPreconditions(config: RunConfig,): Promise<void> {
  expect({
    condition: await pathExists(config.releaseBinary,),
    message: `the release binary ${config.releaseBinary} does not exist; the task builds it first`,
  },);

  expect({
    condition: await pathExists(config.compositorPath,),
    message: `the compositor ${config.compositorPath} does not exist; install it or set E2E_COMPOSITOR`,
  },);

  /**
   What the compositor prints for `--version`, from which its controls are judged.
   */
  const version = await runCommand({
    command: config.compositorPath,
    args: ['--version',],
  },);
  /**
   Why the compositor cannot run the checks, empty when it can.
   */
  const versionProblem = compositorVersionProblem({
    output: version.stdout,
    path: config.compositorPath,
  },);
  expect({
    condition: versionProblem === '',
    message: versionProblem,
  },);

  /**
   Result of asking PipeWire for its objects.
   */
  const pipewire = await runCommand({
    command: 'pw-dump',
    args: [],
  },);
  expect({
    condition: pipewire.status === 0,
    message: 'PipeWire is not reachable',
  },);

  /**
   Active units of any run, this one's own included, since none has started.
   */
  const running = await runCommand({
    command: 'systemctl',
    args: [
      '--user',
      'list-units',
      `${UNIT_PREFIX}*`,
      '--state=active,activating',
      '--no-legend',
      '--plain',
    ],
  },);
  expect({
    condition: runRefusal(running.stdout,) === '',
    message: runRefusal(running.stdout,),
  },);
}

/**
 Runs the sessions the filter selects, with the private sink alive around them.
 
 @param base - scratch directory of the run
 
 @param library - scratch library directory
 
 @param config - paths of the run
 
 @param recorder - the run's recorder
 
 @throws SafetyStopError when the app plays anywhere but the private sink
 
 @example
 ```ts
 await runSessions({ config, base, library, recorder, },);
 ```
 */
async function runSessions(
  {
    config,
    base,
    library,
    recorder,
  }: {
    readonly base: string;
    readonly config: RunConfig;
    readonly library: string;
    readonly recorder: RunContext['recorder'];
  },
): Promise<void> {
  /**
   Tagged logger naming the session loop.
   */
  const rl = tagged({
    tag: runSessions.name,
    l: logger,
  },);

  /**
   The private null sink; leaving the function unloads it.
   */
  await using sink = await createNullSink({ name: `music_player_e2e_${process.pid}`, },);
  printLine(`private null sink ${sink.name} is node ${sink.nodeId}; scratch ${base}`,);

  /**
   Everything a session needs.
   */
  const context: RunContext = {
    base,
    config,
    library,
    recorder,
    runId: process.pid,
    sink,
  };
  for (const name of sessionsFor({ only: config.only, },)) {
    rl.info(`running the ${name} session`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the sessions must not overlap: one compositor and one app at a time
    await SESSION_RUNNERS[name](context,);
  }
}

/**
 Runs the suite once: preconditions, scratch space, sessions, report.
 
 @param config - paths of the run and the optional check filter
 
 @returns every outcome and the counts; failed checks are reported in the result, not thrown
 
 @throws CheckFailureError when a precondition fails or a session cannot start
 
 @throws SafetyStopError when the app plays anywhere but the private sink; the scratch directory is kept for inspection
 
 @example
 ```ts
 const { totals, } = await runE2e({ config, },);
 ```
 */
export async function runE2e(
  {
    config,
  }: {
    readonly config: RunConfig;
  },
): Promise<RunResult> {
  /**
   Tagged logger naming the run.
   */
  const rl = tagged({
    tag: runE2e.name,
    l: logger,
  },);
  await checkPreconditions(config,);
  await mkdir(
    config.scratchRoot,
    {
      recursive: true,
      mode: SCRATCH_MODE,
    },
  );
  await chmod(
    config.scratchRoot,
    SCRATCH_MODE,
  );
  await mkdir(
    config.outputRoot,
    { recursive: true, },
  );

  /**
   Scratch directory of this run.
   */
  const base = await mkdtemp(join(
    config.scratchRoot,
    'run-',
  ),);

  /**
   Start of the run, in milliseconds since the epoch.
   */
  const startedAt = Date.now();
  rl.info(`scratch directory ${base}`,);

  /**
   The scratch library.
   */
  const { library, } = await buildLibrary({
    root: base,
    generatorPath: config.generatorPath,
  },);

  // The trash check needs the library and the scratch XDG data on one filesystem, or the trash crate would use the
  // mount's own trash directory instead of the scratch one.
  expect({
    condition: (await stat(library,)).dev === (await stat(base,)).dev,
    message: 'the library and the scratch directory are on different filesystems',
  },);

  /**
   The recorder that applies the filter.
   */
  const recorder = createRecorder({ only: config.only, },);
  await runSessions({
    config,
    base,
    library,
    recorder,
  },);

  /**
   The totals, after the table and the summary file are written.
   */
  const totals = await writeReport({
    outcomes: recorder.outcomes(),
    startedAt,
    finishedAt: Date.now(),
    outputRoot: config.outputRoot,
  },);
  await rm(
    base,
    {
      recursive: true,
      force: true,
    },
  );
  return {
    outcomes: recorder.outcomes(),
    totals,
  };
}

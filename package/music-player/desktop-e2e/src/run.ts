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

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { runCommand, } from './command.ts';
import { compositorVersionProblem, } from './compositor-version.ts';
import type { RunConfig, } from './config.ts';
import type { RunContext, } from './context.ts';
import { runAutoPlaySession, } from './auto-play-session.ts';
import { runDialogSession, } from './dialog-session.ts';
import {
  CheckFailureError,
  expect,
  SafetyStopError,
} from './errors.ts';
import { pathExists, } from './files.ts';
import { resolveDisplay, } from './gpu-choice.ts';
import { buildLibrary, } from './library.ts';
import { runNarrowSession, } from './narrow-session.ts';
import { createNullSink, } from './pipewire.ts';
import { runQuitSession, } from './quit-session.ts';
import { runStatesSession, } from './states-session.ts';
import { runTextSizeSession, } from './text-size-session.ts';
import {
  createRecorder,
  type Recorder,
} from './recorder.ts';
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
  type DisplayMode,
  runRefusal,
  UNIT_PREFIX,
} from './session-launch.ts';
import { COLD_START_BOUND_MS, } from './timing.ts';
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
 Runs one session against the run's context.
 */
export type SessionRunner<Context> = (context: Context,) => Promise<void>;

/**
 The runner of every session of the suite, by session name.
 */
export type SessionRunners<Context> = ReadonlyMap<SessionName, SessionRunner<Context>>;

/**
 What the session loop needs to record a failure.
 */
type FailureRecorder = Pick<Recorder, 'fail'>;

/**
 The function that runs each session of the suite.
 */
const SESSION_RUNNERS: SessionRunners<RunContext> = new Map([
  [
    'wide',
    runWideSession
  ],
  [
    'narrow',
    runNarrowSession
  ],
  [
    'dialog',
    runDialogSession
  ],
  [
    'auto-play',
    runAutoPlaySession
  ],
  [
    'text-size',
    runTextSizeSession
  ],
  [
    'quit',
    runQuitSession
  ],
  [
    'states',
    runStatesSession
  ],
  [
    'visual',
    runVisualSessions
  ],
],);

/**
 Checks what the run needs before it touches anything: the release binary, a compositor that has the controls the
 checks use, a reachable PipeWire, no other run, and the display the sessions get. A run is headless unless a person
 asked for a visible window; its last step asks the compositor for its render devices, which also loads the compositor's
 libraries before the first session waits for them.
 
 @param config - paths of the run
 
 @returns the display mode every session of the run gets
 
 @throws CheckFailureError when the binary is missing, the compositor is missing or lacks the controls, PipeWire is unreachable,
 another run is active, or a headless run finds no render node
 
 @example
 ```ts
 const display = await checkPreconditions(config,);
 ```
 */
export async function checkPreconditions(config: RunConfig,): Promise<DisplayMode> {
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
    timeoutMs: COLD_START_BOUND_MS,
  },);
  /**
   Why the compositor cannot run the checks, empty when it can.
   */
  const versionProblem = compositorVersionProblem({
    output: version.stdout,
    path: config.compositorPath,
    headless: !config.visible,
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
    timeoutMs: COLD_START_BOUND_MS,
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
    timeoutMs: COLD_START_BOUND_MS,
  },);
  expect({
    condition: runRefusal(running.stdout,) === '',
    message: runRefusal(running.stdout,),
  },);
  return await resolveDisplay({
    compositorPath: config.compositorPath,
    visible: config.visible,
    gpu: config.gpu,
  },);
}

/**
 Runs the chosen sessions in order.
 
 A session that stops between two of its checks (a page it expects never came back) is recorded as a failure of that
 session, and the run goes on with the next session instead of ending without a report. A safety stop ends the run at
 once and records nothing, because nothing may play anywhere but the private sink.
 
 @param context - what every session gets
 
 @param names - the sessions to run, in order
 
 @param recorder - the recorder that keeps the failure of a session that stopped
 
 @param runners - the runner of each session
 
 @throws SafetyStopError when a session's routing proof fails; the rest of the run does not happen
 
 @example
 ```ts
 await runChosenSessions({ context, names: ['wide', 'narrow',], recorder, runners: SESSION_RUNNERS, },);
 ```
 */
export async function runChosenSessions<Context,>(
  {
    context,
    names,
    recorder,
    runners,
  }: {
    readonly context: Context;
    readonly names: readonly SessionName[];
    readonly recorder: FailureRecorder;
    readonly runners: SessionRunners<Context>;
  },
): Promise<void> {
  /**
   Tagged logger naming the session loop.
   */
  const rl = tagged({
    tag: runChosenSessions.name,
    l: logger,
  },);
  for (const name of names) {
    /**
     The runner of this session.
     */
    const runner = runners.get(name,);
    if (runner === undefined)
      throw new CheckFailureError(`the ${name} session has no runner, so the filter selects a session this build cannot run`,);
    rl.info(`running the ${name} session`,);
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- the sessions must not overlap: one compositor and one app at a time
      await runner(context,);
    }
    catch (error: unknown) {
      // The routing proof must still end the run at once.
      if (error instanceof SafetyStopError)
        throw error;
      rl.error(`the ${name} session stopped before its checks ended: ${caughtValueText(error,)}`,);
      // A step between two checks failed (a page to return to never came back). The run goes on with the next session
      // and the failure is recorded with the states the session had read, instead of ending the run without a report.
      // oxlint-disable-next-line eslint/no-await-in-loop -- recorded in order, after the session that failed
      await recorder.fail({
        id: `session/${name}`,
        title: `the ${name} session ran to its end`,
        detail: caughtValueText(error,),
      },);
    }
  }
}

/**
 Runs the sessions the filter selects, with the private sink alive around them.
 
 @param base - scratch directory of the run
 
 @param library - scratch library directory
 
 @param config - paths of the run
 
 @param recorder - the run's recorder
 
 @param display - where every session's compositor draws
 
 @throws SafetyStopError when the app plays anywhere but the private sink
 
 @example
 ```ts
 await runSessions({ config, base, library, recorder, display, },);
 ```
 */
async function runSessions(
  {
    config,
    base,
    library,
    recorder,
    display,
  }: {
    readonly base: string;
    readonly config: RunConfig;
    readonly display: DisplayMode;
    readonly library: string;
    readonly recorder: RunContext['recorder'];
  },
): Promise<void> {
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
    display,
    library,
    recorder,
    runId: process.pid,
    sink,
  };
  await runChosenSessions({
    context,
    names: sessionsFor({ only: config.only, },),
    recorder,
    runners: SESSION_RUNNERS,
  },);
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

  /**
   Where every session's compositor draws, decided before anything starts.
   */
  const display = await checkPreconditions(config,);
  rl.info(`display: ${display.kind === 'headless' ? `headless on ${display.gpu}` : 'a visible nested window, as asked'}`,);
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
   Folder of the failure files, inside the output folder because the scratch directory is removed at the end.
   */
  const evidenceDir = join(
    config.outputRoot,
    'failures',
  );

  // Failure files of an earlier run are not this run's evidence.
  await rm(
    evidenceDir,
    {
      recursive: true,
      force: true,
    },
  );

  /**
   The recorder that applies the filter and keeps each failing check's text and last states.
   */
  const recorder = createRecorder({
    only: config.only,
    evidenceDir,
  },);
  await runSessions({
    config,
    base,
    library,
    recorder,
    display,
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

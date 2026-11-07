/**
 The engine of the guided runner: it shows each stage, runs it, checks it, records it, keeps the cut-over window's
 clock, and stops at the first failure with what completed and how to resume.

 The stage list is data (`guided-*-stages.data.ts`); nothing here knows the move.

 @module
 */

import { spawn, } from 'node:child_process';
import {
  appendFile,
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { once, } from 'node:events';
import { createWriteStream, } from 'node:fs';
import { pathExists, } from './completion-records.ts';
import type {
  GuidedConfig,
  GuidedFlags,
  Stage,
} from './guided-types.ts';
import { MILLISECONDS_PER_SECOND, } from './constants.ts';

/**
 Exit status when the owner or coordinator answered no, or an assertion was missing.
 */
export const EXIT_STOPPED_AT_ANSWER = 2;

/**
 Exit status when the window ran past its stated maximum.
 */
export const EXIT_WINDOW_EXCEEDED = 3;

/**
 Exit status when a stage failed or its check failed.
 */
export const EXIT_STAGE_FAILED = 1;

/**
 Exit status when a start-up condition is not met.
 */
export const EXIT_PRECONDITION = 64;

/**
 A stop with a status and a headline; the runner prints the progress and exits.
 */
export class GuidedStop extends Error {
  public readonly status: number;
  public readonly rollback: string;

  public constructor({ status, message, rollback = '', }: { readonly status: number; readonly message: string; readonly rollback?: string; },) {
    super(message,);
    this.name = 'GuidedStop';
    this.status = status;
    this.rollback = rollback;
  }
}

/**
 What the engine needs to run one phase.
 */
export type EngineRun = {
  readonly config: GuidedConfig;
  readonly flags: GuidedFlags;
  readonly phase: string;
  readonly stages: readonly Stage[];
  readonly out: (text: string,) => void;
};

/**
 Path of a file in the runner's state folder.

 @param config - the configuration
 @param name - the file name
 @returns the path
 */
function wizFile({ config, name, }: { readonly config: GuidedConfig; readonly name: string; },): string {
  return join(config.wiz, name,);
}

/**
 Reads a number from a state file.

 @param path - the file
 @returns the number, or zero when the file is absent or empty
 */
async function readNumber(path: string,): Promise<number> {
  if (!await pathExists(path,)) {
    return 0;
  }
  const text = (await readFile(path, 'utf8',)).trim();
  return text === '' ? 0 : Number(text,);
}

/**
 Seconds since the epoch, whole.

 @returns the seconds
 */
function nowSeconds(): number {
  return Math.floor(Date.now() / MILLISECONDS_PER_SECOND,);
}

/**
 Tells whether the cut-over window is running: started, not ended, with a stated maximum.

 @param config - the configuration
 @returns true while the window runs
 */
export async function windowRunning(config: GuidedConfig,): Promise<boolean> {
  return await readNumber(wizFile({ config, name: 'window-start', },),) > 0
    && !await pathExists(wizFile({ config, name: 'window-end', },),)
    && await readNumber(wizFile({ config, name: 'window-max-seconds', },),) > 0;
}

/**
 Seconds left in the window; negative once it has passed.

 @param config - the configuration
 @returns the seconds
 */
export async function windowRemaining(config: GuidedConfig,): Promise<number> {
  const start = await readNumber(wizFile({ config, name: 'window-start', },),);
  const max = await readNumber(wizFile({ config, name: 'window-max-seconds', },),);
  return max - (nowSeconds() - start);
}

/**
 Appends a line to the runner's log, unless this is a dry run.

 @param config - the configuration
 @param flags - the flags
 @param line - the text
 */
async function logLine(
  { config, flags, line, }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
    readonly line: string;
  },
): Promise<void> {
  if (flags.dryRun) {
    return;
  }
  await mkdir(join(config.wiz, 'logs',), { recursive: true, },);
  await appendFile(wizFile({ config, name: 'logs/wizard.log', },), `${new Date().toISOString()} ${line}\n`,);
}

/**
 Lists what a phase has completed.

 @param config - the configuration
 @param phase - the phase
 @returns one line per completed stage
 */
export async function listProgress(
  { config, phase, }: {
    readonly config: GuidedConfig;
    readonly phase: string;
  },
): Promise<readonly string[]> {
  const folder = join(config.wiz, 'done', phase,);
  if (!await pathExists(folder,)) {
    return [];
  }
  const lines: string[] = [];
  for (const name of (await readdir(folder,)).toSorted()) {
    lines.push(`${name}  at ${(await readFile(join(folder, name,), 'utf8',)).trim()}`,);
  }
  return lines;
}

/**
 Adds seconds to, or sets, a timer file.

 @param config - the configuration
 @param timer - the timer
 @param seconds - the stage's duration
 */
async function recordTimer(
  { config, timer, seconds, }: {
    readonly config: GuidedConfig;
    readonly timer: NonNullable<Stage['timer']>;
    readonly seconds: number;
  },
): Promise<void> {
  const path = wizFile({ config, name: timer.file, },);
  const prior = timer.mode === 'add' ? await readNumber(path,) : 0;
  await writeFile(path, `${String(prior + seconds,)}\n`,);
}

/**
 Runs a shell command with `bash -e -o pipefail -c`, under `timeout`, teeing its output to a file and the screen.

 @param command - the shell text
 @param outputFile - where the output goes
 @param seconds - the deadline, or zero for none
 @param killGrace - seconds between terminate and kill
 @param env - the environment, with `OUTPUT_FILE`
 @returns the exit status
 */
async function runShell(
  { command, outputFile, seconds, killGrace, env, out, }: {
    readonly command: string;
    readonly outputFile: string;
    readonly seconds: number;
    readonly killGrace: number;
    readonly env: NodeJS.ProcessEnv;
    readonly out: (text: string,) => void;
  },
): Promise<number> {
  const [program, args,] = seconds > 0
    ? ['timeout', [`--kill-after=${String(killGrace,)}`, String(seconds,), 'bash', '-e', '-o', 'pipefail', '-c', command,],] as const
    : ['bash', ['-e', '-o', 'pipefail', '-c', command,],] as const;
  const sink = createWriteStream(outputFile,);
  const child = spawn(program, args, { env, stdio: ['ignore', 'pipe', 'pipe',], },);
  for (const stream of [child.stdout, child.stderr,]) {
    stream.on('data', function tee(chunk: Buffer,) {
      sink.write(chunk,);
      out(chunk.toString('utf8',),);
    },);
  }
  const [code,] = await once(child, 'close',) as [number | null];
  sink.end();
  await once(sink, 'close',);
  return code ?? EXIT_STAGE_FAILED;
}

/**
 Runs one stage: show, assert, run, check, record.

 @param run - the phase being run
 @param stage - the stage
 @param index - the stage's position, for the screen
 @throws GuidedStop when the stage fails, its check fails, an assertion is missing, or the window is over
 */
async function runStage(
  { run, stage, index, }: {
    readonly run: EngineRun;
    readonly stage: Stage;
    readonly index: number;
  },
): Promise<void> {
  const { config, flags, phase, out, } = run;
  const doneFile = join(config.wiz, 'done', phase, stage.id,);
  if (!flags.dryRun && stage.noMark !== true && !flags.redo.includes(stage.id,) && await pathExists(doneFile,)) {
    out(`stage ${String(index,)}/${String(run.stages.length,)} ${stage.title}: already done at ${(await readFile(doneFile, 'utf8',)).trim()}; skipped\n`,);
    return;
  }
  out(`\n== stage ${String(index,)}/${String(run.stages.length,)} ${stage.title} [${stage.id}] ==\n`,);
  const windowIsRunning = stage.window === true && await windowRunning(config,);
  if (windowIsRunning && await windowRemaining(config,) <= 0) {
    throw new GuidedStop({ status: EXIT_WINDOW_EXCEEDED, message: 'the cut-over window ran past its stated maximum; nothing further runs', },);
  }
  out(`${stage.says}\n`,);
  out(`root: ${stage.root}; unprivileged route: ${stage.unprivileged}\n`,);
  const remaining = windowIsRunning ? await windowRemaining(config,) : 0;
  const deadline = windowIsRunning ? Math.max(1, Math.min(remaining, config.transferDeadlineSeconds,),) : config.transferDeadlineSeconds;
  const built = typeof stage.command === 'string' ? stage.command : await stage.command({ config, },);
  const command = built.replaceAll('@DEADLINE@', String(deadline,),).replaceAll('@REMAINING@', String(Math.max(1, Math.floor(remaining,),),),);
  out(`command (bash -e -o pipefail -c):\n${command.split('\n',).map(function indent(line,) {
    return `    ${line}`;
  },).join('\n',)}\n`,);
  if (stage.checkText !== undefined) {
    out(`check afterwards: ${stage.checkText}\n`,);
  }
  if (stage.rollback !== undefined) {
    out(`rollback: ${stage.rollback}\n`,);
  }
  if (flags.dryRun) {
    out('dry run: this stage is shown, not executed, and not recorded\n',);
    return;
  }
  if (stage.assertion !== undefined && !flags.asserted.includes(stage.id,)) {
    throw new GuidedStop({
      status: EXIT_STOPPED_AT_ANSWER,
      message: `stage ${stage.id} states a fact only the coordinator or owner can know: ${stage.assertion} `
        + `Run again with --assert=${stage.id} once it is true; nothing was run for this stage.`,
    },);
  }
  await mkdir(join(config.wiz, 'done', phase,), { recursive: true, },);
  await mkdir(join(config.wiz, 'logs',), { recursive: true, },);
  if (stage.windowStart === true && !await pathExists(wizFile({ config, name: 'window-start', },),)) {
    await writeFile(wizFile({ config, name: 'window-start', },), `${String(nowSeconds(),)}\n`,);
    await logLine({ config, flags, line: 'window start', },);
  }
  const outputFile = wizFile({ config, name: `logs/${phase}-${stage.id}.out`, },);
  await logLine({ config, flags, line: `run ${phase}/${stage.id}: ${command}`, },);
  const started = Date.now();
  const internalText = stage.internal === undefined ? '' : await stage.internal({ config, },);
  if (internalText !== '') {
    out(internalText.endsWith('\n',) ? internalText : `${internalText}\n`,);
  }
  const live = stage.window === true && await windowRunning(config,);
  const seconds = live ? Math.ceil(await windowRemaining(config,),) : (stage.timeoutSeconds ?? 0);
  if (live && seconds <= 0) {
    throw new GuidedStop({ status: EXIT_WINDOW_EXCEEDED, message: 'the cut-over window ran past its stated maximum; nothing further runs', },);
  }
  await writeFile(outputFile, internalText,);
  const status = await runShell({
    command,
    outputFile: `${outputFile}.shell`,
    seconds,
    killGrace: config.killGraceSeconds,
    env: { ...process.env, OUTPUT_FILE: outputFile, },
    out,
  },);
  await appendFile(outputFile, await readFile(`${outputFile}.shell`, 'utf8',),);
  await logLine({ config, flags, line: `exit ${String(status,)} for ${phase}/${stage.id} after ${String(Math.round((Date.now() - started) / MILLISECONDS_PER_SECOND,),)} s`, },);
  if (!(stage.okCodes ?? [0,]).includes(status,)) {
    throw new GuidedStop({
      status: EXIT_STAGE_FAILED,
      message: `Stage ${stage.id} (${stage.title}) exited ${String(status,)}; its output is above and in ${outputFile}.`,
      rollback: stage.rollback ?? '',
    },);
  }
  if (stage.check !== undefined) {
    const checkStatus = await runShell({
      command: stage.check,
      outputFile: `${outputFile}.check`,
      seconds: 0,
      killGrace: config.killGraceSeconds,
      env: { ...process.env, OUTPUT_FILE: outputFile, },
      out,
    },);
    if (checkStatus !== 0) {
      throw new GuidedStop({
        status: EXIT_STAGE_FAILED,
        message: `Stage ${stage.id} (${stage.title}) ran, but its check failed: ${stage.checkText ?? stage.check}`,
        rollback: stage.rollback ?? '',
      },);
    }
    out(`check passed: ${stage.checkText ?? stage.check}\n`,);
  }
  if (stage.timer !== undefined) {
    await recordTimer({ config, timer: stage.timer, seconds: Math.round((Date.now() - started) / MILLISECONDS_PER_SECOND,), },);
  }
  if (stage.noMark !== true) {
    await writeFile(doneFile, `${new Date().toISOString()}\n`,);
  }
}

/**
 Runs every stage of a phase in order and stops at the first failure.

 @param run - the phase
 @throws GuidedStop at the first stage that fails
 */
export async function runPhase(run: EngineRun,): Promise<void> {
  let index = 0;
  for (const stage of run.stages) {
    index += 1;
    await runStage({ run, stage, index, },);
  }
}

/**
 Renames the cut-over progress and the window clock aside, so a new attempt starts clean. Nothing is deleted.

 @param config - the configuration
 @param suffix - the suffix, a timestamp
 */
export async function renameCutoverStateAside(
  { config, suffix, }: {
    readonly config: GuidedConfig;
    readonly suffix: string;
  },
): Promise<void> {
  for (const name of ['done/cutover', 'window-start', 'swap-timestamp',]) {
    const path = wizFile({ config, name, },);
    if (await pathExists(path,)) {
      await rename(path, `${path}.aborted-${suffix}`,);
    }
  }
}

/**
 Tells whether a file exists and is not empty.

 @param path - the file
 @returns true when it has content
 */
export async function hasContent(path: string,): Promise<boolean> {
  return await pathExists(path,) && (await stat(path,)).size > 0;
}

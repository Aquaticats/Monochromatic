/**
 Tests of the guided runner: the stage engine against stand-in stages (a failing stage, a failing check, an answer
 that is missing, a timeout, the window clock), the internal computations, the consistency of the stage data (every
 root stage marked and every unmarked stage free of `sudo`), and a dry run of every command through the built bin. No
 test runs `sudo` or touches a real disk.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  childFailureOf,
  createGuidedConfig,
  FIXTURE_COPY_SETS_KEPT,
  FIXTURE_GUARD_MIN_GIB,
  FIXTURE_MAIN_DATA_MIB,
  FIXTURE_SUBVOLUME_COUNT,
  cutoverStages,
  execFileAsync,
  footprintOf,
  type GuidedConfig,
  GuidedStopError,
  holderPatterns,
  parseGuidedFlags,
  preflight,
  prepareStages,
  prepareUnitEdit,
  rollbackStages,
  runGuided,
  runPhase,
  selectVerify,
  type Stage,
  thawStages,
  windowMaximum,
} from '../dist/final/node/index.mjs';

const BIN = new URL('../dist/final/node/run-repository-move.mjs', import.meta.url,).pathname;
/** Milliseconds the window test waits so a one-second window is over (a little more than one second). */
const WINDOW_PAUSE_MILLISECONDS = 2_100;
/** Mode of the stand-in `sudo` script: readable and executable by everyone, writable by its owner. */
const EXECUTABLE_MODE = 0o755;
const TOOLS_FOLDER = dirname(new URL(import.meta.url,).pathname,);

type HomeFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function makeHome(): Promise<HomeFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'guided-home-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

async function withConfig(
  fn: (args: { config: GuidedConfig; home: string; },) => Promise<void>,
  extraEnv: Record<string, string> = {},
): Promise<void> {
  await using home = await makeHome();
  const config = await createGuidedConfig({ env: { HOME: home.path, ...extraEnv, }, nodePath: process.execPath, },);
  await fn({ config, home: home.path, },);
}

type StageInput = Omit<Stage, 'title' | 'says' | 'root' | 'unprivileged'>;

function stage(input: StageInput,): Stage {
  return { title: input.id, says: 'says', root: 'no', unprivileged: 'not needed: test', ...input, };
}

type FlagOverrides = {
  readonly dryRun?: boolean;
  readonly redo?: string[];
  readonly asserted?: string[];
};

/** The words of a shell command, split at spaces, newlines and the characters that end a word. */
function commandWords(command: string,): readonly string[] {
  return ['\n', ';', '|', '&', '(', ')', '\t',].reduce((text, separator,) => text.replaceAll(separator, ' ',), command,)
    .split(' ',).filter((word,) => word !== '');
}

type ResolvedStage = { readonly stage: Stage; readonly command: string; };

/** Pairs every stage with the command text it runs once its internal step has written what the command reads. */
async function withCommands({ stages, config, }: { stages: readonly Stage[]; config: GuidedConfig; },): Promise<readonly ResolvedStage[]> {
  return await Promise.all(stages.map(async (candidate,) => ({ stage: candidate, command: await resolvedCommand({ stage: candidate, config, },), }),),);
}

async function exists(path: string,): Promise<boolean> {
  return await stat(path,).then(() => true, () => false);
}

/** Where the stand-ins for the privileged and mount-reading commands of a fixture run keep their records. */
type StandIns = {
  readonly folder: string;
  readonly log: string;
  readonly mounts: string;
};

/**
 Writes stand-ins for `sudo`, `findmnt` and `mountpoint` that change nothing real: `sudo` records its arguments, runs
 only `mkdir` for real, pretends `mkfs.btrfs`, `mount` and `umount`, and refuses the rest; `findmnt` and `mountpoint`
 answer from the list of folders the pretend `mount` was given, after resolving links the way the real tools do.
 */
async function writeStandIns(home: string,): Promise<StandIns> {
  const folder = join(home, 'stand-in-bin',);
  const log = join(home, 'stand-in-sudo.log',);
  const mounts = join(home, 'stand-in-mounts.txt',);
  await mkdir(folder, { recursive: true, },);
  const scripts: Record<string, string> = {
    sudo: [
      '#!/bin/sh',
      String.raw`printf '%s\n' "$*" >> "$STAND_IN_SUDO_LOG"`,
      'case "$1" in',
      '  --non-interactive|mkfs.btrfs|umount) exit 0 ;;',
      String.raw`  mount) for last; do :; done; printf '%s\n' "$(realpath "$last")" >> "$STAND_IN_MOUNTS"; exit 0 ;;`,
      '  mkdir) exec "$@" ;;',
      'esac',
      'echo "stand-in sudo refuses: $*" >&2',
      'exit 1',
      '',
    ].join('\n',),
    findmnt: [
      '#!/bin/sh',
      'for last; do :; done',
      'resolved=$(realpath "$last" 2>/dev/null) || exit 1',
      'grep --fixed-strings --line-regexp --quiet "$resolved" "$STAND_IN_MOUNTS" 2>/dev/null || exit 1',
      String.raw`printf '/dev/loop0 %s\n' "$resolved"`,
      '',
    ].join('\n',),
    mountpoint: [
      '#!/bin/sh',
      'for last; do :; done',
      'resolved=$(realpath "$last" 2>/dev/null) || exit 1',
      'grep --fixed-strings --line-regexp --quiet "$resolved" "$STAND_IN_MOUNTS" 2>/dev/null',
      '',
    ].join('\n',),
  };
  await Promise.all(Object.entries(scripts,).map(async ([name, text,],) => {
    await writeFile(join(folder, name,), text,);
    await chmod(join(folder, name,), EXECUTABLE_MODE,);
  },),);
  return { folder, log, mounts, };
}

/** Marks stages of the prepare phase done, as a completed earlier run would have. */
async function markPrepareDone({ config, ids, }: { config: GuidedConfig; ids: readonly string[]; },): Promise<void> {
  await mkdir(join(config.wiz, 'done', 'prepare',), { recursive: true, },);
  await Promise.all(ids.map((id,) => writeFile(join(config.wiz, 'done', 'prepare', id,), '2026-01-01T00:00:00.000Z\n',)),);
}

/** Runs the real prepare command through the built bin with the stand-ins first on the path. */
async function runPrepareWithStandIns(
  { home, standIns, flags = [], extraEnv = {}, }: {
    home: string;
    standIns: StandIns;
    flags?: readonly string[];
    extraEnv?: Record<string, string>;
  },
): Promise<string> {
  try {
    const result = await execFileAsync(process.execPath, [BIN, 'prepare', ...flags,], {
      encoding: 'utf8',
      env: {
        ...process.env,
        HOME: home,
        MOVE_TOOLS_DIR: TOOLS_FOLDER,
        PATH: `${standIns.folder}:${process.env.PATH ?? ''}`,
        STAND_IN_SUDO_LOG: standIns.log,
        STAND_IN_MOUNTS: standIns.mounts,
        ...extraEnv,
      },
      cwd: '/',
    },);
    return result.stdout;
  }
  catch (error) {
    return (error as { stdout: string; }).stdout;
  }
}

async function runStages(
  { config, stages, flags = {}, }: {
    config: GuidedConfig;
    stages: readonly Stage[];
    flags?: FlagOverrides;
  },
): Promise<{ error: unknown; output: string; }> {
  const lines: string[] = [];
  try {
    await runPhase({
      config,
      flags: { command: 'prepare', dryRun: false, redo: [], asserted: [], ...flags, },
      phase: 'prepare',
      stages,
      out: (text,) => {
        lines.push(text,);
      },
    },);
    return { error: undefined, output: lines.join('',), };
  }
  catch (error) {
    return { error, output: lines.join('',), };
  }
}

async function resolvedCommand({ stage: s, config, }: { stage: Stage; config: GuidedConfig; },): Promise<string> {
  return (typeof s.command) === 'string' ? s.command : await s.command({ config, },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runPhase.name,
      children: [
        it({
          name: 'runs stages in order, records each, and skips a recorded stage on a rerun',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const stages = [
                stage({ id: 'one', command: `echo one >> ${home}/trace`, },),
                stage({ id: 'two', command: `echo two >> ${home}/trace`, },),
              ];
              expect((await runStages({ config, stages, },)).error,).toBeUndefined();
              expect((await runStages({ config, stages, },)).output,).toContain('already done',);
              expect(await readFile(join(home, 'trace',), 'utf8',),).toBe('one\ntwo\n',);
              expect((await runStages({ config, stages, flags: { redo: ['one',], }, },)).error,).toBeUndefined();
              expect(await readFile(join(home, 'trace',), 'utf8',),).toBe('one\ntwo\none\n',);
            },);
          },
        },),
        it({
          name: 'stops at a failing stage with its status, keeps earlier records and runs nothing after it',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const stages = [
                stage({ id: 'ok', command: 'true', },),
                stage({ id: 'bad', command: 'exit 7', rollback: 'undo bad', },),
                stage({ id: 'never', command: `touch ${home}/never`, },),
              ];
              const { error, } = await runStages({ config, stages, },);
              expect(error,).toBeInstanceOf(GuidedStopError,);
              expect((error as GuidedStopError).status,).toBe(1,);
              expect((error as GuidedStopError).rollback,).toBe('undo bad',);
              expect(
                await exists(join(config.wiz, 'done', 'prepare', 'ok',),),
              ).toBe(true,);
              expect(
                await exists(join(config.wiz, 'done', 'prepare', 'bad',),),
              ).toBe(false,);
              expect(
                await exists(join(home, 'never',),),
              ).toBe(false,);
            },);
          },
        },),
        it({
          name: 'stops when a check fails after a command that succeeded, and accepts a listed non-zero status',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const failedCheck = await runStages({ config, stages: [stage({ id: 'c', command: 'true', check: 'false', checkText: 'the thing', },),], },);
              expect((failedCheck.error as GuidedStopError).message,).toContain('its check failed: the thing',);
              const accepted = await runStages({ config, stages: [stage({ id: 'k', command: 'exit 1', okCodes: [1,], },),], },);
              expect(accepted.error,).toBeUndefined();
              expect(await readFile(join(config.wiz, 'done', 'prepare', 'k',), 'utf8',),).toContain('exit status 1',);
              const stoppedFor = await runStages({ config, stages: [stage({ id: 'two', command: 'exit 2', okCodes: [0, 1,], },),], },);
              expect((stoppedFor.error as GuidedStopError).message,).toContain('exited 2',);
              const finished = await runStages({ config, stages: [stage({ id: 'ok', command: 'true', okCodes: [0, 1,], },),], },);
              expect(finished.error,).toBeUndefined();
              expect(await readFile(join(config.wiz, 'done', 'prepare', 'ok',), 'utf8',),).toContain('exit status 0',);
              const refusedZero = await runStages({ config, stages: [stage({ id: 'z', command: 'true', okCodes: [1,], },),], },);
              expect((refusedZero.error as GuidedStopError).status,).toBe(1,);
            },);
          },
        },),
        it({
          name: 'stops with status 2 and runs nothing when a stage needs an assertion that was not passed, and runs with it',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const stages = [stage({ id: 'ask', command: `touch ${home}/ran`, assertion: 'no agent works.', },),];
              const refused = await runStages({ config, stages, },);
              expect((refused.error as GuidedStopError).status,).toBe(2,);
              expect((refused.error as GuidedStopError).message,).toContain('--assert=ask',);
              expect(
                await exists(join(home, 'ran',),),
              ).toBe(false,);
              expect((await runStages({ config, stages, flags: { asserted: ['ask',], }, },)).error,).toBeUndefined();
              expect(
                await exists(join(home, 'ran',),),
              ).toBe(true,);
            },);
          },
        },),
        it({
          name: 'reports a stage past its timeout as a failure',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const { error, } = await runStages({ config, stages: [stage({ id: 'slow', command: 'sleep 20', timeoutSeconds: 1, },),], },);
              expect((error as GuidedStopError).status,).toBe(1,);
              expect((error as GuidedStopError).message,).toContain('exited 124',);
            },);
          },
        },),
        it({
          name: 'a dry run shows every stage and runs, records and creates nothing',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const { error, output, } = await runStages({
                config,
                stages: [stage({ id: 'touch', command: `touch ${home}/touched`, assertion: 'x', },),],
                flags: { dryRun: true, },
              },);
              expect(error,).toBeUndefined();
              expect(output,).toContain('dry run: this stage is shown',);
              expect(
                await exists(join(home, 'touched',),),
              ).toBe(false,);
              expect(await exists(config.wiz,),).toBe(false,);
            },);
          },
        },),
        it({
          name: 'shows the internal step output to the check, repeats a stage marked noMark, and adds a timer',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const stages = [
                stage({
                  id: 'internal',
                  command: 'true',
                  internal: async () => 'internal said hello',
                  check: 'grep --fixed-strings "internal said hello" "$OUTPUT_FILE"',
                  timer: { file: 'seconds', mode: 'add', },
                },),
                stage({ id: 'always', command: 'true', noMark: true, },),
              ];
              expect((await runStages({ config, stages, },)).error,).toBeUndefined();
              expect((await runStages({ config, stages, },)).output,).toContain('stage 2/2 always',);
              expect(
                Number((await readFile(join(config.wiz, 'seconds',), 'utf8',)).trim(),),
              ).toBeGreaterThanOrEqual(0,);
              expect(
                await exists(join(config.wiz, 'done', 'prepare', 'always',),),
              ).toBe(false,);
            },);
          },
        },),
        it({
          name: 'keeps the window clock: the first window stage starts it, and a stage after the maximum is refused with status 3',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              await mkdir(config.wiz, { recursive: true, },);
              await writeFile(join(config.wiz, 'window-max-seconds',), '1\n',);
              const stages = [
                stage({ id: 'start', command: 'true', windowStart: true, window: true, },),
                stage({ id: 'later', command: `touch ${home}/later`, window: true, },),
              ];
              const first = await runStages({ config, stages: [stages[0] as Stage,], },);
              expect(first.error,).toBeUndefined();
              await wait(WINDOW_PAUSE_MILLISECONDS,);
              const { error, } = await runStages({ config, stages, },);
              expect((error as GuidedStopError).status,).toBe(3,);
              expect(
                await exists(join(home, 'later',),),
              ).toBe(false,);
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'internal computations',
      children: [
        it({
          name: 'footprintOf adds exclusive and set shared of the last row',
          fn: async () => {
            expect(footprintOf('Total Exclusive Set shared Filename\n300 100 50 /x\n',),).toBe(150,);
          },
        },),
        it({
          name: 'preflight passes with the sizes small and refuses when the 860 would be left under the minimum',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              await mkdir(config.wiz, { recursive: true, },);
              await writeFile(join(config.wiz, 'main-du.txt',), 'Total Exclusive Set shared Filename\n1000 1000 1000 /x\n',);
              await writeFile(join(config.wiz, 'external-du.txt',), '2000\t/y\n',);
              const passed = await preflight(config,);
              expect(passed,).toContain('pre-flight passed',);
              expect(passed,).toContain('clone-limit allowance: 13038362624 bytes',);
              expect(passed,).toContain('estimate: 12 GiB',);
              const tight = await createGuidedConfig({
                env: { HOME: home, MOVE_DRIVE: home, MOVE_MIN_FREE_GIB: '99999999', },
                nodePath: process.execPath,
              },);
              let caught: unknown;
              try {
                await preflight(tight,);
              }
              catch (error) {
                caught = error;
              }
              expect((caught as Error).message,).toContain('REFUSED',);
            },);
          },
        },),
        it({
          name: 'windowMaximum derives the maximum, allows a shorter override and refuses a longer one',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              await mkdir(config.wiz, { recursive: true, },);
              await writeFile(join(config.wiz, 'first-pass-seconds',), '100\n',);
              await writeFile(join(config.wiz, 'scan-seconds',), '10\n',);
              await writeFile(join(config.wiz, 'du-seconds',), '5\n',);
              expect(await windowMaximum(config,),).toContain('stated maximum of the window: 230 s',);
              const shorter = await createGuidedConfig({ env: { HOME: home, MOVE_WINDOW_MAX_SECONDS: '60', }, nodePath: process.execPath, },);
              expect(await windowMaximum(shorter,),).toContain('stated maximum of the window: 60 s',);
              expect((await readFile(join(config.wiz, 'window-max-seconds',), 'utf8',)).trim(),).toBe('60',);
              const longer = await createGuidedConfig({ env: { HOME: home, MOVE_WINDOW_MAX_SECONDS: '999', }, nodePath: process.execPath, },);
              let caught: unknown;
              try {
                await windowMaximum(longer,);
              }
              catch (error) {
                caught = error;
              }
              expect((caught as Error).message,).toContain('never lengthened',);
            },);
          },
        },),
        it({
          name: 'selectVerify picks the checkout, dirty worktrees, and the extremes by size',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              await mkdir(config.wiz, { recursive: true, },);
              await mkdir(config.moveState, { recursive: true, },);
              const paths = Array.from({ length: 20, }, (_, index,) => `/r/wt${String(index,).padStart(2, '0',)}`);
              await Promise.all(['/r', ...paths,].map(async (path, index,) => {
                await writeFile(join(config.moveState, `f${String(index,).padStart(3, '0',)}.snapshot.json`,), JSON.stringify({ path, },),);
              },),);
              await writeFile(join(config.wiz, 'baseline-frozen.tsv',), [
                'path\tstate\tchanged\tuntracked\tunfinished\tindexLock\thead\tindexSha256',
                '/r/wt07\tok\t2\t0\t\tfalse\th\ts',
                '/r/wt08\tok\t0\t0\t\tfalse\th\ts',
              ].join('\n',),);
              await writeFile(join(config.wiz, 'subvolume-sizes.txt',), paths.map((path, index,) => `${String(index + 1,)}\t${path}`).join('\n',),);
              expect(await selectVerify(config,),).toBe('content checks cover 14 of 21 subvolumes',);
              const set = await readFile(join(config.wiz, 'verify-set.tsv',), 'utf8',);
              expect(set,).toContain('/r/wt07',);
              expect(set,).not.toContain('/r/wt08',);
              expect(set.split('\n',)[0],).toBe('f000\t/r',);
            },);
          },
        },),
        it({
          name: 'prepareUnitEdit appends the option to the one mount line, keeps the unit, and refuses anything unexpected',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const unit = join(home, 'mount-860.service',);
              const configured = await createGuidedConfig({ env: { HOME: home, MOVE_MOUNT_UNIT: unit, }, nodePath: process.execPath, },);
              await writeFile(unit, 'ExecStart=/bin/bash -c \'mount -o compress=zstd:3,noatime /dev/x /y\'\n',);
              expect(await prepareUnitEdit(configured,),).toContain('compress=zstd:3,noatime,user_subvol_rm_allowed',);
              expect(await readFile(join(configured.wiz, 'mount-860.service.new',), 'utf8',),).toContain('noatime,user_subvol_rm_allowed /dev/x',);
              expect(await readFile(join(configured.wiz, 'mount-860.service.before',), 'utf8',),).not.toContain('user_subvol_rm_allowed',);
              await writeFile(unit, 'mount -o compress=zstd:3,noatime a\nmount -o compress=zstd:3,noatime b\n',);
              let caught: unknown;
              try {
                await prepareUnitEdit(configured,);
              }
              catch (error) {
                caught = error;
              }
              expect((caught as Error).message,).toContain('exactly one',);
              await writeFile(unit, 'mount -o compress=zstd:3,noatime,user_subvol_rm_allowed a\n',);
              expect(await prepareUnitEdit(configured,),).toContain('already holds',);
              expect(config.unitPath,).toBe('/etc/systemd/system/mount-860.service',);
            },);
          },
        },),
        it({
          name: 'holderPatterns covers the repository, the external worktrees, and worktrees elsewhere with their 860 spelling',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              await mkdir(config.wiz, { recursive: true, },);
              await writeFile(join(config.wiz, 'porcelain-start.txt',), [
                `worktree ${config.repo}`,
                `worktree ${config.repo}/.claude/worktrees/x`,
                'worktree /somewhere/else',
                `worktree ${home}/temp/agent/w`,
              ].join('\n',),);
              const patterns = await holderPatterns(config,);
              expect(patterns,).toContain(`${config.repo}*`,);
              expect(patterns,).toContain('/somewhere/else*',);
              expect(patterns,).toContain(`${config.drive}/temp/agent/w*`,);
              expect(patterns.filter((pattern,) => pattern.includes('.claude/worktrees/x',),),).toHaveLength(0,);
            },);
          },
        },),
        it({
          name: 'parseGuidedFlags reads the command and flags and refuses an unknown command',
          fn: async () => {
            expect(parseGuidedFlags(['cutover', '--dry-run', '--redo=a', '--assert=b', '--assert=c',],),).toEqual({
              command: 'cutover',
              dryRun: true,
              redo: ['a',],
              asserted: ['b', 'c',],
            },);
            expect(() => parseGuidedFlags(['sideways',],),).toThrow(GuidedStopError,);
          },
        },),
        it({
          name: 'parseGuidedFlags refuses every flag it does not know, an empty value, and extra words, naming them and the valid flags',
          fn: async () => {
            for (const bad of [['--log-file=/x',], ['--redo=',], ['--assert=',], ['--dryrun',], ['extra',], ['--redo',],]) {
              let caught: unknown;
              try {
                parseGuidedFlags(['prepare', ...bad,],);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(GuidedStopError,);
              expect((caught as GuidedStopError).message,).toContain(bad[0] ?? '',);
              expect((caught as GuidedStopError).message,).toContain('--dry-run',);
            }
          },
        },),
        it({
          name: 'runGuided refuses a --redo or --assert that names no stage of the command, listing the stage ids',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const outcomes = await Promise.all([{ redo: ['no-such-stage',], asserted: [], }, { redo: [], asserted: ['no-such-stage',], },].map(async (flags,) => {
                const lines: string[] = [];
                const status = await runGuided({
                  config,
                  flags: { command: 'prepare', dryRun: true, ...flags, },
                  out: (text,) => {
                    lines.push(text,);
                  },
                },);
                return { status, text: lines.join('',), };
              },),);
              for (const { status, text, } of outcomes) {
                expect(status,).toBe(64,);
                expect(text,).toContain('no-such-stage',);
                expect(text,).toContain('f1-images',);
              }
            },);
          },
        },),
        it({
          name: 'createGuidedConfig refuses a path with a character the shown commands would not quote',
          fn: async () => {
            await withConfig(async ({ home, },) => {
              let caught: unknown;
              try {
                await createGuidedConfig({ env: { HOME: home, MOVE_SNAPSHOTS: '/a b', }, nodePath: process.execPath, },);
              }
              catch (error) {
                caught = error;
              }
              expect((caught as Error).message,).toContain('snapshots=/a b',);
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'stage data',
      children: [
        it({
          name: 'every stage names its root use and unprivileged route, ids are unique, and root stages and only root stages use sudo',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const sets: Record<string, readonly Stage[]> = {
                prepare: await prepareStages({ c: config, fixtureFolder: `${config.fixtureParent}/send-fixture.X`, },),
                cutover: await cutoverStages({ c: config, stamp: '20260101T000000Z', },),
                thaw: thawStages(config,),
                rollback: rollbackStages({ c: config, stamp: '20260101T000000Z', },),
              };
              const resolved = await Promise.all(Object.entries(sets,).map(async ([phase, stages,],) => ({
                phase,
                stages,
                withCommands: await withCommands({ stages, config, },),
              }),),);
              for (const { phase, stages, withCommands: pairs, } of resolved) {
                expect(new Set(stages.map((s,) => s.id),).size,).toBe(stages.length,);
                for (const { stage: s, command, } of pairs) {
                  expect(s.unprivileged.length,).toBeGreaterThan(10,);
                  // The one stage that only prints the removals for a person to run names sudo in text it echoes.
                  const usesSudo = s.id === 'f8-cleanup-printed' ? false : commandWords(command,).includes('sudo',);
                  expect([phase, s.id, usesSudo,],).toEqual([phase, s.id, s.root === 'yes',],);
                  expect(commandWords(command,).includes('node',),).toBe(false,);
                  for (const line of command.split('\n',).filter((row,) => row.includes('sudo',) && (row.includes(config.copyTool,) || row.includes(config.scanTool,)))) {
                    expect(line,).toContain(`sudo ${config.nodePath} `,);
                  }
                }
              }
            },);
          },
        },),
        it({
          name: 'every fixture snapshot folder is inside the source filesystem, made by a rerun-safe stage after the source tree',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const stages = await prepareStages({ c: config, fixtureFolder: '/x', },);
              const ids = stages.map((s,) => s.id,);
              const made = stages.find((s,) => s.id === 'f2b-snapshot-folder',) as Stage;
              expect(ids.indexOf('f2-populate',),).toBeLessThan(ids.indexOf('f2b-snapshot-folder',),);
              expect(ids.indexOf('f2b-snapshot-folder',),).toBeLessThan(ids.indexOf('f3a-print',),);
              expect(made.root,).toBe('yes',);
              expect(made.command,).toContain('sudo mkdir --parents /x/source/snap',);
              for (const { stage: s, command, } of await withCommands({ stages, config, },)) {
                const texts = [command, s.check ?? '', s.rollback ?? '',].join('\n',);
                expect([s.id, texts.includes('/x/snap',),],).toEqual([s.id, false,],);
              }
            },);
          },
        },),
        it({
          name: 'the fixture images are sized from what the fixture stages write, and every stage that guards free space uses the one guard minimum',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const stages = await prepareStages({ c: config, fixtureFolder: '/x', },);
              const images = stages.find((s,) => s.id === 'f1-images',) as Stage;
              const sizes = [...images.command.matchAll(/truncate --size=(\d+)G/g,),].map((match,) => Number(match[1],),);
              const bytesWrittenMiB = FIXTURE_MAIN_DATA_MIB * FIXTURE_SUBVOLUME_COUNT * FIXTURE_COPY_SETS_KEPT;
              const mibPerGib = 1_024;
              expect(sizes.length,).toBeGreaterThan(0,);
              for (const size of sizes) {
                expect(size * mibPerGib,).toBeGreaterThanOrEqual(bytesWrittenMiB + (FIXTURE_GUARD_MIN_GIB * mibPerGib),);
              }
              const populate = stages.find((s,) => s.id === 'f2-populate',) as Stage;
              expect(populate.command,).toContain(`head --bytes=${String(FIXTURE_MAIN_DATA_MIB,)}M`,);
              const guards = stages.flatMap((s,) => [...(s.command ?? '').matchAll(/--min-free-gib=(\d+)/g,),].map((match,) => Number(match[1],),),);
              expect(guards.length,).toBeGreaterThan(0,);
              for (const guard of guards.filter((value,) => value !== config.minFreeGiB,)) {
                expect(guard,).toBe(FIXTURE_GUARD_MIN_GIB,);
              }
            },);
          },
        },),
        it({
          name: 'the deadline stage declares the statuses it accepts instead of swallowing them in its shell text, and works in fresh folders so a rerun never meets its own partial receive',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const stages = await prepareStages({ c: config, fixtureFolder: '/x', },);
              const deadline = stages.find((s,) => s.id === 'f5a-deadline',) as Stage;
              expect(deadline.okCodes,).toEqual([0, 1,],);
              const command = await resolvedCommand({ stage: deadline, config, },);
              expect(command,).not.toContain('$?',);
              expect(command,).not.toContain('script exit status',);
              expect(command,).toContain('mktemp --directory /x/dest/recv-f5a.',);
              expect(deadline.check,).toContain('exited 124',);
              expect((stages.find((s,) => s.id === 'f5b-leftover',) as Stage).okCodes,).toEqual([1,],);
            },);
          },
        },),
        it({
          name: 'the cut-over runs the freeze before the baseline and the final pass, and the swap only after the verification',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const ids = (await cutoverStages({ c: config, stamp: 'S', },)).map((s,) => s.id);
              const at = (id: string,) => ids.indexOf(id,);
              expect(at('c07-freeze',),).toBeLessThan(at('c09-baseline',),);
              expect(at('c09-baseline',),).toBeLessThan(at('c10-final-pass',),);
              expect(at('c17-externals',),).toBeLessThan(at('c19-swap',),);
              expect(at('c19-swap',),).toBeLessThan(at('c20b-record-after',),);
              expect(at('c20b-record-after',),).toBeLessThan(at('c21-release',),);
              expect(at('c06-stop-units',),).toBeLessThan(at('c07-freeze',),);
            },);
          },
        },),
        it({
          name: 'the 860 mount option has a stage that prepares the edit, then applies it and checks the unit and the live options',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const stages = await prepareStages({ c: config, fixtureFolder: '/x', },);
              const apply = stages.find((s,) => s.id === 'm1b-mount-option',) as Stage;
              expect(stages.map((s,) => s.id,).indexOf('m1a-unit-edit',),).toBeLessThan(stages.map((s,) => s.id,).indexOf('m1b-mount-option',),);
              expect(apply.root,).toBe('yes',);
              expect(apply.command,).toContain('mount -o remount,user_subvol_rm_allowed',);
              expect(apply.check,).toContain('findmnt',);
              expect(apply.check,).toContain('systemctl cat mount-860.service',);
            },);
          },
        },),
      ],
    },),
    describe({
      name: runGuided.name,
      children: [
        it({
          name: 'a dry run of every command through the built bin exits 0, lists its stages and creates no state',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const results = await Promise.all(['prepare', 'cutover', 'thaw', 'rollback',].map(async (command,) => ({
                command,
                result: await execFileAsync(process.execPath, [BIN, command, '--dry-run',], {
                  encoding: 'utf8',
                  env: { ...process.env, HOME: home, },
                  cwd: '/',
                },),
              }),),);
              for (const { command, result, } of results) {
                expect(result.stdout,).toContain(`${command} finished (dry run`,);
                expect(result.stdout,).toContain('== stage 1/',);
              }
              expect(await exists(config.wiz,),).toBe(false,);
              expect(await exists(config.moveState,),).toBe(false,);
            },);
          },
        },),
        it({
          name: 'a real prepare run gives the fixture folder that f1a made to the later fixture stages, not the dry-run placeholder',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const standIns = await writeStandIns(home,);
              await markPrepareDone({ config, ids: ['i1-install-link-check', 'i2-record-before',], },);
              const output = await runPrepareWithStandIns({ home, standIns, },);
              const fixture = (await readFile(join(config.wiz, 'fixture-path',), 'utf8',)).trim();
              const sudoLog = await readFile(standIns.log, 'utf8',);
              expect(output,).toContain('f1a-folder',);
              expect(sudoLog,).toContain(`mkfs.btrfs --quiet --force ${fixture}/source.img.partial`,);
              expect(sudoLog,).not.toContain('DRYRUN',);
              const imageExists = await exists(join(fixture, 'source.img',),);
              expect(imageExists,).toBe(true,);
            },);
          },
        },),
        it({
          name: 'a resumed prepare run with f1a already recorded done uses the folder f1a recorded',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const standIns = await writeStandIns(home,);
              const fixture = join(config.fixtureParent, 'send-fixture.RESUMED',);
              await mkdir(fixture, { recursive: true, },);
              await mkdir(config.wiz, { recursive: true, },);
              await writeFile(join(config.wiz, 'fixture-path',), `${fixture}\n`,);
              await markPrepareDone({ config, ids: ['i1-install-link-check', 'i2-record-before', 'f1a-folder',], },);
              await runPrepareWithStandIns({ home, standIns, },);
              const sudoLog = await readFile(standIns.log, 'utf8',);
              expect(sudoLog,).toContain(`mkfs.btrfs --quiet --force ${fixture}/source.img.partial`,);
              expect(sudoLog,).not.toContain('DRYRUN',);
            },);
          },
        },),
        it({
          name: 'f1a records the fixture folder with its links resolved, so every later path text equals what the mount table holds',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const standIns = await writeStandIns(home,);
              const realParent = join(home, 'real-parent',);
              const linkedParent = join(home, 'via-link',);
              await mkdir(realParent, { recursive: true, },);
              await symlink(realParent, linkedParent,);
              await markPrepareDone({ config, ids: ['i1-install-link-check', 'i2-record-before',], },);
              await runPrepareWithStandIns({ home, standIns, extraEnv: { MOVE_FIXTURE_PARENT: linkedParent, }, },);
              const recorded = (await readFile(join(config.wiz, 'fixture-path',), 'utf8',)).trim();
              expect(recorded,).toBe(await realpath(recorded,),);
              expect(recorded.startsWith(`${await realpath(realParent,)}/`,),).toBe(true,);
            },);
          },
        },),
        it({
          name: 'f1-images can be run again after it finished or stopped part way: it reuses what it made and both loop mounts are found',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const standIns = await writeStandIns(home,);
              await markPrepareDone({ config, ids: ['i1-install-link-check', 'i2-record-before',], },);
              const first = await runPrepareWithStandIns({ home, standIns, },);
              expect(first,).toContain('both mount lines are loop devices',);
              expect(
                await exists(join(config.wiz, 'done', 'prepare', 'f1-images',),),
              ).toBe(true,);
              const again = await runPrepareWithStandIns({ home, standIns, flags: ['--redo=f1-images',], },);
              expect(again,).not.toContain('cannot overwrite',);
              expect(again,).not.toContain('File exists',);
              expect(again.split('both mount lines are loop devices',).length - 1,).toBeGreaterThanOrEqual(1,);
            },);
          },
        },),
        it({
          name: 'no stage runs findmnt with two operands, which findmnt reads as one source and one target that must both match',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const sets = [
                await prepareStages({ c: config, fixtureFolder: '/x', },),
                await cutoverStages({ c: config, stamp: 'S', },),
                thawStages(config,),
                rollbackStages({ c: config, stamp: 'S', },),
              ];
              const resolvedSets = await Promise.all(sets.map(async (stages,) => await withCommands({ stages, config, },),),);
              for (const pairs of resolvedSets) {
                for (const { stage: s, command, } of pairs) {
                  const texts = [command, s.check ?? '', s.rollback ?? '',];
                  for (const text of texts) {
                    for (const line of ['\n', ';', '|', '&', '(', ')',].reduce((joined, separator,) => joined.replaceAll(separator, '\n',), text,).split('\n',).filter((row,) => row.includes('findmnt',))) {
                      const words = commandWords(line,);
                      const after = words.slice(words.indexOf('findmnt',) + 1,);
                      const operands: string[] = [];
                      let skipValue = false;
                      for (const word of after) {
                        if (skipValue) {
                          skipValue = false;
                        }
                        else if (['--output', '--target', '--mountpoint', '--source', '--types', '-o', '-T', '-M',].includes(word,)) {
                          skipValue = true;
                        }
                        else if (!word.startsWith('-',)) {
                          operands.push(word,);
                        }
                      }
                      expect([s.id, operands.length <= 1,],).toEqual([s.id, true,],);
                    }
                  }
                }
              }
            },);
          },
        },),
        it({
          name: 'status says not to delete the old tree after the swap until the checksum verification of the final copies is done',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              await mkdir(join(config.wiz, 'done', 'cutover',), { recursive: true, },);
              await writeFile(join(config.wiz, 'swap-timestamp',), '20260101T000000Z\n',);
              const before: string[] = [];
              await runGuided({ config, flags: { command: 'status', dryRun: false, redo: [], asserted: [], }, out: (text,) => {
                before.push(text,);
              }, },);
              expect(before.join('',),).toContain('DO NOT DELETE the old tree',);
              await writeFile(join(config.wiz, 'done', 'cutover', 'c22-verify-final-checksum',), 'done\n',);
              const after: string[] = [];
              await runGuided({ config, flags: { command: 'status', dryRun: false, redo: [], asserted: [], }, out: (text,) => {
                after.push(text,);
              }, },);
              expect(after.join('',),).toContain('may be deleted by you',);
            },);
          },
        },),
        it({
          name: 'the cut-over verifies the final copies inside the window after the final pass and by checksum after the release',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const ids = (await cutoverStages({ c: config, stamp: 'S', },)).map((s,) => s.id);
              const at = (id: string,) => ids.indexOf(id,);
              expect(at('c10-final-pass',),).toBeLessThan(at('c10b-verify-final',),);
              expect(at('c10b-verify-final',),).toBeLessThan(at('c19-swap',),);
              expect(at('c21-release',),).toBeLessThan(at('c22-verify-final-checksum',),);
            },);
          },
        },),
        it({
          name: 'status lists nothing completed on a fresh home, and an unknown command exits 64',
          fn: async () => {
            await withConfig(async ({ home, },) => {
              const status = await execFileAsync(process.execPath, [BIN, 'status',], { encoding: 'utf8', env: { ...process.env, HOME: home, }, cwd: '/', },);
              expect(status.stdout,).toContain('Nothing in phase prepare has completed yet.',);
              let code = 0;
              try {
                await execFileAsync(process.execPath, [BIN, 'sideways',], { encoding: 'utf8', env: { ...process.env, HOME: home, }, cwd: '/', },);
              }
              catch (error) {
                code = Number(childFailureOf(error,).code,);
              }
              expect(code,).toBe(64,);
            },);
          },
        },),
        it({
          name: 'a real run refuses to start from a folder inside the repository',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              await mkdir(config.repo, { recursive: true, },);
              let code = 0;
              let stderr = '';
              try {
                await execFileAsync(process.execPath, [BIN, 'prepare',], {
                  encoding: 'utf8',
                  env: { ...process.env, HOME: home, },
                  cwd: config.repo,
                },);
              }
              catch (error) {
                const failure = childFailureOf(error,);
                code = Number(failure.code,);
                stderr = failure.stdout;
              }
              expect(code,).toBe(64,);
              expect(stderr,).toContain('inside the repository',);
            },);
          },
        },),
      ],
    },),
  ],
},);

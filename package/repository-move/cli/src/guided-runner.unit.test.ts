/**
 Tests of the guided runner: the stage engine against stand-in stages (a failing stage, a failing check, an answer
 that is missing, a timeout, the window clock), the internal computations, the consistency of the stage data (every
 root stage marked and every unmarked stage free of `sudo`), and a dry run of every command through the built bin. No
 test runs `sudo` or touches a real disk.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  createGuidedConfig,
  cutoverStages,
  footprintOf,
  type GuidedConfig,
  GuidedStop,
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

const execFileAsync = promisify(execFile,);
const BIN = new URL('../dist/final/node/run-repository-move.mjs', import.meta.url,).pathname;

async function withConfig(
  fn: (args: { config: GuidedConfig; home: string; },) => Promise<void>,
  extraEnv: Record<string, string> = {},
): Promise<void> {
  const home = await mkdtemp(join(scratchRoot(), 'guided-home-',),);
  try {
    const config = await createGuidedConfig({ env: { HOME: home, ...extraEnv, }, nodePath: process.execPath, },);
    await fn({ config, home, },);
  }
  finally {
    await rm(home, { recursive: true, force: true, },);
  }
}

function stage(overrides: Partial<Stage> & { id: string; command: string; },): Stage {
  return { title: overrides.id, says: 'says', root: 'no', unprivileged: 'not needed: test', ...overrides, };
}

async function exists(path: string,): Promise<boolean> {
  return await stat(path,).then(() => true, () => false);
}

async function runStages(
  { config, stages, flags = {}, }: {
    config: GuidedConfig;
    stages: readonly Stage[];
    flags?: Partial<{ dryRun: boolean; redo: string[]; asserted: string[]; }>;
  },
): Promise<{ error: unknown; output: string; }> {
  let output = '';
  try {
    await runPhase({
      config,
      flags: { command: 'prepare', dryRun: false, redo: [], asserted: [], ...flags, },
      phase: 'prepare',
      stages,
      out: (text,) => {
        output += text;
      },
    },);
    return { error: undefined, output, };
  }
  catch (error) {
    return { error, output, };
  }
}

async function resolvedCommand({ stage: s, config, }: { stage: Stage; config: GuidedConfig; },): Promise<string> {
  return typeof s.command === 'string' ? s.command : await s.command({ config, },);
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
              expect(error,).toBeInstanceOf(GuidedStop,);
              expect((error as GuidedStop).status,).toBe(1,);
              expect((error as GuidedStop).rollback,).toBe('undo bad',);
              expect(await exists(join(config.wiz, 'done', 'prepare', 'ok',),),).toBe(true,);
              expect(await exists(join(config.wiz, 'done', 'prepare', 'bad',),),).toBe(false,);
              expect(await exists(join(home, 'never',),),).toBe(false,);
            },);
          },
        },),
        it({
          name: 'stops when a check fails after a command that succeeded, and accepts a listed non-zero status',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const failedCheck = await runStages({ config, stages: [stage({ id: 'c', command: 'true', check: 'false', checkText: 'the thing', },),], },);
              expect((failedCheck.error as GuidedStop).message,).toContain('its check failed: the thing',);
              const accepted = await runStages({ config, stages: [stage({ id: 'k', command: 'exit 1', okCodes: [1,], },),], },);
              expect(accepted.error,).toBeUndefined();
              const refusedZero = await runStages({ config, stages: [stage({ id: 'z', command: 'true', okCodes: [1,], },),], },);
              expect((refusedZero.error as GuidedStop).status,).toBe(1,);
            },);
          },
        },),
        it({
          name: 'stops with status 2 and runs nothing when a stage needs an assertion that was not passed, and runs with it',
          fn: async () => {
            await withConfig(async ({ config, home, },) => {
              const stages = [stage({ id: 'ask', command: `touch ${home}/ran`, assertion: 'no agent works.', },),];
              const refused = await runStages({ config, stages, },);
              expect((refused.error as GuidedStop).status,).toBe(2,);
              expect((refused.error as GuidedStop).message,).toContain('--assert=ask',);
              expect(await exists(join(home, 'ran',),),).toBe(false,);
              expect((await runStages({ config, stages, flags: { asserted: ['ask',], }, },)).error,).toBeUndefined();
              expect(await exists(join(home, 'ran',),),).toBe(true,);
            },);
          },
        },),
        it({
          name: 'reports a stage past its timeout as a failure',
          fn: async () => {
            await withConfig(async ({ config, },) => {
              const { error, } = await runStages({ config, stages: [stage({ id: 'slow', command: 'sleep 20', timeoutSeconds: 1, },),], },);
              expect((error as GuidedStop).status,).toBe(1,);
              expect((error as GuidedStop).message,).toContain('exited 124',);
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
              expect(await exists(join(home, 'touched',),),).toBe(false,);
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
              expect(Number((await readFile(join(config.wiz, 'seconds',), 'utf8',)).trim(),),).toBeGreaterThanOrEqual(0,);
              expect(await exists(join(config.wiz, 'done', 'prepare', 'always',),),).toBe(false,);
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
              await new Promise((resolve,) => setTimeout(resolve, 2100,));
              const { error, } = await runStages({ config, stages, },);
              expect((error as GuidedStop).status,).toBe(3,);
              expect(await exists(join(home, 'later',),),).toBe(false,);
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
              expect(await preflight(config,),).toContain('pre-flight passed',);
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
              for (const [index, path,] of ['/r', ...paths,].entries()) {
                await writeFile(join(config.moveState, `f${String(index,).padStart(3, '0',)}.snapshot.json`,), JSON.stringify({ path, },),);
              }
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
            expect(() => parseGuidedFlags(['sideways',],),).toThrow(GuidedStop,);
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
              for (const [phase, stages,] of Object.entries(sets,)) {
                expect(new Set(stages.map((s,) => s.id),).size,).toBe(stages.length,);
                for (const s of stages) {
                  const command = await resolvedCommand({ stage: s, config, },);
                  expect(s.unprivileged.length,).toBeGreaterThan(10,);
                  // The one stage that only prints the removals for a person to run names sudo in text it echoes.
                  const usesSudo = s.id === 'f8-cleanup-printed' ? false : /\bsudo\b/v.test(command,);
                  expect([phase, s.id, usesSudo,],).toEqual([phase, s.id, s.root === 'yes',],);
                  expect(/(^|[\s;|&(])node\s/u.test(command,),).toBe(false,);
                  for (const line of command.split('\n',).filter((row,) => row.includes('sudo',) && (row.includes(config.copyTool,) || row.includes(config.scanTool,)))) {
                    expect(line,).toContain(`sudo ${config.nodePath} `,);
                  }
                }
              }
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
              for (const command of ['prepare', 'cutover', 'thaw', 'rollback',]) {
                const result = await execFileAsync(process.execPath, [BIN, command, '--dry-run',], {
                  encoding: 'utf8',
                  env: { ...process.env, HOME: home, },
                  cwd: '/',
                },);
                expect(result.stdout,).toContain(`${command} finished (dry run`,);
                expect(result.stdout,).toContain('== stage 1/',);
              }
              expect(await exists(config.wiz,),).toBe(false,);
              expect(await exists(config.moveState,),).toBe(false,);
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
                code = (error as { code: number; }).code;
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
                code = (error as { code: number; }).code;
                stderr = (error as { stdout: string; }).stdout;
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

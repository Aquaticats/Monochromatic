/**
 Tests for the daily update: the order of its steps, what a failed step restores, what it commits and whom it tells.
 Every effect is a scripted stand-in, so no test touches the network, git or the desktop.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  dailyUpdateGroups,
  type DailyUpdateEffects,
  isAlive,
  renderUnit,
  runCommandBounded,
  runDailyUpdate,
  takeRunLock,
  type UpdateCommand,
  SERVICE_UNIT_NAME,
  TIMER_UNIT_NAME,
} from '../dist/final/node/testing.mjs';

/**
 Folder of the versioned unit sources, next to the package's `src` folder.
 */
const SYSTEMD_DIRECTORY = join(import.meta.dirname, '..', 'systemd',);

/**
 A process id above the kernel's ceiling, so no process can have it.
 */
const DEAD_PROCESS_ID = '9999999';

/**
 Highest disk weight the repository allows an agent-started unit.
 */
const MAXIMUM_IO_WEIGHT = 10;

//region Fixtures: a repository in memory and scripted commands

/**
 Absolute path of the pretend repository.
 */
const REPOSITORY = '/repo';

/**
 Absolute path of the pretend machine-global lock.
 */
const GLOBAL_LOCK = '/config/toolchain.lock.json';

/**
 What a fake run recorded and what the fake files hold.
 */
type World = {
  /**
   Labels of the commands that ran, in order.
   */
  readonly ran: string[];
  /**
   File texts by absolute path.
   */
  readonly files: Map<string, string>;
  /**
   Commits made.
   */
  readonly commits: { readonly message: string; readonly paths: readonly string[]; }[];
  /**
   Notifications sent.
   */
  readonly notifications: { readonly title: string; readonly body: string; }[];
};

/**
 Builds a world and the effects over it.

 @param options - tracked file texts, the labels of commands that fail, per-command file edits, dirty paths and whether commits fail

 @returns the effects and the world they act on
 */
function fakeWorld({
  tracked,
  failing = [],
  edits = {},
  dirty = [],
  commitFails = false,
  timedOut = [],
}: {
  readonly tracked: Readonly<Record<string, string>>;
  readonly failing?: readonly string[];
  readonly edits?: Readonly<Record<string, Readonly<Record<string, string>>>>;
  readonly dirty?: readonly string[];
  readonly commitFails?: boolean;
  readonly timedOut?: readonly string[];
},): { readonly effects: DailyUpdateEffects; readonly world: World; } {
  const world: World = {
    ran: [],
    files: new Map(Object.entries(tracked,).map(([path, text,],) => [`${REPOSITORY}/${path}`, text,]),),
    commits: [],
    notifications: [],
  };
  world.files.set(GLOBAL_LOCK, 'global-before',);
  /**
   Paths that became dirty because a command edited them.
   */
  const edited = new Set<string>();
  const effects: DailyUpdateEffects = {
    repository: REPOSITORY,
    runCommand: (command: UpdateCommand,) => {
      world.ran.push(command.label,);
      for (const [path, text,] of Object.entries(edits[command.label] ?? {},)) {
        world.files.set(path.startsWith('/',) ? path : `${REPOSITORY}/${path}`, text,);
        edited.add(path,);
      }
      /**
       Whether this command is scripted to fail or to run out of time.
       */
      const failed = failing.includes(command.label,) || timedOut.includes(command.label,);
      return Promise.resolve({
        exitCode: failed ? 1 : 0,
        timedOut: timedOut.includes(command.label,),
        outputTail: failed ? 'the last output line' : '',
      },);
    },
    dirtyPaths: () => Promise.resolve([...dirty, ...edited,],),
    trackedPaths: () => Promise.resolve(Object.keys(tracked,),),
    readText: (absolutePath,) => Promise.resolve(world.files.get(absolutePath,) ?? '',),
    writeText: ({ absolutePath, text, },) => {
      world.files.set(absolutePath, text,);
      return Promise.resolve();
    },
    commit: ({ message, paths, },) => {
      if (commitFails) {
        return Promise.reject(new Error('commit refused',),);
      }
      world.commits.push({ message, paths, },);
      return Promise.resolve();
    },
    notify: (request,) => {
      world.notifications.push(request,);
      return Promise.resolve();
    },
    log: () => undefined,
  };
  return { effects, world, };
}

/**
 Tracked files of a small repository: the toolchain lock, the pnpm files, the Rust catalog, one crate.
 */
const TRACKED: Readonly<Record<string, string>> = {
  'toolchain.lock.json': 'lock-before',
  'toolchain.plan.jsonc': 'plan',
  'pnpm-lock.yaml': 'pnpm-lock-before',
  'pnpm-workspace.yaml': 'workspace-before',
  'rust-catalog.toml': 'catalog-before',
  'package/a/Cargo.lock': 'cargo-lock-before',
  'package/a/Cargo.toml': 'manifest-before',
  'README.md': 'readme',
};

/**
 Groups as the real run builds them.
 */
const GROUPS = dailyUpdateGroups({ globalLockPath: GLOBAL_LOCK, },);

//endregion Fixtures: a repository in memory and scripted commands

await describe({
  name: 'daily update',
  children: [
    describe({
      name: dailyUpdateGroups.name,
      children: [
        it({
          name: 'orders the dependency steps so catalogs come before manifests and manifests before lock files',
          fn: async () => {
            const { effects, world, } = fakeWorld({ tracked: TRACKED, },);
            await runDailyUpdate({ effects, groups: GROUPS, },);
            /**
             Position of a command in the run.
             */
            const at = (label: string,): number => world.ran.indexOf(label,);
            expect(at('mise run catalog:tighten',),).toBeGreaterThan(at('mise exec -- cargo-shadow update',),);
            expect(at('mise run catalog:tighten',),).toBeGreaterThan(at('mise run deps:update',),);
            expect(at('mise run sync:files',),).toBeGreaterThan(at('mise run catalog:tighten',),);
            expect(at('mise exec -- cargo-shadow install',),).toBeGreaterThan(at('mise run sync:files',),);
            expect(at('mise exec -- pnpm install',),).toBeGreaterThan(at('mise run sync:files',),);
            expect(at('mise run check:files',),).toBeGreaterThan(at('mise exec -- cargo-shadow install',),);
          },
        },),
        it({
          name: 'updates before it installs in each toolchain group',
          fn: async () => {
            const { effects, world, } = fakeWorld({ tracked: TRACKED, },);
            await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.ran.indexOf('mise run toolchain:update',),).toBeLessThan(world.ran.indexOf('mise run toolchain:install',),);
            expect(world.ran.indexOf('mise run toolchain:update:global',),).toBeLessThan(
              world.ran.indexOf('mise run toolchain:install:global',),
            );
          },
        },),
        it({
          name: 'gives every command a finite time bound',
          fn: async () => {
            for (const group of GROUPS) {
              for (const step of group.steps) {
                for (const command of step.commands) {
                  expect(Number.isFinite(command.timeoutMs,),).toBe(true,);
                  expect(command.timeoutMs,).toBeGreaterThan(0,);
                }
              }
            }
          },
        },),
      ],
    },),
    describe({
      name: runDailyUpdate.name,
      children: [
        it({
          name: 'commits exactly the owned files that changed and tells nobody when everything succeeds',
          fn: async () => {
            const { effects, world, } = fakeWorld({
              tracked: TRACKED,
              edits: {
                'mise run toolchain:update': { 'toolchain.lock.json': 'lock-after', },
                'mise run deps:update': { 'pnpm-lock.yaml': 'pnpm-lock-after', },
                'mise exec -- cargo-shadow update': { 'package/a/Cargo.lock': 'cargo-lock-after', },
              },
            },);
            const reports = await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.commits.map((commit,) => [...commit.paths,].toSorted(),),).toEqual([
              ['toolchain.lock.json',],
              ['package/a/Cargo.lock', 'pnpm-lock.yaml',],
            ],);
            expect(world.notifications,).toEqual([],);
            expect(reports.map((report,) => report.outcome,),).toEqual(['landed', 'unchanged', 'landed',],);
          },
        },),
        it({
          name: 'puts the group back and commits nothing when its install fails, while the other groups still run',
          fn: async () => {
            const { effects, world, } = fakeWorld({
              tracked: TRACKED,
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'lock-after', }, },
              failing: ['mise run toolchain:install',],
            },);
            const reports = await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.files.get(`${REPOSITORY}/toolchain.lock.json`,),).toBe('lock-before',);
            expect(world.commits,).toEqual([],);
            expect(reports[0]?.outcome,).toBe('failed',);
            expect(world.ran,).toContain('mise run deps:update',);
            expect(world.notifications.length,).toBe(1,);
            expect(world.notifications[0]?.body,).toContain('nothing from this group was landed',);
            expect(world.notifications[0]?.body,).toContain('the last output line',);
          },
        },),
        it({
          name: 'puts the machine-global lock back when its install fails, and commits nothing for it',
          fn: async () => {
            const { effects, world, } = fakeWorld({
              tracked: TRACKED,
              edits: { 'mise run toolchain:update:global': { [GLOBAL_LOCK]: 'global-after', }, },
              failing: ['mise run toolchain:install:global',],
            },);
            await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.files.get(GLOBAL_LOCK,),).toBe('global-before',);
            expect(world.commits,).toEqual([],);
          },
        },),
        it({
          name: 'restores only the pnpm files when the pnpm update fails and lands the Cargo update',
          fn: async () => {
            const { effects, world, } = fakeWorld({
              tracked: TRACKED,
              edits: {
                'mise run deps:update': { 'pnpm-lock.yaml': 'pnpm-lock-half-written', },
                'mise exec -- cargo-shadow update': { 'package/a/Cargo.lock': 'cargo-lock-after', },
              },
              failing: ['mise run deps:update',],
            },);
            await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.files.get(`${REPOSITORY}/pnpm-lock.yaml`,),).toBe('pnpm-lock-before',);
            expect(world.commits.at(-1,)?.paths,).toEqual(['package/a/Cargo.lock',],);
            expect(world.notifications[0]?.body,).toContain('the rest of the group went on',);
          },
        },),
        it({
          name: 'reports a command that ran past its time bound as stopped, not as an exit status',
          fn: async () => {
            const { effects, world, } = fakeWorld({ tracked: TRACKED, timedOut: ['mise run toolchain:update',], },);
            await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.notifications[0]?.body,).toContain('ran past its time bound',);
          },
        },),
        it({
          name: 'skips a group whose files have uncommitted changes, runs none of its commands and names the files',
          fn: async () => {
            const { effects, world, } = fakeWorld({ tracked: TRACKED, dirty: ['package/a/Cargo.toml',], },);
            const reports = await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(reports.map((report,) => report.outcome,),).toEqual(['unchanged', 'unchanged', 'skipped',],);
            expect(world.ran,).not.toContain('mise run deps:update',);
            expect(world.notifications[0]?.body,).toContain('package/a/Cargo.toml',);
          },
        },),
        it({
          name: 'leaves a changed file outside the group uncommitted and says so',
          fn: async () => {
            const { effects, world, } = fakeWorld({
              tracked: TRACKED,
              edits: {
                'mise run toolchain:update': { 'toolchain.lock.json': 'lock-after', },
                'mise run toolchain:install': { 'README.md': 'readme-changed', },
              },
            },);
            await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(world.commits[0]?.paths,).toEqual(['toolchain.lock.json',],);
            expect(world.notifications[0]?.body,).toContain('README.md',);
          },
        },),
        it({
          name: 'reports a failed commit and keeps the changed files in the working tree',
          fn: async () => {
            const { effects, world, } = fakeWorld({
              tracked: TRACKED,
              edits: { 'mise run toolchain:update': { 'toolchain.lock.json': 'lock-after', }, },
              commitFails: true,
            },);
            const reports = await runDailyUpdate({ effects, groups: GROUPS, },);
            expect(reports[0]?.outcome,).toBe('failed',);
            expect(world.files.get(`${REPOSITORY}/toolchain.lock.json`,),).toBe('lock-after',);
            expect(world.notifications[0]?.body,).toContain('commit failed',);
          },
        },),
      ],
    },),
    describe({
      name: renderUnit.name,
      children: [
        it({
          name: 'replaces every placeholder and refuses a placeholder without a value',
          fn: async () => {
            expect(renderUnit({ template: 'a=@ONE@ b=@ONE@ c=@TWO@', values: { ONE: '1', TWO: '2', }, },),).toBe('a=1 b=1 c=2',);
            expect(() => renderUnit({ template: '@MISSING@', values: {}, },),).toThrow('MISSING',);
          },
        },),
      ],
    },),
    describe({
      name: 'unit file sources',
      children: [
        it({
          name: 'start nothing at login: the service has no install section and the timer is the only scheduler',
          fn: async () => {
            const service = (await readFile(join(SYSTEMD_DIRECTORY, SERVICE_UNIT_NAME,), 'utf8',)).split('\n',);
            const timer = (await readFile(join(SYSTEMD_DIRECTORY, TIMER_UNIT_NAME,), 'utf8',)).split('\n',);
            expect(service.includes('[Install]',),).toBe(false,);
            expect(timer.includes('OnCalendar=daily',),).toBe(true,);
            expect(timer.includes('Persistent=true',),).toBe(true,);
            expect(timer.some((line,) => line.startsWith('RandomizedDelaySec=',),),).toBe(true,);
          },
        },),
        it({
          name: 'cap the service with a disk weight of at most ten and a finite start timeout',
          fn: async () => {
            const service = (await readFile(join(SYSTEMD_DIRECTORY, SERVICE_UNIT_NAME,), 'utf8',)).split('\n',);
            const weightLine = service.find((line,) => line.startsWith('IOWeight=',),) ?? 'IOWeight=NaN';
            const weight = Number(weightLine.slice('IOWeight='.length,),);
            expect(weight,).toBeLessThanOrEqual(MAXIMUM_IO_WEIGHT,);
            expect(weight,).toBeGreaterThan(0,);
            for (const name of ['TimeoutStartSec', 'CPUQuota', 'MemoryMax', 'TasksMax',]) {
              expect(service.some((line,) => line.startsWith(`${name}=`,),),).toBe(true,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'the run lock and bounded commands',
      children: [
        it({
          name: 'refuses a second holder, replaces the lock of a dead process, and knows which processes live',
          fn: async () => {
            const directory = await mkdtemp(join(scratchRoot(), 'daily-update-lock-',),);
            const lockPath = join(directory, 'run.lock',);
            await using first = await takeRunLock(lockPath,);
            expect(first.acquired,).toBe(true,);
            await using second = await takeRunLock(lockPath,);
            expect(second.acquired,).toBe(false,);
            await writeFile(lockPath, DEAD_PROCESS_ID,);
            expect(
              isAlive(Number(DEAD_PROCESS_ID,),),
            ).toBe(false,);
            await using third = await takeRunLock(lockPath,);
            expect(third.acquired,).toBe(true,);
            expect(isAlive(process.pid,),).toBe(true,);
            await rm(directory, { recursive: true, force: true, },);
          },
        },),
        it({
          name: 'stops a command that runs past its time bound and reports it as timed out',
          fn: async () => {
            const outcome = await runCommandBounded({
              repository: scratchRoot(),
              command: { label: 'sleep', argv: ['node', '-e', 'setTimeout(() => {}, 60000)',], timeoutMs: 300, },
            },);
            expect(outcome.timedOut,).toBe(true,);
            expect(outcome.exitCode,).not.toBe(0,);
          },
        },),
        it({
          name: 'keeps the tail of a finished command output and its exit status',
          fn: async () => {
            const outcome = await runCommandBounded({
              repository: scratchRoot(),
              command: { label: 'echo', argv: ['node', '-e', 'console.log("hello"); process.exit(3)',], timeoutMs: 30_000, },
            },);
            expect(outcome.exitCode,).toBe(3,);
            expect(outcome.timedOut,).toBe(false,);
            expect(outcome.outputTail,).toContain('hello',);
          },
        },),
      ],
    },),
  ],
},);

/**
 The pool gate as a program, against stand-in `systemctl`, `git`, `mise`, `systemd-run` and `pi` programs on a `PATH` of the
 test's own. Nothing here reads this machine's units, starts a unit, or makes a worktree of this repository.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { delimiter, dirname, join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  ALREADY_RUNNING_EXIT_STATUS,
  BUILD_VOLUME,
  BYTES_IN_A_GIBIBYTE,
  FREE_SPACE_FLOOR_BYTES,
  LAUNCHER_ENTRY_NAMES,
  NEURALWATT_STANDARD,
  NO_POOL_HAS_ROOM,
  NO_SLOT_EXIT_STATUS,
  PROVIDER_POOLS,
  type ProviderPool,
  firstDefaultPoolWithRoom,
  freeSpaceFloorTextOf,
  freeSpaceTextOf,
  poolTakesNoNewRun,
  talliesOf,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 The pool gate's entry, run from source so the test exercises the same file the task runs.
 */
const GATE = new URL('start-pi-run-when-a-pool-has-room.ts', import.meta.url,).pathname;

/**
 A folder a test throws away, holding its own programs and runs folder.

 */
type BareFolder = {
  readonly path: string;
  readonly bins: string;
  readonly runs: string;
  readonly processes: string;
  readonly log: string;
  readonly freeSpaceBytes: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Room on the build volume of a test that lets a run start, which is above the floor.

 @returns the room in bytes

 @example
 ```ts
 const room = roomToSpareBytes();
 ```
 */
function roomToSpareBytes(): string {
  return String(FREE_SPACE_FLOOR_BYTES + BYTES_IN_A_GIBIBYTE,);
}

/**
 Room on the build volume of a test that a run does not start in.

 @returns the room in bytes

 @example
 ```ts
 const room = roomUnderTheFloorBytes();
 ```
 */
function roomUnderTheFloorBytes(): string {
  return String(FREE_SPACE_FLOOR_BYTES - 1,);
}

/**
 The pool a run that names no model of its own takes while every pool is empty, read from the pool table rather than written
 down here.

 @returns the pool

 @throws Error when the table takes no default run at all, so the expectation built from it could not hold

 @example
 ```ts
 const pool = theFirstDefaultPool();
 ```
 */
function theFirstDefaultPool(): ProviderPool {
  /**
   What the pool table answers about a machine on which nothing is going.
   */
  const pool = firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: [], },), },);
  if (pool === NO_POOL_HAS_ROOM) {
    throw new Error('no pool takes a default run, so this test cannot expect a pool to take one',);
  }
  return pool;
}

/**
 Every pool a run that names no model of its own may take, which is every pool that takes default runs and is open.

 @returns the pools, in the order such a run takes them

 @example
 ```ts
 const pools = openDefaultPools();
 ```
 */
function openDefaultPools(): readonly ProviderPool[] {
  return PROVIDER_POOLS.filter(function takesDefaultRunsAndIsOpen(pool,) {
    return pool.takesDefaultRuns && (!poolTakesNoNewRun({ pool, },));
  },);
}

/**
 Says the counts line names every pool the table ships, in the table's order, each with the count this test expects and its own
 limit, and ends with the room the build volume has.

 @param line - what the gate printed

 @param counts - the count each pool is at, by the pool's name

 @param freeSpaceBytes - the room the test gave the build volume

 @example
 ```ts
 expectCountsLine({ line: ran.stdout.trim(), counts: new Map([[NEURALWATT_STANDARD.name, 1]]), freeSpaceBytes: roomToSpareBytes() });
 ```
 */
function expectCountsLine({
  line,
  counts,
  freeSpaceBytes,
}: {
  readonly line: string;
  readonly counts: ReadonlyMap<string, number>;
  readonly freeSpaceBytes: string;
},): void {
  /**
   The pools, and the room the volume has, which the line names after a semicolon.
   */
  const [summary, room,] = line.split('; ',);
  /**
   One entry per pool, in the order the line prints them.
   */
  const entries = (summary ?? '').split(', ',);
  expect(entries.length,).toBe(PROVIDER_POOLS.length,);
  for (const [index, pool,] of PROVIDER_POOLS.entries()) {
    expect(entries[index],).toBe(`${pool.name} ${String(counts.get(pool.name,) ?? 0,)}/${String(pool.limit,)}`,);
  }
  expect(room,).toBe(freeSpaceTextOf({
    volume: BUILD_VOLUME,
    freeBytes: Number(freeSpaceBytes,),
  },),);
}

/**
 Makes a folder with a program folder, a runs folder and a log the stand-ins write to.

 @returns the folder

 @example
 ```ts
 await using folder = await makeBareFolder();
 ```
 */
async function makeBareFolder({
  freeSpaceBytes = roomToSpareBytes(),
}: {
  readonly freeSpaceBytes?: string;
} = {},): Promise<BareFolder> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-pool-gate-',),);
  /**
   Where the stand-in programs live.
   */
  const bins = join(path, 'bin',);
  /**
   Where the runs live.
   */
  const runs = join(path, 'runs',);
  /**
   The process folder, holding no process at all.
   */
  const processes = join(path, 'proc',);
  await mkdir(bins,);
  await mkdir(runs,);
  await mkdir(processes,);
  return {
    path,
    bins,
    runs,
    processes,
    log: join(path, 'commands.log',),
    freeSpaceBytes,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one stand-in program.

 @param folder - the folder the programs live in

 @param name - the program's name

 @param body - what it does, as a bash body

 @example
 ```ts
 await writeProgram({ folder: bins, name: 'systemctl', body: 'exit 0', });
 ```
 */
async function writeProgram({
  folder,
  name,
  body,
}: {
  readonly folder: string;
  readonly name: string;
  readonly body: string;
}): Promise<void> {
  /**
   The program's file.
   */
  const file = join(folder, name,);
  await writeFile(file, `#!/usr/bin/env bash\nprintf '%s %s\\n' '${name}' "$*" >> "$FAKE_LOG"\n${body}\n`,);
  await chmod(file, 0o700,);
}

/**
 Writes the stand-ins the gate and the launcher ask, each recording what it was asked.

 @param folder - the folder the programs live in

 @param units - what `systemctl list-units` prints

 @param execStart - what `systemctl show --property=ExecStart --value` prints

 @example
 ```ts
 await writeStandIns({ folder: bins, units: '', execStart: '' });
 ```
 */
async function writeStandIns({
  folder,
  units,
  execStart,
}: {
  readonly folder: string;
  readonly units: string;
  readonly execStart: string;
}): Promise<void> {
  await writeProgram({
    folder,
    name: 'systemctl',
    body: [
      'case "$*" in',
      `  *--property=ExecStart*) printf '%s\\n' "${execStart}" ;;`,
      `  *list-units*) printf '%s' "${units}" ;;`,
      '  *) exit 0 ;;',
      'esac',
    ].join('\n',),
  },);
  await writeProgram({
    folder,
    name: 'git',
    body: [
      'case "$*" in',
      `  *'rev-parse --show-toplevel'*) printf '%s\\n' "$FAKE_CHECKOUT" ;;`,
      `  *'rev-parse --verify'*) exit 0 ;;`,
      `  *'worktree add'*) mkdir --parents "$FAKE_CHECKOUT/.claude/worktrees/pi-copy-route" ;;`,
      `  *'symbolic-ref'*) printf 'refs/heads/feat/a\\n' ;;`,
      `  *'status --porcelain'*) exit 0 ;;`,
      '  *) exit 0 ;;',
      'esac',
    ].join('\n',),
  },);
  await writeProgram({
    folder,
    name: 'mise',
    body: [
      'case "$*" in',
      `  *'printenv PATH'*) printf '%s\\n' "$FAKE_PATH" ;;`,
      `  *'command -v node'*) printf '%s\\n' "$(command -v node)" ;;`,
      '  *) exit 0 ;;',
      'esac',
    ].join('\n',),
  },);
  await writeProgram({
    folder,
    name: 'systemd-run',
    body: 'exit 0',
  },);
  await writeProgram({
    folder,
    name: 'pi',
    body: 'exit 0',
  },);
}

/**
 Writes one launcher process into the test's process folder.

 @param folder - the process folder

 @param pid - the process's number

 @param words - the words the kernel split its command line into

 @example
 ```ts
 await writeProcess({ folder: processes, pid: '101', words: ['node', 'launcher.ts', '--name=a'], });
 ```
 */
async function writeProcess({
  folder,
  pid,
  words,
}: {
  readonly folder: string;
  readonly pid: string;
  readonly words: readonly string[];
}): Promise<void> {
  await mkdir(join(folder, pid,), { recursive: true, },);
  await writeFile(join(folder, pid, 'cmdline',), `${words.join('\0',)}\0`,);
}

/**
 Runs the pool gate.

 @param folder - the folder whose programs and runs the gate uses

 @param words - the gate's own words

 @returns the status and both streams

 @example
 ```ts
 const ran = await runTheGate({ folder, words: ['--counts-only',], });
 ```
 */
async function runTheGate({
  folder,
  words,
}: {
  readonly folder: BareFolder;
  readonly words: readonly string[];
}): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   The programs the gate may run, then the node that runs it.
   */
  const path = [
    folder.bins,
    dirname(process.execPath,),
    '/usr/bin',
  ].join(delimiter,);
  try {
    /**
     What the gate printed.
     */
    const done = await execFileAsync(process.execPath, [
      GATE,
      ...words,
    ], {
      env: {
        PATH: path,
        HOME: folder.path,
        FAKE_LOG: folder.log,
        FAKE_CHECKOUT: folder.path,
        FAKE_PATH: path,
        FAKE_UNITS: '',
        FAKE_EXEC_START: '',
        PI_RUNS_PROCESS_FOLDER: folder.processes,
        PI_RUNS_FREE_SPACE_BYTES: folder.freeSpaceBytes,
      },
      encoding: 'utf8',
      timeout: 60_000,
    },);
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    /**
     What the failure carries.
     */
    const failure = caught as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : 1,
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

/**
 Reads what the stand-in programs were asked.

 @param folder - the folder whose log is read

 @returns one line per program run

 @example
 ```ts
 const asked = await programsAsked(folder);
 ```
 */
async function programsAsked(folder: BareFolder,): Promise<readonly string[]> {
  try {
    return (await readFile(folder.log, 'utf8',)).split('\n',)
      .filter(function isLine(line,) {
        return line !== '';
      },);
  }
  catch (error) {
    // No program ran, which is what a test of the counts asks about.
    if ((error as { readonly code?: string }).code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

await describe({
  name: 'start-pi-run-when-a-pool-has-room',
  children: [
    it({
      name: 'prints the counts with the room the build volume has, and starts nothing, on a command line that names no run',
      fn: async () => {
        await using folder = await makeBareFolder();
        /**
         A unit whose own command line names the standard pool.
         */
        const units = `pi-copy-route.service x\n`;
        /**
         What that unit's command line says.
         */
        const execStart = `{ argv[]=node drive.ts --provider=${NEURALWATT_STANDARD.provider} --model=${NEURALWATT_STANDARD.model} ; }`;
        await writeStandIns({
          folder: folder.bins,
          units,
          execStart,
        },);
        /**
         What the gate answered.
         */
        const ran = await runTheGate({
          folder,
          words: ['--counts-only', '--runs-folder=/nowhere',],
        },);
        expect(ran.status,).toBe(0,);
        expectCountsLine({
          line: ran.stdout.trim(),
          counts: new Map([[NEURALWATT_STANDARD.name, 1,],],),
          freeSpaceBytes: folder.freeSpaceBytes,
        },);
        expect(await programsAsked(folder,),).toEqual([
          'systemctl --user list-units --state=active,activating --no-legend --plain pi-*.service',
          'systemctl --user show pi-copy-route.service --property=ExecStart --value',
        ],);
      },
    },),
    it({
      name: 'answers that the run is already going, and starts nothing',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: 'pi-copy-route.service x\n',
          execStart: `{ argv[]=node drive.ts --provider=${NEURALWATT_STANDARD.provider} --model=${NEURALWATT_STANDARD.model} ; }`,
        },);
        /**
         What the gate answered about a name that is already going.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=copy-route', '--runs-folder=/nowhere',],
        },);
        expect(ran.status,).toBe(ALREADY_RUNNING_EXIT_STATUS,);
        expect(ran.stdout.trim(),).toBe('ALREADY RUNNING: copy-route',);
        expect((await programsAsked(folder,)).some(function ranTheLauncher(line,) {
          return line.startsWith('systemd-run',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'answers no slot while every pool that takes the run is full, with the status the retrying start reads',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         Every pool a run that names a default model could take, which is every pool that takes default runs and is open.
         */
        const pools = openDefaultPools();
        expect(pools.length,).toBeGreaterThan(0,);
        /**
         The process number the launchers are written under.
         */
        let pid = 100;
        for (const filled of pools) {
          for (let index = 0; index < filled.limit; index += 1) {
            pid += 1;
            // oxlint-disable-next-line eslint/no-await-in-loop -- one launcher process after another, each a file of its own; see doc/lint/sequential-awaits.md
            await writeProcess({
              folder: folder.processes,
              pid: String(pid,),
              words: [
                '/usr/bin/node',
                join(folder.path, LAUNCHER_ENTRY_NAMES[0] ?? '',),
                `--name=run-${String(pid,)}`,
                `--provider=${filled.provider}`,
                `--model=${filled.model}`,
              ],
            },);
          }
        }
        /**
         What the gate answered.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=new-run', '--runs-folder=/nowhere',],
        },);
        expect(ran.status,).toBe(NO_SLOT_EXIT_STATUS,);
        expect(ran.stdout,).toContain('NO SLOT:',);
        expect(ran.stdout,).toContain('not started: new-run',);
      },
    },),
    it({
      name: 'starts the run through the launcher, whose unit names the pool the gate chose, and whose folder is a new number',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         A finished run of the same name, so the turn cap is sized instead of refused.
         */
        await mkdir(join(folder.runs, 'copy-route-1',), { recursive: true, },);
        await writeFile(join(folder.runs, 'copy-route-1', 'summary.json',), JSON.stringify({
          endReason: 'idle',
          turns: 40,
        },),);
        /**
         What the gate answered.
         */
        const ran = await runTheGate({
          folder,
          words: [
            '--name=copy-route',
            `--runs-folder=${folder.runs}`,
            '--branch=feat/copy-route',
            `--prompt=${join(folder.path, 'prompt.md',)}`,
            '--cpu=800',
            '--memory=1G',
            '--tasks=100',
            '--wall=600',
          ],
        },);
        expect(ran.status,).toBe(0,);
        expect(ran.stdout,).toContain('slot free:',);
        /**
         The pool the gate's own order picks while nothing is going.
         */
        const chosen = theFirstDefaultPool();
        expect(ran.stdout,).toContain(`copy-route goes on ${chosen.name}`,);
        /**
         What the stand-in programs were asked.
         */
        const asked = await programsAsked(folder,);
        /**
         The unit command the launcher ran.
         */
        const unit = asked.find(function isTheUnit(line,) {
          return line.startsWith('systemd-run --user --wait',);
        },);
        expect(unit,).toContain('--unit=pi-copy-route.service',);
        expect(unit,).toContain(`--run-folder=${join(folder.runs, 'copy-route-2',)}`,);
        expect(unit,).toContain(`--provider=${chosen.provider}`,);
        expect(unit,).toContain(`--model=${chosen.model}`,);
        expect(unit,).toContain('--turn-cap=50',);
        expect(asked.some(function addedAWorktree(line,) {
          return line.startsWith('git worktree add',);
        },),).toBe(true,);
        expect(asked.some(function linkedDependencies(line,) {
          return line.startsWith('mise ') && line.includes('pnpm install --offline --frozen-lockfile',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'refuses to start a run while the build volume is under the floor, saying how much room there is and how much a run needs',
      fn: async () => {
        await using folder = await makeBareFolder({ freeSpaceBytes: roomUnderTheFloorBytes(), },);
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         What the gate answered while the volume was under the floor.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=copy-route', '--runs-folder=/nowhere',],
        },);
        expect(ran.status,).toBe(NO_SLOT_EXIT_STATUS,);
        expect(ran.stdout.trim(),).toBe(
          `NO SLOT: ${freeSpaceTextOf({
            volume: BUILD_VOLUME,
            freeBytes: Number(roomUnderTheFloorBytes(),),
          },)}, under the floor of ${freeSpaceFloorTextOf()}; not started: copy-route`,
        );
        expect((await programsAsked(folder,)).some(function ranTheLauncher(line,) {
          return line.startsWith('systemd-run',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses to size a turn cap from nothing, naming the option that would',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         What the gate answered for a runs folder with no finished run.
         */
        const ran = await runTheGate({
          folder,
          words: [
            '--name=copy-route',
            `--runs-folder=${folder.runs}`,
            '--branch=feat/copy-route',
            `--prompt=${join(folder.path, 'prompt.md',)}`,
            '--cpu=800',
            '--memory=1G',
            '--tasks=100',
            '--wall=600',
          ],
        },);
        expect(ran.status,).not.toBe(0,);
        expect(ran.stderr,).toContain('--turn-cap is required',);
      },
    },),
  ],
},);

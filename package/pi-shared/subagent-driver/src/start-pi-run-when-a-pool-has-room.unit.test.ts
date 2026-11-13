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
  BUILD_VOLUME_FLOOR_BYTES,
  BYTES_IN_A_GIBIBYTE,
  FOLDER_VOLUMES_VARIABLE,
  LAUNCHER_ENTRY_NAMES,
  MACHINE_CAP_DEFAULT,
  MACHINE_CAP_VARIABLE,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_KIMI_FLEX,
  NEURALWATT_STANDARD,
  NO_POOL_HAS_ROOM,
  NO_SLOT_EXIT_STATUS,
  OPENCODE_GO,
  OTHER_AGENTS_GOING_VARIABLE,
  POOL_LIMITS_VARIABLE,
  PROVIDER_POOLS,
  type ProviderPool,
  RUNS_FOLDER_FLOOR_BYTES,
  SESSION_FILES_FLOOR_BYTES,
  VOLUME_FREE_SPACE_BYTES_VARIABLE,
  firstDefaultPoolWithRoom,
  freeSpaceTextOf,
  freeSpaceUnderTheFloorTextOf,
  machineCapCountsTextOf,
  machineCapRefusalTextOf,
  poolTakesNoNewRun,
  roomIn,
  sessionFolderOf,
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
 The volumes of the test's own, which are none of this machine's: one holding the runs folder, and one holding Pi's session files.
 */
const TEST_RUNS_VOLUME = '/a-runs-volume-of-the-test';
const TEST_SESSION_FILES_VOLUME = '/a-session-files-volume-of-the-test';

/**
 A folder a test throws away, holding its own programs and runs folder.

 */
type BareFolder = {
  readonly path: string;
  readonly bins: string;
  readonly runs: string;
  readonly processes: string;
  readonly log: string;
  readonly buildVolumeRoom: string;
  readonly runsVolumeRoom: string;
  readonly sessionFilesVolumeRoom: string;
  readonly runsFolderVolume: string;
  readonly sessionFilesFolder: string;
  readonly sessionFilesFolderVolume: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Room on the build volume of a test that lets a run start, which is above the floor.

 @returns the room in bytes

 @example
 ```ts
 const room = buildVolumeRoomToSpareBytes();
 ```
 */
function buildVolumeRoomToSpareBytes(): string {
  return String(BUILD_VOLUME_FLOOR_BYTES + BYTES_IN_A_GIBIBYTE,);
}

/**
 Room on the build volume of a test that a run does not start in.

 @returns the room in bytes

 @example
 ```ts
 const room = buildVolumeRoomUnderTheFloorBytes();
 ```
 */
function buildVolumeRoomUnderTheFloorBytes(): string {
  return String(BUILD_VOLUME_FLOOR_BYTES - 1,);
}

/**
 Room on a test's own volume that lets a run start, which is above that volume's floor.

 @param floorBytes - the floor a new run needs on that volume

 @returns the room in bytes

 @example
 ```ts
 const room = roomToSpareBytes(RUNS_FOLDER_FLOOR_BYTES);
 ```
 */
function roomToSpareBytes(floorBytes: number,): string {
  return String(floorBytes + BYTES_IN_A_GIBIBYTE,);
}

/**
 Room on a test's own volume that a run does not start in, which is under that volume's floor.

 @param floorBytes - the floor a new run needs on that volume

 @returns the room in bytes

 @example
 ```ts
 const room = roomUnderTheFloorBytes(RUNS_FOLDER_FLOOR_BYTES);
 ```
 */
function roomUnderTheFloorBytes(floorBytes: number,): string {
  return String(floorBytes - 1,);
}

/**
 The room the test gave a volume, by the volume's path.

 @param folder - the folder the test made

 @param volume - the volume's path

 @returns the room in bytes the gate was given for that volume

 @throws Error when the test gave no room for that volume, so the expectation built from it could not hold

 @example
 ```ts
 const room = roomTheTestGave({ folder, volume: TEST_RUNS_VOLUME });
 ```
 */
function roomTheTestGave({
  folder,
  volume,
}: {
  readonly folder: BareFolder;
  readonly volume: string;
},): string {
  if (volume === BUILD_VOLUME) {
    return folder.buildVolumeRoom;
  }
  if (volume === TEST_RUNS_VOLUME) {
    return folder.runsVolumeRoom;
  }
  if (volume === TEST_SESSION_FILES_VOLUME) {
    return folder.sessionFilesVolumeRoom;
  }
  throw new Error(`the test gave no room for the volume ${volume}`,);
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
 limit, then the count of every agent on the machine against its cap, and ends with the room of each distinct volume, once.

 @param line - what the gate printed

 @param counts - the count each pool is at, by the pool's name

 @param folder - the folder whose runs, volumes and rooms the gate was given

 @param agentsGoing - every agent going on the machine, which the test counted in its process folder

 @param cap - the cap the test gave the machine

 @example
 ```ts
 expectCountsLine({ line: ran.stdout.trim(), counts: new Map([[NEURALWATT_STANDARD.name, 1]]), folder, agentsGoing: 0 });
 ```
 */
function expectCountsLine({
  line,
  counts,
  folder,
  agentsGoing,
  cap = MACHINE_CAP_DEFAULT,
}: {
  readonly line: string;
  readonly counts: ReadonlyMap<string, number>;
  readonly folder: BareFolder;
  readonly agentsGoing: number;
  readonly cap?: number;
},): void {
  /**
   The pools, the machine's count, and the room of each volume, which the line names after a semicolon.
   */
  const [summary, machine, ...rooms] = line.split('; ',);
  /**
   One entry per pool, in the order the line prints them.
   */
  const entries = (summary ?? '').split(', ',);
  expect(entries.length,).toBe(PROVIDER_POOLS.length,);
  for (const [index, pool,] of PROVIDER_POOLS.entries()) {
    expect(entries[index],).toBe(`${pool.name} ${String(counts.get(pool.name,) ?? 0,)}/${String(roomIn(pool,),)}`,);
  }
  expect(machine,).toBe(machineCapCountsTextOf({
    agentsGoing,
    cap,
  },),);
  /**
   The distinct volumes the gate was given, in the order it reads its needs: a volume two needs share is named once.
   */
  const distinctVolumes = [
    BUILD_VOLUME,
    folder.runsFolderVolume,
    folder.sessionFilesFolderVolume,
  ].filter(function isTheFirstTime(volume, index, volumes,) {
    return volumes.indexOf(volume,) === index;
  },);
  expect(rooms,).toEqual(distinctVolumes.map(function roomOf(volume,) {
    return freeSpaceTextOf({
      volume,
      freeBytes: Number(roomTheTestGave({
        folder,
        volume,
      },),),
    },);
  },),);
}

/**
 Makes a folder with a program folder, a runs folder and a log the stand-ins write to, and the volumes and rooms the gate is
 given for it.

 The runs folder is on the build volume unless a test says otherwise, which is the machine of 2026-11-12: the runs folder was
 reached through a symbolic link onto the build volume while the drive that filled held Pi's own files.

 @param buildVolumeRoom - the room the test gives the volume that holds the repository

 @param runsFolderVolume - the volume the test says the runs folder is on

 @param runsVolumeRoom - the room the test gives that volume

 @param sessionFilesVolumeRoom - the room the test gives the volume holding Pi's session files

 @param sessionFilesFolderVolume - the volume the test says Pi's session folder is on

 @returns the folder

 @example
 ```ts
 await using folder = await makeBareFolder();
 ```
 */
async function makeBareFolder({
  buildVolumeRoom = buildVolumeRoomToSpareBytes(),
  runsFolderVolume = BUILD_VOLUME,
  runsVolumeRoom = roomToSpareBytes(RUNS_FOLDER_FLOOR_BYTES,),
  sessionFilesVolumeRoom = roomToSpareBytes(SESSION_FILES_FLOOR_BYTES,),
  sessionFilesFolderVolume = TEST_SESSION_FILES_VOLUME,
}: {
  readonly buildVolumeRoom?: string;
  readonly runsFolderVolume?: string;
  readonly runsVolumeRoom?: string;
  readonly sessionFilesVolumeRoom?: string;
  readonly sessionFilesFolderVolume?: string;
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
  /**
   The folder Pi keeps its session files in, which this test's home directory holds.
   */
  const sessionFilesFolder = sessionFolderOf({ home: path, },);
  await mkdir(bins,);
  await mkdir(runs,);
  await mkdir(processes,);
  return {
    path,
    bins,
    runs,
    processes,
    log: join(path, 'commands.log',),
    buildVolumeRoom,
    runsVolumeRoom,
    sessionFilesVolumeRoom,
    runsFolderVolume,
    sessionFilesFolder,
    sessionFilesFolderVolume,
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
  execStartFails = false,
}: {
  readonly folder: string;
  readonly units: string;
  readonly execStart: string;
  readonly execStartFails?: boolean;
}): Promise<void> {
  await writeProgram({
    folder,
    name: 'systemctl',
    body: [
      'case "$*" in',
      `  *--property=ExecStart*) ${execStartFails ? String.raw`printf '%s\n' 'No such unit' >&2; exit 1` : `printf '%s\\n' "${execStart}"`} ;;`,
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
      // The policy namespace help is what the launcher asks for before it starts a session, and only the wrapper answers it.
      `  *'monochromatic-git-policy-cli --help'*) printf '%s\\n' 'Usage: git monochromatic-git-policy-cli <command> [options]' ;;`,
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

 @param poolLimits - the limits the gate is told to use, when a test sets one

 @param machineCap - the cap the gate is told to use, when a test sets one

 @param otherAgentsGoing - the agents the caller says it has going, when a test sets some

 @returns the status and both streams

 @example
 ```ts
 const ran = await runTheGate({ folder, words: ['--counts-only',], });
 ```
 */
async function runTheGate({
  folder,
  words,
  poolLimits,
  machineCap,
  otherAgentsGoing,
}: {
  readonly folder: BareFolder;
  readonly words: readonly string[];
  readonly poolLimits?: string;
  readonly machineCap?: string;
  readonly otherAgentsGoing?: string;
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
        [VOLUME_FREE_SPACE_BYTES_VARIABLE]: [
          `${BUILD_VOLUME}=${folder.buildVolumeRoom}`,
          `${TEST_RUNS_VOLUME}=${folder.runsVolumeRoom}`,
          `${TEST_SESSION_FILES_VOLUME}=${folder.sessionFilesVolumeRoom}`,
        ].join(',',),
        [FOLDER_VOLUMES_VARIABLE]: [
          `${folder.runs}=${folder.runsFolderVolume}`,
          `${folder.sessionFilesFolder}=${folder.sessionFilesFolderVolume}`,
        ].join(',',),
        ...(poolLimits === undefined ? {} : { [POOL_LIMITS_VARIABLE]: poolLimits, }),
        ...(machineCap === undefined ? {} : { [MACHINE_CAP_VARIABLE]: machineCap, }),
        ...(otherAgentsGoing === undefined ? {} : { [OTHER_AGENTS_GOING_VARIABLE]: otherAgentsGoing, }),
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
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
        },);
        expect(ran.status,).toBe(0,);
        expectCountsLine({
          line: ran.stdout.trim(),
          counts: new Map([[NEURALWATT_STANDARD.name, 1,],],),
          folder,
          agentsGoing: 1,
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
          words: ['--name=copy-route', `--runs-folder=${folder.runs}`,],
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
        /**
         How many launchers this test writes, which is one for every slot every open pool has.
         */
        let launchers = 0;
        for (const filled of pools) {
          for (let index = 0; index < filled.limit; index += 1) {
            pid += 1;
            launchers += 1;
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
         What the gate answered, with the machine's cap set above the runs going so that the pools are what refuse it.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=new-run', `--runs-folder=${folder.runs}`,],
          machineCap: String(launchers + 1,),
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
         The prompt file a start needs before it makes anything.
         */
        await writeFile(join(folder.path, 'prompt.md',), 'Answer with one line.\n',);
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
            `--checkout-folder=${folder.path}`,
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
        await using folder = await makeBareFolder({ buildVolumeRoom: buildVolumeRoomUnderTheFloorBytes(), },);
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
          words: ['--name=copy-route', `--runs-folder=${folder.runs}`,],
        },);
        expect(ran.status,).toBe(NO_SLOT_EXIT_STATUS,);
        expect(ran.stdout.trim(),).toBe(
          `NO SLOT: ${freeSpaceUnderTheFloorTextOf({
            space: {
              volume: BUILD_VOLUME,
              freeBytes: Number(buildVolumeRoomUnderTheFloorBytes(),),
            },
            floorBytes: BUILD_VOLUME_FLOOR_BYTES,
          },)}; not started: copy-route`,
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
         The prompt file a start needs before it makes anything.
         */
        await writeFile(join(folder.path, 'prompt.md',), 'Answer with one line.\n',);
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
            `--checkout-folder=${folder.path}`,
          ],
        },);
        expect(ran.status,).not.toBe(0,);
        expect(ran.stderr,).toContain('--turn-cap is required',);
      },
    },),
    it({
      name: 'closes and reopens a pool from the environment alone, which is how a spent weekly quota is set aside',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         The counts as the gate ships them, with the pool the owner set aside closed.
         */
        const closed = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
        },);
        expect(closed.status,).toBe(0,);
        expectCountsLine({
          line: closed.stdout.trim(),
          counts: new Map(),
          folder,
          agentsGoing: 0,
        },);
        /**
         The room a caller gives the set-aside pool, which is another open pool's own limit rather than a number of the test's.
         */
        const reopenedRoom = NEURALWATT_DEEPSEEK_FLEX.limit;
        /**
         The counts with that pool reopened by the caller, and another pool closed in its place.
         */
        const reopened = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
          poolLimits: `${OPENCODE_GO.name}=${String(reopenedRoom,)},${NEURALWATT_STANDARD.name}=0`,
        },);
        expect(reopened.status,).toBe(0,);
        expect(reopened.stdout.trim(),).toContain(`${OPENCODE_GO.name} 0/${String(reopenedRoom,)}`,);
        expect(reopened.stdout.trim(),).toContain(`${NEURALWATT_STANDARD.name} 0/0`,);
        /**
         What a limits setting the gate cannot read answers with, so a mistyped pool name is never counted as a closed pool.
         */
        const refused = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
          poolLimits: 'no-such-pool=1',
        },);
        expect(refused.status,).not.toBe(0,);
        expect(refused.stderr,).toContain('no-such-pool',);
      },
    },),
    it({
      name: 'counts a unit whose own command line could not be read in every pool it could be running on, saying which it could not read',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: 'pi-copy-route.service x\n',
          execStart: '',
          execStartFails: true,
        },);
        /**
         What the gate answered.
         */
        const ran = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
        },);
        expect(ran.status,).toBe(0,);
        expectCountsLine({
          line: ran.stdout.trim(),
          counts: new Map(PROVIDER_POOLS.map(function heldOnce(pool,) {
            return [pool.name, 1,] as const;
          },),),
          folder,
          agentsGoing: 1,
        },);
        expect(ran.stderr,).toContain('pool of unit pi-copy-route.service not read',);
      },
    },),
    it({
      name: 'refuses to start a run while the volume that holds the runs folder is under its own floor, before any program runs',
      fn: async () => {
        await using folder = await makeBareFolder({
          runsFolderVolume: TEST_RUNS_VOLUME,
          runsVolumeRoom: roomUnderTheFloorBytes(RUNS_FOLDER_FLOOR_BYTES,),
        },);
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         What the gate answered while the volume holding the runs folder was under its floor.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=copy-route', `--runs-folder=${folder.runs}`,],
        },);
        expect(ran.status,).toBe(NO_SLOT_EXIT_STATUS,);
        expect(ran.stdout.trim(),).toBe(
          `NO SLOT: ${freeSpaceUnderTheFloorTextOf({
            space: {
              volume: TEST_RUNS_VOLUME,
              freeBytes: Number(roomUnderTheFloorBytes(RUNS_FOLDER_FLOOR_BYTES,),),
            },
            floorBytes: RUNS_FOLDER_FLOOR_BYTES,
          },)}; not started: copy-route`,
        );
        expect((await programsAsked(folder,)).some(function ranTheLauncher(line,) {
          return line.startsWith('systemd-run',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses to start a run while the volume that holds Pi\'s session files is under its own floor, which is the drive that filled on 2026-11-12',
      fn: async () => {
        await using folder = await makeBareFolder({
          sessionFilesVolumeRoom: roomUnderTheFloorBytes(SESSION_FILES_FLOOR_BYTES,),
        },);
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         What the gate answered while the drive holding Pi's session files was under its floor.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=copy-route', `--runs-folder=${folder.runs}`,],
        },);
        expect(ran.status,).toBe(NO_SLOT_EXIT_STATUS,);
        expect(ran.stdout.trim(),).toBe(
          `NO SLOT: ${freeSpaceUnderTheFloorTextOf({
            space: {
              volume: TEST_SESSION_FILES_VOLUME,
              freeBytes: Number(roomUnderTheFloorBytes(SESSION_FILES_FLOOR_BYTES,),),
            },
            floorBytes: SESSION_FILES_FLOOR_BYTES,
          },)}; not started: copy-route`,
        );
      },
    },),
    it({
      name: 'names the volume holding the runs folder once when the runs folder is reached through a link onto the build volume, which is this machine since 2026-10-11',
      fn: async () => {
        await using folder = await makeBareFolder({ runsFolderVolume: BUILD_VOLUME, },);
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         What the gate answered a caller that asked only for the counts.
         */
        const ran = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
        },);
        expect(ran.status,).toBe(0,);
        expectCountsLine({
          line: ran.stdout.trim(),
          counts: new Map(),
          folder,
          agentsGoing: 0,
        },);
        /**
         How many times the line names the build volume, which both the repository and the runs folder are on.
         */
        const named = ran.stdout.trim()
          .split(`free on ${BUILD_VOLUME}`,)
          .length - 1;
        expect(named,).toBe(1,);
      },
    },),
    it({
      name: 'counts a launcher that has not made its unit yet against the cap a caller sets, and refuses the next start naming the count and the cap',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         A cap of the test's own, which the environment sets without a code change.
         */
        const cap = 1;
        await writeProcess({
          folder: folder.processes,
          pid: '101',
          words: [
            '/usr/bin/node',
            join(folder.path, LAUNCHER_ENTRY_NAMES[0] ?? '',),
            '--name=already-going',
          ],
        },);
        /**
         What the gate answered for one agent too many.
         */
        const ran = await runTheGate({
          folder,
          words: ['--name=new-run', `--runs-folder=${folder.runs}`,],
          machineCap: String(cap,),
        },);
        expect(ran.status,).toBe(NO_SLOT_EXIT_STATUS,);
        expect(ran.stdout.trim(),).toBe(
          `NO SLOT: ${machineCapRefusalTextOf({
            agentsGoing: cap,
            cap,
          },)}; not started: new-run`,
        );
        expect((await programsAsked(folder,)).some(function ranTheLauncher(line,) {
          return line.startsWith('systemd-run',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a cap or a count of other agents that is no whole number, naming the variable, rather than starting runs on another number',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         What the gate answered a cap that is no whole number.
         */
        const refusedCap = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
          machineCap: 'half',
        },);
        expect(refusedCap.status,).not.toBe(0,);
        expect(refusedCap.stderr,).toContain(MACHINE_CAP_VARIABLE,);
        expect(refusedCap.stderr,).toContain('half',);
        /**
         What the gate answered a count of other agents that is no whole number.
         */
        const refusedCount = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
          otherAgentsGoing: 'two',
        },);
        expect(refusedCount.status,).not.toBe(0,);
        expect(refusedCount.stderr,).toContain(OTHER_AGENTS_GOING_VARIABLE,);
        expect(refusedCount.stderr,).toContain('two',);
      },
    },),
    it({
      name: 'counts the agents a caller says it has going on the counts line, beside the runs the gate found',
      fn: async () => {
        await using folder = await makeBareFolder();
        await writeStandIns({
          folder: folder.bins,
          units: '',
          execStart: '',
        },);
        /**
         How many agents of its own the caller says it has going, which are no Pi run and so in no unit.
         */
        const otherAgentsGoing = 2;
        /**
         What the gate answered.
         */
        const ran = await runTheGate({
          folder,
          words: ['--counts-only', `--runs-folder=${folder.runs}`,],
          otherAgentsGoing: String(otherAgentsGoing,),
        },);
        expect(ran.status,).toBe(0,);
        expectCountsLine({
          line: ran.stdout.trim(),
          counts: new Map(),
          folder,
          agentsGoing: otherAgentsGoing,
        },);
      },
    },),
  ],
},);

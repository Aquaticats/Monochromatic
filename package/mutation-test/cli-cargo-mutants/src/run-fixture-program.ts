import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { CargoMutantsToolingError } from './error.ts';
import {
  argumentSeparator,
  callTerminator,
} from './fake-programs.ts';
import type { FakePodman } from './fake-podman-fixture.ts';
import {
  killGroup,
  unwatchGroup,
  watchGroup,
} from './fixture-process-group.ts';
import {
  mentionsProcessLimit,
  processLimitMessage,
} from './process-limits.ts';

/**
 Longest a program in a fixture may run before it is killed with everything it started. A program that runs this long is
 hung, and a test that waits for it can only time out without saying why; the default test timeout is longer, so this one
 speaks first.
 */
export const fixtureProgramCeilingMilliseconds = 45_000;

/**
 What a program under test did.
 */
export type ScriptRun = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
  readonly calls: readonly (readonly string[])[];
  readonly containerfiles: readonly string[];
  readonly evidenceFiles: readonly (readonly [
    string,
    string
  ])[];
};

/**
 Read every file under a directory.

 @param root - directory to read
 
 @param relative - path below the root that is being read
 
 @returns pairs of relative path and text, in path order
 
 @example
 ```ts
 await readTree({ root: '/evidence', relative: '' });
 ```
 */
async function readTree({
  root,
  relative,
}: {
  readonly root: string;
  readonly relative: string;
}): Promise<readonly (readonly [
  string,
  string
])[]> {
  /**
   Entries of the directory.
   */
  const entries = await readdir(
    join(
      root,
      relative,
    ),
    { withFileTypes: true },
  );
  /**
   Pairs of each entry, one for a file and the subtree for a directory.
   */
  const pairs = await Promise.all(entries.map(async function readEntry(entry) {
    /**
     Path of the entry below the root.
     */
    const path = join(
      relative,
      entry.name,
    );
    return entry.isDirectory()
      ? readTree({
        root,
        relative: path,
      })
      : [[
        path,
        await readFile(
          join(
            root,
            path,
          ),
          'utf8',
        ),
      ] as const];
  }));
  return pairs.flat()
    .toSorted(function byPath(
      [first],
      [second]
    ) {
      return first.localeCompare(second);
    });
}

/**
 Split the recorded calls into argument lists.

 @param log - text the stand-ins appended to, or empty when nothing was called
 
 @returns one list per call
 
 @example
 ```ts
 splitCalls('a\u0001b\u0002');
 ```
 */
function splitCalls(log: string): readonly (readonly string[])[] {
  return log.split(callTerminator)
    .filter(function isCall(record) {
      return record !== '';
    })
    .map(function splitArguments(record) {
      return record.split(argumentSeparator);
    });
}

/**
 Wait for a program to end, and end everything it started with it.

 @param child - the running program, which leads a process group of its own

 @param ceiling - aborts when the program has run too long

 @param program - executable that was run, for the message

 @param programArguments - its arguments, for the message

 @param ceilingMilliseconds - the ceiling's length, for the message

 @returns the arguments of the close event

 @throws CargoMutantsToolingError when the program ran past the ceiling or could not start a process or a thread

 @example
 ```ts
 await closedWith({ child, ceiling: AbortSignal.timeout(1000), program: 'node', programArguments: [], ceilingMilliseconds: 1000 });
 ```
 */
async function closedWith({
  child,
  ceiling,
  program,
  programArguments,
  ceilingMilliseconds,
}: {
  readonly child: ReturnType<typeof spawn>;
  readonly ceiling: AbortSignal;
  readonly program: string;
  readonly programArguments: readonly string[];
  readonly ceilingMilliseconds: number;
}): Promise<readonly unknown[]> {
  /**
   Group of the program, which is its process ID because it leads the group.
   */
  const group = child.pid;
  if (group !== undefined)
    watchGroup(group);
  /**
   Ends the group: whatever the program left running dies with it, so a test does not outlive its program's helpers.
   */
  function endGroup(): void {
    if (group === undefined)
      return;
    killGroup(group);
    unwatchGroup(group);
  }
  // The leader ending ends the group at once: a helper it left running keeps the output pipes open, and the close event
  // waits for them.
  child.once(
    'exit',
    endGroup,
  );
  try {
    /**
     Arguments of the close event.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    endGroup();
    if (ceiling.aborted)
      throw new CargoMutantsToolingError(`${program} ${programArguments.join(' ')} ran for more than ${String(ceilingMilliseconds)} ms and was killed with everything it started.`);
    return closed;
  }
  catch (error) {
    endGroup();
    if (error instanceof CargoMutantsToolingError)
      throw error;
    if (ceiling.aborted)
      throw new CargoMutantsToolingError(`${program} ${programArguments.join(' ')} ran for more than ${String(ceilingMilliseconds)} ms and was killed with everything it started.`);
    if (mentionsProcessLimit(String(error)))
      throw new CargoMutantsToolingError(await processLimitMessage(program));
    throw error;
  }
}

/**
 Run a program with the stand-in programs first on `PATH` and read back everything it left.

 @param fake - fixture
 
 @param program - executable to run
 
 @param programArguments - its arguments: a script path and its arguments, an inline-source option and the source, or a shell option and command
 
 @param workingDirectory - directory to run in; the fixture's clean directory when absent
 
 @param extraEnvironment - environment added to the fixture's
 
 @param ceilingMilliseconds - how long the program may run before it is killed with everything it started; the default ceiling when absent
 
 @returns exit status, output, the recorded calls, the Containerfiles built, and the evidence files written
 
 @example
 ```ts
 await runInFixture({ fake, program: 'node', programArguments: ['/p/bin/mutate.ts'], extraEnvironment: {} });
 ```
 */
export async function runInFixture({
  fake,
  program,
  programArguments,
  workingDirectory,
  extraEnvironment,
  ceilingMilliseconds = fixtureProgramCeilingMilliseconds,
}: {
  readonly fake: FakePodman;
  readonly program: string;
  readonly programArguments: readonly string[];
  readonly workingDirectory?: string;
  readonly extraEnvironment: Readonly<Record<string, string>>;
  readonly ceilingMilliseconds?: number;
}): Promise<ScriptRun> {
  /**
   Aborts when the program has run too long.
   */
  const ceiling = AbortSignal.timeout(ceilingMilliseconds);
  /**
   The running program.
   */
  const child = spawn(
    program,
    [...programArguments],
    {
      cwd: workingDirectory ?? fake.workingDirectory,
      env: {
        ...fake.environment,
        ...extraEnvironment,
      },
      // A group of its own, so that it can be killed as a whole and is listed with the watcher that kills it if this process dies.
      detached: true,
      // Ends the program itself at the ceiling; the rest of its group is killed with it below.
      signal: ceiling,
      killSignal: 'SIGKILL',
      stdio: [
        'ignore',
        'pipe',
        'pipe'
      ],
    },
  );
  /**
   Group of the program, which is its process ID because it leads the group.
   */
  const group = child.pid;
  if (group !== undefined)
    watchGroup(group);
  /**
   Standard output chunks as they arrive.
   */
  const outChunks: Buffer[] = [];
  /**
   Standard error chunks as they arrive.
   */
  const errorChunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collectOut(chunk: Buffer) {
      outChunks.push(chunk);
    },
    );
  child.stderr
    .on(
    'data',
    function collectError(chunk: Buffer) {
      errorChunks.push(chunk);
    },
    );
  /**
   Arguments of the close event.
   */
  const closed = await closedWith({
    child,
    ceiling,
    program,
    programArguments,
    ceilingMilliseconds,
  });
  /**
   Names in the fixture directory, which show whether the stand-ins were ever called.
   */
  const names = await readdir(fake.directory);
  /**
   Names of the Containerfiles the stand-in copied, in build order.
   */
  const containerfileNames = names
    .filter(function isContainerfile(name) {
      return name.startsWith('containerfile.');
    })
    .toSorted(function byNumber(
      first,
      second
    ) {
      return Number(first.split('.')[1]) - Number(second.split('.')[1]);
    });
  /**
   Exit code, which is not a number when a signal ended the program.
   */
  const [code] = closed;
  /**
   What the program wrote to standard error.
   */
  const stderr = Buffer.concat(errorChunks)
    .toString('utf8');
  if ((code !== 0) && mentionsProcessLimit(stderr))
    throw new CargoMutantsToolingError(`${await processLimitMessage(program)} The program wrote: ${stderr.trim()}`);
  return {
    status: (typeof code) === 'number' ? code : 1,
    stdout: Buffer.concat(outChunks)
      .toString('utf8'),
    stderr,
    calls: names.includes('calls.log')
      ? splitCalls(await readFile(
        join(
          fake.directory,
          'calls.log',
        ),
        'utf8',
      ))
      : [],
    containerfiles: await Promise.all(containerfileNames.map(function readContainerfile(name) {
      return readFile(
        join(
          fake.directory,
          name,
        ),
        'utf8',
      );
    })),
    evidenceFiles: (await readdir(fake.workingDirectory)).includes('target')
      ? await readTree({
        root: join(
          fake.workingDirectory,
          'target',
        ),
        relative: '',
      })
      : [],
  };
}

/**
 Run a Node script.

 @param fake - fixture
 
 @param script - absolute path of the script
 
 @param args - arguments after the script path
 
 @param extraEnvironment - environment added to the fixture's
 
 @returns what the script did
 
 @example
 ```ts
 await runScript({ fake, script: '/p/bin/mutate.ts', args: [], extraEnvironment: {} });
 ```
 */
export function runScript({
  fake,
  script,
  args,
  extraEnvironment,
}: {
  readonly fake: FakePodman;
  readonly script: string;
  readonly args: readonly string[];
  readonly extraEnvironment: Readonly<Record<string, string>>;
}): Promise<ScriptRun> {
  return runInFixture({
    fake,
    program: 'node',
    programArguments: [
      script,
      ...args
    ],
    extraEnvironment,
  });
}

/**
 Run TypeScript source the way a mise task with `shell = "node --input-type=module-typescript -e"` does.

 @param fake - fixture
 
 @param source - the task's `run` text
 
 @param extraEnvironment - environment added to the fixture's
 
 @returns what the source did
 
 @example
 ```ts
 await runInlineSource({ fake, source: 'console.log(1)', extraEnvironment: {} });
 ```
 */
export function runInlineSource({
  fake,
  source,
  extraEnvironment,
}: {
  readonly fake: FakePodman;
  readonly source: string;
  readonly extraEnvironment: Readonly<Record<string, string>>;
}): Promise<ScriptRun> {
  return runInFixture({
    fake,
    program: 'node',
    programArguments: [
      '--input-type=module-typescript',
      '-e',
      source
    ],
    extraEnvironment,
  });
}

/**
 Run a shell command the way a mise task with a plain `run` string does, from a given directory.

 @param fake - fixture
 
 @param command - the task's `run` text
 
 @param workingDirectory - directory the task runs in
 
 @param extraEnvironment - environment added to the fixture's

 @param ceilingMilliseconds - how long the command may run before it is killed with everything it started; the default ceiling when absent
  
 @returns what the command did
 
 @example
 ```ts
 await runShellCommand({ fake, command: 'cargo --version', workingDirectory: '/p', extraEnvironment: {} });
 ```
 */
export function runShellCommand({
  fake,
  command,
  workingDirectory,
  extraEnvironment,
  ceilingMilliseconds,
}: {
  readonly fake: FakePodman;
  readonly command: string;
  readonly workingDirectory: string;
  readonly extraEnvironment: Readonly<Record<string, string>>;
  readonly ceilingMilliseconds?: number;
}): Promise<ScriptRun> {
  return runInFixture({
    fake,
    program: 'sh',
    programArguments: [
      '-c',
      command
    ],
    workingDirectory,
    extraEnvironment,
    ...(ceilingMilliseconds === undefined ? {} : { ceilingMilliseconds }),
  });
}

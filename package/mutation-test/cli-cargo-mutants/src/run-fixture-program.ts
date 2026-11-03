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
import { processLimitMessage } from './process-limits.ts';

/**
 Longest a program in a fixture may run before it is killed with everything it started. A program that runs this long is
 hung, and a test that waits for it can only time out without saying why; the default test timeout is longer, so this one
 speaks first.
 */
export const fixtureProgramCeilingMilliseconds = 45_000;

/**
 Words in the output of a program that failed to start a process or a thread: Node's failed `spawn`, its failed thread
 creation, and the C library's reason.
 */
const processLimitSignature = /EAGAIN|uv_thread_create|pthread_create|Resource temporarily unavailable/v;

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
 Run a program with the stand-in programs first on `PATH` and read back everything it left.

 @param fake - fixture
 
 @param program - executable to run
 
 @param programArguments - its arguments: a script path and its arguments, an inline-source option and the source, or a shell option and command
 
 @param workingDirectory - directory to run in; the fixture's clean directory when absent
 
 @param extraEnvironment - environment added to the fixture's
 
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
}: {
  readonly fake: FakePodman;
  readonly program: string;
  readonly programArguments: readonly string[];
  readonly workingDirectory?: string;
  readonly extraEnvironment: Readonly<Record<string, string>>;
}): Promise<ScriptRun> {
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
   Set when the ceiling ended the program.
   */
  let timedOut = false;
  /**
   Ends the program with everything it started when it runs past the ceiling.
   */
  const ceiling = setTimeout(
    function endHungProgram() {
      timedOut = true;
      if (group !== undefined)
        killGroup(group);
    },
    fixtureProgramCeilingMilliseconds,
  );
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
  const closed: readonly unknown[] = await once(
    child,
    'close',
  )
    .catch(function explainStart(error: unknown): never {
      if (String(error)
        .match(processLimitSignature) !== null)
        throw new CargoMutantsToolingError(processLimitMessage(program));
      throw error;
    });
  clearTimeout(ceiling);
  if (group !== undefined) {
    // Whatever the program left running dies with it: a test does not outlive its program's helpers.
    killGroup(group);
    unwatchGroup(group);
  }
  if (timedOut)
    throw new CargoMutantsToolingError(`${program} ${programArguments.join(' ')} ran for more than ${String(fixtureProgramCeilingMilliseconds)} ms and was killed with everything it started.`);
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
  if (((typeof code) !== 'number' || code !== 0) && (stderr.match(processLimitSignature) !== null))
    throw new CargoMutantsToolingError(`${processLimitMessage(program)} The program wrote: ${stderr.trim()}`);
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
}: {
  readonly fake: FakePodman;
  readonly command: string;
  readonly workingDirectory: string;
  readonly extraEnvironment: Readonly<Record<string, string>>;
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
  });
}

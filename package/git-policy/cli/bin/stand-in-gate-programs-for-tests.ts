/**
 Stand-in `systemd-run` for the gate commands' tests, so a test never starts a unit of this machine.

 The stand-in is a program under a fixture directory that goes first on `PATH`, writes down what it was asked, and answers
 the way the real program would have: it names the unit it would have started, or fails with the message systemd prints
 for a refused start.
 */
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { NativeGateError } from './native-gate-error.ts';
import { isMissingPathError } from './native-gate-paths.ts';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Scratch parent of every fixture directory; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Environment variable that names the fixture directory the stand-in writes to.
 */
export const standInDirectoryVariable = 'GATE_STAND_IN_DIR';

/**
 Environment variable that, when it is `1`, makes the stand-in `systemd-run` refuse the start.
 */
export const standInFailureVariable = 'GATE_STAND_IN_FAIL';

/**
 The stand-in `systemd-run`: records its arguments as one JSON array per line, then fails when the fixture asked for a
 refused start and names the unit it would have started otherwise.
 */
const standInSystemdRunProgram = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.${standInDirectoryVariable};
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'systemd-run.jsonl'), JSON.stringify(args) + '\\n');
if (process.env.${standInFailureVariable} === '1') {
  process.stderr.write('Failed to start transient service unit: the stand-in was asked to refuse this start\\n');
  process.exit(1);
}
const unit = (args.find((argument) => argument.startsWith('--unit=')) ?? '--unit=unknown').slice('--unit='.length);
process.stdout.write('Running as unit: ' + unit + '.service\\n');
`;

/**
 Fixture holding the stand-in program and the environment that puts it first on `PATH`.
 */
export type GateStandIn = AsyncDisposable & {
  /**
   The fixture's own directory, which the stand-in writes its calls into.
   */
  readonly directory: string;
  /**
   The environment a gate command runs with when it should use the stand-in.
   */
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Create the stand-in program of one fixture.

 @returns the fixture; declare it with `await using`

 @example
 ```ts
 await using standIn = await createGateStandIn();
 ```
 */
export async function createGateStandIn(): Promise<GateStandIn> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   Fresh directory for this fixture.
   */
  const directory = await mkdtemp(join(
    scratchParent,
    'git-policy-gate-',
  ));
  /**
   Directory that goes first on `PATH` and holds the stand-in program.
   */
  const programs = join(
    directory,
    'path',
  );
  await mkdir(
    programs,
    { recursive: true, },
  );
  await writeFile(
    join(
      programs,
      'systemd-run',
    ),
    standInSystemdRunProgram,
    { mode: ownerOnlyMode, },
  );
  return {
    directory,
    environment: {
      ...process.env,
      PATH: `${programs}:${process.env
        .PATH
        ?? ''}`,
      [standInDirectoryVariable]: directory,
    } as Readonly<Record<string, string>>,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Tells whether one recorded line of the stand-in `systemd-run` is the argument array it writes.

 @param value - the parsed line

 @returns true for an array whose entries are all text

 @example
 ```ts
 isArgumentArray(['--user', '--unit=u']);
 ```
 */
function isArgumentArray(value: unknown): value is readonly string[] {
  return Array.isArray(value) && value.every(function isText(entry: unknown) {
    return (typeof entry) === 'string';
  });
}

/**
 Reads one recorded file, answering with empty text when the stand-in never wrote it.

 @param path - the file to read

 @returns its text, or empty text when it is not there

 @throws Error when the file is there and cannot be read, which is no fixture this test made

 @example
 ```ts
 await readRecordedLines({ path: join(standIn.directory, 'systemd-run.jsonl') });
 ```
 */
async function readRecordedLines({ path }: { readonly path: string; }): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isMissingPathError({ error, }))
      return '';
    throw error;
  }
}

/**
 Read what the stand-in `systemd-run` was asked, one entry per start.

 @param directory - the fixture's directory

 @returns the argument arrays, in the order the starts were tried

 @throws NativeGateError when a recorded line is not an argument array

 @example
 ```ts
 const [first] = await standInStarts({ directory: standIn.directory });
 ```
 */
export async function standInStarts({ directory }: { readonly directory: string; }): Promise<readonly (readonly string[])[]> {
  /**
   The recorded lines, empty when no start was ever tried.
   */
  const text = await readRecordedLines({ path: join(
    directory,
    'systemd-run.jsonl'
  ), });
  return text.split('\n')
    .filter(function hasText(line) {
      return line !== '';
    })
    .map(function parsed(line) {
      /**
       The recorded line, parsed.
       */
      const value: unknown = JSON.parse(line);
      if (!isArgumentArray(value))
        throw new NativeGateError(`the stand-in systemd-run recorded a line that is no argument array: ${line}`);
      return value;
    });
}

/**
 A directory one test owns, which is removed when the test's scope ends.

 @param prefix - the name every test directory of this fixture starts with, so a leftover is recognisable

 @returns the directory and the removal that ends it, for `await using`

 @example
 ```ts
 await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-main-tip-' });
 ```
 */
export async function createGateScratchDirectory({ prefix }: { readonly prefix: string; }): Promise<AsyncDisposable & { readonly path: string; }> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   The directory this test owns.
   */
  const path = await mkdtemp(join(
    scratchParent,
    prefix,
  ));
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

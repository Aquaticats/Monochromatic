/**
 Stand-in `systemd-run` and `systemctl` for the check runner's tests, so a test never starts or stops a unit or a scope of
 this machine.

 The stand-in `systemd-run` does what `systemd-run --scope` does to the command it is given: it runs it in its own process
 and answers with its status, so a test exercises the runner's real subprocess, its output and its exit status. It records
 every argument array it was given, so a test reads the caps and the unit name a scope was started with. The stand-in
 `systemctl` records the stop it was asked for and answers as a manager whose scope is gone.
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

import { isMissingPathError } from './native-gate-paths.ts';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Scratch parent of every fixture directory; never the system temporary directory (rule `ATH`).
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Environment variable that names the fixture directory the stand-ins write their calls to.
 */
export const scopeStandInDirectoryVariable = 'CHECK_STAND_IN_DIR';

/**
 Environment variable that, when it is `1`, makes the stand-in `systemd-run` refuse the start.
 */
export const scopeStandInFailureVariable = 'CHECK_STAND_IN_SCOPE_FAIL';

/**
 Environment variable that, when it is `1`, makes the stand-in `systemctl` refuse the stop.
 */
export const scopeStandInStopFailureVariable = 'CHECK_STAND_IN_STOP_FAIL';

/**
 The stand-in `systemd-run`: records its arguments, then runs the command after `--` with the caller's streams and answers
 with its status.
 */
const standInSystemdRunProgram = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const directory = process.env.${scopeStandInDirectoryVariable};
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'systemd-run.jsonl'), JSON.stringify(args) + '\\n');
if (process.env.${scopeStandInFailureVariable} === '1') {
  process.stderr.write('Failed to start transient scope unit: the stand-in was asked to refuse this start\\n');
  process.exit(1);
}
const separator = args.indexOf('--');
if (separator === -1) {
  process.stderr.write('the stand-in systemd-run was given no command after --\\n');
  process.exit(2);
}
const [program, ...rest] = args.slice(separator + 1);
const done = spawnSync(program, rest, { stdio: 'inherit' });
process.exit(done.status === null ? 1 : done.status);
`;

/**
 The stand-in `systemctl`: records the stop it was asked for and answers as a manager whose scope is over.
 */
const standInSystemctlProgram = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.${scopeStandInDirectoryVariable};
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'systemctl.jsonl'), JSON.stringify(args) + '\\n');
if (process.env.${scopeStandInStopFailureVariable} === '1') {
  process.stderr.write('Failed to stop unit: the stand-in was asked to refuse this stop\\n');
  process.exit(1);
}
`;

/**
 A fixture holding the stand-in programs and the environment that puts them first on `PATH`.
 */
export type ScopeStandIn = AsyncDisposable & {
  /**
   The fixture's own directory, which the stand-ins write their calls into.
   */
  readonly directory: string;
  /**
   The environment a check runs with when it should use the stand-ins.
   */
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Creates the stand-in programs of one fixture.

 @returns the fixture; declare it with `await using`

 @example
 ```ts
 await using standIn = await createScopeStandIn();
 ```
 */
export async function createScopeStandIn(): Promise<ScopeStandIn> {
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
    'git-policy-check-',
  ));
  /**
   Directory that goes first on `PATH` and holds the stand-in programs.
   */
  const programs = join(
    directory,
    'path',
  );
  await mkdir(
    programs,
    { recursive: true, },
  );
  await Promise.all([
    writeFile(
      join(
        programs,
        'systemd-run',
      ),
      standInSystemdRunProgram,
      { mode: ownerOnlyMode, },
    ),
    writeFile(
      join(
        programs,
        'systemctl',
      ),
      standInSystemctlProgram,
      { mode: ownerOnlyMode, },
    ),
  ]);
  return {
    directory,
    environment: {
      ...process.env,
      PATH: `${programs}:${process.env
        .PATH
        ?? ''}`,
      [scopeStandInDirectoryVariable]: directory,
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
 Tells whether one recorded line is an argument array.

 @param value - the parsed line

 @returns true for an array whose entries are all text

 @example
 ```ts
 isArgumentArray(['--user', '--scope']);
 ```
 */
function isArgumentArray(value: unknown): value is readonly string[] {
  return Array.isArray(value) && value.every(function isText(entry: unknown) {
    return (typeof entry) === 'string';
  });
}

/**
 Reads every argument array one stand-in recorded.

 @param directory - the fixture's directory

 @param fileName - the file the stand-in wrote

 @returns the recorded arrays, in the order the calls were made

 @throws Error when a recorded line is not an argument array, which is no fixture these tests make

 @example
 ```ts
 const [first] = await recordedCalls({ directory: standIn.directory, fileName: 'systemd-run.jsonl' });
 ```
 */
export async function recordedCalls({
  directory,
  fileName,
}: {
  readonly directory: string;
  readonly fileName: string;
}): Promise<readonly (readonly string[])[]> {
  /**
   The recorded text, empty when the stand-in was never called.
   */
  let text = '';
  try {
    text = await readFile(
      join(
        directory,
        fileName
      ),
      'utf8'
    );
  }
  catch (error) {
    if (!isMissingPathError({ error, }))
      throw error;
  }
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
        throw new Error(`the stand-in recorded a line that is no argument array: ${line}`);
      return value;
    });
}

/**
 Reads the scope name one stand-in `systemd-run` was asked for.

 @param directory - the fixture's directory

 @returns the unit argument of the first start

 @throws Error when no start was recorded, so a test never reads an empty name as a pass

 @example
 ```ts
 await startedScope({ directory: standIn.directory });
 ```
 */
export async function startedScope({ directory }: { readonly directory: string; }): Promise<string> {
  /**
   The unit argument of the first start.
   */
  const unitArgument = (await recordedCalls({
    directory,
    fileName: 'systemd-run.jsonl',
  })).at(0)
    ?.find(function isUnit(argument) {
      return argument.startsWith('--unit=');
    });
  if (unitArgument === undefined)
    throw new Error('the stand-in systemd-run was never asked to start a scope');
  return unitArgument.slice('--unit='.length);
}

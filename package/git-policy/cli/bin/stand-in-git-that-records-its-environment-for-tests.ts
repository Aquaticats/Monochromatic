/**
 A stand-in `git` that records the environment it was given and then runs the standing Git, so a test reads which session
 place reached a fixture's `git` and which runtime directory it was handed.

 The stand-in is a stand-in written for this package's tests: it says what it was given and no more, and no claim about a
 released wrapper's behaviour rests on it. It forwards to `/usr/bin/git`, so a fixture that puts it first on its `PATH`
 still makes the commits it asked for, and nothing it does can start a unit of this account's manager.
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
 Scratch parent of every fixture directory; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 The stand-in `git`: it appends one JSON record per call, naming its arguments, its directory, whether
 `DBUS_SESSION_BUS_ADDRESS` reached it, which `XDG_RUNTIME_DIR` it was given and the `PATH` it was found on, then hands
 the same arguments to the standing Git and exits with its status.

 A variable that did not reach the stand-in is left out of the record rather than written as nothing, so a reader tells a
 name no child got from a name a child got empty.
 */
const recordingGitProgram = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const log = path.join(__dirname, '..', 'git.jsonl');
fs.appendFileSync(log, JSON.stringify({
  args: process.argv.slice(2),
  cwd: process.cwd(),
  bus: process.env.DBUS_SESSION_BUS_ADDRESS,
  runtime: process.env.XDG_RUNTIME_DIR,
  searchPath: process.env.PATH,
}) + '\\n');
const done = spawnSync('/usr/bin/git', process.argv.slice(2), { stdio: 'inherit' });
process.exit(done.status === null ? 1 : done.status);
`;

/**
 One `git` call the stand-in recorded.
 */
export type RecordedGitCall = {
  /**
   The arguments after the program name.
   */
  readonly args: readonly string[];
  /**
   The directory the call ran in.
   */
  readonly cwd: string;
  /**
   `DBUS_SESSION_BUS_ADDRESS` as the stand-in found it, absent when no bus address reached it.
   */
  readonly bus?: string;
  /**
   `XDG_RUNTIME_DIR` as the stand-in found it, absent when no runtime directory was given.
   */
  readonly runtime?: string;
  /**
   The `PATH` the stand-in was found on.
   */
  readonly searchPath: string;
};

/**
 Fixture holding the stand-in program, the record it writes, and the directory to put first on a fixture's search path.
 */
export type RecordingGit = AsyncDisposable & {
  /**
   The fixture's own directory.
   */
  readonly directory: string;
  /**
   The directory holding the stand-in, for a fixture's leading search directories.
   */
  readonly searchDirectory: string;
  /**
   The file the stand-in appends its records to, beside the directory it was written into.
   */
  readonly log: string;
};

/**
 Create the stand-in `git` of one fixture.

 @returns the fixture; declare it with `await using`

 @example
 ```ts
 await using recording = await createRecordingGit();
 ```
 */
export async function createRecordingGit(): Promise<RecordingGit> {
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
    'git-policy-recording-git-',
  ));
  /**
   Directory that holds the stand-in and goes first on a fixture's search path.
   */
  const searchDirectory = join(
    directory,
    'path',
  );
  /**
   The file the stand-in records into.
   */
  const log = join(
    directory,
    'git.jsonl',
  );
  await mkdir(
    searchDirectory,
    { recursive: true, },
  );
  await writeFile(
    join(
      searchDirectory,
      'git',
    ),
    recordingGitProgram,
    { mode: ownerOnlyMode, },
  );
  return {
    directory,
    searchDirectory,
    log,
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
 Tells whether one parsed line carries the fields the stand-in writes.

 The fields are read by name, so a line the stand-in did not write is refused rather than read as a call it never made.

 @param value - the parsed line

 @returns true for a line with the stand-in's fields

 @example
 ```ts
 isRecordedCall({ args: [], cwd: '/w', searchPath: '/usr/bin:/bin' });
 ```
 */
function isRecordedCall(value: unknown): value is RecordedGitCall {
  if (((typeof value) !== 'object') || (value === null))
    return false;
  /**
   The line's fields, by name, so no field's type is read before it is checked.
   */
  const fields: ReadonlyMap<string, unknown> = new Map(Object.entries(value));
  /**
   The arguments the call was made with.
   */
  const args = fields.get('args');
  if (!Array.isArray(args))
    return false;
  if (!args.every(function isText(argument: unknown) {
    return (typeof argument) === 'string';
  }))
    return false;
  if ((typeof fields.get('cwd')) !== 'string')
    return false;
  if ((typeof fields.get('searchPath')) !== 'string')
    return false;
  if (!isAbsentOrText({ field: fields.get('bus'), }))
    return false;
  return isAbsentOrText({ field: fields.get('runtime'), });
}

/**
 Tells whether one field is text or was left out of the record.

 @param field - the field's value, which is nothing at all when the record leaves it out

 @returns whether the field is text or absent

 @example
 ```ts
 isAbsentOrText({ field: '/run/user/1000' });
 ```
 */
function isAbsentOrText({ field }: { readonly field: unknown; }): boolean {
  if (field === undefined)
    return true;
  return (typeof field) === 'string';
}

/**
 Read one fixture's record file, answering with empty text when the stand-in never wrote it.

 @param log - the file the fixture's stand-in records into

 @returns its text, or empty text when it is not there

 @throws Error when the file is there and cannot be read, which is no fixture this package made

 @example
 ```ts
 await readRecordedText({ log: recording.log });
 ```
 */
async function readRecordedText({ log }: { readonly log: string; }): Promise<string> {
  try {
    return await readFile(
      log,
      'utf8',
    );
  }
  catch (error) {
    if (!isMissingPathError({ error, }))
      throw error;
    return '';
  }
}

/**
 Read what the stand-in `git` was given, one record per call.

 @param log - the file the fixture's stand-in records into

 @returns the records, in the order the calls were made, empty when the stand-in was never called

 @throws Error when a recorded line is not the record the stand-in writes, which is no fixture this package made

 @example
 ```ts
 const calls = await recordedGitCalls({ log: recording.log });
 ```
 */
export async function recordedGitCalls({ log }: { readonly log: string; }): Promise<readonly RecordedGitCall[]> {
  return (await readRecordedText({ log, })).split('\n')
    .filter(function hasText(line) {
      return line !== '';
    })
    .map(function parsed(line) {
      /**
       The recorded line, parsed.
       */
      const value: unknown = JSON.parse(line);
      if (!isRecordedCall(value))
        throw new Error(`the recording git wrote a line that is no record of a call: ${line}`);
      return value;
    });
}

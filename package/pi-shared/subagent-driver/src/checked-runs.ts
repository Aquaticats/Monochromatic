/**
 What the watch has already reported, so a run that is past a limit is reported once and then left alone until its recheck
 interval passes. It outlives one process, because a coordinator runs the watch again on its next turn and must not be told
 about the same run every time.

 @module
 */

import {
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';
import {
  env,
  pid,
} from 'node:process';
import { errorCodeOf, } from './error-code-of.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The code the operating system gave a missing path.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The name of the folder the watch keeps its state under, inside the user's state folder.
 */
const STATE_FOLDER_NAME = 'pi-overdue-watch';

/**
 The name of the file holding what was reported.
 */
const STATE_FILE_NAME = 'reported-runs.json';

/**
 When each reported run was reported, by the key the watch reports it under.
 */
export type CheckedRuns = Readonly<Record<string, number>>;

/**
 Reads the folder a user's applications keep their state in.

 @returns the state folder named by the user's settings, or the one a user's home folder holds by default

 @example
 ```ts
 const stateHome = stateHomeOf();
 ```
 */
function stateHomeOf(): string {
  /**
   The state folder as the user's settings name it, which an empty value leaves unset.
   */
  const named = env.XDG_STATE_HOME;
  return ((named === undefined) || (named === '')) ? join(
    homedir(),
    '.local',
    'state',
  ) : named;
}

/**
 Where the watch keeps what it has reported, when the caller names no file: the user's state folder, never the runs folder,
 which holds runs and nothing else.

 @returns the state file's path

 @example
 ```ts
 const file = defaultCheckedRunsFile();
 ```
 */
export function defaultCheckedRunsFile(): string {
  return join(
    stateHomeOf(),
    STATE_FOLDER_NAME,
    STATE_FILE_NAME,
  );
}

/**
 Reads what was reported before.

 @param file - the state file

 @returns when each key was reported, and nothing at all when the file is missing or holds no numbers

 @example
 ```ts
 const checked = await readCheckedRuns({ file: '/state/reported-runs.json', },);
 ```
 */
export async function readCheckedRuns({
  file,
}: {
  readonly file: string;
},): Promise<CheckedRuns> {
  try {
    /**
     The file as it was written.
     */
    const text = await readFile(
      file,
      { encoding: 'utf8', },
    );
    /**
     What it holds, which a watch that was killed mid-write may have left unreadable.
     */
    const parsed: unknown = JSON.parse(text,);
    if (!isRecord(parsed,)) {
      return {};
    }
    /**
     The times the file held, with anything else it held left out: a watch that was killed mid-write leaves what it had so
     far, and a file someone edited by hand may hold anything.
     */
    const times: Record<string, number> = {};
    for (const [key, value,] of Object.entries(parsed,)) {
      if ((typeof value) === 'number') {
        times[key] = value;
      }
    }
    return times;
  }
  catch (error) {
    // A state file the watch never wrote, or one it could not finish writing, costs one repeated report and nothing else.
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (error instanceof SyntaxError)) {
      return {};
    }
    throw error;
  }
}

/**
 Writes what was reported, whole or not at all.

 @param file - the state file

 @param checked - when each key was reported

 @example
 ```ts
 await writeCheckedRuns({ file: '/state/reported-runs.json', checked: { 'pi-a': 1_000, }, },);
 ```
 */
export async function writeCheckedRuns({
  file,
  checked,
}: {
  readonly file: string;
  readonly checked: CheckedRuns;
},): Promise<void> {
  await mkdir(
    dirname(file,),
    { recursive: true, },
  );
  /**
   The file's new text, written beside it so no reader ever sees half of it.
   */
  const candidate = `${file}.${String(pid,)}`;
  await writeFile(
    candidate,
    JSON.stringify(checked,),
    { encoding: 'utf8', },
  );
  await rename(
    candidate,
    file,
  );
}

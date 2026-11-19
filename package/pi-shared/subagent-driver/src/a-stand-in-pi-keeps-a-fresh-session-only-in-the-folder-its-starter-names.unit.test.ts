/**
 Where the stand-in `pi` of the driver's tests keeps the session file it makes for a fresh start: in the file or the temporary
 folder its starter names, and never in its home folder or its working folder.

 Before this check the stand-in fell back to `HOME` and then to the working folder, and the connection tests named neither of the
 others, so every run of them left `stand-in-session-<uuid>.jsonl` files in the real home folder of whoever ran them: 3,246 empty
 ones were counted there on 2026-11-19.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readdir,
  rm,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 Milliseconds a stand-in start may take before the test gives up on it. A start whose input is closed at once leaves in well under
 a second; the bound only matters when the stand-in hangs, so it is generous for a starved host.
 */
const START_BOUND_MILLISECONDS = 60_000;

/**
 The status a process leaves with when it ended as it meant to.
 */
const CLEAN_STATUS = 0;

/**
 What the stand-in names a session file it makes for a fresh start, before and after the unique part.
 */
const SESSION_NAME_START = 'stand-in-session-';
const SESSION_NAME_END = '.jsonl';

/**
 The variables that place the stand-in's fresh session, which its error names when a starter gave neither.
 */
const SESSION_FILE_VARIABLE = 'STAND_IN_PI_SESSION_FILE';
const TEMPORARY_FOLDER_VARIABLE = 'TMPDIR';

type StandInFolders = {
  readonly home: string;
  readonly working: string;
  readonly temporary: string;
  readonly log: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

type StandInEnded = {
  readonly status: number;
  readonly standardError: string;
};

/**
 Makes a home, a working folder and a temporary folder for one start, all inside one scratch folder the test removes.

 @returns the folders, and a disposer that removes them
 */
async function makeFolders(): Promise<StandInFolders> {
  /**
   The one scratch folder that holds the others.
   */
  const root = await mkdtemp(join(scratchRoot(), 'stand-in-session-place-',),);
  const home = join(root, 'home',);
  const working = join(root, 'working',);
  const temporary = join(root, 'temporary',);
  await mkdir(home,);
  await mkdir(working,);
  await mkdir(temporary,);
  return {
    home,
    working,
    temporary,
    log: join(root, 'commands.log',),
    async [Symbol.asyncDispose]() {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Whether a folder entry has the name the stand-in gives a fresh session.

 @param name - entry's name, without its folder
 */
function isStandInSession(name: string,): boolean {
  return name.startsWith(SESSION_NAME_START,) && name.endsWith(SESSION_NAME_END,);
}

/**
 Lists the stand-in sessions directly inside a folder.

 @param folder - folder to read

 @returns names of the sessions there
 */
async function standInSessionsIn(folder: string,): Promise<readonly string[]> {
  return (await readdir(folder,)).filter(isStandInSession,);
}

/**
 Starts the stand-in with only the variables named, in the working folder, closes its input so it leaves, and waits for it.

 @param environment - every variable the stand-in gets, so nothing of the test's own environment places its session

 @param working - folder the stand-in starts in

 @returns how it ended and what it wrote on its standard error
 */
async function startStandIn({
  environment,
  working,
}: {
  readonly environment: Readonly<Record<string, string>>;
  readonly working: string;
},): Promise<StandInEnded> {
  /**
   The running stand-in, whose input is closed at once so a stand-in that started leaves as Pi does at the end of its input.
   */
  const running = execFileAsync(
    process.execPath,
    [STAND_IN,],
    {
      cwd: working,
      env: environment,
      encoding: 'utf8',
      timeout: START_BOUND_MILLISECONDS,
      killSignal: 'SIGKILL',
    },
  );
  running.child.stdin?.end();
  try {
    /**
     What a stand-in that left with status zero wrote.
     */
    const ended = await running;
    return {
      status: CLEAN_STATUS,
      standardError: ended.stderr,
    };
  }
  catch (error) {
    /**
     The failure `execFile` reports for a child that left with another status, which carries the status and the output.
     */
    const failed = error as { readonly code?: unknown; readonly stderr?: unknown };
    if ((typeof failed.code) !== 'number') {
      throw error;
    }
    return {
      status: failed.code,
      standardError: String(failed.stderr,),
    };
  }
}

/**
 The variables every start in this file gets: enough to run, and a home and a log of its own.

 @param folders - folders of this start

 @returns the variables
 */
function ordinaryVariables(folders: StandInFolders,): Readonly<Record<string, string>> {
  return {
    PATH: [dirname(process.execPath,), '/usr/bin',].join(delimiter,),
    HOME: folders.home,
    STAND_IN_PI_SCENARIO: 'normal',
    STAND_IN_PI_LOG: folders.log,
  };
}

await describe({
  name: 'where a stand-in Pi keeps the session of a fresh start',
  children: [
    it({
      name: 'refuses to start, and writes no session in its home or working folder, when its starter names no place for it',
      fn: async () => {
        await using folders = await makeFolders();
        const ended = await startStandIn({
          environment: ordinaryVariables(folders,),
          working: folders.working,
        },);
        expect(await standInSessionsIn(folders.home,),).toEqual([],);
        expect(await standInSessionsIn(folders.working,),).toEqual([],);
        expect(ended.status,).not.toBe(CLEAN_STATUS,);
        expect(ended.standardError,).toContain(SESSION_FILE_VARIABLE,);
        expect(ended.standardError,).toContain(TEMPORARY_FOLDER_VARIABLE,);
      },
    },),
    it({
      name: 'treats an empty temporary folder variable as no place, rather than the working folder',
      fn: async () => {
        await using folders = await makeFolders();
        const ended = await startStandIn({
          environment: {
            ...ordinaryVariables(folders,),
            [TEMPORARY_FOLDER_VARIABLE]: '',
          },
          working: folders.working,
        },);
        expect(await standInSessionsIn(folders.working,),).toEqual([],);
        expect(await standInSessionsIn(folders.home,),).toEqual([],);
        expect(ended.status,).not.toBe(CLEAN_STATUS,);
      },
    },),
    it({
      name: 'keeps the session in the temporary folder its starter names, and nothing in its home or working folder',
      fn: async () => {
        await using folders = await makeFolders();
        const ended = await startStandIn({
          environment: {
            ...ordinaryVariables(folders,),
            [TEMPORARY_FOLDER_VARIABLE]: folders.temporary,
          },
          working: folders.working,
        },);
        expect(ended.status,).toBe(CLEAN_STATUS,);
        expect(await standInSessionsIn(folders.temporary,),).not.toEqual([],);
        expect(await standInSessionsIn(folders.home,),).toEqual([],);
        expect(await standInSessionsIn(folders.working,),).toEqual([],);
      },
    },),
  ],
},);

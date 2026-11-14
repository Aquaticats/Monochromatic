/**
 What a later driver makes of a message a killed driver had claimed: the claim's owner is read from the claim's folder, the owner's
 presence from the process folder, and a message whose driver is gone is kept under its own name and named in the summary.
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type CommandAnswer,
  type DriverOptions,
  INBOX_CLAIMS_FOLDER,
  type RecordOfPiRun,
  recoverClaimsOfGoneDrivers,
  type RunSummary,
  startRecordOfPiRun,
  type WatchRequest,
} from '../dist/final/node/index.mjs';

/**
 The message a claim holds in every test here.
 */
const MESSAGE_TEXT = 'a message a driver claimed before it was killed';

/**
 The name that message was published under.
 */
const MESSAGE_NAME = 'm.txt';

/**
 The process number a claim in these tests names, which no process folder here holds unless a test writes it.
 */
const OWNER_PID = '4242';

/**
 The start time a claim in these tests names.
 */
const OWNER_START = '900';

/**
 The fields a process record holds after its command name, up to the one that names when it began.
 */
const FIELDS_BEFORE_START = 20;

type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function makeFolder(): Promise<DisposableFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-claims-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The options of a run whose inbox the tests read.

 @param runFolder - folder of the run

 @returns the options
 */
function optionsOf(runFolder: string,): DriverOptions {
  return {
    runFolder,
    workingFolder: runFolder,
    promptFile: join(runFolder, 'prompt.md',),
    provider: 'stand-in',
    model: 'stand-in-model',
    piArguments: [],
    wallSeconds: 30,
    idleSeconds: 0,
    turnCap: Number.MAX_SAFE_INTEGER,
    retryCap: 0,
    retryBaseSeconds: 0,
    startPauseSeconds: 0,
  };
}

/**
 A connection a test's watch reads, which no message here should reach.
 */
function quietConnection(): WatchRequest['connection'] {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send(): Promise<CommandAnswer> {
      throw new Error('the watch was not expected to send a message',);
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 A watch over one run folder, whose record is opened here and closed by the caller.

 @param folder - the test's folder

 @param processFolder - the folder that says whether a claim's driver is still there

 @returns the watch and its record
 */
async function watchOver({
  folder,
  processFolder,
}: {
  readonly folder: string;
  readonly processFolder: string
},): Promise<{ readonly watch: WatchRequest; readonly record: RecordOfPiRun }> {
  const inbox = join(
    folder,
    'inbox',
  );
  await mkdir(
    inbox,
    { recursive: true, },
  );
  const record = startRecordOfPiRun({ feedFile: join(
    folder,
    'feed.jsonl',
  ), },);
  return {
    watch: {
      connection: quietConnection(),
      record,
      options: optionsOf(folder,),
      inbox,
      deadline: Date.now() + 60_000,
      processFolder,
    },
    record,
  };
}

/**
 The summary of a record whose run ended on its own terms, after the record is closed.

 @param record - the run's record

 @returns what a reader of the run would read
 */
async function summaryOf(record: RecordOfPiRun,): Promise<RunSummary> {
  await record.close();
  return record.summarize({
    endReason: 'idle',
    piExit: {},
    piHasExited: true,
    sessionStats: null,
  },);
}

/**
 Writes a claim under an owner's folder, as a driver killed after the rename leaves it.

 @param folder - the test's folder

 @param owner - the claim folder's name

 @param name - the message's name

 @param text - the message's text
 */
async function writeClaim({
  folder,
  owner,
  name = MESSAGE_NAME,
  text = MESSAGE_TEXT,
}: {
  readonly folder: string;
  readonly owner: string;
  readonly name?: string;
  readonly text?: string
},): Promise<void> {
  /**
   The owner's folder inside the inbox.
   */
  const ownerFolder = join(
    folder,
    'inbox',
    INBOX_CLAIMS_FOLDER,
    owner,
  );
  await mkdir(
    ownerFolder,
    { recursive: true, },
  );
  await writeFile(
    join(
      ownerFolder,
      name,
    ),
    text,
    { encoding: 'utf8', },
  );
}

/**
 Writes one process's record, as the process folder holds it.

 @param processFolder - the folder that holds one entry per process

 @param pid - the process number

 @param start - the clock ticks the process began at

 @param asFolder - whether to make the record a folder instead of a file, which cannot be read
 */
async function writeProcess({
  processFolder,
  pid,
  start,
  asFolder = false,
}: {
  readonly processFolder: string;
  readonly pid: string;
  readonly start: string;
  readonly asFolder?: boolean
},): Promise<void> {
  /**
   The process's folder.
   */
  const entry = join(
    processFolder,
    pid,
  );
  await mkdir(
    entry,
    { recursive: true, },
  );
  /**
   The fields a record holds after its command name: the process's state, then the values up to the one that names when it began.
   */
  const fields = Array.from(
    { length: FIELDS_BEFORE_START, },
    function field(_unused, at,) {
      if (at === 0) {
        return 'S';
      }
      return (at === (FIELDS_BEFORE_START - 1)) ? start : '0';
    },
  );
  if (asFolder) {
    await mkdir(
      join(
        entry,
        'stat',
      ),
      { recursive: true, },
    );
    return;
  }
  await writeFile(
    join(
      entry,
      'stat',
    ),
    `${pid} (node) ${fields.join(' ',)}\n`,
    { encoding: 'utf8', },
  );
}

/**
 The names directly in a folder, in order.

 @param folder - the folder to list

 @returns its entries
 */
async function entriesOf(folder: string,): Promise<readonly string[]> {
  return (await readdir(folder,)).toSorted();
}

await describe({
  name: recoverClaimsOfGoneDrivers.name,
  children: [
    it({
      name: 'keeps a message whose driver is gone under its own name, and names it once in the summary',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder of a machine that holds no such process, which is a driver that is gone.
         */
        const processFolder = join(folder.path, 'proc',);
        await mkdir(processFolder,);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-${OWNER_START}`,
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        /**
         What the inbox holds now, and what the message was kept as.
         */
        const inbox = join(folder.path, 'inbox',);
        expect(await entriesOf(inbox,),).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
        expect(await readFile(join(inbox, `.orphaned-${MESSAGE_NAME}`,), 'utf8',),).toBe(MESSAGE_TEXT,);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toEqual([MESSAGE_NAME,],);
      },
    },),
    it({
      name: 'keeps a message whose process number the operating system gave to something else',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder that holds the process number under a different start time.
         */
        const processFolder = join(folder.path, 'proc',);
        await writeProcess({
          processFolder,
          pid: OWNER_PID,
          start: '111',
        },);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-${OWNER_START}`,
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        expect(await entriesOf(join(folder.path, 'inbox',),),).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
        expect((await summaryOf(record,)).inboxNotTaken,).toEqual([MESSAGE_NAME,],);
      },
    },),
    it({
      name: 'leaves a message whose driver is still there',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder that holds the claiming driver under the start time its claim names.
         */
        const processFolder = join(folder.path, 'proc',);
        await writeProcess({
          processFolder,
          pid: OWNER_PID,
          start: OWNER_START,
        },);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-${OWNER_START}`,
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        expect(await entriesOf(join(folder.path, 'inbox',),),).toEqual([INBOX_CLAIMS_FOLDER,],);
        expect((await summaryOf(record,)).inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'leaves a message whose driver it cannot judge, because its record cannot be read',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder whose entry for the owner has a record no one can read.
         */
        const processFolder = join(folder.path, 'proc',);
        await writeProcess({
          processFolder,
          pid: OWNER_PID,
          start: OWNER_START,
          asFolder: true,
        },);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-${OWNER_START}`,
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        expect(await entriesOf(join(folder.path, 'inbox',),),).toEqual([INBOX_CLAIMS_FOLDER,],);
        expect((await summaryOf(record,)).inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'keeps a message once, however often the inbox is read',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder of a machine that holds no such process.
         */
        const processFolder = join(folder.path, 'proc',);
        await mkdir(processFolder,);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-${OWNER_START}`,
        },);
        const first = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch: first.watch, },);
        expect((await summaryOf(first.record,)).inboxNotTaken,).toEqual([MESSAGE_NAME,],);
        /**
         What a second driver reads on the same inbox.
         */
        const second = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch: second.watch, },);
        expect((await summaryOf(second.record,)).inboxNotTaken,).toBeUndefined();
        expect(await entriesOf(join(folder.path, 'inbox',),),).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
      },
    },),
    it({
      name: 'takes an empty claim folder of a driver that is gone away, and leaves a slow driver its own',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder that holds the slow owner and not the gone one.
         */
        const processFolder = join(folder.path, 'proc',);
        await writeProcess({
          processFolder,
          pid: OWNER_PID,
          start: OWNER_START,
        },);
        await mkdir(
          join(
            folder.path,
            'inbox',
            INBOX_CLAIMS_FOLDER,
            `${OWNER_PID}-${OWNER_START}`,
          ),
          { recursive: true, },
        );
        await mkdir(
          join(
            folder.path,
            'inbox',
            INBOX_CLAIMS_FOLDER,
            '7777-5',
          ),
          { recursive: true, },
        );
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        // The slow driver's folder stays, with its claim folder around it; the one that is gone takes its own folder away.
        expect(await entriesOf(join(folder.path, 'inbox', INBOX_CLAIMS_FOLDER,),),).toEqual([`${OWNER_PID}-${OWNER_START}`,],);
        expect((await summaryOf(record,)).inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'keeps a message whose claim folder names no process at all',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder of a machine that holds no such process.
         */
        const processFolder = join(folder.path, 'proc',);
        await mkdir(processFolder,);
        await writeClaim({
          folder: folder.path,
          owner: 'not-an-owner',
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        expect(await entriesOf(join(folder.path, 'inbox',),),).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
        expect((await summaryOf(record,)).inboxNotTaken,).toEqual([MESSAGE_NAME,],);
      },
    },),
  ],
},);

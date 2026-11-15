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
import { dirname, join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type CommandAnswer,
  deliverReadyMessage,
  type DriverOptions,
  INBOX_CLAIMS_FOLDER,
  keepMessage,
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

/**
 The texts of every file under a folder, so a test can say that a message survived somewhere rather than at one path.

 @param folder - the folder to read

 @returns the text of each file, in no particular order
 */
async function textsUnder(folder: string,): Promise<readonly string[]> {
  /**
   What is directly in the folder.
   */
  const entries = await readdir(folder, { withFileTypes: true, },);
  /**
   The texts of each entry: a folder is read through, a file is read.
   */
  const texts = await Promise.all(entries.map(async function read(entry,) {
    /**
     The entry's path.
     */
    const path = join(folder, entry.name,);
    return entry.isDirectory() ? await textsUnder(path,) : [await readFile(path, 'utf8',)];
  },),);
  return texts.flat();
}

/**
 A connection that refuses every message while Pi rests.

 @returns the connection a watch reads
 */
function refusingWhileResting(): WatchRequest['connection'] {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send(): Promise<CommandAnswer> {
      return {
        ending: 'answered',
        record: {
          success: false,
          error: 'refused while resting',
        },
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
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
        /**
         What the inbox holds now.
         */
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toEqual([MESSAGE_NAME,],);
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
        /**
         What the inbox holds now.
         */
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([INBOX_CLAIMS_FOLDER,],);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toBeUndefined();
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
        /**
         What the inbox holds now.
         */
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([INBOX_CLAIMS_FOLDER,],);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toBeUndefined();
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
        /**
         What the second driver's summary says about the message.
         */
        const secondSummary = await summaryOf(second.record,);
        expect(secondSummary.inboxNotTaken,).toBeUndefined();
        /**
         What the inbox holds now.
         */
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
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
        /**
         The claim folders left in the inbox.
         */
        const owners = await entriesOf(join(folder.path, 'inbox', INBOX_CLAIMS_FOLDER,),);
        expect(owners,).toEqual([`${OWNER_PID}-${OWNER_START}`,],);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'leaves a message whose claim folder names no process at all, because nothing proves its owner gone',
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
        // The message stays under its own name inside the claim, and no driver may take it on a guess.
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([INBOX_CLAIMS_FOLDER,],);
        /**
         The message as the claim still holds it.
         */
        const text = await readFile(join(folder.path, 'inbox', INBOX_CLAIMS_FOLDER, 'not-an-owner', MESSAGE_NAME,), 'utf8',);
        expect(text,).toBe(MESSAGE_TEXT,);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'leaves a claim whose owner start could not be read while its process number names something',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder that holds a process at the number the claim names.
         */
        const processFolder = join(folder.path, 'proc',);
        await writeProcess({
          processFolder,
          pid: OWNER_PID,
          start: OWNER_START,
        },);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-unknown`,
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        // The number names something, which may be the driver that claimed the message; no start was recorded, so nothing here
        // proves that driver gone.
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([INBOX_CLAIMS_FOLDER,],);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'keeps a claim whose owner start could not be read once its process number is gone',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The process folder of a machine that holds no such process.
         */
        const processFolder = join(folder.path, 'proc',);
        await mkdir(processFolder,);
        await writeClaim({
          folder: folder.path,
          owner: `${OWNER_PID}-unknown`,
        },);
        const { watch, record, } = await watchOver({
          folder: folder.path,
          processFolder,
        },);
        await recoverClaimsOfGoneDrivers({ watch, },);
        // A number that names nothing cannot belong to a live driver, which is the one proof an unreadable start leaves.
        const entries = await entriesOf(join(folder.path, 'inbox',),);
        expect(entries,).toEqual([`.orphaned-${MESSAGE_NAME}`,],);
        /**
         What the summary says about the message.
         */
        const summary = await summaryOf(record,);
        expect(summary.inboxNotTaken,).toEqual([MESSAGE_NAME,],);
      },
    },),
    it({
      name: 'a second claim of one name does not overwrite the claim that could not be kept',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The inbox both publications are written into.
         */
        const inbox = join(folder.path, 'inbox',);
        await mkdir(inbox, { recursive: true, },);
        /**
         A name with no room for a kept prefix, so a refused first message stays under its claim.
         */
        const name = 'n'.repeat(255,);
        /**
         What each publication holds.
         */
        const firstText = 'the first message, which must survive';
        const secondText = 'the second message, published under the same name';
        await writeFile(join(inbox, name,), firstText, { encoding: 'utf8', },);
        /**
         The run's own record.
         */
        const record = startRecordOfPiRun({ feedFile: join(
          folder.path,
          'feed.jsonl',
        ), },);
        /**
         The watch both deliveries use.
         */
        const watch: WatchRequest = {
          connection: refusingWhileResting(),
          record,
          options: optionsOf(folder.path,),
          inbox,
          deadline: Date.now() + 5_000,
          processFolder: '/proc',
        };
        /**
         What the first publication came to.
         */
        const first = await deliverReadyMessage({
          watch,
          name,
        },);
        expect(first,).toEqual({ ending: 'not-taken', },);
        await writeFile(join(inbox, name,), secondText, { encoding: 'utf8', },);
        /**
         What the second publication came to.
         */
        const second = await deliverReadyMessage({
          watch,
          name,
        },);
        expect(second,).toEqual({ ending: 'not-taken', },);
        /**
         What the claims still hold, which must be both messages.
         */
        const texts = await textsUnder(join(inbox, INBOX_CLAIMS_FOLDER,),);
        expect(texts,).toContain(firstText,);
        expect(texts,).toContain(secondText,);
        await record.close();
      },
    },),
    it({
      name: 'two keeps of one name on a filesystem without hard links both survive',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The inbox both kept messages live in.
         */
        const inbox = join(folder.path, 'inbox',);
        await mkdir(inbox, { recursive: true, },);
        /**
         The name both messages were published under.
         */
        const name = MESSAGE_NAME;
        /**
         What each claim holds.
         */
        const firstText = 'the first kept message';
        const secondText = 'the second kept message';
        /**
         Where each claim is.
         */
        const firstClaim = join(inbox, '.claims', 'a-1', 'one', name,);
        const secondClaim = join(inbox, '.claims', 'b-2', 'two', name,);
        await mkdir(dirname(firstClaim,), { recursive: true, },);
        await mkdir(dirname(secondClaim,), { recursive: true, },);
        await writeFile(firstClaim, firstText, { encoding: 'utf8', },);
        await writeFile(secondClaim, secondText, { encoding: 'utf8', },);
        /**
         The run's own record.
         */
        const record = startRecordOfPiRun({ feedFile: join(
          folder.path,
          'feed.jsonl',
        ), },);
        /**
         The watch both keeps use.
         */
        const watch: WatchRequest = {
          connection: quietConnection(),
          record,
          options: optionsOf(folder.path,),
          inbox,
          deadline: Date.now() + 5_000,
          processFolder: '/proc',
        };
        /**
         The hard link a filesystem without them refuses, which this test's filesystem gives.
         */
        async function refusesLinks(): Promise<void> {
          /**
           What the filesystem would say.
           */
          const failure = new Error('EPERM: operation not permitted',);
          Object.assign(failure, { code: 'EPERM', },);
          throw failure;
        }
        /**
         Where the first keep went.
         */
        const firstKept = await keepMessage({
          watch,
          name,
          claim: firstClaim,
          keptAs: 'failed',
          link: refusesLinks,
        },);
        /**
         Where the second keep went.
         */
        const secondKept = await keepMessage({
          watch,
          name,
          claim: secondClaim,
          keptAs: 'failed',
          link: refusesLinks,
        },);
        expect(firstKept,).not.toBe(secondKept,);
        expect(await readFile(firstKept, 'utf8',),).toBe(firstText,);
        expect(await readFile(secondKept, 'utf8',),).toBe(secondText,);
        await record.close();
      },
    },),
  ],
},);

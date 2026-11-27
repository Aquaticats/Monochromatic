/**
 The scratch folder one run's agent leaves under `${HOME}/temp/agent`, and what clearing it does: everything beside `keep/` goes,
 `keep/` and what it holds stay, and a folder that some work still touches is left with the reason.

 Every home here is a folder of this test's own under the harness's scratch root, so no test reads or changes `${HOME}/temp/agent`.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  symlink,
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
  AGENT_SCRATCH_FOLDER_PARTS,
  KEPT_FOLDER_NAME,
  SCRATCH_FOLDER_NAME_END,
  type ScratchFolderOutcome,
  clearScratchFolderOfARun,
  isThere,
  scratchFolderOfARun,
} from '../dist/final/node/index.mjs';

/**
 A home directory of one test's own, with the folder every run's scratch lives in and the folder the process table is read from.

 The home is what a brief's `${HOME}` names, so the folders under it are the folders a run's own brief would make, and the test
 removes the whole home however it ends.
 */
type TestHome = {
  /**
   The home directory the run is given, which holds its scratch under `temp/agent`.
   */
  readonly home: string;

  /**
   The folder every run's scratch lives in, `<home>/temp/agent`.
   */
  readonly agentScratchFolder: string;

  /**
   The folder holding one entry per process, which holds none unless a test writes one.
   */
  readonly processFolder: string;

  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   This run's own folder inside `runsFolder`, which is the newest attempt of its name until a test makes another.
   */
  readonly runFolder: string;

  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a home directory of the test's own.

 @returns the home, the folders inside it a run's scratch and the process table are read from, and the disposer

 @example
 ```ts
 await using home = await makeHome();
 ```
 */
async function makeHome(): Promise<TestHome> {
  /**
   The home directory, under the harness's scratch root rather than the system's temporary folder.
   */
  const home = await mkdtemp(join(scratchRoot(), 'pi-run-scratch-folder-',),);
  /**
   The folder every run's scratch lives in, made before the test so the test writes only inside it.
   */
  const agentScratchFolder = join(
    home,
    ...AGENT_SCRATCH_FOLDER_PARTS,
  );
  /**
   The folder the process table is read from.
   */
  const processFolder = join(
    home,
    'processes',
  );
  /**
   Folder the driver writes one run folder per run into.
   */
  const runsFolder = join(
    home,
    'runs',
  );
  /**
   This run's own folder, which the launcher makes before its unit starts.
   */
  const runFolder = join(
    runsFolder,
    'copy-route-1',
  );
  await Promise.all([agentScratchFolder, processFolder, runsFolder, runFolder,].map(function makes(folder,) {
    return mkdir(
      folder,
      { recursive: true, },
    );
  },),);
  return {
    home,
    agentScratchFolder,
    processFolder,
    runsFolder,
    runFolder,
    async [Symbol.asyncDispose]() {
      await rm(
        home,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Writes one file, making the folders above it.

 @param file - the file to write

 @param text - what it holds

 @example
 ```ts
 await writeFileIn({ file: '/runs/a/build/index.html', text: '<p>built</p>' });
 ```
 */
async function writeFileIn({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string;
},): Promise<void> {
  await mkdir(
    dirname(file,),
    { recursive: true, },
  );
  await writeFile(
    file,
    text,
  );
}

/**
 Reads a folder's entries in a fixed order, so a test compares names rather than the order a filesystem lists them in.

 @param folder - the folder to list

 @returns its entries, sorted

 @example
 ```ts
 const entries = await entriesIn('/runs/a');
 ```
 */
async function entriesIn(folder: string,): Promise<readonly string[]> {
  return (await readdir(folder,)).toSorted();
}

/**
 Writes one process into a test's process table, as the kernel would list it.

 @param request - the process folder, the process's number, and the folder it works in

 @example
 ```ts
 await writeProcessAt({ processFolder: '/home/a/processes', pid: '4242', folder: '/home/a/temp/agent/a-scratch' });
 ```
 */
async function writeProcessAt({
  processFolder,
  pid,
  folder,
}: {
  readonly processFolder: string;
  readonly pid: string;
  readonly folder: string;
},): Promise<void> {
  /**
   The process's own entry in the process folder.
   */
  const entry = join(
    processFolder,
    pid,
  );
  await mkdir(
    entry,
    { recursive: true, },
  );
  await writeFile(
    join(
      entry,
      'comm',
    ),
    'bash',
  );
  // The folder a process works in is a link to it on a Linux machine's process folder, so the test writes it as one.
  await symlink(
    folder,
    join(
      entry,
      'cwd',
    ),
  );
}

/**
 Reads why a folder stays or was cleared only in part, which only those answers carry.

 @param outcome - what the clearing answered

 @returns the reason, in words a coordinator acts on

 @throws Error when the folder was not kept, which would make every assertion about the reason test the wrong arm

 @example
 ```ts
 const reason = reasonOf(outcome);
 ```
 */
function reasonOf(outcome: ScratchFolderOutcome,): string {
  if ((outcome.action !== 'kept') && (outcome.action !== 'partly-cleared')) {
    throw new Error(`the folder was neither kept nor cleared in part, so no reason is there to read: ${outcome.line}`,);
  }
  return outcome.reason;
}

/**
 Reads why no folder was named, which only a refusal carries.

 @param named - what naming the folder answered

 @returns the refusal, in words a reader acts on

 @throws Error when a folder was named, which would make an assertion about the refusal test the wrong arm

 @example
 ```ts
 const refusal = refusalOf(named);
 ```
 */
function refusalOf(named: ReturnType<typeof scratchFolderOfARun>,): string {
  if (!('refusal' in named)) {
    throw new Error(`a folder was named where a refusal was expected: ${named.folder}`,);
  }
  return named.refusal;
}

await describe({
  name: clearScratchFolderOfARun.name,
  children: [
    it({
      name: 'clears everything beside keep/ of a run that has ended, and keeps keep/ with what it holds',
      fn: async () => {
        await using home = await makeHome();
        /**
         The run's scratch folder, as its brief names it.
         */
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        await writeFileIn({ file: join(
          folder,
          'measurements.txt',
        ), text: '12 runs', },);
        await writeFileIn({ file: join(
          folder,
          'checkout',
          'README.md',
        ), text: 'a clone of another repository', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'logs',
          'run.log',
        ), text: 'a line of the log', },);
        /**
         What the clearing answered.
         */
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('cleared',);
        expect(outcome,).toMatchObject({ removed: 3, },);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
        /**
         What the kept folder holds, which is everything the run put there.
         */
        const keptHolds = await entriesIn(join(
          folder,
          KEPT_FOLDER_NAME,
        ),);
        expect(keptHolds,).toEqual([
          'logs',
          'report.md',
        ],);
        expect(await readFile(
          join(
            folder,
            KEPT_FOLDER_NAME,
            'logs',
            'run.log',
          ),
          'utf8',
        ),).toBe('a line of the log',);
        expect(outcome.line,).toContain(folder,);
        expect(outcome.line,).toContain('3 entries removed',);
      },
    },),
    it({
      name: 'clears an entry whose name only starts with keep, and says one entry was removed',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'keepsake',
        ), text: 'a file that only starts with keep', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('cleared',);
        expect(outcome,).toMatchObject({ removed: 1, },);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
        expect(outcome.line,).toContain('1 entry removed',);
      },
    },),
    it({
      name: 'keeps the whole folder of a run whose unit the launcher could not see end',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: false,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('kept',);
        expect(reasonOf(outcome,),).toContain('gave up',);
        expect(await entriesIn(folder,),).toEqual([
          'build',
          KEPT_FOLDER_NAME,
        ],);
      },
    },),
    it({
      name: 'keeps the whole folder while the run\'s own work still runs',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: true,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('kept',);
        expect(reasonOf(outcome,),).toContain('own work still runs',);
        expect(await entriesIn(folder,),).toEqual(['build',],);
      },
    },),
    it({
      name: 'keeps a folder some process still works in, and clears it once that process is gone',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        await writeProcessAt({
          processFolder: home.processFolder,
          pid: '4242',
          folder,
        },);
        const whileItWorks = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(whileItWorks.action,).toBe('kept',);
        expect(reasonOf(whileItWorks,),).toContain('a process still works in it',);
        expect(whileItWorks.line,).toContain(folder,);
        expect(await entriesIn(folder,),).toEqual([
          'build',
          KEPT_FOLDER_NAME,
        ],);
        // The control arm: the same folder, the same call, once the process the fixture named is gone.
        await rm(
          join(
            home.processFolder,
            '4242',
          ),
          { recursive: true, force: true, },
        );
        const onceItIsGone = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(onceItIsGone.action,).toBe('cleared',);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
      },
    },),
    it({
      name: 'answers that no scratch folder is there, and deletes nothing of another run',
      fn: async () => {
        await using home = await makeHome();
        /**
         Another run's scratch folder, which this run's clearing must not touch.
         */
        const another = join(
          home.agentScratchFolder,
          `another-run${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          another,
          'measurements.txt',
        ), text: 'another run\'s', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('nothing-there',);
        expect(outcome.line,).toContain(`copy-route${SCRATCH_FOLDER_NAME_END}`,);
        expect(await entriesIn(another,),).toEqual(['measurements.txt',],);
      },
    },),
    it({
      name: 'keeps the folder while a newer run of the same name has started, and clears it once that run is gone',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        // A newer attempt of the same name takes the next run folder, which is what a resumed run and a fix run each do.
        const newer = join(
          home.runsFolder,
          'copy-route-2',
        );
        await mkdir(
          newer,
          { recursive: true, },
        );
        const whileTheNewerRunWorks = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(whileTheNewerRunWorks.action,).toBe('kept',);
        expect(reasonOf(whileTheNewerRunWorks,),).toContain('newer run',);
        expect(await entriesIn(folder,),).toEqual([
          'build',
          KEPT_FOLDER_NAME,
        ],);
        // The control arm: the same call, once no newer attempt of the name is there.
        await rm(
          newer,
          { recursive: true, force: true, },
        );
        const whileOnlyThisRunIsThere = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(whileOnlyThisRunIsThere.action,).toBe('cleared',);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
      },
    },),
    it({
      name: 'says which entries it could not remove, and leaves them, when the operating system refuses some of them',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        await writeFileIn({ file: join(
          folder,
          'locked',
          'inside.txt',
        ), text: 'a file of a folder that may not be written', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        // A folder this account may read and enter but not write in is one `rm` cannot unlink what it holds.
        await chmod(
          join(
            folder,
            'locked',
          ),
          0o500,
        );
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        // The folder is given back its mode first, so the test's own removal of the home reaches what is inside it.
        await chmod(
          join(
            folder,
            'locked',
          ),
          0o700,
        );
        expect(outcome.action,).toBe('partly-cleared',);
        expect(outcome,).toMatchObject({ removed: 1, },);
        expect(reasonOf(outcome,),).toContain('could not be removed',);
        expect(outcome.line,).toContain('cleared only in part',);
        expect(await entriesIn(folder,),).toEqual([
          KEPT_FOLDER_NAME,
          'locked',
        ],);
        /**
         What the folder the operating system refused to write in still holds.
         */
        const lockedHolds = await entriesIn(join(
          folder,
          'locked',
        ),);
        expect(lockedHolds,).toEqual(['inside.txt',],);
      },
    },),
    it({
      name: 'never follows a scratch folder that is a symbolic link',
      fn: async () => {
        await using home = await makeHome();
        /**
         The folder the link names, which holds a file that must stay.
         */
        const elsewhere = join(
          home.home,
          'elsewhere',
        );
        await writeFileIn({ file: join(
          elsewhere,
          'must-stay.txt',
        ), text: 'not this run\'s to remove', },);
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await symlink(
          elsewhere,
          folder,
        );
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('kept',);
        expect(reasonOf(outcome,),).toContain('symbolic link',);
        expect((await lstat(folder,)).isSymbolicLink(),).toBe(true,);
        expect(await entriesIn(elsewhere,),).toEqual(['must-stay.txt',],);
      },
    },),
    it({
      name: 'removes a symbolic link inside the folder as the link it is, and what it named stays',
      fn: async () => {
        await using home = await makeHome();
        /**
         A checkout the run linked to rather than copied, which is not the run's to remove.
         */
        const checkout = join(
          home.home,
          'a-checkout',
        );
        await writeFileIn({ file: join(
          checkout,
          'tracked.txt',
        ), text: 'a file of the checkout', },);
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await mkdir(
          folder,
          { recursive: true, },
        );
        await symlink(
          checkout,
          join(
            folder,
            'link-to-the-checkout',
          ),
        );
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('cleared',);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
        expect(await entriesIn(checkout,),).toEqual(['tracked.txt',],);
      },
    },),
    it({
      name: 'removes a folder that held no keep/ whole, and says so',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('cleared',);
        expect(outcome,).toMatchObject({ removed: 1, },);
        expect(await isThere(folder,),).toBe(false,);
        expect(outcome.line,).toContain(`held no ${KEPT_FOLDER_NAME}/`,);
      },
    },),
    it({
      name: 'removes an empty folder that held no keep/, saying nothing was removed',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await mkdir(
          folder,
          { recursive: true, },
        );
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('cleared',);
        expect(outcome,).toMatchObject({ removed: 0, },);
        expect(outcome.line,).toContain('nothing was removed',);
        expect(await isThere(folder,),).toBe(false,);
      },
    },),
    it({
      name: 'clears nothing beside keep/ when the folder holds nothing beside it',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('cleared',);
        expect(outcome,).toMatchObject({ removed: 0, },);
        expect(outcome.line,).toContain('nothing was removed',);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
      },
    },),
    it({
      name: 'finds the keep/ a resumed run left, and clears beside it again when that run ends',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'build',
          'index.html',
        ), text: '<p>built</p>', },);
        await writeFileIn({ file: join(
          folder,
          KEPT_FOLDER_NAME,
          'report.md',
        ), text: 'the report', },);
        await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        // The resumed run reads its report, and writes the folder a build of its own made.
        expect(await readFile(
          join(
            folder,
            KEPT_FOLDER_NAME,
            'report.md',
          ),
          'utf8',
        ),).toBe('the report',);
        await writeFileIn({ file: join(
          folder,
          'build',
          'again.html',
        ), text: '<p>built again</p>', },);
        const secondEnd = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(secondEnd.action,).toBe('cleared',);
        expect(secondEnd,).toMatchObject({ removed: 1, },);
        expect(await entriesIn(folder,),).toEqual([KEPT_FOLDER_NAME,],);
        expect(await readFile(
          join(
            folder,
            KEPT_FOLDER_NAME,
            'report.md',
          ),
          'utf8',
        ),).toBe('the report',);
      },
    },),
    it({
      name: 'keeps a folder it cannot read, saying what the operating system answered',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          folder,
          'measurements.txt',
        ), text: '12 runs', },);
        await chmod(
          folder,
          0o000,
        );
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        // The folder is given back its mode first, so the test's own removal of the home reaches what is inside it.
        await chmod(
          folder,
          0o700,
        );
        expect(outcome.action,).toBe('kept',);
        expect(reasonOf(outcome,),).toContain('could not be read or cleared',);
        expect(await entriesIn(folder,),).toEqual(['measurements.txt',],);
      },
    },),
    it({
      name: 'keeps a path that is not a folder, since what it names is not this run\'s',
      fn: async () => {
        await using home = await makeHome();
        const folder = join(
          home.agentScratchFolder,
          `copy-route${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFile(
          folder,
          'a file where a folder would be',
        );
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('kept',);
        expect(reasonOf(outcome,),).toContain('not a folder',);
        expect(await readFile(
          folder,
          'utf8',
        ),).toBe('a file where a folder would be',);
      },
    },),
    it({
      name: 'refuses a run with no name before anything is deleted',
      fn: async () => {
        await using home = await makeHome();
        /**
         A folder the empty name would be joined to, which no call may remove from.
         */
        const folderOfAnEmptyName = join(
          home.agentScratchFolder,
          SCRATCH_FOLDER_NAME_END,
        );
        await writeFileIn({ file: join(
          folderOfAnEmptyName,
          'measurements.txt',
        ), text: '12 runs', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: '',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('refused',);
        expect(outcome.line,).toContain('has no name',);
        expect(outcome.line,).toContain('nothing was deleted',);
        expect(await entriesIn(folderOfAnEmptyName,),).toEqual(['measurements.txt',],);
      },
    },),
    it({
      name: 'refuses a scratch root that is not an absolute path, before anything is deleted',
      fn: async () => {
        /**
         A root written the way a home directory that is empty or relative would leave it.
         */
        const relativeRoot = process.cwd();
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: 'temp/agent',
          runsFolder: '/runs',
          runFolder: '/runs/copy-route-1',
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: join(
            relativeRoot,
            'no-processes-here',
          ),
        },);
        expect(outcome.action,).toBe('refused',);
        expect(outcome.line,).toContain('not an absolute path',);
        expect(outcome.line,).toContain('temp/agent',);
      },
    },),
    it({
      name: 'refuses a run name that would name a folder outside the root, and deletes nothing outside it',
      fn: async () => {
        await using home = await makeHome();
        /**
         The folder a name holding `..` would reach, which holds a file that must stay.
         */
        const outside = join(
          home.home,
          'temp',
          `escape${SCRATCH_FOLDER_NAME_END}`,
        );
        await writeFileIn({ file: join(
          outside,
          'must-stay.txt',
        ), text: 'not this run\'s to remove', },);
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: home.agentScratchFolder,
          runsFolder: home.runsFolder,
          runFolder: home.runFolder,
          runName: '../escape',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: home.processFolder,
        },);
        expect(outcome.action,).toBe('refused',);
        expect(outcome.line,).toContain('path separator',);
        expect(await entriesIn(outside,),).toEqual(['must-stay.txt',],);
      },
    },),
    it({
      name: 'refuses an empty scratch root, naming the empty value rather than a folder',
      fn: async () => {
        const outcome = await clearScratchFolderOfARun({
          agentScratchFolder: '',
          runsFolder: '/runs',
          runFolder: '/runs/copy-route-1',
          runName: 'copy-route',
          unitEnded: true,
          ownWorkStillRuns: false,
          processFolder: '/proc',
        },);
        expect(outcome.action,).toBe('refused',);
        expect(outcome.line,).toContain('it is empty',);
      },
    },),
  ],
},);

await describe({
  name: scratchFolderOfARun.name,
  children: [
    it({
      name: 'names the folder from the root every run\'s scratch lives in and the run\'s name',
      fn: async () => {
        expect(scratchFolderOfARun({
          agentScratchFolder: '/home/a/temp/agent',
          runName: 'copy-route',
        },),).toEqual({ folder: `/home/a/temp/agent/copy-route${SCRATCH_FOLDER_NAME_END}`, },);
      },
    },),
    it({
      name: 'names no folder for a run whose name holds a separator, and says why',
      fn: async () => {
        /**
         What the name answered.
         */
        const named = scratchFolderOfARun({
          agentScratchFolder: '/home/a/temp/agent',
          runName: 'a/b',
        },);
        expect(refusalOf(named,),).toContain('/home/a/temp/agent',);
        expect(named,).not.toHaveProperty('folder',);
      },
    },),
  ],
},);

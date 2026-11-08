/**
 Test of the cut-over check that compares the table and the worktree list taken through the link with the frozen
 baseline: Git prints the main worktree under its resolved path, so that one line is spelled as the 860 path through the
 link where the baseline spells the link path. The check must allow exactly that, must require both spellings, and must
 still fail on a difference on any other line.

 @module
 */

import {
  mkdir,
  mkdtemp,
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
  createGuidedConfig,
  cutoverStages,
  runShell,
  type GuidedConfig,
  type Stage,
} from '../dist/final/node/index.mjs';

/**
 Identifier of the stage that checks the repository through the link.
 */
const THROUGH_LINK_STAGE_ID = 'c20-through-link';

/**
 Header of the worktree table the scan writes, as the scan spells it.
 */
const TABLE_HEADER = 'path\tstate\tchanged\tuntracked\tunfinished\tindexLock\thead\tindexSha256';

/**
 A commit of the main worktree, written in every row of both tables.
 */
const MAIN_HEAD = '1a907ccfb94a8e2f0a4fd1b4cf3a4d24f1df5f6a';

/**
 A commit of the nested worktree, which the planted differences change.
 */
const NESTED_HEAD = '642f8e148c5f0f8a4a1a3f0ac0b9ff5dbb3a2f0e';

/**
 The index hash the scan writes for a worktree with nothing to strand.
 */
const CLEAN_INDEX_SHA = '0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f';

/**
 Exit status of a check that passed.
 */
const CHECK_PASSED = 0;

/**
 Exit status of a check that failed on its own terms.
 */
const CHECK_FAILED = 1;

/**
 Deadline of a shell run, in seconds: the check only reads small files.
 */
const SHELL_DEADLINE_SECONDS = 30;

/**
 Seconds between terminate and kill for a shell run.
 */
const SHELL_KILL_GRACE_SECONDS = 5;

/**
 The folder of the one nested worktree the fixtures hold, below the checkout.
 */
const NESTED_WORKTREE = join('.claude', 'worktrees', 'a',);

/**
 The name of the branch of the nested worktree.
 */
const NESTED_BRANCH = 'feat/a';

/**
 A disposable home folder with the move's state folder, removed when the scope ends.
 */
type HomeFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A disposable home, the configuration it gives and the stage's check text.
 */
type ThroughLink = {
  readonly home: HomeFolder;
  readonly config: GuidedConfig;
  readonly check: string;
};

/**
 Makes a disposable home folder.

 @returns the folder, to hold with `await using`

 @example
 ```ts
 await using home = await makeHome();
 ```
 */
async function makeHome(): Promise<HomeFolder> {
  /**
   Path of the new folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'guided-through-link-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The stage, its configuration and its check, built from a disposable home.

 @returns the home, the configuration and the check text

 @example
 ```ts
 const stage = await throughLink();
 ```
 */
async function throughLink(): Promise<ThroughLink> {
  /**
   The disposable home this invocation reads.
   */
  const home = await makeHome();
  /**
   The configuration the stages read.
   */
  const config = await createGuidedConfig({
    env: {
      HOME: home.path,
      MOVE_DRIVE: join(home.path, 'drive',),
      MOVE_DESTINATION_ROOT: join(home.path, 'drive', 'repository',),
    },
    nodePath: process.execPath,
  },);
  await mkdir(config.wiz, { recursive: true, },);
  /**
   The cut-over stages.
   */
  const stages = await cutoverStages({ c: config, stamp: 'S', },);
  /**
   The stage that checks the repository through the link.
   */
  const stage = stages.find((candidate: Stage,) => candidate.id === THROUGH_LINK_STAGE_ID,) as Stage;
  if ((typeof stage.check) !== 'string')
    throw new Error(`stage ${THROUGH_LINK_STAGE_ID} has no plain check text`,);
  return {
    home,
    config,
    check: stage.check,
  };
}

/**
 A table row of one worktree.

 @param path - the worktree's path as the scan prints it

 @param head - the worktree's head

 @returns the tab-separated row

 @example
 ```ts
 const row = tableRow({ path: '/x', head: MAIN_HEAD, });
 ```
 */
function tableRow(
  {
    path,
    head,
  }: {
    readonly path: string;
    readonly head: string;
  },
): string {
  return [
    path,
    'ok',
    '0',
    '0',
    '',
    'false',
    head,
    CLEAN_INDEX_SHA,
  ].join('\t',);
}

/**
 The worktree table as the scan writes it.

 @param mainPath - the spelling of the main worktree's path

 @param nestedHead - the head of the nested worktree

 @returns the table's text

 @example
 ```ts
 const text = table({ mainPath: '/x', nestedHead: MAIN_HEAD, });
 ```
 */
function table(
  {
    mainPath,
    nestedHead,
  }: {
    readonly mainPath: string;
    readonly nestedHead: string;
  },
): string {
  return `${[
    TABLE_HEADER,
    tableRow({ path: mainPath, head: MAIN_HEAD, },),
    tableRow({
      path: join(mainPath, NESTED_WORKTREE,),
      head: nestedHead,
    },),
  ].join('\n',)}\n`;
}

/**
 `git worktree list --porcelain` as Git prints it.

 @param mainPath - the spelling of the main worktree's path

 @param nestedHead - the head of the nested worktree

 @returns the listing's text

 @example
 ```ts
 const listing = porcelain({ mainPath: '/x', nestedHead: MAIN_HEAD, });
 ```
 */
function porcelain(
  {
    mainPath,
    nestedHead,
  }: {
    readonly mainPath: string;
    readonly nestedHead: string;
  },
): string {
  return [
    `worktree ${mainPath}`,
    `HEAD ${MAIN_HEAD}`,
    'branch refs/heads/main',
    '',
    `worktree ${join(mainPath, NESTED_WORKTREE,)}`,
    `HEAD ${nestedHead}`,
    `branch refs/heads/${NESTED_BRANCH}`,
    '',
    '',
  ].join('\n',);
}

/**
 Writes the four files the check reads and runs it.

 @param stage - the stage, its home and its configuration

 @param fixtures - the baseline and through-link texts

 @returns the check's exit status and everything it printed

 @example
 ```ts
 const { status, printed, } = await runCheck({ stage, ...fixtures });
 ```
 */
async function runCheck(
  {
    stage,
    baselineTable,
    throughLinkTable,
    baselineListing,
    throughLinkListing,
  }: {
    readonly stage: ThroughLink;
    readonly baselineTable: string;
    readonly throughLinkTable: string;
    readonly baselineListing: string;
    readonly throughLinkListing: string;
  },
): Promise<{
  readonly status: number;
  readonly printed: string;
}> {
  await writeFile(join(stage.config.wiz, 'baseline-frozen.tsv',), baselineTable,);
  await writeFile(join(stage.config.wiz, 'through-link.tsv',), throughLinkTable,);
  await writeFile(join(stage.config.wiz, 'porcelain-frozen.txt',), baselineListing,);
  await writeFile(join(stage.config.wiz, 'porcelain-through-link.txt',), throughLinkListing,);
  /**
   What the check printed.
   */
  const printed: string[] = [];
  /**
   The exit status of the check.
   */
  const status = await runShell({
    command: stage.check,
    outputFile: join(stage.home.path, 'output',),
    seconds: SHELL_DEADLINE_SECONDS,
    killGrace: SHELL_KILL_GRACE_SECONDS,
    outputFileEnv: join(stage.home.path, 'output',),
    out: (text: string,) => {
      printed.push(text,);
    },
  },);
  return {
    status,
    printed: printed.join('',),
  };
}

await describe({
  name: 'the through-the-link check of the cut-over',
  children: [
    it({
      name: 'passes when only the main worktree is spelled as the 860 path through the link',
      fn: async () => {
        /**
         The stage, its home and its two path spellings.
         */
        const stage = await throughLink();
        const result = await runCheck({
          stage,
          baselineTable: table({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkTable: table({
            mainPath: stage.config.finalPath,
            nestedHead: NESTED_HEAD,
          },),
          baselineListing: porcelain({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkListing: porcelain({
            mainPath: stage.config.finalPath,
            nestedHead: NESTED_HEAD,
          },),
        },);
        await stage.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_PASSED,);
      },
    },),
    it({
      name: 'fails when another line of the table differs',
      fn: async () => {
        const stage = await throughLink();
        const result = await runCheck({
          stage,
          baselineTable: table({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkTable: table({
            mainPath: stage.config.finalPath,
            nestedHead: MAIN_HEAD,
          },),
          baselineListing: porcelain({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkListing: porcelain({
            mainPath: stage.config.finalPath,
            nestedHead: NESTED_HEAD,
          },),
        },);
        await stage.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_FAILED,);
        expect(result.printed,).toContain(NESTED_HEAD,);
      },
    },),
    it({
      name: 'fails when a nested worktree is spelled with the 860 path, which is not the one allowed line',
      fn: async () => {
        const stage = await throughLink();
        /**
         The same table through the link, with the nested path spelled under the 860 path as well.
         */
        const nestedMoved = table({
          mainPath: stage.config.finalPath,
          nestedHead: NESTED_HEAD,
        },);
        const result = await runCheck({
          stage,
          baselineTable: table({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkTable: nestedMoved,
          baselineListing: porcelain({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkListing: porcelain({
            mainPath: stage.config.finalPath,
            nestedHead: NESTED_HEAD,
          },),
        },);
        await stage.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_FAILED,);
        expect(result.printed,).toContain(stage.config.finalPath,);
      },
    },),
    it({
      name: 'fails when the worktree list still spells the main worktree as the link path',
      fn: async () => {
        const stage = await throughLink();
        const result = await runCheck({
          stage,
          baselineTable: table({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkTable: table({
            mainPath: stage.config.finalPath,
            nestedHead: NESTED_HEAD,
          },),
          baselineListing: porcelain({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkListing: porcelain({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
        },);
        await stage.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_FAILED,);
      },
    },),
    it({
      name: 'fails when a line of the worktree list other than the main worktree differs',
      fn: async () => {
        const stage = await throughLink();
        const result = await runCheck({
          stage,
          baselineTable: table({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkTable: table({
            mainPath: stage.config.finalPath,
            nestedHead: NESTED_HEAD,
          },),
          baselineListing: porcelain({
            mainPath: stage.config.repo,
            nestedHead: NESTED_HEAD,
          },),
          throughLinkListing: porcelain({
            mainPath: stage.config.finalPath,
            nestedHead: MAIN_HEAD,
          },),
        },);
        await stage.home[Symbol.asyncDispose]();
        expect(result.status,).toBe(CHECK_FAILED,);
      },
    },),
  ],
},);

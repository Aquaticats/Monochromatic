/**
 Tests for the slice caches' part of the pre-launch cache check: the runs
 directories read, the records they hold, the directories the search could not
 list, and the newest record against the versions set after it.

 What is printed is read off `console.log` through a diverting capture, which
 is process-wide, so the suite runs one case at a time. Every runs directory is
 a scratch directory of invented records.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  chmod,
  mkdir,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
  type DisposableSandbox,
} from '@monochromatic-dev/module-test/ts';

import {
  reportSliceCaches,
  type VersionSetting,
} from '../../dist/final/node/index.mjs';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeSliceRecord, } from './cache-account-repo.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Time of the newest record the cases write, 2026-09-02T10:00:00Z.
 */
const NEWEST_SECONDS = 1_788_343_200;

/**
 Home directory the cases state, so the runbook's runs root reads the same on every machine.
 */
const HOME = '/cats/home';

/**
 Where a launch that follows the runbook writes its runs, under the home directory the cases state.
 */
const RUNBOOK_ROOT = join(
  HOME,
  'temp',
  'agent',
);

/**
 The warnings among the lines a level-keeping logger kept.

 @param lines - every line the logger kept, behind its level

 @returns Warning lines, each without its level

 @example
 ```ts
 const warned = warningsIn({ lines: ['info read', 'warn nothing read',], },); // ['nothing read']
 ```
 */
function warningsIn({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines
    .filter(function isWarning(line,): boolean {
      return line.startsWith('warn ',);
    },)
    .map(function withoutLevel(line,): string {
      return line.slice('warn '.length,);
    },);
}

/**
 What the audit says when it read no slice-cache record.

 @param roots - every root it searched, in the order it printed them

 @returns Sentence, without the stdout prefix

 @example
 ```ts
 const words = noRecordWarning({ roots: ['/cats/runs',], },);
 ```
 */
function noRecordWarning({ roots, }: { readonly roots: readonly string[]; },): string {
  return 'no slice-cache record was read, so the newest record and the versions set after it say nothing of the caches '
    + `a launch would resume. Roots searched: ${roots.join(', ',)}. A launch that follows the runbook writes its `
    + `runs under ${HOME}/temp/agent; pass --runs-under ${HOME}/temp/agent after -- to read them.`;
}

/**
 What the audit says when it read records and no root it searched reaches the runbook's runs root: by its real path,
 within the depth the search walks and through no directory it skips.

 @param roots - every root it searched, in the order it printed them

 @param home - home directory the runbook's runs root sits under

 @returns Sentence, without the stdout prefix

 @example
 ```ts
 const words = unreachedRootWarning({ roots: ['/cats/runs',], home: '/cats/home', },);
 ```
 */
function unreachedRootWarning(
  {
    roots,
    home,
  }: {
    readonly roots: readonly string[];
    readonly home: string;
  },
): string {
  /**
   Where a launch that follows the runbook writes its runs.
   */
  const runbookRoot = join(
    home,
    'temp',
    'agent',
  );
  return `the runbook's runs root ${runbookRoot} is not reached by the roots searched: ${roots.join(', ',)}. A search `
    + 'reaches runs directories at most 4 levels below a root, and the runbook\'s runs sit up to 2 levels below its '
    + 'root; it enters no hidden directory or node_modules, and a directory counts by its real path. The newest record '
    + 'and the versions set after it speak only for those roots, and a launch that follows the runbook writes its '
    + `runs under ${runbookRoot}; pass --runs-under ${runbookRoot} after -- to read them.`;
}

/**
 The warnings the audit logs when it reads one record and searches the given roots for a given home directory.

 @param searched - directories named after --runs-under

 @param home - home directory the runbook's runs root sits under

 @param sinon - case's own sandbox, which restores the console

 @returns Warning lines, each without its level, and the roots searched in the order the audit printed them

 @example
 ```ts
 const { warned, } = await warningsOverHome({ searched: [], home: '/cats/home', sinon: ctx.sinon, },);
 ```
 */
async function warningsOverHome(
  {
    searched,
    home,
    sinon,
  }: {
    readonly searched: readonly string[];
    readonly home: string;
    readonly sinon: DisposableSandbox;
  },
): Promise<{ readonly warned: readonly string[]; readonly roots: readonly string[]; }> {
  using _printed = divertingConsoleLog({ sinon, },);
  await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
  await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
  await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
  const logged: string[] = [];
  await reportSliceCaches({
    root: root.path,
    runsDir: runs.path,
    searched,
    settings: [],
    l: levelCapturingLogger({ lines: logged, },),
    home,
  },);
  return {
    warned: warningsIn({ lines: logged, },),
    roots: [
      join(root.path, 'node_modules', '.monochromatic',),
      runs.path,
      ...searched,
    ],
  };
}

/**
 One version's setting at a time.

 @param name - constant's name

 @param seconds - committer time of the commit that set it

 @returns The setting

 @example
 ```ts
 const setting = settingAt({ name: 'STAGE_CACHE_VERSION', seconds: 1_788_256_800, },);
 ```
 */
function settingAt({ name, seconds, }: { readonly name: string; readonly seconds: number; },): VersionSetting {
  return {
    version: {
      name,
      value: 1,
      path: 'src/stage-cache-version.ts',
      declaration: `${name} = 1`,
    },
    commit: {
      hash: 'b1f20a6e4aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      seconds,
      subject: 'set a cache version',
    },
    account: '',
  };
}

await describe({
  name: reportSliceCaches.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS THE RECORDS UNDER THE RUNS DIRECTORY A PASS WOULD USE, the newest one and how many versions '
        + 'were set after it',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        const record = await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [],
          settings: [
            settingAt({ name: 'STAGE_CACHE_VERSION', seconds: NEWEST_SECONDS - 1, },),
            settingAt({ name: 'PAIRING_CACHE_VERSION', seconds: NEWEST_SECONDS + 1, },),
          ],
          // The runbook's runs root of this home lies under the runs directory, which is a root searched.
          home: runs.path,
        },);

        expect(printed.lines,).toEqual([
          `slice-cache records under 1 runs directory found in ${join(root.path, 'node_modules', '.monochromatic',)}, `
            + `${runs.path}: 1`,
          'directories the search could not list, whose runs the count leaves out: 0',
          `newest slice-cache record: 2026-09-02T10:00Z ${record}`,
          'cache versions set after it: 1 of 2',
        ],);
      },
    },),
    it({
      name: 'SAYS NONE WAS FOUND where the runs hold no record, with the count as the control, and warns that the '
        + 'lines below it say nothing, naming every root searched and the runbook\'s runs root',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        /**
         What the warning says, on stdout after the prefix and in the log without it.
         */
        const warning = noRecordWarning({
          roots: [
            join(root.path, 'node_modules', '.monochromatic',),
            runs.path,
          ],
        },);
        expect({
          printed: printed.lines,
          logged: warningsIn({ lines: logged, },),
        },).toEqual({
          printed: [
            'slice-cache records under 1 runs directory found in '
              + `${join(root.path, 'node_modules', '.monochromatic',)}, ${runs.path}: 0`,
            'directories the search could not list, whose runs the count leaves out: 0',
            `WARNING: ${warning}`,
            'newest slice-cache record: none',
          ],
          logged: [warning,],
        },);
      },
    },),
    it({
      name: 'NAMES EVERY ROOT IT SEARCHED in the warning, the ones given after --runs-under beside the default two, '
        + 'so a reader sees which directories the zero can speak for',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [under.path,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect(warningsIn({ lines: logged, },),).toEqual([
          noRecordWarning({
            roots: [
              join(root.path, 'node_modules', '.monochromatic',),
              runs.path,
              under.path,
            ],
          },),
        ],);
        expect(printed.lines.filter(function isWarning(line,): boolean {
          return line.startsWith('WARNING: ',);
        },),).toEqual([`WARNING: ${warningsIn({ lines: logged, },)[0] ?? ''}`,],);
      },
    },),
    it({
      name: 'DOES NOT WARN where a record was read, so the warning is the zero and not a habit',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          // The runbook's runs root of this home lies under the runs directory, which is a root searched.
          home: runs.path,
        },);

        expect({
          warned: warningsIn({ lines: logged, },),
          printedWarning: printed.lines.some(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },),
        },).toEqual({
          warned: [],
          printedWarning: false,
        },);
      },
    },),
    it({
      name: 'WARNS WHERE RECORDS WERE READ BUT NO ROOT SEARCHED REACHES THE RUNBOOK\'S RUNS ROOT: '
        + 'a non-zero count then speaks for the roots searched and says nothing of the runs a launch that follows the '
        + 'runbook writes, so the warning names every root searched and the runbook\'s root',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        /**
         What the warning says, on stdout after the prefix and in the log without it.
         */
        const warning = unreachedRootWarning({
          roots: [
            join(root.path, 'node_modules', '.monochromatic',),
            runs.path,
          ],
          home: HOME,
        },);
        expect({
          logged: warningsIn({ lines: logged, },),
          printed: printed.lines.filter(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },),
        },).toEqual({
          logged: [warning,],
          printed: [`WARNING: ${warning}`,],
        },);
      },
    },),
    it({
      name: 'DOES NOT WARN OF THE RUNBOOK\'S RUNS ROOT WHERE A ROOT SEARCHED IS IT OR HOLDS IT, whatever the count',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const logged: string[] = [];
        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [RUNBOOK_ROOT,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);
        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [HOME,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect({
          warned: warningsIn({ lines: logged, },),
          printedWarning: printed.lines.some(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },),
        },).toEqual({
          warned: [],
          printedWarning: false,
        },);
      },
    },),
    it({
      name: 'WARNS WHERE THE ONLY ROOT NEAR THE RUNBOOK\'S IS ONE UNDER IT: a searched directory below the runbook\'s '
        + 'root covers its own runs and not the rest of the root',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const logged: string[] = [];
        /**
         A directory below the runbook's root.
         */
        const underRunbookRoot = join(
          RUNBOOK_ROOT,
          'corpus-pass-cats',
        );

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [underRunbookRoot,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect({
          logged: warningsIn({ lines: logged, },),
          printedWarnings: printed.lines.filter(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },).length,
        },).toEqual({
          logged: [
            unreachedRootWarning({
              roots: [
                join(root.path, 'node_modules', '.monochromatic',),
                runs.path,
                underRunbookRoot,
              ],
              home: HOME,
            },),
          ],
          printedWarnings: 1,
        },);
      },
    },),
    it({
      name: 'COUNTS A SEARCHED ROOT BY ITS REAL PATH: the home directory reached through a link and the real directory '
        + 'it points to are the same root, whichever one the audit was given and whichever one the home names',
      fn: async (ctx) => {
        await using real = await scratchDir({ prefix: 'cache-account-slice-real-', },);
        await using links = await scratchDir({ prefix: 'cache-account-slice-links-', },);
        await mkdir(
          join(real.path, 'temp', 'agent',),
          { recursive: true, },
        );
        /**
         A link to the real home directory, beside nothing it covers by name.
         */
        const link = join(
          links.path,
          'home-link',
        );
        await symlink(
          real.path,
          link,
          'dir',
        );

        const rootIsReal = await warningsOverHome({ searched: [real.path,], home: link, sinon: ctx.sinon, },);
        const rootIsLink = await warningsOverHome({ searched: [link,], home: real.path, sinon: ctx.sinon, },);
        expect({
          rootIsRealHomeIsLink: rootIsReal.warned,
          rootIsLinkHomeIsReal: rootIsLink.warned,
        },).toEqual({
          rootIsRealHomeIsLink: [],
          rootIsLinkHomeIsReal: [],
        },);
      },
    },),
    it({
      name: 'WARNS WHERE A SEARCHED ROOT HOLDS THE RUNBOOK\'S ROOT BEYOND THE DEPTH THE SEARCH WALKS: the home '
        + 'directory itself reaches the runs two levels under temp/agent, one level more of home above it does not, '
        + 'so the audit says the runs root was not reached',
      fn: async (ctx) => {
        await using searchedRoot = await scratchDir({ prefix: 'cache-account-slice-deep-', },);
        /**
         A home directory one level below the searched root, so the runbook's root lies three levels below it.
         */
        const deepHome = join(
          searchedRoot.path,
          'cats',
        );
        const searched = [searchedRoot.path,];
        const reached = await warningsOverHome({ searched, home: searchedRoot.path, sinon: ctx.sinon, },);
        const beyond = await warningsOverHome({ searched, home: deepHome, sinon: ctx.sinon, },);

        expect({
          reached: reached.warned,
          beyond: beyond.warned,
        },).toEqual({
          reached: [],
          beyond: [unreachedRootWarning({ roots: beyond.roots, home: deepHome, },),],
        },);
      },
    },),
    it({
      name: 'KEEPS THE ZERO-COUNT SENTENCE ALONE WHERE NO RECORD WAS READ AND THE RUNBOOK\'S ROOT IS SEARCHED: one '
        + 'warning, the sentence of the empty case, and no second one about the root',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [RUNBOOK_ROOT,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect({
          logged: warningsIn({ lines: logged, },),
          printedWarnings: printed.lines.filter(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },).length,
        },).toEqual({
          logged: [
            noRecordWarning({
              roots: [
                join(root.path, 'node_modules', '.monochromatic',),
                runs.path,
                RUNBOOK_ROOT,
              ],
            },),
          ],
          printedWarnings: 1,
        },);
      },
    },),
    it({
      name: 'READS EVERY RUNS DIRECTORY found under a searched directory and beside the default one, and names where '
        + 'it looked',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS - 100, },);

        /**
         A hand-set runs directory beside the default one, and one under a
         searched directory.
         */
        const beside = join(
          root.path,
          'node_modules',
          '.monochromatic',
          'translation-repair-runs-pass1',
        );
        await writeSliceRecord({ runsDir: beside, seconds: NEWEST_SECONDS - 50, },);
        const nested = join(
          under.path,
          'pass2',
        );
        const record = await writeSliceRecord({ runsDir: nested, seconds: NEWEST_SECONDS, },);

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [under.path,],
          settings: [settingAt({ name: 'STAGE_CACHE_VERSION', seconds: NEWEST_SECONDS, },),],
          // The runbook's runs root of this home lies under a directory searched.
          home: under.path,
        },);

        expect(printed.lines,).toEqual([
          `slice-cache records under 3 runs directories found in ${join(root.path, 'node_modules', '.monochromatic',)}, `
            + `${runs.path}, ${under.path}: 3`,
          'directories the search could not list, whose runs the count leaves out: 0',
          `newest slice-cache record: 2026-09-02T10:00Z ${record}`,
          'cache versions set after it: 0 of 1',
        ],);
      },
    },),
    it({
      name: 'LISTS EACH DIRECTORY THE SEARCH COULD NOT LIST with the reason, and still counts the records it could read',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);

        /**
         A directory nobody may list, under the searched one.
         */
        const sealed = join(
          under.path,
          'sealed',
        );
        await mkdir(sealed,);
        await chmod(
          sealed,
          0,
        );
        await using _reopened = {
          [Symbol.asyncDispose]: async function reopen(): Promise<void> {
            await chmod(
              sealed,
              0o700,
            );
          },
        };

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [under.path,],
          settings: [],
        },);

        expect(printed.lines.slice(
          0,
          3,
        ),).toEqual([
          `slice-cache records under 1 runs directory found in ${join(root.path, 'node_modules', '.monochromatic',)}, `
            + `${runs.path}, ${under.path}: 0`,
          'directories the search could not list, whose runs the count leaves out: 1',
          `  ${sealed} (EACCES)`,
        ],);
      },
    },),
  ],
},);

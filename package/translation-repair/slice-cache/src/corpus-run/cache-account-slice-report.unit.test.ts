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
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
  type DisposableSandbox,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  reportSliceCaches,
  type VersionSetting,
} from '../../dist/final/node/index.mjs';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeSliceRecord, } from './cache-account-repo.test-fixture.ts';
import { refusalSaid, } from './refusal-said.test-fixture.ts';
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
 A searched root whose real path could not be resolved, and the filesystem's code.

 @example
 ```ts
 const failure: RootFailure = { root: '/cats/file/below', code: 'ENOTDIR', };
 ```
 */
type RootFailure = {
  /**
   The root as it was given.
   */
  readonly root: string;

  /**
   Filesystem code, such as `ENOTDIR`.
   */
  readonly code: string;
};

/**
 The clause that names the roots the check could not resolve, which the warning ends with.

 @param failures - roots whose real path could not be resolved, each with the code

 @returns Clause with its leading space, or nothing where every root resolved

 @example
 ```ts
 const words = unresolvedRootsWords({ failures: [{ root: '/cats/file/below', code: 'ENOTDIR', },], },);
 ```
 */
function unresolvedRootsWords({ failures, }: { readonly failures: readonly RootFailure[]; },): string {
  if (failures.length === 0)
    return '';
  /**
   Each root with its code.
   */
  const named = failures.map(function withCode(
    {
      root,
      code,
    },
  ): string {
    return `${root} (${code})`;
  },);
  return ` Roots the check could not resolve, and so counted as not reaching it: ${named.join(', ',)}.`;
}

/**
 The line the audit logs for a root it could not resolve.

 @param root - root as it was given

 @param code - filesystem code of the failure

 @param said - text of the failure, as the filesystem words it

 @param home - home directory the runbook's runs root sits under

 @returns Warning line, without its level, with the tags of the functions that wrote it

 @example
 ```ts
 const line = unresolvedRootLine({
   root: '/cats/file/below',
   code: 'ENOTDIR',
   said: 'ENOTDIR: not a directory',
   home: '/cats/home',
 },);
 ```
 */
function unresolvedRootLine(
  {
    root,
    code,
    said,
    home,
  }: {
    readonly root: string;
    readonly code: string;
    readonly said: string;
    readonly home: string;
  },
): string {
  return `[warningWordsOf] [reachOfRoot] ${root} could not be compared with the runbook's runs root `
    + `${join(home, 'temp', 'agent',)} (${code}); it counts as not reaching it. The failure was: ${said}`;
}

/**
 What the audit says when it read records and no root it searched reaches the runbook's runs root: by its real path,
 within the depth the search walks and through no directory it skips.

 @param roots - every root it searched, in the order it printed them

 @param home - home directory the runbook's runs root sits under

 @param failures - roots whose real path could not be resolved, each with the code, none where every root resolved

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
    failures = [],
  }: {
    readonly roots: readonly string[];
    readonly home: string;
    readonly failures?: readonly RootFailure[];
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
  /**
   The sentence about the roots that were read.
   */
  const reached = `the runbook's runs root ${runbookRoot} is not reached by the roots searched: `
    + `${roots.join(', ',)}. A search `
    + 'reaches runs directories at most 4 levels below a root, and the runbook\'s runs sit up to 2 levels below its '
    + 'root; it enters no hidden directory or node_modules, and a directory counts by its real path. The newest record '
    + 'and the versions set after it speak only for those roots, and a launch that follows the runbook writes its '
    + `runs under ${runbookRoot}; pass --runs-under ${runbookRoot} after -- to read them.`;
  return `${reached}${unresolvedRootsWords({ failures, },)}`;
}

/**
 The line the audit logs once for the runbook's runs root when that root cannot be resolved.

 @param code - filesystem code of the failure

 @param said - text of the failure, as the filesystem words it

 @param home - home directory the runbook's runs root sits under

 @returns Warning line, without its level, with the tags of the functions that wrote it

 @example
 ```ts
 const line = unresolvedRunbookRootLine({ code: 'ENOTDIR', said: 'ENOTDIR: not a directory', home: '/cats/file', },);
 ```
 */
function unresolvedRunbookRootLine(
  {
    code,
    said,
    home,
  }: {
    readonly code: string;
    readonly said: string;
    readonly home: string;
  },
): string {
  return `[warningWordsOf] [runbookRootReading] the runbook's runs root ${join(home, 'temp', 'agent',)} could not be `
    + `resolved (${code}); no searched root can be compared with it. The failure was: ${said}`;
}

/**
 What the audit says when it read records and the runbook's runs root cannot be resolved, so no root it searched can be
 compared with it.

 @param roots - every root it searched, in the order it printed them

 @param home - home directory the runbook's runs root sits under

 @param code - filesystem code of the failure

 @returns Sentence, without the stdout prefix

 @example
 ```ts
 const words = unresolvedRunbookRootWarning({ roots: ['/cats/runs',], home: '/cats/file', code: 'ENOTDIR', },);
 ```
 */
function unresolvedRunbookRootWarning(
  {
    roots,
    home,
    code,
  }: {
    readonly roots: readonly string[];
    readonly home: string;
    readonly code: string;
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
  return `the runbook's runs root ${runbookRoot} could not be resolved (${code}), so none of the roots searched could `
    + `be compared with it: ${roots.join(', ',)}. The newest record and the versions set after it speak only for those `
    + `roots, and a launch that follows the runbook writes its runs under ${runbookRoot}; pass --runs-under `
    + `${runbookRoot} after -- to read them.`;
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

 @returns Setting

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
    it({
      name: 'KEEPS GOING AND WARNS WHERE A SEARCHED ROOT RUNS THROUGH A FILE (ENOTDIR): the root counts as not '
        + 'reaching the runbook\'s runs root, the warning says why, one line is logged with the root, and the newest '
        + 'record and the versions set after it are still printed',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const file = join(
          under.path,
          'file',
        );
        await writeFile(
          file,
          'meow',
          'utf8',
        );
        const belowFile = join(
          file,
          'below',
        );
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [belowFile,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect({
          logged: warningsIn({ lines: logged, },),
          printedWarnings: printed.lines.filter(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },),
          lastPrinted: printed.lines.at(-1,)?.startsWith('cache versions set after it: ',),
        },).toEqual({
          logged: [
            unresolvedRootLine({
              root: belowFile,
              code: 'ENOTDIR',
              said: await refusalSaid({ path: belowFile, },),
              home: HOME,
            },),
            unreachedRootWarning({
              roots: [
                join(root.path, 'node_modules', '.monochromatic',),
                runs.path,
                belowFile,
              ],
              home: HOME,
              failures: [{ root: belowFile, code: 'ENOTDIR', },],
            },),
          ],
          printedWarnings: [
            `WARNING: ${unreachedRootWarning({
              roots: [
                join(root.path, 'node_modules', '.monochromatic',),
                runs.path,
                belowFile,
              ],
              home: HOME,
              failures: [{ root: belowFile, code: 'ENOTDIR', },],
            },)}`,
          ],
          lastPrinted: true,
        },);
      },
    },),
    it({
      name: 'KEEPS GOING AND WARNS WHERE A SEARCHED ROOT IS A LOOP OF LINKS (ELOOP), with the same account of why',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const loopA = join(
          under.path,
          'loop-a',
        );
        const loopB = join(
          under.path,
          'loop-b',
        );
        await symlink(
          loopB,
          loopA,
        );
        await symlink(
          loopA,
          loopB,
        );
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [loopA,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect({
          logged: warningsIn({ lines: logged, },),
          lastPrinted: printed.lines.at(-1,)?.startsWith('cache versions set after it: ',),
        },).toEqual({
          logged: [
            unresolvedRootLine({
              root: loopA,
              code: 'ELOOP',
              said: await refusalSaid({ path: loopA, },),
              home: HOME,
            },),
            unreachedRootWarning({
              roots: [
                join(root.path, 'node_modules', '.monochromatic',),
                runs.path,
                loopA,
              ],
              home: HOME,
              failures: [{ root: loopA, code: 'ELOOP', },],
            },),
          ],
          lastPrinted: true,
        },);
      },
    },),
    it({
      name: 'KEEPS GOING AND WARNS WHERE A SEARCHED ROOT SITS BEHIND A DIRECTORY NOBODY MAY ENTER (EACCES), with the '
        + 'same account of why',
      skip: (process.getuid?.() === 0)
        ? 'the process is root, and no mode refuses root a directory'
        : false,
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
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
        const behindSealed = join(
          sealed,
          'below',
        );
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [behindSealed,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: HOME,
        },);

        expect({
          logged: warningsIn({ lines: logged, },),
          lastPrinted: printed.lines.at(-1,)?.startsWith('cache versions set after it: ',),
        },).toEqual({
          logged: [
            unresolvedRootLine({
              root: behindSealed,
              code: 'EACCES',
              said: await refusalSaid({ path: behindSealed, },),
              home: HOME,
            },),
            unreachedRootWarning({
              roots: [
                join(root.path, 'node_modules', '.monochromatic',),
                runs.path,
                behindSealed,
              ],
              home: HOME,
              failures: [{ root: behindSealed, code: 'EACCES', },],
            },),
          ],
          lastPrinted: true,
        },);
      },
    },),
    it({
      name: 'LOGS BUT DOES NOT WARN WHERE A ROOT THE CHECK COULD NOT RESOLVE STANDS BESIDE ONE THAT REACHES THE '
        + 'RUNBOOK\'S RUNS ROOT: the reaching root decides, and the unresolved one leaves its line in the log',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const file = join(
          under.path,
          'file',
        );
        await writeFile(
          file,
          'meow',
          'utf8',
        );
        const belowFile = join(
          file,
          'below',
        );
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [belowFile, RUNBOOK_ROOT,],
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
          warned: [
            unresolvedRootLine({
              root: belowFile,
              code: 'ENOTDIR',
              said: await refusalSaid({ path: belowFile, },),
              home: HOME,
            },),
          ],
          printedWarning: false,
        },);
      },
    },),
    it({
      name: 'NAMES THE RUNBOOK\'S RUNS ROOT ONCE WHERE IT IS THE ROOT THAT CANNOT BE RESOLVED: no searched root is '
        + 'blamed, one line is logged with the runbook\'s root, and the newest record and the versions set after it '
        + 'are still printed',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'cache-account-slice-root-', },);
        await using runs = await scratchDir({ prefix: 'cache-account-slice-runs-', },);
        await using under = await scratchDir({ prefix: 'cache-account-slice-under-', },);
        await using beside = await scratchDir({ prefix: 'cache-account-slice-beside-', },);
        await writeSliceRecord({ runsDir: runs.path, seconds: NEWEST_SECONDS, },);
        const homeFile = join(
          under.path,
          'home-file',
        );
        await writeFile(
          homeFile,
          'meow',
          'utf8',
        );
        const runbookRoot = join(
          homeFile,
          'temp',
          'agent',
        );
        const roots = [
          join(root.path, 'node_modules', '.monochromatic',),
          runs.path,
          under.path,
          beside.path,
        ];
        const logged: string[] = [];

        await reportSliceCaches({
          root: root.path,
          runsDir: runs.path,
          searched: [under.path, beside.path,],
          settings: [],
          l: levelCapturingLogger({ lines: logged, },),
          home: homeFile,
        },);

        expect({
          logged: warningsIn({ lines: logged, },),
          printedWarnings: printed.lines.filter(function isWarning(line,): boolean {
            return line.startsWith('WARNING: ',);
          },),
          lastPrinted: printed.lines.at(-1,)?.startsWith('cache versions set after it: ',),
        },).toEqual({
          logged: [
            unresolvedRunbookRootLine({
              code: 'ENOTDIR',
              said: await refusalSaid({ path: runbookRoot, },),
              home: homeFile,
            },),
            unresolvedRunbookRootWarning({ roots, home: homeFile, code: 'ENOTDIR', },),
          ],
          printedWarnings: [
            `WARNING: ${unresolvedRunbookRootWarning({ roots, home: homeFile, code: 'ENOTDIR', },)}`,
          ],
          lastPrinted: true,
        },);
      },
    },),
  ],
},);

/**
 Tests for the pre-launch cache check's command at its boundary.

 The built command is run as a child process whose environment carries no
 provider key, inside a throwaway repository the cases build commit by commit,
 over runs directories the cases write into scratch directories, so nothing
 here reads this repository's history or a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  auditCacheAccounts,
  citedHash,
  utcMinutes,
} from '../../dist/final/node/index.mjs';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { fixtureGit, } from '@monochromatic-dev/translation-repair-adjudication/ts/hermetic-git-run.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SLICE_CACHE_ENTRY, } from './built-entry.test-fixture.ts';
import {
  commitFileAt,
  makeCacheAccountRepo,
  writeSliceRecord,
} from './cache-account-repo.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Variables keeping the child's git from reading the user's configuration.
 */
const HERMETIC_GIT = {
  GIT_CONFIG_GLOBAL: devNull,
  GIT_CONFIG_SYSTEM: devNull,
};

/**
 What the command printed, apart: the report's lines in the order written, and
 the logger's progress lines with their stamps taken out, in code-point order
 since versions are read together and log in the order git answers.

 @param stdout - everything the command wrote to stdout

 @returns Both parts

 @example
 ```ts
 const { report, progress, } = partsOf({ stdout, },);
 ```
 */
function partsOf({ stdout, }: { readonly stdout: string; },): {
  readonly report: readonly string[];
  readonly progress: readonly string[];
} {
  /**
   Every line, the empty one after the last newline gone.
   */
  const lines = stdout
    .split('\n',)
    .slice(
    0,
    -1,
    );
  return {
    report: lines.filter(function isReport(line,): boolean {
      return !line.startsWith('[info] [',);
    },),
    progress: textsInCodePointOrder({
      texts: lines
        .filter(function isProgress(line,): boolean {
          return line.startsWith('[info] [',);
        },)
        .map(function withoutStamp(line,): string {
          return `[info] [stamp${line.slice(line.indexOf('] ', 8,),)}`;
        },),
    },),
  };
}

await describe({
  name: 'cache-account-audit run',
  concurrency: 1,
  children: [
    describe({
      name: auditCacheAccounts.name,
      children: [
        it({
          name: 'PRINTS THE REPORT OF A REPOSITORY, each version in the order git lists its file, the commits since '
            + 'the earliest that no account names, and the slice caches',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using repo = await makeCacheAccountRepo({ versions: 'two', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);
            const record = await writeSliceRecord({ runsDir: runs.path, seconds: repo.betweenSeconds, },);
            const [pairing,] = repo.pairingSet;
            if (pairing === undefined)
              throw new Error('unreachable: the repository was built with its pairing version',);

            await auditCacheAccounts({
              line: lineOf({ command: 'cache-account-audit', typed: [], },),
              packageDirectory: repo.path,
              runsDir: runs.path,
              // The runbook's runs root of this home lies under the runs directory, which is a root searched.
              home: runs.path,
            },);

            expect(printed.lines.filter(function isReport(line,): boolean {
              return !line.startsWith('[info] [',);
            },),).toEqual([
              'cache-account-audit: 2 cache versions under src',
              `  PAIRING_CACHE_VERSION = 4 (src/pairing-cache-version.ts): set in ${citedHash({ hash: pairing.hash, },)} `
                + `at ${utcMinutes({ seconds: pairing.seconds, },)}`,
              `  STAGE_CACHE_VERSION = 1 (src/stage-cache-version.ts): set in ${citedHash({ hash: repo.stageSet.hash, },)} `
                + `at ${utcMinutes({ seconds: repo.stageSet.seconds, },)}`,
              `source commits since ${citedHash({ hash: repo.stageSet.hash, },)}: 3, named by an account: 1, by none: 2`,
              `  ${citedHash({ hash: repo.purrAdded.hash, },)} ${utcMinutes({ seconds: repo.purrAdded.seconds, },)} `
                + `[rides inside: PAIRING_CACHE_VERSION, STAGE_CACHE_VERSION] ${repo.purrAdded.subject}`,
              `  ${citedHash({ hash: pairing.hash, },)} ${utcMinutes({ seconds: pairing.seconds, },)} `
                + `[rides inside: STAGE_CACHE_VERSION] ${pairing.subject}`,
              `slice-cache records under 1 runs directory found in ${join(repo.path, 'node_modules', '.monochromatic',)}, `
                + `${runs.path}: 1`,
              'directories the search could not list, whose runs the count leaves out: 0',
              `newest slice-cache record: ${utcMinutes({ seconds: repo.betweenSeconds, },)} ${record}`,
              'cache versions set after it: 1 of 2',
            ],);
          },
        },),
        it({
          name: 'PRINTS "1 cache version" in the singular where the source declares one',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using repo = await makeCacheAccountRepo({ versions: 'one', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);

            await auditCacheAccounts({
              line: lineOf({ command: 'cache-account-audit', typed: [], },),
              packageDirectory: repo.path,
              runsDir: runs.path,
            },);

            expect(printed.lines.filter(function opensReport(line,): boolean {
              return line.startsWith('cache-account-audit: ',) || line.includes('source files under',);
            },),).toEqual(['cache-account-audit: 1 cache version under src',],);
          },
        },),
        it({
          name: 'READS THE SOURCE UNDER THE PACKAGE DIRECTORY\'S OWN PREFIX where the package sits below the '
            + 'repository\'s top',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);
            await commitFileAt({
              cloneDir: repo.path,
              day: 5,
              path: 'package/cat/src/whisker-cache-version.ts',
              text: 'export const WHISKER_CACHE_VERSION = 3;\n',
              subject: 'set the whisker cache version',
            },);

            await auditCacheAccounts({
              line: lineOf({ command: 'cache-account-audit', typed: [], },),
              packageDirectory: join(
                repo.path,
                'package',
                'cat',
              ),
              runsDir: runs.path,
            },);

            expect({
              heading: printed.lines.filter(function opensReport(line,): boolean {
                return line.startsWith('cache-account-audit: ',);
              },),
              setting: printed.lines
                .filter(function namesWhisker(line,): boolean {
                  return line.startsWith('  WHISKER',);
                },)
                .map(function beforeHash(line,): string {
                  return line.slice(
                    0,
                    line.indexOf(': set in',),
                  );
                },),
            },).toEqual({
              heading: ['cache-account-audit: 1 cache version under package/cat/src',],
              setting: ['  WHISKER_CACHE_VERSION = 3 (package/cat/src/whisker-cache-version.ts)',],
            },);
          },
        },),
        it({
          name: 'REFUSES A SOURCE THAT DECLARES NO CACHE VERSION as stated, naming the source and where to run from',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);

            /**
             What the audit rejected with.
             */
            const refusal = await rejectionOf({
              promise: auditCacheAccounts({
                line: lineOf({ command: 'cache-account-audit', typed: [], },),
                packageDirectory: repo.path,
                runsDir: runs.path,
              },),
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: no cache version is declared under src; run this from the package directory, '
                + 'as mise run //package/translation-repair/slice-cache:cache-account-audit does.',
            );
          },
        },),
        it({
          name: 'REFUSES A VALUE NO COMMIT SETS as stated, so a version edited and not committed stops the audit',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'one', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);
            await writeFile(
              join(
                repo.path,
                'src',
                'stage-cache-version.ts',
              ),
              'export const STAGE_CACHE_VERSION = 7;\n',
              'utf8',
            );

            /**
             What the audit rejected with.
             */
            const refusal = await rejectionOf({
              promise: auditCacheAccounts({
                line: lineOf({ command: 'cache-account-audit', typed: [], },),
                packageDirectory: repo.path,
                runsDir: runs.path,
              },),
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: STAGE_CACHE_VERSION = 7 in src/stage-cache-version.ts is not in any commit; '
                + 'commit it, then run the audit again.',
            );
          },
        },),
        it({
          name: 'REFUSES A TRACKED FILE MISSING FROM THE WORKING TREE as stated, naming it, rather than failing as a fault',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'one', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);
            await rm(join(
              repo.path,
              'src',
              'nap.ts',
            ),);

            /**
             What the audit rejected with.
             */
            const refusal = await rejectionOf({
              promise: auditCacheAccounts({
                line: lineOf({ command: 'cache-account-audit', typed: [], },),
                packageDirectory: repo.path,
                runsDir: runs.path,
              },),
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: src/nap.ts is tracked but missing from the working tree; restore it or '
                + 'commit its removal, then run the audit again.',
            );
          },
        },),
        it({
          name: 'LETS A READ FAILURE THAT IS NOT A MISSING FILE ESCAPE as the fault it is: a directory where a tracked '
            + 'file stood',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'one', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);
            await rm(join(
              repo.path,
              'src',
              'nap.ts',
            ),);
            await mkdir(join(
              repo.path,
              'src',
              'nap.ts',
            ),);

            /**
             What the audit rejected with.
             */
            const failure = await rejectionOf({
              promise: auditCacheAccounts({
                line: lineOf({ command: 'cache-account-audit', typed: [], },),
                packageDirectory: repo.path,
                runsDir: runs.path,
              },),
            },);

            expect(failure instanceof StatedRefusalError,).toBe(false,);
            expect(String(failure,),).toBe(
              `Error: EISDIR: illegal operation on a directory, read '${join(repo.path, 'src', 'nap.ts',)}'`,
            );
          },
        },),
        it({
          name: 'REFUSES A CACHE VERSION DECLARED IN A FORM IT CANNOT READ as stated, with the line and how to write it, '
            + 'rather than failing as a fault',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'one', },);
            await using runs = await scratchDir({ prefix: 'cache-account-run-runs-', },);
            await commitFileAt({
              cloneDir: repo.path,
              day: 5,
              path: 'src/whisker-cache-version.ts',
              text: 'export const WHISKER_CACHE_VERSION: number = 3;\n',
              subject: 'set the whisker cache version',
            },);

            /**
             What the audit rejected with.
             */
            const refusal = await rejectionOf({
              promise: auditCacheAccounts({
                line: lineOf({ command: 'cache-account-audit', typed: [], },),
                packageDirectory: repo.path,
                runsDir: runs.path,
              },),
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'CacheAccountReadError: src/whisker-cache-version.ts declares a cache version the audit cannot read: '
                + '"export const WHISKER_CACHE_VERSION: number = 3;". Write it as NAME = digits; so every constant is checked.',
            );
          },
        },),
      ],
    },),

    describe({
      name: 'cache-account-audit as built',
      children: [
        it({
          name: 'PRINTS EACH VERSION AND THE COMMITS NO ACCOUNT NAMES, then the slice caches, and exits 0',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'two', },);
            await using runs = await scratchDir({ prefix: 'cache-account-built-runs-', },);
            await using under = await scratchDir({ prefix: 'cache-account-built-under-', },);

            /**
             The repository's top directory as git names it, which the
             command's own reading of it is compared with.
             */
            const top = (await fixtureGit({ cloneDir: repo.path, args: ['rev-parse', '--show-toplevel',], },)).trim();

            /**
             The slice record the default runs directory holds, newest of the
             two, written between the edit and the pairing version's commit.
             */
            const record = await writeSliceRecord({ runsDir: runs.path, seconds: repo.betweenSeconds, },);
            await writeSliceRecord({
              runsDir: join(
                under.path,
                'pass1',
              ),
              seconds: repo.stageSet.seconds,
            },);
            const [pairing,] = repo.pairingSet;
            if (pairing === undefined)
              throw new Error('unreachable: the repository was built with its pairing version',);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cache-account-audit',
              packageEntry: SLICE_CACHE_ENTRY,
              args: [
                '--runs-under',
                under.path,
              ],
              env: {
                ...HERMETIC_GIT,
                TRANSLATION_REPAIR_RUNS_DIR: runs.path,
                // The runbook's runs root of this home lies under the directory searched, so the run warns of nothing.
                HOME: under.path,
              },
              cwd: top,
            },);

            expect(run.stderr,).toBe('',);
            expect(partsOf({ stdout: run.stdout, },),).toEqual({
              progress: textsInCodePointOrder({
                texts: [
                  '[info] [stamp] [cache-account-audit] 4 source files under src, 2 cache versions',
                  `[info] [stamp] [cache-account-audit] STAGE_CACHE_VERSION = 1: set in ${citedHash({ hash: repo.stageSet.hash, },)}, 1 pickaxe candidate`,
                  `[info] [stamp] [cache-account-audit] PAIRING_CACHE_VERSION = 4: set in ${citedHash({ hash: pairing.hash, },)}, 1 pickaxe candidate`,
                  '[info] [stamp] [cache-account-slice-report] runs directories to read: 2; directories not listed: 0',
                  '[info] [stamp] [cache-account-slice-report] slice-cache records read: 2',
                ],
              },),
              report: [
                'cache-account-audit: 2 cache versions under src',
                `  PAIRING_CACHE_VERSION = 4 (src/pairing-cache-version.ts): set in ${citedHash({ hash: pairing.hash, },)} `
                  + `at ${utcMinutes({ seconds: pairing.seconds, },)}`,
                `  STAGE_CACHE_VERSION = 1 (src/stage-cache-version.ts): set in ${citedHash({ hash: repo.stageSet.hash, },)} `
                  + `at ${utcMinutes({ seconds: repo.stageSet.seconds, },)}`,
                `source commits since ${citedHash({ hash: repo.stageSet.hash, },)}: 3, named by an account: 1, by none: 2`,
                `  ${citedHash({ hash: repo.purrAdded.hash, },)} ${utcMinutes({ seconds: repo.purrAdded.seconds, },)} `
                  + `[rides inside: PAIRING_CACHE_VERSION, STAGE_CACHE_VERSION] ${repo.purrAdded.subject}`,
                `  ${citedHash({ hash: pairing.hash, },)} ${utcMinutes({ seconds: pairing.seconds, },)} `
                  + `[rides inside: STAGE_CACHE_VERSION] ${pairing.subject}`,
                `slice-cache records under 2 runs directories found in ${join(top, 'node_modules', '.monochromatic',)}, `
                  + `${runs.path}, ${under.path}: 2`,
                'directories the search could not list, whose runs the count leaves out: 0',
                `newest slice-cache record: ${utcMinutes({ seconds: repo.betweenSeconds, },)} ${record}`,
                'cache versions set after it: 1 of 2',
              ],
            },);
            expect(run.code,).toBe(0,);
          },
        },),
        it({
          name: 'REFUSES A DIRECTORY THAT DECLARES NO CACHE VERSION as stated and exits 6 with its line, which names '
            + 'where to run it from',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await using runs = await scratchDir({ prefix: 'cache-account-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cache-account-audit',
              packageEntry: SLICE_CACHE_ENTRY,
              args: [],
              env: {
                ...HERMETIC_GIT,
                TRANSLATION_REPAIR_RUNS_DIR: runs.path,
              },
              cwd: repo.path,
            },);

            expect(partsOf({ stdout: run.stdout, },),).toEqual({
              progress: ['[info] [stamp] [cache-account-audit] 2 source files under src, 0 cache versions',],
              report: [],
            },);
            expect(run.stderr,).toBe(
              'cache-account-audit: no cache version is declared under src; run this from the package directory, '
                + 'as mise run //package/translation-repair/slice-cache:cache-account-audit does.\n',
            );
            expect(run.code,).toBe(REFUSED_AS_STATED,);
          },
        },),
        it({
          name: 'REFUSES A FLAG WRITTEN WITH NO VALUE as stated and exits 6 with its line, before asking git anything',
          fn: async () => {
            await using runs = await scratchDir({ prefix: 'cache-account-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cache-account-audit',
              packageEntry: SLICE_CACHE_ENTRY,
              args: ['--runs-under',],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'cache-account-audit: --runs-under needs a value written after it. '
                + 'Usage: cache-account-audit [--runs-under <directory holding runs directories> ...]\n',
            );
            expect(run.code,).toBe(REFUSED_AS_STATED,);
          },
        },),
        it({
          name: 'REFUSES A FLAG IT DOES NOT DECLARE as stated and exits 6 with its line',
          fn: async () => {
            await using runs = await scratchDir({ prefix: 'cache-account-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cache-account-audit',
              packageEntry: SLICE_CACHE_ENTRY,
              args: ['--tabby',],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'cache-account-audit: --tabby is not a flag this command reads. '
                + 'Usage: cache-account-audit [--runs-under <directory holding runs directories> ...]\n',
            );
            expect(run.code,).toBe(REFUSED_AS_STATED,);
          },
        },),
      ],
    },),
  ],
},);

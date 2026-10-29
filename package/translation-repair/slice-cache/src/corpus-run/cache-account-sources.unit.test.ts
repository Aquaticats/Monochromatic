/**
 Tests for which source files the pre-launch cache check reads: the package's
 own `src` where the package stands alone, and the `src` of every package of
 the translation-repair family where it stands in the family's directory,
 since a cache version now lives in whichever package declares the constant.

 Each case builds a throwaway repository, so nothing here reads this
 repository's history. Fixtures are cat-themed invention.

 @module
 */

import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  auditCacheAccounts,
  auditedSourcesOf,
  citedHash,
  preSplitSourcesOf,
  preSplitWithoutTestsOf,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  commitFileAt,
  makeCacheAccountRepo,
  moveFileAt,
} from './cache-account-repo.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Where the family's packages sit in the repository.
 */
const FAMILY = 'package/translation-repair';

/**
 Adds two family packages to a repository: one declaring a cache version, one
 declaring another and holding a test file, and a package outside the family
 declaring a third.

 @param cloneDir - repository directory

 @example
 ```ts
 await addFamily({ cloneDir: repo.path, },);
 ```
 */
async function addFamily({ cloneDir, }: { readonly cloneDir: string; },): Promise<void> {
  await commitFileAt({
    cloneDir,
    day: 5,
    path: `${FAMILY}/alpha/src/alpha-cache-version.ts`,
    text: 'export const ALPHA_CACHE_VERSION = 2;\n',
    subject: 'set the alpha cache version',
  },);
  await commitFileAt({
    cloneDir,
    day: 6,
    path: `${FAMILY}/beta/src/beta-cache-version.ts`,
    text: 'export const BETA_CACHE_VERSION = 5;\n',
    subject: 'set the beta cache version',
  },);
  await commitFileAt({
    cloneDir,
    day: 7,
    path: `${FAMILY}/beta/src/beta.unit.test.ts`,
    text: 'export const BETA_TEST_CACHE_VERSION = 9;\n',
    subject: 'test beta',
  },);
  await commitFileAt({
    cloneDir,
    day: 8,
    path: 'package/stray/src/stray-cache-version.ts',
    text: 'export const STRAY_CACHE_VERSION = 1;\n',
    subject: 'set the stray cache version',
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: auditedSourcesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE SOURCE OF EVERY PACKAGE OF THE FAMILY where the package sits in the family directory, '
            + 'leaving out tests and what stands outside the family',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await addFamily({ cloneDir: repo.path, },);

            expect(await auditedSourcesOf({
              packageDirectory: join(
                repo.path,
                FAMILY,
                'alpha',
              ),
            },),).toEqual({
              sources: `${FAMILY}/*/src`,
              files: [
                `${FAMILY}/alpha/src/alpha-cache-version.ts`,
                `${FAMILY}/beta/src/beta-cache-version.ts`,
              ],
            },);
          },
        },),
        it({
          name: 'LISTS ONLY THE PACKAGE\'S OWN SOURCE where the package stands outside the family directory',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await addFamily({ cloneDir: repo.path, },);

            expect(await auditedSourcesOf({
              packageDirectory: join(
                repo.path,
                'package',
                'stray',
              ),
            },),).toEqual({
              sources: 'package/stray/src',
              files: ['package/stray/src/stray-cache-version.ts',],
            },);
          },
        },),
      ],
    },),
    describe({
      name: preSplitSourcesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE PRE-SPLIT SOURCE DIRECTORY where the sources are the family\'s, in either form the audit '
            + 'writes them',
          fn: async () => {
            expect([
              `${FAMILY}/*/src`,
              `:(glob)${FAMILY}/*/src/**`,
            ].map((sources,) => preSplitSourcesOf({ sources, },)),).toEqual([
              ['package/module/translation-repair/src',],
              ['package/module/translation-repair/src',],
            ],);
          },
        },),
        it({
          name: 'NAMES NONE where the sources are one package\'s own, whose sources never stood there',
          fn: async () => {
            expect(preSplitSourcesOf({ sources: 'package/stray/src', },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: preSplitWithoutTestsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES THE TESTS OF THE PRE-SPLIT SOURCE DIRECTORY OUT where the sources are the family\'s',
          fn: async () => {
            expect(preSplitWithoutTestsOf({ sources: `${FAMILY}/*/src`, },),).toEqual([
              ':(exclude,glob)package/module/translation-repair/src/**/*.test.ts',
            ],);
          },
        },),
        it({
          name: 'LEAVES NOTHING OUT where the sources are one package\'s own',
          fn: async () => {
            expect(preSplitWithoutTestsOf({ sources: 'package/stray/src', },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: `${auditCacheAccounts.name} over the family`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE CACHE VERSIONS OF EVERY FAMILY PACKAGE and names the family\'s source directories in its heading',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await using runs = await scratchDir({ prefix: 'cache-account-sources-runs-', },);
            await addFamily({ cloneDir: repo.path, },);

            await auditCacheAccounts({
              line: lineOf({ command: 'cache-account-audit', typed: [], },),
              packageDirectory: join(
                repo.path,
                FAMILY,
                'beta',
              ),
              runsDir: runs.path,
            },);

            expect({
              heading: printed.lines.filter(function opensReport(line,): boolean {
                return line.startsWith('cache-account-audit: ',);
              },),
              settings: printed.lines
                .filter(function namesVersion(line,): boolean {
                  return line.startsWith('  ALPHA',) || line.startsWith('  BETA',);
                },)
                .map(function beforeHash(line,): string {
                  return line.slice(
                    0,
                    line.indexOf(': set in',),
                  );
                },),
            },).toEqual({
              heading: [`cache-account-audit: 2 cache versions under ${FAMILY}/*/src`,],
              settings: [
                `  ALPHA_CACHE_VERSION = 2 (${FAMILY}/alpha/src/alpha-cache-version.ts)`,
                `  BETA_CACHE_VERSION = 5 (${FAMILY}/beta/src/beta-cache-version.ts)`,
              ],
            },);
          },
        },),
        it({
          name: 'NAMES THE COMMIT THAT SET A VERSION BEFORE THE SPLIT as its setting, and reads the source commits '
            + 'from it, not from the commit that moved the file into the family',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using repo = await makeCacheAccountRepo({ versions: 'none', },);
            await using runs = await scratchDir({ prefix: 'cache-account-sources-runs-', },);
            const setBeforeSplit = await commitFileAt({
              cloneDir: repo.path,
              day: 5,
              path: 'package/module/translation-repair/src/gamma-cache-version.ts',
              text: 'export const GAMMA_CACHE_VERSION = 3;\n',
              subject: 'set the gamma cache version',
            },);
            await moveFileAt({
              cloneDir: repo.path,
              day: 6,
              from: 'package/module/translation-repair/src/gamma-cache-version.ts',
              to: `${FAMILY}/gamma/src/gamma-cache-version.ts`,
              subject: 'split the translation-repair package',
            },);

            await auditCacheAccounts({
              line: lineOf({ command: 'cache-account-audit', typed: [], },),
              packageDirectory: join(
                repo.path,
                FAMILY,
                'gamma',
              ),
              runsDir: runs.path,
            },);

            expect({
              settings: printed.lines
                .filter(function namesVersion(line,): boolean {
                  return line.startsWith('  GAMMA',);
                },)
                .map(function beforeTime(line,): string {
                  return line.slice(
                    0,
                    line.indexOf(' at ',),
                  );
                },),
              since: printed.lines.filter(function opensCount(line,): boolean {
                return line.startsWith('source commits since ',);
              },),
            },).toEqual({
              settings: [
                `  GAMMA_CACHE_VERSION = 3 (${FAMILY}/gamma/src/gamma-cache-version.ts): set in ${
                  citedHash({ hash: setBeforeSplit.hash, },)
                }`,
              ],
              since: [
                `source commits since ${citedHash({ hash: setBeforeSplit.hash, },)}: 1, named by an account: 0, by none: 1`,
              ],
            },);
          },
        },),
      ],
    },),
  ],
},);

/**
 Tests for how the pre-launch cache check reads on a terminal: each version
 with the commit that set it, then the commits no account names and the
 versions each one rides inside.

 What is printed is read off `console.log` through a diverting capture, which
 is process-wide, so the suite runs one case at a time.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  printCacheAudit,
  type SourceCommit,
  type VersionSetting,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Commit that set the stage version, 2026-09-01T10:00Z.
 */
const STAGE_COMMIT: SourceCommit = {
  hash: 'b1f20a6e4aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  seconds: 1_788_256_800,
  subject: 'set the stage cache version',
};

/**
 Commit that set the pairing version, a day later.
 */
const PAIRING_COMMIT: SourceCommit = {
  hash: 'a841292adbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
  seconds: 1_788_343_200,
  subject: 'set the pairing cache version',
};

/**
 Commit after both, which no account names.
 */
const PURR_COMMIT: SourceCommit = {
  hash: 'ef61f1be4cccccccccccccccccccccccccccccc',
  seconds: 1_788_429_600,
  subject: 'teach Tabby to purr',
};

/**
 The stage version's setting, whose account names nothing.
 */
const STAGE_SETTING: VersionSetting = {
  version: {
    name: 'STAGE_CACHE_VERSION',
    value: 1,
    path: 'src/stage-cache-version.ts',
    declaration: 'STAGE_CACHE_VERSION = 1',
  },
  commit: STAGE_COMMIT,
  account: 'export const STAGE_CACHE_VERSION = 1;\n',
};

/**
 The pairing version's setting, whose account names the commit that purred.
 */
const PAIRING_SETTING: VersionSetting = {
  version: {
    name: 'PAIRING_CACHE_VERSION',
    value: 4,
    path: 'src/pairing-cache-version.ts',
    declaration: 'PAIRING_CACHE_VERSION = 4',
  },
  commit: PAIRING_COMMIT,
  account: '// Commit ef61f1be4 taught Tabby to purr, which the pairing does not read.\nconst PAIRING_CACHE_VERSION = 4;\n',
};

await describe({
  name: printCacheAudit.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS EACH VERSION WITH ITS SETTING, then the commits since the earliest that no account names, '
        + 'each with the versions it rides inside',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCacheAudit({
          sources: 'src',
          settings: [STAGE_SETTING, PAIRING_SETTING,],
          earliest: STAGE_SETTING,
          commits: [
            {
              hash: 'd00d00d00dddddddddddddddddddddddddddddd',
              seconds: 1_788_429_700,
              subject: 'teach Tabby to knead',
            },
            PAIRING_COMMIT,
          ],
        },);

        expect(printed.lines,).toEqual([
          'cache-account-audit: 2 cache versions under src',
          '  STAGE_CACHE_VERSION = 1 (src/stage-cache-version.ts): set in b1f20a6e4 at 2026-09-01T10:00Z',
          '  PAIRING_CACHE_VERSION = 4 (src/pairing-cache-version.ts): set in a841292ad at 2026-09-02T10:00Z',
          'source commits since b1f20a6e4: 2, named by an account: 0, by none: 2',
          '  d00d00d00 2026-09-03T10:01Z [rides inside: STAGE_CACHE_VERSION, PAIRING_CACHE_VERSION] teach Tabby to knead',
          '  a841292ad 2026-09-02T10:00Z [rides inside: STAGE_CACHE_VERSION] set the pairing cache version',
        ],);
      },
    },),
    it({
      name: 'COUNTS A COMMIT AN ACCOUNT NAMES as named, and lists no commit where every one is named',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCacheAudit({
          sources: 'src',
          settings: [PAIRING_SETTING,],
          earliest: PAIRING_SETTING,
          commits: [PURR_COMMIT,],
        },);

        expect(printed.lines,).toEqual([
          'cache-account-audit: 1 cache version under src',
          '  PAIRING_CACHE_VERSION = 4 (src/pairing-cache-version.ts): set in a841292ad at 2026-09-02T10:00Z',
          'source commits since a841292ad: 1, named by an account: 1, by none: 0',
        ],);
      },
    },),
    it({
      name: 'PRINTS NO COMMIT LINE where no commit follows the earliest setting',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCacheAudit({
          sources: 'src',
          settings: [STAGE_SETTING,],
          earliest: STAGE_SETTING,
          commits: [],
        },);

        expect(printed.lines.slice(2,),).toEqual(['source commits since b1f20a6e4: 0, named by an account: 0, by none: 0',],);
      },
    },),
  ],
},);

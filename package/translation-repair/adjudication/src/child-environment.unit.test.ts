/**
 Tests for the environment every test's child process gets: no variable whose
 name ends in `_API_KEY`, none whose name starts with `TRANSLATION_REPAIR_`,
 a log directory inside the workspace, and the case's own variables.

 THE PROOF IS A REAL CHILD. Each case sets fake variables in this process
 (`TRANSLATION_REPAIR_NEVER_REAL_API_KEY` and its kin, never a real key),
 starts a node child that prints the names it sees, and compares them; the
 child never calls a provider. The suite runs one case at a time, since the
 fake variables are process-wide writes.

 @module
 */

import { randomUUID, } from 'node:crypto';
import type { Dirent, } from 'node:fs';
import {
  readdir,
  readFile,
  realpath,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  environmentWithoutKeys,
  runKeyless,
  spawnKeyless,
  variablesSet,
} from './child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Fake variables this process holds while a case runs, none a real credential:
 two keys (one carrying the package's prefix and one not), one setting of
 the package's, and one plain variable the rule leaves alone.
 */
const FAKE_VARIABLES: Readonly<Record<string, string>> = {
  TRANSLATION_REPAIR_NEVER_REAL_API_KEY: 'a cat naps',
  NEVER_REAL_ELSEWHERE_API_KEY: 'a kitten naps',
  TRANSLATION_REPAIR_NEVER_REAL_SETTING: 'a tabby naps',
  NEVER_REAL_PLAIN: 'a calico naps',
};

/**
 Program a child runs to say what it sees of its own environment: the names
 ending in the key suffix, the names starting with the package's prefix, the
 two variables the case set by hand, whether the inherited search path is
 there, and the directory it runs in.
 */
const REPORT_PROGRAM = [
  'const names = Object.keys(process.env).toSorted();',
  'process.stdout.write(JSON.stringify({',
  '  keys: names.filter((name) => name.endsWith(\'_API_KEY\')),',
  '  settings: names.filter((name) => name.startsWith(\'TRANSLATION_REPAIR_\')),',
  '  plain: process.env.NEVER_REAL_PLAIN ?? null,',
  '  extra: process.env.NEVER_REAL_EXTRA ?? null,',
  '  hasPath: names.includes(\'PATH\'),',
  '  cwd: process.cwd(),',
  '}));',
].join('\n',);

/**
 Where a child's logger writes unless the case names another directory: this
 package's gitignored `node_modules/.monochromatic`, inside the workspace.
 */
const CHILD_LOG_DIRECTORY = fileURLToPath(new URL(
  '../node_modules/.monochromatic',
  import.meta.url,
),);

/**
 Program a child runs to log one marker line through the logger a built
 package bundles, at a level its console sink keeps quiet, so the line reaches
 the file sink alone, and to wait for the file to hold it.

 @param marker - text no other record holds

 @returns Module source for `node --input-type=module --eval`

 @example
 ```ts
 const source = loggingProgram({ marker: randomUUID(), },);
 ```
 */
function loggingProgram({ marker, }: { readonly marker: string; },): string {
  /**
   The built package whose logger the child writes through, by absolute URL so
   the child's working directory can be anywhere.
   */
  const built = import.meta.resolve('@monochromatic-dev/translation-repair-typography',);
  return [
    `const { contextRoot } = await import(${JSON.stringify(built,)});`,
    'const l = contextRoot({ tag: "whiskers" });',
    `l.debug(${JSON.stringify(marker,)});`,
    'await l.flush();',
  ].join('\n',);
}

/**
 What the report program printed, read back.

 @param text - child's whole stdout

 @returns Report as plain data

 @example
 ```ts
 const report = reportOf({ text: stdout, },);
 ```
 */
function reportOf({ text, }: { readonly text: string; },): unknown {
  return JSON.parse(text,);
}

/**
 Whether a directory entry is a log file the file sink writes.

 @param entry - one entry of a log directory listing

 @returns Whether the entry names a log file

 @example
 ```ts
 const mine = entries.filter(isLogFileEntry,);
 ```
 */
function isLogFileEntry(entry: Dirent,): boolean {
  return entry.isFile() && entry.name.endsWith('.log.jsonl',);
}

/**
 Reads every log file under a workspace log directory, where the logger nests
 per-process files under UTC day directories beside its retention stamp.

 @param directory - workspace log directory a child wrote into

 @returns Contents of each log file found, one entry per file

 @example
 ```ts
 const texts = await readLogTexts({ directory: CHILD_LOG_DIRECTORY, });
 ```
 */
async function readLogTexts(
  { directory, }: { readonly directory: string; },
): Promise<string[]> {
  /**
   One listing level of the workspace log directory.
   */
  const entries = await readdir(directory, { withFileTypes: true, },);
  /**
   Contents gathered per root entry, flattened into one list.
   */
  const groups = await Promise.all(entries.map(async function readEntryTexts(entry,) {
    if (entry.isFile() && entry.name.endsWith('.log.jsonl',))
      return [await readFile(join(directory, entry.name,), 'utf8',),];
    if (entry.isDirectory()) {
      /**
       Entries of one UTC day directory holding per-process log files.
       */
      const dayEntries = await readdir(
        join(directory, entry.name,),
        { withFileTypes: true, },
      );
      return await Promise.all(
        dayEntries
          .filter(isLogFileEntry,)
          .map(async function readDayLog(dayEntry,) {
            return await readFile(join(directory, entry.name, dayEntry.name,), 'utf8',);
          },),
      );
    }
    return [];
  },),);
  return groups.flat();
}

await describe({
  name: 'child environment',
  concurrency: 1,
  children: [
    it({
      name: 'STATES every key and every package setting of this process as absent, keeps the plain variable '
        + 'and sets the extra',
      fn: async () => {
        using fakes = variablesSet({ values: FAKE_VARIABLES, },);
        void fakes;
        const env = environmentWithoutKeys({ extra: { NEVER_REAL_EXTRA: 'a kitten plays', }, },);
        /**
         Names of the variables this case set or the fixture states, in code unit order.
         */
        const names = Object
          .keys(env,)
          .filter((name,) => name.startsWith('NEVER_REAL_',) || name.startsWith('TRANSLATION_REPAIR_NEVER_REAL_',))
          .toSorted();
        expect(names.map((name,) => [name, env[name],],),).toEqual([
          ['NEVER_REAL_ELSEWHERE_API_KEY', undefined,],
          ['NEVER_REAL_EXTRA', 'a kitten plays',],
          ['NEVER_REAL_PLAIN', 'a calico naps',],
          ['TRANSLATION_REPAIR_NEVER_REAL_API_KEY', undefined,],
          ['TRANSLATION_REPAIR_NEVER_REAL_SETTING', undefined,],
        ],);
      },
    },),
    it({
      name: 'LEAVES A CHILD OF SPAWNKEYLESS no variable ending in _API_KEY or starting with TRANSLATION_REPAIR_, '
        + 'the plain variable, the search path and the extra in place, and the directory it was given',
      fn: async () => {
        using fakes = variablesSet({ values: FAKE_VARIABLES, },);
        void fakes;
        await using scratch = await scratchDir({ prefix: 'child-environment-', },);
        const { stdout, } = await spawnKeyless({
          file: process.execPath,
          args: ['--input-type=module', '--eval', REPORT_PROGRAM,],
          cwd: scratch.path,
          extra: { NEVER_REAL_EXTRA: 'a kitten plays', },
        },);
        expect(reportOf({ text: stdout, },),).toEqual({
          keys: [],
          settings: [],
          plain: 'a calico naps',
          extra: 'a kitten plays',
          hasPath: true,
          cwd: await realpath(scratch.path,),
        },);
      },
    },),
    it({
      name: 'LEAVES A CHILD OF RUNKEYLESS no variable ending in _API_KEY or starting with TRANSLATION_REPAIR_, '
        + 'the plain variable, the search path and the extra in place, and the directory it was given',
      fn: async () => {
        using fakes = variablesSet({ values: FAKE_VARIABLES, },);
        void fakes;
        await using scratch = await scratchDir({ prefix: 'child-environment-', },);
        const done = await runKeyless({
          file: process.execPath,
          args: ['--input-type=module', '--eval', REPORT_PROGRAM,],
          cwd: scratch.path,
          extra: { NEVER_REAL_EXTRA: 'a kitten plays', },
        },);
        expect(done.code,).toBe(0,);
        expect(done.stderr,).toBe('',);
        expect(reportOf({ text: done.stdout, },),).toEqual({
          keys: [],
          settings: [],
          plain: 'a calico naps',
          extra: 'a kitten plays',
          hasPath: true,
          cwd: await realpath(scratch.path,),
        },);
      },
    },),
    it({
      name: 'HANDS A CHILD the variables a case names even where they end in _API_KEY or start with '
        + 'TRANSLATION_REPAIR_, the control that the report sees a key',
      fn: async () => {
        using fakes = variablesSet({ values: FAKE_VARIABLES, },);
        void fakes;
        const { stdout, } = await spawnKeyless({
          file: process.execPath,
          args: ['--input-type=module', '--eval', REPORT_PROGRAM,],
          extra: {
            NEVER_REAL_CASE_API_KEY: 'a cat plays',
            TRANSLATION_REPAIR_RUNS_DIR: 'a basket',
          },
        },);
        expect(reportOf({ text: stdout, },),).toEqual({
          keys: ['NEVER_REAL_CASE_API_KEY',],
          settings: ['TRANSLATION_REPAIR_RUNS_DIR',],
          plain: 'a calico naps',
          extra: null,
          hasPath: true,
          cwd: await realpath(process.cwd(),),
        },);
      },
    },),
    it({
      name: 'NAMES THE WORKSPACE LOG DIRECTORY for a child\'s logger, so no child falls back to the user\'s state '
        + 'directory, and HANDS ON the directory a case names instead',
      fn: async () => {
        expect(environmentWithoutKeys().MONOCHROMATIC_LOG_DIRECTORY,).toBe(CHILD_LOG_DIRECTORY,);
        expect(
          environmentWithoutKeys({ extra: { MONOCHROMATIC_LOG_DIRECTORY: '/a/basket', }, },)
            .MONOCHROMATIC_LOG_DIRECTORY,
        ).toBe('/a/basket',);
      },
    },),
    it({
      name: 'LOGS A CHILD STARTED OUTSIDE ANY PROJECT into the workspace log directory, writing nothing to stderr '
        + 'and nothing under the state directory the logger would otherwise fall back to',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'child-environment-', },);
        /**
         State directory the child is pointed at, which must stay absent.
         */
        const state = join(
          scratch.path,
          'state',
        );
        /**
         Text only this case's child logs.
         */
        const marker = `whiskers ${randomUUID()}`;
        const done = await runKeyless({
          file: process.execPath,
          args: ['--input-type=module', '--eval', loggingProgram({ marker, },),],
          cwd: scratch.path,
          extra: { XDG_STATE_HOME: state, },
        },);
        expect({
          code: done.code,
          stderr: done.stderr,
        },).toEqual({
          code: 0,
          stderr: '',
        },);
        expect(await readdir(scratch.path,),).toEqual([],);
        /**
         Contents of every log file in the workspace log directory.
         */
        const logs = await readLogTexts({ directory: CHILD_LOG_DIRECTORY, },);
        expect(logs.filter((text,) => text.includes(marker,)).length,).toBe(1,);
      },
    },),
  ],
},);

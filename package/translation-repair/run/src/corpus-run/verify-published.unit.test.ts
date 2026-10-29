/**
 Tests for `verify-published` as an operator runs it: the built CLI over a
 throwaway runs directory, read for what it prints and how it exits.

 A RUN ALWAYS SHIPS (the owner, 2026-09-27; ledger A16b): the verifier prints
 every finding and exits 0, keeping exit 2 for a run it could not read at
 all, which the last case holds as the control that the exit code is read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
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

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import {
  artifactPathOf,
  artifactTextWithoutArchive,
  builtPipelineDigestOf,
  declineEntry,
  ENTRY,
  publishedRun,
  SETTLED_BY,
  strayPage,
} from '@monochromatic-dev/translation-repair-page-check/ts/corpus-run/published-run.test-fixture.ts';

/**
 Exit code the shared fixture reports for a child a signal ended.
 */
const SIGNALLED = -1;

/**
 What the built verifier printed and how it ended, with the runs directory
 the only setting the child is given (and no provider key: the shared fixture
 removes every variable ending in `_API_KEY` before the child starts).

 @param runsDir - directory to verify

 @param args - command-line arguments, none where absent

 @returns Exit code, then stdout and stderr whole

 @example
 ```ts
 const run = await builtVerifier({ runsDir, },);
 ```
 */
async function builtVerifier(
  {
    runsDir,
    args = [],
  }: {
    readonly runsDir: string;
    readonly args?: readonly string[];
  },
): Promise<{
  readonly code: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  return await runBuiltCommand({
    command: 'verify-published',
    packageEntry: RUN_ENTRY,
    args,
    env: { TRANSLATION_REPAIR_RUNS_DIR: runsDir, },
  },);
}

/**
 Runs the verifier over one runs directory.

 @param runsDir - directory to verify

 @returns Its exit code and what it printed

 @throws {@link Error} when a signal ended it, which leaves no exit code to read, or it never started

 @example
 ```ts
 const { status, stdout, } = await verify({ runsDir, },);
 ```
 */
async function verify(
  { runsDir, }: { readonly runsDir: string; },
): Promise<{
  readonly status: number;
  readonly stdout: string;
}> {
  /**
   The finished process.
   */
  const done = await builtVerifier({ runsDir, },);
  if (done.code === SIGNALLED)
    throw new Error('verify-published ended on a signal',);
  return {
    status: done.code,
    stdout: done.stdout,
  };
}

await describe({
  name: 'verify-published',
  children: [
    it({
      name: 'AGREES on the page a pass publishes and exits 0, the control the findings rest on',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;

        /**
         What the verifier did.
         */
        const { status, stdout, } = await verify({ runsDir, },);
        expect(stdout.includes('1 of 1 page carries every wording its artifact promised',),).toBe(true,);
        expect(status,).toBe(0,);
      },
    },),
    it({
      name: 'PRINTS a page that disagrees with its artifact and still exits 0, since a run always ships',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await writeFile(
          pagePath,
          (await readFile(
            pagePath,
            'utf8',
          )).replace(
            'The cat has a bowl.',
            'The cat has.',
          ),
        );

        /**
         What the verifier did.
         */
        const { status, stdout, } = await verify({ runsDir, },);
        expect(stdout.includes('0 of 1 page carry every wording their artifacts promised',),).toBe(true,);
        expect(status,).toBe(0,);
      },
    },),
    it({
      name: 'PRINTS an artifact with no page and still exits 0, naming the pass that writes it',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rm(pagePath,);

        /**
         What the verifier did.
         */
        const { status, stdout, } = await verify({ runsDir, },);
        expect(stdout.includes(`SETTLED AND NEVER PUBLISHED: ${ENTRY}`,),).toBe(true,);
        expect(stdout.includes('writes it from its artifact',),).toBe(true,);
        expect(status,).toBe(0,);
      },
    },),
    it({
      name: 'EXITS 2 on a run it cannot read at all, the control that the exit code is what is read',
      fn: async () => {
        /**
         A runs directory holding nothing.
         */
        await using scratch = await scratchDir({ prefix: 'verify-published-empty-', },);
        const runsDir = scratch.path;

        expect((await verify({ runsDir, },)).status,).toBe(2,);
      },
    },),

    it({
      name: 'PRINTS every line of an agreeing run whole and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0\n'
            + 'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
            + 'missing=0 consistency-groups=0 read-at-generation=22\n'
            + 'verify-published: 1 of 1 page carries every wording its artifact promised; 1 of those at the length it implies, 0 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS the wrong length, the missing slice and the other build that read it, whole, and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await writeFile(
          pagePath,
          (await readFile(
            pagePath,
            'utf8',
          )).replace(
            'The cat has a bowl.',
            'The cat has.',
          ),
        );

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0\n'
            + 'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=74/expected 81 '
            + 'missing=1 consistency-groups=0 read-at-generation=22\n'
            + '  WRONG LENGTH: page is -7 characters off what the archive plus every slice change comes to. '
            + 'Text no slice decided on was lost or added\n'
            + '  MISSING slice 1, 35 characters the page does not carry in order\n'
            + `  READ BY ANOTHER BUILD: settled by ${ SETTLED_BY }, read by ${ await builtPipelineDigestOf({ packageEntry: RUN_ENTRY, },) }; `
            + 'a pass started here on this build rewrites the page to this reading\n'
            + 'verify-published: 0 of 1 page carry every wording their artifacts promised; 0 of those at the length it implies, 0 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS an artifact with no page whole, and the closing line for no matched entry, and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const {
          runsDir,
          pagePath,
        } = published;
        await rm(pagePath,);

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: matched=0 settledWithNoPage=1 pageWithNoArtifact=0 declined=0\n'
            + '  SETTLED AND NEVER PUBLISHED: CatEntry1. The archive ships for it until the next pass started in this runs '
            + 'directory writes it from its artifact\n'
            + 'verify-published: 0 of 0 pages carry every wording their artifacts promised; 0 of those at the length it implies, 0 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS that no published tree exists beside one settled entry, in the singular, and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await rm(
          join(
            runsDir,
            'fixed',
          ),
          { recursive: true, },
        );

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: NO PUBLISHED TREE (ENOENT). 1 settled entry is unpublished, and the next pass started in this '
            + 'runs directory writes each from its artifact\n'
            + 'verify-published: matched=0 settledWithNoPage=1 pageWithNoArtifact=0 declined=0\n'
            + '  SETTLED AND NEVER PUBLISHED: CatEntry1. The archive ships for it until the next pass started in this runs '
            + 'directory writes it from its artifact\n'
            + 'verify-published: 0 of 0 pages carry every wording their artifacts promised; 0 of those at the length it implies, 0 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS a page with no artifact and a declined entry that has a page, whole, and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await strayPage({
          runsDir,
          entryId: 'StrayCat2',
        },);
        await strayPage({
          runsDir,
          entryId: 'DeclinedCat3',
        },);
        await declineEntry({
          runsDir,
          entryId: 'DeclinedCat3',
        },);

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=2 declined=1\n'
            + '  DECLINED AND PUBLISHED ANYWAY: DeclinedCat3. The archive\'s note says the page is the author\'s own English, '
            + 'so no page should stand here; the next pass started in this runs directory removes it\n'
            + '  PUBLISHED AND NOT SETTLED: StrayCat2. It ships as it stands, with no artifact to check it against, until a '
            + 'pass settles the entry again and overwrites it\n'
            + 'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=81=expected '
            + 'missing=0 consistency-groups=0 read-at-generation=22\n'
            + 'verify-published: 1 of 1 page carries every wording its artifact promised; 1 of those at the length it implies, 0 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS an artifact that will not read by the class that refused it, naming no passage, and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await writeFile(
          join(
            runsDir,
            'artifacts',
            `${ENTRY}.json`,
          ),
          '{ not json',
        );

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0\n'
            + 'CatEntry1: REFUSED by SyntaxError\n'
            + 'verify-published: 0 of 1 page carry every wording their artifacts promised; 0 of those at the length it implies, 0 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS a page whose artifact predates the stored archive text as UNWEIGHED, whole, and exits 0',
      fn: async () => {
        await using published = await publishedRun();
        const { runsDir, } = published;
        await writeFile(
          artifactPathOf({
            runsDir,
            entryId: ENTRY,
          },),
          artifactTextWithoutArchive({ entryId: ENTRY, },),
        );

        const run = await builtVerifier({ runsDir, },);
        expect(run,).toEqual({
          code: 0,
          stdout: 'verify-published: matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0\n'
            + 'CatEntry1: wordings=2 silent=0 source-shipped=0 chars=UNWEIGHED(artifact predates stored archive text) '
            + 'missing=0 '
            + 'consistency-groups=0 read-at-generation=22\n'
            + 'verify-published: 1 of 1 page carries every wording its artifact promised; 0 of those at the length it implies, 1 '
            + 'UNWEIGHED because the artifact predates the stored archive text\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS sixty entries in the order the run lists them and exits 0',
      fn: async () => {
        /**
         Entry ids in the order the run lists them.
         */
        const entryIds = Array.from(
          { length: 60, },
          function entryAt(_unused, index,): string {
            return `CatEntry${ String(index,).padStart(
              3,
              '0',
            ) }`;
          },
        );
        await using published = await publishedRun({ entryIds, },);
        const { runsDir, } = published;

        const run = await builtVerifier({ runsDir, },);
        expect(run.code,).toBe(0,);
        expect(run.stdout
          .split('\n',)
          .filter(function isEntryLine(line,): boolean {
            return line.startsWith('CatEntry',);
          },)
          .map(function idOf(line,): string {
            return line.slice(
              0,
              line.indexOf(':',),
            );
          },),).toEqual(entryIds,);
      },
    },),
    it({
      name: 'EXITS 2 and says nothing was verified when the run has no artifacts directory',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'verify-published-bare-', },);

        const run = await builtVerifier({ runsDir: scratch.path, },);
        expect(run,).toEqual({
          code: 2,
          stdout: 'verify-published: NOTHING VERIFIED, no artifacts directory under the run (ENOENT). No page was read and no '
            + 'artifact was compared, so this is not a clean run\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'EXITS 2 and says nothing was verified when the artifacts directory holds no artifact',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'verify-published-hollow-', },);
        await mkdir(join(
          scratch.path,
          'artifacts',
        ),);

        const run = await builtVerifier({ runsDir: scratch.path, },);
        expect(run,).toEqual({
          code: 2,
          stdout: 'verify-published: NOTHING VERIFIED, the artifacts directory holds no settled artifact. No page was read and '
            + 'no artifact was compared, so this is not a clean run\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'REFUSES a flag the command does not declare, as stated, at exit 6 with nothing on stdout',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'verify-published-flag-', },);

        const run = await builtVerifier({
          runsDir: scratch.path,
          args: ['--bogus',],
        },);
        expect(run,).toEqual({ code: 6, stdout: '', stderr: 'verify-published: --bogus is not a flag this command reads. Usage: verify-published\n', },);
      },
    },),
  ],
},);

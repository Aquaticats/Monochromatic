/**
 Tests for the probe of the pinned commit a corpus read runs after it failed
 (`corpus-commit-probe.ts`), read through the corpus reader as a caller
 meets it: what the refusal's kind and cause say when git, or a program
 standing in for it, ends the probe each way it can.

 Exercised against throwaway git repositories built in a temp directory;
 fixture content is cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  chmod,
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
  CorpusReadError,
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { spawnKeyless, } from './child-environment.test-fixture.ts';
import { REAL_GIT, } from './hermetic-git-run.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  makeCloneHoldingOneCommit,
  PAIRED_ENTRY,
} from './corpus-lacked-commit.test-fixture.ts';

/**
 Exit status the program standing in for git ends a read with, as git does
 for a path absent at a commit.
 */
const FAKE_READ_STATUS = 128;

/**
 Exit status the program standing in for git ends its probe with when told
 to end on a status, one the probe of the commit does not read.
 */
const FAKE_PROBE_STATUS = 3;

/**
 What a refusal's cause says when the probe of its commit got no answer.
 */
const PROBE_UNANSWERED = 'the corpus read failed, and the probe of its commit got no answer about it';

/**
 Exit status git ends with when it dies, which it does on an object it cannot
 inflate as on a directory it cannot open as a repository.
 */
const GIT_DIED_STATUS = 128;

/**
 What a read of the paired entry's original page refused with, as `String`
 prints it, where the probe of its commit got no answer.

 @param commitSha - commit the read was at

 @returns The refusal's class and whole message

 @example
 ```ts
 const says = otherReadSays({ commitSha: clone.commitSha, },);
 ```
 */
function otherReadSays({ commitSha, }: { readonly commitSha: string; },): string {
  return `CorpusReadError: corpus read failed for ${commitSha}:people/${PAIRED_ENTRY}/page.md (other); `
    + 'the read failed another way: run the same git read in the clone by hand to see why.';
}

/**
 How the program standing in for git ends the probe: killed by a signal, or
 with an exit status the probe does not read, on the probe of the commit; or
 dying on the commit as git does on one it cannot inflate, and then ending the
 probe of the clone's git directory with a status that probe does not read.
 */
type FakeProbeEnd = 'signal' | 'status' | 'died-then-status';

/**
 Lines of the fake git's program that answer a `rev-parse`, by how the case
 wants the probe to end.

 @param probeEnd - how the probe ends

 @returns The program's lines up to the branch that answers a read

 @example
 ```ts
 const lines = revParseLines({ probeEnd: 'signal', },);
 ```
 */
function revParseLines({ probeEnd, }: { readonly probeEnd: FakeProbeEnd; },): readonly string[] {
  if (probeEnd === 'died-then-status') {
    return [
      "if (process.argv.includes('--git-dir')) {",
      `  process.exit(${String(FAKE_PROBE_STATUS,)});`,
      "} else if (process.argv.includes('rev-parse')) {",
      `  process.exit(${String(GIT_DIED_STATUS,)});`,
    ];
  }
  return [
    "if (process.argv.includes('rev-parse')) {",
    (probeEnd === 'signal')
      ? "  process.kill(process.pid, 'SIGKILL');"
      : `  process.exit(${String(FAKE_PROBE_STATUS,)});`,
  ];
}

/**
 Writes a program standing in for git: every read fails as git fails a path
 absent at a commit, and asked about the commit it ends the way a case names.

 @param dir - scratch directory the program is written into, removed by the case

 @param probeEnd - how the probe ends, each a way it can get no answer

 @returns Path of the program, which a pin names as its git binary

 @example
 ```ts
 const gitPath = await writeFakeGit({ dir: bin.path, probeEnd: 'signal', },);
 ```
 */
async function writeFakeGit(
  {
    dir,
    probeEnd,
  }: {
    readonly dir: string;
    readonly probeEnd: FakeProbeEnd;
  },
): Promise<string> {
  /**
   Where the program is written.
   */
  const path = join(
    dir,
    'git',
  );
  await writeFile(
    path,
    [
      `#!${process.execPath}`,
      ...revParseLines({ probeEnd, },),
      '} else {',
      String.raw`  process.stderr.write("fatal: path 'people/mittens/page.md' does not exist in 'HEAD'\n");`,
      `  process.exitCode = ${String(FAKE_READ_STATUS,)};`,
      '}',
      '',
    ].join('\n',),
    'utf8',
  );
  await chmod(
    path,
    0o755,
  );
  return path;
}

/**
 The facts a case compares about one failure a refusal keeps as its cause.

 @param failure - one failure the refusal's cause holds

 @returns Its class name, the exit status `execFile` records as `code`, and
 the exit status or signal nano-spawn records

 @example
 ```ts
 const facts = failureFacts({ failure: error, },);
 ```
 */
function failureFacts({ failure, }: { readonly failure: unknown; },): {
  readonly name: string;
  readonly code: unknown;
  readonly exitCode: unknown;
  readonly signalName: unknown;
} {
  if (!Error.isError(failure,))
    throw new Error(`a refusal's cause must hold errors, and it held ${String(failure,)}`,);
  return {
    name: failure.name,
    code: Reflect.get(
      failure,
      'code',
    ),
    exitCode: Reflect.get(
      failure,
      'exitCode',
    ),
    signalName: Reflect.get(
      failure,
      'signalName',
    ),
  };
}

/**
 What a read refused with after its probe, as the kind, the sentence, what
 the cause says and the failures it holds.

 @param refusal - what the read rejected with

 @returns Kind, printed sentence, the cause as `String` prints it and each
 failure the cause holds, one when it is the read's alone

 @example
 ```ts
 const facts = probedRefusalFacts({ refusal, },);
 ```
 */
function probedRefusalFacts({ refusal, }: { readonly refusal: unknown; },): {
  readonly kind: string;
  readonly says: string;
  readonly causeSays: string;
  readonly failures: readonly ReturnType<typeof failureFacts>[];
} {
  if (!(refusal instanceof CorpusReadError))
    throw new Error(`the read must refuse with a CorpusReadError, and it refused with ${String(refusal,)}`,);
  /**
   What the refusal keeps as its cause.
   */
  const { cause, } = refusal;
  return {
    kind: refusal.kind,
    says: String(refusal,),
    causeSays: String(cause,),
    failures: ((cause instanceof AggregateError) ? cause.errors : [cause,])
      .map(function factsOf(failure,) {
        return failureFacts({ failure, },);
      },),
  };
}

/**
 Reads the paired entry's original page at the one commit a throwaway clone
 holds, through a program standing in for git that fails the read and ends
 its probe of the commit the way a case names.

 @param probeEnd - how the probe ends, each a way it can get no answer

 @returns Commit the read was at, which the expected message names, and
 the facts of what the read refused with

 @example
 ```ts
 const { commitSha, facts, } = await readThroughFakeGit({ probeEnd: 'signal', },);
 ```
 */
async function readThroughFakeGit(
  { probeEnd, }: { readonly probeEnd: FakeProbeEnd; },
): Promise<{
  readonly commitSha: string;
  readonly facts: ReturnType<typeof probedRefusalFacts>;
}> {
  await using clone = await makeCloneHoldingOneCommit();
  await using bin = await scratchDir({ prefix: 'translation-repair-fake-git-', },);
  /**
   Program standing in for git.
   */
  const gitPath = await writeFakeGit({
    dir: bin.path,
    probeEnd,
  },);
  return {
    commitSha: clone.commitSha,
    facts: probedRefusalFacts({
      refusal: await rejectionOf(async function readsThroughFakeGit(): Promise<unknown> {
        return await readCorpusFile({
          pin: {
            cloneDir: clone.cloneDir,
            commitSha: clone.commitSha,
            gitPath,
          },
          relPath: `people/${PAIRED_ENTRY}/page.md`,
        },);
      },),
    },),
  };
}

await describe({
  name: 'probe of the pinned commit',
  children: [
    it({
      name: 'NAMES A COMMIT GIT CANNOT INFLATE IN A CLONE IT OPENS as other, keeping the probe\'s failure beside the '
        + 'read\'s, for a page and a listing, rather than telling the operator to check that the clone is a '
        + 'repository',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        /**
         The commit's loose object, written over with bytes no zlib stream
         opens, as a damaged disk or an interrupted copy leaves it.
         */
        const commitObject = join(
          clone.cloneDir,
          '.git',
          'objects',
          clone.commitSha.slice(
            0,
            2,
          ),
          clone.commitSha.slice(2,),
        );
        await chmod(
          commitObject,
          0o644,
        );
        await writeFile(
          commitObject,
          'not a zlib stream',
        );
        /**
         The pin at the damaged commit.
         */
        const pin = {
          cloneDir: clone.cloneDir,
          commitSha: clone.commitSha,
        };
        expect(probedRefusalFacts({
          refusal: await rejectionOf(async function readsAtDamagedCommit(): Promise<unknown> {
            return await readCorpusFile({
              pin,
              relPath: `people/${PAIRED_ENTRY}/page.md`,
            },);
          },),
        },),).toEqual({
          kind: 'other',
          says: otherReadSays({ commitSha: clone.commitSha, },),
          causeSays: `AggregateError: ${PROBE_UNANSWERED}`,
          failures: [
            {
              name: 'Error',
              code: GIT_DIED_STATUS,
              exitCode: undefined,
              signalName: undefined,
            },
            {
              name: 'SubprocessError',
              code: undefined,
              exitCode: GIT_DIED_STATUS,
              signalName: undefined,
            },
          ],
        },);
        expect(probedRefusalFacts({
          refusal: await rejectionOf(async function listsAtDamagedCommit(): Promise<unknown> {
            return await listCorpusPeople({ pin, },);
          },),
        },).kind,).toBe('other',);
      },
    },),

    it({
      name: 'KEEPS WHAT THE PROBE FAILED WITH beside the read\'s own failure when git is stopped by a signal on '
        + 'the probe of the commit, and names the kind other',
      fn: async () => {
        /**
         What the read refused with when git was killed by a signal on the
         probe, and the commit it read at.
         */
        const { commitSha, facts, } = await readThroughFakeGit({ probeEnd: 'signal', },);
        expect(facts,).toEqual({
          kind: 'other',
          says: otherReadSays({ commitSha, },),
          causeSays: `AggregateError: ${PROBE_UNANSWERED}`,
          failures: [
            {
              name: 'Error',
              code: FAKE_READ_STATUS,
              exitCode: undefined,
              signalName: undefined,
            },
            {
              name: 'SubprocessError',
              code: undefined,
              exitCode: undefined,
              signalName: 'SIGKILL',
            },
          ],
        },);
      },
    },),

    it({
      name: 'KEEPS WHAT THE PROBE FAILED WITH beside the read\'s own failure when the probe of the commit ends '
        + 'with an exit status it does not read, and names the kind other',
      fn: async () => {
        /**
         What the read refused with when the probe ended on a status it does
         not read, and the commit it read at.
         */
        const { commitSha, facts, } = await readThroughFakeGit({ probeEnd: 'status', },);
        expect(facts,).toEqual({
          kind: 'other',
          says: otherReadSays({ commitSha, },),
          causeSays: `AggregateError: ${PROBE_UNANSWERED}`,
          failures: [
            {
              name: 'Error',
              code: FAKE_READ_STATUS,
              exitCode: undefined,
              signalName: undefined,
            },
            {
              name: 'SubprocessError',
              code: undefined,
              exitCode: FAKE_PROBE_STATUS,
              signalName: undefined,
            },
          ],
        },);
      },
    },),

    it({
      name: 'KEEPS BOTH PROBES\' FAILURES beside the read\'s own when git dies on the commit and the probe of the '
        + 'clone\'s git directory then ends with an exit status it does not read, and names the kind other',
      fn: async () => {
        /**
         What the read refused with when both probes ended without an answer,
         and the commit it read at.
         */
        const { commitSha, facts, } = await readThroughFakeGit({ probeEnd: 'died-then-status', },);
        expect(facts,).toEqual({
          kind: 'other',
          says: otherReadSays({ commitSha, },),
          causeSays: `AggregateError: ${PROBE_UNANSWERED}`,
          failures: [
            {
              name: 'Error',
              code: FAKE_READ_STATUS,
              exitCode: undefined,
              signalName: undefined,
            },
            {
              name: 'SubprocessError',
              code: undefined,
              exitCode: GIT_DIED_STATUS,
              signalName: undefined,
            },
            {
              name: 'SubprocessError',
              code: undefined,
              exitCode: FAKE_PROBE_STATUS,
              signalName: undefined,
            },
          ],
        },);
      },
    },),

    it({
      name: 'STOPS WITH BOTH FAILURES when the probe fails as no subprocess failure: a working directory removed '
        + 'under the process after a read failed names its own fault beside the read\'s failure, rather than '
        + 'reading as a corpus refusal or dropping what the read met',
      fn: async () => {
        await using fixture = await makeCloneHoldingOneCommit();
        /**
         Child program that removes the directory it stands in and reads a
         path the commit lacks, printing what the read threw. Every value it
         carries is written as a JSON literal, which is a JavaScript literal too.
         */
        const program = [
          "import { mkdtempSync, rmdirSync, } from 'node:fs';",
          "import { join, } from 'node:path';",
          `const { CorpusReadError, readCorpusFile, } = await import(${
            JSON.stringify(import.meta.resolve('@monochromatic-dev/translation-repair-corpus-source',),)
          });`,
          `const gone = mkdtempSync(join(${JSON.stringify(scratchRoot(),)}, 'translation-repair-gone-cwd-'));`,
          'process.chdir(gone);',
          'rmdirSync(gone);',
          'try {',
          `  await readCorpusFile({ pin: { cloneDir: ${JSON.stringify(fixture.cloneDir,)}, commitSha: ${
            JSON.stringify(fixture.commitSha,)
          }, gitPath: ${JSON.stringify(REAL_GIT,)} }, relPath: 'people/ghost/page.md' });`,
          "  console.log(JSON.stringify({ read: true }));",
          '} catch (error) {',
          '  console.log(JSON.stringify({ isCorpusReadError: error instanceof CorpusReadError, name: error.name, '
            + 'message: error.message, code: error.code, members: (error.errors ?? []).map(function facts(member) { '
            + 'return { name: member.name, code: member.code }; }), cause: error.cause?.code }));',
          '}',
        ].join('\n',);
        /**
         What the child printed.
         */
        const { stdout, } = await spawnKeyless({
          file: process.execPath,
          args: [
            '--input-type=module',
            '--eval',
            program,
          ],
        },);
        expect(JSON.parse(stdout,),).toEqual({
          isCorpusReadError: false,
          name: 'AggregateError',
          message: 'the corpus read failed, and the probe of its commit failed in this process before git ran',
          members: [
            {
              name: 'Error',
              code: GIT_DIED_STATUS,
            },
            {
              name: 'Error',
              code: 'ENOENT',
            },
          ],
          cause: 'ENOENT',
        },);
      },
    },),
  ],
},);

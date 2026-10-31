/**
 Tests for how one settled artifact answers "which pipeline produced you".

 FOUR ANSWERS, NOT TWO, and the difference between them is what an operator
 does next. A `placed` artifact belongs to a named generation. A `legacy` one
 is a perfectly good result whose pipeline can no longer be named, and the
 remedy is a fresh directory. An `untagged` one belongs nowhere and is
 deleted. A `malformed` one belongs to the reader that reports malformed
 files. Collapsing any pair of those tells an operator to delete good work or
 to keep a file the pool would admit under a name its bytes never claimed.

 THE CENSUS IS THE ONLY CALLER, and it aggregates: it counts artifacts per
 generation and reports totals. Every rule here reaches the suite as a count,
 which is why a rule that placed a file in the wrong bucket could survive as
 long as the total stayed right.

 THREE CASES PIN DOCUMENTED REGRESSIONS the module's own comments describe.
 A tip of `HEAD` resolves against the READER's checkout rather than against
 whatever produced the artifact. An artifact carrying no id at all used to
 skip the identity check and be placed on its file name alone. A digest this
 build cannot read is legacy rather than garbage, because the recorded value
 names the scheme that produced it.

 THE POOL LINES NAME A SHAPE AND NEVER A VALUE. A malformed id or digest is
 whatever bytes a bad file carries, and `readPlacement` runs inside the pass
 as well as in the readers, so those lines reach a pass's stdout. Four cases
 capture what is printed and pin both halves: the type and length are there,
 the recorded value is not.

 DISPOSABLE FIXTURES ONLY: every case writes into its own `mkdtemp` directory
 and nothing here reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactFileName,
  type Placement,
  readPlacement,
} from '../../dist/final/node/index.mjs';
import { readRunLogText, } from '@monochromatic-dev/translation-repair-glossary';
import {
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Commit a fixture records, spelled the way git writes one.
 */
const FIXED_TIP = 'a'.repeat(40,);

/**
 Same commit in the longer hash git also writes.
 */
const LONG_TIP = 'b'.repeat(64,);

/**
 Built output a fixture records, in the scheme this build reads.
 */
const FIXED_DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 Writes exactly these files into a caller-owned artifacts directory.

 @param dir - case-owned directory to write into

 @param files - file name to raw contents, written verbatim so a case can
 write something that is not JSON at all

 @returns Nothing; the files are the effect

 @example
 ```ts
 await artifactsDirWith({ dir: scratch.path, files: { 'Mittens.json': '{}', }, },);
 ```
 */
async function artifactsDirWith(
  { dir, files, }: {
    readonly dir: string;
    readonly files: Readonly<Record<string, string>>;
  },
): Promise<void> {
  await Promise.all(Object.entries(files,)
    .map(async function writeOne([
      name,
      text,
    ],): Promise<void> {
      await writeFile(
        join(
          dir,
          name,
        ),
        text,
        'utf8',
      );
    },),);
}

/**
 Places one artifact written from a record, which is what a real one is.

 @param body - fields this artifact records

 @param name - file name to write it under, which the identity check reads

 @returns How it places

 @example
 ```ts
 const placement = await placementOf({ body: { id: 'Mittens', }, name: 'Mittens.json', },);
 ```
 */
async function placementOf(
  {
    body,
    name = 'Mittens.json',
  }: {
    readonly body: unknown;
    readonly name?: ArtifactFileName;
  },
): Promise<Placement> {
  await using scratch = await scratchDir({ prefix: 'artifact-placement-', },);
  await artifactsDirWith({ dir: scratch.path, files: { [name]: JSON.stringify(body,), }, },);
  return await readPlacement({
    artifactsDir: scratch.path,
    name,
  },);
}

/**
 Keeps the lines a placement printed about one file.

 @param lines - everything captured while the case ran

 @param name - file the case placed

 @returns Lines naming that file

 @example
 ```ts
 const own = linesAbout({ lines: printed.lines, name: 'Mismatch.json', },);
 ```
 */
function linesAbout(
  {
    lines,
    name,
  }: {
    readonly lines: readonly string[];
    readonly name: string;
  },
): readonly string[] {
  return lines.filter(function names(line,): boolean {
    return line.startsWith(`POOL ${name} `,);
  },);
}

await describe({
  name: readPlacement.name,
  // ONE AT A TIME: cases divert the process-wide `console.log` across an
  // await (ledger B79).
  concurrency: 1,
  children: [
    it({
      name: 'PLACES an artifact recording its own name, a canonical commit '
        + 'and a digest this build reads, which is the control every other '
        + 'case departs from one field at a time',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({
            kind: 'placed',
            tip: FIXED_TIP,
            digest: FIXED_DIGEST,
          },);
      },
    },),

    it({
      name: 'PLACES an artifact whose commit is the longer object id, since '
        + 'a repository hashing with SHA-256 writes identities of that length '
        + 'and they are no less canonical',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: LONG_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({
            kind: 'placed',
            tip: LONG_TIP,
            digest: FIXED_DIGEST,
          },);
      },
    },),

    it({
      name: 'reads an artifact recording no pipeline at all as LEGACY, not as '
        + 'unplaceable: it predates generation identity and is a sound result '
        + 'whose pipeline can no longer be named',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: FIXED_TIP,
          },
        },),)
          .toEqual({
            kind: 'legacy',
            tip: FIXED_TIP,
          },);
      },
    },),

    it({
      name: 'reads a digest this build cannot parse as LEGACY, since the '
        + 'recorded value names the scheme that produced it and calling it '
        + 'unplaceable would tell an operator to delete good work',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: 'sha1-tree-v0:whiskers',
          },
        },),)
          .toEqual({
            kind: 'legacy',
            tip: FIXED_TIP,
          },);
      },
    },),

    it({
      name: 'REFUSES a commit spelled as a revision expression, because '
        + '`HEAD` resolves against the READER`s checkout at read time and so '
        + 'silently answers a different question than the one asked',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: 'HEAD',
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a commit in uppercase hex, since git writes lowercase '
        + 'and two spellings of one commit would count as two generations',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: 'A'.repeat(40,),
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a commit one character short of an object id, which is '
        + 'an abbreviation rather than an identity',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: 'a'.repeat(39,),
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a commit that is not a string at all',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: 40,
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES an artifact recording no commit',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES an artifact whose recorded id is not its file name, '
        + 'which is how `Mittens-copy.json` would otherwise become a second '
        + 'settled entry',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
          name: 'Mittens-copy.json',
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES an artifact carrying NO id at all, rather than placing '
        + 'it on its file name. Guarding the comparison on the field being '
        + 'present let exactly this file skip the check, which is the one '
        + 'reading the check exists to refuse',
      fn: async () => {
        expect(await placementOf({
          body: {
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a digest that is not a string, which no scheme could '
        + 'have written and so names nothing',
      fn: async () => {
        expect(await placementOf({
          body: {
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: 7,
          },
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a file whose contents parse to something that is not an '
        + 'object, since a number carries no fields to read',
      fn: async () => {
        expect(await placementOf({ body: 42, },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a file whose contents parse to null, which is an object '
        + 'by `typeof` and carries no fields either',
      fn: async () => {
        expect(await placementOf({ body: null, },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'REFUSES a file whose name is nothing but the suffix, since the '
        + 'id it would be keyed by is the empty string',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'artifact-placement-', },);
        await artifactsDirWith({ dir: scratch.path, files: { '.json': '{}', }, },);
        expect(await readPlacement({
          artifactsDir: scratch.path,
          name: '.json',
        },),)
          .toEqual({ kind: 'untagged', },);
      },
    },),

    it({
      name: 'reports a file that is not JSON as MALFORMED, which is a '
        + 'different finding from unplaceable: it belongs to the reader that '
        + 'reports malformed files rather than to the operator who deletes',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'artifact-placement-', },);
        await artifactsDirWith({
          dir: scratch.path,
          files: { 'Mittens.json': '{ "id": "Mittens", "tip"', },
        },);
        expect(await readPlacement({
          artifactsDir: scratch.path,
          name: 'Mittens.json',
        },),)
          .toEqual({ kind: 'malformed', },);
      },
    },),

    it({
      name: 'reports a file that is not there as MALFORMED and does not '
        + 'throw. The read used to sit outside the guard, so a vanished file '
        + 'aborted the whole census, which now runs at pass startup',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'artifact-placement-', },);
        await artifactsDirWith({ dir: scratch.path, files: {}, },);
        expect(await readPlacement({
          artifactsDir: scratch.path,
          name: 'Vanished.json',
        },),)
          .toEqual({ kind: 'malformed', },);
      },
    },),

    it({
      name: 'reports a DIRECTORY named like an artifact as malformed rather '
        + 'than throwing EISDIR out of the census',
      fn: async () => {
        /**
         Disposable root holding a directory where an artifact should be.
         */
        await using scratch = await scratchDir({ prefix: 'artifact-placement-', },);
        const dir = scratch.path;
        await artifactsDirWith({ dir, files: {}, },);
        await mkdir(join(
          dir,
          'Mittens.json',
        ),);

        expect(await readPlacement({
          artifactsDir: dir,
          name: 'Mittens.json',
        },),)
          .toEqual({ kind: 'malformed', },);
      },
    },),

    it({
      name: 'NAMES A MISMATCHED ID BY SHAPE ALONE on the POOL line and never by '
        + 'value: a malformed id is whatever bytes a bad file carries, and the '
        + 'line reaches the pass stdout',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            id: 'Whiskers',
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
          name: 'Mismatch.json',
        },),)
          .toEqual({ kind: 'untagged', },);
        expect(linesAbout({
          lines: printed.lines,
          name: 'Mismatch.json',
        },),)
          .toEqual([
            'POOL Mismatch.json records an id that is not its file name (a '
              + 'string of 8 characters); treating it as unplaceable',
          ],);
      },
    },),

    it({
      name: 'NAMES AN ABSENT ID as absent on the POOL line',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
          name: 'Absent.json',
        },),)
          .toEqual({ kind: 'untagged', },);
        expect(linesAbout({
          lines: printed.lines,
          name: 'Absent.json',
        },),)
          .toEqual([
            'POOL Absent.json records an id that is not its file name (absent); '
              + 'treating it as unplaceable',
          ],);
      },
    },),

    it({
      name: 'NAMES AN ID RECORDED AS NULL as null on the POOL line, apart from an absent one',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            id: null,
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
          name: 'Nulled.json',
        },),)
          .toEqual({ kind: 'untagged', },);
        expect(linesAbout({
          lines: printed.lines,
          name: 'Nulled.json',
        },),)
          .toEqual([
            'POOL Nulled.json records an id that is not its file name (null); '
              + 'treating it as unplaceable',
          ],);
      },
    },),

    it({
      name: 'NAMES AN ID RECORDED AS AN ARRAY by its kind alone on the POOL line, never its members',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            id: ['Whiskers',],
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
          name: 'Listed.json',
        },),)
          .toEqual({ kind: 'untagged', },);
        expect(linesAbout({
          lines: printed.lines,
          name: 'Listed.json',
        },),)
          .toEqual([
            'POOL Listed.json records an id that is not its file name (an array); '
              + 'treating it as unplaceable',
          ],);
      },
    },),

    it({
      name: 'NAMES AN ID RECORDED AS AN OBJECT by its kind alone on the POOL line, never its fields',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            id: { name: 'Whiskers', },
            tip: FIXED_TIP,
            pipelineDigest: FIXED_DIGEST,
          },
          name: 'Nested.json',
        },),)
          .toEqual({ kind: 'untagged', },);
        expect(linesAbout({
          lines: printed.lines,
          name: 'Nested.json',
        },),)
          .toEqual([
            'POOL Nested.json records an id that is not its file name (an object); '
              + 'treating it as unplaceable',
          ],);
      },
    },),

    it({
      name: 'NAMES A NON-STRING DIGEST BY SHAPE on the POOL line',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            id: 'Digit',
            tip: FIXED_TIP,
            pipelineDigest: 7,
          },
          name: 'Digit.json',
        },),)
          .toEqual({ kind: 'untagged', },);
        expect(linesAbout({
          lines: printed.lines,
          name: 'Digit.json',
        },),)
          .toEqual([
            'POOL Digit.json records a pipeline digest that is not a string '
              + '(a number); treating it as unplaceable',
          ],);
      },
    },),

    it({
      name: 'NAMES AN UNREADABLE DIGEST BY LENGTH ALONE on the POOL line, since '
        + 'a value this build cannot read may be anything at all',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

        expect(await placementOf({
          body: {
            id: 'Unreadable',
            tip: FIXED_TIP,
            pipelineDigest: 'sha1-tree-v0:whiskers',
          },
          name: 'Unreadable.json',
        },),)
          .toEqual({
            kind: 'legacy',
            tip: FIXED_TIP,
          },);

        /**
         What was printed about this file.
         */
        const own = linesAbout({
          lines: printed.lines,
          name: 'Unreadable.json',
        },);

        expect(own,).toEqual([
          'POOL Unreadable.json records a pipeline digest this build cannot '
            + 'read (a string of 21 characters); treating it as legacy',
        ],);
        expect(own.join('\n',)
          .includes('whiskers',),).toBe(false,);
      },
    },),

    it({
      name: 'RECORDS EACH POOL LINE IN THE INSTALLED RUN LOG beside printing it, the four kinds a placement says, so a pass '
        + 'that reads its artifacts leaves them in its run folder (ledger B1111)',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'artifact-placement-', },);
        await using runLog = await openRunLog({ runDir: join(scratch.path, 'run',), },);
        using _installed = installRunLog({ log: runLog, },);

        // Four artifacts that each draw a POOL line: a mismatched id, a digest that is no string, a digest this build
        // cannot read, and a file that is no JSON.
        await placementOf({
          body: { id: 'Whiskers', tip: FIXED_TIP, pipelineDigest: FIXED_DIGEST, },
          name: 'Mismatch.json',
        },);
        await placementOf({
          body: { id: 'Digit', tip: FIXED_TIP, pipelineDigest: 7, },
          name: 'Digit.json',
        },);
        await placementOf({
          body: { id: 'Unreadable', tip: FIXED_TIP, pipelineDigest: 'sha1-whatever', },
          name: 'Unreadable.json',
        },);
        await using broken = await scratchDir({ prefix: 'artifact-placement-', },);
        await artifactsDirWith({ dir: broken.path, files: { 'Broken.json': '{not json', }, },);
        await readPlacement({ artifactsDir: broken.path, name: 'Broken.json', },);
        await runLog.flush();

        /**
         The lines of the records, as the console would have printed them.
         */
        const records = (await readRunLogText({ path: join(scratch.path, 'run',), },)).split('\n',);
        for (const line of printed.lines) {
          expect(records.filter(function isThisLine(record,): boolean {
            return record.endsWith(`Z] ${line}`,);
          },).length,).toBe(1,);
        }
        expect(printed.lines.length,).toBe(4,);
      },
    },),
  ],
},);

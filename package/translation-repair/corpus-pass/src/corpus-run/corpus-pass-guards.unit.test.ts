/**
 Tests for the pass's guards over the artifacts a runs directory already
 holds: what resumes, and how every refusal reaches an operator.

 EACH GUARD'S REFUSAL IS A STATED REFUSAL BY CLASS. While the five guard
 errors were plain errors, a command that let them escape reported an
 operator's refusal as a fault in the command, at exit 5, under stack frames.
 Each case holds the class the guard raised, that it is a stated refusal, and
 its whole message; any other failure passes on as it was.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  GenerationDriftError,
  LegacyPipelineError,
  MislabelledArtifactError,
  SchemaGenerationError,
  UnplaceableArtifactError,
} from '@monochromatic-dev/translation-repair-slice-cache';
import {
  assertPipelineDigest,
  type PipelineDigest,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary';
import { assertPassResumable, } from '../../dist/final/node/index.mjs';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { settledArtifactText, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';

/**
 Build the fixture artifacts record, before it is checked for shape.
 */
const RECORDED_TEXT = `sha256-tree-v1:${'c'.repeat(64,)}`;
assertPipelineDigest(RECORDED_TEXT,);

/**
 Build the fixture artifacts record, which a pass under another digest finds foreign.
 */
const RECORDED_DIGEST: PipelineDigest = RECORDED_TEXT;

/**
 Build a pass of another pipeline would stamp, before it is checked for shape.
 */
const OTHER_TEXT = `sha256-tree-v1:${'e'.repeat(64,)}`;
assertPipelineDigest(OTHER_TEXT,);

/**
 Build a pass of another pipeline would stamp.
 */
const OTHER_DIGEST: PipelineDigest = OTHER_TEXT;

/**
 Entry every case settles.
 */
const ENTRY = 'CatEntry1';

/**
 Commit every hand-written artifact records.
 */
const FIXED_TIP = '1111111111111111111111111111111111111111';

/**
 A complete settled artifact of the entry, recording {@link RECORDED_DIGEST}.

 @returns Artifact's JSON text

 @example
 ```ts
 await writeFile(join(dir, `${ENTRY}.json`,), settledArtifact(),);
 ```
 */
function settledArtifact(): string {
  return settledArtifactText({
    prepared: prepareDocumentPair({
      sourceText: '## 第一节\n\n猫猫在窗台上睡觉。\n',
      targetText: '## Section one\n\nThe cat sleeps on the sill.\n',
      includeFrontMatter: true,
      sealArchiveOriginal: true,
    },),
    entryId: ENTRY,
  },);
}

/**
 What a pass over a directory of artifacts refused with.

 @param artifactsDir - directory the artifacts are in
 @param pipelineDigest - build the pass would stamp
 @param driftAllowed - whether the launch asked to resume across a foreign build

 @returns Whatever the guards threw

 @throws {@link Error} when the guards let the directory through

 @example
 ```ts
 const refusal = await refusalOf({ artifactsDir, pipelineDigest: OTHER_DIGEST, driftAllowed: false, },);
 ```
 */
async function refusalOf(
  {
    artifactsDir,
    pipelineDigest,
    driftAllowed,
  }: {
    readonly artifactsDir: string;
    readonly pipelineDigest: PipelineDigest;
    readonly driftAllowed: boolean;
  },
): Promise<unknown> {
  try {
    await assertPassResumable({
      artifactsDir,
      pipelineDigest,
      driftAllowed,
    },);
  }
  catch (error) {
    return error;
  }
  throw new Error('the guards let the directory through',);
}

await describe({
  name: assertPassResumable.name,
  concurrency: 1,
  children: [
    it({
      name: 'LETS A FRESH DIRECTORY THROUGH, since a first invocation has nothing to disagree with',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);

        await assertPassResumable({
          artifactsDir: scratch.path,
          pipelineDigest: OTHER_DIGEST,
          driftAllowed: false,
        },);
      },
    },),
    it({
      name: 'LETS A RESUME UNDER THE SAME BUILD THROUGH, with a complete artifact of the entry in the directory',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            `${ENTRY}.json`,
          ),
          settledArtifact(),
        );

        await assertPassResumable({
          artifactsDir: scratch.path,
          pipelineDigest: RECORDED_DIGEST,
          driftAllowed: false,
        },);
      },
    },),
    it({
      name: 'LETS A RESUME ACROSS A FOREIGN BUILD THROUGH when the launch asked for it, and says so',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            `${ENTRY}.json`,
          ),
          settledArtifact(),
        );

        await assertPassResumable({
          artifactsDir: scratch.path,
          pipelineDigest: OTHER_DIGEST,
          driftAllowed: true,
        },);

        expect(printed.lines,).toEqual([
          'POOL resuming across 1 foreign pipeline because TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT=yes; a rate '
            + 'over this directory must name a required commit',
        ],);
      },
    },),
    it({
      name: 'STATES THE REFUSAL OF A FOREIGN BUILD as the guard\'s own class, a stated refusal, with its whole message',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            `${ENTRY}.json`,
          ),
          settledArtifact(),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: OTHER_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(GenerationDriftError,);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `GenerationDriftError: ${new GenerationDriftError({
            digests: [RECORDED_DIGEST,],
            digest: OTHER_DIGEST,
          },).message}`,
        );
      },
    },),
    it({
      name: 'STATES THE REFUSAL OF AN ARTIFACT THAT RECORDS NOTHING USABLE',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          '{}',
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: OTHER_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(UnplaceableArtifactError,);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `UnplaceableArtifactError: ${new UnplaceableArtifactError({ entryIds: ['Mittens',], },).message}`,
        );
      },
    },),
    it({
      name: 'STATES THE REFUSAL OF AN ARTIFACT THAT PREDATES GENERATION IDENTITY',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          JSON.stringify({
            id: 'Mittens',
            tip: FIXED_TIP,
            status: 'repaired',
          },),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: OTHER_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(LegacyPipelineError,);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `LegacyPipelineError: ${new LegacyPipelineError({ entryIds: ['Mittens',], },).message}`,
        );
      },
    },),
    it({
      name: 'STATES THE REFUSAL OF AN ARTIFACT OF ANOTHER SCHEMA GENERATION',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          JSON.stringify({
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: RECORDED_DIGEST,
            status: 'repaired',
          },),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: RECORDED_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(SchemaGenerationError,);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `SchemaGenerationError: ${new SchemaGenerationError({
            foreign: new Map([
              [
                'no schema version at all',
                ['Mittens',],
              ],
            ],),
            writes: 21,
          },).message}`,
        );
      },
    },),
    it({
      name: 'REFUSES A RESUME INTO A DIRECTORY OF GENERATION SEVENTEEN ARTIFACTS, which this pass no longer writes: it names them and '
        + 'leaves them alone, since a resumed directory would otherwise mix two generations of the consistency recount',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          JSON.stringify({
            artifactSchemaVersion: 17,
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: RECORDED_DIGEST,
          },),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: RECORDED_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(SchemaGenerationError,);
        expect(String(refusal,),).toBe(
          `SchemaGenerationError: ${new SchemaGenerationError({
            foreign: new Map([
              [
                'schema version 17',
                ['Mittens',],
              ],
            ],),
            writes: 21,
          },).message}`,
        );
      },
    },),
    it({
      name: 'REFUSES A RESUME INTO A DIRECTORY OF GENERATION EIGHTEEN ARTIFACTS, which this pass no longer writes: it names them and '
        + 'leaves them alone, since the recount of those files reads containment and inline code under the rules of eighteen and a '
        + 'resumed directory would mix them with files recounted under the rules of nineteen; the files stay readable by their reader',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          JSON.stringify({
            artifactSchemaVersion: 18,
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: RECORDED_DIGEST,
          },),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: RECORDED_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(SchemaGenerationError,);
        expect(String(refusal,),).toBe(
          `SchemaGenerationError: ${new SchemaGenerationError({
            foreign: new Map([
              [
                'schema version 18',
                ['Mittens',],
              ],
            ],),
            writes: 21,
          },).message}`,
        );
      },
    },),
    it({
      name: 'REFUSES A RESUME INTO A DIRECTORY OF GENERATION NINETEEN ARTIFACTS, which this pass no longer writes: it names them '
        + 'and leaves them alone, since their naturalness verdicts were decided under the rule of 2026-09-09 and a resumed '
        + 'directory would mix them with verdicts decided under the last-seat grace of twenty; the files stay readable by their '
        + 'reader',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          JSON.stringify({
            artifactSchemaVersion: 19,
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: RECORDED_DIGEST,
          },),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: RECORDED_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(SchemaGenerationError,);
        expect(String(refusal,),).toBe(
          `SchemaGenerationError: ${new SchemaGenerationError({
            foreign: new Map([
              [
                'schema version 19',
                ['Mittens',],
              ],
            ],),
            writes: 21,
          },).message}`,
        );
      },
    },),
    it({
      name: 'STATES THE REFUSAL OF AN ARTIFACT THAT CARRIES THE LABEL OF THIS GENERATION AND NOT ITS BODY',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        await writeFile(
          join(
            scratch.path,
            'Mittens.json',
          ),
          JSON.stringify({
            artifactSchemaVersion: 21,
            id: 'Mittens',
            tip: FIXED_TIP,
            pipelineDigest: RECORDED_DIGEST,
          },),
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: scratch.path,
          pipelineDigest: RECORDED_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).toBeInstanceOf(MislabelledArtifactError,);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `MislabelledArtifactError: ${new MislabelledArtifactError({
            entryId: 'Mittens',
            writes: 21,
          },).message}`,
        );
      },
    },),
    it({
      name: 'PASSES ON A FAILURE THAT IS NO REFUSAL as it was: a directory that cannot be listed stays a plain error',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'corpus-pass-guards-', },);
        /**
         A directory that is not there.
         */
        const missing = join(
          scratch.path,
          'no-artifacts-here',
        );

        /**
         What the guards threw.
         */
        const refusal = await refusalOf({
          artifactsDir: missing,
          pipelineDigest: OTHER_DIGEST,
          driftAllowed: false,
        },);

        expect(refusal,).not.toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(`Error: ENOENT: no such file or directory, scandir '${missing}'`,);
      },
    },),
  ],
},);

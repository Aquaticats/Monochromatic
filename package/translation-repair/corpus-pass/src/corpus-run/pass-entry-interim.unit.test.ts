/**
 Tests for what an entry stopped dry does with the progress it made, and for the record it retires once it settles.

 THE FULL PIPELINE CASES ARE IN `pass-entry.unit.test.ts`, where a scripted provider goes dry at a stage. These cover the
 branches that file cannot reach from outside: a stop that is not the every-provider-dry refusal, a stop before the
 slicing exists, a page that cannot be written, and a record that cannot be removed. Each must leave the stop as it was.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import {
  retireInterimRecord,
  shipInterimOnDryStop,
  startEntryProgress,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Original with two sections.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 Built pipeline the cases claim to have run under.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}` as PipelineDigest;

/**
 Progress of an entry that was sliced and went no further.
 */
function slicedProgress(): ReturnType<typeof startEntryProgress> {
  /**
   Preparation the entry was sliced by.
   */
  const prepared = prepareDocumentPair({
    sourceText: SOURCE_DOC,
    targetText: TARGET_DOC,
    includeFrontMatter: true,
    sealArchiveOriginal: true,
  },);
  return {
    ...startEntryProgress(),
    prepared,
    archiveText: prepared.targetText,
  };
}

/**
 The refusal that ends a run.
 */
function dryRefusal(): EveryProviderDryError {
  return new EveryProviderDryError({ measured: 'the meters of the interim cases read dry', },);
}

await describe({
  name: 'the interim ship of a stopped entry',
  children: [
    it({
      name: 'SHIPS NOTHING for a stop that is not the every-provider-dry refusal, however far the entry got',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-interim-', },);
        const publishDir = join(
          root.path,
          'fixed',
        );
        const interimDir = join(
          root.path,
          'interim',
        );
        const messages: string[] = [];
        await shipInterimOnDryStop({
          error: new Error('scripted provider failure',),
          progress: slicedProgress(),
          entryId: 'CatEntry1',
          sourceText: SOURCE_DOC,
          publishDir,
          interimDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          attempt: 1,
          l: capturingLogger({ messages, },),
        },);
        expect(await readdir(root.path,),).toEqual([],);
        expect(messages,).toEqual([],);
      },
    },),

    it({
      name: 'SHIPS NOTHING and says why when every provider goes dry before the entry is sliced',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-interim-', },);
        const messages: string[] = [];
        await shipInterimOnDryStop({
          error: dryRefusal(),
          progress: startEntryProgress(),
          entryId: 'CatEntry1',
          sourceText: SOURCE_DOC,
          publishDir: join(
            root.path,
            'fixed',
          ),
          interimDir: join(
            root.path,
            'interim',
          ),
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          attempt: 1,
          l: capturingLogger({ messages, },),
        },);
        expect(await readdir(root.path,),).toEqual([],);
        expect(messages.some(function namesTheStop(message,): boolean {
          return message.includes('before the entry was sliced',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES A PAGE IT COULD NOT WRITE in the log and returns, writing no record, so the stop it ran beside is '
        + 'still filed',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-interim-', },);
        // A published tree that is a file, so the page cannot be made under it.
        const publishDir = join(
          root.path,
          'fixed',
        );
        await writeFile(
          publishDir,
          'not a directory',
        );
        const messages: string[] = [];
        await shipInterimOnDryStop({
          error: dryRefusal(),
          progress: slicedProgress(),
          entryId: 'CatEntry1',
          sourceText: SOURCE_DOC,
          publishDir,
          interimDir: join(
            root.path,
            'interim',
          ),
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          attempt: 1,
          l: capturingLogger({ messages, },),
        },);
        expect(await readdir(root.path,),).toEqual(['fixed',],);
        expect(messages.some(function namesTheFailure(message,): boolean {
          return message.includes('the interim page could not be shipped',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES A RECORD IT COULD NOT REMOVE in the log and returns, since the artifact already stands',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-interim-', },);
        // A directory where the record would be, which a plain removal refuses.
        await mkdir(
          join(
            root.path,
            'CatEntry1.json',
          ),
          { recursive: true, },
        );
        const messages: string[] = [];
        await retireInterimRecord({
          interimDir: root.path,
          entryId: 'CatEntry1',
          l: capturingLogger({ messages, },),
        },);
        expect(messages.length,).toBe(1,);
        expect(messages[0],).toContain('its interim record could not be removed',);
      },
    },),
  ],
},);

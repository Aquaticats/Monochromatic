/**
 Guards ledger B28: the page title lexicon reads the identity preparation
 builds for every other sheet, the web lookups for the works the original
 names among it. The lexicon is told to give a work its official English
 title, and the lookup that establishes one was bought before the lexicon ran
 and never shown to it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { REFERENCE_ATTEST_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-reference';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { PAGE_TITLE_LEXICON_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-artifact';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { preparePassEntry, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the preparation writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-prepare-lexicon-context-test', },);

/**
 Roster the preparation asks.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Cache generation the preparation's caches are stamped with.
 */
const DIGEST = 'pass-prepare-lexicon-context-test' as PipelineDigest;

/**
 Original repeating one song's title, which the archive leaves unrendered.
 */
const SOURCE = '小猫唱了《猫之歌》，又唱了《猫之歌》。';

/**
 Archive rendering of the original's one block.
 */
const ARCHIVE = 'The kitten sang the song twice.';

/**
 What the web lookup found for the song.
 */
const LOOKUP_LINE = 'web lookup 《猫之歌》: "Song of the Cats", the official English title of the album track';

/**
 Builds a client answering every sheet the preparation asks, and recording
 the text of every lexicon exchange.

 @param lexicon - sink for each lexicon exchange's text

 @returns Client serving only structured calls

 @example
 ```ts
 const client = lexiconClient({ lexicon: [], },);
 ```
 */
function lexiconClient({ lexicon, }: { readonly lexicon: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Sheet asked.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      if (schema === PAGE_TITLE_LEXICON_RESPONSE_FORMAT.json_schema.name) {
        lexicon.push(request.messages
          .map(function textOf(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',),);
      }
      /**
       Reply per sheet.
       */
      const replies: Readonly<Record<string, unknown>> = {
        [REFERENCE_ATTEST_RESPONSE_FORMAT.json_schema.name]: { attested: [], },
        [PAGE_TITLE_LEXICON_RESPONSE_FORMAT.json_schema.name]: {
          titles: [{ title: 1, rendering: 'Song of the Cats', },],
        },
      };
      /**
       Reply to this sheet, one pairing where no other is scripted.
       */
      const value = replies[schema] ?? { pairs: [{ source: 0, target: 0, },], };
      if (!request.validate(value,))
        throw new Error(`scripted ${schema} reply failed validator`,);
      return { kind: 'ok', value, rawText: JSON.stringify(value,), };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

await describe({
  name: 'preparePassEntry shows the page title lexicon the page\'s identity (ledger B28)',
  children: [
    it({
      name: 'THREADS the work-title lookup, bought before the lexicon runs, into every lexicon call',
      fn: async () => {
        /** Lexicon exchanges. */
        const lexicon: string[] = [];
        /** Directory this case owns for its entry caches. */
        await using scratch = await scratchDir({ prefix: 'pass-prepare-lexicon-context-', },);
        const dir = scratch.path;
        await preparePassEntry({
          client: lexiconClient({ lexicon, },),
          entryId: 'CatEntry',
          entryCacheDir: dir,
          pipelineDigest: DIGEST,
          modelIds: ROSTER,
          sourceText: SOURCE,
          targetText: ARCHIVE,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          outsideReads: {
            ...NO_OUTSIDE_READS,
            workTitles: async () => [LOOKUP_LINE,],
          },
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // The lexicon ran, so the case reads calls that exist.
        expect(lexicon.length,).toBeGreaterThan(0,);
        for (const text of lexicon)
          expect(text,).toContain(LOOKUP_LINE,);
      },
    },),
    it({
      name: 'THREADS the names other entries declare and the ARCHIVE\'s own declared name, read off the archive '
        + 'rather than the original, into every lexicon call',
      fn: async () => {
        /** Lexicon exchanges. */
        const lexicon: string[] = [];
        /** Directory this case owns for its entry caches. */
        await using scratch = await scratchDir({ prefix: 'pass-prepare-lexicon-context-', },);
        const dir = scratch.path;
        await preparePassEntry({
          client: lexiconClient({ lexicon, },),
          entryId: 'CatEntry',
          entryCacheDir: dir,
          pipelineDigest: DIGEST,
          modelIds: ROSTER,
          sourceText: `---\nname: 咪咪\n---\n\n${SOURCE}\n`,
          targetText: `---\nname: Mittens\n---\n\n${ARCHIVE}\n`,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          outsideReads: {
            ...NO_OUTSIDE_READS,
            corpusNames: async () => [{ source: '小猫', renderings: ['Little Cat',], entryId: 'OtherCat', },],
          },
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // The lexicon ran, so the case reads calls that exist.
        expect(lexicon.length,).toBeGreaterThan(0,);
        for (const text of lexicon) {
          expect(text,).toContain('- 小猫 (entry OtherCat): "Little Cat"',);
          expect(text,).toContain('- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',);
        }
      },
    },),
  ],
},);

/**
 Guards ledger B28: the archive block review that preparation runs over an
 unclaimed archive block reads the declared names and the cited references
 preparation already holds, on its review sheet and on its correction slate.
 Both were in hand when `pass-prepare.ts` called the review and neither was
 passed, so a detail the translator took from a page the original links was
 judged against the original alone.

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
import { ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-archive';
import { preparePassEntry, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the preparation writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-prepare-archive-review-context-test', },);

/**
 Roster the preparation asks: four seats, so the correction slate has judges
 who did not write the revision.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Cache generation the preparation's caches are stamped with.
 */
const DIGEST = 'pass-prepare-archive-review-context-test' as PipelineDigest;

/**
 Original repeating one song's title, in one block.
 */
const SOURCE = '小猫唱了《猫之歌》，又唱了《猫之歌》。';

/**
 Archive rendering of the original's block, then a block no source block
 claims.
 */
const ARCHIVE = 'The kitten sang the song.\n\nMittens had a brother who sat by the stove.';

/**
 What the one page the original links says.
 */
const REFERENCE_LINES = '- reference 1 https://cats.example/posts/mittens ("Mittens"): Mittens had a brother who sat by the stove.';

/**
 Name the page title lexicon settles for the song, which preparation carries
 in the declared identity.
 */
const SONG_TITLE = 'Song of the Cat';

/**
 Builds a client answering every sheet the preparation asks, and recording
 the text of every archive review and correction slate exchange.

 @param reviews - sink for each archive review exchange's text

 @param selections - sink for each correction slate exchange's text

 @returns Client serving only structured calls

 @example
 ```ts
 const client = contextClient({ reviews: [], selections: [], },);
 ```
 */
function contextClient(
  {
    reviews,
    selections,
  }: {
    readonly reviews: string[];
    readonly selections: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Sheet asked.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Every message of the exchange, as the model reads it.
       */
      const text = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      if (schema === ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT.json_schema.name)
        reviews.push(text,);
      if (text.includes('CURRENT ARCHIVE BLOCK',))
        selections.push(text,);
      /**
       Reply per sheet.
       */
      const replies: Readonly<Record<string, unknown>> = {
        [REFERENCE_ATTEST_RESPONSE_FORMAT.json_schema.name]: { attested: [], },
        [PAGE_TITLE_LEXICON_RESPONSE_FORMAT.json_schema.name]: {
          titles: [{ title: 1, rendering: SONG_TITLE, },],
        },
        [ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT.json_schema.name]: {
          disposition: 'revise',
          sourceQuote: '',
          replacementText: 'Mittens sat by the stove.',
          assessment: 'Remove the brother the original does not name.',
        },
        candidate_ballot: {
          best: 1,
          assessment: 'Keep the archive block.',
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
  name: 'preparePassEntry shows the archive block review what preparation holds (ledger B28)',
  children: [
    it({
      name: 'THREADS the cited references and the declared names to the reviewers and the correction selectors',
      fn: async () => {
        /** Archive review exchanges. */
        const reviews: string[] = [];
        /** Correction slate exchanges. */
        const selections: string[] = [];
        /** Directory this case owns for its entry caches. */
        await using scratch = await scratchDir({ prefix: 'pass-prepare-archive-review-context-', },);
        const dir = scratch.path;
        const paired = await preparePassEntry({
          client: contextClient({ reviews, selections, },),
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
            references: async () => REFERENCE_LINES,
          },
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /** Declared identity preparation built, which the review is to read. */
        const identityContext = paired.prepared.identityContext ?? '';
        // The case holds only while preparation carried both and both rounds ran.
        expect(identityContext,).toContain(SONG_TITLE,);
        expect(reviews.length,).toBeGreaterThan(0,);
        expect(selections.length,).toBeGreaterThan(0,);
        for (const text of [...reviews, ...selections,]) {
          expect(text,).toContain(REFERENCE_LINES,);
          expect(text,).toContain(identityContext,);
        }
      },
    },),
  ],
},);

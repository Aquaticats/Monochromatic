/**
 Guards ledger H16 at the preparation: a page repeating a title the archive
 leaves unpaired carries one settled rendering of it in the identity context
 every slice sheet reads, asked of the roster the preparation's hook hands over
 (ledger X12), and a page repeating none asks nothing.

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
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import { PAGE_TITLE_LEXICON_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-artifact';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor';
import { preparePassEntry, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the preparation writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-prepare-page-titles-test', },);

/**
 Roster the preparation started on.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
];

/**
 Roster the hook hands every round.
 */
const RESEATED: readonly RosterModelId[] = [
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
];

/**
 Cache generation the preparation's caches are stamped with.
 */
const DIGEST = 'pass-prepare-page-titles-test' as PipelineDigest;

/**
 Archive rendering no heading, so the title stays unpaired.
 */
const ARCHIVE = 'The kitten sang.\n\nShe fell asleep.\n';

/**
 Builds a client rendering every title alike on the lexicon sheet, refusing
 every other sheet, and recording the seats the lexicon asked.

 @param lexiconAsked - sink for the seat of every lexicon call

 @returns Client serving only structured calls

 @example
 ```ts
 const client = lexiconClient({ lexiconAsked: [], },);
 ```
 */
function lexiconClient({ lexiconAsked, }: { readonly lexiconAsked: RosterModelId[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.responseFormat?.json_schema.name !== PAGE_TITLE_LEXICON_RESPONSE_FORMAT.json_schema.name)
        return { kind: 'schema-mismatch', rawText: '{}', detail: 'fixture answers the lexicon only', };
      lexiconAsked.push(request.modelId,);
      /**
       Every seat's rendering.
       */
      const value: unknown = { titles: [{ title: 1, rendering: 'Song of the Cat', },], };
      return request.validate(value,)
        ? { kind: 'ok', value, rawText: JSON.stringify(value,), }
        : { kind: 'schema-mismatch', rawText: '{}', detail: 'fixture reply refused', };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Prepares an original against the fixture archive, the hook re-seating every round.

 @param sourceText - original page

 @returns Identity context, findings and the seats the lexicon asked

 @example
 ```ts
 const run = await prepared({ sourceText: '## 猫之歌\n\n《猫之歌》\n', },);
 ```
 */
async function prepared({ sourceText, }: { readonly sourceText: string; },) {
  /**
   Seat of every lexicon call.
   */
  const lexiconAsked: RosterModelId[] = [];
  await using cacheDir = await scratchDir({ prefix: 'pass-prepare-page-titles-', },);
  /**
   The preparation.
   */
  const paired = await preparePassEntry({
    client: lexiconClient({ lexiconAsked, },),
    entryId: 'CatEntry',
    entryCacheDir: cacheDir.path,
    pipelineDigest: DIGEST,
    modelIds: ROSTER,
    sourceText,
    targetText: ARCHIVE,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    beforeItem: async (): Promise<BenchSeating> => ({ modelIds: RESEATED, }),
    outsideReads: NO_OUTSIDE_READS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    identityContext: paired.prepared.identityContext ?? '',
    findings: paired.prepared.alignmentFindings,
    lexiconAsked,
  };
}

await describe({
  name: 'preparePassEntry settles the titles a page repeats (ledger H16)',
  children: [
    it({
      name: 'CARRIES ONE SETTLED RENDERING OF A REPEATED UNPAIRED TITLE on every slice sheet\'s identity context, '
        + 'asked of the roster the hook hands over',
      fn: async () => {
        /**
         Preparation of a page repeating a title.
         */
        const repeating = await prepared({ sourceText: '## 猫之歌\n\n小猫唱了《猫之歌》。\n\n它睡着了。\n', },);
        expect({
          heading: repeating.identityContext.includes('TITLES THIS PAGE REPEATS THAT THE ARCHIVE DOES NOT RENDER',),
          rendering: repeating.identityContext.includes('- 猫之歌 (2 places on the page): "Song of the Cat"',),
          reported: repeating.findings.some(function reportsLexicon(finding,): boolean {
            return finding.startsWith('page title lexicon settled 1 of 1',);
          },),
          askedAny: repeating.lexiconAsked.length > 0,
          offReseated: repeating.lexiconAsked.filter(function outside(seat,): boolean {
            return !RESEATED.includes(seat,);
          },),
        },).toEqual({
          heading: true,
          rendering: true,
          reported: true,
          askedAny: true,
          offReseated: [],
        },);
      },
    },),
    it({
      name: 'ASKS THE LEXICON NOTHING for a page repeating no title',
      fn: async () => {
        /**
         Preparation of a page naming a title once.
         */
        const once = await prepared({ sourceText: '小猫唱了《猫之歌》。\n\n它睡着了。\n', },);
        expect({
          asked: once.lexiconAsked,
          heading: once.identityContext.includes('TITLES THIS PAGE REPEATS',),
        },).toEqual({
          asked: [],
          heading: false,
        },);
      },
    },),
  ],
},);

/**
 Guard that the cited references and the attested details a prepared
 document carries reach the requests `translateDocument` sends: the judges'
 sheets carry what the pages the original cites say, and the translators'
 sheets carry the archive details a cited page states.

 WHY THIS FILE EXISTS: `document-pictures-reach-the-wire.unit.test.ts`
 records why reading the driver's return value proves nothing about what a
 model was shown. The references and the attested details wire through the
 same driver into the run's key and every slice's round, and no driver case
 handed a document either, so nothing showed they arrive.

 A run handed neither sends neither block, the positive control an
 assertion that always passes would otherwise look identical to.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  translateDocument,
  type TranslateModels,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the driver under test.
 */
const l = tagged({ tag: 'document-references-reach-the-wire-test', },);

/**
 Models that render each slice, the roster the pictures guard seats.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Rosters the driver seats for every run in this file.
 */
const MODELS: TranslateModels = {
  translatorModelIds: TRANSLATORS,
  judgeModelIds: [
    ...TRANSLATORS,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

/**
 Original: one section in which the cat naps in a box.
 */
const SOURCE_TEXT = `## 第一节

猫猫窝在纸箱里打盹。
`;

/**
 Archive translation, awkward on purpose so a fresh rendering is never the
 text already there.
 */
const TARGET_TEXT = `## Section one

The cat is doing the napping inside of the cardboard box.
`;

/**
 What every translator renders.
 */
const FRESH_RENDERING = `## Section one

The cat naps inside the cardboard box.
`;

/**
 What the pages the original cites say, distinctive so finding it in a
 request can only mean the reference channel carried it.
 */
const REFERENCES = '- reference 1 https://shelter.example/whiskers ("Whiskers adopted"): Whiskers came home in a cardboard box.';

/**
 Archive quote a cited page states, distinctive for the same reason.
 */
const ATTESTED_QUOTE = 'inside of the cardboard box';

/**
 Builds a client that renders one fixed translation, ballots for the first
 candidate, and records what every exchange was sent before answering.

 @param requests - log every exchange is appended to, as schema and text

 @returns Client over both halves of a round

 @example
 ```ts
 const client = recordingClient({ requests: [], },);
 ```
 */
function recordingClient(
  { requests, }: { readonly requests: { readonly schema: string; readonly content: string; }[]; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Which sheet this is.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      requests.push({
        schema,
        content: request.messages
          .map(function textOf(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',),
      },);
      /**
       Reply this sheet gets.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: FRESH_RENDERING, }
        : (schema === 'candidate_ballot')
          ? {
            best: 1,
            assessment: 'the box is the whole point',
          }
          : undefined;
      if ((value === undefined) || (!request.validate(value,)))
        throw new Error(`no fixture reply for ${schema}`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Drives the driver once over the pair, with or without the reference
 evidence, recording every exchange it sends.

 @param withReferences - whether the prepared pair carries references and an
 attested detail

 @returns Every exchange sent, by schema

 @example
 ```ts
 const requests = await runDocument({ withReferences: true, },);
 ```
 */
async function runDocument(
  { withReferences, }: { readonly withReferences: boolean; },
): Promise<readonly { readonly schema: string; readonly content: string; }[]> {
  /**
   Exchanges the run sends.
   */
  const requests: { readonly schema: string; readonly content: string; }[] = [];

  /**
   Preparation the driver slices its work from, handed the evidence the way
   the corpus pass hands it.
   */
  const prepared = prepareDocumentPair({
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    ...(withReferences
      ? {
        referenceContext: REFERENCES,
        attestedDetails: [{
          archiveQuote: ATTESTED_QUOTE,
          reference: 1,
          referenceQuote: 'came home in a cardboard box',
          voices: 2,
          heard: 3,
        },],
      }
      : {}),
  },);
  await translateDocument({
    client: recordingClient({ requests, },),
    prepared,
    models: MODELS,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return requests;
}

await describe({
  name: 'cited references and attested details reach the wire',
  children: [
    it({
      name: 'SENDS the references to every judge and the attested detail to every translator',
      fn: async () => {
        /**
         Every exchange the run sent.
         */
        const requests = await runDocument({ withReferences: true, },);

        /**
         Sheets each half was shown.
         */
        const judges = requests.filter(function isJudge({ schema, },): boolean {
          return schema === 'candidate_ballot';
        },);
        const translators = requests.filter(function isTranslator({ schema, },): boolean {
          return schema === 'translation_report';
        },);
        expect(judges.length,).toBeGreaterThan(0,);
        expect(translators.length,).toBeGreaterThan(0,);
        expect(judges.every(function showsReferences({ content, },): boolean {
          return content.includes('CITED REFERENCES, EVIDENCE ONLY.',) && content.includes(REFERENCES,);
        },),).toBe(true,);
        expect(translators.every(function showsAttested({ content, },): boolean {
          return content.includes('ATTESTED DETAILS',) && content.includes(`the ARCHIVE's "${ATTESTED_QUOTE}"`,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'SENDS NEITHER BLOCK when the pair carries neither, the positive control for the case beside it',
      fn: async () => {
        /**
         Every exchange the run sent.
         */
        const requests = await runDocument({ withReferences: false, },);
        expect(requests.length,).toBeGreaterThan(0,);
        expect(requests.some(function showsEither({ content, },): boolean {
          return content.includes('CITED REFERENCES, EVIDENCE ONLY.',) || content.includes('ATTESTED DETAILS',);
        },),).toBe(false,);
      },
    },),
  ],
},);

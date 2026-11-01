/**
 Guards ledger B28: the coverage judges insertion admission asks read the
 declared names preparation holds. Coverage looks for a fact, a name or a
 number of a Chinese passage in the English, and its house rules point to a
 DECLARED NAMES block; without it, a passage whose most specific content is a
 name the page renders by a declared handle reads as uncovered.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  makeInsertionChunk,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { decidePassInsertionAdmission, } from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Roster the coverage question goes to.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 Logger the admission writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-insertion-admission-identity-test', },);

/**
 Source passage no target block pairs, naming the cat by her Chinese name.
 */
const PASSAGE = '咪咪在窗台上睡着了。';

/**
 Declared identity preparation built for the page.
 */
const IDENTITY_CONTEXT = '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"';

/**
 Preparation holding one source-only slice and the page's declared identity.
 */
const PREPARED: PreparedDocumentPair = {
  sourceText: `${PASSAGE}\n${'猫在窗台晒太阳。'.repeat(20,)}`,
  targetText: `## Cats\n\n${'The cat sleeps in warm sunlight. '.repeat(20,)}`,
  slices: [{
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: PASSAGE.length,
      text: PASSAGE,
    },
    target: makeInsertionChunk({
      sliceIndex: 0,
      offset: 0,
    },),
  },],
  lineStructuredSliceIndices: new Set(),
  declaredNames: [],
  alignmentFindings: [],
  unclaimedTargetBlocks: [],
  alignmentPairCount: 1,
  identityContext: IDENTITY_CONTEXT,
};

/**
 Client answering every coverage call "none" and recording each call's text.

 @param sheets - sink for the text of every coverage call

 @returns Client serving only the coverage stage

 @example
 ```ts
 const client = recordingClient({ sheets: [], },);
 ```
 */
function recordingClient({ sheets, }: { readonly sheets: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by coverage',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      sheets.push(request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',),);
      /**
       Absence, with its reason.
       */
      const value: unknown = {
        coverage: 'none',
        copiedSpan: '',
        assessment: 'scripted',
      };
      if (!request.validate(value,))
        throw new Error('scripted coverage reply failed wire guard',);
      return { kind: 'ok', value: value as ValueT, rawText: JSON.stringify(value,), };
    },
    quotas: async () => {
      throw new Error('quotas unused by coverage',);
    },
  };
}

await describe({
  name: 'insertion admission shows coverage the declared names (ledger B28)',
  children: [
    it({
      name: 'THREADS preparation\'s declared identity into every coverage call',
      fn: async () => {
        /** Text of every coverage call. */
        const sheets: string[] = [];
        await decidePassInsertionAdmission({
          client: recordingClient({ sheets, },),
          prepared: PREPARED,
          modelIds: ROSTER,
          overlap: 1,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        // The admission asked, so the case reads calls that exist.
        expect(sheets.length,).toBeGreaterThan(0,);
        for (const sheet of sheets)
          expect(sheet,).toContain(IDENTITY_CONTEXT,);
      },
    },),
  ],
},);

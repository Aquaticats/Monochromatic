/**
 Tests for the follow-up turn on a candidate that failed the publication rule (ledger B1100).

 THE TURN IS ASKED OF THE AUTHOR, and where the author's reasoning runaway is re-asked of another translator that model
 gets a conversation of its own: the translation named as another translator's, handed over as plain text and not as a
 turn the asked model wrote. A failure that arrives as the caller stops is written on the seat's line and marked written.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createReaskClaims,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { askAboutFindings, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Translate repair ask tests

/**
 Candidate whose author is asked.
 */
const VOICE = {
  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  value: { translation: 'The cat naps.', },
} as const;

await describe({
  name: askAboutFindings.name,
  concurrency: 1,
  children: [
    it({
      name: 'ASKS THE AUTHOR WITH ITS OWN TURN AND HANDS THE SEAT A CONVERSATION FOR ANOTHER TRANSLATOR: the author\'s '
        + 'conversation ends on its own earlier answer, the other one holds the translation as plain text',
      fn: async () => {
        /**
         Requests the call carried.
         */
        const requests: ChatJsonRequest<unknown>[] = [];
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
            requests.push(request as ChatJsonRequest<unknown>,);
            return { kind: 'schema-mismatch', rawText: '', detail: 'fixture', };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };
        const answer = await askAboutFindings({
          client,
          voice: VOICE,
          findings: ['Your translation lost a heading.',],
          priorMessages: [{ role: 'user', content: 'the sheet', },],
          sourceText: 'x',
          reaskSeat: { role: 'translator', bench: [SEAT_SYNTHETIC_VISION_WITHHELD,], claims: createReaskClaims(), },
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'ask-test', },),
        },);
        expect(answer.heard,).toBe(false,);
        const [seen,] = requests;
        expect(seen?.modelId,).toBe(SEAT_SYNTHETIC_VISION_WITHHELD,);
        expect(seen?.messages.at(-2,)?.role,).toBe('assistant',);
        expect(seen?.reaskSeat?.otherModelMessages?.some(function isAssistant(message,): boolean {
          return message.role === 'assistant';
        },),).toBe(false,);
        expect(JSON.stringify(seen?.reaskSeat?.otherModelMessages,).includes('The translation lost a heading.',),).toBe(true,);
      },
    },),
  ],
},);

//endregion Translate repair ask tests

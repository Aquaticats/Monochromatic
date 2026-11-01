/**
 Tests for the live-run window check.

 WHAT THESE PIN is that the check can fail. Every other test in this family
 proves the window is forwarded against a synthetic client, so the only thing
 left for the live check to catch is the case where the code path is right and
 the wire is wrong. A witness that silently passed would be worse than no
 witness, because the run would then carry a check nobody could distinguish
 from a working one.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  assertWindowReachedJudges,
  WINDOW_LABEL,
  WindowEvidenceError,
  witnessSheets,
} from '../../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Client answering everything, recording what it was asked.

 @returns Client plus the requests it served

 @example
 ```ts
 const rig = recordingClient();
 ```
 */
function recordingClient(): {
  readonly client: ModelClient;
  readonly served: string[];
} {
  /**
   Sheets that reached the underlying client.
   */
  const served: string[] = [];

  return {
    served,
    client: {
      chatText: async () => {
        throw new Error('chatText unused',);
      },
      quotas: async () => {
        throw new Error('quotas unused',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        served.push(request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',),);
        return {
          kind: 'ok',
          value: { best: 0, } as ValueT,
          rawText: '{"best":0}',
        };
      },
    },
  };
}

/**
 Builds one exchange request carrying given text.

 @param content - sheet body

 @returns Request shaped like the stages send

 @example
 ```ts
 const request = sheetOf({ content: '猫。', },);
 ```
 */
function sheetOf(
  { content, }: { readonly content: string; },
): ChatJsonRequest<unknown> {
  return {
    modelId: 'hf:cat/Cat-A' as unknown as ChatJsonRequest<unknown>['modelId'],
    messages: [{
      role: 'user',
      content,
    },],
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    validate: function accepts(value: unknown,): value is unknown {
      return value !== undefined;
    },
  } as unknown as ChatJsonRequest<unknown>;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: witnessSheets.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FORWARDS EVERY CALL rather than answering on the model\'s behalf, since the '
            + 'first slice\'s rows are kept and a substituted answer would make them unlike every '
            + 'later row',
          fn: async () => {
            const rig = recordingClient();
            const witness = witnessSheets({ client: rig.client, },);

            /**
             What the wrapper returned for a sheet.
             */
            const outcome = await witness.client
              .chatJson(sheetOf({ content: 'A sheet about a cat.', },),);

            expect(outcome.kind,).toBe('ok',);
            expect(rig.served,).toEqual(['A sheet about a cat.',],);
          },
        },),
        it({
          name: 'records sheets in the order they went out, joining every message of one exchange, so '
            + 'a label carried by a system message counts as much as one in the user turn',
          fn: async () => {
            const witness = witnessSheets({ client: recordingClient().client, },);
            await witness.client
              .chatJson(sheetOf({ content: 'first', },),);
            await witness.client
              .chatJson(sheetOf({ content: 'second', },),);

            expect(witness.sheets,).toEqual(['first',
              'second',],);
          },
        },),
      ],
    },),

    describe({
      name: assertWindowReachedJudges.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts a run whose wide sheets all carried the window',
          fn: async () => {
            assertWindowReachedJudges({
              sheets: ['narrow sheet',
                `wide sheet with ${WINDOW_LABEL} in it`,
                `another with ${WINDOW_LABEL}`,],
              expected: 2,
            },);
          },
        },),
        it({
          name: 'REFUSES a run where the window reached nobody, which is the failure the whole check '
            + 'exists for: all three arms would then have seen the same evidence and every later row '
            + 'would report a false null',
          fn: async () => {
            expect(function refuses() {
              assertWindowReachedJudges({
                sheets: ['narrow sheet',
                  'another narrow sheet',],
                expected: 2,
              },);
            },).toThrow(WindowEvidenceError,);
          },
        },),
        it({
          name: 'REFUSES A PARTIAL FORWARD, where one judge of several saw the window: that moves a '
            + 'rate slightly in the direction the trial expects, so a presence check would pass it and '
            + 'the run would report a real effect at a fraction of its true size',
          fn: async () => {
            expect(function refuses() {
              assertWindowReachedJudges({
                sheets: [`only this one has ${WINDOW_LABEL}`,
                  'this one does not',
                  'nor this',],
                expected: 3,
              },);
            },).toThrow(WindowEvidenceError,);
          },
        },),
        it({
          name: 'refuses a run carrying MORE labelled sheets than the arms bought, since the window '
            + 'leaking into a narrow arm collapses the comparison just as completely as it missing '
            + 'from the wide one',
          fn: async () => {
            expect(function refuses() {
              assertWindowReachedJudges({
                sheets: [`wide ${WINDOW_LABEL}`,
                  `narrow that should not have ${WINDOW_LABEL}`,],
                expected: 1,
              },);
            },).toThrow(WindowEvidenceError,);
          },
        },),
        it({
          name: 'names both counts in the message, so an operator reading a stopped run learns whether '
            + 'the window missed entirely or only partly',
          fn: async () => {
            /**
             What refuses raised, read for its class as well as its wording.
             */
            const refusalOfRefuses = caught(function refuses() {
              assertWindowReachedJudges({
                sheets: [`one ${WINDOW_LABEL}`,],
                expected: 6,
              },);
            },);

            expect(refusalOfRefuses,).toBeInstanceOf(WindowEvidenceError,);
            expect((refusalOfRefuses as Error).message,).toContain('1 of the judge sheets carried SURROUNDING ORIGINAL where 6',);
          },
        },),
      ],
    },),
  ],
},);

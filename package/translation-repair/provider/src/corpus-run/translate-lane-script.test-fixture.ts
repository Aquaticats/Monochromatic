import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '../../dist/final/node/index.mjs';
import { replyWith, } from '../scripted-reply-outcome.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';

//region Translate lane script
// A MODEL CLIENT ANSWERING THE TRANSLATE LANE FROM A SCRIPT: each translator
// returns the rendering written for it, and every judge backs the candidate
// whose text carries one needle, so a case reads the round it scripted and
// never one a network happened to produce.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. A translator the script names nothing
// for answers with prose around its JSON, which fails the wire guard and
// costs the lane that voice, exactly as a live model's malformed reply does.
// No call goes to a network, and a text call is refused by name because the
// lane makes none.

/**
 Refuses a text call, which no stage of the lane makes.

 @example
 ```ts
 await refuseLaneText();
 ```
 */
function refuseLaneText(): Promise<never> {
  return Promise.reject(new Error('the translate lane makes no text call',),);
}

/**
 Refuses a quota read, which no stage of the lane makes.

 @example
 ```ts
 await refuseLaneQuotas();
 ```
 */
function refuseLaneQuotas(): Promise<never> {
  return Promise.reject(new Error('the translate lane reads no quota',),);
}

/**
 Builds the scripted client.

 @param renderings - what each translator returns, absent for a translator
 that answers unusably

 @param needle - text the judges back, so the candidate carrying it wins

 @returns Client answering both stages of the lane and refusing any text call

 @example
 ```ts
 const client = translateLaneClient({ renderings: { 'minimax-m3': 'The cat naps.', }, needle: 'naps', },);
 ```
 */
export function translateLaneClient(
  {
    renderings,
    needle,
  }: {
    readonly renderings: Readonly<Record<string, string>>;
    readonly needle: string;
  },
): ModelClient {
  return {
    chatText: refuseLaneText,
    chatJson: function answerLane<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Stage the request is for, read off its response format.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name;
      if (stage === 'translation_report') {
        /**
         Rendering scripted for the translator that was asked.
         */
        const written = renderings[request.modelId];
        if (written === undefined) {
          return Promise.resolve({
            kind: 'schema-mismatch',
            rawText: 'Here is my translation:\n{"translation": "..."}',
            detail: 'prose around the JSON',
          },);
        }
        return Promise.resolve(replyWith({
          report: { translation: written, },
          request,
        },),);
      }

      /**
       Sheet the judge was shown, whole.
       */
      const sheet = request.messages
        .map(function toContent(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);

      return Promise.resolve(replyWith({
        report: {
          best: candidateCarrying({
            content: sheet,
            needle,
          },),
          assessment: 'scripted',
        },
        request,
      },),);
    },
    quotas: refuseLaneQuotas,
  };
}

//endregion Translate lane script

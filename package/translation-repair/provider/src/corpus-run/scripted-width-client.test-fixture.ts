/**
 A model client answering each structured stage from a script, for the cases
 that gather a slice's work for the editor width comparison. A stage the
 script does not name is refused by name, so a round the case did not expect
 the probe to buy is reported as that round rather than as an answer shaped
 like one.

 @module
 */

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '../../dist/final/node/index.mjs';

/**
 Refuses a text call, which no stage of the width probe makes.

 @throws Error on any invocation
 */
function unscriptedText(): never {
  throw new Error('chatText is not scripted',);
}

/**
 Refuses a quota read, which no stage of the width probe makes.

 @throws Error on any invocation
 */
function unscriptedQuotas(): never {
  throw new Error('quotas is not scripted',);
}

/**
 Builds a client whose structured answers come from the script.

 @param script - answer per stage, by the stage name in the response format

 @returns Client that answers scripted stages and refuses every other call

 @example
 ```ts
 const client = scriptedClient({ script: { critic_report: { issues: [], }, }, },);
 ```
 */
export function scriptedClient(
  { script, }: { readonly script: Readonly<Record<string, unknown>>; },
): ModelClient {
  return {
    chatText: unscriptedText,
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Stage the request is for, read off its response format.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';
      /**
       Answer scripted for that stage.
       */
      const answer = script[stage];
      if (answer === undefined)
        return Promise.reject(new Error(`the script holds no answer for the ${stage} stage`,),);
      if (!request.validate(answer,))
        return Promise.reject(new Error(`the scripted ${stage} answer failed the stage's own guard`,),);
      return Promise.resolve({
        kind: 'ok',
        value: answer,
        rawText: JSON.stringify(answer,),
      },);
    },
    quotas: unscriptedQuotas,
  };
}

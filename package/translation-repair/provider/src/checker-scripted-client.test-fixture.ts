import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '../dist/final/node/index.mjs';
import { replyWith, } from './scripted-reply-outcome.test-fixture.ts';

//region Checker scripted client
// A CLIENT ANSWERING EVERY RESOLUTION CHECKER WITH ONE SCRIPTED VERDICT PER
// ISSUE OF THE SHEET, chosen by model id, for the cases that run the checker
// stage without a live provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The reply goes through the live request's
// own wire guard, so a fixture that stopped matching the wire fails here.

/**
 Refuses a text call, which this scripted protocol never makes.

 @throws {@link Error} on any invocation
 */
function unexpectedText(): never {
  throw new Error('chatText unused by the checker stage',);
}

/**
 Refuses a quota read, which fixtures cannot answer.

 @throws {@link Error} on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('quotas unused by the checker stage',);
}

/**
 Client answering each checker with the verdicts the script names for it.

 @param verdictsFor - verdicts each model casts, one per issue of the sheet in
 sheet order, chosen by model id

 @param asked - shared log of every model id the client was asked, in order

 @returns Client the stage calls

 @example
 ```ts
 const client = checkerScriptedClient({ verdictsFor: function allFixed() { return ['fixed',]; }, asked: [], },);
 ```
 */
export function checkerScriptedClient(
  {
    verdictsFor,
    asked,
  }: {
    readonly verdictsFor: (modelId: string,) => readonly string[];
    readonly asked: string[];
  },
): ModelClient {
  return {
    chatText: unexpectedText,
    /**
     Answers with the scripted verdicts, through the wire guard.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      asked.push(request.modelId,);
      /**
       Verdicts this model casts.
       */
      const verdicts = verdictsFor(request.modelId,);
      return Promise.resolve(replyWith({
        report: {
          checks: verdicts.map(function toCheck(
            verdict,
            index,
          ) {
            return {
              issue: index + 1,
              verdict,
            };
          },),
        },
        request,
      },),);
    },
    quotas: unexpectedQuotas,
  };
}

//endregion Checker scripted client

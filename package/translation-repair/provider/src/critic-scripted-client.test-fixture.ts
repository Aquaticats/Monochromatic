import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '../dist/final/node/index.mjs';

//region Critic scripted client
// A CLIENT ANSWERING EVERY CRITIC WITH A SCRIPTED REPORT, chosen by model id,
// for cases that need a critic bench without a live provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The chunk-critic-phase and
// critic-stage-attribution tests kept their own copy of this client; both
// now import it from here.

/**
 Refuses a text call, which no critic makes.

 @throws Error on any invocation
 */
function unexpectedCriticText(): never {
  throw new Error('chatText unused',);
}

/**
 Refuses a quota read, which no critic makes.

 @throws Error on any invocation
 */
function unexpectedCriticQuotas(): never {
  throw new Error('quotas unused',);
}

/**
 Client answering each critic with a scripted report.

 @param reportFor - report each model returns, chosen by model id

 @returns Client honoring that script

 @example
 ```ts
 const client = criticClient({ reportFor: function noIssues() { return { issues: [], }; }, },);
 ```
 */
export function criticClient(
  {
    reportFor,
  }: {
    readonly reportFor: (modelId: string,) => unknown;
  },
): ModelClient {
  return {
    /**
     Text calls are outside this scripted protocol.
     */
    chatText: unexpectedCriticText,
    /**
     Answers with the scripted report, refused when the critic guard would
     refuse it.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Scripted report for the answering model.
       */
      const scripted = reportFor(request.modelId,);
      if (!request.validate(scripted,))
        return Promise.reject(new Error('scripted report failed the critic guard',),);
      return Promise.resolve({
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      },);
    },
    /**
     Fixtures cannot query live provider resources.
     */
    quotas: unexpectedCriticQuotas,
  };
}

//endregion Critic scripted client

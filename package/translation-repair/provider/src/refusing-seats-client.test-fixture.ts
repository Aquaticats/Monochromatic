import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '../dist/final/node/index.mjs';

//region Refusing-seats client
// A CLIENT THE ROUTER REFUSES SOME SEATS ON, for cases that need a bench
// whose quorum is out of reach: a refused seat throws the router's own
// refusal, which the round counts as unreachable, and every other seat
// answers one scripted reply.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The pairing stages share it;
// `lane-contest-stage.unit.test.ts` keeps a ballot-writing copy of its own.

/**
 Refuses a text call, which no JSON stage makes.

 @throws Error on any invocation
 */
function unexpectedText(): never {
  throw new Error('chatText unused by a JSON stage',);
}

/**
 Refuses a quota read, which no stage makes.

 @throws Error on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('quotas unused by a JSON stage',);
}

/**
 What one seat's call comes to: the router's refusal for a refused seat, the
 scripted reply for every other.

 @param request - the stage's call, whose guard the reply must pass

 @param reply - JSON every seat that answers gives

 @param refused - seats the router refuses for want of a wet provider

 @returns The answering seat's outcome

 @throws NoProviderForModelError for a refused seat, as the router raises it

 @throws Error when the reply fails the stage's guard, which would mean the
 case scripted a reply the stage cannot read

 @example
 ```ts
 const outcome = scriptedOutcome({ request, reply: '{"pairs":[]}', refused: [], },);
 ```
 */
function scriptedOutcome<ValueT,>(
  {
    request,
    reply,
    refused,
  }: {
    readonly request: ChatJsonRequest<ValueT>;
    readonly reply: string;
    readonly refused: readonly RosterModelId[];
  },
): ChatJsonOutcome<ValueT> {
  if (refused.includes(request.modelId,)) {
    throw new NoProviderForModelError({
      modelId: request.modelId,
      reason: 'every provider serving this cat is out of budget',
    },);
  }
  /**
   This seat's reply, parsed.
   */
  const value: unknown = JSON.parse(reply,);
  if (!request.validate(value,))
    throw new Error('scripted reply failed the stage guard',);
  return {
    kind: 'ok',
    value,
    rawText: reply,
  };
}

/**
 Builds a client the router refuses some seats on, every other seat
 answering one reply.

 @param reply - JSON every seat that answers gives, checked by the stage's own guard

 @param refused - seats the router refuses for want of a wet provider

 @returns Scripted client

 @example
 ```ts
 const client = refusingSeatsClient({ reply: '{"pairs":[]}', refused: [], },);
 ```
 */
export function refusingSeatsClient(
  {
    reply,
    refused,
  }: {
    readonly reply: string;
    readonly refused: readonly RosterModelId[];
  },
): ModelClient {
  return {
    /**
     Text calls are outside this scripted protocol.
     */
    chatText: unexpectedText,
    /**
     Fixtures cannot query live provider resources.
     */
    quotas: unexpectedQuotas,
    /**
     Adapts the scripted outcome to the provider interface.
     */
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      return Promise.resolve(scriptedOutcome({
        request,
        reply,
        refused,
      },),);
    },
  };
}

//endregion Refusing-seats client

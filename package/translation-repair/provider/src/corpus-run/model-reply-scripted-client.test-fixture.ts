import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '../../dist/final/node/index.mjs';

//region Model reply scripted client
// A CLIENT ANSWERING EACH MODEL WITH ONE SCRIPTED OUTCOME OR FAULT, chosen by
// model id, recording every request it was asked, for cases that need a roster
// of models without a provider.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 What one scripted model does when asked.

 @example
 ```ts
 const reply: ScriptedModelReply = { kind: 'ok', text: '{"count":2,"first":"Mittens"}', };
 ```
 */
export type ScriptedModelReply =
  | {
    /**
     The model answers with this JSON text, which the request's guard reads.
     */
    readonly kind: 'ok';

    /**
     JSON text the model answers with.
     */
    readonly text: string;
  }
    | {
    /**
     The model answers with text the request's guard or parser refused.
     */
      readonly kind: 'schema-mismatch';

    /**
     Text the model answered with.
     */
      readonly rawText: string;

    /**
     What failed, in the client's own words.
     */
      readonly detail: string;
    }
    | {
    /**
     The model answers with a refusal.
     */
      readonly kind: 'refusal-shaped';

    /**
     Text the model answered with.
     */
      readonly rawText: string;
    }
    | {
    /**
     The call itself fails with this value.
     */
      readonly kind: 'throws';

    /**
     Value the call rejects with.
     */
      readonly error: unknown;
    };

/**
 Refuses a text call, which a JSON probe never makes.

 @throws Error on any invocation
 */
function unexpectedText(): never {
  throw new Error('chatText unused by a JSON probe',);
}

/**
 Refuses a quota read, which a JSON probe never makes.

 @throws Error on any invocation
 */
function unexpectedQuotas(): never {
  throw new Error('quotas unused by a JSON probe',);
}

/**
 Turns one scripted reply into the outcome the client gives, or the fault it
 throws.

 @param reply - what the model was scripted to do

 @param request - the live request, whose guard an `ok` text must satisfy

 @returns The outcome

 @throws The scripted error of a `throws` reply

 @throws Error when an `ok` text is not JSON the request's guard admits, which
 would mean the case scripted a reply the probe cannot read

 @example
 ```ts
 const outcome = outcomeOf({ reply, request, },);
 ```
 */
function outcomeOf<ValueT,>(
  {
    reply,
    request,
  }: {
    readonly reply: ScriptedModelReply;
    readonly request: ChatJsonRequest<ValueT>;
  },
): ChatJsonOutcome<ValueT> {
  if (reply.kind === 'throws')
    throw reply.error;

  if (reply.kind === 'schema-mismatch') {
    return {
      kind: 'schema-mismatch',
      rawText: reply.rawText,
      detail: reply.detail,
    };
  }

  if (reply.kind === 'refusal-shaped') {
    return {
      kind: 'refusal-shaped',
      rawText: reply.rawText,
      marker: 'api-refusal-field',
    };
  }

  /**
   The scripted text, parsed.
   */
  const value: unknown = JSON.parse(reply.text,);
  if (!request.validate(value,))
    throw new Error('scripted reply failed the probe guard',);
  return {
    kind: 'ok',
    value,
    rawText: reply.text,
  };
}

/**
 Builds a client whose models each do one scripted thing.

 @param replies - what each model does, by model id

 @returns Client plus the requests it was asked, in order

 @example
 ```ts
 const { client, requests, } = modelReplyScriptedClient({ replies: new Map([['mittens', reply,],],), },);
 ```
 */
export function modelReplyScriptedClient(
  { replies, }: { readonly replies: ReadonlyMap<string, ScriptedModelReply>; },
): {
  readonly client: ModelClient;
  readonly requests: ChatJsonRequest<unknown>[];
} {
  /**
   Every request the client was asked, in order.
   */
  const requests: ChatJsonRequest<unknown>[] = [];

  return {
    requests,
    client: {
      chatText: unexpectedText,
      chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
        requests.push(request,);

        /**
         What this model was scripted to do.
         */
        const reply = replies.get(request.modelId,);
        if (reply === undefined)
          return Promise.reject(new Error(`no scripted reply for ${request.modelId}`,),);
        try {
          return Promise.resolve(outcomeOf({
            reply,
            request,
          },),);
        } catch (error) {
          return Promise.reject(Error.isError(error,) ? error : new Error(
            'scripted non-error fault',
            { cause: error, },
          ),);
        }
      },
      quotas: unexpectedQuotas,
    },
  };
}

//endregion Model reply scripted client

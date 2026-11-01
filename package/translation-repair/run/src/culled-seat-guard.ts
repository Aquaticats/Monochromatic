import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
  DecisionReply,
  DecisionRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Culled seat guard
// THE OWNER'S CULL AT THE DOOR EVERY CALL PASSES (ledger P4, 2026-09-28).
// "Cull it from every role" (2026-09-24) held on every bench the run derives,
// and nothing stopped a path that named the seat itself: the recall
// benchmark's default judges still listed it, and the router serves it on
// every provider. The run client wraps its routed client in this, so a culled
// model is refused before any provider is asked, with the refusal a round
// already reads as an unreachable seat. Catalog reach stays as it is: the
// card is kept for the catalogs and the unit fixtures.

/**
 Refuses a culled model, or does nothing.

 @param modelId - model a call names

 @param culled - models the owner culled from every role

 @throws {@link NoProviderForModelError} when the model is culled

 @example
 ```ts
 refuseCulled({ modelId: request.modelId, culled, },);
 ```
 */
function refuseCulled(
  {
    modelId,
    culled,
  }: {
    readonly modelId: string;
    readonly culled: ReadonlySet<string>;
  },
): void {
  if (culled.has(modelId,)) {
    throw new NoProviderForModelError({
      modelId,
      reason: 'the owner culled this model from every role (2026-09-24)',
    },);
  }
}

/**
 Wraps a client so a culled model is refused before any provider is asked.

 @param inner - client every other call goes to

 @param culled - models the owner culled from every role

 @returns Client with the same surface

 @example
 ```ts
 const client = refusingCulledSeats({ inner: routed, culled: OWNER_CULLED, },);
 ```
 */
export function refusingCulledSeats(
  {
    inner,
    culled,
  }: {
    readonly inner: ModelClient;
    readonly culled: ReadonlySet<string>;
  },
): ModelClient {
  /**
   Typed exchange the wrapped client offers, bound once so the closure keeps
   the narrowing.
   */
  const innerDecide = inner.decide;
  return {
    async chatText(request: ForeignBorrowed<ChatTextRequest>,): Promise<ChatTextReply> {
      refuseCulled({
        modelId: request.modelId,
        culled,
      },);
      return await inner.chatText(request,);
    },
    async chatJson<ValueT,>(
      request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      refuseCulled({
        modelId: request.modelId,
        culled,
      },);
      return await inner.chatJson(request,);
    },
    quotas: inner.quotas,
    ...((innerDecide === undefined)
      ? {}
      : {
        decide: async function refusingDecide(request: ForeignBorrowed<DecisionRequest>,): Promise<DecisionReply> {
          refuseCulled({
            modelId: request.modelId,
            culled,
          },);
          return await innerDecide(request,);
        },
      }),
  };
}

//endregion Culled seat guard

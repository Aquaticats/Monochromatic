/**
 Tests the culled-seat guard (ledger P4, reach under ledger T8): a model the
 owner culled is refused on every exchange before the wrapped client is asked,
 any other model passes through untouched, and the guard offers the typed
 decision exchange exactly where the wrapped client does. Model ids and
 replies are cat-themed invention; requests carry only the model id the guard
 reads, cast past their full types.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
  DecisionReply,
  DecisionRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { refusingCulledSeats, } from '../dist/final/node/index.mjs';

/**
 The model the owner culled in these cases.
 */
const CULLED = 'hf:cat/Hairball';

/**
 A model left seated.
 */
const SEATED = 'hf:cat/Tabby';

/**
 The text reply the wrapped client gives.
 */
const TEXT_REPLY: ChatTextReply = { text: 'purr', };

/**
 The decision reply the wrapped client gives.
 */
const DECISION_REPLY = { answers: ['windowsill',], } as unknown as DecisionReply;

/**
 A client recording which exchange each call reached, with or without the
 decision exchange.

 @param withDecide - whether it carries the decision exchange

 @returns Client and calls it received, as `exchange model`
 */
function recordingClient({ withDecide, }: { readonly withDecide: boolean; },): {
  readonly client: ModelClient;
  readonly asked: readonly string[];
} {
  /**
   Calls received.
   */
  const asked: string[] = [];
  /**
   The exchanges every client carries.
   */
  const chat: ModelClient = {
    chatText: async function chatText(request: ForeignBorrowed<ChatTextRequest>,): Promise<ChatTextReply> {
      asked.push(`chatText ${request.modelId}`,);
      return TEXT_REPLY;
    },
    chatJson: async function chatJson<ValueT,>(
      request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      asked.push(`chatJson ${request.modelId}`,);
      return {
        kind: 'refusal-shaped',
        rawText: 'hiss',
        marker: 'cannot',
      };
    },
    quotas: async function quotas(): Promise<never> {
      throw new Error('quotas unused by the guard',);
    },
  };
  return {
    client: withDecide
      ? {
        ...chat,
        decide: async function decide(request: ForeignBorrowed<DecisionRequest>,): Promise<DecisionReply> {
          asked.push(`decide ${request.modelId}`,);
          return DECISION_REPLY;
        },
      }
      : chat,
    asked,
  };
}

/**
 A request naming one model and nothing else the guard reads.

 @param modelId - model named

 @returns Request, cast past the fields the guard never reads
 */
function requestFor(modelId: string,): ChatTextRequest & ChatJsonRequest<unknown> & DecisionRequest {
  return { modelId, } as unknown as ChatTextRequest & ChatJsonRequest<unknown> & DecisionRequest;
}

await describe({
  name: refusingCulledSeats.name,
  children: [
    it({
      name: 'REFUSES A CULLED MODEL ON EVERY EXCHANGE before the wrapped client is asked',
      fn: async () => {
        const {
          client,
          asked,
        } = recordingClient({ withDecide: true, },);
        const guarded = refusingCulledSeats({
          inner: client,
          culled: new Set([CULLED,],),
        },);
        const request = requestFor(CULLED,);
        await expect(guarded.chatText(request,),).rejects.toThrow(NoProviderForModelError,);
        await expect(guarded.chatJson(request,),).rejects.toThrow(NoProviderForModelError,);
        await expect(guarded.decide?.(request,),).rejects.toThrow(NoProviderForModelError,);
        expect(asked,).toStrictEqual([],);
      },
    },),
    it({
      name: 'PASSES A SEATED MODEL THROUGH EACH EXCHANGE with the wrapped client\'s reply and quota reader',
      fn: async () => {
        const {
          client,
          asked,
        } = recordingClient({ withDecide: true, },);
        const guarded = refusingCulledSeats({
          inner: client,
          culled: new Set([CULLED,],),
        },);
        const request = requestFor(SEATED,);
        const text = await guarded.chatText(request,);
        const json = await guarded.chatJson(request,);
        const decision = await guarded.decide?.(request,);
        expect(text,).toBe(TEXT_REPLY,);
        expect(json,).toStrictEqual({
          kind: 'refusal-shaped',
          rawText: 'hiss',
          marker: 'cannot',
        },);
        expect(decision,).toBe(DECISION_REPLY,);
        expect(guarded.quotas,).toBe(client.quotas,);
        expect(asked,).toStrictEqual([
          `chatText ${SEATED}`,
          `chatJson ${SEATED}`,
          `decide ${SEATED}`,
        ],);
      },
    },),
    it({
      name: 'OFFERS NO DECISION EXCHANGE where the wrapped client has none, so a stage records a lost voice',
      fn: async () => {
        const { client, } = recordingClient({ withDecide: false, },);
        expect(refusingCulledSeats({
          inner: client,
          culled: new Set([CULLED,],),
        },).decide,).toBeUndefined();
      },
    },),
  ],
},);

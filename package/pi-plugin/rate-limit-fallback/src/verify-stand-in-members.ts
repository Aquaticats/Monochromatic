/**
 Stand-in writer members both verification arms register, and the labels they record.
 @module
 */

import type {
  Api,
  Model,
  Provider,
} from '@earendil-works/pi-ai';
import type { ModelRuntime, } from '@earendil-works/pi-coding-agent';
import { WRITER_TARGETS, } from './member-order-fixture.ts';
import {
  immediate,
  MEMBER_KEY,
  rateLimitRefusal,
  type ModelIdentity,
} from './stand-in-pi-fixture.ts';

/**
 Stand-in origin for both members: the extension refuses a quota read whose model origin is not
 NeuralWatt's, so verification makes no request to a provider endpoint.
 */
export const STAND_IN_ORIGIN = 'http://127.0.0.1:0/v1';

/**
 Window and output limit one arm registers both of its members with.
 */
export type WriterLimits = Readonly<{
  contextWindow: number;
  maxTokens: number
}>;

/**
 Limits the first arm registers, the size of a modern DeepSeek lane.
 */
export const REAL_LIMITS: WriterLimits = {
  contextWindow: 1_048_560,
  maxTokens: 65_536,
};

/**
 Limits the held-model arm registers, small enough to sit under Pi's built-in members of the same
 pool, so a stale provisional window cannot pass for the registry's value.
 */
export const HELD_LIMITS: WriterLimits = {
  contextWindow: 100_000,
  maxTokens: 32_768,
};

/**
 First two writer members, registered as stand-ins before extension loading.
 */
const [refusingMember, answeringMember,] = WRITER_TARGETS;

/**
 Exact member label, so two members of one provider stay distinguishable in the call record.

 @param model - physical member Pi dispatched

 @returns `provider/id`

 @example
 ```ts
 labelOfMember({ provider: 'neuralwatt', id: 'deepseek-v4.1-flash' }); // 'neuralwatt/deepseek-v4.1-flash'
 ```
 */
export function labelOfMember(model: ModelIdentity,): string {
  return `${model.provider}/${model.id}`;
}

/**
 One registered arm's members, in the order the pool asks them.
 */
export type StandingMembers = Readonly<{
  refusing: Model<Api>;
  answering: Model<Api>
}>;

/**
 Catalog records one arm registers.

 @param limits - window and output limit both members report

 @returns refusing head and answering member

 @example
 ```ts
 writerMembers({ limits: REAL_LIMITS });
 ```
 */
export function writerMembers({ limits }: { readonly limits: WriterLimits },): StandingMembers {
  return {
    refusing: {
      ...refusingMember,
      baseUrl: STAND_IN_ORIGIN,
      contextWindow: limits.contextWindow,
      maxTokens: limits.maxTokens,
    },
    answering: {
      ...answeringMember,
      baseUrl: STAND_IN_ORIGIN,
      contextWindow: limits.contextWindow,
      maxTokens: limits.maxTokens,
    },
  };
}

/**
 The pool has no need for the fixture's API-specific request path.

 @throws Error when verification reaches the wrong request path
 */
function streamUnsupported(): never {
  throw new Error('Unexpected API-specific request',);
}

/**
 Register one scripted Pi provider holding both writer members: the first refuses, the next answers.

 @param modelRuntime - real registry the extension will read

 @param calls - record every dispatched member's label into

 @param limits - window and output limit both members report

 @returns the members this arm registered

 @example
 ```ts
 await registerWriterProvider({ modelRuntime, calls, limits: REAL_LIMITS });
 ```
 */
export async function registerWriterProvider({
  modelRuntime,
  calls,
  limits,
}: {
  readonly modelRuntime: ModelRuntime;
  readonly calls: string[];
  readonly limits: WriterLimits;
},): Promise<StandingMembers> {
  /**
   Members this arm registers, which its streams answer for.
   */
  const members = writerMembers({ limits, },);
  /**
   Provider object registered with the host.
   */
  const provider: Provider = {
    id: refusingMember.provider,
    name: refusingMember.provider,
    getModels() {
      return [
        members.refusing,
        members.answering,
      ];
    },
    auth: {
      apiKey: {
        name: 'Stand-in credential',
        resolve({ credential, }: { readonly credential?: { readonly key?: string; }; }) {
          return Promise.resolve(credential?.key === undefined
            ? undefined
            : {
              auth: { apiKey: credential.key, },
              source: 'test',
            },);
        },
      },
    },
    stream: streamUnsupported,
    streamSimple(
      physical: ModelIdentity,
      _context: unknown,
      options,
    ) {
      if (options?.apiKey !== MEMBER_KEY)
        throw new Error('Pi registry did not resolve the stand-in credential',);
      calls.push(labelOfMember(physical,),);
      /**
       Whether Pi dispatched the member that has to refuse before the handoff.
       */
      /**
       Head member this arm registers, which refuses once per handoff.
       */
      const { refusing, } = members;
      /**
       Whether Pi dispatched the refusing head.
       */
      const refuses = physical.id === refusing.id;
      return immediate({
        model: physical,
        ...(refuses ? { errorMessage: rateLimitRefusal({ model: physical, },), } : {}),
      },);
    },
  };
  modelRuntime.registerNativeProvider(provider,);
  await modelRuntime.setRuntimeApiKey(
    provider.id,
    MEMBER_KEY,
  );
  return members;
}

import type { JsonSchemaResponseFormat, } from '../dist/final/node/index.mjs';

//region Stage trivial reply
// TRIVIAL SCRIPTED REPLY SHAPES, THEIR GUARDS AND THE MEOW REPLY'S RESPONSE
// FORMAT, plus waiting for a round's own abort signal, for cat-themed stage
// tests exercising quorum and retry machinery rather than any real reply
// schema.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several stage tests kept their own copy
// of these guards, the format and the abort wait; all now import them from
// here.

/**
 Purr-shaped trivial reply some scripted clients emit.
 */
export type PurrReply = {
  readonly purr: string;
};

/**
 Guards the trivial payload.

 @param value - candidate reply

 @returns Whether value carries a string purr

 @example
 ```ts
 isPurrReply({ purr: 'loud', },);
 ```
 */
export function isPurrReply(value: unknown,): value is PurrReply {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('purr' in value)
    && ((typeof value.purr) === 'string');
}

/**
 Response format naming the test stage, for asks scripted to answer a
 `PurrReply`.
 */
export const PURR_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'purr_reply',
    schema: { type: 'object', },
  },
};

/**
 Meow-shaped trivial reply other scripted clients emit.
 */
export type MeowReply = {
  readonly meow: string;
};

/**
 Guards the trivial payload.

 @param value - candidate reply

 @returns Whether value carries a string meow

 @example
 ```ts
 isMeowReply({ meow: 'loud', },);
 ```
 */
export function isMeowReply(value: unknown,): value is MeowReply {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('meow' in value)
    && ((typeof value.meow) === 'string');
}

/**
 Response format naming the test stage, for asks scripted to answer a
 `MeowReply`.
 */
export const MEOW_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'meow_reply',
    schema: { type: 'object', },
  },
};

/**
 Resolves when a signal aborts, and never otherwise.

 Deliberately has NO timer of its own. A stub that also gave up after some
 duration would pass the abandonment case whether or not the cut ever reached
 the call, which is the one thing that case exists to prove.

 @param signal - call signal the round owns

 @example
 ```ts
 await untilAborted({ signal, },);
 ```
 */
export async function untilAborted({ signal, }: { readonly signal: AbortSignal; },): Promise<void> {
  if (signal.aborted)
    return;

  /**
   Capability resolved by the abort listener.
   */
  const {
    promise,
    resolve,
  } = Promise.withResolvers<undefined>();
  signal.addEventListener(
    'abort',
    function onAbort(): void {
      resolve(undefined,);
    },
    { once: true, },
  );
  await promise;
}

//endregion Stage trivial reply

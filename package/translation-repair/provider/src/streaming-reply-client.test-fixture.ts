import {
  createSyntheticClient,
  type ModelClient,
} from '../dist/final/node/index.mjs';

//region Streaming reply client
// A CLIENT WHOSE MODELS REPLY IN TURN WITH GIVEN BODIES, over a canned
// streaming transport delivering one delta frame and the terminator.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The consolidate-gate, pair-blocks,
// pair-sections and prepare-with-pairing tests kept their own copy of this
// client; all now import it from here.

/**
 Builds a client whose models reply in turn with the given bodies.

 @param replyByModel - reply body per call, in the order calls are made

 @returns Client over a canned transport

 @example
 ```ts
 const client = cannedClient({ replyByModel: ['{"pairs":[]}',], },);
 ```
 */
export function cannedClient(
  { replyByModel, }: { readonly replyByModel: readonly string[]; },
): ModelClient {
  /**
   Calls served so far, so each model gets its own reply.
   */
  const served: string[] = [];
  return createSyntheticClient({
    apiKey: 'test-key',
    transport: function cannedTransport(exchange,) {
      /**
       Which reply this call receives.
       */
      const at = served.length;
      served.push(exchange.label,);

      /**
       This model's reply text: its own, or the first where the script ran
       short.
       */
      const content = replyByModel[at]
        ?? replyByModel[0]
        ?? '';

      // THE CLIENT READS A STREAM, not a completion body: one delta frame and
      // the terminator, which is the smallest well-formed reply.
      return Promise.resolve({
        status: 200,
        bodyText: `data: ${
          JSON.stringify({
            choices: [
              {
                index: 0,
                delta: { content, },
              },
            ],
          },)
        }\n\ndata: [DONE]\n\n`,
      },);
    },
  },);
}

//endregion Streaming reply client

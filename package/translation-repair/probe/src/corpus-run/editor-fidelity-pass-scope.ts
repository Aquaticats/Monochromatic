import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';

//region Editor fidelity pass scope
// A REPEAT PASS MUST ASK THE MODEL AGAIN. The client the probe asks through has the prompt reuse layer of every run client
// (`promptUniqueClient`), which answers a prompt it has seen from its memory, or from its payload directory, without a call.
// A second pass over the pool asks the same prompts as the first, so on the billed high arm of `dd4b15b75c` every ask of
// the later passes was a replay of the first pass's reply, in no time and at no cost, and the repeat measurement compared
// replies with themselves.
//
// THE SEAM IS THE PASS IN THE IDENTITY OF THE REQUEST. Each row asks through a client that puts its pass on every request
// as a reuse scope, which the reuse layer keeps in the identity it claims the prompt under: the same prompt in another pass
// is another exchange, bought from the model, and the layer still does everything else it does. Dropping the layer for the
// probe was refused: it would need an option through three builders to ask through another client than the one a run is
// asked through, and the payload directory the layer keeps would hold no reply of the arm for audit. The layer stays as it
// is for corpus runs, which name no scope.

/**
 Names a pass in the words of the row line, counted from one.

 @param pass - pass over the pool, from zero, as the walk counts it

 @returns Scope the reuse layer keeps in its identity

 @example
 ```ts
 // yields 'pass 2'
 const scope = passScopeOf({ pass: 1, },);
 ```
 */
export function passScopeOf({ pass, }: { readonly pass: number; },): string {
  return `pass ${String(pass + 1,)}`;
}

/**
 Wraps a client so every exchange names its pass as the reuse scope.

 @param client - client to wrap

 @param pass - pass over the pool that the exchanges belong to, from zero

 @returns Client with every member of the wrapped one, its text and JSON exchanges carrying the pass

 @example
 ```ts
 const asking = scopedToPass({ client, pass: 1, },);
 ```
 */
export function scopedToPass(
  {
    client,
    pass,
  }: {
    readonly client: ModelClient;
    readonly pass: number;
  },
): ModelClient {
  /**
   The scope every exchange of the pass carries.
   */
  const reuseScope = passScopeOf({ pass, },);
  return {
    ...client,
    chatText: function scopedText(request,) {
      return client.chatText({
        ...request,
        reuseScope,
      },);
    },
    chatJson: function scopedJson(request,) {
      return client.chatJson({
        ...request,
        reuseScope,
      },);
    },
  };
}

//endregion Editor fidelity pass scope

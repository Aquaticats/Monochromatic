import type {
  ChatJsonOutcome,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';

//region Editor fidelity asks
// HOW EACH ASK OF A TRIAL ENDED, noted by a wrapper around the client without changing what any exchange returns or
// throws. A trial asks the card through the stage's gather, which retries and recovers, so one trial may ask more than
// once, and the row of the trial states every ask's end. A reading of `unusable` says the trial yielded nothing; the end
// says why: the card stopped at the output-token cap, replied text no parser read, a refusal, a guard's rejection, or the
// exchange threw. The read step scores a trial with an ask that ended at the cap as unusable for its arm, and leaves a
// trial with an ask that threw out of every arm's comparison and counts it.

/**
 How one ask ended: the reply was read, the card refused, the completion or its thinking was cut at the output-token
 cap, the text did not parse, the caller's guard rejected it, or the exchange threw.
 */
export type AskEnd = 'ok' | 'refused' | 'output-cap' | 'unparseable' | 'rejected' | 'threw';

/**
 Reads the end an outcome of a JSON exchange is.

 @param outcome - what the exchange gave

 @returns End: a cut-off completion or thinking is the output cap, whatever else the mismatch says

 @example
 ```ts
 // yields 'output-cap'
 const end = askEndOf({ outcome: { kind: 'schema-mismatch', rawText: '{', reason: 'truncated-completion', detail: 'cut', }, },);
 ```
 */
export function askEndOf({ outcome, }: { readonly outcome: ChatJsonOutcome<unknown>; },): AskEnd {
  if (outcome.kind === 'ok')
    return 'ok';
  if (outcome.kind === 'refusal-shaped')
    return 'refused';
  if ((outcome.reason === 'truncated-completion') || (outcome.reason === 'truncated-thinking'))
    return 'output-cap';
  return (outcome.reason === 'unparseable-json') ? 'unparseable' : 'rejected';
}

/**
 Says the ends of a trial's asks in the word its row line prints.

 @param ends - ends, in the order asked

 @returns Ends joined by a plus, or `none` for a trial that asked nothing

 @example
 ```ts
 // yields 'refused+ok'
 const word = stoppedWord({ ends: ['refused', 'ok',], },);
 ```
 */
export function stoppedWord({ ends, }: { readonly ends: readonly AskEnd[]; },): string {
  return (ends.length === 0) ? 'none' : ends.join('+',);
}

/**
 A client that notes how each exchange ended, and the way to take the notes.

 @example
 ```ts
 const { client, takeEnds, } = recordingAsks({ client: inner, },);
 ```
 */
export type RecordedAsks = {
  /**
   The wrapped client, with every member of the one wrapped.
   */
  readonly client: ModelClient;

  /**
   Takes the ends noted since the last take, in the order asked, and empties the notes.
   */
  readonly takeEnds: () => readonly AskEnd[];
};

/**
 Wraps a client so the end of every exchange is noted.

 @param client - client to wrap

 @returns Wrapped client and reader of its notes

 @throws Whatever the wrapped client throws, after noting it as threw

 @example
 ```ts
 const { client, takeEnds, } = recordingAsks({ client: inner, },);
 ```
 */
export function recordingAsks({ client, }: { readonly client: ModelClient; },): RecordedAsks {
  /**
   The ends noted since the last take.
   */
  const noted: AskEnd[] = [];

  return {
    client: {
      ...client,
      chatText: async function notedText(request,) {
        try {
          /**
           What the exchange answered.
           */
          const reply = await client.chatText(request,);
          noted.push('ok',);
          return reply;
        }
        catch (error) {
          noted.push('threw',);
          throw error;
        }
      },
      chatJson: async function notedJson(request,) {
        try {
          /**
           What the exchange gave.
           */
          const outcome = await client.chatJson(request,);
          noted.push(askEndOf({ outcome, },),);
          return outcome;
        }
        catch (error) {
          noted.push('threw',);
          throw error;
        }
      },
    },
    takeEnds: function takeEnds(): readonly AskEnd[] {
      return noted.splice(
        0,
      );
    },
  };
}

//endregion Editor fidelity asks

import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import {
  type ChatJsonOutcome,
  isReasoningRunaway,
} from '@monochromatic-dev/translation-repair-provider/ts';

//region Recovery nudge
// THE RECOVERY ROUND RE-ASKS AN ANSWER NOTHING COULD READ WITH A COMPLAINT
// NAMING WHAT HAPPENED (`stage-quorum.ts`), and two different things happen
// (ledger P10, 2026-09-28). A reply the length limit stopped ran out of room,
// almost always while still reasoning: of the replies cut at their cap since
// the caps went on the wire, 74 of 75 deepseek-v4.1-flash replies on Hyper and
// 185 of 197 Qwen3.8-27B replies on Synthetic streamed no content at all. The
// round told every such model its reply "did not match the required response
// shape", which names a fault it did not make and says nothing of the one it
// did. Every other unreadable answer arrived whole and missed the shape.
//
// A REPLY THAT STREAMED NO CONTENT AT ALL IS A RUNAWAY, since ledger B1074 (the
// owner's answer of 2026-10-30, "Everywhere (Recommended)"): the model spent
// its whole completion cap thinking, and asking the same weights again repeats
// it. The round asks such a seat's question of another model with the plain
// question (`stage-recovery-round.ts`) and nudges the seat's own model only
// where no other model can take it, under the wording of `cut-short`.
//
// EACH WORDING IS A PROMPT OF ITS OWN. `promptUniqueClient` answers a model
// and prompt it has seen from its claims, so both recovery wordings and the
// cross-provider re-ask's (`nudged-reask.ts`) must differ, or one would be
// answered with the reply that already failed.
//
// PROSE, NOT A PARAMETER. Asking a model cut short to keep its working brief
// is text in the prompt, which works on a seat of any tier. It is not a
// reasoning control, so it needs none of the clearance the owner's rulings of
// 2026-10-22 give first and second rate providers and withhold from third.

/**
 Every reason nothing could read an answer that arrived, in the order the
 recovery round asks them.

 `runaway` is a reply the length limit stopped with no content at all (a
 reasoning runaway). `cut-short` is every other reply the length limit stopped
 before it was complete (`truncated-completion` that holds content, or
 `truncated-thinking`). `off-shape` is every other unreadable answer: one that
 did not parse, one the guard refused, or one that read as a refusal.

 @example
 ```ts
 const groups = UNREADABLE_CAUSES.map(function seatsWith(cause,) { return cause; },);
 ```
 */
export const UNREADABLE_CAUSES = [
  'runaway',
  'cut-short',
  'off-shape',
] as const;

/**
 Why nothing could read an answer that arrived; see `UNREADABLE_CAUSES`.

 @example
 ```ts
 const cause: UnreadableCause = 'cut-short';
 ```
 */
export type UnreadableCause = typeof UNREADABLE_CAUSES[number];

/**
 What the recovery round adds to the prompt of a model whose answer arrived
 whole but off the shape asked for.

 THE COMPLAINT IS THE ROUND'S WHOLE VALUE. `promptUniqueClient` answers a
 second call for the same model and prompt from its cache, schema mismatch
 included, so a recovery round that re-sent the same bytes was answered with
 the same unreadable bytes in 0 to 1 ms on every one of the five occasions
 measured across two passes on 2026-09-02. The guard here is a type
 predicate and carries no message of its own, so the complaint names the
 failure in general terms: the answer arrived and its shape was not the one
 asked for. That is enough to make the digest new and to tell the model what
 to do differently. Worded as it was before ledger P10 split it by cause, so
 a stored payload for such a re-ask still replays.
 */
export const OFF_SHAPE_RECOVERY_NUDGE: ChatMessage = {
  role: 'user',
  content: 'Your previous reply arrived but could not be read: it did not match the required '
    + 'response shape. Answer the same question again, replying with ONLY the JSON object of '
    + 'the shape described, nothing before or after it.',
};

/**
 What the recovery round adds to the prompt of a model whose reply the length
 limit cut before it was complete: that it ran out of room, and to answer
 within it.
 */
export const CUT_SHORT_RECOVERY_NUDGE: ChatMessage = {
  role: 'user',
  content: 'Your previous reply was cut off at the length limit before it was complete, so it could not be '
    + 'read. Answer the same question again within that limit, keeping any working brief, and reply with '
    + 'ONLY the JSON object of the shape described, nothing before or after it.',
};

/**
 Recovery wording for each cause, so the round asks each unreadable seat with
 the complaint that names what happened to it.

 A runaway keeps the cut-short wording because it is the ask of the seat's own
 model, made only where no other model can take the re-ask, and that ask was
 worded so before a runaway was told apart: a payload stored for it replays.

 @example
 ```ts
 const nudge = RECOVERY_NUDGES['cut-short'];
 ```
 */
export const RECOVERY_NUDGES: Readonly<Record<UnreadableCause, ChatMessage>> = {
  runaway: CUT_SHORT_RECOVERY_NUDGE,
  'cut-short': CUT_SHORT_RECOVERY_NUDGE,
  'off-shape': OFF_SHAPE_RECOVERY_NUDGE,
};

/**
 Names why nothing could read an answer that arrived.

 @param outcome - non-ok exchange outcome

 @returns `runaway` for a reply the length limit stopped with no content,
 `cut-short` for any other reply the limit stopped, `off-shape` for any other

 @example
 ```ts
 const cause = unreadableCauseOf({ outcome, },);
 ```
 */
export function unreadableCauseOf(
  { outcome, }: {
    readonly outcome: Exclude<ChatJsonOutcome<unknown>, { readonly kind: 'ok'; }>;
  },
): UnreadableCause {
  if (isReasoningRunaway({ outcome, },))
    return 'runaway';
  if ((outcome.kind === 'schema-mismatch')
    && ((outcome.reason === 'truncated-completion') || (outcome.reason === 'truncated-thinking')))
    return 'cut-short';
  return 'off-shape';
}

//endregion Recovery nudge

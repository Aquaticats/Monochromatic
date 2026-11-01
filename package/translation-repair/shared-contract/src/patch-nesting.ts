import {
  firstNestingExcess,
  nestingAccount,
  type NestingExcess,
  type NestingReading,
  spliceDisjointEdits,
  type SpliceEdit,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Patch nesting
// A PATCH IS REFUSED FOR WHAT THE TEXT WOULD BE, not for what the editor wrote: the
// text the patch leaves is read against the nesting bound as the parser will
// read it, so the refusal of an unreadable edit belongs to the voice that wrote
// it and no later parse of the chunk can raise it again.

/**
 Reads the text an edit leaves, written beside the edits already accepted,
 against the nesting bound.

 THE WHOLE TEXT IS READ, not the replacement alone, since a replacement that
 reads on its own can still nest past the bound inside the markers around it.
 Plain markdown is the grammar asked: the document parse downgrades to it, so
 a text it refuses is a text nothing reads.

 @param targetText - text the edits are written into

 @param accepted - edits already accepted, disjoint from the candidate

 @param candidate - the edit being judged

 @returns Where the text the edits leave first passes the bound, or that it
 stays within

 @example
 ```ts
 const reading = nestingWithEdit({ targetText: 'A cat.', accepted: [], candidate: { start: 0, end: 6, text: 'A kitten.', }, },);
 // => { kind: 'within', }
 ```
 */
export function nestingWithEdit(
  {
    targetText,
    accepted,
    candidate,
  }: {
    readonly targetText: string;
    readonly accepted: readonly SpliceEdit[];
    readonly candidate: SpliceEdit;
  },
): NestingReading {
  return firstNestingExcess({
    body: spliceDisjointEdits({
      text: targetText,
      edits: [
        ...accepted,
        candidate,
      ],
    },),
    grammar: 'markdown',
  },);
}

/**
 What a rejection's reason begins with when the text the edit leaves is read
 by no grammar.
 */
const UNREADABLE_KIND = 'unreadable-replacement';

/**
 The reason a patch gives for refusing an edit whose text nothing reads.

 @param excess - where the text the edit leaves first passes the bound

 @returns Reason in the wording a scorecard counts, the account in its
 parenthetical detail, quoting nothing of the text

 @example
 ```ts
 const reason = unreadableReason({ excess, },);
 // => 'unreadable-replacement (nested too deeply to read: its container markers pass the bound of 256 at line 3, column 513)'
 ```
 */
export function unreadableReason({ excess, }: { readonly excess: NestingExcess; },): string {
  return `${UNREADABLE_KIND} (${nestingAccount({ excess, },)})`;
}

/**
 Whether a rejection's reason is the refusal of an edit whose text nothing
 reads.

 @param reason - reason a patch gave

 @returns True for a reason {@link unreadableReason} made

 @example
 ```ts
 isUnreadableReason({ reason: 'unchanged-region', },);
 // => false
 ```
 */
export function isUnreadableReason({ reason, }: { readonly reason: string; },): boolean {
  return reason.startsWith(`${UNREADABLE_KIND} (`,);
}

//endregion Patch nesting

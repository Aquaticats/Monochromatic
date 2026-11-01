import type { EditableEnvelope, } from '../dist/final/node/index.mjs';

//region Envelope span
// WHAT A CASE COMPARES OF AN ENVELOPE. An envelope carries two fields derived
// by hashing (`envelopeId`, `baseHash`), which a case would have to compute
// with the code under test to spell out; the four others say where the
// envelope lies, what text it holds and which issues it serves.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Two test files each kept their own copy of
// this projection, which the duplicate-bodies scan found; both import it from
// here.

/**
 Reads the fields of each envelope a case spells out.

 TAKES THE LIST, NOT ONE ENVELOPE, so a case never hands a function reference
 to `map`, which the linter refuses for a function whose arity it cannot see
 across the fixture's import.

 @param envelopes - envelopes the code under test derived

 @returns Offsets, base text and issue ids of each, without the hashed fields, in the order given

 @example
 ```ts
 expect(envelopeSpansOf(envelopes,),).toEqual([{ startOffset: 4, endOffset: 12, baseText: 'cat naps', issueIds: ['adjudicated/nap',], },],);
 ```
 */
export function envelopeSpansOf(
  envelopes: readonly EditableEnvelope[],
): readonly Pick<EditableEnvelope, 'baseText' | 'endOffset' | 'issueIds' | 'startOffset'>[] {
  return envelopes.map(function spanOf(envelope,): Pick<EditableEnvelope, 'baseText' | 'endOffset' | 'issueIds' | 'startOffset'> {
    return {
      startOffset: envelope.startOffset,
      endOffset: envelope.endOffset,
      baseText: envelope.baseText,
      issueIds: envelope.issueIds,
    };
  },);
}

//endregion Envelope span

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
} from '../dist/final/node/index.mjs';

//region Scripted reply outcome
// VALIDATES A SCRIPTED REPLY AGAINST THE LIVE REQUEST'S WIRE GUARD AND WRAPS
// IT AS AN OUTCOME, so a scripted client answers through the same guard a
// live one would.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The corpus-run/editor-width-arm and
// repair-editor-stage tests kept their own copy of this wrap; both now
// import it from here.

/**
 Validates a scripted reply against the live request's wire guard and wraps
 it as an outcome.

 @param report - scripted reply for this call

 @param request - live request, whose guard the reply must satisfy

 @returns Outcome carrying the validated reply

 @throws {@link Error} when the fixture itself fails the guard it is meant
 to satisfy

 @example
 ```ts
 return replyWith({ report: { edits: [], }, request, },);
 ```
 */
export function replyWith<ValueT,>(
  {
    report,
    request,
  }: {
    readonly report: unknown;
    readonly request: ChatJsonRequest<ValueT>;
  },
): ChatJsonOutcome<ValueT> {
  if (!request.validate(report,))
    throw new Error('scripted reply failed the wire guard',);
  return {
    kind: 'ok',
    value: report,
    rawText: JSON.stringify(report,),
  };
}

//endregion Scripted reply outcome

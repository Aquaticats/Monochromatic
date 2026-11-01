//region Straggler cut
// WHY A CALL WAS ABORTED, when the round closed its grace window on it. A call's signal aborts for three reasons that
// want three different counts: the caller left (a stage deadline, an entry stop, an operator's stop), the call's own
// deadline or stream guard fired, or the round's grace window closed on a call that was still in flight after quorum
// stood. Only the last says the seat had its chance and did not take it, so the round aborts with this reason and the
// seat tally reads it off the signal (`seat-tally.ts`); a plain `abort()` carried a bare AbortError that looked the same
// as every other.

/**
 The reason the round aborts a straggler's signal with when its grace window closes.

 @example
 ```ts
 abandon.abort(new StragglerCutError(),);
 ```
 */
export class StragglerCutError extends Error {
  /**
   Declares this message safe to forward: it names no model and carries no text a model wrote.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the reason.
   */
  constructor() {
    super('the round closed its grace window on this call after quorum stood',);
    this.name = 'StragglerCutError';
  }
}

//endregion Straggler cut

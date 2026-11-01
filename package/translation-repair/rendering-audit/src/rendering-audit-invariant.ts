//region Rendering audit invariant
// A state the rendering audit's own construction rules out, met anyway.
//
// ONE CLASS FOR EVERY SITE. Six places in the audit throw on a state their
// callers cannot produce: a canonicalization that changed a length, a category
// outside every anchoring rule, a group with no members, a delivered slice no
// comparison row names. Each was a bare `Error`, which the package's rule on
// refusals (PP4) asks to be a named class, so a reader of a stack or a log can
// tell an invariant from an operator refusal or a provider fault by name alone.
//
// UNMARKED ON PURPOSE. The class forwards each site's own sentence, and the
// inventory in `message-names-only.unit.test.ts` reserves the marker for
// classes that write theirs. The sentences name slice indexes, vocabulary words
// and the shape of a group, never text, so nothing is lost when
// `reportingRefusals` prints `refused by RenderingAuditInvariantError` and the
// frames, which is the right report for a fault in the command.

/**
 An invariant of the rendering audit that did not hold.

 Unreachable by construction; reaching it means the code above the site
 changed and the site's assumption did not.

 @example
 ```ts
 throw new RenderingAuditInvariantError({
   invariant: 'a defect group with no members cannot occur, since groups are built from claims',
 },);
 ```
 */
export class RenderingAuditInvariantError extends Error {
  /**
   @param invariant - what was supposed to hold, in the site's own words,
   naming indexes and vocabulary words and never text
   */
  constructor(
    { invariant, }: { readonly invariant: string; },
  ) {
    super(`rendering audit invariant broken: ${invariant}`,);
    this.name = 'RenderingAuditInvariantError';
  }
}

//endregion Rendering audit invariant

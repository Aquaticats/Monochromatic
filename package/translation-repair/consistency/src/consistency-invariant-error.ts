//region Consistency invariant
// A state the page-level consistency pass's own construction rules out, met
// anyway.
//
// MARKED, SO A CONTAINED FAILURE RECORDS ITS SENTENCE. The pass contains this
// error (a run always ships) and writes what it caught into a group's verdict
// and into the section's findings through `exchangeFailureText`, which repeats
// the message of a marked class and only the class name of any other. Every
// throw site writes its sentence from positions, counts and vocabulary words
// (a slice index, `OCCURRENCE 2`, `group 1`) and never from page text, which is
// the claim the marker makes and `message-names-only.unit.test.ts` reads.

/**
 An invariant of the consistency pass that did not hold.

 Unreachable by construction; reaching it means the code above the site
 changed and the site's assumption did not.

 @example
 ```ts
 throw new ConsistencyInvariantError({
   invariant: 'a repeat group lists no place, and the detector reports a repeat only where it stands in two',
 },);
 ```
 */
export class ConsistencyInvariantError extends Error {
  /**
   Message names positions, counts and vocabulary words only.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param invariant - what was supposed to hold, in the site's own words,
   naming positions, counts and vocabulary words and never page text
   */
  constructor(
    { invariant, }: { readonly invariant: string; },
  ) {
    super(`unreachable: ${invariant}`,);
    this.name = 'ConsistencyInvariantError';
  }
}

//endregion Consistency invariant

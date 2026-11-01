import type { MissingWording, } from './published-page-check.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Published page disagreement
// The refusal a page earns when it does not carry what its artifact says would
// ship, composed here from counts and indices so the class can vouch for every
// character of its own message.
//
// A CLASS THAT FORWARDS A SENTENCE CANNOT CARRY THE MARKER: the inventory in
// `message-names-only.unit.test.ts` refuses one, because such a class cannot
// know what its caller wrote. Before this file the class took a finished
// `message`, documented that every caller wrote only slice indices and
// character counts, and went unmarked anyway, so the boundary muted it. Now the
// callers hand over the disagreement itself and the sentence is written here.

/**
 How a page disagreed with its artifact.

 @example
 ```ts
 const disagreement: PageDisagreement = { kind: 'weight-off', actual: 10, expected: 12, exact: true, };
 ```
 */
export type PageDisagreement = {
  /**
   Page does not carry every wording the artifact says would ship, in slice
   order.
   */
  readonly kind: 'wordings-missing';

  /**
   Wordings the page did not carry, in slice order.
   */
  readonly missing: readonly MissingWording[];
} | {
  /**
   Page weighs something other than the archive plus every slice change.
   */
  readonly kind: 'weight-off';

  /**
   Characters the page has.
   */
  readonly actual: number;

  /**
   Characters the artifact accounts for.
   */
  readonly expected: number;

  /**
   Whether the expectation is an equality, or a floor a filled anchor made it.
   */
  readonly exact: boolean;
};

/**
 Sentence for one disagreement, carrying counts and indices only.

 @param disagreement - what the check found

 @returns Sentence naming it without quoting any text

 @throws {@link UnhandledMemberInvariantError} when a member of `PageDisagreement` has no branch here, which the compiler rules out

 @example
 ```ts
 const said = disagreementSentence({ disagreement, },);
 ```
 */
function disagreementSentence(
  { disagreement, }: { readonly disagreement: PageDisagreement; },
): string {
  if (disagreement.kind === 'wordings-missing') {
    /**
     Wordings the page did not carry.
     */
    const { missing, } = disagreement;

    return `${String(missing.length,)} wording(s) the artifact says would ship are not in `
      + `the page in slice order, at slices ${
        missing
          .map(function named(gone,): string {
            return `${String(gone.sliceIndex,)} (${String(gone.characters,)} ${
              wordForCount({
                count: gone.characters,
                one: 'character',
                many: 'characters',
              },)
            })`;
          },)
          .join(', ',)
      }`;
  }

  if (disagreement.kind === 'weight-off') {
    /**
     Note that a filled anchor makes the expectation a floor, said only where it
     applies so an ordinary refusal does not carry an irrelevant caveat.
     */
    const caveat = disagreement.exact
      ? ''
      : ', which a filled anchor makes a floor rather than an equality';

    return `page is ${String(disagreement.actual - disagreement.expected,)} ${
      wordForCount({
        count: disagreement.actual - disagreement.expected,
        one: 'character',
        many: 'characters',
      },)
    } off the `
      + `${String(disagreement.expected,)} the archive plus every slice change comes to${caveat}`
      + '. Text no slice decided on was lost or added';
  }
  throw new UnhandledMemberInvariantError({
    what: 'published page disagreement',
    member: disagreement,
  },);
}

/**
 Raised when a page does not carry what its artifact says would ship.

 @example
 ```ts
 throw new PublishedPageDisagreesError({
   entryId: 'lintong',
   disagreement: { kind: 'wordings-missing', missing, },
 },);
 ```
 */
export class PublishedPageDisagreesError extends Error {
  /**
   Declares this message safe to print whole at a boundary: it is written
   here from the entry id, slice indices and character counts, and quotes
   nothing.
   */
  readonly messageNamesOnly: true = true;

  /**
   What the check found, for a caller that reads the disagreement rather than
   the sentence.
   */
  readonly disagreement: PageDisagreement;

  /**
   @param entryId - person entry, named in the refusal

   @param disagreement - what the check found
   */
  constructor(
    {
      entryId,
      disagreement,
    }: {
      readonly entryId: string;
      readonly disagreement: PageDisagreement;
    },
  ) {
    super(`${entryId}: ${disagreementSentence({ disagreement, },)}`,);
    this.name = 'PublishedPageDisagreesError';
    this.disagreement = disagreement;
  }
}

//endregion Published page disagreement

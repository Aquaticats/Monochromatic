import type { IssueCategory, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Seeded errors
// Ground truth for the scorecard: deterministic edits planted into a translation,
// each with a known region, so critic findings can be graded mechanically.
// Seeds for real corpus entries are derived at runtime from the text itself
// (the corpus is UNLICENSED; no quoted content may live in this repository).

/**
 Edit family of one planted error.

 @example
 ```ts
 const kind: SeededErrorKind = 'deletion';
 ```
 */
export type SeededErrorKind = 'deletion' | 'replacement' | 'insertion';

/**
 One planted error: a deterministic edit over the target text.

 @example
 ```ts
 const spec: SeededErrorSpec = {
   id: 'seed/omission-0',
   category: 'accuracy/omission',
   kind: 'deletion',
   needle: 'The cat also chases butterflies.',
   replacement: '',
 };
 ```
 */
export type SeededErrorSpec = {
  /**
   Stable handle hits are reported under.
   */
  readonly id: string;

  /**
   Category a perfect critic would assign; matching is span-based,
   category agreement is tracked separately.
   */
  readonly category: IssueCategory;

  /**
   Edit family: remove needle, replace needle, or insert after needle.
   */
  readonly kind: SeededErrorKind;

  /**
   Exact substring the edit targets;
   must occur exactly once at application time.
   */
  readonly needle: string;

  /**
   Replacement text for `replacement`,
   inserted text for `insertion`,
   ignored (empty) for `deletion`.
   */
  readonly replacement: string;
};

/**
 One applied seed with its region in seeded-text coordinates.
 Deletion regions are zero-width at the deletion point.

 @example
 ```ts
 const application: SeededErrorApplication = {
   spec,
   startOffset: 120,
   endOffset: 120,
 };
 ```
 */
export type SeededErrorApplication = {
  /**
   Spec that produced this application.
   */
  readonly spec: SeededErrorSpec;

  /**
   Region start in the final seeded text.
   */
  readonly startOffset: number;

  /**
   Region end (exclusive) in the final seeded text.
   */
  readonly endOffset: number;
};

/**
 Seeded text plus every applied region.

 @example
 ```ts
 const { seededText, applications, } = applySeededErrors({ text, specs, },);
 ```
 */
export type SeededDocumentResult = {
  /**
   Target text after every edit.
   */
  readonly seededText: string;

  /**
   Applications in spec order with final-coordinate regions.
   */
  readonly applications: readonly SeededErrorApplication[];
};

/**
 Signals a seed whose needle is absent or ambiguous at application time;
 always harness misconfiguration, never model fault.

 @example
 ```ts
 throw new SeedApplicationError({ seedId: 'seed/omission-0', reason: 'needle absent', },);
 ```
 */
export class SeedApplicationError extends Error {
  /**
   Declares this message safe to forward: it names the seed and which application failure it hit, never the needle.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the offending seed.

   @param seedId - seed that failed to apply

   @param reason - what the needle check found

   @example
   ```ts
   new SeedApplicationError({ seedId: 'seed/x', reason: 'needle occurs 2 times', },);
   ```
   */
  public constructor(
    {
      seedId,
      reason,
    }: {
      readonly seedId: string;
      readonly reason: string;
    },
  ) {
    super(`seed ${seedId} cannot apply: ${reason}.`,);
    this.name = 'SeedApplicationError';
  }
}

/**
 Applies seeds in order, tracking every region in final coordinates.
 Each needle must occur exactly once in the text as it stands when the seed
 applies; later regions shift earlier ones' coordinates, so regions are
 re-based after every edit.

 @param text - clean target text to plant errors into

 @param specs - seeds in application order

 @returns Seeded text and application regions

 @throws {@link SeedApplicationError} when any needle is absent or ambiguous

 @example
 ```ts
 const { seededText, applications, } = applySeededErrors({ text, specs, },);
 ```
 */
export function applySeededErrors(
  {
    text,
    specs,
  }: {
    readonly text: string;
    readonly specs: readonly SeededErrorSpec[];
  },
): SeededDocumentResult {
  /**
   Unseeded starting state the reduction folds edits onto.
   */
  const initial: SeededDocumentResult = {
    seededText: text,
    applications: [],
  };

  return specs.reduce(
    function applyOne(
      state: SeededDocumentResult,
      spec,
    ): SeededDocumentResult {
      /**
       Needle length reused across the offset arithmetic.
       */
      const needleLength = spec
        .needle
        .length;

      /**
       Needle position in the current text.
       */
      const at = state
        .seededText
        .indexOf(spec.needle,);
      if (at === (-1)) {
        throw new SeedApplicationError({
          seedId: spec.id,
          reason: 'needle absent from current text',
        },);
      }
      if (state
        .seededText
        .includes(
        spec.needle,
        at + 1,
        ))
        {
        throw new SeedApplicationError({
          seedId: spec.id,
          reason: 'needle occurs more than once',
        },);
      }

      /**
       Replacement written over the needle:
       empty for deletion, new text for replacement,
       needle plus addition for insertion.
       */
      const written = spec.kind === 'deletion'
        ? ''
        : (spec.kind === 'replacement'
          ? spec.replacement
          : `${spec.needle}${spec.replacement}`);

      /**
       Region of this edit in the text produced by this step:
       zero-width at the cut for deletions,
       written extent for replacements,
       inserted extent for insertions.
       */
      const region = spec.kind === 'insertion'
        ? {
          startOffset: at + needleLength,
          endOffset: at + written.length,
        }
        : {
          startOffset: at,
          endOffset: at + written.length,
        };

      /**
       Coordinate delta this edit imposes on later positions.
       */
      const shift = written.length - needleLength;

      /**
       Text before the edit point.
       */
      const before = state
        .seededText
        .slice(
        0,
        at,
        );

      /**
       Text after the replaced needle.
       */
      const after = state
        .seededText
        .slice(at + needleLength,);

      return {
        seededText: `${before}${written}${after}`,
        applications: [
          // Earlier regions after the edit point shift by this edit's delta.
          ...state
            .applications
            .map(function rebase(application,) {
              if (application.startOffset < at)
                return application;
              return {
                ...application,
                startOffset: application.startOffset + shift,
                endOffset: application.endOffset + shift,
              };
            },),
          {
            spec,
            ...region,
          },
        ],
      };
    },
    initial,
  );
}

/**
 Tolerance in characters when matching claim spans to seed regions;
 zero-width deletion regions need a neighborhood, and honest critics anchor
 on surrounding context.
 */
export const SEED_MATCH_TOLERANCE = 30;

/**
 Whether one claimed target region hits one seeded region,
 within tolerance.

 @param spanStart - claimed region start in seeded coordinates

 @param spanEnd - claimed region end (exclusive)

 @param application - seeded region under test

 @returns Whether the regions overlap within tolerance

 @example
 ```ts
 seedHitByRegion({ spanStart: 100, spanEnd: 140, application, },);
 ```
 */
export function seedHitByRegion(
  {
    spanStart,
    spanEnd,
    application,
  }: {
    readonly spanStart: number;
    readonly spanEnd: number;
    readonly application: SeededErrorApplication;
  },
): boolean {
  /**
   Seed region expanded by tolerance on both sides.
   */
  const regionStart = application.startOffset - SEED_MATCH_TOLERANCE;

  /**
   Expanded region end.
   */
  const regionEnd = application.endOffset + SEED_MATCH_TOLERANCE;

  return (spanStart < regionEnd) && (regionStart < spanEnd);
}

//endregion Seeded errors

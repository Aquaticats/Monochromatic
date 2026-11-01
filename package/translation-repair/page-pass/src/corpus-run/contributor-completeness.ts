import { droppedContributorNameForms, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Contributor completeness

/**
 Failure raised when final page changes target-authoritative contributor name.

 @example
 ```ts
 throw new ContributorCompletenessError({ entryId: 'CatEntry', droppedCount: 1, });
 ```
 */
export class ContributorCompletenessError extends Error {
  /**
   Declares message safe to forward because it names entry and count only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose contributor authority failed.
   */
  readonly entryId: string;

  /**
   Number of target contributor forms final page lost.
   */
  readonly droppedCount: number;

  /**
   @param entryId - entry refused

   @param droppedCount - target contributor forms final page lost
   */
  public constructor(
    {
      entryId,
      droppedCount,
    }: {
      readonly entryId: string;
      readonly droppedCount: number;
    },
  ) {
    super(`entry ${entryId} changed ${String(droppedCount,)} target-authoritative contributor names`,);
    this.name = 'ContributorCompletenessError';
    this.entryId = entryId;
    this.droppedCount = droppedCount;
  }
}

/**
 Refuses final page that drops or respells contributor identity established by
 existing English archive attribution line.

 @param entryId - corpus entry being published

 @param archiveText - complete existing English archive page

 @param pageText - assembled final page

 @throws {@link ContributorCompletenessError} when target contributor identity changed

 @example
 ```ts
 assertContributorNamesComplete({ entryId: 'CatEntry', archiveText, pageText, });
 ```
 */
export function assertContributorNamesComplete(
  {
    entryId,
    archiveText,
    pageText,
  }: {
    readonly entryId: string;
    readonly archiveText: string;
    readonly pageText: string;
  },
): void {
  /**
   Established contributor forms final attribution no longer names whole.
   */
  const dropped = droppedContributorNameForms({
    archiveText,
    candidateText: pageText,
  },);
  if (dropped.length === 0)
    return;
  throw new ContributorCompletenessError({
    entryId,
    droppedCount: dropped.length,
  },);
}

//endregion Contributor completeness

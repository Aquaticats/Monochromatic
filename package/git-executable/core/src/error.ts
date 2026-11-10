import { SELF_SHIM_INSPECTION_KIBIBYTES, } from './self-shim.ts';

//region Failure evidence

/**
 Evidence describing every candidate the resolver examined and rejected.
 */
export type RealGitNotFoundEvidence = {
  /**
   PATH-derived candidates examined before failure.
   */
  readonly candidateCount: number;
  /**
   Self-referential wrappers rejected during examination.
   */
  readonly skippedSelfShimCount: number;
  /**
   Candidates whose opened filesystem object is not a regular file.
   */
  readonly skippedNonRegularCandidatePaths: readonly string[];
  /**
   Non-native candidates larger than the script inspection bound.
   */
  readonly skippedOversizedCandidatePaths: readonly string[];
};

/**
 Builds failure text naming every rejected candidate whose reason PATH alone cannot show.
 
 The generic install hint is dropped once a rejection names its own remediation,
 because that rejection contradicts a missing-installation reading.
 
 @param evidence - Examined and rejected candidate counts and paths.
 
 @returns Message naming the counts, the rejected candidate paths, and each remediation.
 
 @example
 ```ts
 resolutionFailureMessage({
   candidateCount: 1,
   skippedSelfShimCount: 0,
   skippedNonRegularCandidatePaths: ['/fixture/bin/git'],
   skippedOversizedCandidatePaths: [],
 });
 ```
 */
function resolutionFailureMessage({
  candidateCount,
  skippedSelfShimCount,
  skippedNonRegularCandidatePaths,
  skippedOversizedCandidatePaths,
}: RealGitNotFoundEvidence,): string {
  /**
   Sentences naming rejections whose reason PATH alone cannot show.
   */
  const skipSentences: string[] = [];
  if (skippedNonRegularCandidatePaths.length > 0) {
    skipSentences.push(
      `Candidates rejected because they are not regular files: ${skippedNonRegularCandidatePaths.join(', ',)}. `
        + 'PATH entries must name a regular file, or Git must be installed as a native executable.',
    );
  }
  if (skippedOversizedCandidatePaths.length > 0) {
    skipSentences.push(
      `Candidates rejected because they are non-native scripts larger than the `
        + `${String(SELF_SHIM_INSPECTION_KIBIBYTES,)} KiB inspection bound: `
        + `${skippedOversizedCandidatePaths.join(', ',)}. `
        + 'Install native Git, or shrink each wrapper script below that bound.',
    );
  }
  /**
   Generic installation hint, which only applies when no rejection explains itself.
   */
  const installHint = skipSentences.length === 0
    ? ['Ensure Git is installed and PATH/PATHEXT expose its executable.',]
    : [];

  return [
    `Could not find a real Git executable after examining ${String(candidateCount,)} PATH candidates `
      + `and skipping ${String(skippedSelfShimCount,)} self-referential wrappers.`,
    ...skipSentences,
    ...installHint,
  ].join(' ',);
}

//endregion Failure evidence

//region Failure

/**
 Reports that no PATH-exposed candidate can act as real Git.
 
 @example
 ```ts
 throw new RealGitNotFoundError({
   candidateCount: 4,
   skippedSelfShimCount: 1,
   skippedNonRegularCandidatePaths: [],
   skippedOversizedCandidatePaths: [],
 });
 ```
 */
export class RealGitNotFoundError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'RealGitNotFoundError';

  /**
   Creates resolution failure with attempted-candidate evidence.
   
   @param evidence - Examined and rejected candidate counts and paths.
   
   @example
   ```ts
   const error = new RealGitNotFoundError({
     candidateCount: 4,
     skippedSelfShimCount: 1,
     skippedNonRegularCandidatePaths: ['/fixture/bin/git'],
     skippedOversizedCandidatePaths: [],
   });
   ```
   */
  constructor(evidence: RealGitNotFoundEvidence,) {
    super(resolutionFailureMessage(evidence,),);
  }
}

//endregion Failure
